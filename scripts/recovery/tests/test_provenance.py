from __future__ import annotations

from pathlib import Path
import re
import struct
import sys
import tempfile
import tomllib
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from common import CACHE, ROOT, file_hashes, git, inside, lf, read_json, sha256, source_files, verify_elf
from reproduce_baseline import extract_python_blocks, reproduce, source_repository


def elf(*, alignment=16384, machine=183, virtual=0, file_length=128):
    data = bytearray(128)
    data[:6] = b"\x7fELF\x02\x01"
    struct.pack_into("<H", data, 18, machine)
    struct.pack_into("<Q", data, 32, 64)
    struct.pack_into("<HH", data, 54, 56, 1)
    struct.pack_into("<IIQQQQQQ", data, 64, 1, 5, 0, virtual, 0, file_length, file_length, alignment)
    return bytes(data)


class ElfTests(unittest.TestCase):
    def test_arm64_16k(self):
        self.assertEqual(verify_elf(elf()), [16384])

    def test_4k_fails(self):
        with self.assertRaisesRegex(ValueError, "not 16 KiB"):
            verify_elf(elf(alignment=4096))

    def test_other_abi_fails(self):
        with self.assertRaisesRegex(ValueError, "not ARM64"):
            verify_elf(elf(machine=62))

    def test_misaligned_virtual_address_fails(self):
        with self.assertRaisesRegex(ValueError, "not 16 KiB"):
            verify_elf(elf(virtual=4096))

    def test_truncated_segment_fails(self):
        with self.assertRaisesRegex(ValueError, "truncated"):
            verify_elf(elf(file_length=129))

    def test_broken_header_fails(self):
        with self.assertRaises(ValueError):
            verify_elf(elf()[:65])


class ProvenanceTests(unittest.TestCase):
    def test_locked_git_inputs(self):
        lock = read_json(ROOT / "recovery/baseline-source-lock.json")
        self.assertEqual(len(lock["inputs"]), 43)
        repository = source_repository(ROOT, lock)
        for name, value in lock["inputs"].items():
            with self.subTest(path=name):
                actual = git(repository, "show", f"{value['revision']}:{value['path']}")
                self.assertEqual(sha256(lf(actual)), value["sha256_lf"])

    def test_baseline_build_lock_integrity(self):
        lock = read_json(ROOT / "recovery/baseline-build-lock.json")
        self.assertEqual(sha256(lf((ROOT / "recovery/baseline-Cargo.lock").read_bytes())), lock["cargo_lock_sha256_lf"])

    def test_baseline_and_candidate_share_identical_registry_dependencies(self):
        baseline = tomllib.loads((ROOT / "recovery/baseline-Cargo.lock").read_text(encoding="utf-8"))["package"]
        candidate = tomllib.loads((ROOT / "hachimi_ura_plugin/Cargo.lock").read_text(encoding="utf-8"))["package"]
        # Candidate may add dependencies. Existing dependency versions and crate
        # bytes must stay identical so the source comparison remains attributable.
        actual = {(p["name"], p["version"], p.get("checksum")) for p in candidate if "source" in p}
        for package in baseline:
            if "source" in package:
                self.assertIn((package["name"], package["version"], package["checksum"]), actual)

    def test_heredoc_can_have_different_command_indentation(self):
        value = "          python3 - <<'PY'\n          a=1\n          PY\n            python3 - <<'PY'\n          b=2\n          PY\n"
        self.assertEqual(extract_python_blocks(value), ["a=1\n", "b=2\n"])

    def test_reproduce_from_git_without_diagnostic_directory(self):
        output = reproduce(build_lock=False)
        lock = read_json(ROOT / "recovery/baseline-source-lock.json")
        self.assertEqual(file_hashes(output, list(lock["generated_files"])), lock["generated_files"])
        with self.assertRaisesRegex(ValueError, "already exists"):
            reproduce(output=output)
        # Keep the generated evidence, matching the no-overwrite/no-delete policy.

    def test_reproduction_rejects_outside_cache(self):
        with self.assertRaisesRegex(ValueError, "inside"):
            reproduce(output=ROOT / "unexpected-source-write")

    def test_no_legacy_workflow_can_run_on_recovery_branch(self):
        count = 0
        for path in (ROOT / ".github/workflows").glob("*.yml"):
            if path.name == "recovery-candidate.yml":
                continue
            jobs = path.read_text(encoding="utf-8").split("\njobs:\n")[1]
            for match in re.finditer(r"^  [A-Za-z0-9_-]+:\s*\n(.*)", jobs, re.M):
                self.assertIn("!startsWith(github.ref_name, 'workbench/so-recovery-')", match.group(1), path)
                self.assertIn("!startsWith(github.head_ref, 'workbench/so-recovery-')", match.group(1), path)
                count += 1
        self.assertEqual(count, 53)

    def test_source_inventory_includes_portable_crates_and_build_scripts(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            for name in ("hachimi_ura_plugin/build.rs", "hachimi_ura_plugin/src/guard.c", "ramen_observation/src/lib.rs", "ramen_observation/Cargo.toml", "ramen_observation/target/no.rs"):
                path = root / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text("test", encoding="utf-8")
            files = source_files(root)
            self.assertEqual(len(files), 4)
            self.assertIn("ramen_observation/src/lib.rs", files)
            self.assertFalse(any("target/" in name for name in files))


if __name__ == "__main__":
    unittest.main()
