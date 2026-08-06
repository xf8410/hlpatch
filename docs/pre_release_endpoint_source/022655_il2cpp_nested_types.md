# `il2cpp_nested_types`

source_commit: `ffc3748df2d3c8c57b34aa3fdd64f75d09ed0866`
source_line: `22655`

```rust
unsafe fn il2cpp_nested_types(uri: &str) -> String {
    let pairs = match parse_query_pairs(uri) { Ok(value) => value, Err(error) => return format!(r#"{{"ok":false,"error":"{}"}}"#, json_escape(&error)) };
    let requested = query_pair(&pairs, "type");
    if requested.is_empty() { return r#"{"ok":false,"error":"missing_type"}"#.to_string(); }
    let class = find_class_by_full_declaring_name(&requested);
    if class.is_null() { return format!(r#"{{"ok":false,"error":"class_not_found_or_ambiguous","type":"{}"}}"#, json_escape(&requested)); }
    let nested_ptr = resolve_il2cpp_symbol("il2cpp_class_get_nested_types");
    if nested_ptr.is_null() { return r#"{"ok":false,"error":"il2cpp_class_get_nested_types_unavailable"}"#.to_string(); }
    let get_nested: unsafe extern "C" fn(*mut c_void, *mut *mut c_void) -> *mut c_void = std::mem::transmute(nested_ptr);
    let mut iterator = ptr::null_mut();
    let mut items = Vec::new();
    loop {
        let nested = get_nested(class, &mut iterator);
        if nested.is_null() { break; }
        items.push(format!(r#"{{"type":"{}","class_pointer":"0x{:x}"}}"#, json_escape(&class_full_declaring_name(nested)), nested as usize));
    }
    format!(r#"{{"ok":true,"requested":"{}","direct_only":true,"count":{},"nested_types":[{}]}}"#, json_escape(&requested), items.len(), items.join(","))
}
```
