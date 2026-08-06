# Unified K implementation anchors

## storage

### `CREATE TABLE IF NOT EXISTS observation_files`
matches=1

```rust
e_path, content_type, byte_length, sha256, created_at_ms) VALUES(?1, ?2, ?3, ?4, NULL, ?5)",
            rusqlite::params![session_id, relative, content_type, bytes.len() as i64, now as i64],
        ).map_err(|error| format!("index_protocol_file:{}:{}", name, error))?;
    }
    transaction.commit().map_err(|error| format!("commit_protocol_index:{}", error))?;
    storage_clear_error();
    Ok(())
}

fn observation_storage_root() -> std::path::PathBuf {
    if let Ok(command_line) = std::fs::read("/proc/self/cmdline") {
        let package_bytes = command_line.split(|byte| *byte == 0).next().unwrap_or(&[]);
        if let Ok(package_name) = std::str::from_utf8(package_bytes) {
            if !package_name.is_empty() {
                return std::path::PathBuf::from("/data/user/0")
                    .join(package_name)
                    .join("files")
                    .join("hlpatch-observations");
            }
        }
    }
    std::path::PathBuf::from("/data/user/0/jp.co.cygames.umamusume/files/hlpatch-observations")
}

fn observation_storage_db_path() -> std::path::PathBuf {
    observation_storage_root().join("index.sqlite")
}

fn open_observation_storage() -> Result<Connection, String> {
    let root = observation_storage_root();
    std::fs::create_dir_all(root.join("sessions")).map_err(|error| format!("create_sessions_dir:{}", error))?;
    std::fs::create_dir_all(root.join("blobs")).map_err(|error| format!("create_blobs_dir:{}", error))?;
    let db_path = observation_storage_db_path();
    let connection = Connection::open_with_flags(
        &db_path,
        OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE,
    ).map_err(|error| format!("open_index:{}", error))?;
    connection.execute_batch(
        "PRAGMA journal_mode=WAL;
         PRAGMA synchronous=FULL;
         PRAGMA foreign_keys=ON;
         CREATE TABLE IF NOT EXISTS storage_meta(
             key TEXT PRIMARY KEY NOT NULL,
             value TEXT NOT NULL
         );
         CREATE TABLE IF NOT EXISTS observation_sessions(
             session_id TEXT PRIMARY KEY NOT NULL,
             process_id INTEGER NOT NULL,
             process_start_token TEXT NOT NULL DEFAULT '',
             plugin_version TEXT NOT NULL,
             started_at_ms INTEGER NOT NULL,
             last_flush_ms INTEGER NOT NULL,
             state TEXT NOT NULL,
             recovered_after_restart INTEGER NOT NULL DEFAULT 0,
             root_path TEXT NOT NULL
         );
         CREATE TABLE IF NOT EXISTS observation_files(
             file_id INTEGER PRIMARY KEY AUTOINCREMENT,
             session_id TEXT NOT NULL,
             relative_path TEXT NOT NULL,
             content_type TEXT NOT NULL,
             byte_length INTEGER NOT NULL,
             sha256 TEXT,
             created_at_ms INTEGER NOT NULL,
             UNIQUE(session_id, relative_path),
             FOREIGN KEY(session_id) REFERENCES observation_sessions(session_id)
         );
         CREATE INDEX IF NOT EXISTS idx_observation_files_session_id_file_id
             ON observation_files(session_id, file_id);"
    ).map_err(|error| format!("initialize_schema:{}", error))?;
    let has_start_token = connection.prepare("PRAGMA table_info(observation_sessions)")
        .and_then(|mut statement| statement.query_map([], |row| row.get::<_, String>(1))
            .map(|rows| rows.filter_map(Result::ok).any(|name| name == "process_start_token")))
        .unwrap_or(false);
    if !has_start_token {
        connection.execute("ALTER TABLE observation_sessions ADD COLUMN process_start_token TEXT NOT NULL DEFAULT ''", [])
            .map_err(|error| format!("migrate_process_start_token:{}", error))?;
    }
    Ok(connection)
}

fn observation_process_start_token() -> String {
    let stat = std::fs::read_to_string("/proc/self/stat").unwrap_or_default();
    let start_ticks = stat.rsplit_once(')').map(|(_, tail)| tail.split_whitespace().nth(19).unwrap_or("")).unwrap_or("");
    format!("{}:{}", std::process::id(), start_ticks)
}

fn ensure_observation_session() -> Result<String, String> {
    if let Ok(value) = STORAGE_SESSION_ID.lock() {
        if let Some(session_id) = value.as_ref() {
            return Ok(session_id.clone());
        }
    }
    let connection = open_observation_storage()?;
    let now = sniff_timestamp_ms();
    let process_id = std::process::id();
    let process_start_token = observation_process_start_token();
    let session_id = format!("{}-{}", now, process_id);
    let root_text = observation_storage_root().to_string_lossy().into_owned();
    connection.execute(
        "UPDATE observation_sessions
         SET state='interrupted', recovered_after_restart=1
         WHERE state='open' AND process_start_token<>?1",
        rusqlite::params![process_start_token],
    ).map_err(|error| format!("recover_previous_sessions:{}", error))?;
    connection.execute(
        "INSERT INTO observation_sessions(
             session_id, process_id, process_start_token, plugin_version, started_at_ms,
             last_flush_ms, state, recovered_after_restart, root_path
         ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, 'open', 0, ?7)",
        rusqlite::params![session_id, process_id as i64, process_start_token, PLUGIN_VERSION, now as i64, now as i64, root_text],
    ).map_err(|error| format!("insert_session:{}", error))?;
    let session_directory = observation_storage_root().join("sessions").join(&session_id);
    if let Err(error) = std::fs::create_dir_all(&session_directory) {
        let _ = connection.execute("DELETE FROM observation_sessions WHERE session_id=?1", rusqlite::params![session_id]);
        return Err(format!("create_session_dir:{}", error));
    }
    let session_json = format!(
        r#"{{"session_id":"{}","process_id":{},"plugin_version":"{}","started_at_ms":{},"state":"open","recovered_after_restart":false,"root_path":"{}"}}"#,
        json_escape(&session_id), process_id, json_escape(PLUGIN_VERSION), now, json_escape(&root_text)
    );
    if let Err(error) = std::fs::write(session_directory.join("session.json"), session_json.as_bytes()) {
        let _ = connection.execute("DELETE FROM observation_sessions WHERE session_id=?1", rusqlite::params![session_id]);
        let _ = std::fs::remove_dir_all(&session_directory);
        return Err(format!("write_session_json:{}", error));
    }
    if let Err(error) = connection.execute(
        "INSERT OR REPLACE INTO observation_files(
             session_id, relative_path, content_type, byte_length, sha256, created_at_ms
         ) VALUES(?1, 'session.json', 'application/json', ?2, NULL, ?3)",
        rusqlite::params![session_id, session_json.as_bytes().len() as i64, now as i64],
    ) {
        let _ = connection.execute("DELETE FROM observation_sessions WHERE session_id=?1", rusqlite::params![session_id]);
        let _ = std::fs::remove_dir_all(&session_directory);
        return Err(format!("index_session_json:{}", error));
    }
    STORAGE_LAST_FLUSH_MS.store(now, Ordering::Relaxed);
    let mut state = STORAGE_SESSION_ID.lock().map_err(|_| "storage_session_lock_poisoned".to_string())?;
    *state = Some(session_id.clone());
    Ok(session_id)
}

fn storage_status_endpoint() -> String {
    let root = observation_storage_root();
    let db_path = observation_storage_db_path();
    let session = ensure_observation_session();
    if let Err(error) = session.as_ref() { storage_set_error(error); }
    let current_session = session.ok();
    let writable_probe_path = root.join(".write_probe");
    let writable = std::fs::write(&writable_probe_path, b"hlpatch-storage-probe")
        .and_then(|_| std::fs::remove_file(&writable_probe_path)).is_ok();
    let error = STORAGE_LAST_ERROR.lock().ok().and_then(|value| value.clone());
    let session_json = current_session.as_ref().map(|value| format!("\"{}\"", json_escape(value))).unwrap_or_else(|| "null".to_string());
    let error_json = error.map(|value| format!("\"{}\"", json_escape(&value))).unwrap_or_else(|| "null".to_string());
    format!(
        r#"{{"ok":{},"schema_version":1,"root_path":"{}","index_path":"{}","writable":{},"current_session_id":{},"last_flush_ms":{},"last_error":{},"storage_format":{{"index":"sqlite","timeline":"ndjson","payloads":"raw_files"}}}}"#,
        writable && current_session.is_some(), json_escape(&root.to_string_lossy()),
        json_escape(&db_path.to_string_lossy()), writable, session_json,
        STORAGE_LAST_FLUSH_MS.load(Ordering::Relaxed), error_json
    )
}

fn storage_sessions_endpoint() -> String {
    if let Err(error) = ensure_observation_session() {
        storage_set_error(&error);
        return format!(r#"{{"ok":false,"error":"{}","sessions":[]}}"#, json_escape(&error));
    }
    let connection = match open_observation_storage() {
        Ok(value) => value,
        Err(error) => return format!(r#"{{"ok":false,"error":"{}","sessions":[]}}"#, json_escape(&error)),
    };
    let mut statement = match connection.prepare(
        "SELECT session_id, process_id, plugin_version, started_at_ms, last_flush_ms,
                state, recovered_after_restart, root_path
         FROM observation_sessions ORDER BY started_at_ms, session_id"
    ) {
        Ok(value) => value,
        Err(error) => return format!(r#"{{"ok":false,"error":"prepare_sessions:{}","sessions":[]}}"#, json_escape(&error.to_string())),
    };
    let rows = match statement.query_map([], |row| {
        Ok(format!(
            r#"{{"session_id"
```

### `fn storage_session_endpoint`
matches=1

```rust
_id":{},"last_flush_ms":{},"last_error":{},"storage_format":{{"index":"sqlite","timeline":"ndjson","payloads":"raw_files"}}}}"#,
        writable && current_session.is_some(), json_escape(&root.to_string_lossy()),
        json_escape(&db_path.to_string_lossy()), writable, session_json,
        STORAGE_LAST_FLUSH_MS.load(Ordering::Relaxed), error_json
    )
}

fn storage_sessions_endpoint() -> String {
    if let Err(error) = ensure_observation_session() {
        storage_set_error(&error);
        return format!(r#"{{"ok":false,"error":"{}","sessions":[]}}"#, json_escape(&error));
    }
    let connection = match open_observation_storage() {
        Ok(value) => value,
        Err(error) => return format!(r#"{{"ok":false,"error":"{}","sessions":[]}}"#, json_escape(&error)),
    };
    let mut statement = match connection.prepare(
        "SELECT session_id, process_id, plugin_version, started_at_ms, last_flush_ms,
                state, recovered_after_restart, root_path
         FROM observation_sessions ORDER BY started_at_ms, session_id"
    ) {
        Ok(value) => value,
        Err(error) => return format!(r#"{{"ok":false,"error":"prepare_sessions:{}","sessions":[]}}"#, json_escape(&error.to_string())),
    };
    let rows = match statement.query_map([], |row| {
        Ok(format!(
            r#"{{"session_id":"{}","process_id":{},"plugin_version":"{}","started_at_ms":{},"last_flush_ms":{},"state":"{}","recovered_after_restart":{},"root_path":"{}"}}"#,
            json_escape(&row.get::<_, String>(0)?), row.get::<_, i64>(1)?,
            json_escape(&row.get::<_, String>(2)?), row.get::<_, i64>(3)?, row.get::<_, i64>(4)?,
            json_escape(&row.get::<_, String>(5)?), row.get::<_, i64>(6)? != 0,
            json_escape(&row.get::<_, String>(7)?)
        ))
    }) {
        Ok(value) => value,
        Err(error) => return format!(r#"{{"ok":false,"error":"query_sessions:{}","sessions":[]}}"#, json_escape(&error.to_string())),
    };
    let mut sessions = Vec::new();
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
    format!(r#"{{"ok":true,"count":{},"sessions":[{}]}}"#, sessions.len(), sessions.join(","))
}

fn storage_session_endpoint(uri: &str) -> String {
    let pairs = match parse_query_pairs(uri) {
        Ok(value) => value,
        Err(error) => return format!(r#"{{"ok":false,"error":"{}"}}"#, json_escape(&error)),
    };
    let session_id = query_pair(&pairs, "id");
    if session_id.is_empty() { return r#"{"ok":false,"error":"missing_id"}"#.to_string(); }
    let connection = match open_observation_storage() {
        Ok(value) => value,
        Err(error) => return format!(r#"{{"ok":false,"error":"{}"}}"#, json_escape(&error)),
    };
    let result = connection.query_row(
        "SELECT process_id, plugin_version, started_at_ms, last_flush_ms,
                state, recovered_after_restart, root_path
         FROM observation_sessions WHERE session_id=?1",
        rusqlite::params![session_id],
        |row| Ok(format!(
            r#"{{"ok":true,"session":{{"session_id":"{}","process_id":{},"plugin_version":"{}","started_at_ms":{},"last_flush_ms":{},"state":"{}","recovered_after_restart":{},"root_path":"{}"}}}}"#,
            json_escape(&session_id), row.get::<_, i64>(0)?, json_escape(&row.get::<_, String>(1)?),
            row.get::<_, i64>(2)?, row.get::<_, i64>(3)?, json_escape(&row.get::<_, String>(4)?),
            row.get::<_, i64>(5)? != 0, json_escape(&row.get::<_, String>(6)?)
        )),
    );
    match result {
        Ok(value) => value,
        Err(rusqlite::Error::QueryReturnedNoRows) => r#"{"ok":true,"session":null,"status":"none"}"#.to_string(),
        Err(error) => format!(r#"{{"ok":false,"error":"query_session:{}"}}"#, json_escape(&error.to_string())),
    }
}

fn storage_flush_endpoint() -> String {
    let session_id = match ensure_observation_session() {
        Ok(value) => value,
        Err(error) => return format!(r#"{{"ok":false,"error":"{}"}}"#, json_escape(&error)),
    };
    let connection = match open_observation_storage() {
        Ok(value) => value,
        Err(error) => return format!(r#"{{"ok":false,"error":"{}"}}"#, json_escape(&error)),
    };
    let now = sniff_timestamp_ms();
    if let Err(error) = connection.execute_batch("PRAGMA wal_checkpoint(FULL);") {
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
    format!(r#"{{"ok":true,"session_id":"{}","last_flush_ms":{},"checkpoint":"full"}}"#, json_escape(&session_id), now)
}

fn storage_recover_endpoint() -> String {
    let connection = match open_observation_storage() {
        Ok(value) => value,
        Err(error) => return format!(r#"{{"ok":false,"error":"{}"}}"#, json_escape(&error)),
    };
    let process_start_token = observation_process_start_token();
    let recovered = match connection.execute(
        "UPDATE observation_sessions
         SET state='interrupted', recovered_after_restart=1
         WHERE state='open' AND process_start_token<>?1",
        rusqlite::params![process_start_token],
    ) {
        Ok(value) => value,
        Err(error) => return format!(r#"{{"ok":false,"error":"recover:{}"}}"#, json_escape(&error.to_string())),
    };
    match ensure_observation_session() {
        Ok(session_id) => format!(r#"{{"ok":true,"recovered_session_count":{},"current_session_id":"{}"}}"#, recovered, json_escape(&session_id)),
        Err(error) => format!(r#"{{"ok":false,"error":"{}","recovered_session_count":{}}}"#, json_escape(&error), recovered),
    }
}

// ===== Unified inheritance pair compatibility C-stage =====
fn inherit_pair_compat_endpoint(uri: &str) -> String {
    let pairs = match parse_query_pairs(uri) {
        Ok(value) => value,
        Err(error) => return format!(r#"{{"ok":false,"error":"{}"}}"#, json_escape(&error)),
    };
    if pairs.iter().filter(|(key, _)| key == "chara_id_a").count() != 1 || pairs.iter().filter(|(key, _)| key == "chara_id_b").count() != 1 { return r#"{\"ok\":false,\"error\":\"missing_or_duplicate_character_key\"}"#.to_string(); }
    let chara_id_a = match query_pair(&pairs, "chara_id_a").parse::<i32>() {
        Ok(value) if value > 0 => value,
        _ => return r#"{"ok":false,"error":"invalid_or_missing_chara_id_a"}"#.to_string(),
    };
    let chara_id_b = match query_pair(&pairs, "chara_id_b").parse::<i32>() {
        Ok(value) if value > 0 => value,
        _ => return r#"{"ok":false,"error":"invalid_or_missing_chara_id_b"}"#.to_string(),
    };
    let mdb_path = match find_mdb_path() {
        Some(value) => value,
        None => return r#"{"ok":false,"error":"mdb_not_found"}"#.to_string(),
    };
    let connection = match Connection::open_with_flags(&mdb_path, OpenFlags::SQLITE_OPEN_READ_ONLY) {
        Ok(value) => value,
        Err(error) => return format!(r#"{{"ok":false,"error":"mdb_open_failed","detail":"{}"}}"#, json_escape(&error.to_string())),
    };
    for (label, value) in [("chara_id_a", chara_id_a), ("chara_id_b", chara_id_b)] {
        let exists = connection.query_row("SELECT EXISTS(SELECT 1 FROM chara_data WHERE id=?1)", rusqlite::params![value], |row| row.get::<_, i64>(0));
        match exists { Ok(1) => {}, Ok(_) => return format!(r#"{{\"ok\":false,\"error\":\"character_not_found\",\"field\":\"{}\",\"value\":{}}}"#, label, value), Err(error) => return format!(r#"{{\"ok\":false,\"error\":\"character_validation_failed\",\"detail\":\"{}\"}}"#, json_escape(&error.to_string())) }
    }
    let mut statement = match connection.prepare(
        "SELECT DISTINCT r.relation_type, r.relation_point
         FROM succession_relation r
         INNER JOIN succession_relation_member a
             ON a.relation_type = r.relation_type AND a.chara_id = ?1
         INNER JOIN succession_relation_member b
             ON b.relation_type = r.relation_type AND b.chara_id = ?2
         ORDER BY r.relation_type, r.relation_point"
    ) {
        Ok(value) => value,
        Err(error) => return format!(r#"{{"ok":false,"error":"pair_query_prepare_failed","detail":"{}"}}"#, json_escape(&error.to_string())),
    };
    let mapped = match statement.query_map(rusqlite::params![chara_id_a, chara_id_b], |row| {
        Ok((row.get::<_, i32>(0)?, row.get::<_, i32>(1)?))
    }) {
        Ok(value) => value,
        Err(error) => return format!(r#"{{"ok":false,"error":"pair_query_failed","detail":"{}"}}"#, json_escape(&error.to_string())),
    };
    let mut relation_items = Vec::new();
    let mut base_compatibility = 0i64;
    for row in mapped {
        let (relation_type, relation_point) = match row {
            Ok(value) => value,
            Err(error)
```

