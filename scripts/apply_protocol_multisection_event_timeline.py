from pathlib import Path

SOURCE = Path("hachimi_ura_plugin/src/lib.rs")
s = SOURCE.read_text(encoding="utf-8")
MARKER = "// ===== Protocol multi-section event timeline O-stage ====="
if MARKER in s:
    print("protocol_multisection_event_timeline=already_applied")
    raise SystemExit(0)


def replace_once(old: str, new: str, label: str) -> None:
    global s
    count = s.count(old)
    assert count == 1, f"{label} anchor count={count}"
    s = s.replace(old, new, 1)

storage_anchor = '''fn persist_protocol_observation_boundary(
    direction: &str,
'''
assert s.count(storage_anchor) == 1

rust = r'''// ===== Protocol multi-section event timeline O-stage =====
#[derive(Default)]
struct ProtocolSectionScan {
    turn_panel_paths: Vec<String>,
    event_paths: Vec<String>,
    choice_prompt_paths: Vec<String>,
    choice_result_paths: Vec<String>,
    training_paths: Vec<String>,
    choice_index: Option<i64>,
    story_id: Option<i64>,
    event_id: Option<i64>,
    decode_error: Option<String>,
}

#[derive(Clone)]
struct PendingProtocolChoice {
    request_id: u64,
    choice_index: i64,
    story_id: Option<i64>,
    event_id: Option<i64>,
    submitted_at_ms: u64,
}

static PROTOCOL_PENDING_CHOICE: Mutex<Option<PendingProtocolChoice>> = Mutex::new(None);

fn msgpack_read_u16(data: &[u8], offset: usize) -> Option<u16> {
    Some(u16::from_be_bytes([*data.get(offset)?, *data.get(offset + 1)?]))
}
fn msgpack_read_u32(data: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_be_bytes([
        *data.get(offset)?, *data.get(offset + 1)?, *data.get(offset + 2)?, *data.get(offset + 3)?,
    ]))
}
fn msgpack_read_u64(data: &[u8], offset: usize) -> Option<u64> {
    Some(u64::from_be_bytes([
        *data.get(offset)?, *data.get(offset + 1)?, *data.get(offset + 2)?, *data.get(offset + 3)?,
        *data.get(offset + 4)?, *data.get(offset + 5)?, *data.get(offset + 6)?, *data.get(offset + 7)?,
    ]))
}

fn msgpack_string_at(data: &[u8], offset: &mut usize) -> Result<Option<String>, String> {
    let marker = *data.get(*offset).ok_or_else(|| "unexpected_eof_string_marker".to_string())?;
    *offset += 1;
    let length = match marker {
        0xa0..=0xbf => usize::from(marker & 0x1f),
        0xd9 => { let value = *data.get(*offset).ok_or_else(|| "unexpected_eof_str8".to_string())?; *offset += 1; usize::from(value) },
        0xda => { let value = msgpack_read_u16(data, *offset).ok_or_else(|| "unexpected_eof_str16".to_string())?; *offset += 2; usize::from(value) },
        0xdb => { let value = msgpack_read_u32(data, *offset).ok_or_else(|| "unexpected_eof_str32".to_string())?; *offset += 4; value as usize },
        _ => { *offset -= 1; return Ok(None); }
    };
    let end = offset.checked_add(length).ok_or_else(|| "string_length_overflow".to_string())?;
    let bytes = data.get(*offset..end).ok_or_else(|| "unexpected_eof_string_data".to_string())?;
    *offset = end;
    Ok(Some(String::from_utf8_lossy(bytes).into_owned()))
}

fn protocol_classify_key(scan: &mut ProtocolSectionScan, path: &str, key: &str, integer: Option<i64>) {
    let lower = key.to_ascii_lowercase();
    let full_path = if path.is_empty() { key.to_string() } else { format!("{}.{}", path, key) };
    let push_once = |list: &mut Vec<String>, value: &str| {
        if !list.iter().any(|item| item == value) { list.push(value.to_string()); }
    };
    if lower == "choice_index" || lower == "select_index" || lower == "selected_index" {
        if let Some(value) = integer { scan.choice_index = Some(value); }
    }
    if lower == "story_id" { if let Some(value) = integer { scan.story_id = Some(value); } }
    if lower == "event_id" { if let Some(value) = integer { scan.event_id = Some(value); } }

    if lower.contains("home_info") || lower.contains("command_info") || lower == "turn" || lower.ends_with("_turn") {
        push_once(&mut scan.turn_panel_paths, &full_path);
    }
    if lower.contains("training") || lower.contains("command_info") {
        push_once(&mut scan.training_paths, &full_path);
    }
    if lower.contains("choice_result") || lower.contains("event_result") || lower.contains("select_result") {
        push_once(&mut scan.choice_result_paths, &full_path);
    }
    if lower.contains("choice") && !lower.contains("result") && lower != "choice_index" {
        push_once(&mut scan.choice_prompt_paths, &full_path);
    }
    if (lower.contains("event") || lower.contains("story")) && !lower.contains("result") {
        push_once(&mut scan.event_paths, &full_path);
    }
}

fn msgpack_walk_value(
    data: &[u8], offset: &mut usize, path: &str, scan: &mut ProtocolSectionScan, depth: usize,
) -> Result<Option<i64>, String> {
    if depth > 512 { return Err("msgpack_nesting_exceeds_512".to_string()); }
    let marker = *data.get(*offset).ok_or_else(|| "unexpected_eof_value".to_string())?;
    if marker <= 0x7f { *offset += 1; return Ok(Some(i64::from(marker))); }
    if marker >= 0xe0 { *offset += 1; return Ok(Some(i64::from(marker as i8))); }
    if matches!(marker, 0xa0..=0xbf | 0xd9 | 0xda | 0xdb) {
        let _ = msgpack_string_at(data, offset)?;
        return Ok(None);
    }
    *offset += 1;
    match marker {
        0x80..=0x8f => {
            let count = usize::from(marker & 0x0f);
            msgpack_walk_map(data, offset, path, scan, depth + 1, count)?;
        }
        0x90..=0x9f => {
            let count = usize::from(marker & 0x0f);
            for index in 0..count {
                let child = format!("{}[{}]", path, index);
                msgpack_walk_value(data, offset, &child, scan, depth + 1)?;
            }
        }
        0xc0 | 0xc2 | 0xc3 => {}
        0xc4 => { let n = usize::from(*data.get(*offset).ok_or_else(|| "unexpected_eof_bin8".to_string())?); *offset += 1; *offset = offset.checked_add(n).filter(|end| *end <= data.len()).ok_or_else(|| "unexpected_eof_bin8_data".to_string())?; }
        0xc5 => { let n = usize::from(msgpack_read_u16(data, *offset).ok_or_else(|| "unexpected_eof_bin16".to_string())?); *offset += 2; *offset = offset.checked_add(n).filter(|end| *end <= data.len()).ok_or_else(|| "unexpected_eof_bin16_data".to_string())?; }
        0xc6 => { let n = msgpack_read_u32(data, *offset).ok_or_else(|| "unexpected_eof_bin32".to_string())? as usize; *offset += 4; *offset = offset.checked_add(n).filter(|end| *end <= data.len()).ok_or_else(|| "unexpected_eof_bin32_data".to_string())?; }
        0xca => { *offset = offset.checked_add(4).filter(|end| *end <= data.len()).ok_or_else(|| "unexpected_eof_float32".to_string())?; }
        0xcb => { *offset = offset.checked_add(8).filter(|end| *end <= data.len()).ok_or_else(|| "unexpected_eof_float64".to_string())?; }
        0xcc => { let value = *data.get(*offset).ok_or_else(|| "unexpected_eof_uint8".to_string())?; *offset += 1; return Ok(Some(i64::from(value))); }
        0xcd => { let value = msgpack_read_u16(data, *offset).ok_or_else(|| "unexpected_eof_uint16".to_string())?; *offset += 2; return Ok(Some(i64::from(value))); }
        0xce => { let value = msgpack_read_u32(data, *offset).ok_or_else(|| "unexpected_eof_uint32".to_string())?; *offset += 4; return Ok(Some(i64::from(value))); }
        0xcf => { let value = msgpack_read_u64(data, *offset).ok_or_else(|| "unexpected_eof_uint64".to_string())?; *offset += 8; return Ok(i64::try_from(value).ok()); }
        0xd0 => { let value = *data.get(*offset).ok_or_else(|| "unexpected_eof_int8".to_string())? as i8; *offset += 1; return Ok(Some(i64::from(value))); }
        0xd1 => { let value = msgpack_read_u16(data, *offset).ok_or_else(|| "unexpected_eof_int16".to_string())? as i16; *offset += 2; return Ok(Some(i64::from(value))); }
        0xd2 => { let value = msgpack_read_u32(data, *offset).ok_or_else(|| "unexpected_eof_int32".to_string())? as i32; *offset += 4; return Ok(Some(i64::from(value))); }
        0xd3 => { let value = msgpack_read_u64(data, *offset).ok_or_else(|| "unexpected_eof_int64".to_string())? as i64; *offset += 8; return Ok(Some(value)); }
        0xdc => { let count = usize::from(msgpack_read_u16(data, *offset).ok_or_else(|| "unexpected_eof_array16".to_string())?); *offset += 2; for index in 0..count { let child = format!("{}[{}]", path, index); msgpack_walk_value(data, offset, &child, scan, depth + 1)?; } }
        0xdd => { let count = msgpack_read_u32(data, *offset).ok_or_else(|| "unexpected_eof_array32".to_string())? as usize; *offset += 4; for index in 0..count { let child = format!("{}[{}]", path, index); msgpack_walk_value(data, offset, &child, scan, depth + 1)?; } }
        0xde => { let count = usize::from(msgpack_read_u16(data, *offset).ok_or_else(|| "unexpected_eof_map16".to_string())?); *offset += 2; msgpack_walk_map(data, offset, path, scan, depth + 1, count)?; }
        0xdf => { let count = msgpack_read_u32(data, *offset).ok_or_else(|| "unexpected_eof_map32".to_string())? as usize; *offset += 4; msgpack_walk_map(data, offset, path, scan, depth + 1, count)?; }
        0xd4 => { *offset = offset.checked_add(2).filter(|end| *end <= data.len()).ok_or_else(|| "unexpected_eof_fixext1".to_string())?; }
        0xd5 => { *offset = offset.checked_add(3).filter(|end| *end <= data.len()).ok_or_else(|| "unexpected_eof_fixext2".to_string())?; }
        0xd6 => { *offset = offset.checked_add(5).filter(|end| *end <= data.len()).ok_or_else(|| "unexpected_eof_fixext4".to_string())?; }
        0xd7 => { *offset = offset.checked_add(9).filter(|end| *end <= data.len()).ok_or_else(|| "unexpected_eof_fixext8".to_string())?; }
        0xd8 => { *offset = offset.checked_add(17).filter(|end| *end <= data.len()).ok_or_else(|| "unexpected_eof_fixext16".to_string())?; }
        0xc7 => { let n = usize::from(*data.get(*offset).ok_or_else(|| "unexpected_eof_ext8".to_string())?); *offset += 1; *offset = offset.checked_add(n + 1).filter(|end| *end <= data.len()).ok_or_else(|| "unexpected_eof_ext8_data".to_string())?; }
        0xc8 => { let n = usize::from(msgpack_read_u16(data, *offset).ok_or_else(|| "unexpected_eof_ext16".to_string())?); *offset += 2; *offset = offset.checked_add(n + 1).filter(|end| *end <= data.len()).ok_or_else(|| "unexpected_eof_ext16_data".to_string())?; }
        0xc9 => { let n = msgpack_read_u32(data, *offset).ok_or_else(|| "unexpected_eof_ext32".to_string())? as usize; *offset += 4; *offset = offset.checked_add(n + 1).filter(|end| *end <= data.len()).ok_or_else(|| "unexpected_eof_ext32_data".to_string())?; }
        0xc1 => return Err("reserved_msgpack_marker_c1".to_string()),
        _ => return Err(format!("unsupported_msgpack_marker_{:02x}", marker)),
    }
    Ok(None)
}

fn msgpack_walk_map(
    data: &[u8], offset: &mut usize, path: &str, scan: &mut ProtocolSectionScan, depth: usize, count: usize,
) -> Result<(), String> {
    for _ in 0..count {
        let key_start = *offset;
        let key = match msgpack_string_at(data, offset)? {
            Some(value) => value,
            None => {
                *offset = key_start;
                msgpack_walk_value(data, offset, path, scan, depth + 1)?;
                format!("<non_string_key@{}>", key_start)
            }
        };
        let child_path = if path.is_empty() { key.clone() } else { format!("{}.{}", path, key) };
        let integer = msgpack_walk_value(data, offset, &child_path, scan, depth + 1)?;
        protocol_classify_key(scan, path, &key, integer);
    }
    Ok(())
}

fn scan_protocol_sections(payload: &[u8]) -> ProtocolSectionScan {
    let mut scan = ProtocolSectionScan::default();
    let mut offset = 0usize;
    if let Err(error) = msgpack_walk_value(payload, &mut offset, "", &mut scan, 0) {
        scan.decode_error = Some(format!("{}@{}", error, offset));
    } else if offset != payload.len() {
        scan.decode_error = Some(format!("trailing_bytes:{}", payload.len() - offset));
    }
    scan
}

fn string_array_json(values: &[String]) -> String {
    values.iter().map(|value| format!("\"{}\"", json_escape(value))).collect::<Vec<_>>().join(",")
}

fn optional_i64_json(value: Option<i64>) -> String {
    value.map(|item| item.to_string()).unwrap_or_else(|| "null".to_string())
}

fn persist_protocol_semantic_timeline(
    direction: &str, request_id: u64, url: &str, relative_base: &str, payload: &[u8],
) -> Result<(), String> {
    let scan = scan_protocol_sections(payload);
    let mut visibility = "not_applicable";
    let mut linked_request_id = None;
    let mut linked_choice_index = None;

    if direction == "request" {
        if let Some(choice_index) = scan.choice_index {
            let pending = PendingProtocolChoice {
                request_id, choice_index, story_id: scan.story_id, event_id: scan.event_id,
                submitted_at_ms: sniff_timestamp_ms(),
            };
            let mut state = PROTOCOL_PENDING_CHOICE.lock()
                .map_err(|_| "pending_protocol_choice_lock_poisoned".to_string())?;
            *state = Some(pending);
            visibility = "choice_submitted";
        }
    } else if direction == "response" && !scan.choice_result_paths.is_empty() {
        let pending = PROTOCOL_PENDING_CHOICE.lock()
            .map_err(|_| "pending_protocol_choice_lock_poisoned".to_string())?.take();
        if let Some(choice) = pending {
            visibility = "post_choice_pre_ui";
            linked_request_id = Some(choice.request_id);
            linked_choice_index = Some(choice.choice_index);
        } else {
            visibility = "received_without_observed_choice_request";
        }
    }

    let decode_error = scan.decode_error.as_ref()
        .map(|value| format!("\"{}\"", json_escape(value))).unwrap_or_else(|| "null".to_string());
    let payload_json = format!(
        r#"{{"direction":"{}","request_id":{},"url":"{}","relative_base":"{}","payload_length":{},"decoder":"messagepack_recursive","decode_error":{},"sections":{{"turn_panel":{{"present":{},"paths":[{}]}},"event_declaration":{{"present":{},"paths":[{}]}},"choice_prompt":{{"present":{},"paths":[{}]}},"choice_result":{{"present":{},"paths":[{}]}},"training_home_info":{{"present":{},"paths":[{}]}}}},"choice_index":{},"story_id":{},"event_id":{},"linked_choice_request_id":{},"linked_choice_index":{},"visibility":"{}","all_sections_are_nonexclusive":true}}"#,
        json_escape(direction), request_id, json_escape(url), json_escape(relative_base), payload.len(), decode_error,
        !scan.turn_panel_paths.is_empty(), string_array_json(&scan.turn_panel_paths),
        !scan.event_paths.is_empty(), string_array_json(&scan.event_paths),
        !scan.choice_prompt_paths.is_empty(), string_array_json(&scan.choice_prompt_paths),
        !scan.choice_result_paths.is_empty(), string_array_json(&scan.choice_result_paths),
        !scan.training_paths.is_empty(), string_array_json(&scan.training_paths),
        optional_i64_json(scan.choice_index), optional_i64_json(scan.story_id), optional_i64_json(scan.event_id),
        optional_i64_json(linked_request_id.map(|value| value as i64)), optional_i64_json(linked_choice_index),
        visibility
    );
    append_global_observation("protocol_multisection", "complete", &payload_json, true).map(|_| ())
}

'''
replace_once(storage_anchor, rust + storage_anchor, "multisection_parser")

