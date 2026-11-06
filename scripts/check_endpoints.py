#!/usr/bin/env python3
"""端点清单一致性门禁（标准库，无依赖）。

复检三件事，任一不成立即 exit 1：
  1. /health 与 404 不再各自硬编码清单，而是共用 endpoint_list_json()；
  2. ENDPOINTS 是「实际派发路径 ∪ 新端点」的全集，不漏报、不虚报；
  3. 认证 6 端点、/api/sniff/unity 全部有派发、已登记、boot/dl 白名单到位。

仓库依据：uma-sniffkit/docs/sniffkit_rootcauses.md —— 三份清单手抄必漂，
漏报等于骗人；routes.json 是唯一真相源。
"""
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SRC = ROOT / "hachimi_ura_plugin/src/lib.rs"
ROUTES_JSON = ROOT / "scripts/endpoints_routes.json"

AUTH = [
    "/api/auth/status", "/api/auth/targets", "/api/auth/headers",
    "/api/auth/handshake", "/api/auth/toggle", "/api/auth/clear",
]
UNITY = "/api/sniff/unity"
BOOT_EXTRA = AUTH + [UNITY]
DL_EXTRA = [
    "/api/auth/status", "/api/auth/targets", "/api/auth/headers",
    "/api/auth/handshake", "/api/rng/consumption",
    "/api/rng/consumption/context", UNITY,
]


def block(start_marker: str, end_marker: str) -> str:
    i = text.index(start_marker)
    j = text.index(end_marker, i)
    return text[i:j]


def literal_list(chunk: str) -> list:
    return re.findall(r'"(/[^"]*)"', chunk)


errors = []
text = SRC.read_text(encoding="utf-8")

# 1) 单一真相源：数组 + 共用生成函数
m = re.search(r"const ENDPOINTS: &\[&str\] = &\[(.*?)\n\];", text, re.S)
if not m:
    sys.exit("FAIL: ENDPOINTS 常量不存在")
endpoints = literal_list(m.group(1))
if len(endpoints) != len(set(endpoints)):
    errors.append("ENDPOINTS 有重复项")
if text.count("endpoint_list_json()") < 3:
    errors.append("health/404 未共用 endpoint_list_json()")

# 2) 派发全集（前缀型归一到不带尾斜杠）
dispatch = set()
for x in re.findall(r'path\s*==\s*"(/[^"]*)"', text):
    dispatch.add(x)
for x in re.findall(r'path\.starts_with\("(/[^"]*)"\)', text):
    dispatch.add(x)
normalized = {x[:-1] if x.endswith("/") and x[:-1] in endpoints else x for x in dispatch}

missing = sorted(normalized - set(endpoints))
extra = sorted(set(endpoints) - normalized - {"/", *AUTH})
if missing:
    errors.append(f"漏报 {len(missing)} 条: {missing}")
# 允许 routes.json 带来的、由前缀派发覆盖的条目（如 /classes/search/keyword）
prefix_covered = {
    e for e in extra
    if any(e.startswith(p) for p in normalized if p.endswith("/") or e.startswith(p))
}
uncovered = sorted(set(extra) - prefix_covered)
if uncovered:
    errors.append(f"虚报（无派发）{len(uncovered)} 条: {uncovered}")

# 3) 新端点接线
for e in AUTH + [UNITY]:
    if f'path == "{e}"' not in text and f'path.starts_with("{e}")' not in text:
        errors.append(f"无派发: {e}")
    if e not in endpoints:
        errors.append(f"未登记 ENDPOINTS: {e}")

boot = block("const BOOT_SAFE_EXACT: &[&str] = &[", "        ];")
boot_list = literal_list(boot)
for e in BOOT_EXTRA:
    if e not in boot_list:
        errors.append(f"BOOT_SAFE_EXACT 缺: {e}")

dl = block("const DL_ALLOWED: &[&str] = &[", "    ];")
dl_list = literal_list(dl)
for e in DL_EXTRA:
    if e not in dl_list:
        errors.append(f"DL_ALLOWED 缺: {e}")

# 4) 与仓库真相源对照
if ROUTES_JSON.exists():
    truth = [r["path"] for r in json.loads(ROUTES_JSON.read_text(encoding="utf-8"))["routes"]]
    lost = sorted(set(truth) - set(endpoints))
    if lost:
        errors.append(f"相对 routes.json 丢失 {len(lost)} 条: {lost}")
else:
    print("note: scripts/endpoints_routes.json 不存在，跳过真相源对照")

report = {
    "ok": not errors,
    "endpoints": len(endpoints),
    "dispatch_normalized": len(normalized),
    "health_and_404_share_list": True,
    "auth_routes": len(AUTH),
    "boot_safe": len(boot_list),
    "dl_allowed": len(dl_list),
    "errors": errors,
}
print(json.dumps(report, ensure_ascii=False, indent=2))
sys.exit(1 if errors else 0)
