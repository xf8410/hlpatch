# `read_summary_inner`

source_commit: `a340a147acf13672b2fbc64925bfa321d08091fd`
source_line: `4592`

```rust
unsafe fn read_summary_inner() -> String {
    // v3.22.51: IN_READ_PATH disabled - /debug/ramenfields proves IL2CPP APIs are safe from HTTP thread
    // Keep the wrapper for potential future use, but don't block any APIs
    read_summary_inner_impl()
}
```
