"""Exercise production page helpers, cache route functions and actual index SQL."""
from pathlib import Path
import os
import re
import sqlite3
import subprocess
import unittest

ROOT = Path(__file__).resolve().parents[2]


def production_function(source, name):
    start = source.index(f"fn {name}(")
    return source[start:source.index("\n}", start) + 2]


class BoundedPagesTests(unittest.TestCase):
    def test_production_cache_routes_and_byte_pages(self):
        cache = ROOT / ".recovery-cache/bounded-pages"
        cache.mkdir(parents=True, exist_ok=True)
        self.assertTrue(cache.resolve().is_relative_to(ROOT.resolve()))
        manifest = cache / "Cargo.toml"
        if not manifest.exists():
            manifest.write_text('[package]\nname="bounded-pages-recovery-tests"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n', encoding="utf-8")
            for dependency in ("serde_json@1.0.151", "sha2@0.10.9"):
                subprocess.run(["cargo", "add", "--offline", "--manifest-path", str(manifest), dependency], check=True, timeout=60)
        source = (ROOT / "hachimi_ura_plugin/src/lib.rs").read_text(encoding="utf-8")
        module = (ROOT / "hachimi_ura_plugin/src/bounded_pages.rs").as_posix()
        harness = f'#[path="{module}"]\npub mod bounded_pages;\n'
        harness += '''
use std::sync::Mutex;
static MD5_LOG:Mutex<Vec<(String,String)>>=Mutex::new(Vec::new());
static EVENT_OBSERVATIONS:Mutex<Vec<String>>=Mutex::new(Vec::new());
fn k_json_error(error:&str)->String {serde_json::json!({"ok":false,"error":error}).to_string()}
fn parse_query_pairs(uri:&str)->Result<Vec<(String,String)>,String> {
 Ok(uri.split_once('?').map(|(_,query)|query.split('&').map(|pair|{
 let (key,value)=pair.split_once('=').unwrap_or((pair,""));(key.to_string(),value.to_string())}).collect()).unwrap_or_default())
}
'''
        harness += production_function(source, "cached_md5_page") + "\n"
        harness += production_function(source, "cached_event_page") + "\n"
        harness += '''
#[test]fn actual_md5_route_pages_complete_control_characters_and_rejects_stale_windows(){
 *MD5_LOG.lock().unwrap()=(0..100).map(|n|(format!("{}:{}",n,"中\\n\\t\\0".repeat(600)),"out".into())).collect();
 let first=cached_md5_page("/api/md5log");assert!(first.len()<=bounded_pages::MAX_BYTES);
 let page:serde_json::Value=serde_json::from_str(&first).unwrap();assert_eq!(page["ok"],true);assert_eq!(page["has_more"],true);
 assert_eq!(page["entries"][0]["input"],MD5_LOG.lock().unwrap()[0].0);
 let next=format!("/api/md5log?after_index={}&cache_window_id={}",page["next_after_index"],page["cache_window_id"].as_str().unwrap());
 let following:serde_json::Value=serde_json::from_str(&cached_md5_page(&next)).unwrap();
 assert_eq!(following["entries"][0]["cache_index"],page["next_after_index"]);
 MD5_LOG.lock().unwrap().remove(0);
 let stale:serde_json::Value=serde_json::from_str(&cached_md5_page(&next)).unwrap();assert!(stale["error"].as_str().unwrap().contains("stale_cache_window"));
 assert!(MD5_LOG.try_lock().is_ok());
}
#[test]fn actual_event_route_pages_and_reports_invalid_cache_records(){
 *EVENT_OBSERVATIONS.lock().unwrap()=(0..16).map(|id|serde_json::json!({"observation_id":id,"preview":"中".repeat(16000)}).to_string()).collect();
 let text=cached_event_page("/api/event/observations");assert!(text.len()<=bounded_pages::MAX_BYTES);
 let page:serde_json::Value=serde_json::from_str(&text).unwrap();assert_eq!(page["has_more"],true);assert_eq!(page["observations"][0]["preview"],"中".repeat(16000));
 *EVENT_OBSERVATIONS.lock().unwrap()=vec!["{invalid}".into()];
 let invalid:serde_json::Value=serde_json::from_str(&cached_event_page("/api/event/observations")).unwrap();
 assert_eq!(invalid["observations"][0]["representation"],"invalid_cached_record");assert!(EVENT_OBSERVATIONS.try_lock().is_ok());
}
'''
        (cache / "lib.rs").write_text(harness, encoding="utf-8")
        environment = dict(os.environ)
        environment["CARGO_TARGET_DIR"] = str(ROOT / ".recovery-cache/bounded-pages-target")
        subprocess.run(["cargo", "test", "--release", "--offline", "--locked", "--manifest-path", str(manifest), "--", "--test-threads=1"], check=True, timeout=180, env=environment)

    def test_production_storage_sql_has_bounded_rows_and_stable_cursors(self):
        source = (ROOT / "hachimi_ura_plugin/src/lib.rs").read_text(encoding="utf-8")
        files = production_function(source, "storage_files_endpoint")
        sessions = production_function(source, "storage_sessions_endpoint")
        sql_files = re.search(r'"(SELECT file_id,CASE.*?LIMIT \?3)"', files, re.S).group(1)
        sql_sessions = re.search(r'"(SELECT CASE.*?LIMIT \?2)"', sessions, re.S).group(1)
        with sqlite3.connect(":memory:") as database:
            database.execute("CREATE TABLE observation_files(file_id INTEGER PRIMARY KEY,session_id TEXT,relative_path TEXT,content_type TEXT,byte_length INTEGER,sha256 TEXT,created_at_ms INTEGER)")
            database.execute("CREATE TABLE observation_sessions(session_id TEXT PRIMARY KEY,process_id INTEGER,plugin_version TEXT,started_at_ms INTEGER,last_flush_ms INTEGER,state TEXT,recovered_after_restart INTEGER,root_path TEXT)")
            for identity in range(1, 151):
                database.execute("INSERT INTO observation_files VALUES(?,?,?,?,?,?,?)", (identity, "s", f"file-{identity}", "raw", 1, None, 1000 - identity))
                database.execute("INSERT INTO observation_sessions VALUES(?,?,?,?,?,?,?,?)", (f"s{identity:03}", 1, "test", 1000 - identity, 0, "open", 0, "/private"))
            self.assertEqual([row[0] for row in database.execute(sql_files, ("s", 80, 65))], list(range(81, 146)))
            self.assertEqual([row[0] for row in database.execute(sql_sessions, ("s080", 65))], [f"s{i:03}" for i in range(81, 146)])
            database.execute("UPDATE observation_files SET relative_path=? WHERE file_id=81", ("中" * 500,))
            self.assertIsNone(database.execute(sql_files, ("s", 80, 1)).fetchone()[1], "oversized UTF-8 field must not be returned as complete metadata")
            database.execute("UPDATE observation_sessions SET root_path=? WHERE session_id='s081'", ("中" * 500,))
            self.assertIsNone(database.execute(sql_sessions, ("s080", 1)).fetchone()[7])


if __name__ == "__main__":
    unittest.main()
