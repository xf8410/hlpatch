# `storage_set_error`

source_commit: `a340a147acf13672b2fbc64925bfa321d08091fd`
source_line: `22688`

```rust
fn storage_set_error(error: &str) {
    if let Ok(mut value) = STORAGE_LAST_ERROR.lock() {
        *value = Some(error.to_string());
    }
}
```
