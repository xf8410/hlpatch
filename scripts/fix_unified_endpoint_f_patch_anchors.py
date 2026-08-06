from pathlib import Path

path = Path("scripts/apply_unified_endpoint_f_pre_release_fix.py")
s = path.read_text(encoding="utf-8")

# Correct the session-row source anchor.
start = s.index("# Never silently discard malformed session rows.\n")
end = s.index("# Checkpoint must succeed before publishing a new flush timestamp.\n", start)
replacement = '''# Never silently discard malformed session rows.
session_rows_old = '    let sessions: Vec<String> = rows.filter_map(Result::ok).collect();\\n'
session_rows_new = ''' + "'''" + '''    let mut sessions = Vec::new();
    for row in rows {
        match row {
            Ok(value) => sessions.push(value),
            Err(error) => {
                let detail = format!("decode_session_row:{}", error);
                storage_set_error(&detail);
                return format!(r#"{{"ok":false,"error":"{}","sessions":[]}}"#, json_escape(&detail));
            }
        }
    }
    storage_clear_error();
''' + "'''" + '''
replace_once(session_rows_old, session_rows_new, "sessions_rows")
'''
s = s[:start] + replacement + s[end:]

# Source routes and raw JSON strings contain ordinary quote characters, not
# backslash-escaped quote bytes. Validate route presence with the real token and
# insert ordinary JSON list elements into the two raw format strings.
start = s.index('for endpoint in [\n', s.index('# Add the new status route'))
end = s.index('\nanchor = "/// 辅助函数', start)
route_block = '''for endpoint in [
    "/storage/status", "/storage/sessions", "/storage/session", "/storage/flush", "/storage/recover",
    "/il2cpp/method_index_status", "/il2cpp/method_by_addr", "/il2cpp/method_detail",
    "/il2cpp/nested_types", "/il2cpp/enum_values", "/inherit/pair_compat", "/inherit/selected_parent_runtime",
]:
    token = f'"{endpoint}"'
    if token not in s:
        raise AssertionError(f"route token absent before advertisement: {endpoint}")
new_advertised = '"/storage/status","/storage/sessions","/storage/session","/storage/flush","/storage/recover","/il2cpp/method_index_status","/il2cpp/method_by_addr","/il2cpp/method_detail","/il2cpp/nested_types","/il2cpp/enum_values","/inherit/pair_compat","/inherit/selected_parent_runtime",'
health_needle = 'r#"{{"status":"ok","version":"{}","endpoints":['
assert s.count(health_needle) == 1, s.count(health_needle)
s = s.replace(health_needle, health_needle + new_advertised, 1)
available_needle = 'r#"{{"error":"not_found","path":"{}","available":['
assert s.count(available_needle) == 1, s.count(available_needle)
s = s.replace(available_needle, available_needle + new_advertised, 1)
'''
s = s[:start] + route_block + s[end:]
path.write_text(s, encoding="utf-8")
print("f_patch_anchor_fix=applied")
