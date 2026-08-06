# `observation_storage_root`

source_commit: `a340a147acf13672b2fbc64925bfa321d08091fd`
source_line: `22694`

```rust
fn observation_storage_root() -> std::path::PathBuf {
    if let Ok(command_line) = std::fs::read("/proc/self/cmdline") {
        let package_bytes = command_line.split(|byte| *byte == 0).next().unwrap_or(&[]);
        if let Ok(package_name) = std::str::from_utf8(package_bytes) {
            if !package_name.is_empty() {
                return std::path::PathBuf::from("/data/user/0")
                    .join(package_name)
                    .join("files")
                    .join("hlpatch-observations");
            }
        }
    }
    std::path::PathBuf::from("/data/user/0/jp.co.cygames.umamusume/files/hlpatch-observations")
}
```
