"""Prove observers cannot replace a successful original crypto result on Rust errors."""
from pathlib import Path
import os
import re
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]


class CryptoForwarding(unittest.TestCase):
    def test_real_callbacks_keep_arguments_returns_and_disabled_fast_path(self):
        source_path = Path(os.environ.get('RECOVERY_PROBE_SOURCE', str(ROOT / 'hachimi_ura_plugin/src/lib.rs')))
        if os.environ.get('CI') and os.environ.get('RECOVERY_PROBE_SOURCE'):
            self.fail('CI must test the checked-out candidate')
        source = source_path.read_text(encoding='utf-8')
        callbacks = []
        for name in ('makemd5_hook_handler', 'computehash_hook_handler'):
            match = re.search(r'(?m)^extern "C" fn ' + name + r'\(', source)
            self.assertIsNotNone(match)
            callbacks.append(source[match.start():source.index('\n}', match.start()) + 2])
        code = r'''
#![allow(dead_code)]
use std::ffi::c_void;
use std::sync::{Mutex,atomic::{AtomicUsize,AtomicBool,Ordering}};
static SNIFF_ENABLED:AtomicBool=AtomicBool::new(false);
static MD5_CAPTURE_ENABLED:AtomicBool=AtomicBool::new(false);
static MD5_LOG:Mutex<Vec<(String,String)>>=Mutex::new(Vec::new());
static CALLS:AtomicUsize=AtomicUsize::new(0);
static ARGUMENT:AtomicUsize=AtomicUsize::new(0);
static READS:AtomicUsize=AtomicUsize::new(0);
static FAIL:AtomicBool=AtomicBool::new(false);
unsafe extern "C" fn original(input:*mut c_void)->*mut c_void {
 CALLS.fetch_add(1,Ordering::SeqCst);ARGUMENT.store(input as usize,Ordering::SeqCst);0x222usize as _
}
unsafe fn interceptor_get_trampoline(_:usize)->usize{original as *const () as usize}
unsafe fn read_il2cpp_string(_: *const c_void)->String {
 READS.fetch_add(1,Ordering::SeqCst);if FAIL.load(Ordering::SeqCst){panic!("observer failure");}"observed".into()
}
fn set_hook_status(_: &str,_:&str){}
fn reset(){CALLS.store(0,Ordering::SeqCst);ARGUMENT.store(0,Ordering::SeqCst);READS.store(0,Ordering::SeqCst);}
'''
        code += '\n'.join(callbacks)
        code += r'''
#[test]fn callbacks_preserve_original_even_when_observation_panics(){
 for callback in [makemd5_hook_handler,computehash_hook_handler]{
  for fail in [false,true]{
   reset();SNIFF_ENABLED.store(true,Ordering::SeqCst);FAIL.store(fail,Ordering::SeqCst);
   assert_eq!(callback(0x111usize as _ ) as usize,0x222);
   assert_eq!(CALLS.load(Ordering::SeqCst),1);
   assert_eq!(ARGUMENT.load(Ordering::SeqCst),0x111);
  }
 }
}
#[test]fn disabled_capture_only_forwards_original(){
 SNIFF_ENABLED.store(false,Ordering::SeqCst);MD5_CAPTURE_ENABLED.store(false,Ordering::SeqCst);FAIL.store(true,Ordering::SeqCst);
 for callback in [makemd5_hook_handler,computehash_hook_handler]{
  reset();assert_eq!(callback(0x111usize as _) as usize,0x222);
  assert_eq!(CALLS.load(Ordering::SeqCst),1);assert_eq!(READS.load(Ordering::SeqCst),0);
 }
}
'''
        cache = ROOT / '.recovery-cache/tests'
        cache.mkdir(parents=True, exist_ok=True)
        self.assertTrue(cache.resolve().is_relative_to(ROOT.resolve()))
        with tempfile.TemporaryDirectory(prefix='crypto-forwarding-', dir=cache) as directory:
            path = Path(directory)
            test = path / 'crypto.rs'
            test.write_text(code, encoding='utf-8')
            binary = path / ('crypto.exe' if os.name == 'nt' else 'crypto')
            subprocess.run(['rustc', '--edition=2021', '--test', '-O', str(test), '-o', str(binary)], check=True)
            subprocess.run([str(binary), '--test-threads=1'], cwd=path, check=True)


if __name__ == '__main__':
    unittest.main()
