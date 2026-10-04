//! Bounded derived views of indexed raw captures. Original files are never changed.
//! The caller queries only indexed response payloads after `after_file_id`, ordered
//! by file_id, LIMIT MAX_SCANNED_FILES + 1. No matched-record ordinal is a V2 cursor.
use serde_json::{json, Value};
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::{Component, Path};

pub const MAX_PAGE_RECORDS: usize = 64;
pub const MAX_SCANNED_FILES: usize = 64;
pub const MAX_PAGE_BYTES: usize = 256 * 1024;
pub const MAX_FILE_DECODE_BYTES: usize = 1024 * 1024;
pub const MAX_PAGE_DECODE_BYTES: usize = 4 * 1024 * 1024;
const ENVELOPE_RESERVE: usize = 2048;

#[derive(Debug, Clone)]
pub struct Request {
    pub session_id: String,
    pub after_file_id: i64,
    pub limit: usize,
    pub page_bytes: usize,
    pub decode_bytes: usize,
}

impl Request {
    pub fn parse(pairs: &[(String, String)]) -> Result<Self, String> {
        let get = |name: &str| -> Result<Option<&str>, String> {
            let mut values = pairs.iter().filter(|(key, _)| key == name);
            let value = values.next().map(|(_, value)| value.as_str());
            if values.next().is_some() {
                return Err(format!("duplicate_{}", name));
            }
            Ok(value)
        };
        if get("after_sequence")?.is_some() || get("cursor")?.is_some() {
            return Err(
                "legacy_cursor_not_supported_use_cursor_version_2_and_after_file_id".into(),
            );
        }
        if get("max_json_chars")?.is_some() {
            return Err(
                "legacy_max_json_chars_not_supported_use_page_bytes_and_decode_bytes".into(),
            );
        }
        if get("cursor_version")? != Some("2") {
            return Err("cursor_version_2_required".into());
        }
        let number = |name: &str, default: usize| -> Result<usize, String> {
            match get(name)? {
                None => Ok(default),
                Some(value)
                    if !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit()) =>
                {
                    value.parse().map_err(|_| format!("invalid_{}", name))
                }
                _ => Err(format!("invalid_{}", name)),
            }
        };
        let cursor = get("after_file_id")?.unwrap_or("0");
        if cursor.is_empty() || !cursor.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err("invalid_after_file_id".into());
        }
        let value = Self {
            session_id: get("session_id")?.unwrap_or("").to_owned(),
            after_file_id: cursor.parse().map_err(|_| "invalid_after_file_id")?,
            limit: number("limit", MAX_PAGE_RECORDS)?,
            page_bytes: number("page_bytes", MAX_PAGE_BYTES)?,
            decode_bytes: number("decode_bytes", MAX_FILE_DECODE_BYTES)?,
        };
        value.validate()?;
        Ok(value)
    }

    fn validate(&self) -> Result<(), String> {
        if self.session_id.is_empty()
            || self.session_id.len() > 128
            || self.session_id == "."
            || self.session_id == ".."
            || !self
                .session_id
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || b"._-".contains(&c))
        {
            return Err("invalid_session_id".into());
        }
        if self.after_file_id < 0 {
            return Err("invalid_after_file_id".into());
        }
        if !(1..=MAX_PAGE_RECORDS).contains(&self.limit) {
            return Err("invalid_limit".into());
        }
        if !(4096..=MAX_PAGE_BYTES).contains(&self.page_bytes) {
            return Err("invalid_page_bytes".into());
        }
        if !(1..=MAX_FILE_DECODE_BYTES).contains(&self.decode_bytes) {
            return Err("invalid_decode_bytes".into());
        }
        Ok(())
    }
}

/// Construct only from a row in observation_files, never directly from query paths.
#[derive(Debug, Clone)]
pub struct IndexedFile {
    pub file_id: i64,
    pub session_id: String,
    pub relative_path: String,
    pub byte_length: u64,
    pub sha256: Option<String>,
    pub created_at_ms: i64,
}

fn raw_reference(row: &IndexedFile, reason: &str) -> Value {
    json!({"representation":"raw_file_reference", "reason":reason,
        "source_file_id":row.file_id, "file_id":row.file_id, "session_id":row.session_id,
        "byte_length":row.byte_length,"sha256":row.sha256,"checksum_source":"storage_index",
        "response_timestamp_ms":row.created_at_ms,
        "download_url":format!("/storage/read_range?file_id={}&offset=0&length={}",row.file_id,row.byte_length.clamp(1,MAX_FILE_DECODE_BYTES as u64)),
        "range_byte_limit":MAX_FILE_DECODE_BYTES})
}

