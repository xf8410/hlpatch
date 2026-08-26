from pathlib import Path

p = Path('hachimi_ura_plugin/src/lib.rs')
s = p.read_text(encoding='utf-8')
marker = '// ===== v3.27.15 current ramen checkpoint snapshot ====='
if marker in s:
    print('ramen_checkpoint_snapshot=already_applied')
    raise SystemExit(0)

route_candidates = [
    '    } else if path == "/debug/home_training_gains" {\n',
    '    } else if path == "/debug/ramengains" {\n',
]
route = next((x for x in route_candidates if s.count(x) == 1), None)
assert route is not None, 'no route anchor for ramen checkpoint snapshot'
s = s.replace(route, '''    } else if path == "/debug/ramen_checkpoint" {
        ramen_checkpoint_snapshot_v32715()
''' + route, 1)

anchor = '// ===== v3.27.15 home training gains endpoint =====\n'
assert s.count(anchor) == 1, f'function anchor count={s.count(anchor)}'

rust = r'''// ===== v3.27.15 current ramen checkpoint snapshot =====
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
        rmpv::Value::Map(fields) => fields.iter().find_map(|(key, value)| {
            match key {
                rmpv::Value::String(text) if text.as_str() == Some(name) => Some(value),
                _ => None,
            }
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

fn ramen_checkpoint_snapshot_v32715() -> String {
    let _guard = match SNIFF_MUTEX.lock() {
        Ok(value) => value,
        Err(_) => return r#"{"ok":false,"error":"sniff_lock_poisoned"}"#.to_string(),
    };
    unsafe {
        for entry in SNIFF_METADATA.iter().rev() {
            if entry.direction != "response" || !entry.path.contains("/single_mode_ramen/") {
                continue;
            }
            let bytes = match ramen_snapshot_hex_decode(&entry.body_hex) {
                Some(value) => value,
                None => continue,
            };
            let mut cursor = std::io::Cursor::new(&bytes);
            let decoded = match rmpv::decode::read_value(&mut cursor) {
                Ok(value) if cursor.position() == bytes.len() as u64 => value,
                _ => continue,
            };
            let checkpoint = ramen_snapshot_i64(ramen_snapshot_find_field(&decoded, "check_point_pt"));
            let expected = ramen_snapshot_i64(ramen_snapshot_find_field(&decoded, "expected_check_point_pt"));
            if checkpoint.is_none() && expected.is_none() { continue; }
            return format!(
                r#"{{"ok":true,"schema_version":1,"source":"latest_ramen_protocol_snapshot","observation_id":{},"request_id":{},"path":"{}","check_point_pt":{},"expected_check_point_pt":{}}}"#,
                entry.id,
                entry.request_id,
                json_escape(&entry.path),
                checkpoint.map(|value| value.to_string()).unwrap_or_else(|| "null".to_string()),
                expected.map(|value| value.to_string()).unwrap_or_else(|| "null".to_string()),
            );
        }
    }
    r#"{"ok":false,"schema_version":1,"source":"latest_ramen_protocol_snapshot","error":"no_ramen_checkpoint_snapshot","hint":"perform a ramen action or turn transition while protocol capture is enabled"}"#.to_string()
}

'''
s = s.replace(anchor, rust + anchor, 1)
p.write_text(s, encoding='utf-8')
print('ramen_checkpoint_snapshot=applied')
