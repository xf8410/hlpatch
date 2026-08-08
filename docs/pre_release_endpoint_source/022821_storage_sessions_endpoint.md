# `storage_sessions_endpoint`

source_commit: `a340a147acf13672b2fbc64925bfa321d08091fd`
source_line: `22821`

```rust
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
```