struct LimitedWriter {
    bytes: Vec<u8>,
    limit: usize,
}
impl Write for LimitedWriter {
    fn write(&mut self, value: &[u8]) -> io::Result<usize> {
        if value.len() > self.limit.saturating_sub(self.bytes.len()) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "derived_json_byte_budget",
            ));
        }
        self.bytes.extend_from_slice(value);
        Ok(value.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
fn encode_bounded(value: &Value, limit: usize) -> Result<Vec<u8>, ()> {
    let mut output = LimitedWriter {
        bytes: Vec::new(),
        limit,
    };
    serde_json::to_writer(&mut output, value).map_err(|_| ())?;
    Ok(output.bytes)
}

pub const MAX_MSGPACK_DEPTH: usize = 32;
pub const MAX_MSGPACK_NODES: usize = 32 * 1024;

enum Token<'a> {
    Nil,
    Bool(bool),
    Unsigned(u64),
    Signed(i64),
    Float(f64),
    Text(&'a str),
    Binary(&'a [u8]),
    Ext(i8, &'a [u8]),
    Array(usize),
    Map(usize),
}
struct Msgpack<'a> {
    bytes: &'a [u8],
    position: usize,
    nodes: usize,
}
impl<'a> Msgpack<'a> {
    fn new(bytes: &'a [u8]) -> Result<Self, String> {
        if bytes.len() > MAX_FILE_DECODE_BYTES {
            return Err("msgpack_input_budget".into());
        }
        Ok(Self {
            bytes,
            position: 0,
            nodes: 0,
        })
    }
    fn take(&mut self, n: usize) -> Result<&'a [u8], String> {
        let end = self
            .position
            .checked_add(n)
            .filter(|end| *end <= self.bytes.len())
            .ok_or("truncated_msgpack")?;
        let value = &self.bytes[self.position..end];
        self.position = end;
        Ok(value)
    }
    fn unsigned(&mut self, n: usize) -> Result<u64, String> {
        Ok(self
            .take(n)?
            .iter()
            .fold(0u64, |value, byte| (value << 8) | u64::from(*byte)))
    }
    fn token(&mut self, depth: usize) -> Result<Token<'a>, String> {
        if depth > MAX_MSGPACK_DEPTH {
            return Err("msgpack_depth_budget".into());
        }
        self.nodes += 1;
        if self.nodes > MAX_MSGPACK_NODES {
            return Err("msgpack_node_budget".into());
        }
        let tag = self.unsigned(1)? as u8;
        let token = match tag {
            0x00..=0x7f => Token::Unsigned(u64::from(tag)),
            0xe0..=0xff => Token::Signed(i64::from(tag as i8)),
            0xc0 => Token::Nil,
            0xc2 => Token::Bool(false),
            0xc3 => Token::Bool(true),
            0xcc..=0xcf => Token::Unsigned(self.unsigned(1usize << (tag - 0xcc))?),
            0xd0..=0xd3 => {
                let n = 1usize << (tag - 0xd0);
                let value = self.unsigned(n)?;
                Token::Signed(((value << (64 - n * 8)) as i64) >> (64 - n * 8))
            }
            0xca => Token::Float(f32::from_bits(self.unsigned(4)? as u32) as f64),
            0xcb => Token::Float(f64::from_bits(self.unsigned(8)?)),
            0xa0..=0xbf | 0xd9..=0xdb => {
                let len = if tag <= 0xbf {
                    usize::from(tag & 31)
                } else {
                    self.unsigned(1usize << (tag - 0xd9))? as usize
                };
                Token::Text(
                    std::str::from_utf8(self.take(len)?).map_err(|_| "invalid_msgpack_utf8")?,
                )
            }
            0xc4..=0xc6 => {
                let len = self.unsigned(1usize << (tag - 0xc4))? as usize;
                Token::Binary(self.take(len)?)
            }
            0xc7..=0xc9 | 0xd4..=0xd8 => {
                let len = if tag >= 0xd4 {
                    1usize << (tag - 0xd4)
                } else {
                    self.unsigned(1usize << (tag - 0xc7))? as usize
                };
                let kind = self.unsigned(1)? as i8;
                Token::Ext(kind, self.take(len)?)
            }
            0x90..=0x9f | 0xdc..=0xdd => {
                let len = if tag <= 0x9f {
                    usize::from(tag & 15)
                } else {
                    self.unsigned(if tag == 0xdc { 2 } else { 4 })? as usize
                };
                self.children(len)?;
                Token::Array(len)
            }
            0x80..=0x8f | 0xde..=0xdf => {
                let len = if tag <= 0x8f {
                    usize::from(tag & 15)
                } else {
                    self.unsigned(if tag == 0xde { 2 } else { 4 })? as usize
                };
                self.children(len.checked_mul(2).ok_or("msgpack_node_budget")?)?;
                Token::Map(len)
            }
            _ => return Err("invalid_msgpack_marker".into()),
        };
        Ok(token)
    }
    fn children(&self, count: usize) -> Result<(), String> {
        if count > MAX_MSGPACK_NODES - self.nodes {
            return Err("msgpack_node_budget".into());
        }
        if count > self.bytes.len() - self.position {
            return Err("truncated_msgpack".into());
        }
        Ok(())
    }
    fn skip(&mut self, depth: usize) -> Result<(), String> {
        let count = match self.token(depth)? {
            Token::Array(n) => n,
            Token::Map(n) => n * 2,
            _ => 0,
        };
        for _ in 0..count {
            self.skip(depth + 1)?;
        }
        Ok(())
    }
    fn span(&mut self, depth: usize) -> Result<&'a [u8], String> {
        let start = self.position;
        self.skip(depth)?;
        Ok(&self.bytes[start..self.position])
    }
}

