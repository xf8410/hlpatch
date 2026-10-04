"""Rebuild the historical v3.28.2 generated source from pinned Git inputs.

Generation is confined to a new .recovery-cache directory. Candidate source is
never overwritten. This reproduces source, not the original release binary.
"""
from __future__ import annotations

import argparse
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import textwrap

from common import CACHE, ROOT, file_hashes, git, inside, lf, read_json, sha256, write_json


def source_repository(preferred: Path, lock: dict) -> Path:
    revisions = {entry["revision"] for entry in lock["inputs"].values()}
    try:
        for revision in revisions:
            git(preferred, "cat-file", "-e", revision + "^{commit}")
        return preferred
    except subprocess.CalledProcessError:
        cached = CACHE / "pinned-upstream.git"
        if not cached.exists():
            subprocess.run(["git", "init", "--bare", str(cached)], check=True, capture_output=True)
        for revision in sorted(revisions):
            git(cached, "fetch", "--no-tags", "--depth=1", lock["repository"], revision)
        return cached


def extract_python_blocks(workflow: str) -> list[str]:
    """Read exact heredoc Python blocks; avoid reimplementing the released guard."""
    pattern = r"^ +python3 - <<'PY'\r?\n(?P<body>.*?)^ +PY[ \t]*$"
    blocks = []
    for match in re.finditer(pattern, workflow, re.M | re.S):
        blocks.append(textwrap.dedent(match.group("body")))
    if len(blocks) != 2:
        raise ValueError(f"Expected the two pinned v3.28.2 generation blocks, got {len(blocks)}")
    return blocks


def reproduce(*, output: Path | None = None, source_repo: Path = ROOT, build_lock: bool = True) -> Path:
    lock = read_json(ROOT / "recovery/baseline-source-lock.json")
    CACHE.mkdir(exist_ok=True)
    if output is None:
        output = Path(tempfile.mkdtemp(prefix="baseline-v3.28.2-", dir=CACHE))
    else:
        output = inside(output, CACHE)
        if output.exists():
            raise ValueError("Output already exists; baseline reproduction never replaces existing files")
        output.mkdir(parents=True)
    source_repo = source_repository(source_repo.resolve(), lock)
    for name, expected in lock["inputs"].items():
        data = lf(git(source_repo, "show", f"{expected['revision']}:{expected['path']}"))
        if sha256(data) != expected["sha256_lf"]:
            raise ValueError(f"Pinned generation input mismatch: {name}")
        target = inside(output / name, output)
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(data)
    log = output / "source-generation.log"
    env = os.environ.copy()
    env["PYTHONUTF8"] = "1"
    env["PYTHONIOENCODING"] = "utf-8"

    def run(script: str | None = None, code: str | None = None) -> None:
        command = [sys.executable, "-X", "utf8"] + (["-c", code] if code is not None else [script])
        with log.open("a", encoding="utf-8") as stream:
            subprocess.run(command, check=True, cwd=output, env=env, stdout=stream, stderr=subprocess.STDOUT)

    # The released generator invokes `python3`. Adapt only interpreter selection
    # on Windows, preserving every source generator and its execution order.
    driver = ("import runpy,subprocess,sys; original=subprocess.run; "
              "subprocess.run=lambda args,**kw: original([sys.executable,*args[1:]],**kw) "
              "if args[0]=='python3' else (_ for _ in ()).throw(RuntimeError('unexpected generator command')); "
              "runpy.run_path('scripts/run_generated_succession_l_cumulative.py',run_name='__main__')")
    run(code=driver)
    workflow = (output / ".github/workflows/release-v3282-crashlog.yml").read_text(encoding="utf-8")
    blocks = extract_python_blocks(workflow)
    run(code=blocks[0])  # 3.27.11 -> 3.27.23, exactly as the release workflow.
    run(code=blocks[1])  # SIGSEGV wrappers.
    first = sha256(lf((output / "hachimi_ura_plugin/src/lib.rs").read_bytes()))
    run(code=blocks[1])  # The release requires guard idempotence.
    if sha256(lf((output / "hachimi_ura_plugin/src/lib.rs").read_bytes())) != first:
        raise ValueError("Released SIGSEGV guard is not idempotent")
    run("scripts/apply_screen_mirror_frame_a.py")
    run("scripts/apply_crash_log_path_fix.py")
    actual = file_hashes(output, list(lock["generated_files"]))
    if actual != lock["generated_files"]:
        mismatch = [name for name in actual if actual[name] != lock["generated_files"][name]]
        raise ValueError(f"Historical generation differs from the reviewed baseline: {mismatch}")
    provenance = {"kind": "v3.28.2-source-reproduction", "binary_identical_to_published_release": False,
                  "source_lock_sha256": sha256((ROOT / "recovery/baseline-source-lock.json").read_bytes()),
                  "generated_files": actual, "build_adapters": []}
    if build_lock:
        resolved_lock = ROOT / "recovery/baseline-Cargo.lock"
        resolved_meta = read_json(ROOT / "recovery/baseline-build-lock.json")
        data = lf(resolved_lock.read_bytes())
        if sha256(data) != resolved_meta["cargo_lock_sha256_lf"]:
            raise ValueError("Pinned baseline dependency lock checksum differs")
        (output / "hachimi_ura_plugin/Cargo.lock").write_bytes(data)
        manifest = output / "hachimi_ura_plugin/Cargo.toml"
        manifest.write_text(manifest.read_text(encoding="utf-8") + "\n[workspace]\n", encoding="utf-8", newline="\n")
        provenance["build_adapters"] = ["separate empty workspace", "pinned modern dependency lock",
                                          "NDK/API/alignment/debug profile from recovery/toolchains.json"]
        provenance["build_lock"] = resolved_meta
    write_json(output / "baseline-provenance.json", provenance)
    print(f"Baseline source verified: {actual['hachimi_ura_plugin/src/lib.rs']}", flush=True)
    return output


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--source-repo", type=Path, default=ROOT)
    parser.add_argument("--source-only", action="store_true", help="Verify historical generated files without modern build adapters")
    args = parser.parse_args()
    print(reproduce(output=args.output, source_repo=args.source_repo, build_lock=not args.source_only))


if __name__ == "__main__":
    main()
