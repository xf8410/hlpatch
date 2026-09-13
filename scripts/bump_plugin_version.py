#!/usr/bin/env python3
"""Robust idempotent version bump for hachimi_ura_plugin.

Usage: python3 scripts/bump_plugin_version.py <target-version>

Bumps Cargo.toml and Cargo.lock from ANY current version to the target,
so release pipelines never die on a drifted literal anchor (the old
replace('version = "3.27.9"', ...) chain silently no-oped when the
starting version had moved, which blocked releases). The final state is
asserted, keeping the deterministic contract of the old approach.
"""
import re
import sys
from pathlib import Path


def main() -> None:
    if len(sys.argv) != 2:
        raise SystemExit("usage: bump_plugin_version.py <target-version>")
    target = sys.argv[1].strip()
    if not re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+", target):
        raise SystemExit(f"invalid semver target: {target!r}")

    cargo = Path("hachimi_ura_plugin/Cargo.toml")
    text = cargo.read_text(encoding="utf-8")
    new_text, n = re.subn(
        r'(?m)^version = "[0-9]+\.[0-9]+\.[0-9]+"$',
        f'version = "{target}"',
        text,
        count=1,
    )
    if n != 1:
        raise RuntimeError(f"Cargo.toml package version line not found (count={n})")
    cargo.write_text(new_text, encoding="utf-8")

    lock = Path("hachimi_ura_plugin/Cargo.lock")
    lock_text = lock.read_text(encoding="utf-8")
    pattern = re.compile(r'(name = "hachimi_ura"\nversion = ")[0-9]+\.[0-9]+\.[0-9]+(")')
    new_lock, m = pattern.subn(rf"\g<1>{target}\g<2>", lock_text, count=1)
    if m != 1:
        raise RuntimeError(f"Cargo.lock hachimi_ura package block not found (count={m})")
    lock.write_text(new_lock, encoding="utf-8")

    # final-state contract (same guarantee the old literal asserts gave)
    assert f'version = "{target}"' in cargo.read_text(encoding="utf-8")
    assert f'name = "hachimi_ura"\nversion = "{target}"' in lock.read_text(encoding="utf-8")
    print(f"bumped to {target}")


if __name__ == "__main__":
    main()