### `fn storage_flush_endpoint`
matches=1

```rust
<_, i64>(3)?, row.get::<_, i64>(4)?,
            json_escape(&row.get::<_, String>(5)?), row.get::<_, i64>(6)? != 0,
            json_escape(&row.get::<_, String>(7)?)
        ))
    }) {
        Ok(value) => value,
        Err(error) => return format!(r#"{{"ok":false,"error":"query_sessions:{}","sessions":[]}}"#, json_escape(&error.to_string())),
    };
    let mut sessions = Vec::new();
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
    format!(r#"{{"ok":true,"count":{},"sessions":[{}]}}"#, sessions.len(), sessions.join(","))
}

fn storage_session_endpoint(uri: &str) -> String {
    let pairs = match parse_query_pairs(uri) {
        Ok(value) => value,
        Err(error) => return format!(r#"{{"ok":false,"error":"{}"}}"#, json_escape(&error)),
    };
    let session_id = query_pair(&pairs, "id");
    if session_id.is_empty() { return r#"{"ok":false,"error":"missing_id"}"#.to_string(); }
    let connection = match open_observation_storage() {
        Ok(value) => value,
        Err(error) => return format!(r#"{{"ok":false,"error":"{}"}}"#, json_escape(&error)),
    };
    let result = connection.query_row(
        "SELECT process_id, plugin_version, started_at_ms, last_flush_ms,
                state, recovered_after_restart, root_path
         FROM observation_sessions WHERE session_id=?1",
        rusqlite::params![session_id],
        |row| Ok(format!(
            r#"{{"ok":true,"session":{{"session_id":"{}","process_id":{},"plugin_version":"{}","started_at_ms":{},"last_flush_ms":{},"state":"{}","recovered_after_restart":{},"root_path":"{}"}}}}"#,
            json_escape(&session_id), row.get::<_, i64>(0)?, json_escape(&row.get::<_, String>(1)?),
            row.get::<_, i64>(2)?, row.get::<_, i64>(3)?, json_escape(&row.get::<_, String>(4)?),
            row.get::<_, i64>(5)? != 0, json_escape(&row.get::<_, String>(6)?)
        )),
    );
    match result {
        Ok(value) => value,
        Err(rusqlite::Error::QueryReturnedNoRows) => r#"{"ok":true,"session":null,"status":"none"}"#.to_string(),
        Err(error) => format!(r#"{{"ok":false,"error":"query_session:{}"}}"#, json_escape(&error.to_string())),
    }
}

fn storage_flush_endpoint() -> String {
    let session_id = match ensure_observation_session() {
        Ok(value) => value,
        Err(error) => return format!(r#"{{"ok":false,"error":"{}"}}"#, json_escape(&error)),
    };
    let connection = match open_observation_storage() {
        Ok(value) => value,
        Err(error) => return format!(r#"{{"ok":false,"error":"{}"}}"#, json_escape(&error)),
    };
    let now = sniff_timestamp_ms();
    if let Err(error) = connection.execute_batch("PRAGMA wal_checkpoint(FULL);") {
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
    format!(r#"{{"ok":true,"session_id":"{}","last_flush_ms":{},"checkpoint":"full"}}"#, json_escape(&session_id), now)
}

fn storage_recover_endpoint() -> String {
    let connection = match open_observation_storage() {
        Ok(value) => value,
        Err(error) => return format!(r#"{{"ok":false,"error":"{}"}}"#, json_escape(&error)),
    };
    let process_start_token = observation_process_start_token();
    let recovered = match connection.execute(
        "UPDATE observation_sessions
         SET state='interrupted', recovered_after_restart=1
         WHERE state='open' AND process_start_token<>?1",
        rusqlite::params![process_start_token],
    ) {
        Ok(value) => value,
        Err(error) => return format!(r#"{{"ok":false,"error":"recover:{}"}}"#, json_escape(&error.to_string())),
    };
    match ensure_observation_session() {
        Ok(session_id) => format!(r#"{{"ok":true,"recovered_session_count":{},"current_session_id":"{}"}}"#, recovered, json_escape(&session_id)),
        Err(error) => format!(r#"{{"ok":false,"error":"{}","recovered_session_count":{}}}"#, json_escape(&error), recovered),
    }
}

// ===== Unified inheritance pair compatibility C-stage =====
fn inherit_pair_compat_endpoint(uri: &str) -> String {
    let pairs = match parse_query_pairs(uri) {
        Ok(value) => value,
        Err(error) => return format!(r#"{{"ok":false,"error":"{}"}}"#, json_escape(&error)),
    };
    if pairs.iter().filter(|(key, _)| key == "chara_id_a").count() != 1 || pairs.iter().filter(|(key, _)| key == "chara_id_b").count() != 1 { return r#"{\"ok\":false,\"error\":\"missing_or_duplicate_character_key\"}"#.to_string(); }
    let chara_id_a = match query_pair(&pairs, "chara_id_a").parse::<i32>() {
        Ok(value) if value > 0 => value,
        _ => return r#"{"ok":false,"error":"invalid_or_missing_chara_id_a"}"#.to_string(),
    };
    let chara_id_b = match query_pair(&pairs, "chara_id_b").parse::<i32>() {
        Ok(value) if value > 0 => value,
        _ => return r#"{"ok":false,"error":"invalid_or_missing_chara_id_b"}"#.to_string(),
    };
    let mdb_path = match find_mdb_path() {
        Some(value) => value,
        None => return r#"{"ok":false,"error":"mdb_not_found"}"#.to_string(),
    };
    let connection = match Connection::open_with_flags(&mdb_path, OpenFlags::SQLITE_OPEN_READ_ONLY) {
        Ok(value) => value,
        Err(error) => return format!(r#"{{"ok":false,"error":"mdb_open_failed","detail":"{}"}}"#, json_escape(&error.to_string())),
    };
    for (label, value) in [("chara_id_a", chara_id_a), ("chara_id_b", chara_id_b)] {
        let exists = connection.query_row("SELECT EXISTS(SELECT 1 FROM chara_data WHERE id=?1)", rusqlite::params![value], |row| row.get::<_, i64>(0));
        match exists { Ok(1) => {}, Ok(_) => return format!(r#"{{\"ok\":false,\"error\":\"character_not_found\",\"field\":\"{}\",\"value\":{}}}"#, label, value), Err(error) => return format!(r#"{{\"ok\":false,\"error\":\"character_validation_failed\",\"detail\":\"{}\"}}"#, json_escape(&error.to_string())) }
    }
    let mut statement = match connection.prepare(
        "SELECT DISTINCT r.relation_type, r.relation_point
         FROM succession_relation r
         INNER JOIN succession_relation_member a
             ON a.relation_type = r.relation_type AND a.chara_id = ?1
         INNER JOIN succession_relation_member b
             ON b.relation_type = r.relation_type AND b.chara_id = ?2
         ORDER BY r.relation_type, r.relation_point"
    ) {
        Ok(value) => value,
        Err(error) => return format!(r#"{{"ok":false,"error":"pair_query_prepare_failed","detail":"{}"}}"#, json_escape(&error.to_string())),
    };
    let mapped = match statement.query_map(rusqlite::params![chara_id_a, chara_id_b], |row| {
        Ok((row.get::<_, i32>(0)?, row.get::<_, i32>(1)?))
    }) {
        Ok(value) => value,
        Err(error) => return format!(r#"{{"ok":false,"error":"pair_query_failed","detail":"{}"}}"#, json_escape(&error.to_string())),
    };
    let mut relation_items = Vec::new();
    let mut base_compatibility = 0i64;
    for row in mapped {
        let (relation_type, relation_point) = match row {
            Ok(value) => value,
            Err(error) => return format!(r#"{{"ok":false,"error":"pair_row_decode_failed","detail":"{}"}}"#, json_escape(&error.to_string())),
        };
        base_compatibility += i64::from(relation_point);
        relation_items.push(format!(
            r#"{{"relation_type":{},"relation_point":{},"chara_id_a_member":true,"chara_id_b_member":true}}"#,
            relation_type, relation_point
        ));
    }
    format!(
        r#"{{"ok":true,"source":"current_mdb","calculation":"sum_shared_succession_relation_points","chara_id_a":{},"chara_id_b":{},"shared_relation_count":{},"base_compatibility":{},"shared_relations":[{}],"race_bonus":null,"specific_trained_chara_adjustments":null,"runtime_consumer_result":null,"scope":"character_pair_base_only"}}"#,
        chara_id_a, chara_id_b, relation_items.len(), base_compatibility, relation_items.join(",")
    )
}

// ===== Unified selected inheritance parents D-stage =====
unsafe fn inherit_selected_parent_runtime_endpoint() -> String {
    if API.is_null() {
        return r#"{"ok":false,"error":"api_null"}"#.to_string();
    }
    let image = get_image();
    if image.is_null() {
        return r#"{"ok":false,"error":"image_null"}"#.to_string();
    }
    let wdm_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkDataManager").as_ptr(),
    );
    if wdm_class.is_null() {
        return r#"{"ok":false,"error":"work_data_manager_class_not_found"}"#.to_string();
    }
    let wdm = get_singleton(wdm_class);
    if wdm.is_null() {
        return r#"{"ok":false,"error":"work_data_manager_instance_not_found"}"#.to_string();
  
```

## method_index

### `struct MethodIndexEntry`
matches=1

```rust
   }
                })
                .unwrap_or_else(|| return_type_str.clone())
        };

        // 检查是否是本类定义的方法（不是继承的）
        let is_own_method = method_get_class_fn
            .map(|f| {
                let declaring_class = f(method_info);
                declaring_class == class
            })
            .unwrap_or(true);

        let mut parameter_items = Vec::new();
        for index in 0..param_count {
            let parameter_type = method_get_param_fn.map(|f| f(method_info, index)).unwrap_or(ptr::null());
            let parameter_name = method_get_param_name_fn.and_then(|f| {
                let value = f(method_info, index);
                if value.is_null() { None } else { Some(CStr::from_ptr(value).to_string_lossy().into_owned()) }
            });
            let parameter_type_enum = if parameter_type.is_null() { 0 } else { il2cpp_type_get_type_enum(parameter_type) };
            let parameter_type_name = if parameter_type.is_null() { "unknown".to_string() } else {
                type_get_name_fn.and_then(|f| {
                    let value = f(parameter_type);
                    if value.is_null() { None } else { Some(CStr::from_ptr(value).to_string_lossy().into_owned()) }
                }).unwrap_or_else(|| type_enum_to_name(parameter_type_enum))
            };
            parameter_items.push(format!(
                r#"{{"index":{},"name":{},"type":"{}","type_name":"{}","resolved":{}}}"#,
                index,
                parameter_name.map(|value| format!("\"{}\"", json_escape(&value))).unwrap_or_else(|| "null".to_string()),
                type_enum_to_name(parameter_type_enum), json_escape(&parameter_type_name), !parameter_type.is_null()
            ));
        }
        methods.push(format!(
            r#"{{"name":"{}","params":{},"parameters":[{}],"return_type":"{}","return_type_name":"{}","static":{},"own":{}}}"#,
            json_escape(&method_name), param_count, parameter_items.join(","), return_type_str,
            json_escape(&return_type_name), is_static, is_own_method
        ));
    }
    format!(
        r#"{{"ok":true,"requested":"{}","found":"{}","method_count":{},"methods":[{}]}}"#,
        class_name,
        real_name,
        methods.len(),
        methods.join(",")
    )
}

// ===== Unified observation endpoint A-stage =====
// The index is built once per game process. Addresses are stored as usize so the
// synchronized state never contains raw pointers shared between threads.
#[derive(Clone)]
struct MethodIndexEntry {
    method_info: usize,
    method_pointer: usize,
    namespace: String,
    declaring_type: String,
    method_name: String,
    return_type: String,
    parameter_names: Vec<Option<String>>,
    parameter_types: Vec<String>,
    flags: u32,
}

struct MethodIndexState {
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

fn percent_decode_component(input: &str) -> Result<String, String> {
    let bytes = input.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0usize;
    while index < bytes.len() {
        match bytes[index] {
            b'%' => {
                if index + 2 >= bytes.len() {
                    return Err("incomplete_percent_escape".to_string());
                }
                let hex = &input[index + 1..index + 3];
                let value = u8::from_str_radix(hex, 16)
                    .map_err(|_| "invalid_percent_escape".to_string())?;
                output.push(value);
                index += 3;
            }
            b'+' => {
                output.push(b' ');
                index += 1;
            }
            value => {
                output.push(value);
                index += 1;
            }
        }
    }
    String::from_utf8(output).map_err(|_| "query_not_utf8".to_string())
}

fn parse_request_uri(request: &str) -> Result<String, String> {
    let line = request.lines().next().ok_or_else(|| "missing_request_line".to_string())?;
    let mut parts = line.split_whitespace();
    let method = parts.next().ok_or_else(|| "missing_http_method".to_string())?;
    let uri = parts.next().ok_or_else(|| "missing_request_uri".to_string())?;
    let version = parts.next().ok_or_else(|| "missing_http_version".to_string())?;
    if method.is_empty() || !version.starts_with("HTTP/") || parts.next().is_some() {
        return Err("invalid_request_line".to_string());
    }
    Ok(uri.to_string())
}

fn parse_query_pairs(uri: &str) -> Result<Vec<(String, String)>, String> {
    let query = match uri.split_once('?') {
        Some((_, value)) => value.split('#').next().unwrap_or(""),
        None => return Ok(Vec::new()),
    };
    let mut pairs = Vec::new();
    for item in query.split('&') {
        if item.is_empty() { continue; }
        let (raw_key, raw_value) = item.split_once('=').unwrap_or((item, ""));
        pairs.push((percent_decode_component(raw_key)?, percent_decode_component(raw_value)?));
    }
    Ok(pairs)
}

fn query_pair(pairs: &[(String, String)], name: &str) -> String {
    pairs.iter().find(|(key, _)| key == name).map(|(_, value)| value.clone()).unwrap_or_default()
}

fn parse_address(value: &str) -> Option<usize> {
    let trimmed = value.trim();
    if let Some(hex) = trimmed.strip_prefix("0x").or_else(|| trimmed.strip_prefix("0X")) {
        usize::from_str_radix(hex, 16).ok()
    } else {
        trimmed.parse::<usize>().ok()
    }
}

unsafe fn il2cpp_c_string(pointer: *const c_char) -> String {
    if pointer.is_null() { String::new() } else { CStr::from_ptr(pointer).to_string_lossy().into_owned() }
}

unsafe fn class_full_declaring_name(class: *mut c_void) -> String {
    if class.is_null() { return String::new(); }
    let get_name_ptr = resolve_il2cpp_symbol("il2cpp_class_get_name");
    let get_namespace_ptr = resolve_il2cpp_symbol("il2cpp_class_get_namespace");
    let get_declaring_ptr = resolve_il2cpp_symbol("il2cpp_class_get_declaring_type");
    if get_name_ptr.is_null() || get_namespace_ptr.is_null() || get_declaring_ptr.is_null() {
        return String::new();
    }
    let get_name: unsafe extern "C" fn(*mut c_void) -> *const c_char = std::mem::transmute(get_name_ptr);
    let get_namespace: unsafe extern "C" fn(*mut c_void) -> *const c_char = std::mem::transmute(get_namespace_ptr);
    let get_declaring: unsafe extern "C" fn(*mut c_void) -> *mut c_void = std::mem::transmute(get_declaring_ptr);
    let mut names = Vec::new();
    let mut current = class;
    let mut namespace = String::new();
    for _ in 0..64 {
        if current.is_null() { break; }
        names.push(il2cpp_c_string(get_name(current)));
        namespace = il2cpp_c_string(get_namespace(current));
        current = get_declaring(current);
    }
    names.reverse();
    let chain = names.join("/");
    if namespace.is_empty() { chain } else { format!("{}.{}", namespace, chain) }
}

unsafe fn find_class_by_full_declaring_name(requested: &str) -> *mut c_void {
    let image = get_image();
    if image.is_null() || requested.is_empty() { return ptr::null_mut(); }
    let (namespace, type_chain) = match requested.split_once('.') {
        Some((namespace, rest)) => (namespace, rest),
        None => ("", requested),
    };
    let mut names = type_chain.split('/');
    let outer = match names.next() { Some(value) if !value.is_empty() => value, _ => return ptr::null_mut() };
    let mut class = find_class(image, to_cstr(namespace).as_ptr(), to_cstr(outer).as_ptr());
    if class.is_null() { return ptr::null_mut(); }
    let nested_ptr = resolve_il2cpp_symbol("il2cpp_class_get_nested_types");
    let name_ptr = resolve_il2cpp_symbol("il2cpp_class_get_name");
    if names.clone().next().is_some() && (nested_ptr.is_null() || name_ptr.is_null()) { return ptr::null_mut(); }
    let get_nested: unsafe extern "C" fn(*mut c_void, *mut *mut c_void) -> *mut c_void = std::mem::transmute(nested_ptr);
    let get_name: unsafe extern "C" fn(*mut c_void) -> *const c_char = std::mem::transmute(name_ptr);
    for requested_nested in names {
        let mut iterator = ptr::null_mut();
        let mut found: *mut c_void = ptr::null_mut();
        loop {
            let candidate = get_nested(class, &mut iterator);
            if candidate.is_null() { break; }
            if il2cpp_c_string(get_name(candidate)) == requested_nested {
                if !found.is_null() { return ptr::null_mut(); }
                found = candidate;
            }
        }
        if found.is_null() { return ptr::null_mut(); }
        class = found;
    }
    class
}

unsafe fn build_method_index(generation: u64) -> Result<Vec<MethodIndexEntry>, String> {
    let image 
```

