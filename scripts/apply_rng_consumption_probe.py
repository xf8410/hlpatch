#!/usr/bin/env python3
"""Wire rng_consumption_probe into hlpatch lib.rs (3.29.x lineage).

方向变更：RNG 老账从「猜种子/重放随机流」改为「消费点插桩」。
_fixedTurnCharaSeed @0x198 是 ObscuredInt（加密 int），不是 PRNG 状态，
单值无法重放（2026-07 证伪）。新路子直接 hook：
  RaceRandom.GetRandom / RaceRandomProgramArray / SingleModeRaceRandomProgram
记「什么时候、在哪个消费点、用了什么随机数」。

Insertions (all anchors asserted, idempotent):
  1. `mod rng_consumption_probe;` after PLUGIN_VERSION const.
  2. `rng_consumption_probe::install();` after every install_event_choice_hook();
     (3 sites, same lazy-retry pattern as factor_index_badge).
  3. 5 HTTP routes before the /api/sniff/clear route.
  4. BOOT_SAFE_EXACT entries (these read only plugin-local atomics, boot-safe).
  5. /health endpoint list gains the new routes.
  6. Legacy deprecated seed endpoints gain `superseded_by` pointing at the new
     direction (the whole point of this change: 老账改道，不是只留一句 deprecated).
  7. Cargo.toml / Cargo.lock version bump 3.28.8 -> 3.29.1 (single source of
     truth via env!("CARGO_PKG_VERSION"); keeps /health and endpoint JSON aligned).

NOTE: this script edits SOURCE only. The user-supplied libhachimi_ura-3.29.0.so
is a stripped aarch64 binary and does NOT contain this probe. Rebuild requires
CI + NDK r26c (no NDK / no push credential in this sandbox).
"""
from pathlib import Path
import sys

SRC = Path("hachimi_ura_plugin/src/lib.rs")
CARGO = Path("hachimi_ura_plugin/Cargo.toml")
LOCK = Path("hachimi_ura_plugin/Cargo.lock")

text = SRC.read_text(encoding="utf-8")

if "mod rng_consumption_probe;" in text:
    print("rng_consumption_probe=already_applied")
