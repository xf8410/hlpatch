"""A leftover device opt-in flag must not replace the fixed diagnostic binary."""
from pathlib import Path
import os
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]


class UpdatePolicy(unittest.TestCase):
    def test_startup_excludes_legacy_upload_and_key_hook_paths(self):
        source = (ROOT / 'hachimi_ura_plugin/src/lib.rs').read_text(encoding='utf-8')
        begin = source.index('pub unsafe extern "C" fn hachimi_init_v3(')
        entrypoint = source[begin:source.index('\n}', begin) + 2]
        for forbidden in ('std::thread::spawn', 'check_and_upload_crash_log(',
                          'upload_file_to_repo(', 'install_sqlcipher_safe_hooks(',
                          'interceptor_hook(', 'ura_sqlcipher_hooks.flag'):
            self.assertNotIn(forbidden, entrypoint)
        self.assertIn('background_upload_disabled', entrypoint)
        self.assertIn('legacy_key_hooks_disabled', entrypoint)

    def test_actual_startup_never_starts_an_updater_thread(self):
        source = (ROOT / 'hachimi_ura_plugin/src/lib.rs').read_text(encoding='utf-8')
        begin = source.index('fn start_auto_update_thread() {')
        function = source[begin:source.index('\n}', begin) + 2]
        code = r'''
#![allow(dead_code,unused_imports)]
extern crate std as real_std;
mod std {
 pub use real_std::{fs,path,time,sync};
 pub mod thread {
  pub fn spawn<F,T>(_:F) where F:FnOnce()->T+Send+'static,T:Send+'static {crate::SPAWNS.fetch_add(1,crate::Ordering::SeqCst);}
  pub fn sleep(_:real_std::time::Duration){}
 }
}
use real_std::sync::{Mutex,atomic::{AtomicUsize,Ordering}};
static AUTO_UPDATE_STATUS:Mutex<Option<String>>=Mutex::new(None);
static SPAWNS:AtomicUsize=AtomicUsize::new(0);
static UPDATES:AtomicUsize=AtomicUsize::new(0);
fn update_so()->String{UPDATES.fetch_add(1,Ordering::SeqCst);"forbidden".into()}
'''
        code += function + r'''
#[test]fn startup_does_not_schedule_legacy_flag_or_network_work(){
 start_auto_update_thread();start_auto_update_thread();
 assert_eq!(SPAWNS.load(Ordering::SeqCst),0);
 assert_eq!(UPDATES.load(Ordering::SeqCst),0);
 assert!(AUTO_UPDATE_STATUS.lock().unwrap().as_ref().unwrap().contains("disabled_recovery_candidate"));
}
'''
        cache = ROOT / '.recovery-cache/tests'
        cache.mkdir(parents=True, exist_ok=True)
        self.assertTrue(cache.resolve().is_relative_to(ROOT.resolve()))
        with tempfile.TemporaryDirectory(prefix='update-policy-', dir=cache) as directory:
            path = Path(directory)
            test = path / 'update.rs'
            test.write_text(code, encoding='utf-8')
            binary = path / ('update.exe' if os.name == 'nt' else 'update')
            subprocess.run(['rustc', '--edition=2021', '--test', '-O', str(test), '-o', str(binary)], check=True)
            subprocess.run([str(binary)], cwd=path, check=True)


if __name__ == '__main__':
    unittest.main()
