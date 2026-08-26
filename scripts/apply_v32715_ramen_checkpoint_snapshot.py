from pathlib import Path

p = Path('hachimi_ura_plugin/src/lib.rs')
s = p.read_text(encoding='utf-8')
marker = '// ===== v3.27.15 complete ramen runtime snapshot ====='
if marker in s:
    print('ramen_runtime_snapshot=already_applied')
    raise SystemExit(0)

route_candidates = [
    '    } else if path == "/debug/home_training_gains" {\n',
    '    } else if path == "/debug/ramengains" {\n',
]
route = next((x for x in route_candidates if s.count(x) == 1), None)
assert route is not None, 'no route anchor for complete ramen runtime snapshot'
s = s.replace(route, '''    } else if path == "/debug/ramen_checkpoint" {
        ramen_checkpoint_snapshot_v32715()
    } else if path == "/debug/ramen_runtime" {
        ramen_runtime_snapshot_v32715()
    } else if path == "/debug/ramen_transitions_v2" {
        ramen_runtime_transitions_v32715()
    } else if path == "/training/result" {
        training_result_honest_v32715()
''' + route, 1)

anchor = '// ===== v3.27.15 home training gains endpoint =====\n'
assert s.count(anchor) == 1, f'function anchor count={s.count(anchor)}'

