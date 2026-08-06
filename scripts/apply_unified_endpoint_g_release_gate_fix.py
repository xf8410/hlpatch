from pathlib import Path

SOURCE = Path("hachimi_ura_plugin/src/lib.rs")
s = SOURCE.read_text(encoding="utf-8")
MARKER = "// ===== Unified release-gate correction G-stage ====="
if MARKER in s:
    print("unified_endpoint_g_release_gate_fix=already_applied")
    raise SystemExit(0)


def replace_once(old: str, new: str, label: str) -> None:
    global s
    count = s.count(old)
    assert count == 1, f"{label} anchor count={count}"
    s = s.replace(old, new, 1)

# A watchdog changes the observable state even if one IL2CPP call never returns.
# The stuck native call cannot be safely killed; generation prevents a late worker
# from publishing stale entries.
spawn_anchor = '''    if let Err(error) = spawn {
        let mut state = METHOD_INDEX.lock().unwrap_or_else(|poison| poison.into_inner());
        if state.generation == generation {
            state.worker_active = false;
            state.status = "failed";
            state.error = format!("method_index_spawn_failed:{}", error);
        }
        return Err("method_index_spawn_failed".to_string());
    }
    Err("method_index_building".to_string())
'''
spawn_new = '''    if let Err(error) = spawn {
        let mut state = METHOD_INDEX.lock().unwrap_or_else(|poison| poison.into_inner());
        if state.generation == generation {
            state.worker_active = false;
            state.status = "failed";
            state.error = format!("method_index_spawn_failed:{}", error);
        }
        return Err("method_index_spawn_failed".to_string());
    }
    let watchdog_generation = generation;
    let watchdog = std::thread::Builder::new()
        .name(format!("hlpatch-method-index-watchdog-{}", watchdog_generation))
        .spawn(move || {
            std::thread::sleep(std::time::Duration::from_secs(181));
            let mut state = METHOD_INDEX.lock().unwrap_or_else(|error| error.into_inner());
            if state.generation == watchdog_generation && state.status == "building" {
                state.generation = state.generation.saturating_add(1);
                state.worker_active = false;
                state.status = "failed";
                state.error = "method_index_watchdog_timeout".to_string();
                state.heartbeat_at_ms = method_index_now_ms();
            }
        });
    if let Err(error) = watchdog {
        let mut state = METHOD_INDEX.lock().unwrap_or_else(|poison| poison.into_inner());
        if state.generation == generation {
            state.generation = state.generation.saturating_add(1);
            state.worker_active = false;
            state.status = "failed";
            state.error = format!("method_index_watchdog_spawn_failed:{}", error);
        }
        return Err("method_index_watchdog_spawn_failed".to_string());
    }
    Err("method_index_building".to_string())
'''
replace_once(spawn_anchor, spawn_new, "method_index_watchdog")

