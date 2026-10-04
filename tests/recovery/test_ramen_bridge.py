"""Compile the production bridge with portable observations; never load game hooks."""
from pathlib import Path
import os
import subprocess
import unittest

ROOT = Path(__file__).resolve().parents[2]


class RamenBridgeTests(unittest.TestCase):
    def test_single_sampler_and_reliable_receipts(self):
        cache = ROOT / ".recovery-cache/ramen-bridge"
        cache.mkdir(parents=True, exist_ok=True)
        self.assertTrue(cache.resolve().is_relative_to(ROOT.resolve()))
        manifest = cache / "Cargo.toml"
        if not manifest.exists():
            # Harness scaffolding contains no production dependencies until cargo add records them.
            manifest.write_text('[package]\nname="ramen-bridge-recovery-tests"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n', encoding="utf-8")
            subprocess.run(["cargo", "add", "--offline", "--manifest-path", str(manifest),
                            "--path", str(ROOT / "ramen_observation"), "ramen_observation"], check=True, timeout=60)
            subprocess.run(["cargo", "add", "--offline", "--manifest-path", str(manifest),
                            "serde_json@1.0.151"], check=True, timeout=60)
        module = (ROOT / "hachimi_ura_plugin/src/ramen_bridge.rs").as_posix()
        (cache / "lib.rs").write_text(f'#[path = "{module}"]\npub mod ramen_bridge;\n', encoding="utf-8")
        env = dict(os.environ)
        env["CARGO_TARGET_DIR"] = str(ROOT / ".recovery-cache/ramen-bridge-target")
        subprocess.run(["cargo", "test", "--release", "--offline", "--locked", "--manifest-path", str(manifest),
                        "--", "--nocapture"], check=True, timeout=180, env=env)


if __name__ == "__main__":
    unittest.main()
