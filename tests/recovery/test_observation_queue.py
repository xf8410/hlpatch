"""Exercise the production queue with bounded byte budgets and synthetic raw files."""
from pathlib import Path
import os
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]


class ObservationQueueTests(unittest.TestCase):
    def test_queue_lifetime_failures_and_complete_chunked_raw(self):
        cache = ROOT / ".recovery-cache/tests"
        cache.mkdir(parents=True, exist_ok=True)
        self.assertTrue(cache.resolve().is_relative_to(ROOT.resolve()))
        with tempfile.TemporaryDirectory(prefix="observation-queue-", dir=cache) as directory:
            binary = Path(directory) / ("observation-queue.exe" if os.name == "nt" else "observation-queue")
            subprocess.run(["rustc", "--edition=2021", "--test", "-O",
                            str(ROOT / "hachimi_ura_plugin/src/observation_queue.rs"), "-o", str(binary)], check=True)
            environment = os.environ.copy()
            environment["OBSERVATION_QUEUE_TEST_DIR"] = directory
            subprocess.run([str(binary)], env=environment, check=True, timeout=45)


if __name__ == "__main__":
    unittest.main()
