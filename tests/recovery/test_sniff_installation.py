"""Exercise actual sniff installers through their exact method-resolution seam."""
from pathlib import Path
import os
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]


class SniffInstallation(unittest.TestCase):
    def test_disabled_ambiguous_and_partially_installed_states(self):
        source = (ROOT / "hachimi_ura_plugin/src/lib.rs").read_text(encoding="utf-8")
        functions = []
        for name in ("install_crypto_observers", "install_api_sniff_hooks"):
            start = source.index("unsafe fn " + name + "(")
            functions.append(source[start:source.index("\n}", start) + 2])
        code = r'''
#![allow(dead_code,static_mut_refs,function_casts_as_integer)]
use std::ffi::{c_void,c_char,CStr,CString};
use std::sync::{Mutex,atomic::{AtomicUsize,AtomicBool,Ordering}};
struct Api{
 interceptor:usize,
 il2cpp_get_assembly_image_fn:Option<unsafe extern "C" fn(*const c_char)->*const c_void>,
 il2cpp_get_class_fn:Option<unsafe extern "C" fn(*const c_void,*const c_char,*const c_char)->*mut c_void>,
 il2cpp_get_method_addr_fn:Option<unsafe extern "C" fn(usize,*const c_char,i32)->usize>,
}
static mut API:*mut Api=std::ptr::null_mut();
static SNIFF_INSTALL_MUTEX:Mutex<()>=Mutex::new(());
static SNIFF_ENABLED:AtomicBool=AtomicBool::new(false);
static RESOLVE_OK:AtomicBool=AtomicBool::new(true);
static CLASS_OK:AtomicBool=AtomicBool::new(true);
static HOOKS:AtomicUsize=AtomicUsize::new(0);
static RESOLVES:AtomicUsize=AtomicUsize::new(0);
static REFLECTIONS:AtomicUsize=AtomicUsize::new(0);
static PREFLIGHT_ALLOWED:AtomicBool=AtomicBool::new(true);
static mut COMPRESS_REQUEST_ADDR:usize=0;
static mut DECOMPRESS_RESPONSE_ADDR:usize=0;
static mut POST_ADDR:usize=0;
static mut UNITY_SEND_ADDR:usize=0;
static mut UNITY_COMPLETE_ADDR:usize=0;
static mut MAKEMD5_ADDR:usize=0;
static mut COMPUTEHASH_ADDR:usize=0;
unsafe extern "C" fn assembly(_: *const c_char)->*const c_void{REFLECTIONS.fetch_add(1,Ordering::SeqCst);1usize as _}
unsafe extern "C" fn class(_: *const c_void,_:*const c_char,_:*const c_char)->*mut c_void{
 REFLECTIONS.fetch_add(1,Ordering::SeqCst);
 if CLASS_OK.load(Ordering::SeqCst){1usize as _}else{std::ptr::null_mut()}
}
unsafe extern "C" fn forbidden(_:usize,_:*const c_char,_:i32)->usize{panic!("bypassed unique resolver")}
unsafe fn find_class_fuzzy(_: *const c_void,_:&str)->*mut c_void{panic!("fuzzy class install")}
unsafe fn find_method_fuzzy(_: *mut c_void,_:&str)->usize{panic!("fuzzy method install")}
fn to_cstr(s:&str)->CString{CString::new(s).unwrap()}
fn ura_log(_:i32,_:&str){}
fn set_hook_status(_: &str,_:&str){}
fn managed_hook_preflight(_: &[&str])->bool{PREFLIGHT_ALLOWED.load(Ordering::SeqCst)}
unsafe fn install_text_common_observer_hook(){}
unsafe fn find_method_addr(_: *mut c_void,name:&str,arity:i32)->usize{
 RESOLVES.fetch_add(1,Ordering::SeqCst);
 let (id,expected)=match name{
  "MakeMd5"=>(1,1),"ComputeHash"=>(2,1),"SendWebRequest"=>(3,0),"InvokeCompletionEvent"=>(4,0),
  "CompressRequest"=>(5,1),"DecompressResponse"=>(6,1),"Post"=>(7,3),_=>panic!("unknown specification")
 };
 assert_eq!(arity,expected);
 if RESOLVE_OK.load(Ordering::SeqCst){id}else{0}
}
unsafe fn interceptor_hook(_:usize,_:usize)->bool{HOOKS.fetch_add(1,Ordering::SeqCst);true}
extern "C" fn makemd5_hook_handler(){}
extern "C" fn computehash_hook_handler(){}
extern "C" fn unity_send_hook_handler(){}
extern "C" fn unity_complete_hook_handler(){}
extern "C" fn compress_request_hook_handler(){}
extern "C" fn decompress_response_hook_handler(){}
extern "C" fn post_hook_handler(){}
fn reset(){unsafe{
 API=Box::into_raw(Box::new(Api{interceptor:1,il2cpp_get_assembly_image_fn:Some(assembly),il2cpp_get_class_fn:Some(class),il2cpp_get_method_addr_fn:Some(forbidden)}));
 COMPRESS_REQUEST_ADDR=0;DECOMPRESS_RESPONSE_ADDR=0;POST_ADDR=0;UNITY_SEND_ADDR=0;UNITY_COMPLETE_ADDR=0;MAKEMD5_ADDR=0;COMPUTEHASH_ADDR=0;
 }HOOKS.store(0,Ordering::SeqCst);RESOLVES.store(0,Ordering::SeqCst);RESOLVE_OK.store(true,Ordering::SeqCst);CLASS_OK.store(true,Ordering::SeqCst);
 REFLECTIONS.store(0,Ordering::SeqCst);PREFLIGHT_ALLOWED.store(true,Ordering::SeqCst);
}
'''
        code += "\n".join(functions)
        code += r'''
#[test]fn disabled_capture_never_installs_or_probes_methods(){
 reset();SNIFF_ENABLED.store(false,Ordering::SeqCst);unsafe{install_api_sniff_hooks();}
 assert_eq!(HOOKS.load(Ordering::SeqCst),0);assert_eq!(RESOLVES.load(Ordering::SeqCst),0);
}
#[test]fn absent_abi_admission_rejects_before_any_http_thread_reflection(){
 reset();SNIFF_ENABLED.store(true,Ordering::SeqCst);PREFLIGHT_ALLOWED.store(false,Ordering::SeqCst);
 unsafe{install_api_sniff_hooks();install_crypto_observers();}
 assert_eq!(HOOKS.load(Ordering::SeqCst),0);assert_eq!(RESOLVES.load(Ordering::SeqCst),0);assert_eq!(REFLECTIONS.load(Ordering::SeqCst),0);
}
#[test]fn ambiguous_resolution_and_absent_exact_class_never_fall_back(){
 reset();SNIFF_ENABLED.store(true,Ordering::SeqCst);RESOLVE_OK.store(false,Ordering::SeqCst);
 unsafe{install_api_sniff_hooks();}assert_eq!(HOOKS.load(Ordering::SeqCst),0);assert_eq!(RESOLVES.load(Ordering::SeqCst),7);
 reset();CLASS_OK.store(false,Ordering::SeqCst);unsafe{install_api_sniff_hooks();}assert_eq!(HOOKS.load(Ordering::SeqCst),0);
}
#[test]fn standalone_md5_install_does_not_skip_computehash_or_duplicate_hooks(){
 reset();unsafe{MAKEMD5_ADDR=1;install_crypto_observers();assert_eq!(COMPUTEHASH_ADDR,2);}
 assert_eq!(HOOKS.load(Ordering::SeqCst),1);
 SNIFF_ENABLED.store(true,Ordering::SeqCst);unsafe{install_api_sniff_hooks();install_api_sniff_hooks();}
 assert_eq!(HOOKS.load(Ordering::SeqCst),6);
}
'''
        cache = ROOT / ".recovery-cache/tests"
        cache.mkdir(parents=True, exist_ok=True)
        self.assertTrue(cache.resolve().is_relative_to(ROOT.resolve()))
        with tempfile.TemporaryDirectory(prefix="sniff-installation-", dir=cache) as directory:
            test = Path(directory) / "installation.rs"
            test.write_text(code, encoding="utf-8")
            binary = Path(directory) / ("installation.exe" if os.name == "nt" else "installation")
            subprocess.run(["rustc", "--edition=2021", "--test", "-O", str(test), "-o", str(binary)], check=True)
            subprocess.run([str(binary), "--test-threads=1"], check=True, timeout=20)


if __name__ == "__main__":
    unittest.main()
