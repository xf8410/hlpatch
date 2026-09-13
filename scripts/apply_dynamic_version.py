#!/usr/bin/env python3
"""Single-source the runtime-reported plugin version.

Background: ~30 endpoint responses carried a hardcoded "version":"3.22.91"
string, so query clients (e.g. Agora) saw an ancient version even though
the injected .so was current. This patch rewrites every such literal to a
format placeholder backed by PLUGIN_VERSION (env!(CARGO_PKG_VERSION)), and
fixes the load banner, so the reported version always equals the Cargo
version that the release pipeline bumps.

Idempotent: once no literal remains, re-runs print already_applied.
"""
from pathlib import Path

SOURCE = Path("hachimi_ura_plugin/src/lib.rs")
LITERAL = '"version":"3.22.91"'
REPLACED = '"version":"{}"'

s = SOURCE.read_text(encoding="utf-8")
if LITERAL not in s:
    print("dynamic_version=already_applied")
    raise SystemExit(0)

lines = s.split("\n")
out = []
fixed = 0
for idx, line in enumerate(lines):
    if LITERAL in line and line.strip().startswith('r#"') and line.rstrip().endswith('"#,'):
        prev = lines[idx - 1].rstrip() if idx > 0 else ""
        if not prev.endswith("format!("):
            raise RuntimeError(
                f"line {idx + 1}: expected 'format!(' on the previous line, got: {prev!r}"
            )
        if line.count(LITERAL) != 1:
            raise RuntimeError(f"line {idx + 1}: unexpected literal count")
        indent = len(line) - len(line.lstrip())
        out.append(line.replace(LITERAL, REPLACED, 1))
        out.append(" " * (indent + 4) + "PLUGIN_VERSION,")
        fixed += 1
    else:
        out.append(line)

banner_old = 'ura_log(3, "URA plugin v3.24.9 loaded (Interceptor API hooks)");'
banner_new = (
    'ura_log(3, &format!("URA plugin v{} loaded (Interceptor API hooks)", PLUGIN_VERSION));'
)
result = "\n".join(out)
if banner_old in result:
    assert result.count(banner_old) == 1, "load banner anchor count != 1"
    result = result.replace(banner_old, banner_new, 1)

if LITERAL in result:
    raise RuntimeError(
        f"{result.count(LITERAL)} hardcoded version literals could not be converted"
    )

SOURCE.write_text(result, encoding="utf-8")
print(f"dynamic_version=applied json_fields={fixed} banner=fixed")