### `unsafe fn build_method_index`
matches=1

```rust
rn String::new();
    }
    let get_name: unsafe extern "C" fn(*mut c_void) -> *const c_char = std::mem::transmute(get_name_ptr);
    let get_namespace: unsafe extern "C" fn(*mut c_void) -> *const c_char = std::mem::transmute(get_namespace_ptr);
    let get_declaring: unsafe extern "C" fn(*mut c_void) -> *mut c_void = std::mem::transmute(get_declaring_ptr);
    let mut names = Vec::new();
    let mut current = class;
    let mut namespace = String::new();
    for _ in 0..64 {
        if current.is_null() { break; }
        names.push(il2cpp_c_string(get_name(current)));
        namespace = il2cpp_c_string(get_namespace(current));
        current = get_declaring(current);
    }
    names.reverse();
    let chain = names.join("/");
    if namespace.is_empty() { chain } else { format!("{}.{}", namespace, chain) }
}

unsafe fn find_class_by_full_declaring_name(requested: &str) -> *mut c_void {
    let image = get_image();
    if image.is_null() || requested.is_empty() { return ptr::null_mut(); }
    let (namespace, type_chain) = match requested.split_once('.') {
        Some((namespace, rest)) => (namespace, rest),
        None => ("", requested),
    };
    let mut names = type_chain.split('/');
    let outer = match names.next() { Some(value) if !value.is_empty() => value, _ => return ptr::null_mut() };
    let mut class = find_class(image, to_cstr(namespace).as_ptr(), to_cstr(outer).as_ptr());
    if class.is_null() { return ptr::null_mut(); }
    let nested_ptr = resolve_il2cpp_symbol("il2cpp_class_get_nested_types");
    let name_ptr = resolve_il2cpp_symbol("il2cpp_class_get_name");
    if names.clone().next().is_some() && (nested_ptr.is_null() || name_ptr.is_null()) { return ptr::null_mut(); }
    let get_nested: unsafe extern "C" fn(*mut c_void, *mut *mut c_void) -> *mut c_void = std::mem::transmute(nested_ptr);
    let get_name: unsafe extern "C" fn(*mut c_void) -> *const c_char = std::mem::transmute(name_ptr);
    for requested_nested in names {
        let mut iterator = ptr::null_mut();
        let mut found: *mut c_void = ptr::null_mut();
        loop {
            let candidate = get_nested(class, &mut iterator);
            if candidate.is_null() { break; }
            if il2cpp_c_string(get_name(candidate)) == requested_nested {
                if !found.is_null() { return ptr::null_mut(); }
                found = candidate;
            }
        }
        if found.is_null() { return ptr::null_mut(); }
        class = found;
    }
    class
}

unsafe fn build_method_index(generation: u64) -> Result<Vec<MethodIndexEntry>, String> {
    let image = get_image();
    if image.is_null() { return Err("image_null".to_string()); }
    let symbols = [
        "il2cpp_image_get_class_count", "il2cpp_image_get_class", "il2cpp_class_get_methods",
        "il2cpp_method_get_name", "il2cpp_method_get_param_count", "il2cpp_method_get_param",
        "il2cpp_method_get_param_name", "il2cpp_method_get_return_type", "il2cpp_type_get_name",
        "il2cpp_method_get_flags", "il2cpp_method_get_class",
    ];
    let resolved: Vec<*mut c_void> = symbols.iter().map(|name| resolve_il2cpp_symbol(name)).collect();
    if let Some(index) = resolved.iter().position(|value| value.is_null()) {
        return Err(format!("missing_symbol:{}", symbols[index]));
    }
    let get_class_count: FnImageGetClassCount = std::mem::transmute(resolved[0]);
    let get_class: FnImageGetClass = std::mem::transmute(resolved[1]);
    let get_methods: FnClassGetMethods = std::mem::transmute(resolved[2]);
    let get_method_name: FnMethodGetName = std::mem::transmute(resolved[3]);
    let get_param_count: unsafe extern "C" fn(*const c_void) -> u32 = std::mem::transmute(resolved[4]);
    let get_param: unsafe extern "C" fn(*const c_void, u32) -> *const c_void = std::mem::transmute(resolved[5]);
    let get_param_name: unsafe extern "C" fn(*const c_void, u32) -> *const c_char = std::mem::transmute(resolved[6]);
    let get_return_type: unsafe extern "C" fn(*const c_void) -> *const c_void = std::mem::transmute(resolved[7]);
    let get_type_name: unsafe extern "C" fn(*const c_void) -> *const c_char = std::mem::transmute(resolved[8]);
    let get_flags: unsafe extern "C" fn(*const c_void, *mut u32) -> u32 = std::mem::transmute(resolved[9]);
    let get_method_class: unsafe extern "C" fn(*const c_void) -> *mut c_void = std::mem::transmute(resolved[10]);
    let mut entries = Vec::new();
    let class_count = get_class_count(image);
    {
        let mut state = METHOD_INDEX.lock().unwrap_or_else(|error| error.into_inner());
        if state.generation != generation { return Err("method_index_generation_superseded".to_string()); }
        state.image_class_count = class_count;
        state.heartbeat_at_ms = method_index_now_ms();
    }
    for class_index in 0..class_count {
        if class_index % 32 == 0 {
            let now = method_index_now_ms();
            let mut state = METHOD_INDEX.lock().unwrap_or_else(|error| error.into_inner());
            if state.generation != generation { return Err("method_index_generation_superseded".to_string()); }
            if now.saturating_sub(state.started_at_ms) > 180_000 { return Err("method_index_build_timeout".to_string()); }
            state.indexed_class_count = class_index;
            state.indexed_method_count = entries.len();
            state.heartbeat_at_ms = now;
        }
        let class = get_class(image, class_index);
        if class.is_null() { continue; }
        let mut iterator = ptr::null_mut();
        loop {
            let method_info = get_methods(class, &mut iterator);
            if method_info.is_null() { break; }
            let declaring_class = get_method_class(method_info);
            let declaring_type = class_full_declaring_name(declaring_class);
            let namespace = declaring_type.split_once('.').map(|(value, _)| value.to_string()).unwrap_or_default();
            let parameter_count = get_param_count(method_info);
            let mut parameter_names = Vec::with_capacity(parameter_count as usize);
            let mut parameter_types = Vec::with_capacity(parameter_count as usize);
            for parameter_index in 0..parameter_count {
                let parameter_type = get_param(method_info, parameter_index);
                parameter_types.push(if parameter_type.is_null() { "unresolved".to_string() } else { il2cpp_c_string(get_type_name(parameter_type)) });
                let parameter_name = il2cpp_c_string(get_param_name(method_info, parameter_index));
                parameter_names.push(if parameter_name.is_empty() { None } else { Some(parameter_name) });
            }
            let return_type_pointer = get_return_type(method_info);
            let return_type = if return_type_pointer.is_null() { "unresolved".to_string() } else { il2cpp_c_string(get_type_name(return_type_pointer)) };
            let mut iflags = 0u32;
            let flags = get_flags(method_info, &mut iflags);
            let method_pointer = if is_readable_range(method_info as usize, std::mem::size_of::<usize>()) {
                std::ptr::read_unaligned::<usize>(method_info as *const usize)
            } else { 0 };
            entries.push(MethodIndexEntry {
                method_info: method_info as usize,
                method_pointer,
                namespace,
                declaring_type,
                method_name: il2cpp_c_string(get_method_name(method_info)),
                return_type,
                parameter_names,
                parameter_types,
                flags,
            });
        }
    }
    entries.sort_by(|left, right| left.method_pointer.cmp(&right.method_pointer).then(left.method_info.cmp(&right.method_info)));
    Ok(entries)
}

unsafe fn ensure_method_index() -> Result<(), String> {
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
                    let mut duplicate_count 
```

### `fn il2cpp_method_detail`
matches=1

```rust
y_pair(&pairs, "addr");
    let address = match parse_address(&raw) { Some(value) if value != 0 => value, _ => return r#"{"ok":false,"error":"invalid_or_missing_addr"}"#.to_string() };
    if let Err(error) = ensure_method_index() { return format!(r#"{{"ok":false,"error":"{}"}}"#, json_escape(&error)); }
    let state = match METHOD_INDEX.lock() { Ok(value) => value, Err(_) => return r#"{"ok":false,"error":"method_index_lock_poisoned"}"#.to_string() };
    let method_info_matches: Vec<&MethodIndexEntry> = state.entries.iter().filter(|entry| entry.method_info == address).collect();
    let exact_pointer_matches: Vec<&MethodIndexEntry> = state.entries.iter().filter(|entry| entry.method_pointer == address && entry.method_pointer != 0).collect();
    let (kind, matches): (&str, Vec<&MethodIndexEntry>) = if !method_info_matches.is_empty() {
        (if method_info_matches.len() == 1 { "exact_method_info" } else { "ambiguous" }, method_info_matches)
    } else if !exact_pointer_matches.is_empty() {
        (if exact_pointer_matches.len() == 1 { "exact_method_pointer" } else { "ambiguous" }, exact_pointer_matches)
    } else {
        let mut distinct: Vec<usize> = state.entries.iter().map(|entry| entry.method_pointer).filter(|value| *value != 0).collect();
        distinct.dedup();
        match distinct.binary_search(&address) {
            Ok(_) => ("none", Vec::new()),
            Err(position) if position > 0 && position < distinct.len() => {
                let start = distinct[position - 1];
                let candidates: Vec<&MethodIndexEntry> = state.entries.iter().filter(|entry| entry.method_pointer == start).collect();
                (if candidates.len() == 1 { "upper_bound_candidate" } else { "ambiguous" }, candidates)
            }
            _ => ("none", Vec::new()),
        }
    };
    let items = matches.iter().map(|entry| {
        let upper = state.entries.iter().map(|candidate| candidate.method_pointer).filter(|pointer| *pointer > entry.method_pointer).min();
        method_entry_json(entry, upper)
    }).collect::<Vec<_>>().join(",");
    format!(r#"{{"ok":true,"query":"0x{:x}","status":"{}","ambiguous":{},"matches":[{}],"index":{{"status":"{}","classes":{},"methods":{},"null_method_pointers":{},"duplicate_method_pointers":{}}}}}"#,
        address, kind, kind == "ambiguous", items, state.status, state.indexed_class_count,
        state.indexed_method_count, state.null_method_pointer_count, state.duplicate_method_pointer_count)
}

unsafe fn il2cpp_method_detail(uri: &str) -> String {
    let pairs = match parse_query_pairs(uri) { Ok(value) => value, Err(error) => return format!(r#"{{"ok":false,"error":"{}"}}"#, json_escape(&error)) };
    let namespace = query_pair(&pairs, "namespace");
    let declaring_type = query_pair(&pairs, "declaring_type");
    let method = query_pair(&pairs, "method");
    let parameter_text = query_pair(&pairs, "parameter_types");
    if declaring_type.is_empty() || method.is_empty() { return r#"{"ok":false,"error":"missing_declaring_type_or_method"}"#.to_string(); }
    let parameter_types: Vec<String> = if parameter_text.is_empty() { Vec::new() } else { parameter_text.split(',').map(|value| value.trim().to_string()).collect() };
    if let Err(error) = ensure_method_index() { return format!(r#"{{"ok":false,"error":"{}"}}"#, json_escape(&error)); }
    let state = match METHOD_INDEX.lock() { Ok(value) => value, Err(_) => return r#"{"ok":false,"error":"method_index_lock_poisoned"}"#.to_string() };
    let matches: Vec<&MethodIndexEntry> = state.entries.iter().filter(|entry| {
        (namespace.is_empty() || entry.namespace == namespace) && entry.declaring_type == declaring_type &&
        entry.method_name == method && entry.parameter_types == parameter_types
    }).collect();
    let status = if matches.is_empty() { "none" } else if matches.len() == 1 { "exact" } else { "ambiguous" };
    let items = matches.iter().map(|entry| {
        let upper = state.entries.iter().map(|candidate| candidate.method_pointer).filter(|pointer| *pointer > entry.method_pointer).min();
        method_entry_json(entry, upper)
    }).collect::<Vec<_>>().join(",");
    format!(r#"{{"ok":true,"status":"{}","ambiguous":{},"matches":[{}]}}"#, status, status == "ambiguous", items)
}

unsafe fn il2cpp_nested_types(uri: &str) -> String {
    let pairs = match parse_query_pairs(uri) { Ok(value) => value, Err(error) => return format!(r#"{{"ok":false,"error":"{}"}}"#, json_escape(&error)) };
    let requested = query_pair(&pairs, "type");
    if requested.is_empty() { return r#"{"ok":false,"error":"missing_type"}"#.to_string(); }
    let class = find_class_by_full_declaring_name(&requested);
    if class.is_null() { return format!(r#"{{"ok":false,"error":"class_not_found_or_ambiguous","type":"{}"}}"#, json_escape(&requested)); }
    let nested_ptr = resolve_il2cpp_symbol("il2cpp_class_get_nested_types");
    if nested_ptr.is_null() { return r#"{"ok":false,"error":"il2cpp_class_get_nested_types_unavailable"}"#.to_string(); }
    let get_nested: unsafe extern "C" fn(*mut c_void, *mut *mut c_void) -> *mut c_void = std::mem::transmute(nested_ptr);
    let mut iterator = ptr::null_mut();
    let mut items = Vec::new();
    loop {
        let nested = get_nested(class, &mut iterator);
        if nested.is_null() { break; }
        items.push(format!(r#"{{"type":"{}","class_pointer":"0x{:x}"}}"#, json_escape(&class_full_declaring_name(nested)), nested as usize));
    }
    format!(r#"{{"ok":true,"requested":"{}","direct_only":true,"count":{},"nested_types":[{}]}}"#, json_escape(&requested), items.len(), items.join(","))
}

unsafe fn il2cpp_enum_values_capability(uri: &str) -> String {
    let pairs = match parse_query_pairs(uri) { Ok(value) => value, Err(error) => return format!(r#"{{"ok":false,"error":"{}"}}"#, json_escape(&error)) };
    let requested = query_pair(&pairs, "type");
    if requested.is_empty() { return r#"{\"ok\":false,\"error\":\"missing_type\"}"#.to_string(); }
    let required = ["il2cpp_class_get_fields", "il2cpp_field_get_flags", "il2cpp_field_static_get_value"];
    let available: Vec<bool> = required.iter().map(|name| !resolve_il2cpp_symbol(name).is_null()).collect();
    format!(r#"{{"ok":true,"requested":"{}","value_status":"unresolved","integer_values":null,"declaration_order_inference":false,"runtime_api":{{"il2cpp_class_get_fields":{},"il2cpp_field_get_flags":{},"il2cpp_field_static_get_value":{}}}}}"#,
        json_escape(&requested), available[0], available[1], available[2])
}

// ===== Unified observation persistent storage B-stage =====
static STORAGE_SESSION_ID: Mutex<Option<String>> = Mutex::new(None);
static STORAGE_LAST_FLUSH_MS: AtomicU64 = AtomicU64::new(0);
static STORAGE_LAST_ERROR: Mutex<Option<String>> = Mutex::new(None);

fn storage_set_error(error: &str) {
    if let Ok(mut value) = STORAGE_LAST_ERROR.lock() {
        *value = Some(error.to_string());
    }
}

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

fn observation_storage_root() -> std::path::PathBuf {
    if let Ok(command_line) = std::fs::read("/proc/self/cmdline") {
        let package_bytes = command_line.split(|byte| *byte == 0).next().unwrap_or(&[]);
        if let Ok(package_name) = std::str::from_utf8(package_bytes) {
            if !package_name.is_empty() {
                return std::path::PathBuf::from("/data/user/0")
                    .join(package_name)
        
```

## routing

### `let response = if path ==`
matches=0

### `Content-Length`
matches=18

