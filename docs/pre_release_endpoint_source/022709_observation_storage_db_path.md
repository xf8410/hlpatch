# `observation_storage_db_path`

source_commit: `a340a147acf13672b2fbc64925bfa321d08091fd`
source_line: `22709`

```rust
fn observation_storage_db_path() -> std::path::PathBuf {
    observation_storage_root().join("index.sqlite")
}
```
