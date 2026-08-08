# `query_pair`

source_commit: `a340a147acf13672b2fbc64925bfa321d08091fd`
source_line: `22395`

```rust
fn query_pair(pairs: &[(String, String)], name: &str) -> String {
    pairs.iter().find(|(key, _)| key == name).map(|(_, value)| value.clone()).unwrap_or_default()
}
```
