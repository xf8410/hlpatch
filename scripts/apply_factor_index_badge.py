#!/usr/bin/env python3
"""Wire factor_index_badge into the generated lib.rs (3.28.2 lineage).

Insertions (all anchors asserted, idempotent):
  1. `mod factor_index_badge;` after the PLUGIN_VERSION const.
  2. `factor_index_badge::install();` after every `install_event_choice_hook();`
     (3 sites: push-loop fallback, lazy retry, game-init callback).
  3. /api/factor_index routes before the /api/sniff/clear route.
  4. BOOT_SAFE_EXACT entries for the new routes.
  5. v3.28.2 crash-log parity: uma_predict.log path -> user-readable media dir.

Anchor shapes verified by docs/FACTOR_BADGE_ANCHOR_DUMP.md
(run 39032660932, 2026-10-17, chain cumulative->3.27.23->guard->mirror).
"""
from pathlib import Path
import sys

SRC = Path("hachimi_ura_plugin/src/lib.rs")
text = SRC.read_text(encoding="utf-8")

if "mod factor_index_badge;" in text:
    print("factor_index_badge=already_applied")
    sys.exit(0)

# 1) module declaration
anchor = 'const PLUGIN_VERSION: &str = env!("CARGO_PKG_VERSION");'
assert text.count(anchor) == 1, f"PLUGIN_VERSION anchor count={text.count(anchor)}"
text = text.replace(anchor, anchor + "\nmod factor_index_badge;", 1)

# 2) install call sites
anchor = "install_event_choice_hook();"
count = text.count(anchor)
assert count == 3, f"install_event_choice_hook count={count}"
text = text.replace(anchor, anchor + "\n                factor_index_badge::install();")

# 3) HTTP routes
anchor = '} else if path == "/api/sniff/clear" {'
assert text.count(anchor) == 1, f"route anchor count={text.count(anchor)}"
routes = (
    '} else if path == "/api/factor_index" {\n'
    "        factor_index_badge::endpoint()\n"
    '    } else if path == "/api/factor_index/on" {\n'
    "        factor_index_badge::enable_endpoint()\n"
    '    } else if path == "/api/factor_index/off" {\n'
    "        factor_index_badge::disable_endpoint()\n"
    "    "
)
text = text.replace(anchor, routes + anchor, 1)

# 4) BOOT_SAFE_EXACT entries (queryable before game init)
anchor = '            "/api/sniff/clear",'
assert text.count(anchor) == 1, f"boot_safe anchor count={text.count(anchor)}"
text = text.replace(
    anchor,
    anchor
    + '\n            "/api/factor_index",\n            "/api/factor_index/on",\n            "/api/factor_index/off",',
    1,
)

# 5) v3.28.2 crash-log parity: predict/crash log -> media dir (user-readable)
old = "/data/data/jp.pokemon.pokeuma/files/uma_predict.log"
new = "/sdcard/Android/media/jp.co.cygames.umamusume/hachimi/uma_predict.log"
count = text.count(old)
assert count >= 1, "uma_predict.log path not found"
text = text.replace(old, new)
print(f"crash_log_path_rewrites={count}")

SRC.write_text(text, encoding="utf-8")
print("factor_index_badge=applied")
