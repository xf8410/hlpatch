from pathlib import Path

path = Path("scripts/apply_unified_endpoint_f_pre_release_fix.py")
s = path.read_text(encoding="utf-8")
marker = "# F session-row anchor correction"
if marker in s:
    print("f_patch_anchor_fix=already_applied")
    raise SystemExit(0)
start = s.index("# Never silently discard malformed session rows.\n")
end = s.index("# Checkpoint must succeed before publishing a new flush timestamp.\n", start)
replacement = '''# Never silently discard malformed session rows.
# F session-row anchor correction
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
path.write_text(s, encoding="utf-8")
print("f_patch_anchor_fix=applied")
