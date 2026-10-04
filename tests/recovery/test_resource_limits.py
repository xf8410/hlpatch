"""Run production resource-budget primitives without a game process."""
from pathlib import Path
import os
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]


class ResourceLimits(unittest.TestCase):
    def test_production_limits_and_streaming(self):
        cache = ROOT / '.recovery-cache/tests'
        cache.mkdir(parents=True, exist_ok=True)
        self.assertTrue(cache.resolve().is_relative_to(ROOT.resolve()))
        with tempfile.TemporaryDirectory(prefix='limits-', dir=cache) as directory:
            binary = Path(directory) / ('limits.exe' if os.name == 'nt' else 'limits')
            subprocess.run(['rustc', '--edition=2021', '--test', '-O',
                            str(ROOT / 'hachimi_ura_plugin/src/observer_limits.rs'), '-o', str(binary)], check=True)
            subprocess.run([str(binary)], check=True)


if __name__ == '__main__':
    unittest.main()
