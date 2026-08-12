from pathlib import Path

SOURCE = Path("hachimi_ura_plugin/src/lib.rs")
CARGO = Path("hachimi_ura_plugin/Cargo.toml")
MARKER = "// ===== Ordered turn and event JSON export C1 ====="

cargo = CARGO.read_text(encoding="utf-8")
if 'rmpv = ' not in cargo:
    anchor = 'libc = "0.2"\n'
    assert cargo.count(anchor) == 1, f"rmpv dependency anchor count={cargo.count(anchor)}"
    cargo = cargo.replace(anchor, anchor + 'rmpv = "1"\n', 1)
    CARGO.write_text(cargo, encoding="utf-8")

s = SOURCE.read_text(encoding="utf-8")
if MARKER in s:
    print("turn_event_json_export=already_applied")
    raise SystemExit(0)

anchor = "/// 辅助函数：IL2CPP类型枚举转可读名称\n"
assert s.count(anchor) == 1, f"turn-event insertion anchor count={s.count(anchor)}"

rust = r'''// ===== Ordered turn and event JSON export C1 =====
fn turn_event_msgpack_json(value: &rmpv::Value) -> String {
    match value {
        rmpv::Value::Nil => "null".to_string(),
        rmpv::Value::Boolean(value) => value.to_string(),
        rmpv::Value::Integer(value) => value.to_string(),
        rmpv::Value::F32(value) => {
            if value.is_finite() { value.to_string() } else { "null".to_string() }
        }
        rmpv::Value::F64(value) => {
            if value.is_finite() { value.to_string() } else { "null".to_string() }
        }
        rmpv::Value::String(value) => format!("\"{}\"", json_escape(value.as_str().unwrap_or(""))),
        rmpv::Value::Binary(value) => format!(
            r#"{{"messagepack_type":"binary","body_hex":"{}"}}"#,
            hex_encode(value),
        ),
        rmpv::Value::Array(values) => format!(
            "[{}]",
            values.iter().map(turn_event_msgpack_json).collect::<Vec<_>>().join(","),
        ),
        rmpv::Value::Map(values) => {
            let fields = values.iter().map(|(key, value)| {
                let key_text = match key {
                    rmpv::Value::String(text) => text.as_str().unwrap_or("").to_string(),
                    _ => turn_event_msgpack_json(key),
                };
                format!("\"{}\":{}", json_escape(&key_text), turn_event_msgpack_json(value))
            }).collect::<Vec<_>>();
            format!("{{{}}}", fields.join(","))
        }
        rmpv::Value::Ext(kind, value) => format!(
            r#"{{"messagepack_type":"ext","ext_type":{},"body_hex":"{}"}}"#,
            kind, hex_encode(value),
        ),
    }
}

fn turn_event_map_field<'a>(value: &'a rmpv::Value, name: &str) -> Option<&'a rmpv::Value> {
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

fn turn_event_selected_data(value: &rmpv::Value) -> Option<String> {
    let data = turn_event_map_field(value, "data")?;
    let fields = match data {
        rmpv::Value::Map(fields) => fields,
        _ => return None,
    };
    let has_chara = fields.iter().any(|(key, _)| {
        matches!(key, rmpv::Value::String(text) if text.as_str() == Some("chara_info"))
    });
    let has_data_set = fields.iter().any(|(key, _)| {
        matches!(key, rmpv::Value::String(text) if text.as_str().map(|value| value.ends_with("_data_set")).unwrap_or(false))
    });
    if !has_chara || !has_data_set {
        return None;
    }
    let selected = fields.iter().filter_map(|(key, value)| {
        let name = match key {
            rmpv::Value::String(text) => text.as_str()?,
            _ => return None,
        };
        if name == "chara_info" || name == "unchecked_event_array" || name.ends_with("_data_set") {
            Some(format!("\"{}\":{}", json_escape(name), turn_event_msgpack_json(value)))
        } else {
            None
        }
    }).collect::<Vec<_>>();
    Some(format!("{{{}}}", selected.join(",")))
}

fn storage_turn_event_jsons(uri: &str) -> String {
    let pairs = match parse_query_pairs(uri) {
        Ok(value) => value,
        Err(error) => return format!(r#"{{"ok":false,"error":"{}"}}"#, json_escape(&error)),
    };
    let session_id = query_pair(&pairs, "session_id");
    if session_id.is_empty() {
        return r#"{"ok":false,"error":"missing_session_id"}"#.to_string();
    }
    let after_sequence = if query_pair(&pairs, "after_sequence").is_empty() {
        0i64
    } else {
        match query_pair(&pairs, "after_sequence").parse::<i64>() {
            Ok(value) if value >= 0 => value,
            _ => return r#"{"ok":false,"error":"invalid_after_sequence"}"#.to_string(),
        }
    };
    let limit = if query_pair(&pairs, "limit").is_empty() {
        100usize
    } else {
        match query_pair(&pairs, "limit").parse::<usize>() {
            Ok(value) if value >= 1 && value <= 500 => value,
            _ => return r#"{"ok":false,"error":"invalid_limit"}"#.to_string(),
        }
    };
    let connection = match open_observation_storage() {
        Ok(value) => value,
        Err(error) => return format!(r#"{{"ok":false,"error":"{}"}}"#, json_escape(&error)),
    };
    let exists = connection.query_row(
        "SELECT 1 FROM observation_sessions WHERE session_id=?1",
        rusqlite::params![session_id],
        |_| Ok(()),
    );
    if matches!(exists, Err(rusqlite::Error::QueryReturnedNoRows)) {
        return r#"{"ok":false,"error":"session_not_found"}"#.to_string();
    }
    if let Err(error) = exists {
        return format!(r#"{{"ok":false,"error":"query_session:{}"}}"#, json_escape(&error.to_string()));
    }
    let mut statement = match connection.prepare(
        "SELECT file_id,relative_path,created_at_ms FROM observation_files \
         WHERE session_id=?1 AND relative_path LIKE 'protocol/response/%/payload.bin' \
         ORDER BY created_at_ms ASC,file_id ASC"
    ) {
        Ok(value) => value,
        Err(error) => return format!(r#"{{"ok":false,"error":"prepare_responses:{}"}}"#, json_escape(&error.to_string())),
    };
    let rows = match statement.query_map(rusqlite::params![session_id], |row| {
        Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?, row.get::<_, i64>(2)?))
    }) {
        Ok(value) => value,
        Err(error) => return format!(r#"{{"ok":false,"error":"query_responses:{}"}}"#, json_escape(&error.to_string())),
    };
    let session_root = observation_storage_root().join("sessions").join(&session_id);
    let mut matched_sequence = 0i64;
    let mut emitted = Vec::new();
    let mut scanned_responses = 0usize;
    let mut decode_failures = Vec::new();
    for row in rows {
        let (file_id, relative_path, response_timestamp_ms) = match row {
            Ok(value) => value,
            Err(error) => {
                decode_failures.push(format!(r#"{{"file_id":null,"error":"row:{}"}}"#, json_escape(&error.to_string())));
                continue;
            }
        };
        scanned_responses += 1;
        let bytes = match std::fs::read(session_root.join(&relative_path)) {
            Ok(value) => value,
            Err(error) => {
                decode_failures.push(format!(r#"{{"file_id":{},"error":"read:{}"}}"#, file_id, json_escape(&error.to_string())));
                continue;
            }
        };
        let mut cursor = std::io::Cursor::new(&bytes);
        let decoded = match rmpv::decode::read_value(&mut cursor) {
            Ok(value) => value,
            Err(error) => {
                decode_failures.push(format!(r#"{{"file_id":{},"error":"decode:{}"}}"#, file_id, json_escape(&error.to_string())));
                continue;
            }
        };
        if cursor.position() != bytes.len() as u64 {
            decode_failures.push(format!(r#"{{"file_id":{},"error":"trailing_bytes","decoded_bytes":{},"total_bytes":{}}}"#, file_id, cursor.position(), bytes.len()));
            continue;
        }
        let selected_data = match turn_event_selected_data(&decoded) {
            Some(value) => value,
            None => continue,
        };
        matched_sequence += 1;
        if matched_sequence <= after_sequence {
            continue;
        }
        if emitted.len() >= limit {
            break;
        }
        let request_id = relative_path.strip_prefix("protocol/response/")
            .and_then(|value| value.split('-').next())
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or(0);
        let url_path = format!("protocol/response/{}/url.txt", relative_path
            .strip_prefix("protocol/response/").and_then(|value| value.split('/').next()).unwrap_or(""));
        let source_url = std::fs::read_to_string(session_root.join(&url_path)).unwrap_or_default();
        emitted.push(format!(
            r#"{{"sequence":{},"request_id":{},"response_timestamp_ms":{},"source_file_id":{},"source_relative_path":"{}","source_url":"{}","data":{}}}"#,
            matched_sequence, request_id, response_timestamp_ms, file_id,
            json_escape(&relative_path), json_escape(&source_url), selected_data,
        ));
    }
    let has_more = emitted.len() == limit;
    let next_sequence = emitted.last().and_then(|item| {
        let prefix = "{\"sequence\":";
        item.strip_prefix(prefix).and_then(|rest| rest.split(',').next()).and_then(|value| value.parse::<i64>().ok())
    }).unwrap_or(after_sequence);
    format!(
        r#"{{"ok":true,"session_id":"{}","ordering":"response_received_at_then_file_id","after_sequence":{},"next_sequence":{},"count":{},"has_more":{},"scanned_responses":{},"decode_failure_count":{},"decode_failures":[{}],"records":[{}]}}"#,
        json_escape(&session_id), after_sequence, next_sequence, emitted.len(), has_more,
        scanned_responses, decode_failures.len(), decode_failures.join(","), emitted.join(","),
    )
}

'''
s = s.replace(anchor, rust + anchor, 1)

boot_anchor = '    "/storage/download",\n'
assert s.count(boot_anchor) == 1, f"turn-event boot anchor count={s.count(boot_anchor)}"
s = s.replace(boot_anchor, boot_anchor + '    "/storage/turn_event_jsons",\n', 1)

route_anchor = '    } else if path == "/storage/files" {\n'
assert s.count(route_anchor) == 1, f"turn-event route anchor count={s.count(route_anchor)}"
s = s.replace(
    route_anchor,
    '''    } else if path == "/storage/turn_event_jsons" {
        storage_turn_event_jsons(&full_uri)
''' + route_anchor,
    1,
)

s = s.replace(anchor, MARKER + "\n" + anchor, 1)
SOURCE.write_text(s, encoding="utf-8")
print("turn_event_json_export=applied")
