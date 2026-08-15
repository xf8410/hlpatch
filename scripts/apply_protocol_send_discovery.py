from pathlib import Path
import re

SOURCE = Path("hachimi_ura_plugin/src/lib.rs")
MARKER = "// ===== Passive in-game protocol send discovery ====="
s = SOURCE.read_text(encoding="utf-8")
if MARKER in s:
    print("protocol_send_discovery=already_applied")
    raise SystemExit(0)

module_anchor = "#![allow(dead_code)]\n"
assert s.count(module_anchor) == 1, f"module anchor count={s.count(module_anchor)}"
s = s.replace(module_anchor, module_anchor + "mod protocol_send_discovery;\n", 1)

boot = re.search(r"((?:static|const)\s+BOOT_SAFE_EXACT\b[^=]*=\s*&\[)", s, re.M)
assert boot is not None, "BOOT_SAFE_EXACT declaration missing"
boot_end = s.find("];", boot.end())
assert boot_end >= 0, "BOOT_SAFE_EXACT terminator missing"
for route in [
    "/api/protocol/send/discovery",
    "/api/protocol/send/candidates",
    "/api/protocol/send/evidence",
]:
    if f'"{route}"' not in s[boot.start():boot_end]:
        s = s[:boot.end()] + f'\n    "{route}",' + s[boot.end():]
        boot_end += len(route) + 8

route_anchor = '    } else if path == "/api/sniff/signup_plaintext" {\n'
assert s.count(route_anchor) == 1, f"route anchor count={s.count(route_anchor)}"
routes = '''    } else if path == "/api/protocol/send/discovery" {
        unsafe { protocol_send_discovery::discovery_endpoint() }
    } else if path == "/api/protocol/send/candidates" {
        unsafe { protocol_send_discovery::candidates_endpoint() }
    } else if path == "/api/protocol/send/evidence" {
        unsafe { protocol_send_discovery::evidence_endpoint(&full_uri) }
'''
s = s.replace(route_anchor, routes + route_anchor, 1)

source_anchor = "/// 辅助函数：IL2CPP类型枚举转可读名称\n"
assert s.count(source_anchor) == 1, f"source anchor count={s.count(source_anchor)}"
s = s.replace(source_anchor, MARKER + "\n" + source_anchor, 1)
SOURCE.write_text(s, encoding="utf-8")
print("protocol_send_discovery=applied")
