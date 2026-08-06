# `observation_storage_db_path`

source_commit: `ffc3748df2d3c8c57b34aa3fdd64f75d09ed0866`
source_line: `22709`

```rust
fn observation_storage_db_path() -> std::path::PathBuf {
    observation_storage_root().join("index.sqlite")
}
```
