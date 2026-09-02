from pathlib import Path

SOURCE = Path("hachimi_ura_plugin/src/lib.rs")
s = SOURCE.read_text(encoding="utf-8")
MARKER = "// ===== Unified pre-release correction F-stage ====="
if MARKER in s:
    print("unified_endpoint_f_pre_release_fix=already_applied")
    raise SystemExit(0)


def replace_once(old: str, new: str, label: str) -> None:
    global s
    count = s.count(old)
    assert count == 1, f"{label} anchor count={count}"
    s = s.replace(old, new, 1)

# /summary identity: +36 is _cardId, while CharaId is a separate computed property.
replace_once(
    '    let chara_id = read_obscured_int_at(chara_obj, 36); // _cardId\n',
    '    let card_id = read_obscured_int_at(chara_obj, 36); // _cardId\n'
    '    let chara_id = call_getter_int(chara_class, chara_obj, "get_CharaId");\n',
    "summary_identity_read",
)
replace_once(
    '\"scenario\":\"{}\",\"chara_id\":{},\"stats\"',
    '\"scenario\":\"{}\",\"card_id\":{},\"chara_id\":{},\"stats\"',
    "summary_identity_json",
)
replace_once(
    '        scn_s,\n        chara_id,\n        spd,\n',
    '        scn_s,\n        card_id,\n        chara_id,\n        spd,\n',
    "summary_identity_args",
)

# Enum endpoint must reject a missing type.
replace_once(
    '    let requested = query_pair(&pairs, "type");\n    let required = ["il2cpp_class_get_fields",',
    '    let requested = query_pair(&pairs, "type");\n'
    '    if requested.is_empty() { return r#"{\\"ok\\":false,\\"error\\":\\"missing_type\\"}"#.to_string(); }\n'
    '    let required = ["il2cpp_class_get_fields",',
    "enum_missing_type",
)

