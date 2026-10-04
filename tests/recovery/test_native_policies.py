"""Exercise actual candidate functions against a fake host; never install game hooks."""
from pathlib import Path
import os
import re
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]


def function(source, name):
    match = re.search(r'(?m)^(?:unsafe )?fn ' + re.escape(name) + r'\(', source)
    if match is None:
        raise AssertionError(f"Missing production function: {name}")
    return source[match.start():source.index('\n}', match.start()) + 2]


class NativePolicies(unittest.TestCase):
    def test_candidate_does_not_replace_game_text_or_host_signal_handlers(self):
        source = (ROOT / 'hachimi_ura_plugin/src/lib.rs').read_text(encoding='utf-8')
        code = r'''
#![allow(dead_code,static_mut_refs)]
use std::ffi::{c_void,c_char,CString};
use std::sync::atomic::{AtomicUsize,Ordering};
static HOOKS: AtomicUsize=AtomicUsize::new(0);
static SIGNALS: AtomicUsize=AtomicUsize::new(0);
static mut TEXT_COMMON_SET_TEXT_ADDR:usize=0;
struct Api {
 interceptor:usize,
 il2cpp_get_assembly_image_fn:Option<unsafe extern "C" fn(*const c_char)->*const c_void>,
 il2cpp_get_class_fn:Option<unsafe extern "C" fn(*const c_void,*const c_char,*const c_char)->*mut c_void>,
 il2cpp_get_method_addr_fn:Option<unsafe extern "C" fn(usize,*const c_char,i32)->usize>,
}
static mut API:*mut Api=std::ptr::null_mut();
unsafe extern "C" fn assembly(_: *const c_char)->*const c_void{0x1000 as _}
unsafe extern "C" fn class(_: *const c_void,_: *const c_char,_: *const c_char)->*mut c_void{0x2000 as _}
unsafe extern "C" fn method(_:usize,_:*const c_char,_:i32)->usize{0x3000}
fn to_cstr(s:&str)->CString{CString::new(s).unwrap()}
fn set_hook_status(_: &str,_:&str){}
unsafe fn interceptor_hook(_:usize,_:usize)->bool{HOOKS.fetch_add(1,Ordering::SeqCst);true}
extern "C" fn text_common_set_text_hook_handler(_: *mut c_void,_: *const c_void){}
fn hl_crash_log_init(){}
fn hl_crash_log_file()->&'static str{"unused-panic.log"}
extern "C" fn crash_signal_handler(_:i32){}
unsafe fn sys_signal(_:i32,_:usize){SIGNALS.fetch_add(1,Ordering::SeqCst);}
'''
        code += function(source, 'install_text_common_observer_hook') + '\n'
        code += function(source, 'init_crash_handler') + '\n'
        code += r'''
#[test]fn text_target_is_never_replaced_in_diagnostic_candidate(){
 unsafe{
  API=Box::into_raw(Box::new(Api{interceptor:1,il2cpp_get_assembly_image_fn:Some(assembly),
    il2cpp_get_class_fn:Some(class),il2cpp_get_method_addr_fn:Some(method)}));
  install_text_common_observer_hook();install_text_common_observer_hook();
 }
 assert_eq!(HOOKS.load(Ordering::SeqCst),0,"text hook changes the game's setter despite diagnostic exclusion");
}
#[test]fn native_crash_handling_remains_owned_by_host(){
 init_crash_handler();
 assert_eq!(SIGNALS.load(Ordering::SeqCst),0,"plugin overrides native signal handlers");
}
'''
        cache = ROOT / '.recovery-cache/tests'
        cache.mkdir(parents=True, exist_ok=True)
        self.assertTrue(cache.resolve().is_relative_to(ROOT.resolve()))
        with tempfile.TemporaryDirectory(prefix='native-policy-', dir=cache) as directory:
            path = Path(directory)
            test = path / 'policy.rs'
            test.write_text(code, encoding='utf-8')
            binary = path / ('policy.exe' if os.name == 'nt' else 'policy')
            subprocess.run(['rustc', '--edition=2021', '--test', '-O', str(test), '-o', str(binary)], check=True)
            subprocess.run([str(binary), '--test-threads=1'], cwd=path, check=True)


if __name__ == '__main__':
    unittest.main()