fn text_key(bytes: &[u8]) -> Option<&str> {
    match Msgpack::new(bytes).ok()?.token(0).ok()? {
        Token::Text(value) => Some(value),
        _ => None,
    }
}

fn write_msgpack_json(
    reader: &mut Msgpack<'_>,
    depth: usize,
    out: &mut LimitedWriter,
) -> Result<(), String> {
    let io_error = |_: io::Error| "msgpack_output_budget".to_string();
    let json_error = |_: serde_json::Error| "msgpack_output_budget".to_string();
    match reader.token(depth)? {
        Token::Nil => out.write_all(b"null").map_err(io_error)?,
        Token::Bool(value) => serde_json::to_writer(out, &value).map_err(json_error)?,
        Token::Unsigned(value) => serde_json::to_writer(out, &value).map_err(json_error)?,
        Token::Signed(value) => serde_json::to_writer(out, &value).map_err(json_error)?,
        Token::Float(value) => serde_json::to_writer(out, &value).map_err(json_error)?,
        Token::Text(value) => serde_json::to_writer(out, value).map_err(json_error)?,
        Token::Binary(value) => write_hex_object(out, None, value)?,
        Token::Ext(kind, value) => write_hex_object(out, Some(kind), value)?,
        Token::Array(length) => {
            out.write_all(b"[").map_err(io_error)?;
            for i in 0..length {
                if i != 0 {
                    out.write_all(b",").map_err(io_error)?;
                }
                write_msgpack_json(reader, depth + 1, out)?;
            }
            out.write_all(b"]").map_err(io_error)?;
        }
        Token::Map(length) => {
            out.write_all(b"{").map_err(io_error)?;
            for i in 0..length {
                if i != 0 {
                    out.write_all(b",").map_err(io_error)?;
                }
                let key = match reader.token(depth + 1)? {
                    Token::Text(value) => value,
                    _ => return Err("non_text_msgpack_map_key".into()),
                };
                serde_json::to_writer(&mut *out, key).map_err(json_error)?;
                out.write_all(b":").map_err(io_error)?;
                write_msgpack_json(reader, depth + 1, out)?;
            }
            out.write_all(b"}").map_err(io_error)?;
        }
    };
    Ok(())
}
fn write_hex_object(out: &mut LimitedWriter, kind: Option<i8>, bytes: &[u8]) -> Result<(), String> {
    let io_error = |_: io::Error| "msgpack_output_budget".to_string();
    match kind {
        None => out
            .write_all(b"{\"messagepack_type\":\"binary\",\"body_hex\":\"")
            .map_err(io_error)?,
        Some(kind) => {
            out.write_all(b"{\"messagepack_type\":\"ext\",\"ext_type\":")
                .map_err(io_error)?;
            serde_json::to_writer(&mut *out, &kind).map_err(|_| "msgpack_output_budget")?;
            out.write_all(b",\"body_hex\":\"").map_err(io_error)?;
        }
    }
    const HEX: &[u8] = b"0123456789abcdef";
    for byte in bytes {
        out.write_all(&[HEX[usize::from(byte >> 4)], HEX[usize::from(byte & 15)]])
            .map_err(io_error)?;
    }
    out.write_all(b"\"}").map_err(io_error)?;
    Ok(())
}