# Recoverable asynchronous MethodIndex. The worker has a generation, progress,
# heartbeat and a bounded build deadline; failed builds can be explicitly retried.
old_state = '''struct MethodIndexState {
    status: &'static str,
    error: String,
    entries: Vec<MethodIndexEntry>,
    image_class_count: u32,
    indexed_class_count: u32,
    indexed_method_count: usize,
    null_method_pointer_count: usize,
    duplicate_method_pointer_count: usize,
}

static METHOD_INDEX: Mutex<MethodIndexState> = Mutex::new(MethodIndexState {
    status: "empty",
    error: String::new(),
    entries: Vec::new(),
    image_class_count: 0,
    indexed_class_count: 0,
    indexed_method_count: 0,
    null_method_pointer_count: 0,
    duplicate_method_pointer_count: 0,
});
'''
new_state = '''struct MethodIndexState {
    status: &'static str,
    error: String,
    entries: Vec<MethodIndexEntry>,
    image_class_count: u32,
    indexed_class_count: u32,
    indexed_method_count: usize,
    null_method_pointer_count: usize,
    duplicate_method_pointer_count: usize,
    generation: u64,
    started_at_ms: u64,
    heartbeat_at_ms: u64,
    worker_active: bool,
}

static METHOD_INDEX: Mutex<MethodIndexState> = Mutex::new(MethodIndexState {
    status: "empty",
    error: String::new(),
    entries: Vec::new(),
    image_class_count: 0,
    indexed_class_count: 0,
    indexed_method_count: 0,
    null_method_pointer_count: 0,
    duplicate_method_pointer_count: 0,
    generation: 0,
    started_at_ms: 0,
    heartbeat_at_ms: 0,
    worker_active: false,
});

fn method_index_now_ms() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default().as_millis() as u64
}
'''
replace_once(old_state, new_state, "method_index_state")
replace_once(
    'unsafe fn build_method_index() -> Result<Vec<MethodIndexEntry>, String> {',
    'unsafe fn build_method_index(generation: u64) -> Result<Vec<MethodIndexEntry>, String> {',
    "method_index_build_signature",
)
replace_once(
    '    let class_count = get_class_count(image);\n    for class_index in 0..class_count {\n',
    '    let class_count = get_class_count(image);\n'
    '    {\n'
    '        let mut state = METHOD_INDEX.lock().unwrap_or_else(|error| error.into_inner());\n'
    '        if state.generation != generation { return Err("method_index_generation_superseded".to_string()); }\n'
    '        state.image_class_count = class_count;\n'
    '        state.heartbeat_at_ms = method_index_now_ms();\n'
    '    }\n'
    '    for class_index in 0..class_count {\n'
    '        if class_index % 32 == 0 {\n'
    '            let now = method_index_now_ms();\n'
    '            let mut state = METHOD_INDEX.lock().unwrap_or_else(|error| error.into_inner());\n'
    '            if state.generation != generation { return Err("method_index_generation_superseded".to_string()); }\n'
    '            if now.saturating_sub(state.started_at_ms) > 180_000 { return Err("method_index_build_timeout".to_string()); }\n'
    '            state.indexed_class_count = class_index;\n'
    '            state.indexed_method_count = entries.len();\n'
    '            state.heartbeat_at_ms = now;\n'
    '        }\n',
    "method_index_progress",
)
old_ensure_start = s.index('unsafe fn ensure_method_index() -> Result<(), String> {')
old_ensure_end = s.index('\nfn method_entry_json(', old_ensure_start)
old_ensure = s[old_ensure_start:old_ensure_end]
new_ensure = '''unsafe fn ensure_method_index() -> Result<(), String> {
    let generation;
    {
        let mut state = METHOD_INDEX.lock().unwrap_or_else(|error| error.into_inner());
        match state.status {
            "ready" => return Ok(()),
            "building" => return Err("method_index_building".to_string()),
            "failed" => return Err(if state.error.is_empty() { "method_index_failed".to_string() } else { state.error.clone() }),
            _ => {}
        }
        state.generation = state.generation.saturating_add(1);
        generation = state.generation;
        state.status = "building";
        state.error.clear();
        state.entries.clear();
        state.image_class_count = 0;
        state.indexed_class_count = 0;
        state.indexed_method_count = 0;
        state.started_at_ms = method_index_now_ms();
        state.heartbeat_at_ms = state.started_at_ms;
        state.worker_active = true;
    }
    let spawn = std::thread::Builder::new()
        .name(format!("hlpatch-method-index-{}", generation))
        .spawn(move || {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
                build_method_index(generation)
            })).map_err(|_| "method_index_worker_panic".to_string()).and_then(|value| value);
            let mut state = METHOD_INDEX.lock().unwrap_or_else(|error| error.into_inner());
            if state.generation != generation { return; }
            state.worker_active = false;
            state.heartbeat_at_ms = method_index_now_ms();
            match result {
                Ok(entries) => {
                    let null_count = entries.iter().filter(|entry| entry.method_pointer == 0).count();
                    let mut duplicate_count = 0usize;
                    let mut previous = 0usize;
                    for entry in entries.iter().filter(|entry| entry.method_pointer != 0) {
                        if entry.method_pointer == previous { duplicate_count += 1; }
                        previous = entry.method_pointer;
                    }
                    state.indexed_class_count = state.image_class_count;
                    state.indexed_method_count = entries.len();
                    state.null_method_pointer_count = null_count;
                    state.duplicate_method_pointer_count = duplicate_count;
                    state.entries = entries;
                    state.error.clear();
                    state.status = "ready";
                }
                Err(error) => {
                    state.status = "failed";
                    state.error = error;
                }
            }
        });
    if let Err(error) = spawn {
        let mut state = METHOD_INDEX.lock().unwrap_or_else(|poison| poison.into_inner());
        if state.generation == generation {
            state.worker_active = false;
            state.status = "failed";
            state.error = format!("method_index_spawn_failed:{}", error);
        }
        return Err("method_index_spawn_failed".to_string());
    }
    Err("method_index_building".to_string())
}

fn method_index_status_endpoint(uri: &str) -> String {
    let retry = parse_query_pairs(uri).ok().map(|pairs| query_pair(&pairs, "retry") == "1").unwrap_or(false);
    if retry {
        let mut state = METHOD_INDEX.lock().unwrap_or_else(|error| error.into_inner());
        if state.status == "failed" && !state.worker_active {
            state.status = "empty";
            state.error.clear();
        }
    }
    if retry { let _ = unsafe { ensure_method_index() }; }
    let state = METHOD_INDEX.lock().unwrap_or_else(|error| error.into_inner());
    let now = method_index_now_ms();
    format!(r#"{{"ok":true,"status":"{}","generation":{},"worker_active":{},"started_at_ms":{},"heartbeat_at_ms":{},"heartbeat_age_ms":{},"classes_total":{},"classes_indexed":{},"methods_indexed":{},"error":{}}}"#,
        state.status, state.generation, state.worker_active, state.started_at_ms, state.heartbeat_at_ms,
        now.saturating_sub(state.heartbeat_at_ms), state.image_class_count, state.indexed_class_count,
        state.indexed_method_count, if state.error.is_empty() { "null".to_string() } else { format!("\\\"{}\\\"", json_escape(&state.error)) })
}
'''
s = s[:old_ensure_start] + new_ensure + s[old_ensure_end:]

