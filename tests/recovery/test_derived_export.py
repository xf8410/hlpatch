"""Host-test the production bounded indexed export; no game hooks are loaded."""
from pathlib import Path
import os
import re
import sqlite3
import subprocess
import unittest

ROOT = Path(__file__).resolve().parents[2]


class DerivedExportTests(unittest.TestCase):
    def test_production_sql_applies_source_cursor_session_and_limit(self):
        source = (ROOT / "hachimi_ura_plugin/src/lib.rs").read_text(encoding="utf-8")
        function = source.split("fn storage_turn_event_jsons(uri: &str) -> String {", 1)[1].split(
            "// ===== Ordered turn and event JSON export C1 =====", 1)[0]
        sql = re.search(r'"(SELECT file_id,session_id,relative_path,byte_length,sha256,created_at_ms FROM observation_files.*?)"', function, re.S).group(1)
        sql = sql.replace("\\\n", " ")
        ddl = re.search(r"CREATE TABLE IF NOT EXISTS observation_files\(.*?\);", source, re.S).group(0)
        with sqlite3.connect(":memory:") as database:
            database.execute(ddl)
            for identity in range(1, 201):
                session = "other" if identity == 91 else "current"
                relative = f"protocol/response/{identity}/payload.bin" if identity != 92 else "protocol/request/92/payload.bin"
                database.execute("INSERT INTO observation_files(file_id,session_id,relative_path,content_type,byte_length,sha256,created_at_ms) VALUES(?,?,?,?,?,?,?)",
                                 (identity, session, relative, "application/msgpack", 12, "a" * 64, 1000 - identity))
            first = database.execute(sql, ("current", 80, 65)).fetchall()
            self.assertEqual([row[0] for row in first], [i for i in range(81, 148) if i not in (91, 92)])
            self.assertTrue(all(row[1] == "current" for row in first))
            following = database.execute(sql, ("current", first[-1][0], 65)).fetchall()
            self.assertEqual(following[0][0], 148)
            self.assertEqual(len(following), 53)

    def test_index_cursor_io_and_serialized_byte_budgets(self):
        cache = ROOT / ".recovery-cache/derived-export"
        cache.mkdir(parents=True, exist_ok=True)
        self.assertTrue(cache.resolve().is_relative_to(ROOT.resolve()))
        manifest = cache / "Cargo.toml"
        if not manifest.exists():
            manifest.write_text('[package]\nname="derived-export-recovery-tests"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n', encoding="utf-8")
            subprocess.run(["cargo", "add", "--offline", "--manifest-path", str(manifest),
                            "serde_json@1.0.151"], check=True, timeout=60)
        module = (ROOT / "hachimi_ura_plugin/src/derived_export.rs").as_posix()
        (cache / "lib.rs").write_text(f'#[path = "{module}"]\npub mod derived_export;\n', encoding="utf-8")
        environment = dict(os.environ)
        environment["CARGO_TARGET_DIR"] = str(ROOT / ".recovery-cache/derived-export-target")
        subprocess.run(["cargo", "test", "--release", "--offline", "--locked", "--manifest-path", str(manifest),
                        "--", "--nocapture"], check=True, timeout=180, env=environment)


if __name__ == "__main__":
    unittest.main()