```rust
ra_grade":{},"difficulty":{},"fixed_turn_chara_seed":{},"trainings":{},"support_cards":{},"evaluation":{},"training_levels":{},"buffs":{},"chara_effect_ids":[{}],"skills":{{"eval":{},"count":{},"list":{}}},"ai":{}{}{}{} }}"#,
        PLUGIN_VERSION,
        year,
        cumulative_turn,
        raw_total_turn_num,
        mon,
        half,
        scn_s,
        card_id,
        chara_id,
        spd,
        sta,
        pow_,
        gut,
        wiz,
        vit,
        mvit,
        mot_s,
        spt,
        fan,
        max_spd,
        max_sta,
        max_pow,
        max_gut,
        max_wiz,
        proper_dist_short,
        proper_dist_mile,
        proper_dist_mid,
        proper_dist_long,
        proper_ground_turf,
        proper_ground_dirt,
        running_style,
        scenario_progress,
        training_event_type,
        talent_level,
        chara_grade,
        difficulty,
        fixed_turn_chara_seed,
        tr_json,
        sc_json,
        ev_json,
        tl_json,
        buff_json,
        effect_ids_str.join(","),
        skill_eval,
        skill_count,
        skills_json,
        ai_json,
        team_json,
        ramen_json,
        last_action_json
    )
}

// ============================================================
// HTTP Server
// ============================================================

// ============================================================
// ★ Push-to-app (v3.10.0): auto-push /summary JSON to uma-juece
// When game data changes, POST the /summary JSON to 127.0.0.1:18766/data
// The uma-juece floating window app receives and displays the data
// ============================================================

fn simple_hash(s: &str) -> u64 {
    let mut h: u64 = 5381;
    for b in s.bytes() {
        h = h.wrapping_mul(33).wrapping_add(b as u64);
    }
    h
}

fn push_to_app(json: &str) {
    use std::io::{Read, Write};
    let cfg = unsafe { get_config() };
    if !cfg.push_enabled {
        return;
    }
    let addr_str = cfg.push_addr();
    let addr: std::net::SocketAddr = match addr_str.parse() {
        Ok(a) => a,
        Err(_) => return,
    };
    let mut stream =
        match std::net::TcpStream::connect_timeout(&addr, std::time::Duration::from_secs(2)) {
            Ok(s) => s,
            Err(_) => return, // App not running, that's fine
        };
    let body = json.as_bytes();
    let req = format!(
        "POST /data HTTP/1.1\r\nHost: {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        addr_str, body.len()
    );
    let _ = stream.write_all(req.as_bytes());
    let _ = stream.write_all(body);
    let _ = stream.flush();
    let mut buf = [0u8; 256];
    let _ = stream.read(&mut buf);
}

fn push_loop() {
    let interval =
        std::time::Duration::from_secs(unsafe { get_config() }.push_interval_secs.max(2));
    let mut consecutive_errors: u32 = 0;

    // ★ Initial push: try pushing current data on startup
    // Don't rely solely on GAME_INITIALIZED callback — it may never fire
    // if the game was already initialized before the plugin loaded.
    // Instead, try reading data; if it succeeds, the game is ready.
    for wait_round in 0..60 {
        if GAME_INITIALIZED.load(Ordering::Relaxed) {
            break;
        }
        boot_trace("push_probe_begin");
        // Try a probe read — if it doesn't error, game is ready
        let probe = read_summary();
        if !probe.contains("\"error\"") {
            GAME_INITIALIZED.store(true, Ordering::Relaxed);
            unsafe {
                ura_log(3, "Push: game detected via probe (no callback)");
                // v3.22.98: Install hooks in fallback (on_game_initialized may never fire)
                install_training_hook();
                install_exec_training_hook();
                install_failure_rate_hook();
                install_event_choice_hook();
                // ★ v3.24.40: sniff hooks were missing here — fallback mode
                // left /api/sniff permanently unhooked.
                install_api_sniff_hooks();
            }
            break;
        }
        if wait_round % 10 == 0 {
            unsafe {
                ura_log(
                    3,
                    &format!("Push: waiting for game... round={}", wait_round),
                );
            }
        }
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
    let init_summary = read_summary();
    if !init_summary.contains("\"error\"") {
        unsafe {
            LAST_PUSH_HASH = simple_hash(&init_summary);
        }
        push_to_app(&init_summary);
        unsafe {
            ura_log(3, "Push: initial data pushed");
        }
    }

    loop {
        std::thread::sleep(interval);
        // Don't gate on GAME_INITIALIZED — just try reading;
        // if the game isn't ready, read_summary returns error and we skip.
        let summary = read_summary();
        if summary.contains("\"error\"") {
            consecutive_errors += 1;
            // ★ v3.22.89: Extra cooldown for SIGSEGV recovery — game state transition
            if summary.contains("sigsegv") {
                let cool = std::time::Duration::from_secs(60);
                unsafe {
                    ura_log(
                        2,
                        "Push: SIGSEGV recovered, cooling 60s for game state transition",
                    );
                }
                std::thread::sleep(cool);
            }
            // ★ v3.14.2: backoff on consecutive errors to avoid crash loop
            if consecutive_errors >= 1 {
                let backoff =
                    std::time::Duration::from_secs((consecutive_errors as u64 * 5).min(60));
                unsafe {
                    ura_log(
                        3,
                        &format!(
                            "Push: {} consecutive errors, backing off {}s",
                            consecutive_errors,
                            backoff.as_secs()
                        ),
                    );
                }
                std::thread::sleep(backoff);
            }
            continue;
        }
        consecutive_errors = 0;
        // If we got here, game is definitely ready
        if !GAME_INITIALIZED.load(Ordering::Relaxed) {
            GAME_INITIALIZED.store(true, Ordering::Relaxed);
        }
        let hash = simple_hash(&summary);
        let should_push = unsafe {
            if hash != LAST_PUSH_HASH {
                LAST_PUSH_HASH = hash;
                true
            } else {
                false
            }
        };
        if should_push {
            unsafe {
                ura_log(3, "Push: data changed, pushing to app");
            }
            push_to_app(&summary);
        }
    }
}

fn start_http_server() {
    if HTTP_RUNNING.load(Ordering::Relaxed) {
        return;
    }
    HTTP_RUNNING.store(true, Ordering::Relaxed);
    std::thread::spawn(|| {
        unsafe {
            // ★ v3.24.32: bind loopback only. The floating-window App talks to
            // the plugin on the same device, and desktop/LAN debugging works
            // via `adb forward tcp:18765 tcp:18765`. Binding 0.0.0.0 exposed
            // /il2cpp/call, /il2cpp/read_mem, /update etc. to the whole LAN
            // without authentication.
            ura_log(3, "HTTP starting on 127.0.0.1:18765");
        }
        let listener = match std::net::TcpListener::bind("127.0.0.1:18765") {
            Ok(l) => l,
            Err(e) => {
                unsafe {
                    ura_log(1, &format!("HTTP bind failed: {}", e));
                }
                HTTP_RUNNING.store(false, Ordering::Relaxed);
                return;
            }
        };
        unsafe {
            ura_log(3, "HTTP listening on :18765");
        }
        unsafe {
            ura_notify("URA HTTP :18765 ON");
        }

        // ★ Start push-to-app loop (v3.10.0)
        std::thread::spawn(|| {
            push_loop();
        });

        for stream in listener.incoming() {
            if !HTTP_RUNNING.load(Ordering::Relaxed) {
                break;
            }
            match stream {
                Ok(stream) => {
                    // ★ v3.18.8: spawn thread per request — prevents slow endpoint from blocking others
                    let _ = stream.set_read_timeout(Some(std::time::Duration::from_secs(5)));
                    let _ = stream.set_write_timeout(Some(std::time::Duration::from_secs(10)));
                    std::thread::spawn(move || handle_http(stream));
                }
                Err(_) => continue,
            }
        }
        HTTP_RUNNING.store(false, Ordering::Relaxed);
    });
}

fn parse_path(req: &str) -> String {
    let first_line = req.lines().next().unwrap_or("");
    let uri = first_line.split(' ').nth(1).unwrap_or("/");
    let path = uri.split('?').next().unwrap_or(uri);
    if path.starts_with("http://") || path.starts_with("https://") {
        if let Some(after_host) = path.splitn(4, '/').nth(3) {
            let result = if after_host.is_empty() {
                "/".to_string()
            } else {
                format!("/{}", after_host)
            };
            return result.trim_end_matches('/').to_string();
        }
        return "/".to_string();
    }
    if path.len() > 1 && path.ends_with('/') {
        path[..path.len() - 1].to_string()
    } else {
        path.to_string()
    }
}

/// ★ v
```

```rust
  let path = parse_path(req);
    let full_uri = req
        .lines()
        .next()
        .unwrap_or("")
        .split(' ')
        .nth(1)
        .unwrap_or("/");

    // ★ v3.24.55: boot gate. Crash autopsy via hachimi.log: the floating app
    // polls /summary during game boot; IL2CPP reads on the HTTP thread against
    // transitional objects SIGSEGV the process (sigjmp recovery only exists on
    // the push thread). Until the game is initialized, refuse every endpoint
    // that touches game memory; static/self-state endpoints stay available.
    if !GAME_INITIALIZED.load(Ordering::Relaxed) {
        const BOOT_SAFE_EXACT: &[&str] = &[
            "/",
            "/health",
            "/status",
            "/config",
            "/config.html",
            "/update",
            "/update/status",
            "/debug/hookdiag",
            "/debug/hooklog",
            "/debug/crashlog",
            "/debug/upload",
            "/api/sniff",
            "/api/sniff/metadata",
            "/api/sniff/status",
            "/api/sniff/diag",
            "/api/sniff/toggle",
            "/api/sniff/clear",
            "/api/md5log",
            "/api/md5log/clear",
            "/api/md5log/install",
            "/api/event/choices",
            "/api/event/observations",
            "/api/event/observations/clear",
            "/api/event/clear",
            "/action/latest",
            "/seed/history",
            "/seed/stats",
            "/log",
            "/carddb",
            "/skilldata",
            "/debug/table",
            "/debug/push_table",
            "/debug/download_table",
            "/debug/mdb_all_tables",
            "/debug/mdb_schema_dump",
        ];
        const BOOT_SAFE_PREFIX: &[&str] = &[
            "/mdb",
            "/debug/resource_",
            "/debug/private_file",
            "/debug/mem_scan_sqlite",
            "/debug/mem_scan_zdict",
            "/debug/mem_scan_hex",
            "/debug/file_scan_hex",
            "/debug/maps_list",
            "/debug/file_dl",
            "/debug/file_range_hex",
            "/il2cpp/read_string",
            "/il2cpp/read_mem",
        ];
        let safe = BOOT_SAFE_EXACT.iter().any(|p| path == *p)
            || BOOT_SAFE_PREFIX.iter().any(|p| path.starts_with(p));
        if !safe {
            let b = r#"{"status":"booting","game_initialized":false}"#;
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                b.len(), b
            );
            let _ = stream.write_all(resp.as_bytes());
            return;
        }
    }

    // ★ 白名单下载开关：名单内端点追加 ?dl=1 即以附件形式返回（解决手机复制长度上限）
    //    ?dl=1&name=xxx 可自定义文件名（仅保留字母数字和下划线/连字符）
    //    大文件仍走各专用流式 _dl 端点，避免此路径内存翻倍
    const DL_ALLOWED: &[&str] = &[
        "/summary",
        "/scenario",
        "/data",
        "/ramen",
        "/debug/ramen_transition",
        "/api/sniff",
        "/api/sniff/metadata",
        "/api/sniff/diag",
        "/api/event/choices",
        "/api/event/observations",
        "/debug/event_reward_targets",
        "/debug/resource_meta_schema",
        "/debug/resource_meta_probe",
        "/debug/resource_crypto_symbols",
        "/debug/all",
        "/debug/params",
        "/debug/cmdinfo",
        "/debug/breeders",
        "/debug/training_partners",
        "/debug/rameninfo",
        "/debug/laststep",
        "/debug/storydata",
        "/debug/ramenfields",
        "/debug/gauge",
        "/debug/gauge2",
        "/debug/ramengains",
        "/debug/paramsincdec",
        "/debug/training_seed",
        "/debug/unique_skills",
        "/debug/hint_gain",
        "/debug/sc_effect",
        "/debug/unique_detail",
        "/classes",
    ];
    let dl_flag = parse_query(&full_uri, "dl");
    let dl_name = parse_query(&full_uri, "name");
    let dl_enabled = !dl_flag.is_empty() && dl_flag != "0" && DL_ALLOWED.iter().any(|p| path == *p);

    let _parsed_request_uri = parse_request_uri(req).unwrap_or_else(|_| full_uri.to_string());
    let body = if path == "/debug/global_metadata_probe" {
        safe_mem_scan(req, true)
    } else if path == "/debug/mem_scan_hex" {
        safe_mem_scan(req, false)
    } else if path == "/debug/mem_maps" {
        safe_maps_summary()
    } else if path == "/" || path == "/health" {
        format!(
            r#"{{"status":"ok","version":"{}","endpoints":["/storage/status","/storage/sessions","/storage/session","/storage/flush","/storage/recover","/il2cpp/method_index_status","/il2cpp/method_by_addr","/il2cpp/method_detail","/il2cpp/nested_types","/il2cpp/enum_values","/inherit/pair_compat","/inherit/selected_parent_runtime","/summary","/data","/scenario","/debug/rameninfo","/debug/laststep","/event/recommend","/inherit/compat","/saddle-analysis","/log/turn","/debug/params","/debug/breeders","/debug/cmdinfo","/debug/training_partners","/debug/crashlog","/debug/upload","/debug/dumpclass","/debug/storydata","/debug/ramenfields","/debug/gauge","/debug/gauge2","/debug/ramengains","/debug/paramsincdec","/debug/training_seed","/debug/training_log","/debug/training_log_dl","/update","/update/status","/debug/all","/debug/unique_skills","/debug/mdb_all_tables","/debug/mdb_schema_dump","/debug/hint_gain","/debug/sc_effect","/debug/unique_detail","/debug/table","/debug/push_table","/debug/download_table","/mdb","/carddb","/skilldata","/hall","/saddles","/saddles-dl","/log","/status","/health","/mdb/schema","/mdb/search","/mdb/raw","/mdb/dl_batch","/il2cpp/dump","/il2cpp/call","/il2cpp/tree","/il2cpp/field","/il2cpp/classes","/il2cpp/static","/il2cpp/methods","/il2cpp/disassemble","/il2cpp/disassemble_dl","/il2cpp/disassemble_addr","/il2cpp/disassemble_addr_dl","/il2cpp/dump_all_methods","/il2cpp/dump_all_methods_dl","/il2cpp/search_float","/il2cpp/search_float_dl","/il2cpp/search_int","/il2cpp/search_int_dl","/il2cpp/search_methods","/il2cpp/search_methods_dl","/il2cpp/read_mem","/il2cpp/read_mem_dl","/training/result","/api/sniff","/api/sniff/metadata","/api/sniff/status","/api/sniff/toggle","/api/sniff/clear","/api/sniff/diag","/api/event/choices","/api/event/clear","/debug/hooklog","/debug/hookdiag","/debug/resource_meta_key","/debug/resource_db_keys","/debug/resource_reads","/debug/mem_scan_sqlite","/debug/meta_dump","/action/latest","/seed/history","/seed/stats","/debug/ramen_planner_state","/debug/ramen_participants","/debug/ramen_transition","/debug/ramen_dataset_path","/debug/ramen_formula_targets","/debug/event_reward_targets", "/debug/resource_storage","/debug/resource_meta_schema","/debug/resource_meta_probe", "/debug/resource_crypto_symbols","/debug/resource_meta_dl","/debug/resource_file_dl","/debug/private_file_inventory","/debug/private_file_dl"]}}"#,
            PLUGIN_VERSION
        )
    } else if path == "/scan" {
        unsafe { scan_il2cpp_classes() }
    } else if path == "/data" {
        let result = unsafe { read_training_data() };
        unsafe {
            log_snapshot("data", &result);
        }
        result
    } else if path == "/status" {
        format!(
            r#"{{"game_initialized":{},"http_running":{}}}"#,
            GAME_INITIALIZED.load(Ordering::Relaxed),
            HTTP_RUNNING.load(Ordering::Relaxed)
        )
    } else if path == "/singletons" {
        unsafe { find_all_singletons() }
    } else if path.starts_with("/find_method") {
        let method_name = if path == "/find_method" || path == "/find_method/" {
            "get_SingleMode"
        } else {
            path.strip_prefix("/find_method/")
                .unwrap_or("get_SingleMode")
        };
        unsafe { find_method_in_all_classes(method_name) }
    } else if path.starts_with("/fields") {
        let class_name = if path == "/fields" || path == "/fields/" {
            "WorkDataManager"
        } else {
            path.strip_prefix("/fields/").unwrap_or("WorkDataManager")
        };
        unsafe {
            let image = get_image();
            if image.is_null() {
                r#"{"error":"image_null"}"#.to_string()
            } else {
                let cls = find_class_by_short_name(image, class_name);
                if cls.is_null() {
                    format!(r#"{{"error":"class_not_found","name":"{}"}}"#, class_name)
                } else {
                    enumerate_class_fields(cls)
                }
            }
        }
    } else if path.starts_with("/methods") {
        let class_name = if path == "/methods" || path == "/methods/" {
            "WorkDataManager"
        } else {
            path.strip_prefix("/methods/").unwrap_or("WorkDataManager")
        };
        unsafe {
            let image = get_image();
            if image.is_null() {
                r#"{"error":"image_null"}"#.to_string()
            } else {
                let cls = find_class_by_short_name(image, class_name);
                if cls.is_null() {
                    format!(r#"{{"error":"class_not_found","name":"{}"}}"#, class_name)
                } else {
                    enumerate_class_methods(cls)
                }
            }
        }
    } else if path == "/summary" {
        read_summary()
    } else if path == "/debug/turn_probe" {
        // v3.24.72: expose decrypted raw field only; UI is a countdown and mapping is unknown.
        let s = read_summary();
        format!(
            r#"{{"status":"ok","raw_total_turn_num":{},"ui_turn_semantic
```

