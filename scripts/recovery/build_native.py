"""Build checked-in candidate source, or reproduce a separately pinned v3.28.2 baseline.

No candidate source generators run here. Outputs are retained in unique artifact
directories with symbols, build ID, hashes, inputs and compiler version evidence.
"""
from __future__ import annotations

import argparse
from datetime import datetime, timezone
import os
from pathlib import Path
import platform
import re
import shutil
import subprocess
import sys
import tempfile
import tomllib

from common import (ARTIFACTS, CACHE, ROOT, build_environment, file_hashes, fingerprint,
                    git, lf, read_json, sha256, source_files, verify_elf, write_json)
from reproduce_baseline import reproduce


FLAGS = "-C link-arg=-Wl,-z,max-page-size=16384 -C link-arg=-Wl,-z,common-page-size=16384 -C link-arg=-Wl,--build-id=sha1"


def infrastructure_hashes() -> dict:
    paths = [p.relative_to(ROOT).as_posix() for folder in ("scripts/recovery", "tests/recovery", "recovery")
             for p in (ROOT / folder).rglob("*") if p.is_file() and "__pycache__" not in p.parts]
    paths += ["rust-toolchain.toml", ".github/workflows/recovery-candidate.yml"]
    return file_hashes(ROOT, paths)


def build_id(readelf: Path, library: Path, env: dict) -> str:
    output = subprocess.check_output([str(readelf), "-n", str(library)], env=env, text=True)
    matches = re.findall(r"Build ID: ([0-9a-f]+)", output)
    if len(matches) != 1 or len(matches[0]) != 40:
        raise ValueError(f"Expected one SHA1 ELF build ID in {library.name}")
    return matches[0]


def prepare_environment(ndk: Path, cargo_ndk_dir: Path | None) -> tuple[dict, dict, Path]:
    lock = read_json(ROOT / "recovery/toolchains.json")
    env = build_environment()
    if cargo_ndk_dir:
        env["PATH"] = str(cargo_ndk_dir.resolve()) + os.pathsep + env.get("PATH", "")
    env["RUSTUP_TOOLCHAIN"] = lock["rust"]
    env["ANDROID_NDK_HOME"] = str(ndk.resolve())
    env["RUSTFLAGS"] = FLAGS
    env["CARGO_TERM_COLOR"] = "never"
    actual = {"rust": subprocess.check_output(["rustc", "--version"], env=env, text=True).strip(),
              "cargo": subprocess.check_output(["cargo", "--version"], env=env, text=True).strip(),
              "cargo_ndk": subprocess.check_output(["cargo", "ndk", "--version"], env=env, text=True).strip(),
              "python": platform.python_version(), "host": platform.platform()}
    if actual["rust"].split()[1] != lock["rust"] or actual["cargo_ndk"].split()[-1] != lock["cargo_ndk"]:
        raise ValueError(f"Compiler/cargo-ndk versions differ from recovery/toolchains.json: {actual}")
    if actual["python"] != lock["python"]:
        raise ValueError(f"Python {lock['python']} is required; got {actual['python']}")
    properties = (ndk / "source.properties").read_text(encoding="utf-8")
    revision = re.search(r"^Pkg.Revision\s*=\s*(.+)$", properties, re.M)
    if not revision or revision.group(1).strip() != lock["ndk"]:
        raise ValueError("NDK revision differs from recovery/toolchains.json")
    host = {"Windows": "windows-x86_64", "Linux": "linux-x86_64", "Darwin": "darwin-x86_64"}[platform.system()]
    binary_dir = ndk / "toolchains/llvm/prebuilt" / host / "bin"
    suffix = ".exe" if os.name == "nt" else ""
    actual["ndk"] = revision.group(1).strip()
    actual["clang"] = subprocess.check_output([str(binary_dir / ("clang" + suffix)), "--version"], env=env, text=True).strip()
    return env, actual, binary_dir


