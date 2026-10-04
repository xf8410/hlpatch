"""Verify the real installation gate and the target-version ABI contract without hooks."""
from pathlib import Path
import os
import re
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]


class HookAbiTests(unittest.TestCase):
    def test_contract_and_production_fail_closed_gate(self):
        source = (ROOT / "hachimi_ura_plugin/src/lib.rs").read_text(encoding="utf-8")
        names = ("managed_hook_id", "managed_hook_preflight", "authorize_hook_installation", "interceptor_hook", "refresh_hook_abi_metadata")
        extracted = []
        for name in names:
            match = re.search(r"(?m)^(?:unsafe )?fn " + name + r"\(", source)
            self.assertIsNotNone(match)
            extracted.append(source[match.start():source.index("\n}", match.start()) + 2])
        managed = ("training_hook_handler", "exec_training_hook", "failure_rate_hook", "event_add_choice_hook_handler",
                   "event_choice_hook_handler", "story_set_hook_handler", "unity_send_hook_handler", "unity_complete_hook_handler",
                   "makemd5_hook_handler", "computehash_hook_handler", "compress_request_hook_handler", "decompress_response_hook_handler", "post_hook_handler")
        native = ("mirror_egl_swap_handler", "sqlite3_open_v2_hook", "sqlite3_open_hook", "sqlite3mc_config_hook",
                  "sqlcipher_key_hook", "sqlcipher_key_v2_hook", "sqlite3_prepare_v2_hook", "sqlite3_exec_hook")
        code = '#![allow(dead_code,static_mut_refs,function_casts_as_integer)]\n'
        for module in ("hook_abi", "hook_registry"):
            path = (ROOT / "hachimi_ura_plugin/src" / (module + ".rs")).as_posix()
            code += f'#[path="{path}"] mod {module};\n'
        code += r'''
use std::ffi::{c_void,c_char};
use std::sync::{Mutex,atomic::{AtomicUsize,AtomicBool,Ordering}};
static NATIVE_CALLS:AtomicUsize=AtomicUsize::new(0);
static MAP_READS:AtomicUsize=AtomicUsize::new(0);
static METADATA_READS:AtomicUsize=AtomicUsize::new(0);
static METADATA_COMPLETE:AtomicBool=AtomicBool::new(true);
static GAME_INITIALIZED:AtomicBool=AtomicBool::new(false);
static LAST_STATUS:Mutex<String>=Mutex::new(String::new());
struct Api{
 interceptor:usize,
 interceptor_hook_fn:Option<unsafe extern "C" fn(usize,*mut c_void,*mut c_void)->*mut c_void>,
 interceptor_get_trampoline_addr_fn:Option<unsafe extern "C" fn(usize,*mut c_void)->*mut c_void>,
 interceptor_unhook_fn:Option<unsafe extern "C" fn(usize,*mut c_void)->*mut c_void>,
 il2cpp_get_assembly_image_fn:Option<unsafe extern "C" fn(*const c_char)->*const c_void>,
}
static mut API:*mut Api=std::ptr::null_mut();
unsafe extern "C" fn native_install(_:usize,_:*mut c_void,_:*mut c_void)->*mut c_void{NATIVE_CALLS.fetch_add(1,Ordering::SeqCst);std::ptr::null_mut()}
unsafe extern "C" fn native_lookup(_:usize,_:*mut c_void)->*mut c_void{NATIVE_CALLS.fetch_add(1,Ordering::SeqCst);std::ptr::null_mut()}
unsafe extern "C" fn assembly(_: *const c_char)->*const c_void{1usize as _}
unsafe extern "C" fn current()->*mut c_void{1usize as _}
unsafe extern "C" fn attach(_: *mut c_void)->*mut c_void{1usize as _}
fn to_cstr(text:&str)->std::ffi::CString{std::ffi::CString::new(text).unwrap()}
unsafe fn resolve_il2cpp_symbol(name:&str)->*mut c_void{
 match name{"il2cpp_domain_get"|"il2cpp_thread_current"=>current as *mut c_void,"il2cpp_thread_attach"=>attach as *mut c_void,_=>std::ptr::null_mut()}
}
unsafe fn collect_hook_abi_metadata(target:hook_abi::HookTarget)->Vec<hook_abi::MethodDescriptor>{
 METADATA_READS.fetch_add(1,Ordering::SeqCst);
 vec![hook_abi::MethodDescriptor::unavailable(target,if METADATA_COMPLETE.load(Ordering::SeqCst){""}else{"test_missing_field"})]
}
fn set_hook_status(_: &str,message:&str){*LAST_STATUS.lock().unwrap()=message.to_string();}
fn hl_parse_maps_readable()->Vec<(usize,usize)>{MAP_READS.fetch_add(1,Ordering::SeqCst);Vec::new()}
fn sniff_timestamp_ms()->u64{123}
fn hook_registry()->&'static hook_registry::HookRegistry{static VALUE:std::sync::OnceLock<hook_registry::HookRegistry>=std::sync::OnceLock::new();VALUE.get_or_init(hook_registry::HookRegistry::new)}
struct HookAbiSnapshot{
 attempted:std::time::Instant,captured_at_ms:u64,state:&'static str,
 methods:Vec<hook_abi::MethodDescriptor>,error:Option<String>,attempts:u32,
}
static HOOK_ABI_METADATA:Mutex<Option<HookAbiSnapshot>>=Mutex::new(None);
static HOOK_ABI_REFRESH:Mutex<()>=Mutex::new(());
fn initialize(){unsafe{API=Box::into_raw(Box::new(Api{interceptor:1,interceptor_hook_fn:Some(native_install),
 interceptor_get_trampoline_addr_fn:Some(native_lookup),interceptor_unhook_fn:Some(native_lookup),il2cpp_get_assembly_image_fn:Some(assembly)}));}}
'''
        for index, name in enumerate(managed + native, 1):
            code += f'#[inline(never)] extern "C" fn {name}(){{std::hint::black_box({index});}}\n'
        code += "\n".join(extracted)
        code += '\n#[test]fn every_managed_hook_is_rejected_before_native_install_or_prologue_read(){initialize();\n'
        for name in managed:
            code += f'unsafe{{assert!(!interceptor_hook(0x1000,{name} as *const () as usize));}}assert_eq!(LAST_STATUS.lock().unwrap().as_str(),"unsupported_native_abi");\n'
        code += r'''
assert_eq!(NATIVE_CALLS.load(Ordering::SeqCst),0);assert_eq!(MAP_READS.load(Ordering::SeqCst),0);
assert!(!managed_hook_preflight(&hook_abi::TARGETS.iter().map(|target|target.id).collect::<Vec<_>>()));
assert_eq!(authorize_hook_installation(0x1000,0xdead),Err("unsupported_native_hook"));
}
#[test]fn metadata_does_not_depend_on_callback_and_success_is_cached(){
 initialize();*HOOK_ABI_METADATA.lock().unwrap()=None;METADATA_READS.store(0,Ordering::SeqCst);
 METADATA_COMPLETE.store(true,Ordering::SeqCst);GAME_INITIALIZED.store(false,Ordering::SeqCst);
 assert!(refresh_hook_abi_metadata());assert!(refresh_hook_abi_metadata());
 assert_eq!(METADATA_READS.load(Ordering::SeqCst),hook_abi::TARGETS.len());
 assert_eq!(NATIVE_CALLS.load(Ordering::SeqCst),0);
}
#[test]fn incomplete_metadata_has_three_bounded_attempts_and_no_native_calls(){
 initialize();*HOOK_ABI_METADATA.lock().unwrap()=None;METADATA_READS.store(0,Ordering::SeqCst);
 METADATA_COMPLETE.store(false,Ordering::SeqCst);
 for attempt in 1..=3{
  assert!(!refresh_hook_abi_metadata());assert!(!refresh_hook_abi_metadata());
  assert_eq!(HOOK_ABI_METADATA.lock().unwrap().as_ref().unwrap().attempts,attempt);
  HOOK_ABI_METADATA.lock().unwrap().as_mut().unwrap().attempted=std::time::Instant::now()-std::time::Duration::from_secs(31);
 }
 assert!(!refresh_hook_abi_metadata());
 assert_eq!(METADATA_READS.load(Ordering::SeqCst),hook_abi::TARGETS.len()*3);
 assert_eq!(NATIVE_CALLS.load(Ordering::SeqCst),0);
}
'''
        cache = ROOT / ".recovery-cache/tests"
        cache.mkdir(parents=True, exist_ok=True)
        self.assertTrue(cache.resolve().is_relative_to(ROOT.resolve()))
        with tempfile.TemporaryDirectory(prefix="hook-abi-", dir=cache) as directory:
            path = Path(directory) / "abi.rs"
            path.write_text(code, encoding="utf-8")
            binary = Path(directory) / ("abi.exe" if os.name == "nt" else "abi")
            subprocess.run(["rustc", "--edition=2021", "--test", "-O", str(path), "-o", str(binary)], check=True)
            subprocess.run([str(binary), "--test-threads=1"], check=True, timeout=30)
