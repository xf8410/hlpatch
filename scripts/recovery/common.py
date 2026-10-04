"""Shared deterministic provenance and native artifact validation."""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import struct
import subprocess

ROOT = Path(__file__).resolve().parents[2]
CACHE = ROOT / ".recovery-cache"
ARTIFACTS = ROOT / ".recovery-artifacts"


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def lf(data: bytes) -> bytes:
    """Canonical UTF-8 text encoding used for source comparison across hosts."""
    data.decode("utf-8")
    return data.replace(b"\r\n", b"\n")


def read_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def write_json(path: Path, value: dict) -> None:
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8", newline="\n")


def git(repo: Path, *args: str) -> bytes:
    return subprocess.check_output(["git", "-C", str(repo), *args], stderr=subprocess.PIPE)


def inside(path: Path, parent: Path) -> Path:
    resolved, boundary = path.resolve(), parent.resolve()
    if resolved == boundary or not resolved.is_relative_to(boundary):
        raise ValueError(f"Path must be inside {boundary.name}: {path}")
    return resolved


def file_hashes(directory: Path, paths: list[str]) -> dict[str, str]:
    return {name: sha256(lf((directory / name).read_bytes())) for name in sorted(paths)}


def source_files(directory: Path) -> list[str]:
    """Capture every crate source/config file, excluding compiler caches."""
    result = []
    roots = [directory / "hachimi_ura_plugin", directory / "ramen_observation"]
    for path in (p for crate in roots if crate.exists() for p in crate.rglob("*")):
        relative = path.relative_to(directory)
        if not path.is_file() or any(p in ("target", ".git") for p in relative.parts):
            continue
        if path.suffix in (".rs", ".c", ".h", ".toml", ".lock") or path.name in ("Cargo.lock", ".gitignore"):
            if path.is_symlink():
                raise ValueError(f"Refusing source symlink: {relative}")
            result.append(relative.as_posix())
    return sorted(result)


def fingerprint(values: dict) -> str:
    return sha256(json.dumps(values, sort_keys=True, separators=(",", ":")).encode())


def verify_elf(data: bytes, name: str = "library") -> list[int]:
    """Require an intact ARM64 ELF64 with every LOAD segment aligned to 16 KiB."""
    if len(data) < 64 or data[:6] != b"\x7fELF\x02\x01":
        raise ValueError(f"{name}: not a little-endian ELF64 library")
    if struct.unpack_from("<H", data, 18)[0] != 183:
        raise ValueError(f"{name}: not ARM64")
    offset = struct.unpack_from("<Q", data, 32)[0]
    size, count = struct.unpack_from("<HH", data, 54)
    if size < 56 or count == 0 or offset + size * count > len(data):
        raise ValueError(f"{name}: invalid program headers")
    alignments = []
    for i in range(count):
        kind, _, file_offset, virtual, _, length, _, alignment = struct.unpack_from("<IIQQQQQQ", data, offset + i * size)
        if kind != 1:
            continue
        if alignment < 16384 or alignment & (alignment - 1) or (virtual - file_offset) % 16384:
            raise ValueError(f"{name}: LOAD segment {i} is not 16 KiB aligned")
        if file_offset + length > len(data):
            raise ValueError(f"{name}: truncated LOAD segment")
        alignments.append(alignment)
    if not alignments:
        raise ValueError(f"{name}: no LOAD segments")
    return alignments


def build_environment() -> dict[str, str]:
    """Initialize only the child build environment; cargo-ndk panic reports expose env."""
    allowed = {"PATH", "PATHEXT", "SYSTEMROOT", "WINDIR", "COMSPEC", "TEMP", "TMP", "HOME",
               "USERPROFILE", "APPDATA", "LOCALAPPDATA", "PROGRAMFILES", "PROGRAMFILES(X86)",
               "PROGRAMW6432", "CARGO_HOME", "RUSTUP_HOME", "ANDROID_HOME", "ANDROID_SDK_ROOT",
               "ANDROID_NDK_HOME", "LIB", "INCLUDE", "NUMBER_OF_PROCESSORS"}
    env = {name: value for name, value in os.environ.items() if name.upper() in allowed}
    if os.name == "nt":
        vswhere = Path(os.environ["ProgramFiles(x86)"]) / "Microsoft Visual Studio/Installer/vswhere.exe"
        install = subprocess.check_output([str(vswhere), "-latest", "-products", "*", "-requires",
                                          "Microsoft.VisualStudio.Component.VC.Tools.x86.x64",
                                          "-property", "installationPath"], text=True).strip()
        if not install:
            raise ValueError("Visual Studio C++ Build Tools are required for host build scripts")
        script = Path(install) / "Common7/Tools/VsDevCmd.bat"
        values = subprocess.check_output(f'"{script}" -no_logo -arch=x64 >nul && set',
                                         shell=True, env=env, text=True, errors="replace")
        for line in values.splitlines():
            name, separator, value = line.partition("=")
            if separator and name and not name.startswith("="):
                env[name.upper()] = value
    return env
