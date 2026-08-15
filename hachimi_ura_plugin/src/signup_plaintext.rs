use rmpv::Value;
use std::collections::BTreeSet;
use std::io::Cursor;

const MAX_RECORDS: usize = 32;
const MAX_DECODE_BYTES: usize = 8 * 1024 * 1024;
const MAX_PREFIX_PROBE: usize = 64;
const MAX_INLINE_BINARY: usize = 4096;
const MAX_VALUE_DEPTH: usize = 12;
const MAX_CONTAINER_ITEMS: usize = 256;
const MAX_STRING_CHARS: usize = 16 * 1024;
const MAX_FINDINGS: usize = 64;
const MAX_FINDING_CHARS: usize = 512;
const MAX_HEX_PREVIEW: usize = 32;

fn is_signup_url(url: &str) -> bool {
    let path = super::sniff_path(url).to_ascii_lowercase();
    path.contains("pre_signup") || path.contains("pre-signup") || path.contains("/signup")
}

fn bounded_text(value: &str, max_chars: usize) -> (String, bool) {
    let mut output = String::new();
    let mut chars = value.chars();
    for _ in 0..max_chars {
        match chars.next() {
            Some(ch) => output.push(ch),
            None => return (output, false),
        }
    }
    (output, chars.next().is_some())
}

fn decode_messagepack(bytes: &[u8], prefix_probe: bool) -> Option<(usize, Value)> {
    if bytes.is_empty() || bytes.len() > MAX_DECODE_BYTES {
        return None;
    }
    let max_offset = if prefix_probe {
        MAX_PREFIX_PROBE.min(bytes.len().saturating_sub(1))
    } else {
        0
    };
    for offset in 0..=max_offset {
        let remaining = &bytes[offset..];
        let mut cursor = Cursor::new(remaining);
        if let Ok(value) = rmpv::decode::read_value(&mut cursor) {
            if cursor.position() as usize == remaining.len() {
                return Some((offset, value));
            }
        }
    }
    None
}