# Persistent raw protocol files. Every captured byte is written before the in-memory ring can evict it.
storage_anchor = '''fn storage_set_error(error: &str) {
    if let Ok(mut value) = STORAGE_LAST_ERROR.lock() {
        *value = Some(error.to_string());
    }
}
'''
storage_new = storage_anchor + '''
fn storage_clear_error() {
    if let Ok(mut value) = STORAGE_LAST_ERROR.lock() { *value = None; }
}

fn persist_protocol_capture(direction: &str, request_id: u64, url: &str, headers: &[u8], payload: &[u8]) -> Result<(), String> {
    let session_id = ensure_observation_session()?;
    let now = sniff_timestamp_ms();
    let suffix = if direction == "response" { format!("{}-{}", request_id, now) } else { request_id.to_string() };
    let relative_base = format!("protocol/{}/{}", direction, suffix);
    let session_dir = observation_storage_root().join("sessions").join(&session_id);
    let target_dir = session_dir.join(&relative_base);
    std::fs::create_dir_all(&target_dir).map_err(|error| format!("create_protocol_dir:{}", error))?;
    let files: [(&str, &[u8], &str); 3] = [
        ("url.txt", url.as_bytes(), "text/plain; charset=utf-8"),
        ("headers.raw", headers, "application/octet-stream"),
        ("payload.bin", payload, "application/octet-stream"),
    ];
    for (name, bytes, _) in &files {
        let temporary = target_dir.join(format!("{}.tmp", name));
        std::fs::write(&temporary, bytes).map_err(|error| format!("write_protocol_file:{}:{}", name, error))?;
        std::fs::rename(&temporary, target_dir.join(name)).map_err(|error| format!("commit_protocol_file:{}:{}", name, error))?;
    }
    let mut connection = open_observation_storage()?;
    let transaction = connection.transaction().map_err(|error| format!("protocol_index_transaction:{}", error))?;
    for (name, bytes, content_type) in &files {
        let relative = format!("{}/{}", relative_base, name);
        transaction.execute(
            "INSERT OR REPLACE INTO observation_files(session_id, relative_path, content_type, byte_length, sha256, created_at_ms) VALUES(?1, ?2, ?3, ?4, NULL, ?5)",
            rusqlite::params![session_id, relative, content_type, bytes.len() as i64, now as i64],
        ).map_err(|error| format!("index_protocol_file:{}:{}", name, error))?;
    }
    transaction.commit().map_err(|error| format!("commit_protocol_index:{}", error))?;
    storage_clear_error();
    Ok(())
}
'''
replace_once(storage_anchor, storage_new, "storage_helpers")
replace_once(
    '                SNIFF_RESPONSES.push((rid, bytes));\n',
    '                if let Err(error) = persist_protocol_capture("response", rid, &response_url, &[], &bytes) { storage_set_error(&error); }\n'
    '                SNIFF_RESPONSES.push((rid, bytes));\n',
    "persist_response",
)
replace_once(
    '                SNIFF_REQUESTS.push((rid, url_str, headers_json, body));\n',
    '                if let Err(error) = persist_protocol_capture("request", rid, &url_str, headers_json.as_bytes(), &body) { storage_set_error(&error); }\n'
    '                SNIFF_REQUESTS.push((rid, url_str, headers_json, body));\n',
    "persist_request",
)

