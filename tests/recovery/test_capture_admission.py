"""Exercise actual native-copy admission functions with readable host object fixtures."""
from pathlib import Path
import os
import re
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]


class CaptureAdmission(unittest.TestCase):
    def test_real_copy_path_reserves_before_reading_and_handles_faults(self):
        source = (ROOT / 'hachimi_ura_plugin/src/lib.rs').read_text(encoding='utf-8')
        functions = []
        for name in ('array_length', 'capture_byte_array', 'read_il2cpp_byte_array', 'read_il2cpp_string'):
            match = re.search(r'(?m)^(?:unsafe )?fn ' + name + r'\(', source)
            self.assertIsNotNone(match)
            functions.append(source[match.start():source.index('\n}', match.start()) + 2])
        code = r'''
#![allow(dead_code)]
use std::ffi::c_void;
use std::sync::{Mutex,atomic::{AtomicU64,AtomicUsize,AtomicBool,Ordering}};
mod observer_limits { pub const CAPTURE_BYTES:usize=1024*1024; }
struct Reservation { bytes:usize }
static LIVE:AtomicUsize=AtomicUsize::new(0);
impl Drop for Reservation {fn drop(&mut self){LIVE.fetch_sub(self.bytes,Ordering::SeqCst);}}
mod observation_queue {pub type Reservation=super::Reservation;}
struct Writer;
impl Writer {fn reserve(&self,bytes:usize)->Result<Reservation,()> {
 LIVE.fetch_update(Ordering::SeqCst,Ordering::SeqCst,|v|v.checked_add(bytes).filter(|n|*n<=observer_limits::CAPTURE_BYTES))
 .map(|_|Reservation{bytes}).map_err(|_|())
}}
fn raw_writer()->Result<&'static Writer,String>{static W:Writer=Writer;Ok(&W)}
struct CapturedBytes{payload:Vec<u8>,reservation:Reservation}
static OBSERVATION_READ_ERRORS:AtomicU64=AtomicU64::new(0);
static BODY_READS:AtomicUsize=AtomicUsize::new(0);
static FAIL_BODY:AtomicBool=AtomicBool::new(false);
static MEMORY:Mutex<Vec<u8>>=Mutex::new(Vec::new());
fn safe_read(addr:u64,dst:&mut[u8])->isize {
 let offset=(addr as usize).wrapping_sub(0x1000);
 if offset>=32 {BODY_READS.fetch_add(1,Ordering::SeqCst);if FAIL_BODY.load(Ordering::SeqCst){return -1;}}
 let memory=MEMORY.lock().unwrap();
 let Some(src)=offset.checked_add(dst.len()).filter(|end|*end<=memory.len()).map(|end|&memory[offset..end]) else{return -1;};
 dst.copy_from_slice(src);dst.len() as isize
}
fn fixture(length:usize,stored:usize){
 let mut memory=vec![0x5a;stored+32];memory[24..32].copy_from_slice(&(length as u64).to_le_bytes());
 *MEMORY.lock().unwrap()=memory;BODY_READS.store(0,Ordering::SeqCst);FAIL_BODY.store(false,Ordering::SeqCst);
}
'''
        code += '\n'.join(functions)
        code += r'''
#[test]fn over_budget_length_never_allocates_or_reads_body(){
 fixture(64*1024*1024,0);
 assert!(unsafe{capture_byte_array(0x1000usize as _)}.is_none());
 assert_eq!(LIVE.load(Ordering::SeqCst),0);assert_eq!(BODY_READS.load(Ordering::SeqCst),0);
 fixture(usize::MAX,0);assert!(unsafe{capture_byte_array(0x1000usize as _)}.is_none());
 assert_eq!(BODY_READS.load(Ordering::SeqCst),0);
}
#[test]fn complete_copy_keeps_admission_until_owner_is_dropped(){
 fixture(128*1024,128*1024);
 let copy=unsafe{capture_byte_array(0x1000usize as _)}.unwrap();
 assert_eq!(copy.payload,vec![0x5a;128*1024]);assert_eq!(LIVE.load(Ordering::SeqCst),192*1024);
 drop(copy);assert_eq!(LIVE.load(Ordering::SeqCst),0);
}
#[test]fn native_read_failure_releases_admission(){
 fixture(1024,1024);FAIL_BODY.store(true,Ordering::SeqCst);
 assert!(unsafe{capture_byte_array(0x1000usize as _)}.is_none());
 assert_eq!(LIVE.load(Ordering::SeqCst),0);assert!(OBSERVATION_READ_ERRORS.load(Ordering::SeqCst)>0);
}
#[test]fn missing_or_unreadable_header_is_an_explicit_capture_error(){
 let before=OBSERVATION_READ_ERRORS.load(Ordering::SeqCst);fixture(1,1);
 assert!(unsafe{capture_byte_array(std::ptr::null())}.is_none());
 assert!(unsafe{capture_byte_array(0x1234usize as _)}.is_none());
 assert_eq!(OBSERVATION_READ_ERRORS.load(Ordering::SeqCst),before+2);
 assert_eq!(LIVE.load(Ordering::SeqCst),0);
}
#[test]fn legacy_byte_reader_refuses_large_body_instead_of_truncating(){
 fixture(128*1024,0);assert!(unsafe{read_il2cpp_byte_array(0x1000usize as _)}.is_empty());
 assert_eq!(BODY_READS.load(Ordering::SeqCst),0);
}
#[test]fn string_header_and_length_are_checked_before_utf16_copy(){
 let mut memory=vec![0u8;24];memory[16..20].copy_from_slice(&2i32.to_le_bytes());memory[20..24].copy_from_slice(&[65,0,66,0]);
 *MEMORY.lock().unwrap()=memory;assert_eq!(unsafe{read_il2cpp_string(0x1000usize as _ )},"AB");
 MEMORY.lock().unwrap()[16..20].copy_from_slice(&i32::MAX.to_le_bytes());
 assert!(unsafe{read_il2cpp_string(0x1000usize as _ )}.is_empty());
 assert!(unsafe{read_il2cpp_string(0x1234usize as _ )}.is_empty());
}
'''
        cache = ROOT / '.recovery-cache/tests'
        cache.mkdir(parents=True, exist_ok=True)
        self.assertTrue(cache.resolve().is_relative_to(ROOT.resolve()))
        with tempfile.TemporaryDirectory(prefix='capture-admission-', dir=cache) as directory:
            path = Path(directory)
            test = path / 'capture.rs'
            test.write_text(code, encoding='utf-8')
            binary = path / ('capture.exe' if os.name == 'nt' else 'capture')
            subprocess.run(['rustc', '--edition=2021', '--test', '-O', str(test), '-o', str(binary)], check=True)
            subprocess.run([str(binary), '--test-threads=1'], cwd=path, check=True)


if __name__ == '__main__':
    unittest.main()
