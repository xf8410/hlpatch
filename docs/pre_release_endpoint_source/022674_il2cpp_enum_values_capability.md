# `il2cpp_enum_values_capability`

source_commit: `ffc3748df2d3c8c57b34aa3fdd64f75d09ed0866`
source_line: `22674`

```rust
unsafe fn il2cpp_enum_values_capability(uri: &str) -> String {
    let pairs = match parse_query_pairs(uri) { Ok(value) => value, Err(error) => return format!(r#"{{"ok":false,"error":"{}"}}"#, json_escape(&error)) };
    let requested = query_pair(&pairs, "type");
    let required = ["il2cpp_class_get_fields", "il2cpp_field_get_flags", "il2cpp_field_static_get_value"];
    let available: Vec<bool> = required.iter().map(|name| !resolve_il2cpp_symbol(name).is_null()).collect();
    format!(r#"{{"ok":true,"requested":"{}","value_status":"unresolved","integer_values":null,"declaration_order_inference":false,"runtime_api":{{"il2cpp_class_get_fields":{},"il2cpp_field_get_flags":{},"il2cpp_field_static_get_value":{}}}}}"#,
        json_escape(&requested), available[0], available[1], available[2])
}
```
