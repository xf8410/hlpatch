# `parse_query`

source_commit: `ffc3748df2d3c8c57b34aa3fdd64f75d09ed0866`
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
