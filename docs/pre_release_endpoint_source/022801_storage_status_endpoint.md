# `storage_status_endpoint`

source_commit: `ffc3748df2d3c8c57b34aa3fdd64f75d09ed0866`
source_line: `22801`

```rust
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
```
