# `storage_session_endpoint`

source_commit: `a340a147acf13672b2fbc64925bfa321d08091fd`
source_line: `22854`

```rust
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
```
