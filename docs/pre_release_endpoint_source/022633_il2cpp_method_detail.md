# `il2cpp_method_detail`

source_commit: `a340a147acf13672b2fbc64925bfa321d08091fd`
source_line: `22633`

```rust
unsafe fn il2cpp_method_detail(uri: &str) -> String {
    let pairs = match parse_query_pairs(uri) { Ok(value) => value, Err(error) => return format!(r#"{{"ok":false,"error":"{}"}}"#, json_escape(&error)) };
    let namespace = query_pair(&pairs, "namespace");
    let declaring_type = query_pair(&pairs, "declaring_type");
    let method = query_pair(&pairs, "method");
    let parameter_text = query_pair(&pairs, "parameter_types");
    if declaring_type.is_empty() || method.is_empty() { return r#"{"ok":false,"error":"missing_declaring_type_or_method"}"#.to_string(); }
    let parameter_types: Vec<String> = if parameter_text.is_empty() { Vec::new() } else { parameter_text.split(',').map(|value| value.trim().to_string()).collect() };
    if let Err(error) = ensure_method_index() { return format!(r#"{{"ok":false,"error":"{}"}}"#, json_escape(&error)); }
    let state = match METHOD_INDEX.lock() { Ok(value) => value, Err(_) => return r#"{"ok":false,"error":"method_index_lock_poisoned"}"#.to_string() };
    let matches: Vec<&MethodIndexEntry> = state.entries.iter().filter(|entry| {
        (namespace.is_empty() || entry.namespace == namespace) && entry.declaring_type == declaring_type &&
        entry.method_name == method && entry.parameter_types == parameter_types
    }).collect();
    let status = if matches.is_empty() { "none" } else if matches.len() == 1 { "exact" } else { "ambiguous" };
    let items = matches.iter().map(|entry| {
        let upper = state.entries.iter().map(|candidate| candidate.method_pointer).filter(|pointer| *pointer > entry.method_pointer).min();
        method_entry_json(entry, upper)
    }).collect::<Vec<_>>().join(",");
    format!(r#"{{"ok":true,"status":"{}","ambiguous":{},"matches":[{}]}}"#, status, status == "ambiguous", items)
}
```