/// Decode production MessagePack without allocating from declared string/container
/// lengths. Scan the complete structure first, then serialize only selected fields
/// through a byte-limited writer. JSON Value construction happens after that limit.
pub fn decode_turn_event(bytes: &[u8], max_json_bytes: usize) -> Result<Option<Value>, String> {
    if max_json_bytes > MAX_PAGE_BYTES {
        return Err("msgpack_output_budget".into());
    }
    let mut checked = Msgpack::new(bytes)?;
    checked.skip(0)?;
    if checked.position != bytes.len() {
        return Err("trailing_msgpack_bytes".into());
    }
    let mut root = Msgpack::new(bytes)?;
    let root_count = match root.token(0)? {
        Token::Map(count) => count,
        _ => return Ok(None),
    };
    let mut data = None;
    for _ in 0..root_count {
        let key = root.span(1)?;
        let value = root.span(1)?;
        if data.is_none() && text_key(key) == Some("data") {
            data = Some(value);
        }
    }
    let Some(data) = data else { return Ok(None) };
    let mut reader = Msgpack::new(data)?;
    let count = match reader.token(0)? {
        Token::Map(count) => count,
        _ => return Ok(None),
    };
    let mut selected = Vec::new();
    let mut has_chara = false;
    let mut has_data_set = false;
    for _ in 0..count {
        let key = reader.span(1)?;
        let value = reader.span(1)?;
        if let Some(name) = text_key(key) {
            has_chara |= name == "chara_info";
            has_data_set |= name.ends_with("_data_set");
            if name == "chara_info"
                || name == "unchecked_event_array"
                || name.ends_with("_data_set")
            {
                selected.push((name, value));
            }
        }
    }
    if !has_chara || !has_data_set {
        return Ok(None);
    };
    let mut output = LimitedWriter {
        bytes: Vec::new(),
        limit: max_json_bytes,
    };
    let io_error = |_: io::Error| "msgpack_output_budget".to_string();
    output.write_all(b"{").map_err(io_error)?;
    for (index, (key, value)) in selected.iter().enumerate() {
        if index != 0 {
            output.write_all(b",").map_err(io_error)?;
        }
        serde_json::to_writer(&mut output, key).map_err(|_| "msgpack_output_budget")?;
        output.write_all(b":").map_err(io_error)?;
        write_msgpack_json(&mut Msgpack::new(value)?, 0, &mut output)?;
    }
    output.write_all(b"}").map_err(io_error)?;
    serde_json::from_slice(&output.bytes)
        .map(Some)
        .map_err(|_| "derived_json_parse_failed".into())
}

fn read_indexed(root: &Path, row: &IndexedFile, limit: usize) -> Result<Vec<u8>, &'static str> {
    if row.relative_path.is_empty()
        || row.relative_path.len() > 1024
        || row.relative_path.contains('\\')
        || row.relative_path.contains(':')
        || Path::new(&row.relative_path)
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err("invalid_indexed_path");
    }
    let path = root
        .join(&row.relative_path)
        .canonicalize()
        .map_err(|_| "indexed_file_unavailable")?;
    if !path.starts_with(root) {
        return Err("indexed_path_outside_session");
    }
    let file = File::open(path).map_err(|_| "indexed_file_unavailable")?;
    let metadata = file.metadata().map_err(|_| "file_metadata_unavailable")?;
    if !metadata.is_file() {
        return Err("indexed_path_not_file");
    }
    if metadata.len() != row.byte_length {
        return Err("indexed_file_length_changed");
    }
    if metadata.len() > limit as u64 {
        return Err("file_decode_byte_budget");
    }
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    file.take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "indexed_file_read_failed")?;
    if bytes.len() as u64 != row.byte_length {
        return Err("indexed_file_length_changed");
    }
    if bytes.len() > limit {
        return Err("file_decode_byte_budget");
    }
    Ok(bytes)
}

