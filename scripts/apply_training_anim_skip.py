from pathlib import Path

SOURCE = Path("hachimi_ura_plugin/src/lib.rs")
MARKER = "// ===== Training animation skip D-T1 ====="
s = SOURCE.read_text(encoding="utf-8")
if MARKER in s:
    print("training_anim_skip=already_applied")
    raise SystemExit(0)

# 1. module declaration
module_anchor = "#![allow(dead_code)]\n"
assert s.count(module_anchor) == 1, f"module anchor count={s.count(module_anchor)}"
s = s.replace(module_anchor, module_anchor + "mod training_anim_skip;\n", 1)

# 2. boot-safe exact routes (status/toggle stay available pre-game-init)
boot_anchor = '"/api/sniff/signup_plaintext",'
assert boot_anchor in s, "signup boot-safe anchor missing"
for route in ('"/api/training/anim_skip"', '"/api/training/anim_skip/on"', '"/api/training/anim_skip/off"'):
    if route not in s:
        s = s.replace(
            boot_anchor,
            boot_anchor + "\n    " + route + ",",
            1,
        )

# 3. route dispatch (exact-path style, mirrors sibling observers)
route_anchor = '    } else if path == "/api/sniff/signup_plaintext" {\n'
assert route_anchor in s, f"route anchor missing"
dispatch_block = (
    '    } else if path == "/api/training/anim_skip" {\n'
    "        training_anim_skip::endpoint()\n"
    '    } else if path == "/api/training/anim_skip/on" {\n'
    "        training_anim_skip::enable_endpoint()\n"
    '    } else if path == "/api/training/anim_skip/off" {\n'
    "        training_anim_skip::disable_endpoint()\n"
)
s = s.replace(route_anchor, dispatch_block + route_anchor, 1)

# 4. installer invocation near other observers' install sites
install_anchor = "keychain_source_observer::install();"
if install_anchor in s:
    s = s.replace(install_anchor, install_anchor + "\n    training_anim_skip::install();", 1)
else:
    import re
    m = re.search(r"^(\s*)(\w+_observer::install\(\);)", s, re.M)
    assert m, "no observer install anchor found"
    s = s[: m.end()] + "\n" + m.group(1) + "training_anim_skip::install();" + s[m.end():]

# 5. source marker for idempotency bookkeeping
source_anchor = "/// 辅助函数：IL2CPP类型枚举转可读名称\n"
assert s.count(source_anchor) == 1, f"source marker anchor count={s.count(source_anchor)}"
s = s.replace(source_anchor, MARKER + "\n" + source_anchor, 1)

SOURCE.write_text(s, encoding="utf-8")
print("training_anim_skip=applied")
