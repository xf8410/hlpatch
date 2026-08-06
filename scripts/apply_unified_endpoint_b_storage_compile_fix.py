from pathlib import Path

SOURCE = Path("hachimi_ura_plugin/src/lib.rs")
s = SOURCE.read_text(encoding="utf-8")

old = 'let session_json = current_session.map(|value| format!("\\\"{}\\\"", json_escape(&value))).unwrap_or_else(|| "null".to_string());'
new = 'let session_json = current_session.as_ref().map(|value| format!("\\\"{}\\\"", json_escape(value))).unwrap_or_else(|| "null".to_string());'
old_count = s.count(old)
new_count = s.count(new)
if old_count == 1 and new_count == 0:
    s = s.replace(old, new, 1)
    status = "applied"
elif old_count == 0 and new_count == 1:
    status = "already_applied"
else:
    raise AssertionError(f"unexpected storage compile fix state old_count={old_count} new_count={new_count}")

SOURCE.write_text(s, encoding="utf-8")
print(f"unified_endpoint_b_storage_compile_fix={status}")
