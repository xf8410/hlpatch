# `storage_flush_endpoint`

source_commit: `ffc3748df2d3c8c57b34aa3fdd64f75d09ed0866`
source_line: `22884`

```rust
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
```