else:
    # ---- 1) module declaration -------------------------------------------
    anchor = 'const PLUGIN_VERSION: &str = env!("CARGO_PKG_VERSION");'
    assert text.count(anchor) == 1, f"PLUGIN_VERSION anchor count={text.count(anchor)}"
    text = text.replace(anchor, anchor + "\nmod rng_consumption_probe;", 1)

    # ---- 2) install call sites (3 lazy-retry hooks) ----------------------
    anchor = "install_event_choice_hook();"
    assert text.count(anchor) == 3, f"install_event_choice_hook count={text.count(anchor)}"
    text = text.replace(anchor, anchor + "\n                rng_consumption_probe::install();")

    # ---- 3) HTTP routes ---------------------------------------------------
    anchor = '} else if path == "/api/sniff/clear" {'
    assert text.count(anchor) == 1, f"route anchor count={text.count(anchor)}"
    routes = (
        '} else if path == "/api/rng/consumption" {\n'
        "        rng_consumption_probe::endpoint(&full_uri)\n"
        '    } else if path == "/api/rng/consumption/status" {\n'
        "        rng_consumption_probe::status_endpoint()\n"
        '    } else if path == "/api/rng/consumption/toggle" {\n'
        "        rng_consumption_probe::toggle_endpoint(&full_uri)\n"
        '    } else if path == "/api/rng/consumption/clear" {\n'
        "        rng_consumption_probe::clear_endpoint()\n"
        '    } else if path == "/api/rng/consumption/context" {\n'
        "        rng_consumption_probe::context_endpoint(&full_uri)\n"
        "    "
    )
    text = text.replace(anchor, routes + anchor, 1)

    # ---- 4) BOOT_SAFE_EXACT ----------------------------------------------
    anchor = '            "/api/sniff/clear",'
    assert text.count(anchor) == 1, f"boot_safe anchor count={text.count(anchor)}"
    text = text.replace(
        anchor,
        anchor
        + '\n            "/api/rng/consumption",'
        + '\n            "/api/rng/consumption/status",'
        + '\n            "/api/rng/consumption/toggle",'
        + '\n            "/api/rng/consumption/clear",'
        + '\n            "/api/rng/consumption/context",',
        1,
    )

    # ---- 5) /health + 404-available endpoint lists (2 sites) ------------
    anchor = '"/api/event/choices","/api/event/clear"'
    assert text.count(anchor) == 2, f"endpoint list anchor count={text.count(anchor)}"
    text = text.replace(
        anchor,
        anchor
        + ',"/api/rng/consumption","/api/rng/consumption/status"'
        + ',"/api/rng/consumption/toggle","/api/rng/consumption/clear"'
        + ',"/api/rng/consumption/context"',
    )

    # ---- 6) legacy deprecated endpoints: point at the new direction ------
    # /seed/history + /seed/stats + /debug/training_log_dl
    old_seed = (
        '{"ok":false,"deprecated":true,"rng_observation_valid":false,'
        '"rng_invalid_reason":"offset_0x198_is_ObscuredInt_not_u32x4"}'
    )
    new_seed = (
        '{"ok":false,"deprecated":true,"rng_observation_valid":false,'
        '"rng_invalid_reason":"offset_0x198_is_ObscuredInt_not_u32x4",'
        '"direction":"consumption_probe_not_seed_replay",'
        '"superseded_by":"/api/rng/consumption",'
        '"note":"_fixedTurnCharaSeed 是 ObscuredInt 加密 int，非 PRNG 状态，单值不可重放；改看消费点流水"}'
    )
    n = text.count(old_seed)
    assert n == 2, f"seed deprecated count={n}"
    text = text.replace(old_seed, new_seed)

    # /debug/training_log (hooked/addr variant) — 源码里是 Rust format! 双花括号
    old_log = (
        '{{"hooked":{},"addr":"0x{:x}","rng_observation_valid":false,'
        '"rng_invalid_reason":"offset_0x198_is_ObscuredInt_not_u32x4"}}'
    )
    new_log = (
        '{{"hooked":{},"addr":"0x{:x}","rng_observation_valid":false,'
        '"rng_invalid_reason":"offset_0x198_is_ObscuredInt_not_u32x4",'
        '"superseded_by":"/api/rng/consumption"}}'
    )
    assert text.count(old_log) == 1, f"training_log count={text.count(old_log)}"
    text = text.replace(old_log, new_log, 1)

    # /debug/training_seed: keep the field read, but state the conclusion
    old_ts = '"is_complete_prng_state":false}}"'
    new_ts = (
        '"is_complete_prng_state":false,"single_value_replay":false,'
        '"superseded_by":"/api/rng/consumption","note":"字段可读但不可用于重放随机流"}}"'
    )
    assert text.count(old_ts) == 1, f"training_seed count={text.count(old_ts)}"
    text = text.replace(old_ts, new_ts, 1)

    # ExecTraining hook comment: reflect the direction change
    old_c = "// ExecTraining hook is retained only to preserve existing hook behavior; no RNG observations are captured."
    new_c = (
        "// ExecTraining hook is retained only to preserve existing hook behavior.\n"
        "// RNG observations now come from rng_consumption_probe (consumption points),\n"
        "// NOT from _fixedTurnCharaSeed — that field is ObscuredInt, not a PRNG state."
    )
    assert text.count(old_c) == 1, f"exec comment count={text.count(old_c)}"
    text = text.replace(old_c, new_c, 1)

    SRC.write_text(text, encoding="utf-8")
    print("rng_consumption_probe=applied")

# ---- 7) version bump (single source of truth) ----------------------------
for path, old, new, expected in [
    (CARGO, 'version = "3.28.8"', 'version = "3.29.1"', 1),
    (LOCK, 'version = "3.28.8"', 'version = "3.29.1"', 1),
]:
    t = path.read_text(encoding="utf-8")
    if new in t:
        print(f"{path.name}=already_{new}")
        continue
    c = t.count(old)
    assert c == expected, f"{path.name} old-version count={c}"
    path.write_text(t.replace(old, new, 1), encoding="utf-8")
    print(f"{path.name}=bumped_{new}")

print("done")