rust = r'''// ===== v3.27.15 complete ramen runtime snapshot =====
fn ramen_snapshot_hex_decode(text: &str) -> Option<Vec<u8>> {
    if text.len() % 2 != 0 { return None; }
    let mut out = Vec::with_capacity(text.len() / 2);
    let bytes = text.as_bytes();
    for i in (0..bytes.len()).step_by(2) {
        let hi = (bytes[i] as char).to_digit(16)?;
        let lo = (bytes[i + 1] as char).to_digit(16)?;
        out.push(((hi << 4) | lo) as u8);
    }
    Some(out)
}

fn ramen_snapshot_map_field<'a>(value: &'a rmpv::Value, name: &str) -> Option<&'a rmpv::Value> {
    match value {
        rmpv::Value::Map(fields) => fields.iter().find_map(|(key, value)| match key {
            rmpv::Value::String(text) if text.as_str() == Some(name) => Some(value),
            _ => None,
        }),
        _ => None,
    }
}

fn ramen_snapshot_find_field<'a>(value: &'a rmpv::Value, name: &str) -> Option<&'a rmpv::Value> {
    if let Some(found) = ramen_snapshot_map_field(value, name) { return Some(found); }
    match value {
        rmpv::Value::Map(fields) => fields.iter().find_map(|(_, value)| ramen_snapshot_find_field(value, name)),
        rmpv::Value::Array(values) => values.iter().find_map(|value| ramen_snapshot_find_field(value, name)),
        _ => None,
    }
}

fn ramen_snapshot_i64(value: Option<&rmpv::Value>) -> Option<i64> {
    value.and_then(|value| match value {
        rmpv::Value::Integer(number) => number.as_i64(),
        _ => None,
    })
}

fn ramen_decode_metadata(entry: &SniffMetadata) -> Option<rmpv::Value> {
    let bytes = ramen_snapshot_hex_decode(&entry.body_hex)?;
    let mut cursor = std::io::Cursor::new(&bytes);
    let decoded = rmpv::decode::read_value(&mut cursor).ok()?;
    if cursor.position() != bytes.len() as u64 { return None; }
    Some(decoded)
}

fn ramen_selected_runtime_json(decoded: &rmpv::Value) -> Option<String> {
    let data = ramen_snapshot_map_field(decoded, "data")?;
    let chara = ramen_snapshot_map_field(data, "chara_info");
    let ramen = ramen_snapshot_map_field(data, "ramen_data_set")
        .or_else(|| ramen_snapshot_map_field(data, "ramen_data_set_check_event"));
    let home = ramen_snapshot_map_field(data, "home_info");
    if chara.is_none() && ramen.is_none() && home.is_none() { return None; }
    Some(format!(
        r#"{{"chara_info":{},"ramen_data_set":{},"home_info":{}}}"#,
        chara.map(turn_event_msgpack_json).unwrap_or_else(|| "null".to_string()),
        ramen.map(turn_event_msgpack_json).unwrap_or_else(|| "null".to_string()),
        home.map(turn_event_msgpack_json).unwrap_or_else(|| "null".to_string()),
    ))
}

fn ramen_checkpoint_snapshot_v32715() -> String {
    let _guard = match SNIFF_MUTEX.lock() {
        Ok(value) => value,
        Err(_) => return r#"{"ok":false,"error":"sniff_lock_poisoned"}"#.to_string(),
    };
    unsafe {
        for entry in SNIFF_METADATA.iter().rev() {
            if entry.direction != "response" || !entry.path.contains("/single_mode_ramen/") { continue; }
            let decoded = match ramen_decode_metadata(entry) { Some(value) => value, None => continue };
            let checkpoint = ramen_snapshot_i64(ramen_snapshot_find_field(&decoded, "check_point_pt"));
            let expected = ramen_snapshot_i64(ramen_snapshot_find_field(&decoded, "expected_check_point_pt"));
            if checkpoint.is_none() && expected.is_none() { continue; }
            return format!(
                r#"{{"ok":true,"schema_version":2,"source":"latest_ramen_protocol_snapshot","observation_id":{},"request_id":{},"path":"{}","check_point_pt":{},"expected_check_point_pt":{}}}"#,
                entry.id, entry.request_id, json_escape(&entry.path),
                checkpoint.map(|value| value.to_string()).unwrap_or_else(|| "null".to_string()),
                expected.map(|value| value.to_string()).unwrap_or_else(|| "null".to_string()),
            );
        }
    }
    r#"{"ok":false,"schema_version":2,"error":"no_ramen_checkpoint_snapshot"}"#.to_string()
}

fn ramen_runtime_snapshot_v32715() -> String {
    let _guard = match SNIFF_MUTEX.lock() {
        Ok(value) => value,
        Err(_) => return r#"{"ok":false,"error":"sniff_lock_poisoned"}"#.to_string(),
    };
    unsafe {
        for entry in SNIFF_METADATA.iter().rev() {
            if entry.direction != "response" || !entry.path.contains("/single_mode_ramen/") { continue; }
            let decoded = match ramen_decode_metadata(entry) { Some(value) => value, None => continue };
            let selected = match ramen_selected_runtime_json(&decoded) { Some(value) => value, None => continue };
            return format!(
                r#"{{"ok":true,"schema_version":1,"source":"latest_complete_ramen_protocol_snapshot","observation_id":{},"request_id":{},"timestamp_ms":{},"path":"{}","runtime":{}}}"#,
                entry.id, entry.request_id, entry.timestamp_ms, json_escape(&entry.path), selected,
            );
        }
    }
    r#"{"ok":false,"schema_version":1,"error":"no_complete_ramen_runtime_snapshot"}"#.to_string()
}

fn ramen_runtime_transitions_v32715() -> String {
    let _guard = match SNIFF_MUTEX.lock() {
        Ok(value) => value,
        Err(_) => return r#"{"ok":false,"error":"sniff_lock_poisoned"}"#.to_string(),
    };
    let mut rows = Vec::new();
    unsafe {
        for entry in SNIFF_METADATA.iter() {
            if entry.direction != "response" || !entry.path.contains("/single_mode_ramen/") { continue; }
            let decoded = match ramen_decode_metadata(entry) { Some(value) => value, None => continue };
            let selected = match ramen_selected_runtime_json(&decoded) { Some(value) => value, None => continue };
            rows.push(format!(
                r#"{{"observation_id":{},"request_id":{},"timestamp_ms":{},"path":"{}","runtime":{}}}"#,
                entry.id, entry.request_id, entry.timestamp_ms, json_escape(&entry.path), selected,
            ));
            if rows.len() > 32 { rows.remove(0); }
        }
    }
    format!(r#"{{"ok":true,"schema_version":1,"source":"protocol_snapshot_timeline","count":{},"observations":[{}]}}"#, rows.len(), rows.join(","))
}

fn training_result_honest_v32715() -> String {
    let state = match ACTION_STATE.lock() {
        Ok(value) => value,
        Err(_) => return r#"{"ok":false,"error":"action_state_lock_poisoned"}"#.to_string(),
    };
    format!(
        r#"{{"ok":true,"schema_version":2,"source":"legacy_training_hook","semantic_verified":false,"result_name":"unknown","raw_result_type":{},"raw_sub_id":{},"raw_command_id":{},"sequence":{},"warning":"legacy hook values are not mapped to success or failure; use protocol snapshots and actual stat deltas"}}"#,
        state.training_result, state.training_sub_id, state.command_id, state.sequence,
    )
}

'''
s = s.replace(anchor, rust + anchor, 1)
p.write_text(s, encoding='utf-8')
print('ramen_runtime_snapshot=applied')