def build(*, baseline: bool, ndk: Path, cargo_ndk_dir: Path | None = None) -> Path:
    lock = read_json(ROOT / "recovery/toolchains.json")
    env, versions, binaries = prepare_environment(ndk, cargo_ndk_dir)
    source = reproduce() if baseline else ROOT
    crate = source / "hachimi_ura_plugin"
    mode = "baseline" if baseline else "candidate"
    before = file_hashes(source, source_files(source))
    infrastructure = infrastructure_hashes()
    source_fingerprint = fingerprint(before)
    source_commit = git(ROOT, "rev-parse", "HEAD").decode().strip()
    env["HLPATCH_SOURCE_FINGERPRINT"] = source_fingerprint
    env["HLPATCH_SOURCE_COMMIT"] = source_commit
    version = tomllib.loads((crate / "Cargo.toml").read_text(encoding="utf-8"))["package"]["version"]
    ARTIFACTS.mkdir(exist_ok=True)
    output = Path(tempfile.mkdtemp(prefix=f"{mode}-{version}-{source_fingerprint[:12]}-", dir=ARTIFACTS))
    target = CACHE / "targets" / mode
    env["CARGO_TARGET_DIR"] = str(target)
    command = ["cargo", "ndk", "-t", lock["abi"], "--platform", str(lock["android_api"]),
               "build", "--release", "--locked", "--lib", "--manifest-path", str(crate / "Cargo.toml"),
               "--config", "profile.release.strip=false", "--config", "profile.release.debug=2"]
    print(f"Building {mode} {version}; log: {output / 'build.log'}", flush=True)
    with (output / "build.log").open("w", encoding="utf-8") as stream:
        subprocess.run(command, cwd=crate, env=env, stdout=stream, stderr=subprocess.STDOUT, check=True)
    if before != file_hashes(source, source_files(source)):
        raise ValueError("Source changed during compilation; refusing to attest the artifact")
    if infrastructure != infrastructure_hashes():
        raise ValueError("Build inputs/scripts changed during compilation; rerun after freezing inputs")
    suffix = ".exe" if os.name == "nt" else ""
    compiled = target / lock["target"] / "release/libhachimi_ura.so"
    symbols = output / "symbols" / lock["abi"] / "libhachimi_ura.so"
    symbols.parent.mkdir(parents=True)
    shutil.copy2(compiled, symbols)
    stripped = output / "libhachimi_ura.so"
    subprocess.run([str(binaries / ("llvm-strip" + suffix)), "--strip-unneeded", "-o", str(stripped), str(symbols)], env=env, check=True)
    readelf = binaries / ("llvm-readelf" + suffix)
    identifier = build_id(readelf, symbols, env)
    if build_id(readelf, stripped, env) != identifier:
        raise ValueError("Stripped/unstripped ELF build IDs do not match")
    sections = subprocess.check_output([str(readelf), "-S", str(symbols)], env=env, text=True)
    if ".debug_info" not in sections:
        raise ValueError("Unstripped symbol artifact has no .debug_info")
    libraries = {}
    for path in (stripped, symbols):
        data = path.read_bytes()
        libraries[path.relative_to(output).as_posix()] = {"sha256": sha256(data), "size": len(data),
                                                        "load_alignments": verify_elf(data, path.name)}
    manifest = {"schema_version": 1, "kind": mode, "version": version,
                "created_utc": datetime.now(timezone.utc).isoformat(), "build_id": identifier,
                "source_git_commit": source_commit,
                "source_git_dirty": bool(git(ROOT, "status", "--porcelain", "--untracked-files=normal").strip()),
                "source_fingerprint_sha256": source_fingerprint, "source_files_sha256_lf": before,
                "build_inputs_sha256_lf": infrastructure, "toolchain_lock": lock, "actual_tools": versions,
                "cargo_command": command, "rustflags": FLAGS, "artifacts": libraries,
                "device_validation": "not_performed", "binary_identical_to_published_v3_28_2": False}
    if baseline:
        manifest["baseline_provenance"] = read_json(source / "baseline-provenance.json")
    write_json(output / "BUILD-MANIFEST.json", manifest)
    sums = {name: value["sha256"] for name, value in libraries.items()}
    sums["BUILD-MANIFEST.json"] = sha256((output / "BUILD-MANIFEST.json").read_bytes())
    (output / "SHA256SUMS").write_text("".join(f"{value}  {name}\n" for name, value in sorted(sums.items())), encoding="utf-8", newline="\n")
    print(f"Verified ARM64 16 KiB ELF; Build ID {identifier}\nArtifacts: {output}", flush=True)
    return output


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--baseline", action="store_true")
    parser.add_argument("--ndk", type=Path, default=Path(os.environ["ANDROID_NDK_HOME"]) if os.environ.get("ANDROID_NDK_HOME") else None)
    parser.add_argument("--cargo-ndk-dir", type=Path)
    args = parser.parse_args()
    if not args.ndk:
        parser.error("Pass --ndk or set ANDROID_NDK_HOME to the pinned NDK directory")
    build(baseline=args.baseline, ndk=args.ndk, cargo_ndk_dir=args.cargo_ndk_dir)


if __name__ == "__main__":
    main()