# Never silently discard malformed session rows.
session_rows_old = '    let sessions: Vec<String> = rows.filter_map(Result::ok).collect();\n'
session_rows_new = '''    let mut sessions = Vec::new();
    for row in rows {
        match row {
            Ok(value) => sessions.push(value),
            Err(error) => {
                let detail = format!("decode_session_row:{}", error);
                storage_set_error(&detail);
                return format!(r#"{{"ok":false,"error":"{}","sessions":[]}}"#, json_escape(&detail));
            }
        }
    }
    storage_clear_error();
'''
replace_once(session_rows_old, session_rows_new, "sessions_rows")
# Checkpoint must succeed before publishing a new flush timestamp.
replace_once(
    '''    if let Err(error) = connection.execute(
        "UPDATE observation_sessions SET last_flush_ms=?1 WHERE session_id=?2",
        rusqlite::params![now as i64, session_id],
    ) {
        return format!(r#"{{"ok":false,"error":"update_flush:{}"}}"#, json_escape(&error.to_string()));
    }
    if let Err(error) = connection.execute_batch("PRAGMA wal_checkpoint(FULL);") {
        return format!(r#"{{"ok":false,"error":"checkpoint:{}"}}"#, json_escape(&error.to_string()));
    }
    STORAGE_LAST_FLUSH_MS.store(now, Ordering::Relaxed);
''',
    '''    if let Err(error) = connection.execute_batch("PRAGMA wal_checkpoint(FULL);") {
        let detail = format!("checkpoint:{}", error); storage_set_error(&detail);
        return format!(r#"{{"ok":false,"error":"{}"}}"#, json_escape(&detail));
    }
    if let Err(error) = connection.execute(
        "UPDATE observation_sessions SET last_flush_ms=?1 WHERE session_id=?2",
        rusqlite::params![now as i64, session_id],
    ) {
        let detail = format!("update_flush:{}", error); storage_set_error(&detail);
        return format!(r#"{{"ok":false,"error":"{}"}}"#, json_escape(&detail));
    }
    STORAGE_LAST_FLUSH_MS.store(now, Ordering::Relaxed);
    storage_clear_error();
''',
    "flush_order",
)

