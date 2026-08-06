# `il2cpp_method_by_addr`

source_commit: `a340a147acf13672b2fbc64925bfa321d08091fd`
source_line: `22599`

```rust
unsafe fn il2cpp_method_by_addr(uri: &str) -> String {
    let pairs = match parse_query_pairs(uri) { Ok(value) => value, Err(error) => return format!(r#"{{"ok":false,"error":"{}"}}"#, json_escape(&error)) };
    let raw = query_pair(&pairs, "addr");
    let address = match parse_address(&raw) { Some(value) if value != 0 => value, _ => return r#"{"ok":false,"error":"invalid_or_missing_addr"}"#.to_string() };
    if let Err(error) = ensure_method_index() { return format!(r#"{{"ok":false,"error":"{}"}}"#, json_escape(&error)); }
    let state = match METHOD_INDEX.lock() { Ok(value) => value, Err(_) => return r#"{"ok":false,"error":"method_index_lock_poisoned"}"#.to_string() };
    let method_info_matches: Vec<&MethodIndexEntry> = state.entries.iter().filter(|entry| entry.method_info == address).collect();
    let exact_pointer_matches: Vec<&MethodIndexEntry> = state.entries.iter().filter(|entry| entry.method_pointer == address && entry.method_pointer != 0).collect();
    let (kind, matches): (&str, Vec<&MethodIndexEntry>) = if !method_info_matches.is_empty() {
        (if method_info_matches.len() == 1 { "exact_method_info" } else { "ambiguous" }, method_info_matches)
    } else if !exact_pointer_matches.is_empty() {
        (if exact_pointer_matches.len() == 1 { "exact_method_pointer" } else { "ambiguous" }, exact_pointer_matches)
    } else {
        let mut distinct: Vec<usize> = state.entries.iter().map(|entry| entry.method_pointer).filter(|value| *value != 0).collect();
        distinct.dedup();
        match distinct.binary_search(&address) {
            Ok(_) => ("none", Vec::new()),
            Err(position) if position > 0 && position < distinct.len() => {
                let start = distinct[position - 1];
                let candidates: Vec<&MethodIndexEntry> = state.entries.iter().filter(|entry| entry.method_pointer == start).collect();
                (if candidates.len() == 1 { "upper_bound_candidate" } else { "ambiguous" }, candidates)
            }
            _ => ("none", Vec::new()),
        }
    };
    let items = matches.iter().map(|entry| {
        let upper = state.entries.iter().map(|candidate| candidate.method_pointer).filter(|pointer| *pointer > entry.method_pointer).min();
        method_entry_json(entry, upper)
    }).collect::<Vec<_>>().join(",");
    format!(r#"{{"ok":true,"query":"0x{:x}","status":"{}","ambiguous":{},"matches":[{}],"index":{{"status":"{}","classes":{},"methods":{},"null_method_pointers":{},"duplicate_method_pointers":{}}}}}"#,
        address, kind, kind == "ambiguous", items, state.status, state.indexed_class_count,
        state.indexed_method_count, state.null_method_pointer_count, state.duplicate_method_pointer_count)
}
```