# Persist a process-start token, not only PID. Existing databases are migrated.
replace_once(
    '''             process_id INTEGER NOT NULL,
             plugin_version TEXT NOT NULL,
''',
    '''             process_id INTEGER NOT NULL,
             process_start_token TEXT NOT NULL DEFAULT '',
             plugin_version TEXT NOT NULL,
''',
    "session_schema_token",
)
replace_once(
    '''    ).map_err(|error| format!("initialize_schema:{}", error))?;
    Ok(connection)
}
''',
    '''    ).map_err(|error| format!("initialize_schema:{}", error))?;
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
''',
    "storage_schema_migration",
)
replace_once(
    '''    let process_id = std::process::id();
    let session_id = format!("{}-{}", now, process_id);
''',
    '''    let process_id = std::process::id();
    let process_start_token = observation_process_start_token();
    let session_id = format!("{}-{}", now, process_id);
''',
    "session_start_token",
)
replace_once(
    '''         WHERE state='open' AND process_id<>?1",
        rusqlite::params![process_id as i64],
''',
    '''         WHERE state='open' AND process_start_token<>?1",
        rusqlite::params![process_start_token],
''',
    "ensure_recovery_token",
)
replace_once(
    '''             session_id, process_id, plugin_version, started_at_ms,
             last_flush_ms, state, recovered_after_restart, root_path
         ) VALUES(?1, ?2, ?3, ?4, ?5, 'open', 0, ?6)",
        rusqlite::params![session_id, process_id as i64, PLUGIN_VERSION, now as i64, now as i64, root_text],
''',
    '''             session_id, process_id, process_start_token, plugin_version, started_at_ms,
             last_flush_ms, state, recovered_after_restart, root_path
         ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, 'open', 0, ?7)",
        rusqlite::params![session_id, process_id as i64, process_start_token, PLUGIN_VERSION, now as i64, now as i64, root_text],
''',
    "session_insert_token",
)
replace_once(
    '''    let process_id = std::process::id() as i64;
    let recovered = match connection.execute(
        "UPDATE observation_sessions
         SET state='interrupted', recovered_after_restart=1
         WHERE state='open' AND process_id<>?1",
        rusqlite::params![process_id],
''',
    '''    let process_start_token = observation_process_start_token();
    let recovered = match connection.execute(
        "UPDATE observation_sessions
         SET state='interrupted', recovered_after_restart=1
         WHERE state='open' AND process_start_token<>?1",
        rusqlite::params![process_start_token],
''',
    "recover_endpoint_token",
)

# On a partial session creation failure, remove both DB rows and filesystem state.
replace_once(
    '''    std::fs::create_dir_all(&session_directory).map_err(|error| format!("create_session_dir:{}", error))?;
''',
    '''    if let Err(error) = std::fs::create_dir_all(&session_directory) {
        let _ = connection.execute("DELETE FROM observation_sessions WHERE session_id=?1", rusqlite::params![session_id]);
        return Err(format!("create_session_dir:{}", error));
    }
''',
    "session_dir_compensation",
)
replace_once(
    '''    std::fs::write(session_directory.join("session.json"), session_json.as_bytes())
        .map_err(|error| format!("write_session_json:{}", error))?;
''',
    '''    if let Err(error) = std::fs::write(session_directory.join("session.json"), session_json.as_bytes()) {
        let _ = connection.execute("DELETE FROM observation_sessions WHERE session_id=?1", rusqlite::params![session_id]);
        let _ = std::fs::remove_dir_all(&session_directory);
        return Err(format!("write_session_json:{}", error));
    }
''',
    "session_json_compensation",
)
replace_once(
    '''    connection.execute(
        "INSERT OR REPLACE INTO observation_files(
             session_id, relative_path, content_type, byte_length, sha256, created_at_ms
         ) VALUES(?1, 'session.json', 'application/json', ?2, NULL, ?3)",
        rusqlite::params![session_id, session_json.as_bytes().len() as i64, now as i64],
    ).map_err(|error| format!("index_session_json:{}", error))?;
''',
    '''    if let Err(error) = connection.execute(
        "INSERT OR REPLACE INTO observation_files(
             session_id, relative_path, content_type, byte_length, sha256, created_at_ms
         ) VALUES(?1, 'session.json', 'application/json', ?2, NULL, ?3)",
        rusqlite::params![session_id, session_json.as_bytes().len() as i64, now as i64],
    ) {
        let _ = connection.execute("DELETE FROM observation_sessions WHERE session_id=?1", rusqlite::params![session_id]);
        let _ = std::fs::remove_dir_all(&session_directory);
        return Err(format!("index_session_json:{}", error));
    }
''',
    "session_index_compensation",
)

anchor = "/// 辅助函数：IL2CPP类型枚举转可读名称\n"
replace_once(anchor, MARKER + "\n" + anchor, "g_marker")
SOURCE.write_text(s, encoding="utf-8")
print("unified_endpoint_g_release_gate_fix=applied")
