"""Probe checked-out event/training callbacks and failed installation paths."""
from pathlib import Path
import os
import re
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]


class AuxiliaryHooks(unittest.TestCase):
    def test_actual_callbacks_and_installers_preserve_original_calls(self):
        source_path = Path(os.environ.get("RECOVERY_PROBE_SOURCE", str(ROOT / "hachimi_ura_plugin/src/lib.rs")))
        if os.environ.get("CI") and os.environ.get("RECOVERY_PROBE_SOURCE"):
            self.fail("CI must test the checked-out candidate")
        source = source_path.read_text(encoding="utf-8")
        names = ("event_add_choice_hook_handler", "event_choice_hook_handler", "story_set_hook_handler",
                 "failure_rate_hook", "exec_training_hook", "install_event_choice_hook",
                 "install_failure_rate_hook", "install_exec_training_hook")
        functions = []
        for name in names:
            found = re.search(r'(?m)^(?:unsafe |extern "C" )?fn ' + name + r'\(', source)
            self.assertIsNotNone(found, name)
            functions.append(source[found.start():source.index("\n}", found.start()) + 2])
        code = r'''
#![allow(dead_code,static_mut_refs,function_casts_as_integer,unused_unsafe)]
use std::ffi::{c_void,c_char,CString};
use std::sync::{Mutex,atomic::{AtomicUsize,AtomicBool,Ordering}};
static CALLS:AtomicUsize=AtomicUsize::new(0);
static PANIC_OBSERVER:AtomicBool=AtomicBool::new(false);
static ACCEPT_INSTALL:AtomicBool=AtomicBool::new(false);
static INSTALLS:AtomicUsize=AtomicUsize::new(0);
static REJECT_TARGET:AtomicUsize=AtomicUsize::new(0);
static mut LAST_ARGS:[usize;5]=[0;5];
static mut API:*mut u8=1usize as _;
static EVENT_STATE_MUTEX:Mutex<()>=Mutex::new(());
#[derive(Clone)]struct EventChoice{label:String,gain_id:i32,next_block_idx:i32,loop_exit_gain_id:i32}
struct PendingEventSelection{captured_at:u64,generation:u64,story_id:i32,chara_id:i32,selected_idx_raw:i32,choice:Option<EventChoice>}
static EVENT_PENDING_RESULT:Mutex<Option<PendingEventSelection>>=Mutex::new(None);
static mut EVENT_CHOICES:Vec<EventChoice>=Vec::new();
static mut EVENT_SELECTED_IDX:i32=-1;
static mut EVENT_GENERATION:u64=0;
static mut EVENT_STORY_ID:i32=0;
static mut EVENT_CHARA_ID:i32=0;
static EVENT_CHOICES_MAX:usize=16;
static mut EVENT_CHOICE_HOOK_INSTALLED:bool=false;
static mut EVENT_ADD_BTN_ADDR:usize=0;
static mut EVENT_CHOICE_ADDR:usize=0;
static mut STORY_SET_HOOK_INSTALLED:bool=false;
static mut STORY_SET_ADDR:usize=0;
static mut ORIG_EVENT_ADD_BTN_PROLOGUE:[u8;16]=[0;16];
static mut ORIG_EVENT_CHOICE_PROLOGUE:[u8;16]=[0;16];
static mut ORIG_STORY_SET_PROLOGUE:[u8;16]=[0;16];
static mut FAILURE_RATE_HOOK_INSTALLED:bool=false;
static mut FAILURE_RATE_ADDR:usize=0;
static mut ORIG_FAILURE_RATE_PROLOGUE:[u8;16]=[0;16];
static mut LAST_FAILURE_RATE:i32=-1;
static mut EXEC_TRAINING_HOOK_INSTALLED:bool=false;
static mut EXEC_TRAINING_ADDR:usize=0;
static mut ORIG_EXEC_TRAINING_PROLOGUE:[u8;16]=[0;16];
fn to_cstr(s:&str)->CString{CString::new(s).unwrap()}
fn sniff_timestamp()->u64{0}
fn set_hook_status(_: &str,_:&str){}
fn managed_hook_preflight(_: &[&str])->bool{true}
fn ura_log(_:i32,_:&str){if PANIC_OBSERVER.load(Ordering::SeqCst){panic!("observer failure")}}
unsafe fn obj_class(_: *mut c_void)->*mut c_void{if PANIC_OBSERVER.load(Ordering::SeqCst){panic!("observer failure")};1usize as _}
unsafe fn class_name_of(_: *mut c_void)->String{"StoryChoiceParam".into()}
unsafe fn read_il2cpp_string_from_obj(_: *mut c_void,_:&str)->String{"label".into()}
unsafe fn call_fuzzy_string(_: *mut c_void,_:*mut c_void,_:&str)->String{"label".into()}
unsafe fn call_getter_int_raw(_: *mut c_void,_:&str)->i32{1}
unsafe fn call_fuzzy_int(_: *mut c_void,_:*mut c_void,_:&str,_:&[&str])->i32{1}
unsafe fn get_image()->*const c_void{1usize as _}
unsafe fn find_class(_: *const c_void,_:*const c_char,_:*const c_char)->*mut c_void{1usize as _}
unsafe fn find_class_fuzzy(_: *const c_void,_:&str)->*mut c_void{1usize as _}
unsafe fn find_class_by_short_name(_: *const c_void,_:&str)->*mut c_void{1usize as _}
unsafe fn find_method_addr(_: *mut c_void,name:&str,_:i32)->usize{
 match name{"AddChoiceButton"=>100,"Choice"=>200,"SetStory"=>300,_=>400}
}
unsafe fn find_method_fuzzy(_: *mut c_void,_:&str)->usize{100}
unsafe fn install_hook_safe(_: &str,target:usize,_:usize,_:&mut [u8;16])->bool{
 INSTALLS.fetch_add(1,Ordering::SeqCst);ACCEPT_INSTALL.load(Ordering::SeqCst) && target!=REJECT_TARGET.load(Ordering::SeqCst)
}
unsafe extern "C" fn original_add(a:*mut c_void,b:*mut c_void){CALLS.fetch_add(1,Ordering::SeqCst);LAST_ARGS=[a as usize,b as usize,0,0,0];}
unsafe extern "C" fn original_choice(a:*mut c_void,b:i32,c:*mut c_void){CALLS.fetch_add(1,Ordering::SeqCst);LAST_ARGS=[a as usize,b as usize,c as usize,0,0];}
unsafe extern "C" fn original_story(a:*mut c_void,b:i32,c:i64,d:i64,e:i64){CALLS.fetch_add(1,Ordering::SeqCst);LAST_ARGS=[a as usize,b as usize,c as usize,d as usize,e as usize];}
unsafe extern "C" fn original_failure(a:*mut c_void,b:*mut c_void)->i32{original_add(a,b);5432}
unsafe fn interceptor_get_trampoline(handler:usize)->usize{
 if handler==event_add_choice_hook_handler as *const () as usize || handler==exec_training_hook as *const () as usize{original_add as *const () as usize}
 else if handler==event_choice_hook_handler as *const () as usize{original_choice as *const () as usize}
 else if handler==story_set_hook_handler as *const () as usize{original_story as *const () as usize}
 else{original_failure as *const () as usize}
}
fn reset(){CALLS.store(0,Ordering::SeqCst);PANIC_OBSERVER.store(false,Ordering::SeqCst);}
'''
        code += "\n".join(functions)
        code += r'''
#[test]fn null_event_argument_still_forwards_original(){
 reset();unsafe{EVENT_CHOICE_HOOK_INSTALLED=false;EVENT_ADD_BTN_ADDR=0;}
 event_add_choice_hook_handler(11usize as _,std::ptr::null_mut());
 assert_eq!(CALLS.load(Ordering::SeqCst),1);unsafe{assert_eq!(LAST_ARGS,[11,0,0,0,0]);}
}
#[test]fn observer_panics_and_unpublished_flags_never_swallow_event_calls(){
 for fail in [false,true]{
  reset();PANIC_OBSERVER.store(fail,Ordering::SeqCst);
  unsafe{EVENT_CHOICE_HOOK_INSTALLED=false;EVENT_ADD_BTN_ADDR=0;STORY_SET_HOOK_INSTALLED=false;STORY_SET_ADDR=0;}
  event_add_choice_hook_handler(11usize as _,22usize as _);
  assert_eq!(CALLS.load(Ordering::SeqCst),1);
  reset();PANIC_OBSERVER.store(fail,Ordering::SeqCst);
  story_set_hook_handler(11usize as _,22,33,44,55);
  assert_eq!(CALLS.load(Ordering::SeqCst),1);unsafe{assert_eq!(LAST_ARGS,[11,22,33,44,55]);}
  reset();PANIC_OBSERVER.store(fail,Ordering::SeqCst);
  event_choice_hook_handler(11usize as _,22,33usize as _);
  assert_eq!(CALLS.load(Ordering::SeqCst),1);unsafe{assert_eq!(LAST_ARGS,[11,22,33,0,0]);}
 }
 reset();
}
#[test]fn training_callbacks_preserve_arguments_and_return(){
 reset();exec_training_hook(11usize as _,22usize as _);
 assert_eq!(CALLS.load(Ordering::SeqCst),1);unsafe{assert_eq!(LAST_ARGS,[11,22,0,0,0]);}
 reset();assert_eq!(failure_rate_hook(11usize as _,22usize as _),5432);
 assert_eq!(CALLS.load(Ordering::SeqCst),1);unsafe{assert_eq!(LAST_FAILURE_RATE,5432);}
}
#[test]fn rejected_installation_never_publishes_success_and_is_retryable(){
 reset();ACCEPT_INSTALL.store(false,Ordering::SeqCst);INSTALLS.store(0,Ordering::SeqCst);REJECT_TARGET.store(0,Ordering::SeqCst);
 unsafe{
  EVENT_CHOICE_HOOK_INSTALLED=false;EVENT_ADD_BTN_ADDR=0;EVENT_CHOICE_ADDR=0;STORY_SET_HOOK_INSTALLED=false;STORY_SET_ADDR=0;
  FAILURE_RATE_HOOK_INSTALLED=false;FAILURE_RATE_ADDR=0;EXEC_TRAINING_HOOK_INSTALLED=false;EXEC_TRAINING_ADDR=0;
  install_event_choice_hook();install_failure_rate_hook();install_exec_training_hook();
  assert!(!EVENT_CHOICE_HOOK_INSTALLED && !STORY_SET_HOOK_INSTALLED && !FAILURE_RATE_HOOK_INSTALLED && !EXEC_TRAINING_HOOK_INSTALLED);
  assert_eq!((EVENT_ADD_BTN_ADDR,EVENT_CHOICE_ADDR,STORY_SET_ADDR,FAILURE_RATE_ADDR,EXEC_TRAINING_ADDR),(0,0,0,0,0));
  assert_eq!(INSTALLS.load(Ordering::SeqCst),5);
  ACCEPT_INSTALL.store(true,Ordering::SeqCst);
  install_event_choice_hook();install_failure_rate_hook();install_exec_training_hook();
  assert!(EVENT_CHOICE_HOOK_INSTALLED && STORY_SET_HOOK_INSTALLED && FAILURE_RATE_HOOK_INSTALLED && EXEC_TRAINING_HOOK_INSTALLED);
  assert_eq!(INSTALLS.load(Ordering::SeqCst),10);
 }
}
#[test]fn partial_event_installation_retries_only_the_missing_hook(){
 reset();ACCEPT_INSTALL.store(true,Ordering::SeqCst);INSTALLS.store(0,Ordering::SeqCst);REJECT_TARGET.store(200,Ordering::SeqCst);
 unsafe{
  EVENT_CHOICE_HOOK_INSTALLED=false;EVENT_ADD_BTN_ADDR=0;EVENT_CHOICE_ADDR=0;STORY_SET_HOOK_INSTALLED=false;STORY_SET_ADDR=0;
  install_event_choice_hook();
  assert!(!EVENT_CHOICE_HOOK_INSTALLED);assert_eq!((EVENT_ADD_BTN_ADDR,EVENT_CHOICE_ADDR,STORY_SET_ADDR),(100,0,300));
  assert_eq!(INSTALLS.load(Ordering::SeqCst),3);
  REJECT_TARGET.store(0,Ordering::SeqCst);install_event_choice_hook();
  assert!(EVENT_CHOICE_HOOK_INSTALLED);assert_eq!(INSTALLS.load(Ordering::SeqCst),4);
 }
}
'''
        cache = ROOT / ".recovery-cache/tests"
        cache.mkdir(parents=True, exist_ok=True)
        self.assertTrue(cache.resolve().is_relative_to(ROOT.resolve()))
        with tempfile.TemporaryDirectory(prefix="auxiliary-hooks-", dir=cache) as directory:
            test = Path(directory) / "auxiliary.rs"
            test.write_text(code, encoding="utf-8")
            binary = Path(directory) / ("auxiliary.exe" if os.name == "nt" else "auxiliary")
            subprocess.run(["rustc", "--edition=2021", "--test", "-O", str(test), "-o", str(binary)], check=True)
            arguments = [str(binary), "--test-threads=1"]
            if os.environ.get("RECOVERY_PROBE_FILTER"):
                if os.environ.get("CI"):
                    self.fail("CI must run all probe cases")
                arguments.append(os.environ["RECOVERY_PROBE_FILTER"])
            subprocess.run(arguments, check=True, timeout=20)


if __name__ == "__main__":
    unittest.main()
