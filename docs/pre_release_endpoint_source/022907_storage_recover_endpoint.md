# `storage_recover_endpoint`

source_commit: `ffc3748df2d3c8c57b34aa3fdd64f75d09ed0866`
source_line: `22907`

```rust
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
```
