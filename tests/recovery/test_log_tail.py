"""Exercise the production bounded diagnostic reader against a large saved file."""
from pathlib import Path
import os
import subprocess
import unittest

ROOT = Path(__file__).resolve().parents[2]


class LogTail(unittest.TestCase):
    def test_saved_log_export_is_bounded_and_explicitly_a_preview(self):
        source = (ROOT / 'hachimi_ura_plugin/src/lib.rs').read_text(encoding='utf-8')
        begin = source.index('fn bounded_log_tail(')
        function = source[begin:source.index('\n}', begin) + 2]
        cache = ROOT / '.recovery-cache/log-tail'
        cache.mkdir(parents=True, exist_ok=True)
        self.assertTrue(cache.resolve().is_relative_to(ROOT.resolve()))
        manifest = cache / 'Cargo.toml'
        if not manifest.exists():
            manifest.write_text('[package]\nname="recovery-log-tail-tests"\nversion="0.0.0"\nedition="2021"\n[workspace]\n[lib]\npath="lib.rs"\n', encoding='utf-8')
            subprocess.run(['cargo', 'add', '--offline', '--manifest-path', str(manifest), 'serde_json@1.0.151'], check=True)
        code = 'use std::io::Read;\n' + function + r'''
#[test]fn huge_saved_log_uses_only_tail_and_escapes_controls(){
 use std::io::{Write,Seek};
 let path=std::path::Path::new("large-saved-log.bin");
 let mut file=std::fs::File::create(path).unwrap();file.set_len(64*1024*1024).unwrap();
 file.seek(std::io::SeekFrom::End(-8192)).unwrap();
 let mut bytes=vec![0u8;8192];bytes[8188..].copy_from_slice(&[b'"',b'\n',0xff,0xfe]);
 file.write_all(&bytes).unwrap();drop(file);
 let value=bounded_log_tail(path).unwrap();
 assert_eq!(value["file_bytes"],64*1024*1024u64);assert_eq!(value["truncated"],true);
 assert_eq!(value["offset"],64*1024*1024u64-8192);assert_eq!(value["representation"],"utf8_lossy_tail_preview");
 let encoded=value.to_string();assert!(encoded.len()<64*1024);assert_eq!(serde_json::from_str::<serde_json::Value>(&encoded).unwrap(),value);
 std::fs::remove_file(path).unwrap();
}
#[test]fn missing_log_is_an_error_not_an_empty_success(){assert!(bounded_log_tail(std::path::Path::new("absent-file")).is_err());}
'''
        (cache / 'lib.rs').write_text(code, encoding='utf-8')
        env = dict(os.environ)
        env['CARGO_TARGET_DIR'] = str(ROOT / '.recovery-cache/log-tail-target')
        subprocess.run(['cargo', 'test', '--release', '--offline', '--locked', '--manifest-path', str(manifest)],
                       cwd=cache, env=env, check=True, timeout=180)


if __name__ == '__main__':
    unittest.main()
