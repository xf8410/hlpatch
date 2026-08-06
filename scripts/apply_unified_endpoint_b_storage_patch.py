from pathlib import Path

SOURCE = Path("hachimi_ura_plugin/src/lib.rs")
s = SOURCE.read_text(encoding="utf-8")

MARKER = "// ===== Unified observation persistent storage B-stage ====="
if MARKER in s:
    print("unified_endpoint_b_storage_patch=already_applied")
    raise SystemExit(0)

anchor = "/// 辅助函数：IL2CPP类型枚举转可读名称\n"
assert s.count(anchor) == 1, f"storage insertion anchor count={s.count(anchor)}"

rust = r'''// ===== Unified observation persistent storage B-stage =====
static STORAGE_SESSION_ID: Mutex<Option<String>> = Mutex::new(None);
static STORAGE_LAST_FLUSH_MS: AtomicU64 = AtomicU64::new(0);
static STORAGE_LAST_ERROR: Mutex<Option<String>> = Mutex::new(None);

fn storage_set_error(error: &str) {
    if let Ok(mut value) = STORAGE_LAST_ERROR.lock() {
        *value = Some(error.to_string());
    }
}

fn observation_storage_root() -> std::path::PathBuf {
    if let Some(so_path) = find_own_so_path() {
        if let Some(parent) = std::path::Path::new(&so_path).parent() {
            return parent.join("hlpatch-observations");
        }
    }
    std::path::PathBuf::from("/data/data/jp.pokemon.pokeuma/files/hlpatch-observations")
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
    Ok(connection)
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
    let session_id = format!("{}-{}", now, process_id);
    let root_text = observation_storage_root().to_string_lossy().into_owned();
    connection.execute(
        "UPDATE observation_sessions
         SET state='interrupted', recovered_after_restart=1
         WHERE state='open' AND process_id<>?1",
        rusqlite::params![process_id as i64],
    ).map_err(|error| format!("recover_previous_sessions:{}", error))?;
    connection.execute(
        "INSERT INTO observation_sessions(
             session_id, process_id, plugin_version, started_at_ms,
             last_flush_ms, state, recovered_after_restart, root_path
         ) VALUES(?1, ?2, ?3, ?4, ?5, 'open', 0, ?6)",
        rusqlite::params![session_id, process_id as i64, PLUGIN_VERSION, now as i64, now as i64, root_text],
    ).map_err(|error| format!("insert_session:{}", error))?;
    let session_directory = observation_storage_root().join("sessions").join(&session_id);
    std::fs::create_dir_all(&session_directory).map_err(|error| format!("create_session_dir:{}", error))?;
    let session_json = format!(
        r#"{{"session_id":"{}","process_id":{},"plugin_version":"{}","started_at_ms":{},"state":"open","recovered_after_restart":false,"root_path":"{}"}}"#,
        json_escape(&session_id), process_id, json_escape(PLUGIN_VERSION), now, json_escape(&root_text)
    );
    std::fs::write(session_directory.join("session.json"), session_json.as_bytes())
        .map_err(|error| format!("write_session_json:{}", error))?;
    connection.execute(
        "INSERT OR REPLACE INTO observation_files(
             session_id, relative_path, content_type, byte_length, sha256, created_at_ms
         ) VALUES(?1, 'session.json', 'application/json', ?2, NULL, ?3)",
        rusqlite::params![session_id, session_json.as_bytes().len() as i64, now as i64],
    ).map_err(|error| format!("index_session_json:{}", error))?;
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
    let session_json = current_session.map(|value| format!("\"{}\"", json_escape(&value))).unwrap_or_else(|| "null".to_string());
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
    let sessions: Vec<String> = rows.filter_map(Result::ok).collect();
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
    if let Err(error) = connection.execute(
        "UPDATE observation_sessions SET last_flush_ms=?1 WHERE session_id=?2",
        rusqlite::params![now as i64, session_id],
    ) {
        return format!(r#"{{"ok":false,"error":"update_flush:{}"}}"#, json_escape(&error.to_string()));
    }
    if let Err(error) = connection.execute_batch("PRAGMA wal_checkpoint(FULL);") {
        return format!(r#"{{"ok":false,"error":"checkpoint:{}"}}"#, json_escape(&error.to_string()));
    }
    STORAGE_LAST_FLUSH_MS.store(now, Ordering::Relaxed);
    format!(r#"{{"ok":true,"session_id":"{}","last_flush_ms":{},"checkpoint":"full"}}"#, json_escape(&session_id), now)
}

fn storage_recover_endpoint() -> String {
    let connection = match open_observation_storage() {
        Ok(value) => value,
        Err(error) => return format!(r#"{{"ok":false,"error":"{}"}}"#, json_escape(&error)),
    };
    let process_id = std::process::id() as i64;
    let recovered = match connection.execute(
        "UPDATE observation_sessions
         SET state='interrupted', recovered_after_restart=1
         WHERE state='open' AND process_id<>?1",
        rusqlite::params![process_id],
    ) {
        Ok(value) => value,
        Err(error) => return format!(r#"{{"ok":false,"error":"recover:{}"}}"#, json_escape(&error.to_string())),
    };
    match ensure_observation_session() {
        Ok(session_id) => format!(r#"{{"ok":true,"recovered_session_count":{},"current_session_id":"{}"}}"#, recovered, json_escape(&session_id)),
        Err(error) => format!(r#"{{"ok":false,"error":"{}","recovered_session_count":{}}}"#, json_escape(&error), recovered),
    }
}

'''
s = s.replace(anchor, rust + anchor, 1)

route_anchor = '''    } else if path == "/il2cpp/method_by_addr" {
'''
assert s.count(route_anchor) == 1, f"storage route anchor count={s.count(route_anchor)}"
routes = '''    } else if path == "/storage/status" {
        storage_status_endpoint()
    } else if path == "/storage/sessions" {
        storage_sessions_endpoint()
    } else if path == "/storage/session" {
        storage_session_endpoint(&full_uri)
    } else if path == "/storage/flush" {
        storage_flush_endpoint()
    } else if path == "/storage/recover" {
        storage_recover_endpoint()
'''
s = s.replace(route_anchor, routes + route_anchor, 1)

SOURCE.write_text(s, encoding="utf-8")
print("unified_endpoint_b_storage_patch=applied")
