from pathlib import Path

SOURCE = Path("hachimi_ura_plugin/src/lib.rs")
s = SOURCE.read_text(encoding="utf-8")
MARKER = "// ===== Ramen/Hachimi global durable observation M-stage ====="
if MARKER in s:
    print("ramen_global_durable_observation=already_applied")
    raise SystemExit(0)


def replace_once(old: str, new: str, label: str) -> None:
    global s
    count = s.count(old)
    assert count == 1, f"{label} anchor count={count}"
    s = s.replace(old, new, 1)

storage_anchor = '''fn storage_clear_error() {
    if let Ok(mut value) = STORAGE_LAST_ERROR.lock() { *value = None; }
}
'''

durable = storage_anchor + r'''

// 全局观测记录先追加到当前会话的NDJSON，再允许调用方更新内存索引。
// 每行以换行符作为完整提交边界；读取方不得把无换行的尾部当作完整记录。
static GLOBAL_OBSERVATION_SEQUENCE: AtomicU64 = AtomicU64::new(0);
static GLOBAL_OBSERVATION_WRITE_LOCK: Mutex<()> = Mutex::new(());

fn append_global_observation(
    observation_type: &str,
    completeness: &str,
    payload_json: &str,
    critical: bool,
) -> Result<(String, u64, u64), String> {
    let _write_guard = GLOBAL_OBSERVATION_WRITE_LOCK
        .lock().map_err(|_| "global_observation_write_lock_poisoned".to_string())?;
    let session_id = ensure_observation_session()?;
    let sequence = GLOBAL_OBSERVATION_SEQUENCE.fetch_add(1, Ordering::SeqCst).saturating_add(1);
    let timestamp_ms = sniff_timestamp_ms();
    let session_directory = observation_storage_root().join("sessions").join(&session_id);
    std::fs::create_dir_all(&session_directory)
        .map_err(|error| format!("create_global_observation_dir:{}", error))?;
    let journal_path = session_directory.join("timeline.ndjson");
    let line = format!(
        r#"{{"session_id":"{}","sequence":{},"timestamp_ms":{},"type":"{}","completeness":"{}","payload":{}}}\n"#,
        json_escape(&session_id), sequence, timestamp_ms, json_escape(observation_type),
        json_escape(completeness), payload_json
    );
    let mut file = std::fs::OpenOptions::new().create(true).append(true).open(&journal_path)
        .map_err(|error| format!("open_global_observation_journal:{}", error))?;
    std::io::Write::write_all(&mut file, line.as_bytes())
        .map_err(|error| format!("append_global_observation:{}", error))?;
    if critical {
        file.sync_data().map_err(|error| format!("sync_global_observation:{}", error))?;
    }
    let byte_length = file.metadata().map_err(|error| format!("stat_global_observation:{}", error))?.len();
    drop(file);
    let connection = open_observation_storage()?;
    connection.execute(
        "INSERT OR REPLACE INTO observation_files(
             session_id, relative_path, content_type, byte_length, sha256, created_at_ms
         ) VALUES(?1, 'timeline.ndjson', 'application/x-ndjson', ?2, NULL, ?3)",
        rusqlite::params![session_id, byte_length as i64, timestamp_ms as i64],
    ).map_err(|error| format!("index_global_observation:{}", error))?;
    STORAGE_LAST_FLUSH_MS.store(timestamp_ms, Ordering::Release);
    storage_clear_error();
    Ok((session_id, sequence, timestamp_ms))
}

fn persist_protocol_observation_boundary(
    direction: &str,
    request_id: u64,
    url: &str,
    relative_base: &str,
    headers_length: usize,
    payload_length: usize,
) -> Result<(), String> {
    let payload = format!(
        r#"{{"direction":"{}","request_id":{},"url":"{}","relative_base":"{}","headers_length":{},"payload_length":{}}}"#,
        json_escape(direction), request_id, json_escape(url), json_escape(relative_base),
        headers_length, payload_length
    );
    append_global_observation("protocol_exchange_part", "complete", &payload, true).map(|_| ())
}
'''
replace_once(storage_anchor, durable, "durable_storage_primitives")

# 原始协议三件套逐文件写入临时文件、同步数据，再原子改名；随后提交SQLite索引和全局时间线。
old_files = '''    for (name, bytes, _) in &files {
        let temporary = target_dir.join(format!("{}.tmp", name));
        std::fs::write(&temporary, bytes).map_err(|error| format!("write_protocol_file:{}:{}", name, error))?;
        std::fs::rename(&temporary, target_dir.join(name)).map_err(|error| format!("commit_protocol_file:{}:{}", name, error))?;
    }
'''
new_files = '''    for (name, bytes, _) in &files {
        let temporary = target_dir.join(format!("{}.tmp", name));
        let mut file = std::fs::File::create(&temporary)
            .map_err(|error| format!("create_protocol_file:{}:{}", name, error))?;
        std::io::Write::write_all(&mut file, bytes)
            .map_err(|error| format!("write_protocol_file:{}:{}", name, error))?;
        file.sync_data().map_err(|error| format!("sync_protocol_file:{}:{}", name, error))?;
        drop(file);
        std::fs::rename(&temporary, target_dir.join(name))
            .map_err(|error| format!("commit_protocol_file:{}:{}", name, error))?;
    }
'''
replace_once(old_files, new_files, "durable_protocol_files")

old_commit = '''    transaction.commit().map_err(|error| format!("commit_protocol_index:{}", error))?;
    storage_clear_error();
    Ok(())
}
'''
new_commit = '''    transaction.commit().map_err(|error| format!("commit_protocol_index:{}", error))?;
    persist_protocol_observation_boundary(
        direction, request_id, url, &relative_base, headers.len(), payload.len()
    )?;
    storage_clear_error();
    Ok(())
}
'''
replace_once(old_commit, new_commit, "protocol_timeline_boundary")

anchor = "/// 辅助函数：IL2CPP类型枚举转可读名称\n"
replace_once(anchor, MARKER + "\n" + anchor, "m_marker")
SOURCE.write_text(s, encoding="utf-8")
print("ramen_global_durable_observation=applied")
