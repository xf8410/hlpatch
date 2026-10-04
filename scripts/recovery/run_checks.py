"""Run source provenance, production policy and portable Rust contract checks."""
from pathlib import Path
import os
import subprocess
import sys

from common import CACHE, ROOT, build_environment, read_json


def main() -> None:
    env = build_environment()
    env["RUSTUP_TOOLCHAIN"] = read_json(ROOT / "recovery/toolchains.json")["rust"]
    CACHE.mkdir(exist_ok=True)
    commands = [
        [sys.executable, "-m", "unittest", "discover", "-s", "scripts/recovery/tests", "-v"],
        [sys.executable, "-m", "unittest", "discover", "-s", "tests/recovery", "-v"],
        ["cargo", "test", "--manifest-path", "ramen_observation/Cargo.toml", "--release", "--locked"],
    ]
    for command in commands:
        print("Running: " + " ".join(command), flush=True)
        subprocess.run(command, cwd=ROOT, env=env, check=True)


if __name__ == "__main__":
    main()
