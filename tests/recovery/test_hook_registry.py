"""Run the production hook registry with native adapter spies, never real hooks."""
from pathlib import Path
import os
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]


class HookRegistryTests(unittest.TestCase):
    def test_production_ownership_concurrency_and_verified_rollback(self):
        cache = ROOT / ".recovery-cache/tests"
        cache.mkdir(parents=True, exist_ok=True)
        self.assertTrue(cache.resolve().is_relative_to(ROOT.resolve()))
        with tempfile.TemporaryDirectory(prefix="registry-", dir=cache) as directory:
            binary = Path(directory) / ("registry.exe" if os.name == "nt" else "registry")
            subprocess.run(["rustc", "--edition=2021", "--test", "-O",
                            str(ROOT / "hachimi_ura_plugin/src/hook_registry.rs"), "-o", str(binary)], check=True)
            # A registry mutex accidentally held over the install closure must
            # fail this suite, rather than leaving the CI runner hung forever.
            subprocess.run([str(binary)], check=True, timeout=20)


if __name__ == "__main__":
    unittest.main()
