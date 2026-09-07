from pathlib import Path
import re

SOURCE = Path("hachimi_ura_plugin/src/lib.rs")
MARKER = "// ===== HTTP proxy endpoint integration (workbench/http-proxy-endpoint-20260907) ====="
s = SOURCE.read_text(encoding="utf-8")
if MARKER in s:
    print("http_proxy_integration=already_applied")
    raise SystemExit(0)

# 1. module declaration
module_anchor = "#![allow(dead_code)]\n"
assert s.count(module_anchor) == 1, f"module anchor count={s.count(module_anchor)}"
s = s.replace(module_anchor, module_anchor + "mod http_proxy;\n", 1)

# 2. register routes as boot-safe (usable before game init; /proxy never touches
#    game memory, so it is safe while the engine is still loading)
boot = re.search(r"((?:static|const)\s+BOOT_SAFE_EXACT\b[^=]*=\s*&\[)", s, re.M)
assert boot is not None, "BOOT_SAFE_EXACT declaration missing"
boot_end = s.find("];", boot.end())
assert boot_end >= 0, "BOOT_SAFE_EXACT terminator missing"
for route in ["/proxy", "/proxy/status"]:
    if f'"{route}"' not in s[boot.start():boot_end]:
        s = s[:boot.end()] + f'\n    "{route}",' + s[boot.end():]
        boot_end += len(route) + 8

# 3. route handlers — inserted before the /api/sniff/signup_plaintext branch
#    (stable anchor used by the keychain observer patch)
route_anchor = '    } else if path == "/api/sniff/signup_plaintext" {\n'
assert s.count(route_anchor) == 1, f"route anchor count={s.count(route_anchor)}"
routes = '''    } else if path == "/proxy" {
        http_proxy::endpoint(&full_uri)
    } else if path == "/proxy/status" {
        http_proxy::status_endpoint()
'''
s = s.replace(route_anchor, routes + route_anchor, 1)

# 4. marker before the standard helper anchor
source_anchor = "/// 辅助函数：IL2CPP类型枚举转可读名称\n"
assert s.count(source_anchor) == 1, f"source anchor count={s.count(source_anchor)}"
s = s.replace(source_anchor, MARKER + "\n" + source_anchor, 1)

SOURCE.write_text(s, encoding="utf-8")
print("http_proxy_integration=applied")
