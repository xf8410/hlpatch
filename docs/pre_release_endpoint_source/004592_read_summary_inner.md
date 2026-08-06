# `read_summary_inner`

source_commit: `ffc3748df2d3c8c57b34aa3fdd64f75d09ed0866`
source_line: `4592`

```rust
unsafe fn read_summary_inner() -> String {
    // v3.22.51: IN_READ_PATH disabled - /debug/ramenfields proves IL2CPP APIs are safe from HTTP thread
    // Keep the wrapper for potential future use, but don't block any APIs
    read_summary_inner_impl()
}
```