# Pair query contract: duplicate keys are errors and both characters must exist in MDB.
replace_once(
    '    let chara_id_a = match query_pair(&pairs, "chara_id_a").parse::<i32>() {\n',
    '    if pairs.iter().filter(|(key, _)| key == "chara_id_a").count() != 1 || pairs.iter().filter(|(key, _)| key == "chara_id_b").count() != 1 { return r#"{\\"ok\\":false,\\"error\\":\\"missing_or_duplicate_character_key\\"}"#.to_string(); }\n'
    '    let chara_id_a = match query_pair(&pairs, "chara_id_a").parse::<i32>() {\n',
    "pair_duplicate_keys",
)
replace_once(
    '    let mut statement = match connection.prepare(\n        "SELECT DISTINCT r.relation_type, r.relation_point\n',
    '    for (label, value) in [("chara_id_a", chara_id_a), ("chara_id_b", chara_id_b)] {\n'
    '        let exists = connection.query_row("SELECT EXISTS(SELECT 1 FROM chara_data WHERE id=?1)", rusqlite::params![value], |row| row.get::<_, i64>(0));\n'
    '        match exists { Ok(1) => {}, Ok(_) => return format!(r#"{{\\"ok\\":false,\\"error\\":\\"character_not_found\\",\\"field\\":\\"{}\\",\\"value\\":{}}}"#, label, value), Err(error) => return format!(r#"{{\\"ok\\":false,\\"error\\":\\"character_validation_failed\\",\\"detail\\":\\"{}\\"}}"#, json_escape(&error.to_string())) }\n'
    '    }\n'
    '    let mut statement = match connection.prepare(\n        "SELECT DISTINCT r.relation_type, r.relation_point\n',
    "pair_character_validation",
)

# Runtime endpoint describes what this invocation actually did.
replace_once(
    '\"getter_decode\":\"existing_runtime_invoke_int_path\",\"runtime_validation\":\"pending_device_execution\"',
    '\"getter_decode\":\"obscured_int_runtime_invoke_path\",\"runtime_validation\":\"executed\"',
    "selected_parent_runtime_status",
)

# Unsafe legacy analyses remain addressable but can no longer claim valid results.
replace_once(
    '''    } else if path == "/inherit/compat" {
        unsafe { read_inherit_compat() }
    } else if path == "/saddle-analysis" {
        unsafe { read_win_saddle_analysis() }
''',
    '''    } else if path == "/inherit/compat" {
        r#"{"ok":false,"status":"deprecated","error":"legacy_inherit_contract_unreliable","replacement":"/inherit/pair_compat and /inherit/selected_parent_runtime"}"#.to_string()
    } else if path == "/saddle-analysis" {
        r#"{"ok":false,"status":"unavailable","error":"legacy_saddle_runtime_chain_unverified"}"#.to_string()
''',
    "legacy_route_degrade",
)

# Add the new status route and advertise every A-F route in both existing lists.
replace_once(
    '    } else if path == "/il2cpp/method_by_addr" {\n',
    '    } else if path == "/il2cpp/method_index_status" {\n        method_index_status_endpoint(&full_uri)\n'
    '    } else if path == "/il2cpp/method_by_addr" {\n',
    "method_index_status_route",
)
for endpoint in [
    "/storage/status", "/storage/sessions", "/storage/session", "/storage/flush", "/storage/recover",
    "/il2cpp/method_index_status", "/il2cpp/method_by_addr", "/il2cpp/method_detail",
    "/il2cpp/nested_types", "/il2cpp/enum_values", "/inherit/pair_compat", "/inherit/selected_parent_runtime",
]:
    token = f'"{endpoint}"'
    if token not in s:
        raise AssertionError(f"route token absent before advertisement: {endpoint}")
new_advertised = '"/storage/status","/storage/sessions","/storage/session","/storage/flush","/storage/recover","/il2cpp/method_index_status","/il2cpp/method_by_addr","/il2cpp/method_detail","/il2cpp/nested_types","/il2cpp/enum_values","/inherit/pair_compat","/inherit/selected_parent_runtime",'
health_needle = 'r#"{{"status":"ok","version":"{}","endpoints":['
assert s.count(health_needle) == 1, s.count(health_needle)
s = s.replace(health_needle, health_needle + new_advertised, 1)
available_needle = 'r#"{{"error":"not_found","path":"{}","available":['
assert s.count(available_needle) == 1, s.count(available_needle)
s = s.replace(available_needle, available_needle + new_advertised, 1)

anchor = "/// 辅助函数：IL2CPP类型枚举转可读名称\n"
replace_once(anchor, MARKER + "\n" + anchor, "f_marker")
SOURCE.write_text(s, encoding="utf-8")
print("unified_endpoint_f_pre_release_fix=applied")