```rust
      let abs = sa + off + i;
                                // dump 256KB context starting at hit
                                let dump_len = 256 * 1024usize;
                                let mut dbuf = vec![0u8; dump_len];
                                let got = mem.read_at(&mut dbuf, abs as u64).unwrap_or(0);
                                dbuf.truncate(got);
                                let fname = format!(
                                    "/sdcard/Android/media/jp.co.cygames.umamusume/hachimi/zdict_{:x}.bin",
                                    abs
                                );
                                let saved = std::fs::write(&fname, &dbuf).is_ok();
                                let map_name = cols.get(5).copied().unwrap_or("");
                                hits.push(format!(
                                    "0x{:x} saved={} bytes={} map={}",
                                    abs, saved, got, map_name
                                ));
                                if hits.len() >= max_hits {
                                    break 'outer;
                                }
                            }
                        }
                        off += chunk;
                    }
                }
            }
        }
        format!(
            r#"{{"needle":"37a430ec","hits":{},"locations":[{}],"note":"raw-content dicts have no magic; if 0 hits use /debug/mem_scan_hex"}}"#,
            hits.len(),
            hits.iter()
                .map(|h| format!("\"{}\"", json_escape(h)))
                .collect::<Vec<_>>()
                .join(",")
        )
    } else if path.starts_with("/debug/mem_scan_hex") {
        // ★ v3.24.63: arbitrary <=32B hex pattern scan across readable maps
        let hexq = parse_query(&full_uri, "hex");
        let mut needle: Vec<u8> = Vec::new();
        let hb = hexq.as_bytes();
        let mut i = 0;
        while i + 1 < hb.len() && needle.len() < 32 {
            if let Ok(b) = u8::from_str_radix(&hexq[i..i + 2], 16) {
                needle.push(b);
            }
            i += 2;
        }
        let max_hits: usize = parse_query(&full_uri, "max").parse().unwrap_or(8);
        let mut hits: Vec<String> = Vec::new();
        if needle.is_empty() {
            let body = r#"{"error":"empty_needle","usage":"/debug/mem_scan_hex?hex=37a430ec"}"#
                .to_string();
            let resp = format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", body.len(), body);
            let _ = stream.write_all(resp.as_bytes());
            return;
        }
        if let Ok(maps) = std::fs::read_to_string("/proc/self/maps") {
            if let Ok(mem) = std::fs::File::open("/proc/self/mem") {
                use std::os::unix::fs::FileExt;
                'outer: for line in maps.lines() {
                    let cols: Vec<&str> = line.split_whitespace().collect();
                    if cols.len() < 2 || !cols[1].starts_with('r') {
                        continue;
                    }
                    let range: Vec<&str> = cols[0].split('-').collect();
                    if range.len() != 2 {
                        continue;
                    }
                    let (Ok(sa), Ok(ea)) = (
                        usize::from_str_radix(range[0], 16),
                        usize::from_str_radix(range[1], 16),
                    ) else {
                        continue;
                    };
                    let len = ea - sa;
                    if len < 4096 || len > 1024 * 1024 * 1024 {
                        continue;
                    }
                    let mut off = 0usize;
                    while off < len {
                        let chunk = (8 * 1024 * 1024usize).min(len - off);
                        let mut buf = vec![0u8; chunk];
                        if mem.read_at(&mut buf, (sa + off) as u64).is_err() {
                            break;
                        }
                        for (i, w) in buf.windows(needle.len()).enumerate() {
                            if w == needle.as_slice() {
                                let abs = sa + off + i;
                                let tail_s = i + needle.len();
                                let tail_e = (tail_s + 24).min(buf.len());
                                hits.push(format!(
                                    "0x{:x} {} {}",
                                    abs,
                                    hex_encode(&buf[tail_s..tail_e]),
                                    cols.get(5).copied().unwrap_or("")
                                ));
                                if hits.len() >= max_hits {
                                    break 'outer;
                                }
                            }
                        }
                        off += chunk;
                    }
                }
            }
        }
        format!(
            r#"{{"hits":{},"locations":[{}]}}"#,
            hits.len(),
            hits.iter()
                .map(|h| format!("\"{}\"", json_escape(h)))
                .collect::<Vec<_>>()
                .join(",")
        )
    } else if path.starts_with("/debug/file_scan_hex") {
        // ★ v3.24.64: scan device files for a hex pattern.
        // path= empty -> scan every file-backed .so/.dat region listed in maps (dedup).
        // Reports file offset hits with 24 bytes of trailing context.
        let hexq = parse_query(&full_uri, "hex");
        let mut needle: Vec<u8> = Vec::new();
        let hb = hexq.as_bytes();
        let mut i = 0;
        while i + 1 < hb.len() && needle.len() < 64 {
            if let Ok(b) = u8::from_str_radix(&hexq[i..i + 2], 16) {
                needle.push(b);
            }
            i += 2;
        }
        let max_hits: usize = parse_query(&full_uri, "max").parse().unwrap_or(8);
        let pathq = parse_query(&full_uri, "path");
        let mut targets: Vec<String> = Vec::new();
        if !pathq.is_empty() {
            targets.push(pathq);
        } else if let Ok(maps) = std::fs::read_to_string("/proc/self/maps") {
            for line in maps.lines() {
                let cols: Vec<&str> = line.split_whitespace().collect();
                if let Some(name) = cols.get(5) {
                    if (name.ends_with(".so") || name.ends_with(".apk") || name.contains("/dat/"))
                        && !targets.iter().any(|t| t == name)
                    {
                        targets.push(name.to_string());
                    }
                }
            }
        }
        let mut hits: Vec<String> = Vec::new();
        if needle.is_empty() {
            hits.push("error: empty needle, use ?hex=37a430ec".to_string());
        } else {
            use std::io::Read;
            'files: for t in &targets {
                if let Ok(mut f) = std::fs::File::open(t) {
                    let mut fbuf: Vec<u8> = Vec::new();
                    if f.read_to_end(&mut fbuf).is_err() {
                        continue;
                    }
                    for (i, w) in fbuf.windows(needle.len()).enumerate() {
                        if w == needle.as_slice() {
                            let ts = i + needle.len();
                            let te = (ts + 24).min(fbuf.len());
                            hits.push(format!("{}@0x{:x} {}", t, i, hex_encode(&fbuf[ts..te])));
                            if hits.len() >= max_hits {
                                break 'files;
                            }
                        }
                    }
                }
            }
        }
        format!(
            r#"{{"targets":{},"hits":{},"locations":[{}]}}"#,
            targets.len(),
            hits.len(),
            hits.iter()
                .map(|h| format!("\"{}\"", json_escape(h)))
                .collect::<Vec<_>>()
                .join(",")
        )
    } else if path == "/debug/maps_list" {
        // ★ v3.24.65: list file-backed memory maps (find libzstd / codec hosts)
        let filter = parse_query(&full_uri, "filter");
        let mut out: Vec<String> = Vec::new();
        if let Ok(maps) = std::fs::read_to_string("/proc/self/maps") {
            for line in maps.lines() {
                let cols: Vec<&str> = line.split_whitespace().collect();
                if let Some(name) = cols.get(5) {
                    if name.starts_with('/') && (filter.is_empty() || name.contains(&filter)) {
                        let e = format!("{} {}", cols[0], name);
                        if !out.contains(&e) {
                            out.push(e);
                        }
                    }
                }
            }
        }
        format!(
            r#"{{"count":{},"maps":[{}]}}"#,
            out.len(),
            out.iter()
                .map(|h| format!("\"{}\"", json_escape(h)))
                .collect::<Vec<_>>()
                .join(",")
        )
    } else if path.starts_with("/debug/file_range_hex") {
        // ★ v3.24.67: read a byte range of a maps-listed file, return hex (chunked RE)
        let want = parse_query(&full_uri, "path");
        let off_str = parse_query(&full_uri, "offset");
        let len_str = parse_query(&full_uri, "len");
        let off = usize::from_str_radix(off_str.trim_start_matches("0x"), 16).unwrap_or(0);
        let max_len: usize = len_str.parse().unwrap_or(65536).min(4 * 1024 * 1024);
```

## inherit

### `fn inherit_pair_compat_endpoint`
matches=1

```rust
on":null,"status":"none"}"#.to_string(),
        Err(error) => format!(r#"{{"ok":false,"error":"query_session:{}"}}"#, json_escape(&error.to_string())),
    }
}

fn storage_flush_endpoint() -> String {
    let session_id = match ensure_observation_session() {
        Ok(value) => value,
        Err(error) => return format!(r#"{{"ok":false,"error":"{}"}}"#, json_escape(&error)),
    };
    let connection = match open_observation_storage() {
        Ok(value) => value,
        Err(error) => return format!(r#"{{"ok":false,"error":"{}"}}"#, json_escape(&error)),
    };
    let now = sniff_timestamp_ms();
    if let Err(error) = connection.execute_batch("PRAGMA wal_checkpoint(FULL);") {
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
    format!(r#"{{"ok":true,"session_id":"{}","last_flush_ms":{},"checkpoint":"full"}}"#, json_escape(&session_id), now)
}

fn storage_recover_endpoint() -> String {
    let connection = match open_observation_storage() {
        Ok(value) => value,
        Err(error) => return format!(r#"{{"ok":false,"error":"{}"}}"#, json_escape(&error)),
    };
    let process_start_token = observation_process_start_token();
    let recovered = match connection.execute(
        "UPDATE observation_sessions
         SET state='interrupted', recovered_after_restart=1
         WHERE state='open' AND process_start_token<>?1",
        rusqlite::params![process_start_token],
    ) {
        Ok(value) => value,
        Err(error) => return format!(r#"{{"ok":false,"error":"recover:{}"}}"#, json_escape(&error.to_string())),
    };
    match ensure_observation_session() {
        Ok(session_id) => format!(r#"{{"ok":true,"recovered_session_count":{},"current_session_id":"{}"}}"#, recovered, json_escape(&session_id)),
        Err(error) => format!(r#"{{"ok":false,"error":"{}","recovered_session_count":{}}}"#, json_escape(&error), recovered),
    }
}

// ===== Unified inheritance pair compatibility C-stage =====
fn inherit_pair_compat_endpoint(uri: &str) -> String {
    let pairs = match parse_query_pairs(uri) {
        Ok(value) => value,
        Err(error) => return format!(r#"{{"ok":false,"error":"{}"}}"#, json_escape(&error)),
    };
    if pairs.iter().filter(|(key, _)| key == "chara_id_a").count() != 1 || pairs.iter().filter(|(key, _)| key == "chara_id_b").count() != 1 { return r#"{\"ok\":false,\"error\":\"missing_or_duplicate_character_key\"}"#.to_string(); }
    let chara_id_a = match query_pair(&pairs, "chara_id_a").parse::<i32>() {
        Ok(value) if value > 0 => value,
        _ => return r#"{"ok":false,"error":"invalid_or_missing_chara_id_a"}"#.to_string(),
    };
    let chara_id_b = match query_pair(&pairs, "chara_id_b").parse::<i32>() {
        Ok(value) if value > 0 => value,
        _ => return r#"{"ok":false,"error":"invalid_or_missing_chara_id_b"}"#.to_string(),
    };
    let mdb_path = match find_mdb_path() {
        Some(value) => value,
        None => return r#"{"ok":false,"error":"mdb_not_found"}"#.to_string(),
    };
    let connection = match Connection::open_with_flags(&mdb_path, OpenFlags::SQLITE_OPEN_READ_ONLY) {
        Ok(value) => value,
        Err(error) => return format!(r#"{{"ok":false,"error":"mdb_open_failed","detail":"{}"}}"#, json_escape(&error.to_string())),
    };
    for (label, value) in [("chara_id_a", chara_id_a), ("chara_id_b", chara_id_b)] {
        let exists = connection.query_row("SELECT EXISTS(SELECT 1 FROM chara_data WHERE id=?1)", rusqlite::params![value], |row| row.get::<_, i64>(0));
        match exists { Ok(1) => {}, Ok(_) => return format!(r#"{{\"ok\":false,\"error\":\"character_not_found\",\"field\":\"{}\",\"value\":{}}}"#, label, value), Err(error) => return format!(r#"{{\"ok\":false,\"error\":\"character_validation_failed\",\"detail\":\"{}\"}}"#, json_escape(&error.to_string())) }
    }
    let mut statement = match connection.prepare(
        "SELECT DISTINCT r.relation_type, r.relation_point
         FROM succession_relation r
         INNER JOIN succession_relation_member a
             ON a.relation_type = r.relation_type AND a.chara_id = ?1
         INNER JOIN succession_relation_member b
             ON b.relation_type = r.relation_type AND b.chara_id = ?2
         ORDER BY r.relation_type, r.relation_point"
    ) {
        Ok(value) => value,
        Err(error) => return format!(r#"{{"ok":false,"error":"pair_query_prepare_failed","detail":"{}"}}"#, json_escape(&error.to_string())),
    };
    let mapped = match statement.query_map(rusqlite::params![chara_id_a, chara_id_b], |row| {
        Ok((row.get::<_, i32>(0)?, row.get::<_, i32>(1)?))
    }) {
        Ok(value) => value,
        Err(error) => return format!(r#"{{"ok":false,"error":"pair_query_failed","detail":"{}"}}"#, json_escape(&error.to_string())),
    };
    let mut relation_items = Vec::new();
    let mut base_compatibility = 0i64;
    for row in mapped {
        let (relation_type, relation_point) = match row {
            Ok(value) => value,
            Err(error) => return format!(r#"{{"ok":false,"error":"pair_row_decode_failed","detail":"{}"}}"#, json_escape(&error.to_string())),
        };
        base_compatibility += i64::from(relation_point);
        relation_items.push(format!(
            r#"{{"relation_type":{},"relation_point":{},"chara_id_a_member":true,"chara_id_b_member":true}}"#,
            relation_type, relation_point
        ));
    }
    format!(
        r#"{{"ok":true,"source":"current_mdb","calculation":"sum_shared_succession_relation_points","chara_id_a":{},"chara_id_b":{},"shared_relation_count":{},"base_compatibility":{},"shared_relations":[{}],"race_bonus":null,"specific_trained_chara_adjustments":null,"runtime_consumer_result":null,"scope":"character_pair_base_only"}}"#,
        chara_id_a, chara_id_b, relation_items.len(), base_compatibility, relation_items.join(",")
    )
}

// ===== Unified selected inheritance parents D-stage =====
unsafe fn inherit_selected_parent_runtime_endpoint() -> String {
    if API.is_null() {
        return r#"{"ok":false,"error":"api_null"}"#.to_string();
    }
    let image = get_image();
    if image.is_null() {
        return r#"{"ok":false,"error":"image_null"}"#.to_string();
    }
    let wdm_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkDataManager").as_ptr(),
    );
    if wdm_class.is_null() {
        return r#"{"ok":false,"error":"work_data_manager_class_not_found"}"#.to_string();
    }
    let wdm = get_singleton(wdm_class);
    if wdm.is_null() {
        return r#"{"ok":false,"error":"work_data_manager_instance_not_found"}"#.to_string();
    }
    let single_mode_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkSingleModeData").as_ptr(),
    );
    if single_mode_class.is_null() {
        return r#"{"ok":false,"error":"work_single_mode_data_class_not_found"}"#.to_string();
    }
    let single_mode = call_getter_ref(wdm_class, wdm, "get_SingleMode");
    if single_mode.is_null() {
        return r#"{"ok":false,"error":"single_mode_instance_not_found"}"#.to_string();
    }
    let chara_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkSingleModeCharaData").as_ptr(),
    );
    if chara_class.is_null() {
        return r#"{"ok":false,"error":"work_single_mode_chara_data_class_not_found"}"#.to_string();
    }
    let chara = call_getter_ref(single_mode_class, single_mode, "get_Character");
    if chara.is_null() {
        return r#"{"ok":false,"error":"single_mode_character_instance_not_found"}"#.to_string();
    }
    let succession_info_class = find_class_by_short_name(image, "SuccessionCharaInfo");
    if succession_info_class.is_null() {
        return r#"{"ok":false,"error":"succession_chara_info_class_not_found"}"#.to_string();
    }

    let target_card_id = call_getter_int(chara_class, chara, "get_CardId");
    let target_chara_id = call_getter_int(chara_class, chara, "get_CharaId");
    let first = call_getter_ref(
        chara_class,
        chara,
        "get_SuccessionTrainedCharaInfoFirst",
    );
    let second = call_getter_ref(
        chara_class,
        chara,
        "get_SuccessionTrainedCharaInfoSecond",
    );

    let render_slot = |slot: &str, info: *mut c_void| -> String {
        if info.is_null() {
            return format!(
                r#"{{"slot":"{}","selected":false,"trained_chara_id":null,"trained_chara_record":null}}"#,
                slot
            );
        }
        let trained_chara_id = call_getter_obscured_int(
            succession_info_class,
            info,
            "get_TrainedCharaId",
        );
        format!(
            r#"{{"slot":"{}","selected":true,"trained_chara_id":{},"trained_chara_record":null}}"#,
            slot, trained_chara_id
        )
    };

    format!(
        r#"{{"ok":true,"source":"current_work_single_mode_character","scope":"selected_parent_ids_only","target":{{"
```

### `fn inherit_selected_parent_runtime_endpoint`
matches=1

