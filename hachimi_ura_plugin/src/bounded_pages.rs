//! Small JSON pages shared by immutable storage indices and mutable observer caches.
//! Copy a bounded cache slice under its lock, then call `encode` after releasing it.
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::io::{self, Write};
pub const MAX_RECORDS: usize = 64;
pub const MAX_BYTES: usize = 256 * 1024;
const RESERVE: usize = 2048;

pub fn parameter<'a>(pairs: &'a [(String, String)], name: &str) -> Result<Option<&'a str>, String> {
    let mut found = pairs.iter().filter(|(key, _)| key == name);
    let value = found.next().map(|(_, value)| value.as_str());
    if found.next().is_some() {
        return Err(format!("duplicate_{}", name));
    }
    Ok(value)
}
pub fn unsigned(pairs: &[(String, String)], name: &str, default: u64) -> Result<u64, String> {
    match parameter(pairs, name)? {
        None => Ok(default),
        Some(value) if !value.is_empty() && value.bytes().all(|c| c.is_ascii_digit()) => {
            value.parse().map_err(|_| format!("invalid_{}", name))
        }
        _ => Err(format!("invalid_{}", name)),
    }
}
#[derive(Clone, Copy)]
pub struct Limits {
    pub records: usize,
    pub bytes: usize,
}
impl Limits {
    pub fn parse(pairs: &[(String, String)]) -> Result<Self, String> {
        if parameter(pairs, "cursor")?.is_some() || parameter(pairs, "max_json_chars")?.is_some() {
            return Err("legacy_paging_parameter_use_explicit_after_cursor_and_page_bytes".into());
        }
        let count = unsigned(pairs, "limit", MAX_RECORDS as u64)?;
        let bytes = unsigned(pairs, "page_bytes", MAX_BYTES as u64)?;
        if !(1..=MAX_RECORDS as u64).contains(&count) || !(4096..=MAX_BYTES as u64).contains(&bytes)
        {
            return Err("invalid_page_budget".into());
        }
        Ok(Self {
            records: count as usize,
            bytes: bytes as usize,
        })
    }
}
struct Writer {
    bytes: Vec<u8>,
    limit: usize,
}
impl Write for Writer {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if bytes.len() > self.limit.saturating_sub(self.bytes.len()) {
            return Err(io::Error::other("page_byte_budget"));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
fn bounded(value: &Value, max: usize) -> Result<Vec<u8>, String> {
    let mut writer = Writer {
        bytes: Vec::new(),
        limit: max,
    };
    serde_json::to_writer(&mut writer, value)
        .map_err(|_| "record_exceeds_page_byte_budget".to_string())?;
    Ok(writer.bytes)
}
/// Advance the cursor only after a whole row has been returned. The lookahead is
/// metadata only; callers fetch at most MAX_RECORDS + 1 indexed rows.
pub fn encode(
    mut meta: Value,
    array: &str,
    cursor_field: &str,
    after: Value,
    rows: impl IntoIterator<Item = (Value, Value)>,
    source_has_more: bool,
    limits: Limits,
) -> Result<String, String> {
    if !(1..=MAX_RECORDS).contains(&limits.records) || !(4096..=MAX_BYTES).contains(&limits.bytes) {
        return Err("invalid_page_budget".into());
    }
    let mut records = Vec::new();
    let mut next = after.clone();
    let mut used = 0;
    let mut more = source_has_more;
    for (cursor, row) in rows.into_iter().take(MAX_RECORDS + 1) {
        if records.len() == limits.records {
            more = true;
            break;
        }
        let available = limits.bytes - RESERVE - used;
        let bytes = match bounded(&row, available) {
            Ok(bytes) if bytes.len() + 1 <= available => bytes,
            _ if records.is_empty() => return Err("record_exceeds_page_byte_budget".into()),
            _ => {
                more = true;
                break;
            }
        };
        used += bytes.len() + 1;
        records.push(bytes);
        next = cursor;
    }
    meta["ok"] = json!(true);
    meta["schema_version"] = json!(2);
    meta["cursor_version"] = json!(2);
    meta[cursor_field] = after;
    meta[format!("next_{}", cursor_field)] = next;
    meta["count"] = json!(records.len());
    meta["has_more"] = json!(more);
    meta["page_bytes"] = json!(limits.bytes);
    let mut result =
        String::from_utf8(bounded(&meta, RESERVE - 32)?).map_err(|_| "invalid_json_utf8")?;
    result.pop();
    result.push(',');
    result.push_str(&serde_json::to_string(array).map_err(|_| "invalid_array_name")?);
    result.push_str(":[");
    for (i, bytes) in records.iter().enumerate() {
        if i != 0 {
            result.push(',');
        }
        result.push_str(std::str::from_utf8(bytes).map_err(|_| "invalid_json_utf8")?);
    }
    result.push_str("]}");
    if result.len() > limits.bytes {
        return Err("page_byte_budget".into());
    }
    Ok(result)
}

pub struct CacheRequest {
    pub limits: Limits,
    after: usize,
    window: Option<String>,
}
impl CacheRequest {
    pub fn parse(pairs: &[(String, String)]) -> Result<Self, String> {
        let limits = Limits::parse(pairs)?;
        if parameter(pairs, "after_id")?.is_some() {
            return Err("legacy_cache_cursor_use_cache_window_id_and_after_index".into());
        }
        let after = usize::try_from(unsigned(pairs, "after_index", 0)?)
            .map_err(|_| "invalid_after_index")?;
        let window = parameter(pairs, "cache_window_id")?.map(str::to_string);
        if after > 0 && window.is_none() {
            return Err("cache_window_id_required_for_continuation".into());
        }
        if window
            .as_ref()
            .is_some_and(|value| value.len() != 64 || !value.bytes().all(|c| c.is_ascii_hexdigit()))
        {
            return Err("invalid_cache_window_id".into());
        }
        Ok(Self {
            limits,
            after,
            window,
        })
    }
}
#[derive(Clone, Copy)]
pub enum CacheRef<'a> {
    Pair(&'a str, &'a str),
    Json(&'a str),
}
enum Owned {
    Pair(String, String),
    Json(String),
}
pub struct CacheSlice {
    request: CacheRequest,
    window: String,
    items: Vec<(usize, Owned)>,
    more: bool,
}
/// No formatting/JSON parsing while holding the producer's cache lock. Hashing
/// borrows the capped ring's bytes; only a page-budgeted subset is cloned.
pub fn copy_cache<'a, I>(rows: I, request: CacheRequest) -> Result<CacheSlice, String>
where
    I: Iterator<Item = CacheRef<'a>> + Clone,
{
    let mut hash = Sha256::new();
    let mut count = 0;
    for row in rows.clone() {
        count += 1;
        let (kind, first, second) = match row {
            CacheRef::Pair(a, b) => (b'p', a, Some(b)),
            CacheRef::Json(a) => (b'j', a, None),
        };
        hash.update([kind]);
        for value in [Some(first), second].into_iter().flatten() {
            hash.update((value.len() as u64).to_le_bytes());
            hash.update(value.as_bytes());
        }
    }
    let window = format!("{:x}", hash.finalize());
    if request
        .window
        .as_ref()
        .is_some_and(|wanted| wanted != &window)
    {
        return Err("stale_cache_window_restart_from_after_index_0".into());
    }
    if request.after > count {
        return Err("after_index_outside_cache_window".into());
    }
    let mut items = Vec::new();
    let mut copied = 0usize;
    for (index, row) in rows.enumerate().skip(request.after) {
        if items.len() == request.limits.records {
            break;
        }
        let size = match row {
            CacheRef::Pair(a, b) => a
                .len()
                .checked_add(b.len())
                .ok_or("cache_record_size_overflow")?,
            CacheRef::Json(value) => value.len(),
        };
        if size > request.limits.bytes - copied {
            if items.is_empty() {
                return Err(format!("oversized_cached_record_at_index_{}", index));
            }
            break;
        }
        let owned = match row {
            CacheRef::Pair(a, b) => Owned::Pair(a.into(), b.into()),
            CacheRef::Json(value) => Owned::Json(value.into()),
        };
        copied += size;
        items.push((index + 1, owned));
    }
    let more = request.after + items.len() < count;
    Ok(CacheSlice {
        request,
        window,
        items,
        more,
    })
}
impl CacheSlice {
    /// Call outside the cache mutex. Invalid legacy JSON is an explicit observation
    /// error; it never masquerades as a parsed game event or a complete raw file.
    pub fn encode(self, array: &str) -> Result<String, String> {
        let items=self.items.into_iter().map(|(next,value)|{
            let record=match value {
                Owned::Pair(input,output)=>json!({"cache_index":next-1,"input":input,"output":output}),
                Owned::Json(text)=>serde_json::from_str::<Value>(&text).unwrap_or_else(|_|json!({
                    "cache_index":next-1,"representation":"invalid_cached_record","error":"invalid_json","byte_length":text.len()})),
            };(json!(next),record)
        });
        encode(
            json!({"cache_window_id":self.window,"cursor_scope":"immutable_cache_window"}),
            array,
            "after_index",
            json!(self.request.after),
            items,
            self.more,
            self.request.limits,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn request() -> CacheRequest {
        CacheRequest::parse(&[]).unwrap()
    }
    #[test]
    fn utf8_control_characters_are_complete_valid_json() {
        let data = vec![("中\0\n\t\"".to_string(), "文\\".to_string())];
        let page = copy_cache(data.iter().map(|(a, b)| CacheRef::Pair(a, b)), request()).unwrap();
        let result: Value = serde_json::from_str(&page.encode("entries").unwrap()).unwrap();
        assert_eq!(result["entries"][0]["input"], data[0].0);
    }
    #[test]
    fn cache_continuation_rejects_changed_window_instead_of_reusing_shifted_indices() {
        let mut entries = vec!["{}".to_string(), "{\"a\":1}".to_string()];
        let mut req = request();
        req.limits.records = 1;
        let page = copy_cache(entries.iter().map(|s| CacheRef::Json(s)), req)
            .unwrap()
            .encode("observations")
            .unwrap();
        let first: Value = serde_json::from_str(&page).unwrap();
        let pairs = vec![
            (
                "cache_window_id".into(),
                first["cache_window_id"].as_str().unwrap().into(),
            ),
            ("after_index".into(), "1".into()),
        ];
        entries[0] = "{\"changed\":true}".into();
        assert!(copy_cache(
            entries.iter().map(|s| CacheRef::Json(s)),
            CacheRequest::parse(&pairs).unwrap()
        )
        .err()
        .unwrap()
        .contains("stale_cache"));
    }
    #[test]
    fn immutable_copy_survives_producer_clear_without_holding_lock() {
        let mut entries = vec!["{\"id\":7}".to_string()];
        let snapshot = copy_cache(entries.iter().map(|s| CacheRef::Json(s)), request()).unwrap();
        entries.clear();
        let result: Value =
            serde_json::from_str(&snapshot.encode("observations").unwrap()).unwrap();
        assert_eq!(result["observations"][0]["id"], 7);
    }
    #[test]
    fn byte_page_does_not_advance_past_unemitted_index() {
        let limits = Limits {
            records: 64,
            bytes: 4096,
        };
        let values: Vec<_> = (1..=10)
            .map(|n| (json!(n), json!({"value":"中".repeat(300)})))
            .collect();
        let page = encode(
            json!({}),
            "files",
            "after_file_id",
            json!(0),
            values.clone(),
            false,
            limits,
        )
        .unwrap();
        assert!(page.len() <= 4096);
        let first: Value = serde_json::from_str(&page).unwrap();
        let last = first["next_after_file_id"].as_u64().unwrap();
        assert_eq!(last, 2);
        assert_eq!(first["has_more"], true);
        let second: Value = serde_json::from_str(
            &encode(
                json!({}),
                "files",
                "after_file_id",
                json!(last),
                values
                    .into_iter()
                    .filter(|(id, _)| id.as_u64().unwrap() > last),
                false,
                limits,
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(second["next_after_file_id"], 4);
    }
    #[test]
    fn oversized_copy_and_serialization_are_explicit_errors() {
        let text = "a".repeat(MAX_BYTES + 1);
        assert!(
            copy_cache(std::iter::once(CacheRef::Json(&text)), request())
                .err()
                .unwrap()
                .contains("oversized_cached_record")
        );
        assert!(encode(
            json!({}),
            "entries",
            "after_index",
            json!(0),
            vec![(json!(1), json!({"text":"\n".repeat(MAX_BYTES)}))],
            false,
            Limits {
                records: 64,
                bytes: MAX_BYTES
            }
        )
        .is_err());
    }
    #[test]
    fn invalid_legacy_json_is_not_repaired_into_a_fake_event() {
        let result: Value = serde_json::from_str(
            &copy_cache(std::iter::once(CacheRef::Json("{bad\0}")), request())
                .unwrap()
                .encode("observations")
                .unwrap(),
        )
        .unwrap();
        assert_eq!(
            result["observations"][0]["representation"],
            "invalid_cached_record"
        );
        assert_eq!(result["observations"][0]["byte_length"], 6);
    }
    #[test]
    fn empty_and_final_pages_have_precise_cursors() {
        let result: Value = serde_json::from_str(
            &encode(
                json!({}),
                "sessions",
                "after_session_id",
                json!("old"),
                Vec::new(),
                false,
                Limits {
                    records: 64,
                    bytes: MAX_BYTES,
                },
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(result["next_after_session_id"], "old");
        assert_eq!(result["has_more"], false);
    }
    #[test]
    fn malformed_limits_and_legacy_cursors_are_not_silently_clamped() {
        for (key, value) in [
            ("limit", "1000"),
            ("page_bytes", "9999999"),
            ("cursor", "1"),
            ("max_json_chars", "1"),
            ("after_id", "1"),
        ] {
            assert!(CacheRequest::parse(&[(key.into(), value.into())]).is_err());
        }
        assert!(CacheRequest::parse(&[("after_index".into(), "1".into())]).is_err());
    }
}
