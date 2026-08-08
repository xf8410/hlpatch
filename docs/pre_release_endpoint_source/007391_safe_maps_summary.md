# `safe_maps_summary`

source_commit: `a340a147acf13672b2fbc64925bfa321d08091fd`
source_line: `7391`

```rust
fn safe_maps_summary() -> String {
    let maps=match safe_maps(){Ok(v)=>v,Err(e)=>return format!(r#"{{"ok":false,"error":"maps_read_failed","detail":"{}"}}"#,safe_json(&e.to_string()))};
    let readable=maps.iter().filter(|m|m.perms.starts_with('r')).count();
    let sample=maps.iter().filter(|m|m.perms.starts_with('r')).take(64).map(|m|format!(r#"{{"start":"0x{:x}","end":"0x{:x}","size":{},"perms":"{}","path":"{}"}}"#,m.start,m.end,m.end-m.start,safe_json(&m.perms),safe_json(&m.path))).collect::<Vec<_>>().join(",");
    format!(r#"{{"ok":true,"maps_total":{},"readable":{},"sample_limited":true,"maps":[{}]}}"#,maps.len(),readable,sample)
}
```