```rust
ELECT 1 FROM chara_data WHERE id=?1)", rusqlite::params![value], |row| row.get::<_, i64>(0));
        match exists { Ok(1) => {}, Ok(_) => return format!(r#"{{\"ok\":false,\"error\":\"character_not_found\",\"field\":\"{}\",\"value\":{}}}"#, label, value), Err(error) => return format!(r#"{{\"ok\":false,\"error\":\"character_validation_failed\",\"detail\":\"{}\"}}"#, json_escape(&error.to_string())) }
    }
    let mut statement = match connection.prepare(
        "SELECT DISTINCT r.relation_type, r.relation_point
         FROM succession_relation r
         INNER JOIN succession_relation_member a
             ON a.relation_type = r.relation_type AND a.chara_id = ?1
         INNER JOIN succession_relation_member b
             ON b.relation_type = r.relation_type AND b.chara_id = ?2
         ORDER BY r.relation_type, r.relation_point"
    ) {
        Ok(value) => value,
        Err(error) => return format!(r#"{{"ok":false,"error":"pair_query_prepare_failed","detail":"{}"}}"#, json_escape(&error.to_string())),
    };
    let mapped = match statement.query_map(rusqlite::params![chara_id_a, chara_id_b], |row| {
        Ok((row.get::<_, i32>(0)?, row.get::<_, i32>(1)?))
    }) {
        Ok(value) => value,
        Err(error) => return format!(r#"{{"ok":false,"error":"pair_query_failed","detail":"{}"}}"#, json_escape(&error.to_string())),
    };
    let mut relation_items = Vec::new();
    let mut base_compatibility = 0i64;
    for row in mapped {
        let (relation_type, relation_point) = match row {
            Ok(value) => value,
            Err(error) => return format!(r#"{{"ok":false,"error":"pair_row_decode_failed","detail":"{}"}}"#, json_escape(&error.to_string())),
        };
        base_compatibility += i64::from(relation_point);
        relation_items.push(format!(
            r#"{{"relation_type":{},"relation_point":{},"chara_id_a_member":true,"chara_id_b_member":true}}"#,
            relation_type, relation_point
        ));
    }
    format!(
        r#"{{"ok":true,"source":"current_mdb","calculation":"sum_shared_succession_relation_points","chara_id_a":{},"chara_id_b":{},"shared_relation_count":{},"base_compatibility":{},"shared_relations":[{}],"race_bonus":null,"specific_trained_chara_adjustments":null,"runtime_consumer_result":null,"scope":"character_pair_base_only"}}"#,
        chara_id_a, chara_id_b, relation_items.len(), base_compatibility, relation_items.join(",")
    )
}

// ===== Unified selected inheritance parents D-stage =====
unsafe fn inherit_selected_parent_runtime_endpoint() -> String {
    if API.is_null() {
        return r#"{"ok":false,"error":"api_null"}"#.to_string();
    }
    let image = get_image();
    if image.is_null() {
        return r#"{"ok":false,"error":"image_null"}"#.to_string();
    }
    let wdm_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkDataManager").as_ptr(),
    );
    if wdm_class.is_null() {
        return r#"{"ok":false,"error":"work_data_manager_class_not_found"}"#.to_string();
    }
    let wdm = get_singleton(wdm_class);
    if wdm.is_null() {
        return r#"{"ok":false,"error":"work_data_manager_instance_not_found"}"#.to_string();
    }
    let single_mode_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkSingleModeData").as_ptr(),
    );
    if single_mode_class.is_null() {
        return r#"{"ok":false,"error":"work_single_mode_data_class_not_found"}"#.to_string();
    }
    let single_mode = call_getter_ref(wdm_class, wdm, "get_SingleMode");
    if single_mode.is_null() {
        return r#"{"ok":false,"error":"single_mode_instance_not_found"}"#.to_string();
    }
    let chara_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkSingleModeCharaData").as_ptr(),
    );
    if chara_class.is_null() {
        return r#"{"ok":false,"error":"work_single_mode_chara_data_class_not_found"}"#.to_string();
    }
    let chara = call_getter_ref(single_mode_class, single_mode, "get_Character");
    if chara.is_null() {
        return r#"{"ok":false,"error":"single_mode_character_instance_not_found"}"#.to_string();
    }
    let succession_info_class = find_class_by_short_name(image, "SuccessionCharaInfo");
    if succession_info_class.is_null() {
        return r#"{"ok":false,"error":"succession_chara_info_class_not_found"}"#.to_string();
    }

    let target_card_id = call_getter_int(chara_class, chara, "get_CardId");
    let target_chara_id = call_getter_int(chara_class, chara, "get_CharaId");
    let first = call_getter_ref(
        chara_class,
        chara,
        "get_SuccessionTrainedCharaInfoFirst",
    );
    let second = call_getter_ref(
        chara_class,
        chara,
        "get_SuccessionTrainedCharaInfoSecond",
    );

    let render_slot = |slot: &str, info: *mut c_void| -> String {
        if info.is_null() {
            return format!(
                r#"{{"slot":"{}","selected":false,"trained_chara_id":null,"trained_chara_record":null}}"#,
                slot
            );
        }
        let trained_chara_id = call_getter_obscured_int(
            succession_info_class,
            info,
            "get_TrainedCharaId",
        );
        format!(
            r#"{{"slot":"{}","selected":true,"trained_chara_id":{},"trained_chara_record":null}}"#,
            slot, trained_chara_id
        )
    };

    format!(
        r#"{{"ok":true,"source":"current_work_single_mode_character","scope":"selected_parent_ids_only","target":{{"card_id":{},"chara_id":{}}},"parents":[{},{}],"trained_chara_record_resolution":null,"ancestor_tree":null,"pair_compatibility":null,"race_bonus":null,"runtime_consumer_result":null,"id_semantics":"trained_chara_id","getter_decode":"obscured_int_runtime_invoke_path","runtime_validation":"executed"}}"#,
        target_card_id,
        target_chara_id,
        render_slot("first", first),
        render_slot("second", second),
    )
}

// ===== Unified runtime correction E-stage =====
// ===== Unified pre-release correction F-stage =====
// ===== Unified release-gate correction G-stage =====
// ===== Unified response-header capture H-stage =====
// ===== Unified selected-parent multi-source resolver I-stage =====
unsafe fn find_exact_instance_method(
    class: *mut c_void,
    name: &str,
    parameter_types: &[&str],
) -> *const c_void {
    let get_methods_ptr = resolve_il2cpp_symbol("il2cpp_class_get_methods");
    let get_name_ptr = resolve_il2cpp_symbol("il2cpp_method_get_name");
    let get_param_count_ptr = resolve_il2cpp_symbol("il2cpp_method_get_param_count");
    let get_param_ptr = resolve_il2cpp_symbol("il2cpp_method_get_param");
    let get_type_name_ptr = resolve_il2cpp_symbol("il2cpp_type_get_name");
    if class.is_null() || get_methods_ptr.is_null() || get_name_ptr.is_null()
        || get_param_count_ptr.is_null() || get_param_ptr.is_null() || get_type_name_ptr.is_null() {
        return ptr::null();
    }
    let get_methods: FnClassGetMethods = std::mem::transmute(get_methods_ptr);
    let get_name: FnMethodGetName = std::mem::transmute(get_name_ptr);
    let get_param_count: unsafe extern "C" fn(*const c_void) -> u32 = std::mem::transmute(get_param_count_ptr);
    let get_param: unsafe extern "C" fn(*const c_void, u32) -> *const c_void = std::mem::transmute(get_param_ptr);
    let get_type_name: unsafe extern "C" fn(*const c_void) -> *const c_char = std::mem::transmute(get_type_name_ptr);
    let mut iterator = ptr::null_mut();
    let mut found: *const c_void = ptr::null();
    loop {
        let method = get_methods(class, &mut iterator);
        if method.is_null() { break; }
        if il2cpp_c_string(get_name(method)) != name || get_param_count(method) as usize != parameter_types.len() { continue; }
        let mut exact = true;
        for (index, expected) in parameter_types.iter().enumerate() {
            let parameter = get_param(method, index as u32);
            if parameter.is_null() || il2cpp_c_string(get_type_name(parameter)) != *expected { exact = false; break; }
        }
        if exact {
            if !found.is_null() { return ptr::null(); }
            found = method;
        }
    }
    found
}

unsafe fn invoke_parent_store_get(store_class: *mut c_void, store: *mut c_void, trained_chara_id: i32) -> *mut c_void {
    let method = find_exact_instance_method(store_class, "Get", &["System.Int32", "System.Boolean"]);
    let invoke_ptr = resolve_il2cpp_symbol("il2cpp_runtime_invoke");
    if method.is_null() || invoke_ptr.is_null() { return ptr::null_mut(); }
    let invoke: unsafe extern "C" fn(*const c_void, *mut c_void, *mut *mut c_void, *mut *mut c_void) -> *mut c_void = std::mem::transmute(invoke_ptr);
    let mut id = trained_chara_id;
    let mut all = true;
    let mut arguments = [
        (&mut id as *mut i32).cast::<c_void>(),
        (&mut all as *mut bool).cast::<c_void>(),
    ];
    let mut exception = ptr::null_mut();
    let result = invoke(method, store, arguments.as_mut_ptr(), &mut exception);
    if exception.is_null() { result } else { ptr::null_mut() }
}

unsafe fn selected_parent_record_json(
    slot: &str,
    trained_chara_id: i32,
    own_store_class: *mut c_void,
    own_store: *mut c_void,
    succession_store_class: *mut c_void,
    succession_store: *mut c_void,
    record_class: *mut c_void,
) -> String {
    let mut source = "not_found";
    let mut record = if own_store.is_null() { ptr::null_mut() } else {
        invoke_
```

### `fn inherit_selected_parent_records_endpoint`
matches=1

```rust
:mem::transmute(invoke_ptr);
    let mut id = trained_chara_id;
    let mut all = true;
    let mut arguments = [
        (&mut id as *mut i32).cast::<c_void>(),
        (&mut all as *mut bool).cast::<c_void>(),
    ];
    let mut exception = ptr::null_mut();
    let result = invoke(method, store, arguments.as_mut_ptr(), &mut exception);
    if exception.is_null() { result } else { ptr::null_mut() }
}

unsafe fn selected_parent_record_json(
    slot: &str,
    trained_chara_id: i32,
    own_store_class: *mut c_void,
    own_store: *mut c_void,
    succession_store_class: *mut c_void,
    succession_store: *mut c_void,
    record_class: *mut c_void,
) -> String {
    let mut source = "not_found";
    let mut record = if own_store.is_null() { ptr::null_mut() } else {
        invoke_parent_store_get(own_store_class, own_store, trained_chara_id)
    };
    if !record.is_null() { source = "trained_chara_data"; }
    if record.is_null() && !succession_store.is_null() {
        record = invoke_parent_store_get(succession_store_class, succession_store, trained_chara_id);
        if !record.is_null() { source = "succession_only_chara_data"; }
    }
    if record.is_null() {
        return format!(r#"{{"slot":"{}","trained_chara_id":{},"resolved":false,"source":"{}","record":null}}"#,
            slot, trained_chara_id, source);
    }
    // Runtime MethodInfo says Id/CardId return System.Int32, while CharaId returns ObscuredInt.
    let id = call_getter_int(record_class, record, "get_Id");
    let card_id = call_getter_int(record_class, record, "get_CardId");
    let chara_id = call_getter_obscured_int(record_class, record, "get_CharaId");
    // Boolean return values are boxed with one payload byte at object + 0x10.
    let boxed_bool = |method_name: &str| -> bool {
        let boxed = call_getter_ref(record_class, record, method_name);
        !boxed.is_null() && std::ptr::read_unaligned::<u8>((boxed as *const u8).add(16)) != 0
    };
    let is_player = boxed_bool("get_IsPlayer");
    let is_rental = boxed_bool("get_IsRental");
    let is_others = boxed_bool("get_IsOthers");
    let is_succession_only = boxed_bool("get_IsSuccessionOnly");
    format!(r#"{{"slot":"{}","trained_chara_id":{},"resolved":true,"source":"{}","record":{{"id":{},"card_id":{},"chara_id":{},"is_player":{},"is_rental":{},"is_others":{},"is_succession_only":{}}}}}"#,
        slot, trained_chara_id, source, id, card_id, chara_id, is_player, is_rental, is_others, is_succession_only)
}

unsafe fn inherit_selected_parent_records_endpoint() -> String {
    if API.is_null() { return r#"{"ok":false,"error":"api_null"}"#.to_string(); }
    let image = get_image();
    if image.is_null() { return r#"{"ok":false,"error":"image_null"}"#.to_string(); }
    let wdm_class = find_class(image, to_cstr("Gallop").as_ptr(), to_cstr("WorkDataManager").as_ptr());
    let single_mode_class = find_class(image, to_cstr("Gallop").as_ptr(), to_cstr("WorkSingleModeData").as_ptr());
    let chara_class = find_class(image, to_cstr("Gallop").as_ptr(), to_cstr("WorkSingleModeCharaData").as_ptr());
    let own_store_class = find_class(image, to_cstr("Gallop").as_ptr(), to_cstr("WorkTrainedCharaData").as_ptr());
    let succession_store_class = find_class(image, to_cstr("Gallop").as_ptr(), to_cstr("WorkSuccessionOnlyCharaData").as_ptr());
    let record_class = find_class_by_short_name(image, "TrainedCharaData");
    let succession_info_class = find_class_by_short_name(image, "SuccessionCharaInfo");
    if wdm_class.is_null() || single_mode_class.is_null() || chara_class.is_null()
        || own_store_class.is_null() || succession_store_class.is_null()
        || record_class.is_null() || succession_info_class.is_null() {
        return r#"{"ok":false,"error":"required_class_not_found"}"#.to_string();
    }
    let wdm = get_singleton(wdm_class);
    if wdm.is_null() { return r#"{"ok":false,"error":"work_data_manager_instance_not_found"}"#.to_string(); }
    let single_mode = call_getter_ref(wdm_class, wdm, "get_SingleMode");
    let chara = call_getter_ref(single_mode_class, single_mode, "get_Character");
    if single_mode.is_null() || chara.is_null() { return r#"{"ok":false,"error":"single_mode_character_not_found"}"#.to_string(); }
    let own_store = call_getter_ref(wdm_class, wdm, "get_TrainedCharaData");
    let succession_store = call_getter_ref(wdm_class, wdm, "get_SuccessionOnlyCharaData");
    let first_info = call_getter_ref(chara_class, chara, "get_SuccessionTrainedCharaInfoFirst");
    let second_info = call_getter_ref(chara_class, chara, "get_SuccessionTrainedCharaInfoSecond");
    let first_id = if first_info.is_null() { 0 } else { call_getter_obscured_int(succession_info_class, first_info, "get_TrainedCharaId") };
    let second_id = if second_info.is_null() { 0 } else { call_getter_obscured_int(succession_info_class, second_info, "get_TrainedCharaId") };
    let first = selected_parent_record_json("first", first_id, own_store_class, own_store, succession_store_class, succession_store, record_class);
    let second = selected_parent_record_json("second", second_id, own_store_class, own_store, succession_store_class, succession_store, record_class);
    format!(r#"{{"ok":true,"scope":"selected_parent_record_multisource","lookup_order":["trained_chara_data","succession_only_chara_data"],"selected_temp_lookup":"via_succession_only_get_all_contract","selected_temp_runtime_hit":"pending_device_execution","parents":[{},{}],"ancestor_tree":null,"race_bonus":null,"full_compatibility":null,"runtime_validation":"pending_device_execution"}}"#, first, second)
}

// ===== Unified selected-parent multi-source resolver I-stage =====
// ===== Selected-parent runtime semantics J-stage =====
/// 辅助函数：IL2CPP类型枚举转可读名称
fn type_enum_to_name(te: u8) -> String {
    match te {
        1 => "void".to_string(),
        2 => "boolean".to_string(),
        3 => "char".to_string(),
        4 => "i1".to_string(),
        5 => "u1".to_string(),
        6 => "i2".to_string(),
        7 => "u2".to_string(),
        8 => "i4".to_string(),
        9 => "u4".to_string(),
        10 => "i8".to_string(),
        11 => "u8".to_string(),
        12 => "r4".to_string(),
        13 => "r8".to_string(),
        14 => "string".to_string(),
        17 => "ptr".to_string(),
        18 => "byref".to_string(),
        21 => "valuetype".to_string(),
        22 => "class".to_string(),
        24 => "array".to_string(),
        25 => "genericinst".to_string(),
        28 => "cmplx".to_string(),
        29 => "fnptr".to_string(),
        30 => "object".to_string(),
        _ => format!("type_{}", te),
    }
}

/// v3.22.89: /il2cpp/search_methods_page — 搜索方法名HTML页面（A-Z分组下载）
fn search_methods_page() -> String {
    let letters = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let mut btns = String::new();
    for ch in letters.chars() {
        btns.push_str(&format!(
            r#"<button class="g" onclick="goLetter('{}')">{}</button> "#,
            ch, ch
        ));
    }
    format!(
        r#"<!DOCTYPE html><html><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Search Methods</title><style>body{{font-family:system-ui;max-width:600px;margin:12px auto;padding:0 8px;background:#1a1a2e;color:#e0e0e0}}h1{{color:#4fc3f7;font-size:1.2em;margin:8px 0}}.g{{display:inline-block;margin:4px 2px;padding:8px 12px;background:#16213e;border:1px solid #333;border-radius:4px;color:#fff;cursor:pointer;font-size:14px;min-width:36px;text-align:center}}.g:disabled{{background:#555;color:#333;cursor:default}}.g.ok{{background:#2e7d32;border-color:#4caf50}}.g.run{{background:#e65100;border-color:#ff9800}}input{{width:100%;padding:8px;background:#16213e;border:1px solid #333;border-radius:4px;color:#fff;box-sizing:border-box;font-size:16px}}.p{{margin:8px 0;font-size:0.95em}}.ok{{color:#4caf50}}.err{{color:#ff5252}}#lst{{margin:8px 0;font-size:0.8em;color:#aaa;max-height:300px;overflow-y:auto}}</style></head><body><h1>IL2CPP Method Search</h1><input id="kw" placeholder="keyword (e.g. Motivation)" value="Motivation"><div style="margin:8px 0">{}</div><div class="p">Click a letter to search classes starting with that letter, or click ALL for all classes. Results download as JSON.</div><div class="p" id="pg">Ready</div><div id="lst"></div><script>function goLetter(ch){{var kw=document.getElementById("kw").value;if(!kw){{document.getElementById("pg").innerHTML='<span class="err">Enter a keyword first</span>';return;}}var btn=event.target;btn.disabled=true;btn.className="g run";var url="/il2cpp/search_methods_dl?keyword="+encodeURIComponent(kw)+"&letter="+ch;document.getElementById("pg").innerHTML='<span class="ok">Searching '+ch+'...</span>';fetch(url).then(r=>{{if(!r.ok)throw new Error("HTTP "+r.status);return r.blob();}}).then(blob=>{{var url2=URL.createObjectURL(blob);var a=document.createElement("a");a.href=url2;a.download="search_methods_"+ch+"_"+kw+".json";a.click();URL.revokeObjectURL(url2);btn.className="g ok";btn.disabled=false;document.getElementById("pg").innerHTML='<span class="ok">'+ch+': downloaded!</span>';}}).catch(e=>{{btn.className="g ok";btn.disabled=false;document.getElementById("pg").innerHTML='<span class="err">Error: '+e+'</span>';}});}}</script></body></html>"#,
        btns
    )
}

/// v3.22.89: /il2cpp/search_methods?keyword=X — 跨类搜索方法名
/// 遍历所有IL2CPP类的方法表，按方法名关键词过滤，返回匹配的类名+方法名
/// 用于定位やる気系数等散落在各类中的计算方法
unsafe fn il2cpp_search_methods(keyword: &str, letter: &str) -> String {
   
```

## protocol

### `fn persist_protocol_capture`
matches=1

