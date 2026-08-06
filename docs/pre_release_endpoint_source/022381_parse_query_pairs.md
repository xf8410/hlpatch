# `parse_query_pairs`

source_commit: `ffc3748df2d3c8c57b34aa3fdd64f75d09ed0866`
source_line: `22381`

```rust
fn parse_query_pairs(uri: &str) -> Result<Vec<(String, String)>, String> {
    let query = match uri.split_once('?') {
        Some((_, value)) => value.split('#').next().unwrap_or(""),
        None => return Ok(Vec::new()),
    };
    let mut pairs = Vec::new();
    for item in query.split('&') {
        if item.is_empty() { continue; }
        let (raw_key, raw_value) = item.split_once('=').unwrap_or((item, ""));
        pairs.push((percent_decode_component(raw_key)?, percent_decode_component(raw_value)?));
    }
    Ok(pairs)
}
```
