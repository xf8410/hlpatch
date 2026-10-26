#!/usr/bin/env python3
"""Wire protocol_msgpack into lib.rs（与 apply_factor_index_badge.py 同一套锚点约定）。

插入项（锚点全部断言、幂等）：
  1. `mod protocol_msgpack;` —— 插在 PLUGIN_VERSION 常量之后。
  2. `/api/proto/self_test` 路由 —— 插在 `/api/sniff/clear` 路由之前。
  3. BOOT_SAFE_EXACT 条目 —— 让该路由在游戏初始化前也可查。

不新增任何 hook、不改 sniff/event 热路径；本步只验证「解码器能在真机跑起来」。

跑法：python3 scripts/apply_protocol_msgpack_module.py
"""
from pathlib import Path
import sys

SRC = Path("hachimi_ura_plugin/src/lib.rs")
text = SRC.read_text(encoding="utf-8")

if "mod protocol_msgpack;" in text:
    print("protocol_msgpack=already_applied")
    sys.exit(0)

# 1) module declaration
anchor = 'const PLUGIN_VERSION: &str = env!("CARGO_PKG_VERSION");'
assert text.count(anchor) == 1, f"PLUGIN_VERSION anchor count={text.count(anchor)}"
text = text.replace(anchor, anchor + "\nmod protocol_msgpack;", 1)

# 2) HTTP route
anchor = '} else if path == "/api/sniff/clear" {'
assert text.count(anchor) == 1, f"route anchor count={text.count(anchor)}"
routes = (
    '} else if path == "/api/proto/self_test" {\n'
    "        protocol_msgpack::self_test()\n"
    "    "
)
text = text.replace(anchor, routes + anchor, 1)

# 3) BOOT_SAFE_EXACT entry
anchor = '            "/api/sniff/clear",'
assert text.count(anchor) == 1, f"boot_safe anchor count={text.count(anchor)}"
text = text.replace(
    anchor,
    anchor + '\n            "/api/proto/self_test",',
    1,
)

SRC.write_text(text, encoding="utf-8")
print("protocol_msgpack=applied")