```rust
equested.is_empty() { return r#"{"ok":false,"error":"missing_type"}"#.to_string(); }
    let class = find_class_by_full_declaring_name(&requested);
    if class.is_null() { return format!(r#"{{"ok":false,"error":"class_not_found_or_ambiguous","type":"{}"}}"#, json_escape(&requested)); }
    let nested_ptr = resolve_il2cpp_symbol("il2cpp_class_get_nested_types");
    if nested_ptr.is_null() { return r#"{"ok":false,"error":"il2cpp_class_get_nested_types_unavailable"}"#.to_string(); }
    let get_nested: unsafe extern "C" fn(*mut c_void, *mut *mut c_void) -> *mut c_void = std::mem::transmute(nested_ptr);
    let mut iterator = ptr::null_mut();
    let mut items = Vec::new();
    loop {
        let nested = get_nested(class, &mut iterator);
        if nested.is_null() { break; }
        items.push(format!(r#"{{"type":"{}","class_pointer":"0x{:x}"}}"#, json_escape(&class_full_declaring_name(nested)), nested as usize));
    }
    format!(r#"{{"ok":true,"requested":"{}","direct_only":true,"count":{},"nested_types":[{}]}}"#, json_escape(&requested), items.len(), items.join(","))
}

unsafe fn il2cpp_enum_values_capability(uri: &str) -> String {
    let pairs = match parse_query_pairs(uri) { Ok(value) => value, Err(error) => return format!(r#"{{"ok":false,"error":"{}"}}"#, json_escape(&error)) };
    let requested = query_pair(&pairs, "type");
    if requested.is_empty() { return r#"{\"ok\":false,\"error\":\"missing_type\"}"#.to_string(); }
    let required = ["il2cpp_class_get_fields", "il2cpp_field_get_flags", "il2cpp_field_static_get_value"];
    let available: Vec<bool> = required.iter().map(|name| !resolve_il2cpp_symbol(name).is_null()).collect();
    format!(r#"{{"ok":true,"requested":"{}","value_status":"unresolved","integer_values":null,"declaration_order_inference":false,"runtime_api":{{"il2cpp_class_get_fields":{},"il2cpp_field_get_flags":{},"il2cpp_field_static_get_value":{}}}}}"#,
        json_escape(&requested), available[0], available[1], available[2])
}

// ===== Unified observation persistent storage B-stage =====
static STORAGE_SESSION_ID: Mutex<Option<String>> = Mutex::new(None);
static STORAGE_LAST_FLUSH_MS: AtomicU64 = AtomicU64::new(0);
static STORAGE_LAST_ERROR: Mutex<Option<String>> = Mutex::new(None);

fn storage_set_error(error: &str) {
    if let Ok(mut value) = STORAGE_LAST_ERROR.lock() {
        *value = Some(error.to_string());
    }
}

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

fn observation_storage_root() -> std::path::PathBuf {
    if let Ok(command_line) = std::fs::read("/proc/self/cmdline") {
        let package_bytes = command_line.split(|byte| *byte == 0).next().unwrap_or(&[]);
        if let Ok(package_name) = std::str::from_utf8(package_bytes) {
            if !package_name.is_empty() {
                return std::path::PathBuf::from("/data/user/0")
                    .join(package_name)
                    .join("files")
                    .join("hlpatch-observations");
            }
        }
    }
    std::path::PathBuf::from("/data/user/0/jp.co.cygames.umamusume/files/hlpatch-observations")
}

fn observation_storage_db_path() -> std::path::PathBuf {
    observation_storage_root().join("index.sqlite")
}

fn open_observation_storage() -> Result<Connection, String> {
    let root = observation_storage_root();
    std::fs::create_dir_all(root.join("sessions")).map_err(|error| format!("create_sessions_dir:{}", error))?;
    std::fs::create_dir_all(root.join("blobs")).map_err(|error| format!("create_blobs_dir:{}", error))?;
    let db_path = observation_storage_db_path();
    let connection = Connection::open_with_flags(
        &db_path,
        OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE,
    ).map_err(|error| format!("open_index:{}", error))?;
    connection.execute_batch(
        "PRAGMA journal_mode=WAL;
         PRAGMA synchronous=FULL;
         PRAGMA foreign_keys=ON;
         CREATE TABLE IF NOT EXISTS storage_meta(
             key TEXT PRIMARY KEY NOT NULL,
             value TEXT NOT NULL
         );
         CREATE TABLE IF NOT EXISTS observation_sessions(
             session_id TEXT PRIMARY KEY NOT NULL,
             process_id INTEGER NOT NULL,
             process_start_token TEXT NOT NULL DEFAULT '',
             plugin_version TEXT NOT NULL,
             started_at_ms INTEGER NOT NULL,
             last_flush_ms INTEGER NOT NULL,
             state TEXT NOT NULL,
             recovered_after_restart INTEGER NOT NULL DEFAULT 0,
             root_path TEXT NOT NULL
         );
         CREATE TABLE IF NOT EXISTS observation_files(
             file_id INTEGER PRIMARY KEY AUTOINCREMENT,
             session_id TEXT NOT NULL,
             relative_path TEXT NOT NULL,
             content_type TEXT NOT NULL,
             byte_length INTEGER NOT NULL,
             sha256 TEXT,
             created_at_ms INTEGER NOT NULL,
             UNIQUE(session_id, relative_path),
             FOREIGN KEY(session_id) REFERENCES observation_sessions(session_id)
         );
         CREATE INDEX IF NOT EXISTS idx_observation_files_session_id_file_id
             ON observation_files(session_id, file_id);"
    ).map_err(|error| format!("initialize_schema:{}", error))?;
    let has_start_token = connection.prepare("PRAGMA table_info(observation_sessions)")
        .and_then(|mut statement| statement.query_map([], |row| row.get::<_, String>(1))
            .map(|rows| rows.filter_map(Result::ok).any(|name| name == "process_start_token")))
        .unwrap_or(false);
    if !has_start_token {
        connection.execute("ALTER TABLE observation_sessions ADD COLUMN process_start_token TEXT NOT NULL DEFAULT ''", [])
            .map_err(|error| format!("migrate_process_start_token:{}", error))?;
    }
    Ok(connection)
}

fn observation_process_start_token() -> String {
    let stat = std::fs::read_to_string("/proc/self/stat").unwrap_or_default();
    let start_ticks = stat.rsplit_once(')').map(|(_, tail)| tail.split_whitespace().nth(19).unwrap_or("")).unwrap_or("");
    format!("{}:{}", std::process::id(), start_ticks)
}

fn ensure_observation_session() -> Result<String, String> {
    if let Ok(value) = STORAGE_SESSION_ID.lock() {
        if let Some(session_id) = value.as_ref() {
            return Ok(session_id.clone());
        }
    }
    let connection = open_observation_storage()?;
    let now = sniff_timestamp_ms();
    let process_id = std::process::id();
    let process_start_token = observation_process_start_token();
    let session_id = format!("{}-{}", now, process_id);
    let root_text = observation_storage_root().to_string_lossy().into_owned();
    connection.execute(
        "UPDATE observation_sessions
         SET state='interrupted', recovered_after_restart=1
         WHERE state='open' AND process_start_token<>?1",
        rusqlite::params![process_start_token],
    ).map_err(|error| format!("recover_previous_sessions:{}", error))?;
    connection.execute(
        "INSERT INTO observation_sessions(
             session_id, process_id, process_start_token, plugin_version, started_at_ms,
             last_flush_ms, state, recovered_after_restart, root_path
         ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, 'open', 0, ?7)",
        rusqlite::params![session_id, process_id as i64, process_start_token, PLUGIN_VERSION, now as i64, now as i64, root_text],
    ).map_err(|error| format!("insert_session:{}", error))?;
    let session_direct
```

### `push_sniff_metadata`
matches=4

```rust
  id: u64,
    timestamp_ms: u64,
    method: String,
    path: String,
    body_size: usize,
    body_hex: String,
    content_type: String,
}
static UNITY_OBSERVATIONS: Mutex<Vec<UnityRequestObservation>> = Mutex::new(Vec::new());
// Completed response headers are keyed by the full request URL. Capture occurs
// immediately before Unity dispatches the completion callback that reaches the
// game's DecompressResponse path.
static UNITY_COMPLETED_RESPONSE_HEADERS: Mutex<Vec<(String, Vec<(String, String)>)>> = Mutex::new(Vec::new());

unsafe fn observe_unity_response_completion(operation: *mut c_void) {
    if operation.is_null() || !SNIFF_ENABLED.load(Ordering::Relaxed) { return; }
    let operation_class = get_class_from_object(operation);
    if operation_class.is_null() { return; }
    let request = call_getter_ref(operation_class, operation, "get_webRequest");
    if request.is_null() { return; }
    let url = unity_get_string(request, "get_url");
    if url.is_empty() || !url.contains("/umamusume/") { return; }
    let request_class = get_class_from_object(request);
    if request_class.is_null() { return; }
    let dictionary = call_getter_on_instance(request_class, request, "GetResponseHeaders");
    if dictionary.is_null() { return; }
    let headers = read_string_dict(dictionary);
    if let Ok(mut completed) = UNITY_COMPLETED_RESPONSE_HEADERS.lock() {
        completed.push((url, headers));
    }
}

unsafe fn take_unity_response_headers(url: &str) -> Option<Vec<(String, String)>> {
    UNITY_COMPLETED_RESPONSE_HEADERS.lock().ok().and_then(|mut completed| {
        let wanted = sniff_path(url);
        let index = completed.iter().position(|(candidate, _)| sniff_path(candidate) == wanted)?;
        Some(completed.remove(index).1)
    })
}
// Pending request body parking (CompressRequest → Post matching)
static mut PENDING_REQ_BODY: Option<Vec<u8>> = None;
static mut PENDING_COMPRESSED: usize = 0;

fn sniff_timestamp_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis().min(u128::from(u64::MAX)) as u64)
        .unwrap_or(0)
}

fn sniff_path(url: &str) -> String {
    let no_query = url.split('?').next().unwrap_or(url);
    if let Some(i) = no_query.find("://") {
        let rest = &no_query[i + 3..];
        return rest
            .find('/')
            .map(|j| rest[j..].to_string())
            .unwrap_or_else(|| "/".to_string());
    }
    no_query.to_string()
}

unsafe fn push_sniff_metadata(
    request_id: u64,
    direction: &'static str,
    url: &str,
    size: usize,
    body: &[u8],
    headers: Vec<(String, String)>,
) {
    let id = SNIFF_METADATA_ID.fetch_add(1, Ordering::Relaxed);
    let body_hex = body.iter().map(|b| format!("{:02x}", b)).collect::<String>();
    SNIFF_METADATA.push(SniffMetadata {
        id,
        request_id,
        timestamp_ms: sniff_timestamp_ms(),
        direction,
        path: sniff_path(url),
        size,
        body_hex,
        headers,
    });
    if SNIFF_METADATA.len() > SNIFF_METADATA_MAX {
        SNIFF_METADATA.remove(0);
    }
}
// ★ Mutex to prevent concurrent read_summary_inner calls from HTTP + push threads
static READ_MUTEX: Mutex<()> = Mutex::new(());

// ★ v3.24.2: Story event choice hook — capture career event choices (options, effects, branches)
static mut EVENT_CHOICE_HOOK_INSTALLED: bool = false;
static mut EVENT_CHOICE_ADDR: usize = 0; // StoryChoiceController.Choice
static mut EVENT_ADD_BTN_ADDR: usize = 0; // StoryChoiceController.AddChoiceButton
static mut ORIG_EVENT_CHOICE_PROLOGUE: [u8; 16] = [0; 16];
static mut ORIG_EVENT_ADD_BTN_PROLOGUE: [u8; 16] = [0; 16];
// ★ v3.24.2: StoryManager.SetStory hook — capture story_id and chara_id for event type identification
static mut STORY_SET_HOOK_INSTALLED: bool = false;
static mut STORY_SET_ADDR: usize = 0;
static mut ORIG_STORY_SET_PROLOGUE: [u8; 16] = [0; 16];
// Event state: accumulated choices for current event
static EVENT_STATE_MUTEX: Mutex<()> = Mutex::new(());

// ★ v3.24.40: mirror every ura_log line into a queryable ring buffer
// (Hachimi logcat was the only outlet before; /debug/hooklog exposes it).
static HOOK_LOG: Mutex<Vec<String>> = Mutex::new(Vec::new());
const HOOK_LOG_MAX: usize = 256;

// ★ v3.24.42: high-frequency read_summary/push spam is excluded from the
// ring (still goes to logcat) so event/sniff diagnostics survive.
const HOOK_LOG_NOISE: &[&str] = &[
    "★ read_summary",
    "ramen scalar",
    "ramen arrays",
    "evaluation_list",
    "sc: ",
    "skill_eval=",
    "v3.22.51 ramen",
    "★ Scenario 14",
    "Push:",
    "call_getter: 'get_Skill",
    "call_getter: 'get_PossessSkill",
    "find_field_offset: 'RemainTurn'",
];
fn hook_log(msg: &str) {
    if HOOK_LOG_NOISE.iter().any(|n| msg.contains(n)) {
        return;
    }
    if let Ok(mut g) = HOOK_LOG.lock() {
        if g.len() >= HOOK_LOG_MAX {
            g.remove(0);
        }
        g.push(msg.to_string());
    }
}

// ★ v3.24.40: per-hook install status for /debug/hookdiag
static HOOK_STATUS: Mutex<Vec<(String, String)>> = Mutex::new(Vec::new());
fn set_hook_status(name: &str, status: &str) {
    hook_log(&format!("hook[{}] = {}", name, status));
    if let Ok(mut g) = HOOK_STATUS.lock() {
        if let Some(e) = g.iter_mut().find(|(n, _)| n == name) {
            e.1 = status.to_string();
        } else {
            g.push((name.to_string(), status.to_string()));
        }
    }
}
static mut EVENT_CHOICES: Vec<EventChoice> = Vec::new();
static mut EVENT_SELECTED_IDX: i32 = -1;
static mut EVENT_STORY_ID: i32 = 0;
static mut EVENT_CHARA_ID: i32 = 0;

// Incremented whenever a new story_id takes over (or state is cleared).
// Guarded by EVENT_STATE_MUTEX; never read/write outside the lock.
static mut EVENT_GENERATION: u64 = 0;

// Cap against runaway AddChoiceButton repeats in abnormal UI rebuilds.
const EVENT_CHOICES_MAX: usize = 32;

#[derive(Clone)]
struct EventChoice {
    label: String,
    gain_id: i32,
    next_block_idx: i32,
    loop_exit_gain_id: i32,
}

// v3.24.73: bounded cache-only pairing. This is temporal co-occurrence,
// never a success/failure classification or a causality claim.
#[derive(Clone)]
struct PendingEventSelection {
    captured_at: u64,
    generation: u64,
    story_id: i32,
    chara_id: i32,
    selected_idx_raw: i32,
    choice: Option<EventChoice>,
}
static EVENT_PENDING_RESULT: Mutex<Option<PendingEventSelection>> = Mutex::new(None);
static EVENT_OBSERVATIONS: Mutex<Vec<String>> = Mutex::new(Vec::new());
static EVENT_OBSERVATION_ID: AtomicU64 = AtomicU64::new(1);
const EVENT_OBSERVATIONS_MAX: usize = 16;
const EVENT_RESPONSE_PREVIEW_MAX: usize = 16 * 1024;

// ★ v3.24.2: Read C# string from IL2CPP String object
unsafe fn read_il2cpp_string(s: *const c_void) -> String {
    if s.is_null() {
        return String::new();
    }
    let len = std::ptr::read::<i32>((s as *const u8).offset(16) as *const i32);
    if len <= 0 || len > 4096 {
        return String::new();
    }
    let chars_ptr = (s as *const u8).offset(20);
    let chars_slice = std::slice::from_raw_parts(chars_ptr as *const u16, len as usize);
    String::from_utf16_lossy(chars_slice)
}

// ★ Push-to-app state (v3.10.0): auto-push /summary to uma-juece when data changes
static mut LAST_PUSH_HASH: u64 = 0;
static PUSH_INTERVAL_SECS: u64 = 1;

// ★ Config (v3.11.0): runtime config updated via POST /config from App
// No file editing needed — App settings page sends config to plugin HTTP endpoint
#[derive(Clone)]
struct PluginConfig {
    push_host: String,       // default: "127.0.0.1"
    push_port: u16,          // default: 18766
    http_port: u16,          // default: 18765
    push_interval_secs: u64, // default: 1
    push_enabled: bool,      // default: true
    http_enabled: bool,      // default: true
}

impl PluginConfig {
    fn defaults() -> Self {
        Self {
            push_host: "127.0.0.1".to_string(),
            push_port: 18766,
            http_port: 18765,
            push_interval_secs: 5,
            push_enabled: true,
            http_enabled: true,
        }
    }

    fn push_addr(&self) -> String {
        format!("{}:{}", self.push_host, self.push_port)
    }

    // Parse JSON config from POST /config body (simple manual parse, no serde)
    fn from_json(data: &str) -> Option<Self> {
        let mut cfg = Self::defaults();
        let mut changed = false;
        // Extract key-value pairs from JSON
        for line in data.lines() {
            let l = line.trim().trim_end_matches(',');
            if l.is_empty() || l == "{" || l == "}" {
                continue;
            }
            if let Some((k, v)) = l.split_once(':') {
                let k = k.trim().trim_matches('"');
                let v = v.trim().trim_matches('"');
                match k {
                    "push_host" => {
                        cfg.push_host = v.to_string();
                        changed = true;
                    }
                    "push_port" => {
                        if let Ok(n) = v.parse::<u16>() {
                            cfg.push_port = n;
                            changed = true;
                        }
                    }
                    "http_port" => {
                        if let Ok(n) = v.parse::<u16>() {
                            cfg.http_port = n;
                            changed = true;
                        }
                    }
                    "push_interva
```

