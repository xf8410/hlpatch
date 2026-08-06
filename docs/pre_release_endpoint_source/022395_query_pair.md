# `query_pair`

source_commit: `ffc3748df2d3c8c57b34aa3fdd64f75d09ed0866`
source_line: `22395`

```rust
fn query_pair(pairs: &[(String, String)], name: &str) -> String {
    pairs.iter().find(|(key, _)| key == name).map(|(_, value)| value.clone()).unwrap_or_default()
}
```