fn value_json(value: &Value, depth: usize) -> String {
    if depth >= MAX_VALUE_DEPTH {
        return r#"{"truncated":true,"reason":"max_depth"}"#.to_string();
    }
    match value {
        Value::Nil => "null".to_string(),
        Value::Boolean(v) => v.to_string(),
        Value::Integer(v) => v.to_string(),
        Value::F32(v) => if v.is_finite() { v.to_string() } else { "null".to_string() },
        Value::F64(v) => if v.is_finite() { v.to_string() } else { "null".to_string() },
        Value::String(v) => {
            let text = v.as_str().unwrap_or("");
            let (bounded, truncated) = bounded_text(text, MAX_STRING_CHARS);
            if truncated {
                format!(r#"{{"type":"string","value":"{}","truncated":true}}"#, super::json_escape(&bounded))
            } else {
                format!("\"{}\"", super::json_escape(&bounded))
            }
        }
        Value::Binary(bytes) => binary_json(bytes, depth + 1),
        Value::Array(values) => {
            let shown = values.iter().take(MAX_CONTAINER_ITEMS)
                .map(|item| value_json(item, depth + 1)).collect::<Vec<_>>();
            if values.len() > shown.len() {
                format!(r#"{{"type":"array","length":{},"items":[{}],"truncated":true}}"#,
                    values.len(), shown.join(","))
            } else {
                format!("[{}]", shown.join(","))
            }
        }
        Value::Map(values) => {
            let shown = values.iter().take(MAX_CONTAINER_ITEMS).map(|(key, value)| {
                let key = match key {
                    Value::String(v) => v.as_str().unwrap_or("").to_string(),
                    _ => format!("{:?}", key),
                };
                format!("\"{}\":{}", super::json_escape(&key), value_json(value, depth + 1))
            }).collect::<Vec<_>>();
            if values.len() > shown.len() {
                format!(r#"{{"type":"map","length":{},"fields":{{{}}},"truncated":true}}"#,
                    values.len(), shown.join(","))
            } else {
                format!("{{{}}}", shown.join(","))
            }
        }
        Value::Ext(kind, bytes) => {
            format!(r#"{{"type":"extension","kind":{},"payload":{}}}"#,
                kind, binary_json(bytes, depth + 1))
        }
    }
}

fn binary_json(bytes: &[u8], depth: usize) -> String {
    // Binary fields may contain a second protocol envelope. Analyze them regardless
    // of whether their parent was captured at a plaintext or Unity wire boundary.
    if depth < MAX_VALUE_DEPTH {
        if let Some((prefix, nested)) = decode_messagepack(bytes, true) {
            return format!(r#"{{"type":"binary","length":{},"nested_candidate":{{"codec":"messagepack","confidence":"heuristic_candidate","prefix_bytes":{},"value":{}}}}}"#,
                bytes.len(), prefix, value_json(&nested, depth + 1));
        }
        if let Ok(text) = std::str::from_utf8(bytes) {
            let (bounded, truncated) = bounded_text(text, MAX_STRING_CHARS);
            return format!(r#"{{"type":"binary","length":{},"nested_candidate":{{"codec":"utf8","confidence":"heuristic_candidate","value":"{}","truncated":{}}}}}"#,
                bytes.len(), super::json_escape(&bounded), truncated);
        }
    }
    if bytes.len() <= MAX_INLINE_BINARY {
        format!(r#"{{"type":"binary","length":{},"hex":"{}"}}"#, bytes.len(), super::hex_encode(bytes))
    } else {
        format!(r#"{{"type":"binary","length":{},"inline":false}}"#, bytes.len())
    }
}

fn printable_findings(bytes: &[u8]) -> (Vec<String>, Vec<String>, Vec<String>) {
    let mut strings = BTreeSet::new();
    let mut routes = BTreeSet::new();
    let mut classes = BTreeSet::new();
    let mut current = Vec::new();
    let flush = |current: &mut Vec<u8>, strings: &mut BTreeSet<String>, routes: &mut BTreeSet<String>, classes: &mut BTreeSet<String>| {
        if current.len() < 4 { current.clear(); return; }
        let raw = String::from_utf8_lossy(current);
        let (text, _) = bounded_text(raw.trim(), MAX_FINDING_CHARS);
        if text.is_empty() { current.clear(); return; }
        for token in text.split(|ch: char| ch.is_whitespace() || matches!(ch, '\"' | '\'' | ',' | ';' | '(' | ')' | '[' | ']' | '{' | '}')) {
            if token.starts_with('/') && token.len() >= 3 { routes.insert(token.to_string()); }
            let tail = token.rsplit(['.', '/', '+']).next().unwrap_or(token);
            if tail.len() >= 4 && tail.len() <= 160
                && tail.chars().next().map(|ch| ch.is_ascii_uppercase()).unwrap_or(false)
                && tail.chars().all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '`') {
                classes.insert(token.to_string());
            }
        }
        strings.insert(text);
        current.clear();
    };
    for byte in bytes.iter().copied() {
        if byte == b'\n' || byte == b'\r' || byte == b'\t' || (0x20..=0x7e).contains(&byte) {
            current.push(byte);
            if current.len() >= MAX_FINDING_CHARS { flush(&mut current, &mut strings, &mut routes, &mut classes); }
        } else {
            flush(&mut current, &mut strings, &mut routes, &mut classes);
        }
        if strings.len() >= MAX_FINDINGS { break; }
    }
    flush(&mut current, &mut strings, &mut routes, &mut classes);
    (strings.into_iter().take(MAX_FINDINGS).collect(), routes.into_iter().take(MAX_FINDINGS).collect(), classes.into_iter().take(MAX_FINDINGS).collect())
}

fn string_array_json(values: &[String]) -> String {
    values.iter().map(|value| format!("\"{}\"", super::json_escape(value)))
        .collect::<Vec<_>>().join(",")
}

fn byte_pattern_json(bytes: &[u8]) -> String {
    if bytes.is_empty() { return "null".to_string(); }
    let first = bytes[0];
    let same = bytes.iter().filter(|value| **value == first).count();
    if same.saturating_mul(100) / bytes.len().max(1) >= 80 {
        format!(r#"{{"kind":"repeated_byte","byte":{},"ratio_percent":{}}}"#, first, same * 100 / bytes.len())
    } else {
        "null".to_string()
    }
}

fn analyze_payload(bytes: &[u8], source_stage: &str, confidence: &str) -> String {
    if bytes.is_empty() {
        return format!(r#"{{"source_stage":"{}","confidence":"{}","codec":"empty","decoded":true,"value":null}}"#,
            source_stage, confidence);
    }
    if let Some((prefix, value)) = decode_messagepack(bytes, true) {
        return format!(r#"{{"source_stage":"{}","confidence":"{}","codec":"messagepack","decoded":true,"prefix_bytes":{},"value":{}}}"#,
            source_stage, confidence, prefix, value_json(&value, 0));
    }
    if let Ok(text) = std::str::from_utf8(bytes) {
        let (bounded, truncated) = bounded_text(text, MAX_STRING_CHARS);
        return format!(r#"{{"source_stage":"{}","confidence":"{}","codec":"utf8","decoded":true,"value":"{}","truncated":{}}}"#,
            source_stage, confidence, super::json_escape(&bounded), truncated);
    }
    let (strings, routes, classes) = printable_findings(bytes);
    let preview_len = bytes.len().min(MAX_HEX_PREVIEW);
    format!(r#"{{"source_stage":"{}","confidence":"heuristic_candidate","codec":"unknown","decoded":false,"size":{},"raw_inline":false,"hex_preview":"{}","hex_preview_truncated":{},"byte_pattern":{},"findings":{{"strings":[{}],"route_candidates":[{}],"class_name_candidates":[{}]}},"hint":"source stage is metadata, not a parsing prohibition; use raw capture for complete bytes"}}"#,
        source_stage, bytes.len(), super::hex_encode(&bytes[..preview_len]), bytes.len() > preview_len,
        byte_pattern_json(bytes), string_array_json(&strings), string_array_json(&routes), string_array_json(&classes))
}

pub unsafe fn endpoint() -> String {
    let _guard = match super::SNIFF_MUTEX.lock() {
        Ok(value) => value,
        Err(poisoned) => poisoned.into_inner(),
    };
    let mut request_ids = super::SNIFF_REQUESTS.iter()
        .filter(|(_, url, _, _)| is_signup_url(url))
        .map(|(id, _, _, _)| *id).collect::<Vec<_>>();
    request_ids.sort_unstable();
    request_ids.dedup();
    if request_ids.len() > MAX_RECORDS {
        request_ids.drain(0..request_ids.len() - MAX_RECORDS);
    }
    let records = request_ids.iter().filter_map(|id| {
        let request = super::SNIFF_REQUESTS.iter().rev().find(|item| item.0 == *id)?;
        let response = super::SNIFF_RESPONSES.iter().rev().find(|item| item.0 == *id);
        let response_json = response.map(|item| analyze_payload(&item.1, "response_plain", "verified_plaintext_boundary"))
            .unwrap_or_else(|| "null".to_string());
        Some(format!(r#"{{"request_id":{},"url":"{}","method":"{}","request_size":{},"request":{},"response_size":{},"response":{}}}"#,
            id, super::json_escape(&request.1), super::json_escape(&request.2), request.3.len(),
            analyze_payload(&request.3, "request_plain", "verified_plaintext_boundary"),
            response.map(|item| item.1.len()).unwrap_or(0), response_json))
    }).collect::<Vec<_>>();
    format!(r#"{{"ok":true,"schema_version":2,"mode":"provenance_aware_layered_analysis","filter":["pre_signup","signup"],"source_hooks":["HttpHelper.CompressRequest:input","HttpHelper.DecompressResponse:output","WWWRequest.Post:correlation"],"policy":{{"stage_is_metadata_not_prohibition":true,"wire_analysis_allowed":true,"nested_payload_analysis":true,"unknown_payloads_are_not_presented_as_plaintext":true,"raw_payload_omitted_from_compact_view":true}},"raw_payload_endpoint":"/api/sniff/metadata","count":{},"records":[{}]}}"#,
        records.len(), records.join(","))
}