```rust
= match sel.choice {
                        Some(c) => (c.label, c.gain_id, c.next_block_idx, c.loop_exit_gain_id),
                        None => (String::new(), -1, -1, -1),
                    };
                    let observation_id = EVENT_OBSERVATION_ID.fetch_add(1, Ordering::Relaxed);
                    let record = format!(
                        r#"{{"schema_version":2,"observation_id":{},"source":"runtime_observation","causality":"unknown","result_label":"unknown","captured_at":{},"generation":{},"story_id":{},"chara_id":{},"selected_idx_raw":{},"choice":{{"label":"{}","gain_id":{},"next_block_idx":{},"loop_exit_gain_id":{}}},"response":{{"request_id":{},"url":"{}","size_captured":{},"preview_truncated":{},"hex_prefix":"{}","text_preview":"{}"}}}}"#,
                        observation_id,
                        sel.captured_at,
                        sel.generation,
                        sel.story_id,
                        sel.chara_id,
                        sel.selected_idx_raw,
                        json_escape(&label),
                        gain_id,
                        next_block_idx,
                        loop_exit_gain_id,
                        PENDING_REQ_ID,
                        json_escape(&PENDING_URL),
                        bytes.len(),
                        bytes.len() > preview_len,
                        hex_encode(&bytes[..bytes.len().min(64)]),
                        json_escape(&preview)
                    );
                    if let Ok(mut obs) = EVENT_OBSERVATIONS.lock() {
                        if obs.len() >= EVENT_OBSERVATIONS_MAX {
                            obs.remove(0);
                        }
                        obs.push(record);
                    }
                }
            }
        }
        if SNIFF_ENABLED.load(Ordering::Relaxed) {
            if !bytes.is_empty() {
                let _lock = SNIFF_MUTEX.lock();
                let (rid, response_url) = if SNIFF_RESPONSE_QUEUE.is_empty() {
                    (0, String::new())
                } else {
                    SNIFF_RESPONSE_QUEUE.remove(0)
                };
                let response_headers = take_unity_response_headers(&response_url);
                let response_headers_json = response_headers.as_ref().map(|headers| format_headers_json(headers));
                match (response_headers, response_headers_json) {
                    (Some(headers), Some(headers_json)) => {
                        push_sniff_metadata(rid, "response", &response_url, bytes.len(), &bytes, headers);
                        if let Err(error) = persist_protocol_capture("response", rid, &response_url, headers_json.as_bytes(), &bytes) { storage_set_error(&error); }
                    }
                    _ => {
                        storage_set_error("response_headers_not_correlated");
                        push_sniff_metadata(rid, "response", &response_url, bytes.len(), &bytes, Vec::new());
                        if let Err(error) = persist_protocol_capture("response", rid, &response_url, &[], &bytes) { storage_set_error(&error); }
                    }
                }
                SNIFF_RESPONSES.push((rid, bytes));
                if SNIFF_RESPONSES.len() > SNIFF_RAW_MAX {
                    SNIFF_RESPONSES.remove(0);
                }
            }
        }
        decompressed
    }
}

// ★ v3.23.3: Hook handler for WWWRequest.Post(this, url, postData, headers)
// Captures URL + headers directly, and matches the parked request body from CompressRequest.
// This replaces the old _Send + SetHeader approach.
extern "C" fn post_hook_handler(
    this: *mut c_void,
    url: *const c_void,
    post_data: *mut c_void,
    headers: *mut c_void,
) -> *mut c_void {
    unsafe {
        let trampoline = interceptor_get_trampoline(post_hook_handler as usize);
        if trampoline == 0 {
            return std::ptr::null_mut();
        }
        type FnType = unsafe extern "C" fn(
            *mut c_void,
            *const c_void,
            *mut c_void,
            *mut c_void,
        ) -> *mut c_void;
        let original: FnType = std::mem::transmute(trampoline);

        // Capture URL
        let game_url = if !url.is_null() {
            read_il2cpp_string(url)
        } else {
            String::new()
        };
        let game_url = if game_url.is_empty() {
            None
        } else {
            Some(game_url)
        };

        // Capture headers from Dictionary<string,string>
        let req_headers = read_string_dict(headers);

        if SNIFF_ENABLED.load(Ordering::Relaxed) {
            let rid = SNIFF_REQ_ID.fetch_add(1, Ordering::Relaxed);
            PENDING_REQ_ID = rid;
            let body = PENDING_REQ_BODY.take().unwrap_or_default();
            let headers_json = format_headers_json(&req_headers);
            let url_str = game_url.clone().unwrap_or_default();
            {
                let _lock = SNIFF_MUTEX.lock();
                push_sniff_metadata(rid, "request", &url_str, body.len(), &body, req_headers.clone());
                SNIFF_RESPONSE_QUEUE.push((rid, url_str.clone()));
                if SNIFF_RESPONSE_QUEUE.len() > SNIFF_METADATA_MAX {
                    SNIFF_RESPONSE_QUEUE.remove(0);
                }
                if let Err(error) = persist_protocol_capture("request", rid, &url_str, headers_json.as_bytes(), &body) { storage_set_error(&error); }
                SNIFF_REQUESTS.push((rid, url_str, headers_json, body));
                if SNIFF_REQUESTS.len() > SNIFF_RAW_MAX {
                    SNIFF_REQUESTS.remove(0);
                }
            }
            PENDING_URL = game_url.clone().unwrap_or_default();
            PENDING_HEADERS = req_headers.clone();
        }

        let _ = this;
        original(this, url, post_data, headers)
    }
}

// ★ v3.23.3: Read IL2CPP Dictionary<string,string> into Vec<(String,String)>
// Layout: [hdr 0x10][fields...]; _entries @+0x18, _count @+0x20
// Entry: [hashCode:i32][next:i32][key:ptr][value:ptr] = 24B per entry
unsafe fn read_string_dict(dict: *mut c_void) -> Vec<(String, String)> {
    let mut out = Vec::new();
    if dict.is_null() {
        return out;
    }
    let count = std::ptr::read_unaligned::<i32>((dict as *const u8).add(0x20) as *const i32);
    if count <= 0 {
        return out;
    }
    let entries = std::ptr::read_unaligned::<usize>((dict as *const u8).add(0x18) as *const usize);
    if entries == 0 {
        return out;
    }
    // Il2CppArray header: 0x20 bytes, then entries
    let capacity =
        std::ptr::read_unaligned::<usize>((entries as *const u8).add(0x18) as *const usize);
    let entries_base = entries + 0x20;
    for i in 0..capacity {
        let entry_addr = entries_base + i * 24;
        let hash_code = std::ptr::read_unaligned::<i32>((entry_addr as *const u8) as *const i32);
        if hash_code < 0 {
            continue;
        } // free entry
        let key =
            std::ptr::read_unaligned::<usize>((entry_addr as *const u8).add(8) as *const usize);
        let value =
            std::ptr::read_unaligned::<usize>((entry_addr as *const u8).add(16) as *const usize);
        let key_str = read_il2cpp_string(key as *const c_void);
        let val_str = read_il2cpp_string(value as *const c_void);
        out.push((key_str, val_str));
        if out.len() >= count as usize {
            break;
        }
    }
    out
}

// Format headers Vec to JSON string: {"key1":"val1","key2":"val2"}
unsafe fn format_headers_json(headers: &[(String, String)]) -> String {
    if headers.is_empty() {
        return "{}".to_string();
    }
    let mut s = String::from("{");
    for (i, (k, v)) in headers.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        let v_escaped = v.replace('\\', "\\\\").replace('"', "\\\"");
        s.push_str(&format!("\"{}\":\"{}\"", k, v_escaped));
    }
    s.push('}');
    s
}

// ============================================================
// ★ v3.24.44: SQLCipher key capture (route B: offline meta decryption)
// The game's resource index `meta` is a SQLCipher-encrypted SQLite DB
// (no plain header; libnative.so exports sqlite3_key/sqlite3_key_v2).
// Hook the keying functions at plugin init (before the game opens meta),
// capture the key bytes, persist to the private files dir.
static META_KEY_HEX: Mutex<String> = Mutex::new(String::new());
static mut SQLCIPHER_KEY_HOOK_DONE: bool = false;

// ★ v3.24.45: pair db handle -> filename -> key + cipher config.
// (v3.24.44's "first key wins" caught the WRONG database's key.)
static DB_HANDLES: Mutex<Vec<(usize, String)>> = Mutex::new(Vec::new());
static DB_KEY_LOG: Mutex<Vec<String>> = Mutex::new(Vec::new());
fn db_track(entry: String) {
    hook_log(&entry);
    if let Ok(mut g) = DB_KEY_LOG.lock() {
        if g.len() >= 96 {
            g.remove(0);
        }
        g.push(entry);
    }
}
fn db_file_of(handle: usize) -> String {
    DB_HANDLES
        .lock()
        .ok()
        .and_then(|g| g.iter().find(|(h, _)| *h == handle).map(|(_, f)| f.clone()))
        .unwrap_or_else(|| "?".to_string())
}

/// ★ v3.24.46: read a C string at a raw address ONLY if it lies inside a
/// readable mapped region (mc_config varargs may or may not be pointers).
unsafe fn safe_read_cstr(addr: usize, max: usize) -> String {
    if addr < 0x10000 {
        return String::new();
    }
    if let Ok(maps) = std::fs::read_to_string("/proc/self/maps"
```

```rust
id":{},"source":"runtime_observation","causality":"unknown","result_label":"unknown","captured_at":{},"generation":{},"story_id":{},"chara_id":{},"selected_idx_raw":{},"choice":{{"label":"{}","gain_id":{},"next_block_idx":{},"loop_exit_gain_id":{}}},"response":{{"request_id":{},"url":"{}","size_captured":{},"preview_truncated":{},"hex_prefix":"{}","text_preview":"{}"}}}}"#,
                        observation_id,
                        sel.captured_at,
                        sel.generation,
                        sel.story_id,
                        sel.chara_id,
                        sel.selected_idx_raw,
                        json_escape(&label),
                        gain_id,
                        next_block_idx,
                        loop_exit_gain_id,
                        PENDING_REQ_ID,
                        json_escape(&PENDING_URL),
                        bytes.len(),
                        bytes.len() > preview_len,
                        hex_encode(&bytes[..bytes.len().min(64)]),
                        json_escape(&preview)
                    );
                    if let Ok(mut obs) = EVENT_OBSERVATIONS.lock() {
                        if obs.len() >= EVENT_OBSERVATIONS_MAX {
                            obs.remove(0);
                        }
                        obs.push(record);
                    }
                }
            }
        }
        if SNIFF_ENABLED.load(Ordering::Relaxed) {
            if !bytes.is_empty() {
                let _lock = SNIFF_MUTEX.lock();
                let (rid, response_url) = if SNIFF_RESPONSE_QUEUE.is_empty() {
                    (0, String::new())
                } else {
                    SNIFF_RESPONSE_QUEUE.remove(0)
                };
                let response_headers = take_unity_response_headers(&response_url);
                let response_headers_json = response_headers.as_ref().map(|headers| format_headers_json(headers));
                match (response_headers, response_headers_json) {
                    (Some(headers), Some(headers_json)) => {
                        push_sniff_metadata(rid, "response", &response_url, bytes.len(), &bytes, headers);
                        if let Err(error) = persist_protocol_capture("response", rid, &response_url, headers_json.as_bytes(), &bytes) { storage_set_error(&error); }
                    }
                    _ => {
                        storage_set_error("response_headers_not_correlated");
                        push_sniff_metadata(rid, "response", &response_url, bytes.len(), &bytes, Vec::new());
                        if let Err(error) = persist_protocol_capture("response", rid, &response_url, &[], &bytes) { storage_set_error(&error); }
                    }
                }
                SNIFF_RESPONSES.push((rid, bytes));
                if SNIFF_RESPONSES.len() > SNIFF_RAW_MAX {
                    SNIFF_RESPONSES.remove(0);
                }
            }
        }
        decompressed
    }
}

// ★ v3.23.3: Hook handler for WWWRequest.Post(this, url, postData, headers)
// Captures URL + headers directly, and matches the parked request body from CompressRequest.
// This replaces the old _Send + SetHeader approach.
extern "C" fn post_hook_handler(
    this: *mut c_void,
    url: *const c_void,
    post_data: *mut c_void,
    headers: *mut c_void,
) -> *mut c_void {
    unsafe {
        let trampoline = interceptor_get_trampoline(post_hook_handler as usize);
        if trampoline == 0 {
            return std::ptr::null_mut();
        }
        type FnType = unsafe extern "C" fn(
            *mut c_void,
            *const c_void,
            *mut c_void,
            *mut c_void,
        ) -> *mut c_void;
        let original: FnType = std::mem::transmute(trampoline);

        // Capture URL
        let game_url = if !url.is_null() {
            read_il2cpp_string(url)
        } else {
            String::new()
        };
        let game_url = if game_url.is_empty() {
            None
        } else {
            Some(game_url)
        };

        // Capture headers from Dictionary<string,string>
        let req_headers = read_string_dict(headers);

        if SNIFF_ENABLED.load(Ordering::Relaxed) {
            let rid = SNIFF_REQ_ID.fetch_add(1, Ordering::Relaxed);
            PENDING_REQ_ID = rid;
            let body = PENDING_REQ_BODY.take().unwrap_or_default();
            let headers_json = format_headers_json(&req_headers);
            let url_str = game_url.clone().unwrap_or_default();
            {
                let _lock = SNIFF_MUTEX.lock();
                push_sniff_metadata(rid, "request", &url_str, body.len(), &body, req_headers.clone());
                SNIFF_RESPONSE_QUEUE.push((rid, url_str.clone()));
                if SNIFF_RESPONSE_QUEUE.len() > SNIFF_METADATA_MAX {
                    SNIFF_RESPONSE_QUEUE.remove(0);
                }
                if let Err(error) = persist_protocol_capture("request", rid, &url_str, headers_json.as_bytes(), &body) { storage_set_error(&error); }
                SNIFF_REQUESTS.push((rid, url_str, headers_json, body));
                if SNIFF_REQUESTS.len() > SNIFF_RAW_MAX {
                    SNIFF_REQUESTS.remove(0);
                }
            }
            PENDING_URL = game_url.clone().unwrap_or_default();
            PENDING_HEADERS = req_headers.clone();
        }

        let _ = this;
        original(this, url, post_data, headers)
    }
}

// ★ v3.23.3: Read IL2CPP Dictionary<string,string> into Vec<(String,String)>
// Layout: [hdr 0x10][fields...]; _entries @+0x18, _count @+0x20
// Entry: [hashCode:i32][next:i32][key:ptr][value:ptr] = 24B per entry
unsafe fn read_string_dict(dict: *mut c_void) -> Vec<(String, String)> {
    let mut out = Vec::new();
    if dict.is_null() {
        return out;
    }
    let count = std::ptr::read_unaligned::<i32>((dict as *const u8).add(0x20) as *const i32);
    if count <= 0 {
        return out;
    }
    let entries = std::ptr::read_unaligned::<usize>((dict as *const u8).add(0x18) as *const usize);
    if entries == 0 {
        return out;
    }
    // Il2CppArray header: 0x20 bytes, then entries
    let capacity =
        std::ptr::read_unaligned::<usize>((entries as *const u8).add(0x18) as *const usize);
    let entries_base = entries + 0x20;
    for i in 0..capacity {
        let entry_addr = entries_base + i * 24;
        let hash_code = std::ptr::read_unaligned::<i32>((entry_addr as *const u8) as *const i32);
        if hash_code < 0 {
            continue;
        } // free entry
        let key =
            std::ptr::read_unaligned::<usize>((entry_addr as *const u8).add(8) as *const usize);
        let value =
            std::ptr::read_unaligned::<usize>((entry_addr as *const u8).add(16) as *const usize);
        let key_str = read_il2cpp_string(key as *const c_void);
        let val_str = read_il2cpp_string(value as *const c_void);
        out.push((key_str, val_str));
        if out.len() >= count as usize {
            break;
        }
    }
    out
}

// Format headers Vec to JSON string: {"key1":"val1","key2":"val2"}
unsafe fn format_headers_json(headers: &[(String, String)]) -> String {
    if headers.is_empty() {
        return "{}".to_string();
    }
    let mut s = String::from("{");
    for (i, (k, v)) in headers.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        let v_escaped = v.replace('\\', "\\\\").replace('"', "\\\"");
        s.push_str(&format!("\"{}\":\"{}\"", k, v_escaped));
    }
    s.push('}');
    s
}

// ============================================================
// ★ v3.24.44: SQLCipher key capture (route B: offline meta decryption)
// The game's resource index `meta` is a SQLCipher-encrypted SQLite DB
// (no plain header; libnative.so exports sqlite3_key/sqlite3_key_v2).
// Hook the keying functions at plugin init (before the game opens meta),
// capture the key bytes, persist to the private files dir.
static META_KEY_HEX: Mutex<String> = Mutex::new(String::new());
static mut SQLCIPHER_KEY_HOOK_DONE: bool = false;

// ★ v3.24.45: pair db handle -> filename -> key + cipher config.
// (v3.24.44's "first key wins" caught the WRONG database's key.)
static DB_HANDLES: Mutex<Vec<(usize, String)>> = Mutex::new(Vec::new());
static DB_KEY_LOG: Mutex<Vec<String>> = Mutex::new(Vec::new());
fn db_track(entry: String) {
    hook_log(&entry);
    if let Ok(mut g) = DB_KEY_LOG.lock() {
        if g.len() >= 96 {
            g.remove(0);
        }
        g.push(entry);
    }
}
fn db_file_of(handle: usize) -> String {
    DB_HANDLES
        .lock()
        .ok()
        .and_then(|g| g.iter().find(|(h, _)| *h == handle).map(|(_, f)| f.clone()))
        .unwrap_or_else(|| "?".to_string())
}

/// ★ v3.24.46: read a C string at a raw address ONLY if it lies inside a
/// readable mapped region (mc_config varargs may or may not be pointers).
unsafe fn safe_read_cstr(addr: usize, max: usize) -> String {
    if addr < 0x10000 {
        return String::new();
    }
    if let Ok(maps) = std::fs::read_to_string("/proc/self/maps") {
        for line in maps.lines() {
            let mut parts = line.split_whitespace();
            let range = match parts.next() {
                Some(r) => r,
                None => continue,
            };
            let (a, b) = match range.split_once('-') {
                Some(x) => x,
                None => continue,
            };
            let sa = match usize::from_str_radix(
```
