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

# 2. boot-safe exact route (status/toggle read stays available pre-game-init)
boot_anchor = '"/api/sniff/signup_plaintext",'
assert s.count(boot_anchor) >= 1, "signup boot-safe anchor missing"
if '"/api/training/anim_skip"' not in s:
    s = s.replace(
        boot_anchor,
        boot_anchor + '\n    "/api/training/anim_skip",',
        1,
    )

# 3. route dispatch
route_anchor = '    } else if path == "/api/sniff/signup_plaintext" {\n'
assert s.count(route_anchor) == 1, f"route anchor count={s.count(route_anchor)}"
s = s.replace(
    route_anchor,
    '''    } else if path.starts_with("/api/training/anim_skip") {
        unsafe { training_anim_skip::endpoint(&uri) }
''' + route_anchor,
    1,
)

# 4. installer invocation near other observers' install sites
install_anchor = "keychain_source_observer::install();"
if install_anchor in s:
    s = s.replace(
        install_anchor,
        install_anchor + "\n    training_anim_skip::install();",
        1,
    )
else:
    # Fallback: register alongside any single observer install call.
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