old_commit = '''    persist_protocol_observation_boundary(
        direction, request_id, url, &relative_base, headers.len(), payload.len()
    )?;
    storage_clear_error();
'''
new_commit = '''    persist_protocol_observation_boundary(
        direction, request_id, url, &relative_base, headers.len(), payload.len()
    )?;
    persist_protocol_semantic_timeline(direction, request_id, url, &relative_base, payload)?;
    storage_clear_error();
'''
replace_once(old_commit, new_commit, "semantic_timeline_call")

# 完整响应/请求解析不能继续继承旧65536字节切片；原始byte[]按其实际长度读取。
old_array = '''    if len == 0 || len > 2 * 1024 * 1024 {
        return vec![];
    }
    let cap = len.min(65536);
    let data_ptr = (arr as *const u8).offset(32);
    std::slice::from_raw_parts(data_ptr, cap).to_vec()
'''
new_array = '''    if len == 0 {
        return vec![];
    }
    let data_ptr = (arr as *const u8).offset(32);
    std::slice::from_raw_parts(data_ptr, len).to_vec()
'''
replace_once(old_array, new_array, "full_il2cpp_byte_array")

anchor = "/// 辅助函数：IL2CPP类型枚举转可读名称\n"
replace_once(anchor, MARKER + "\n" + anchor, "o_marker")
SOURCE.write_text(s, encoding="utf-8")
print("protocol_multisection_event_timeline=applied")
