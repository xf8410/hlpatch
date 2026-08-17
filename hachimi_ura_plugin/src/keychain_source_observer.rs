use std::collections::VecDeque;
use std::ffi::c_void;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

const MAX_EVENTS: usize = 512;

#[derive(Clone)]
struct KeychainEvent {
    sequence: u64,
    timestamp_ms: u64,
    operation: &'static str,
    object_address: usize,
    value: String,
}

static EVENTS: Mutex<VecDeque<KeychainEvent>> = Mutex::new(VecDeque::new());
static NEXT_SEQUENCE: AtomicU64 = AtomicU64::new(1);
static mut GET_ADDR: usize = 0;
static mut SET_ADDR: usize = 0;
static mut DELETE_ADDR: usize = 0;

fn record(operation: &'static str, object_address: usize, value: String) {
    if !super::SNIFF_ENABLED.load(Ordering::Acquire) {
        return;
    }
    let event = KeychainEvent {
        sequence: NEXT_SEQUENCE.fetch_add(1, Ordering::AcqRel),
        timestamp_ms: super::sniff_timestamp_ms(),
        operation,
        object_address,
        value,
    };
    let mut events = match EVENTS.lock() {
        Ok(value) => value,
        Err(poisoned) => poisoned.into_inner(),
    };
    if events.len() >= MAX_EVENTS {
        events.pop_front();
    }
    events.push_back(event);
}

extern "C" fn get_hook(this: *mut c_void) -> *const c_void {
    unsafe {
        let trampoline = super::interceptor_get_trampoline(get_hook as usize);
        if trampoline == 0 {
            return std::ptr::null();
        }
        type FnType = unsafe extern "C" fn(*mut c_void) -> *const c_void;
        let original: FnType = std::mem::transmute(trampoline);
        let result = original(this);
        let value = super::read_il2cpp_string(result);
        record("GetKeyChainViewerId", this as usize, value);
        result
    }
}

extern "C" fn set_hook(value: *const c_void) {
    unsafe {
        let trampoline = super::interceptor_get_trampoline(set_hook as usize);
        if trampoline == 0 {
            return;
        }
        type FnType = unsafe extern "C" fn(*const c_void);
        let original: FnType = std::mem::transmute(trampoline);
        original(value);
        record("SetKeyChainViewerId", 0, super::read_il2cpp_string(value));
    }
}

extern "C" fn delete_hook() {
    unsafe {
        let trampoline = super::interceptor_get_trampoline(delete_hook as usize);
        if trampoline == 0 {
            return;
        }
        type FnType = unsafe extern "C" fn();
        let original: FnType = std::mem::transmute(trampoline);
        original();
        record("DeleteKeyChainViewerId", 0, String::new());
    }
}

unsafe fn resolve_method(class: *mut c_void, name: &str, parameter_count: i32) -> usize {
    let api = &*super::API;
    match api.il2cpp_get_method_addr_fn {
        Some(get_method_addr) => get_method_addr(class as usize, super::to_cstr(name).as_ptr(), parameter_count),
        None => 0,
    }
}

pub unsafe fn install() {
    if super::API.is_null() {
        super::set_hook_status("keychain.source", "failed: api_null");
        return;
    }
    let api = &*super::API;
    if api.interceptor == 0 {
        super::set_hook_status("keychain.source", "failed: interceptor_unavailable");
        return;
    }
    let get_image = match api.il2cpp_get_assembly_image_fn {
        Some(value) => value,
        None => {
            super::set_hook_status("keychain.source", "failed: assembly_api_unavailable");
            return;
        }
    };
    let get_class = match api.il2cpp_get_class_fn {
        Some(value) => value,
        None => {
            super::set_hook_status("keychain.source", "failed: class_api_unavailable");
            return;
        }
    };
    let image = get_image(super::to_cstr("umamusume.dll").as_ptr());
    if image.is_null() {
        super::set_hook_status("keychain.source", "failed: image_not_found");
        return;
    }
    let class = get_class(
        image,
        super::to_cstr("Gallop").as_ptr(),
        super::to_cstr("Certification").as_ptr(),
    );
    if class.is_null() {
        super::set_hook_status("keychain.source", "failed: class_not_found");
        return;
    }

    if GET_ADDR == 0 {
        let address = resolve_method(class, "GetKeyChainViewerId", 0);
        if address != 0 && super::interceptor_hook(address, get_hook as usize) {
            GET_ADDR = address;
            super::set_hook_status("keychain.get", &format!("hooked@0x{:x}", address));
        } else {
            super::set_hook_status("keychain.get", "failed: resolve_or_hook");
        }
    }
    if SET_ADDR == 0 {
        let address = resolve_method(class, "SetKeyChainViewerId", 1);
        if address != 0 && super::interceptor_hook(address, set_hook as usize) {
            SET_ADDR = address;
            super::set_hook_status("keychain.set", &format!("hooked@0x{:x}", address));
        } else {
            super::set_hook_status("keychain.set", "failed: resolve_or_hook");
        }
    }
    if DELETE_ADDR == 0 {
        let address = resolve_method(class, "DeleteKeyChainViewerId", 0);
        if address != 0 && super::interceptor_hook(address, delete_hook as usize) {
            DELETE_ADDR = address;
            super::set_hook_status("keychain.delete", &format!("hooked@0x{:x}", address));
        } else {
            super::set_hook_status("keychain.delete", "failed: resolve_or_hook");
        }
    }
}

pub fn endpoint(uri: &str) -> String {
    let pairs = match super::parse_query_pairs(uri) {
        Ok(value) => value,
        Err(error) => return super::k_json_error(&error),
    };
    let after = super::query_pair(&pairs, "after_sequence").parse::<u64>().unwrap_or(0);
    let limit = super::query_pair(&pairs, "limit").parse::<usize>().unwrap_or(200).clamp(1, MAX_EVENTS);
    let events = match EVENTS.lock() {
        Ok(value) => value,
        Err(poisoned) => poisoned.into_inner(),
    };
    let selected = events.iter().filter(|event| event.sequence > after).take(limit).cloned().collect::<Vec<_>>();
    let rows = selected.iter().map(|event| format!(
        r#"{{"sequence":{},"timestamp_ms":{},"operation":"{}","object_address":"0x{:x}","value":"{}"}}"#,
        event.sequence,
        event.timestamp_ms,
        event.operation,
        event.object_address,
        super::json_escape(&event.value),
    )).collect::<Vec<_>>().join(",");
    let next = selected.last().map(|event| event.sequence).unwrap_or(after);
    unsafe {
        format!(
            r#"{{"ok":true,"capture_enabled":{},"hooks":{{"get":"{}","set":"{}","delete":"{}"}},"after_sequence":{},"next_sequence":{},"count":{},"events":[{}]}}"#,
            super::SNIFF_ENABLED.load(Ordering::Acquire),
            if GET_ADDR == 0 { "not_installed" } else { "installed" },
            if SET_ADDR == 0 { "not_installed" } else { "installed" },
            if DELETE_ADDR == 0 { "not_installed" } else { "installed" },
            after,
            next,
            selected.len(),
            rows,
        )
    }
}

pub fn clear_endpoint() -> String {
    let mut events = match EVENTS.lock() {
        Ok(value) => value,
        Err(poisoned) => poisoned.into_inner(),
    };
    let cleared = events.len();
    events.clear();
    format!(r#"{{"ok":true,"cleared":{}}}"#, cleared)
}
