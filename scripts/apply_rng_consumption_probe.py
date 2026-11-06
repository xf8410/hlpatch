#!/usr/bin/env python3
"""把 rng_consumption_probe 接线进 hlpatch lib.rs（3.29.x，幂等）。

方向变更：RNG 老账从「猜种子/重放随机流」改为「消费点插桩」。
_fixedTurnCharaSeed @0x198 是 ObscuredInt（加密 int），不是 PRNG 状态，
单值无法重放。改为 hook GetRandom / RaceRandomProgramArray，记
「什么时候、在哪个消费点、用了什么随机数」。

只改源码，不产出二进制；.so 由 CI + NDK r26c 编译。
"""
from pathlib import Path

SRC = Path("hachimi_ura_plugin/src/lib.rs")
CARGO = Path("hachimi_ura_plugin/Cargo.toml")
LOCK = Path("hachimi_ura_plugin/Cargo.lock")

text = SRC.read_text(encoding="utf-8")

# 下面三引号里出现的 {{ }} 是 lib.rs 既有 Rust format! 的花括号转义，
# 属于要精确匹配的目标文本，不是 Python 转义。

ROUTES = '''} else if path == "/api/rng/consumption" {
        rng_consumption_probe::endpoint(&full_uri)
    } else if path == "/api/rng/consumption/status" {
        rng_consumption_probe::status_endpoint()
    } else if path == "/api/rng/consumption/toggle" {
        rng_consumption_probe::toggle_endpoint(&full_uri)
    } else if path == "/api/rng/consumption/clear" {
        rng_consumption_probe::clear_endpoint()
    } else if path == "/api/rng/consumption/context" {
        rng_consumption_probe::context_endpoint(&full_uri)
    '''

ROUTE_ANCHOR = '} else if path == "/api/sniff/clear" {'
BOOT_ANCHOR = '            "/api/sniff/clear",'
LIST_ANCHOR = '"/api/event/choices","/api/event/clear"'

OLD_SEED = '''{"ok":false,"deprecated":true,"rng_observation_valid":false,"rng_invalid_reason":"offset_0x198_is_ObscuredInt_not_u32x4"}'''
NEW_SEED = '''{"ok":false,"deprecated":true,"rng_observation_valid":false,"rng_invalid_reason":"offset_0x198_is_ObscuredInt_not_u32x4","direction":"consumption_probe_not_seed_replay","superseded_by":"/api/rng/consumption","note":"_fixedTurnCharaSeed 是 ObscuredInt 加密 int，非 PRNG 状态，单值不可重放；改看消费点流水"}'''

# 源码里是 Rust format! 双花括号，必须照原样匹配
OLD_LOG = '''{{"hooked":{},"addr":"0x{:x}","rng_observation_valid":false,"rng_invalid_reason":"offset_0x198_is_ObscuredInt_not_u32x4"}}'''
NEW_LOG = '''{{"hooked":{},"addr":"0x{:x}","rng_observation_valid":false,"rng_invalid_reason":"offset_0x198_is_ObscuredInt_not_u32x4","superseded_by":"/api/rng/consumption"}}'''

OLD_TS = '"is_complete_prng_state":false}}"'
NEW_TS = '"is_complete_prng_state":false,"single_value_replay":false,"superseded_by":"/api/rng/consumption","note":"字段可读但不可用于重放随机流"}}"'

OLD_COMMENT = "// ExecTraining hook is retained only to preserve existing hook behavior; no RNG observations are captured."
NEW_COMMENT = """// ExecTraining hook is retained only to preserve existing hook behavior.
// RNG observations now come from rng_consumption_probe (consumption points),
// NOT from _fixedTurnCharaSeed — that field is ObscuredInt, not a PRNG state."""

RNG_ROUTES = '''"/api/rng/consumption","/api/rng/consumption/status","/api/rng/consumption/toggle","/api/rng/consumption/clear","/api/rng/consumption/context"'''
BOOT_ROUTES = '''            "/api/rng/consumption",
            "/api/rng/consumption/status",
            "/api/rng/consumption/toggle",
            "/api/rng/consumption/clear",
            "/api/rng/consumption/context",'''


def replace_once(s: str, old: str, new: str, count: int = 1) -> str:
    """按期望次数断言锚点唯一，再替换。"""
    n = s.count(old)
    assert n == count, f"anchor count={n}, expect {count}: {old[:60]}"
    return s.replace(old, new, count)


if "mod rng_consumption_probe;" in text:
    print("rng_consumption_probe=already_applied")
else:
    text = replace_once(
        text,
        'const PLUGIN_VERSION: &str = env!("CARGO_PKG_VERSION");',
        'const PLUGIN_VERSION: &str = env!("CARGO_PKG_VERSION");\nmod rng_consumption_probe;',
    )

    # 3 处懒重试安装点
    anchor = "install_event_choice_hook();"
    n = text.count(anchor)
    assert n == 3, f"install_event_choice_hook count={n}"
    text = text.replace(anchor, anchor + "\n                rng_consumption_probe::install();")

    text = replace_once(text, ROUTE_ANCHOR, ROUTES + ROUTE_ANCHOR)
    text = replace_once(text, BOOT_ANCHOR, BOOT_ANCHOR + "\n" + BOOT_ROUTES)
    text = replace_once(text, LIST_ANCHOR, LIST_ANCHOR + "," + RNG_ROUTES, count=2)
    text = replace_once(text, OLD_SEED, NEW_SEED, count=2)
    text = replace_once(text, OLD_LOG, NEW_LOG)
    text = replace_once(text, OLD_TS, NEW_TS)
    text = replace_once(text, OLD_COMMENT, NEW_COMMENT)

    SRC.write_text(text, encoding="utf-8")
    print("rng_consumption_probe=applied")

# 版本号：单一真相 env!("CARGO_PKG_VERSION")，toml 与 lock 同步
for path in (CARGO, LOCK):
    t = path.read_text(encoding="utf-8")
    if 'version = "3.29.1"' in t:
        print(f"{path.name}=already_3.29.1")
        continue
    n = t.count('version = "3.28.8"')
    assert n == 1, f"{path.name} old-version count={n}"
    path.write_text(t.replace('version = "3.28.8"', 'version = "3.29.1"', 1), encoding="utf-8")
    print(f"{path.name}=bumped_3.29.1")

print("done")