/// Decode is injected to retain the existing MessagePack field selection. It must
/// bound recursion/expansion itself; `max_json_bytes` is its output allowance.
/// This helper bounds input IO and serialized JSON, not a decoder's arbitrary heap.
/// `None` means a valid response that contains no selected turn/event fields.
pub fn export_page<I, D>(
    session_root: &Path,
    request: &Request,
    rows: I,
    mut decode: D,
) -> Result<String, String>
where
    I: IntoIterator<Item = IndexedFile>,
    D: FnMut(&[u8], usize) -> Result<Option<Value>, String>,
{
    request.validate()?;
    let root = session_root
        .canonicalize()
        .map_err(|_| "session_directory_unavailable")?;
    if !root.is_dir() {
        return Err("session_directory_unavailable".into());
    }
    let mut records: Vec<Vec<u8>> = Vec::new();
    let mut emitted_bytes = 0usize;
    let mut decoded_bytes = 0usize;
    let mut scanned = 0usize;
    let mut next = request.after_file_id;
    let mut previous = 0i64;
    let mut has_more = false;
    let mut stop_reason = "end_of_index";
    // The extra indexed row only detects another page; it is never opened/decoded.
    for (index, row) in rows.into_iter().take(MAX_SCANNED_FILES + 1).enumerate() {
        if row.file_id <= previous || row.file_id <= 0 {
            return Err("index_not_strictly_ordered_by_file_id".into());
        }
        previous = row.file_id;
        if row.session_id != request.session_id {
            return Err("indexed_session_mismatch".into());
        }
        if row
            .sha256
            .as_ref()
            .is_some_and(|hash| hash.len() != 64 || !hash.bytes().all(|c| c.is_ascii_hexdigit()))
        {
            return Err("invalid_indexed_checksum".into());
        }
        // Apply source cursor before filesystem metadata, reads, and MessagePack decode.
        if row.file_id <= request.after_file_id {
            continue;
        }
        if index == MAX_SCANNED_FILES
            || scanned == MAX_SCANNED_FILES
            || records.len() == request.limit
        {
            has_more = true;
            stop_reason = "record_or_scan_budget";
            break;
        }
        scanned += 1;
        let available = request.page_bytes - ENVELOPE_RESERVE - emitted_bytes;
        let reference = if row.byte_length > request.decode_bytes as u64 {
            Some(raw_reference(&row, "file_decode_byte_budget"))
        } else if row.byte_length > (MAX_PAGE_DECODE_BYTES - decoded_bytes) as u64 {
            Some(raw_reference(&row, "page_decode_byte_budget"))
        } else {
            match read_indexed(&root, &row, request.decode_bytes) {
                Err(reason) => Some(raw_reference(&row, reason)),
                Ok(bytes) => {
                    decoded_bytes += bytes.len();
                    match decode(&bytes, available) {
                        Ok(Some(data)) => Some(
                            json!({"representation":"derived_turn_event_json", "source_file_id":row.file_id,
                            "response_timestamp_ms":row.created_at_ms, "data":data}),
                        ),
                        Ok(None) => None,
                        Err(_) => Some(raw_reference(&row, "decode_failed")),
                    }
                }
            }
        };
        if let Some(value) = reference {
            let encoded = encode_bounded(&value, available).or_else(|_| {
                encode_bounded(
                    &raw_reference(&row, "derived_record_byte_budget"),
                    available,
                )
            });
            let encoded = match encoded {
                Ok(value) => value,
                Err(()) => {
                    has_more = true;
                    stop_reason = "page_byte_budget";
                    break;
                }
            };
            // Account for the comma separately, including the final reserved comma.
            if encoded.len() + 1 > available {
                has_more = true;
                stop_reason = "page_byte_budget";
                break;
            }
            emitted_bytes += encoded.len() + 1;
            records.push(encoded);
        }
        next = row.file_id;
    }
    let mut header = json!({"ok":true,"schema_version":2,"cursor_version":2,"session_id":request.session_id,
        "ordering":"file_id","after_file_id":request.after_file_id,"next_after_file_id":next,
        "count":records.len(),"has_more":has_more,"stop_reason":stop_reason,
        "scanned_files":scanned,"decoded_bytes":decoded_bytes,
        "budgets":{"records":request.limit,"page_bytes":request.page_bytes,"file_decode_bytes":request.decode_bytes,
            "page_decode_bytes":MAX_PAGE_DECODE_BYTES,"scanned_files":MAX_SCANNED_FILES}}).to_string();
    header.pop();
    header.push_str(",\"records\":[");
    for (i, record) in records.iter().enumerate() {
        if i != 0 {
            header.push(',');
        }
        header.push_str(std::str::from_utf8(record).map_err(|_| "invalid_serialized_utf8")?);
    }
    header.push_str("]}");
    if header.len() > request.page_bytes {
        return Err("page_envelope_byte_budget".into());
    }
    Ok(header)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicUsize, Ordering};
    static SERIAL: AtomicUsize = AtomicUsize::new(0);
    struct Fixture(std::path::PathBuf);
    impl Fixture {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "derived-export-{}-{}",
                std::process::id(),
                SERIAL.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }
        fn row(&self, id: i64, bytes: &[u8]) -> IndexedFile {
            let path = format!("payload-{}.bin", id);
            fs::write(self.0.join(&path), bytes).unwrap();
            IndexedFile {
                file_id: id,
                session_id: "session".into(),
                relative_path: path,
                byte_length: bytes.len() as u64,
                sha256: None,
                created_at_ms: id,
            }
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    fn request() -> Request {
        Request::parse(&[
            ("session_id".into(), "session".into()),
            ("cursor_version".into(), "2".into()),
        ])
        .unwrap()
    }
    fn mp_text(value: &str) -> Vec<u8> {
        let bytes = value.as_bytes();
        let mut out = vec![0xdb];
        out.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
        out.extend_from_slice(bytes);
        out
    }
    fn mp_map(fields: Vec<(&str, Vec<u8>)>) -> Vec<u8> {
        let mut out = vec![0xde];
        out.extend_from_slice(&(fields.len() as u16).to_be_bytes());
        for (key, value) in fields {
            out.extend(mp_text(key));
            out.extend(value);
        }
        out
    }
    fn mp_response(chara: Vec<u8>, other: Vec<u8>) -> Vec<u8> {
        mp_map(vec![(
            "data",
            mp_map(vec![
                ("chara_info", chara),
                ("single_mode_data_set", vec![0x80]),
                ("ignored_large", other),
            ]),
        )])
    }
    #[test]
    fn production_decoder_selects_fields_and_preserves_scalars_binary_and_ext() {
        let chara = mp_map(vec![
            ("name", mp_text("中文\n\"")),
            (
                "signed",
                vec![0xd3, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe],
            ),
            (
                "unsigned",
                vec![0xcf, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff],
            ),
            ("binary", vec![0xc4, 2, 0xab, 0x01]),
            ("ext", vec![0xd4, 0xff, 0xab]),
            ("array", vec![0x94, 0xc0, 0xc2, 0xc3, 0xff]),
            ("nan", vec![0xca, 0x7f, 0xc0, 0, 0]),
        ]);
        let bytes = mp_response(chara, mp_text(&"ignored".repeat(1000)));
        let selected = decode_turn_event(&bytes, 2048).unwrap().unwrap();
        assert_eq!(selected["chara_info"]["name"], "中文\n\"");
        assert_eq!(selected["chara_info"]["signed"], -2);
        assert_eq!(selected["chara_info"]["unsigned"].as_u64(), Some(u64::MAX));
        assert_eq!(selected["chara_info"]["binary"]["body_hex"], "ab01");
        assert_eq!(selected["chara_info"]["ext"]["ext_type"], -1);
        assert_eq!(
            selected["chara_info"]["array"],
            json!([null, false, true, -1])
        );
        assert!(selected["chara_info"]["nan"].is_null());
        assert!(selected.get("ignored_large").is_none());
    }
    #[test]
    fn production_decoder_rejects_untrusted_lengths_before_any_container_allocation() {
        for bytes in [
            vec![0xdd, 0xff, 0xff, 0xff, 0xff],
            vec![0xdf, 0xff, 0xff, 0xff, 0xff],
        ] {
            assert_eq!(
                decode_turn_event(&bytes, 4096).unwrap_err(),
                "msgpack_node_budget"
            );
        }
        for bytes in [
            vec![0xdb, 0xff, 0xff, 0xff, 0xff],
            vec![0xc6, 0xff, 0xff, 0xff, 0xff],
            vec![0xc9, 0xff, 0xff, 0xff, 0xff, 0],
        ] {
            assert_eq!(
                decode_turn_event(&bytes, 4096).unwrap_err(),
                "truncated_msgpack"
            );
        }
    }
    #[test]
    fn production_decoder_enforces_depth_node_input_and_output_budgets() {
        let mut deep = vec![0x91; MAX_MSGPACK_DEPTH + 1];
        deep.push(0xc0);
        assert_eq!(
            decode_turn_event(&deep, 4096).unwrap_err(),
            "msgpack_depth_budget"
        );
        let mut wide = vec![0xdc, 0x80, 0];
        wide.extend(vec![0xc0; MAX_MSGPACK_NODES]);
        assert_eq!(
            decode_turn_event(&wide, 4096).unwrap_err(),
            "msgpack_node_budget"
        );
        assert_eq!(
            decode_turn_event(&vec![0; MAX_FILE_DECODE_BYTES + 1], 4096).unwrap_err(),
            "msgpack_input_budget"
        );
        let expanded = mp_response(mp_text(&"\n".repeat(3000)), vec![0xc0]);
        assert_eq!(
            decode_turn_event(&expanded, 4096).unwrap_err(),
            "msgpack_output_budget"
        );
    }
    #[test]
    fn production_decoder_checks_ignored_structure_and_trailing_bytes() {
        let bytes = mp_response(vec![0x80], vec![0xdb, 0xff, 0xff, 0xff, 0xff]);
        assert_eq!(
            decode_turn_event(&bytes, 4096).unwrap_err(),
            "truncated_msgpack"
        );
        let mut bytes = mp_response(vec![0x80], vec![0xc0]);
        bytes.push(0xc0);
        assert_eq!(
            decode_turn_event(&bytes, 4096).unwrap_err(),
            "trailing_msgpack_bytes"
        );
        assert!(decode_turn_event(
            &mp_map(vec![("data", mp_map(vec![("unrelated", vec![0xc0])]))]),
            4096
        )
        .unwrap()
        .is_none());
    }
    #[test]
    fn production_decoder_reports_unsupported_keys_and_invalid_utf8_without_data_loss() {
        let nontextmap = vec![0x81, 0x01, 0xc0];
        assert_eq!(
            decode_turn_event(&mp_response(nontextmap, vec![0xc0]), 4096).unwrap_err(),
            "non_text_msgpack_map_key"
        );
        assert_eq!(
            decode_turn_event(&mp_response(vec![0xa1, 0xff], vec![0xc0]), 4096).unwrap_err(),
            "invalid_msgpack_utf8"
        );
    }
    #[test]
    fn production_decoder_is_used_by_the_page_raw_reference_path() {
        let f = Fixture::new();
        let bytes = mp_response(mp_text(&"\n".repeat(2000)), vec![0xc0]);
        let row = f.row(1, &bytes);
        let mut r = request();
        r.page_bytes = 4096;
        let p = page(&f, &r, vec![row], decode_turn_event);
        assert_eq!(p["records"][0]["representation"], "raw_file_reference");
        assert_eq!(p["next_after_file_id"], 1);
        assert_eq!(fs::read(f.0.join("payload-1.bin")).unwrap(), bytes);
    }
    #[test]
    fn legacy_character_limit_is_an_explicit_migration_error() {
        let pairs = vec![
            ("cursor_version".into(), "2".into()),
            ("session_id".into(), "session".into()),
            ("max_json_chars".into(), "100".into()),
        ];
        assert!(Request::parse(&pairs)
            .unwrap_err()
            .contains("legacy_max_json_chars"));
    }
    fn page<F: FnMut(&[u8], usize) -> Result<Option<Value>, String>>(
        f: &Fixture,
        r: &Request,
        rows: Vec<IndexedFile>,
        decode: F,
    ) -> Value {
        serde_json::from_str(&export_page(&f.0, r, rows, decode).unwrap()).unwrap()
    }
    #[test]
    fn old_cursor_is_explicitly_rejected() {
        assert!(Request::parse(&[("session_id".into(), "session".into())]).is_err());
        for key in ["after_sequence", "cursor"] {
            assert!(Request::parse(&[
                ("session_id".into(), "session".into()),
                ("cursor_version".into(), "2".into()),
                (key.into(), "0".into())
            ])
            .unwrap_err()
            .contains("legacy_cursor"));
        }
    }
    #[test]
    fn invalid_limits_and_duplicate_cursor_fail_before_io() {
        let mut r = request();
        r.page_bytes = 4095;
        assert!(
            export_page(Path::new("missing"), &r, Vec::new(), |_, _| Ok(None))
                .unwrap_err()
                .contains("page_bytes")
        );
        assert!(Request::parse(&[
            ("cursor_version".into(), "2".into()),
            ("cursor_version".into(), "2".into())
        ])
        .unwrap_err()
        .contains("duplicate"));
    }
    #[test]
    fn source_cursor_is_applied_before_read_or_decode() {
        let f = Fixture::new();
        let mut r = request();
        r.after_file_id = 5;
        let old = IndexedFile {
            file_id: 1,
            relative_path: "missing.bin".into(),
            ..f.row(2, b"old")
        };
        let mut calls = 0;
        let p = page(&f, &r, vec![old, f.row(6, b"new")], |bytes, _| {
            calls += 1;
            assert_eq!(bytes, b"new");
            Ok(Some(json!({"turn":1})))
        });
        assert_eq!(calls, 1);
        assert_eq!(p["next_after_file_id"], 6);
        assert_eq!(p["scanned_files"], 1);
    }
    #[test]
    fn page_limit_stops_before_reading_next_file() {
        let f = Fixture::new();
        let mut r = request();
        r.limit = 1;
        let mut calls = 0;
        let p = page(&f, &r, vec![f.row(1, b"a"), f.row(2, b"b")], |_, _| {
            calls += 1;
            Ok(Some(json!({"turn":1})))
        });
        assert_eq!(calls, 1);
        assert_eq!(p["has_more"], true);
        assert_eq!(p["next_after_file_id"], 1);
    }
    #[test]
    fn oversized_indexed_file_is_not_opened_or_decoded() {
        let f = Fixture::new();
        let mut row = f.row(1, b"old");
        row.relative_path = "nonexistent.bin".into();
        row.byte_length = MAX_FILE_DECODE_BYTES as u64 + 1;
        let p = page(&f, &request(), vec![row], |_, _| {
            panic!("oversized file must not decode")
        });
        assert_eq!(p["records"][0]["reason"], "file_decode_byte_budget");
        assert_eq!(p["decoded_bytes"], 0);
        assert_eq!(p["next_after_file_id"], 1);
    }
    #[test]
    fn changed_indexed_length_is_reported_with_original_reference() {
        let f = Fixture::new();
        let row = f.row(1, b"original");
        fs::write(f.0.join(&row.relative_path), b"changed-length").unwrap();
        let p = page(&f, &request(), vec![row], |_, _| {
            panic!("changed file must not decode")
        });
        assert_eq!(p["records"][0]["reason"], "indexed_file_length_changed");
        assert_eq!(p["records"][0]["byte_length"], 8);
    }
    #[test]
    fn output_expansion_returns_raw_reference_without_truncation() {
        let f = Fixture::new();
        let mut r = request();
        r.page_bytes = 4096;
        let body = export_page(&f.0, &r, vec![f.row(1, b"original")], |_, _| {
            Ok(Some(json!({"text":"\"中".repeat(5000)})))
        })
        .unwrap();
        assert!(body.len() <= 4096);
        let p: Value = serde_json::from_str(&body).unwrap();
        assert_eq!(p["records"][0]["reason"], "derived_record_byte_budget");
        assert_eq!(fs::read(f.0.join("payload-1.bin")).unwrap(), b"original");
    }
    #[test]
    fn nonmatches_and_decode_errors_advance_file_cursor() {
        let f = Fixture::new();
        let p = page(
            &f,
            &request(),
            vec![f.row(1, b"no"), f.row(2, b"bad"), f.row(3, b"no")],
            |bytes, _| {
                if bytes == b"bad" {
                    Err("bad msgpack".into())
                } else {
                    Ok(None)
                }
            },
        );
        assert_eq!(p["next_after_file_id"], 3);
        assert_eq!(p["count"], 1);
        assert_eq!(p["records"][0]["reason"], "decode_failed");
        assert_eq!(p["has_more"], false);
    }
    #[test]
    fn scan_budget_bounds_nonmatching_history_work() {
        let f = Fixture::new();
        let rows = (1..=100).map(|id| f.row(id, b"no")).collect();
        let mut calls = 0;
        let p = page(&f, &request(), rows, |_, _| {
            calls += 1;
            Ok(None)
        });
        assert_eq!(calls, MAX_SCANNED_FILES);
        assert_eq!(p["has_more"], true);
        assert_eq!(p["next_after_file_id"], 64);
    }
    #[test]
    fn cumulative_decode_work_has_its_own_byte_budget() {
        let f = Fixture::new();
        let rows = (1..=5)
            .map(|id| f.row(id, &vec![b'a'; MAX_FILE_DECODE_BYTES]))
            .collect();
        let mut calls = 0;
        let p = page(&f, &request(), rows, |_, _| {
            calls += 1;
            Ok(None)
        });
        assert_eq!(calls, 4);
        assert_eq!(p["decoded_bytes"], MAX_PAGE_DECODE_BYTES);
        assert_eq!(p["records"][0]["reason"], "page_decode_byte_budget");
        assert_eq!(p["next_after_file_id"], 5);
    }
    #[test]
    fn page_byte_limit_never_loses_an_unemitted_file() {
        let f = Fixture::new();
        let mut r = request();
        r.page_bytes = 4096;
        let rows: Vec<_> = (1..=20).map(|id| f.row(id, b"data")).collect();
        let first = page(&f, &r, rows.clone(), |_, _| {
            Ok(Some(json!({"text":"a".repeat(600)})))
        });
        assert_eq!(first["has_more"], true);
        let cursor = first["next_after_file_id"].as_i64().unwrap();
        assert!(cursor > 0 && cursor < 20);
        r.after_file_id = cursor;
        let second = page(
            &f,
            &r,
            rows.into_iter()
                .filter(|row| row.file_id > cursor)
                .collect(),
            |_, _| Ok(Some(json!({"text":"a".repeat(600)}))),
        );
        assert_eq!(second["records"][0]["source_file_id"], cursor + 1);
    }
    #[test]
    fn indexed_path_and_session_boundaries_are_enforced() {
        let f = Fixture::new();
        let mut row = f.row(1, b"a");
        row.relative_path = "../secret".into();
        let p = page(&f, &request(), vec![row.clone()], |_, _| {
            panic!("traversal must not decode")
        });
        assert_eq!(p["records"][0]["reason"], "invalid_indexed_path");
        row.session_id = "other".into();
        assert!(export_page(&f.0, &request(), vec![row], |_, _| Ok(None))
            .unwrap_err()
            .contains("session_mismatch"));
    }
    #[test]
    fn out_of_order_index_rows_are_not_reinterpreted_as_cursor_order() {
        let f = Fixture::new();
        assert!(export_page(
            &f.0,
            &request(),
            vec![f.row(2, b"a"), f.row(1, b"b")],
            |_, _| Ok(None)
        )
        .unwrap_err()
        .contains("strictly_ordered"));
    }
}
