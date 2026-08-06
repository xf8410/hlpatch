# `parse_query`

source_commit: `a340a147acf13672b2fbc64925bfa321d08091fd`
source_line: `13917`

```rust
fn parse_query(full_uri: &str, key: &str) -> String {
    let pattern = format!("{}=", key);
    if let Some(q) = full_uri.find(&format!("?{}", pattern)) {
        let start = q + 1 + pattern.len();
        let end = full_uri[start..]
            .find('&')
            .map(|e| start + e)
            .unwrap_or(full_uri.len());
        url_decode(&full_uri[start..end])
    } else if let Some(q) = full_uri.find(&format!("&{}", pattern)) {
        let start = q + 1 + pattern.len();
        let end = full_uri[start..]
            .find('&')
            .map(|e| start + e)
            .unwrap_or(full_uri.len());
        url_decode(&full_uri[start..end])
    } else {
        String::new()
    }
}
```
