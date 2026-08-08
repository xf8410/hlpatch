from pathlib import Path

SOURCE = Path("hachimi_ura_plugin/src/lib.rs")
s = SOURCE.read_text(encoding="utf-8")

replacements = [
    (
        "let mut found = ptr::null_mut();",
        "let mut found: *mut c_void = ptr::null_mut();",
    ),
    (
        "parse_request_uri(req).unwrap_or_else(|_| full_uri.clone());",
        "parse_request_uri(req).unwrap_or_else(|_| full_uri.to_string());",
    ),
]

changed = False
for old, new in replacements:
    old_count = s.count(old)
    new_count = s.count(new)
    if old_count == 1 and new_count == 0:
        s = s.replace(old, new, 1)
        changed = True
    elif old_count == 0 and new_count == 1:
        continue
    else:
        raise AssertionError(
            f"unexpected replacement state old={old!r} old_count={old_count} new_count={new_count}"
        )

SOURCE.write_text(s, encoding="utf-8")
print(f"unified_endpoint_a_compile_fix={'applied' if changed else 'already_applied'}")
