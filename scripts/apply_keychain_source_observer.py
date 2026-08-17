from pathlib import Path
import re

SOURCE = Path("hachimi_ura_plugin/src/lib.rs")
MARKER = "// ===== Keychain source observer integration ====="
s = SOURCE.read_text(encoding="utf-8")
if MARKER in s:
    print("keychain_source_observer=already_applied")
    raise SystemExit(0)

module_anchor = "#![allow(dead_code)]\n"
assert s.count(module_anchor) == 1, f"module anchor count={s.count(module_anchor)}"
s = s.replace(module_anchor, module_anchor + "mod keychain_source_observer;\n", 1)

boot = re.search(r"((?:static|const)\s+BOOT_SAFE_EXACT\b[^=]*=\s*&\[)", s, re.M)
assert boot is not None, "BOOT_SAFE_EXACT declaration missing"
boot_end = s.find("];", boot.end())
assert boot_end >= 0, "BOOT_SAFE_EXACT terminator missing"
for route in ["/api/keychain/source", "/api/keychain/source/clear"]:
    if f'"{route}"' not in s[boot.start():boot_end]:
        s = s[:boot.end()] + f'\n    "{route}",' + s[boot.end():]
        boot_end += len(route) + 8

install_anchor = '''unsafe fn install_api_sniff_hooks() {
    install_text_common_observer_hook();
'''
install_replacement = '''unsafe fn install_api_sniff_hooks() {
    install_text_common_observer_hook();
    keychain_source_observer::install();
'''
assert s.count(install_anchor) == 1, f"install anchor count={s.count(install_anchor)}"
s = s.replace(install_anchor, install_replacement, 1)

route_anchor = '    } else if path == "/api/sniff/signup_plaintext" {\n'
assert s.count(route_anchor) == 1, f"route anchor count={s.count(route_anchor)}"
routes = '''    } else if path == "/api/keychain/source" {
        keychain_source_observer::endpoint(&full_uri)
    } else if path == "/api/keychain/source/clear" {
        keychain_source_observer::clear_endpoint()
'''
s = s.replace(route_anchor, routes + route_anchor, 1)

source_anchor = "/// 辅助函数：IL2CPP类型枚举转可读名称\n"
assert s.count(source_anchor) == 1, f"source anchor count={s.count(source_anchor)}"
s = s.replace(source_anchor, MARKER + "\n" + source_anchor, 1)
SOURCE.write_text(s, encoding="utf-8")
print("keychain_source_observer=applied")
