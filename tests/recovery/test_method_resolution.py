"""Probe actual method resolution with fake metadata; never dereference game pointers."""
from pathlib import Path
import os
import re
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]


class MethodResolution(unittest.TestCase):
    def test_unique_name_arity_and_missing_reflection(self):
        source_path = Path(os.environ.get("RECOVERY_PROBE_SOURCE", str(ROOT / "hachimi_ura_plugin/src/lib.rs")))
        if os.environ.get("CI") and os.environ.get("RECOVERY_PROBE_SOURCE"):
            self.fail("CI must test the checked-out candidate")
        source = source_path.read_text(encoding="utf-8")
        start = source.index("unsafe fn find_method_addr(")
        function = source[start:source.index("\n}", start) + 2]
        code = r'''
#![allow(dead_code,static_mut_refs,function_casts_as_integer)]
use std::ffi::{c_void,c_char,CStr,CString};
use std::sync::atomic::{AtomicUsize,AtomicBool,Ordering};
struct Api{il2cpp_get_method_addr_fn:Option<unsafe extern "C" fn(usize,*const c_char,i32)->usize>}
static mut API:*mut Api=std::ptr::null_mut();
static EXACT_CALLS:AtomicUsize=AtomicUsize::new(0);
static REFLECTION:AtomicBool=AtomicBool::new(true);
static mut COUNTS:[u32;2]=[1,2];
static mut METHODS:[usize;2]=[100,200];
unsafe extern "C" fn exact(_:usize,name:*const c_char,count:i32)->usize{
 EXACT_CALLS.fetch_add(1,Ordering::SeqCst);assert_eq!(CStr::from_ptr(name).to_bytes(),b"Target");assert_eq!(count,2);0xabc0
}
unsafe extern "C" fn methods(_: *mut c_void,iter:*mut *mut c_void)->*const c_void{
 let i=*iter as usize;if i>=2{return std::ptr::null();}*iter=(i+1) as _;std::ptr::addr_of!(METHODS[i]) as _
}
unsafe extern "C" fn name(_: *const c_void)->*const c_char{b"Target\0".as_ptr() as _}
unsafe extern "C" fn count(method:*const c_void)->u32{
 let index=if method==std::ptr::addr_of!(METHODS[0]) as _{0}else{1};COUNTS[index]
}
unsafe extern "C" fn guessed(_: *const c_void)->*const c_void{0xbad0usize as _}
unsafe fn resolve_il2cpp_symbol(symbol:&str)->*mut c_void{
 match symbol{
  "il2cpp_class_get_methods"=>methods as *mut c_void,
  "il2cpp_method_get_name"=>name as *mut c_void,
  "il2cpp_method_get_param_count" if REFLECTION.load(Ordering::SeqCst)=>count as *mut c_void,
  "il2cpp_method_get_pointer"=>guessed as *mut c_void,
  _=>std::ptr::null_mut()
 }
}
fn to_cstr(s:&str)->CString{CString::new(s).unwrap()}
fn set_hook_status(_: &str,_:&str){}
fn ura_log(_:i32,_:&str){}
fn reset(){unsafe{API=Box::into_raw(Box::new(Api{il2cpp_get_method_addr_fn:Some(exact)}));COUNTS=[1,2];}REFLECTION.store(true,Ordering::SeqCst);EXACT_CALLS.store(0,Ordering::SeqCst);}
'''
        code += function
        code += r'''
#[test]fn selects_unique_exact_arity_not_first_name(){
 reset();unsafe{assert_eq!(find_method_addr(1usize as _,"Target",2),0xabc0);}assert_eq!(EXACT_CALLS.load(Ordering::SeqCst),1);
}
#[test]fn ambiguous_same_arity_is_rejected_without_guessing(){
 reset();unsafe{COUNTS=[2,2];assert_eq!(find_method_addr(1usize as _,"Target",2),0);}assert_eq!(EXACT_CALLS.load(Ordering::SeqCst),0);
}
#[test]fn missing_reflection_never_uses_methodinfo_offsets(){
 reset();REFLECTION.store(false,Ordering::SeqCst);unsafe{assert_eq!(find_method_addr(1usize as _,"Target",2),0);}assert_eq!(EXACT_CALLS.load(Ordering::SeqCst),0);
}
#[test]fn missing_exact_api_wrong_arity_and_wildcard_are_rejected(){
 reset();unsafe{
  assert_eq!(find_method_addr(1usize as _,"Target",3),0);assert_eq!(find_method_addr(1usize as _,"Target",-1),0);
  (*API).il2cpp_get_method_addr_fn=None;assert_eq!(find_method_addr(1usize as _,"Target",2),0);
 }assert_eq!(EXACT_CALLS.load(Ordering::SeqCst),0);
}
'''
        cache = ROOT / ".recovery-cache/tests"
        cache.mkdir(parents=True, exist_ok=True)
        self.assertTrue(cache.resolve().is_relative_to(ROOT.resolve()))
        with tempfile.TemporaryDirectory(prefix="method-resolution-", dir=cache) as directory:
            test = Path(directory) / "resolution.rs"
            test.write_text(code, encoding="utf-8")
            binary = Path(directory) / ("resolution.exe" if os.name == "nt" else "resolution")
            subprocess.run(["rustc", "--edition=2021", "--test", "-O", str(test), "-o", str(binary)], check=True)
            subprocess.run([str(binary), "--test-threads=1"], check=True, timeout=20)


if __name__ == "__main__":
    unittest.main()
