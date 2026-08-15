use rmpv::Value;
use std::io::Cursor;

const MAX_RECORDS: usize = 32;
const MAX_DECODE_BYTES: usize = 8 * 1024 * 1024;
const MAX_PREFIX_PROBE: usize = 64;
const MAX_INLINE_BINARY: usize = 4096;

fn is_signup_url(url: &str) -> bool {
    let path = super::sniff_path(url).to_ascii_lowercase();
    path.contains("pre_signup") || path.contains("pre-signup") || path.contains("/signup")
}

fn value_json(value: &Value) -> String {
    match value {
        Value::Nil => "null".to_string(),
        Value::Boolean(v) => v.to_string(),
        Value::Integer(v) => v.to_string(),
        Value::F32(v) => if v.is_finite() { v.to_string() } else { "null".to_string() },
        Value::F64(v) => if v.is_finite() { v.to_string() } else { "null".to_string() },
        Value::String(v) => format!("\"{}\"", super::json_escape(v.as_str().unwrap_or(""))),
        Value::Binary(v) => {
            if v.len() <= MAX_INLINE_BINARY {
                format!(r#"{{"type":"binary","length":{},"hex":"{}"}}"#, v.len(), super::hex_encode(v))
            } else {
                format!(r#"{{"type":"binary","length":{},"inline":false}}"#, v.len())
            }
        }
        Value::Array(values) => format!(
            "[{}]",
            values.iter().map(value_json).collect::<Vec<_>>().join(",")
        ),
        Value::Map(values) => {
            let fields = values.iter().map(|(key, value)| {
                let key = match key {
                    Value::String(v) => v.as_str().unwrap_or("").to_string(),
                    _ => format!("{:?}", key),
                };
                format!("\"{}\":{}", super::json_escape(&key), value_json(value))
            }).collect::<Vec<_>>();
            format!("{{{}}}", fields.join(","))
        }
        Value::Ext(kind, bytes) => {
            if bytes.len() <= MAX_INLINE_BINARY {
                format!(r#"{{"type":"extension","kind":{},"length":{},"hex":"{}"}}"#,
                    kind, bytes.len(), super::hex_encode(bytes))
            } else {
                format!(r#"{{"type":"extension","kind":{},"length":{},"inline":false}}"#,
                    kind, bytes.len())
            }
        }
    }
}

fn decode_messagepack(bytes: &[u8]) -> Option<(usize, String)> {
    if bytes.is_empty() || bytes.len() > MAX_DECODE_BYTES {
        return None;
    }
    let max_offset = MAX_PREFIX_PROBE.min(bytes.len().saturating_sub(1));
    for offset in 0..=max_offset {
        let remaining = &bytes[offset..];
        let mut cursor = Cursor::new(remaining);
        if let Ok(value) = rmpv::decode::read_value(&mut cursor) {
            if cursor.position() as usize == remaining.len() {
                return Some((offset, value_json(&value)));
            }
        }
    }
    None
}

fn decode_payload(bytes: &[u8]) -> String {
    if bytes.is_empty() {
        return r#"{"codec":"empty","value":null}"#.to_string();
    }
    if let Some((prefix, value)) = decode_messagepack(bytes) {
        return format!(r#"{{"codec":"messagepack","prefix_bytes":{},"value":{}}}"#, prefix, value);
    }
    if let Ok(text) = std::str::from_utf8(bytes) {
        let trimmed = text.trim();
        if (trimmed.starts_with('{') && trimmed.ends_with('}'))
            || (trimmed.starts_with('[') && trimmed.ends_with(']')) {
            return format!(r#"{{"codec":"json","value":{}}}"#, trimmed);
        }
        return format!(r#"{{"codec":"utf8","value":"{}"}}"#, super::json_escape(text));
    }
    format!(
        r#"{{"codec":"unknown","decoded":false,"size":{},"hint":"read raw bytes from /api/sniff/metadata or session protocol archive"}}"#,
        bytes.len()
    )
}

pub unsafe fn endpoint() -> String {
    let _guard = match super::SNIFF_MUTEX.lock() {
        Ok(value) => value,
        Err(poisoned) => poisoned.into_inner(),
    };
    let mut request_ids = super::SNIFF_REQUESTS.iter()
        .filter(|(_, url, _, _)| is_signup_url(url))
        .map(|(id, _, _, _)| *id)
        .collect::<Vec<_>>();
    request_ids.sort_unstable();
    request_ids.dedup();
    if request_ids.len() > MAX_RECORDS {
        request_ids.drain(0..request_ids.len() - MAX_RECORDS);
    }
    let records = request_ids.iter().filter_map(|id| {
        let request = super::SNIFF_REQUESTS.iter().rev().find(|item| item.0 == *id)?;
        let response = super::SNIFF_RESPONSES.iter().rev().find(|item| item.0 == *id);
        let response_json = response
            .map(|item| decode_payload(&item.1))
            .unwrap_or_else(|| "null".to_string());
        Some(format!(
            r#"{{"request_id":{},"url":"{}","method":"{}","request_size":{},"request":{},"response_size":{},"response":{}}}"#,
            id,
            super::json_escape(&request.1),
            super::json_escape(&request.2),
            request.3.len(),
            decode_payload(&request.3),
            response.map(|item| item.1.len()).unwrap_or(0),
            response_json,
        ))
    }).collect::<Vec<_>>();
    format!(
        r#"{{"ok":true,"mode":"plaintext_compact","filter":["pre_signup","signup"],"source_hooks":["HttpHelper.CompressRequest:input","HttpHelper.DecompressResponse:output","WWWRequest.Post:correlation"],"raw_payload_omitted":true,"raw_payload_endpoint":"/api/sniff/metadata","count":{},"records":[{}]}}"#,
        records.len(), records.join(",")
    )
}
