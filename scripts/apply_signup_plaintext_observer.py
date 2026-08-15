from pathlib import Path
import re

SOURCE = Path("hachimi_ura_plugin/src/lib.rs")
MARKER = "// ===== Signup plaintext observer D1 ====="
s = SOURCE.read_text(encoding="utf-8")
if MARKER in s:
    print("signup_plaintext_observer=already_applied")
    raise SystemExit(0)

module_anchor = "#![allow(dead_code)]\n"
assert s.count(module_anchor) == 1, f"module anchor count={s.count(module_anchor)}"
s = s.replace(module_anchor, module_anchor + "mod signup_plaintext;\n", 1)

boot = re.search(r"((?:static|const)\s+BOOT_SAFE_EXACT\b[^=]*=\s*&\[)", s, re.M)
assert boot is not None, "BOOT_SAFE_EXACT declaration missing"
boot_end = s.find("];", boot.end())
assert boot_end >= 0, "BOOT_SAFE_EXACT terminator missing"
if '"/api/sniff/signup_plaintext"' not in s[boot.start():boot_end]:
    s = s[:boot.end()] + '\n    "/api/sniff/signup_plaintext",' + s[boot.end():]

route_anchor = '    } else if path == "/storage/files" {\n'
assert s.count(route_anchor) == 1, f"route anchor count={s.count(route_anchor)}"
s = s.replace(
    route_anchor,
    '''    } else if path == "/api/sniff/signup_plaintext" {
        unsafe { signup_plaintext::endpoint() }
''' + route_anchor,
    1,
)

source_anchor = "/// 辅助函数：IL2CPP类型枚举转可读名称\n"
assert s.count(source_anchor) == 1, f"source marker anchor count={s.count(source_anchor)}"
s = s.replace(source_anchor, MARKER + "\n" + source_anchor, 1)
SOURCE.write_text(s, encoding="utf-8")
print("signup_plaintext_observer=applied")
