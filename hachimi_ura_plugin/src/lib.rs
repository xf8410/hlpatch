//! URA Plugin v3.18.6
//! ★ v3.15.2: AI evaluation — score, training recommendation, rest/outgoing evaluation
//! ★ v3.15.2: Fix read_field_value argument swap bug (field_info,obj was swapped → obj,field_info)
//! ★ v3.10.0: Add /summary endpoint — clean player-friendly JSON for floating window app
//! ★ v3.13.0: Add all training runtime fields to /summary via HomeInfoData path (all scenarios)
//! ★ v3.12.0: Add gui_ui_text_edit_singleline for config input fields (Push Host/Port)
//! ★ v3.8.7: TargetType 3=Guts,4=Power (实测); CommandId→name mapping
//! ★ v3.8.1: Fix crash — safe class name detection via il2cpp_class_get_name
//! ★ v3.7.8: Fix crash from null namespace ptr + expand ParamsIncDecInfoArray (TargetType+Value)
//! ★ ObscuredInt fix: All chara fields use getter methods instead of field reads
//! CY encrypts speed/stamina/etc as ObscuredInt, must call get_Speed()/get_Stamina() etc
//! which return plain Int32 after decryption
//!
//! Data path: WorkDataManager (Singleton) -> get_SingleMode() -> WorkSingleModeData
//!            WorkSingleModeData -> get_Character() -> WorkSingleModeCharaData
//!            WorkSingleModeCharaData -> get_Speed(), get_Stamina(), get_Hp(), etc.
//!
//! Motivation enum: 1=Worst, 2=Bad, 3=Normal, 4=Good, 5=Best
//! ObscuredInt getters: get_SkillPoint() returns ObscuredInt (boxed) - needs special handling

#![allow(dead_code)]

use std::ffi::{c_char, c_void, CString};
use std::ptr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use rusqlite::{Connection, OpenFlags};

#[repr(i32)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum InitResult { Error = 0, Ok = 1 }

struct Api {
    log_fn: Option<unsafe extern "C" fn(i32, *const c_char, *const c_char)>,
    gui_show_notification_fn: Option<unsafe extern "C" fn(*const c_char) -> bool>,
    gui_register_menu_item_fn: Option<unsafe extern "C" fn(*const c_char, Option<extern "C" fn(*mut c_void)>, *mut c_void) -> bool>,
    gui_register_menu_section_fn: Option<unsafe extern "C" fn(Option<extern "C" fn(*mut c_void, *mut c_void)>, *mut c_void) -> bool>,
    hachimi_register_on_game_initialized_fn: Option<unsafe extern "C" fn(Option<extern "C" fn(*mut c_void)>, *mut c_void) -> bool>,
    gui_ui_heading_fn: Option<unsafe extern "C" fn(*mut c_void, *const c_char) -> bool>,
    gui_ui_label_fn: Option<unsafe extern "C" fn(*mut c_void, *const c_char) -> bool>,
    gui_ui_colored_label_fn: Option<unsafe extern "C" fn(*mut c_void, u8, u8, u8, u8, *const c_char) -> bool>,
    gui_ui_separator_fn: Option<unsafe extern "C" fn(*mut c_void) -> bool>,
    gui_ui_text_edit_singleline_fn: Option<unsafe extern "C" fn(*mut c_void, *mut c_char, i32) -> bool>,
    il2cpp_get_assembly_image_fn: Option<unsafe extern "C" fn(*const c_char) -> *const c_void>,
    il2cpp_get_class_fn: Option<unsafe extern "C" fn(*const c_void, *const c_char, *const c_char) -> *mut c_void>,
    il2cpp_get_field_from_name_fn: Option<unsafe extern "C" fn(*mut c_void, *const c_char) -> *mut c_void>,
    il2cpp_get_field_value_fn: Option<unsafe extern "C" fn(*const c_void, *const c_void, *mut c_void)>,
    il2cpp_get_static_field_value_fn: Option<unsafe extern "C" fn(*const c_void, *mut c_void)>,
    il2cpp_resolve_symbol_fn: Option<unsafe extern "C" fn(*const c_char) -> *mut c_void>,
    il2cpp_get_singleton_like_instance_fn: Option<unsafe extern "C" fn(*mut c_void) -> *const c_void>,
    il2cpp_string_chars_fn: Option<unsafe extern "C" fn(*const c_void) -> *mut u16>,
    il2cpp_string_length_fn: Option<unsafe extern "C" fn(*const c_void) -> i32>,
}

static mut API: *const Api = ptr::null();
static GAME_INITIALIZED: AtomicBool = AtomicBool::new(false);
static HTTP_RUNNING: AtomicBool = AtomicBool::new(false);
// ★ Mutex to prevent concurrent read_summary_inner calls from HTTP + push threads
static READ_MUTEX: Mutex<()> = Mutex::new(());

// ★ Push-to-app state (v3.10.0): auto-push /summary to uma-juece when data changes
static mut LAST_PUSH_HASH: u64 = 0;
static PUSH_INTERVAL_SECS: u64 = 1;

// ★ Config (v3.11.0): runtime config updated via POST /config from App
// No file editing needed — App settings page sends config to plugin HTTP endpoint
#[derive(Clone)]
struct PluginConfig {
    push_host: String,      // default: "127.0.0.1"
    push_port: u16,         // default: 18766
    http_port: u16,         // default: 18765
    push_interval_secs: u64, // default: 1
    push_enabled: bool,     // default: true
    http_enabled: bool,     // default: true
}

impl PluginConfig {
    fn defaults() -> Self {
        Self {
            push_host: "127.0.0.1".to_string(),
            push_port: 18766,
            http_port: 18765,
            push_interval_secs: 1,
            push_enabled: true,
            http_enabled: true,
        }
    }

    fn push_addr(&self) -> String {
        format!("{}:{}", self.push_host, self.push_port)
    }

    // Parse JSON config from POST /config body (simple manual parse, no serde)
    fn from_json(data: &str) -> Option<Self> {
        let mut cfg = Self::defaults();
        let mut changed = false;
        // Extract key-value pairs from JSON
        for line in data.lines() {
            let l = line.trim().trim_end_matches(',');
            if l.is_empty() || l == "{" || l == "}" { continue; }
            if let Some((k, v)) = l.split_once(':') {
                let k = k.trim().trim_matches('"');
                let v = v.trim().trim_matches('"');
                match k {
                    "push_host" => { cfg.push_host = v.to_string(); changed = true; }
                    "push_port" => if let Ok(n) = v.parse::<u16>() { cfg.push_port = n; changed = true; }
                    "http_port" => if let Ok(n) = v.parse::<u16>() { cfg.http_port = n; changed = true; }
                    "push_interval_secs" => if let Ok(n) = v.parse::<u64>() { cfg.push_interval_secs = n.max(1); changed = true; }
                    "push_enabled" => { cfg.push_enabled = v == "true"; changed = true; }
                    "http_enabled" => { cfg.http_enabled = v == "true"; changed = true; }
                    _ => {}
                }
            }
        }
        if changed { Some(cfg) } else { None }
    }

    fn to_json(&self) -> String {
        format!(
            r#"{{"push_host":"{}","push_port":{},"http_port":{},"push_interval_secs":{},"push_enabled":{},"http_enabled":{}}}"#,
            self.push_host, self.push_port, self.http_port,
            self.push_interval_secs, self.push_enabled, self.http_enabled
        )
    }
}

static mut PLUGIN_CONFIG: Option<PluginConfig> = None;

// ★ Text edit buffers for GUI config (v3.12.0): persist across frames for egui immediate mode
static mut GUI_HOST_BUF: [u8; 64] = [0u8; 64];  // push_host input buffer
static mut GUI_HOST_BUF_LEN: i32 = 0;
static mut GUI_PORT_BUF: [u8; 8] = [0u8; 8];    // push_port input buffer
static mut GUI_PORT_BUF_LEN: i32 = 0;

unsafe fn get_config() -> &'static PluginConfig {
    if PLUGIN_CONFIG.is_none() {
        PLUGIN_CONFIG = Some(PluginConfig::defaults());
    }
    PLUGIN_CONFIG.as_ref().unwrap()
}

unsafe fn update_config(new_cfg: PluginConfig) {
    PLUGIN_CONFIG = Some(new_cfg);
}

// ★ Training log (v3.7.9): auto-record snapshots from /data and /scenario
const MAX_LOG_ENTRIES: usize = 30;
static mut TRAINING_LOG: Vec<String> = Vec::new();

unsafe fn log_snapshot(source: &str, data: &str) {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let entry = format!(r#"{{"ts":{},"src":"{}","data":{}}}"#, ts, source, data);
    if TRAINING_LOG.len() >= MAX_LOG_ENTRIES {
        TRAINING_LOG.remove(0);
    }
    TRAINING_LOG.push(entry);
}

unsafe fn get_training_log() -> String {
    if TRAINING_LOG.is_empty() {
        return r#"{"entries":0,"log":[]}"#.to_string();
    }
    format!(r#"{{"entries":{},"log":[{}]}}"#, TRAINING_LOG.len(), TRAINING_LOG.join(","))
}

#[derive(Copy, Clone)]
struct CharaCache {
    speed: i32, stamina: i32, power: i32, guts: i32, wiz: i32,
    vital: i32, max_vital: i32, motivation: i32, turn: i32,
    skill_point: i32, scenario_id: i32, fan_count: i32,
    month: i32, half: i32,
    playing_state: i32, is_playing: bool,
    valid: bool,
}

static mut CHARA: CharaCache = CharaCache {
    speed: 0, stamina: 0, power: 0, wiz: 0, guts: 0,
    vital: 0, max_vital: 0, motivation: 0, turn: 0,
    skill_point: 0, scenario_id: 0, fan_count: 0,
    month: 0, half: 0,
    playing_state: 0, is_playing: false,
    valid: false,
};

fn to_cstr(s: &str) -> CString {
    CString::new(s).unwrap_or_else(|_| CString::new("<err>").unwrap())
}

unsafe fn ura_log(level: i32, msg: &str) {
    if API.is_null() { return; }
    if let Some(log_fn) = (*API).log_fn {
        let tag = to_cstr("URA");
        let text = to_cstr(msg);
        log_fn(level, tag.as_ptr(), text.as_ptr());
    }
}

unsafe fn ura_notify(msg: &str) {
    if API.is_null() { return; }
    if let Some(notify_fn) = (*API).gui_show_notification_fn {
        let text = to_cstr(msg);
        notify_fn(text.as_ptr());
    }
}

// ============================================================
// IL2CPP Helpers
// ============================================================

unsafe fn get_image() -> *const c_void {
    if API.is_null() { return ptr::null(); }
    match (*API).il2cpp_get_assembly_image_fn {
        Some(fn_ptr) => {
            let name = to_cstr("umamusume.dll");
            let img = fn_ptr(name.as_ptr());
            if img.is_null() {
                ura_log(1, "get_image: umamusume.dll image = null");
            }
            img
        }
        None => { ura_log(1, "get_image: no get_assembly_image_fn"); ptr::null() }
    }
}

unsafe fn find_class(image: *const c_void, ns: *const c_char, name: *const c_char) -> *mut c_void {
    if image.is_null() || API.is_null() { return ptr::null_mut(); }
    match (*API).il2cpp_get_class_fn {
        Some(fn_ptr) => fn_ptr(image, ns, name),
        None => ptr::null_mut(),
    }
}

unsafe fn find_class_by_short_name(image: *const c_void, class_name: &str) -> *mut c_void {
    let name_c = to_cstr(class_name);
    // Try known namespaces first (fast path)
    let ns_gallop = to_cstr("Gallop");
    let ns_empty = to_cstr("");
    for ns in [ns_gallop.as_ptr(), ns_empty.as_ptr()] {
        let cls = find_class(image, ns, name_c.as_ptr());
        if !cls.is_null() { return cls; }
    }
    // Fallback: iterate all classes to find by name (slow but handles any namespace)
    find_class_by_iteration(image, class_name)
}

/// Slow fallback: iterate all classes in the assembly to find one by name
unsafe fn find_class_by_iteration(image: *const c_void, class_name: &str) -> *mut c_void {
    let get_count_fn = resolve_il2cpp_symbol("il2cpp_image_get_class_count");
    let get_class_fn = resolve_il2cpp_symbol("il2cpp_image_get_class");
    if get_count_fn.is_null() || get_class_fn.is_null() { return ptr::null_mut(); }

    let get_count: FnImageGetClassCount = std::mem::transmute(get_count_fn);
    let get_class: FnImageGetClass = std::mem::transmute(get_class_fn);
    let get_name_fn = resolve_il2cpp_symbol("il2cpp_class_get_name");

    let count = get_count(image);
    for i in 0..count {
        let cls = get_class(image, i);
        if cls.is_null() { continue; }
        if !get_name_fn.is_null() {
            let get_name: FnClassGetName = std::mem::transmute(get_name_fn);
            let name_ptr = get_name(cls);
            if !name_ptr.is_null() {
                let len = (0usize..).find(|&j| *name_ptr.add(j) == 0).unwrap_or(0);
                let bytes = std::slice::from_raw_parts(name_ptr as *const u8, len);
                if bytes == class_name.as_bytes() {
                    return cls;
                }
            }
        }
    }
    ptr::null_mut()
}

/// ★ Get class name directly from an Il2CppClass pointer (no iteration needed)
unsafe fn get_class_name_from_pointer(klass: *mut c_void) -> String {
    if klass.is_null() { return String::new(); }
    let get_name_fn = resolve_il2cpp_symbol("il2cpp_class_get_name");
    if get_name_fn.is_null() { return String::new(); }
    let get_name: FnClassGetName = std::mem::transmute(get_name_fn);
    let name_ptr = get_name(klass);
    if name_ptr.is_null() { return String::new(); }
    std::ffi::CStr::from_ptr(name_ptr).to_string_lossy().into_owned()
}

/// ★ Get class name from an object instance by reading its klass pointer from the object header
/// IL2CPP object layout: offset 0 = Il2CppClass* klass (8 bytes on 64-bit)
unsafe fn get_object_class_name(obj: *const c_void) -> String {
    if obj.is_null() { return String::new(); }
    let klass = std::ptr::read_unaligned::<*mut c_void>(obj as *const *mut c_void);
    get_class_name_from_pointer(klass)
}

unsafe fn get_singleton(class: *mut c_void) -> *const c_void {
    if class.is_null() || API.is_null() { return ptr::null(); }
    match (*API).il2cpp_get_singleton_like_instance_fn {
        Some(fn_ptr) => fn_ptr(class),
        None => ptr::null(),
    }
}

unsafe fn read_field_ptr(obj: *const c_void, class: *mut c_void, field_name: &str) -> *const c_void {
    if obj.is_null() || class.is_null() || API.is_null() { return ptr::null(); }
    let field = match (*API).il2cpp_get_field_from_name_fn {
        Some(fn_ptr) => {
            let name_c = to_cstr(field_name);
            fn_ptr(class, name_c.as_ptr())
        }
        None => return ptr::null(),
    };
    if field.is_null() { return ptr::null(); }
    let mut value: *const c_void = ptr::null();
    match (*API).il2cpp_get_field_value_fn {
        Some(fn_ptr) => fn_ptr(obj as *mut c_void, field, &mut value as *mut *const c_void as *mut c_void),
        None => return ptr::null(),
    }
    value
}


// ★ Read a field value from an object by class + field name (returns *mut c_void)
// Used for reading public fields (not getter properties) like CommandInfoArray
unsafe fn read_field_value(class: *mut c_void, obj: *const c_void, field_name: &str) -> *mut c_void {
    if class.is_null() || obj.is_null() || API.is_null() { return ptr::null_mut(); }
    let field_info = match (*API).il2cpp_get_field_from_name_fn {
        Some(f) => f(class, to_cstr(field_name).as_ptr()),
        None => return ptr::null_mut(),
    };
    if field_info.is_null() { return ptr::null_mut(); }
    let mut value: *mut c_void = ptr::null_mut();
    match (*API).il2cpp_get_field_value_fn {
        Some(f) => f(obj, field_info, &mut value as *mut _ as *mut c_void),
        None => return ptr::null_mut(),
    };
    value
}

// ============================================================
// IL2CPP Runtime API types
// ============================================================

type FnClassGetFields = unsafe extern "C" fn(*mut c_void, *mut *mut c_void) -> *mut Il2CppFieldInfo;
type FnClassGetParent = unsafe extern "C" fn(*mut c_void) -> *mut c_void;
type FnClassGetName = unsafe extern "C" fn(*mut c_void) -> *const c_char;
type FnClassGetMethodFromName = unsafe extern "C" fn(*mut c_void, *const c_char, i32) -> *const c_void;
type FnRuntimeInvoke = unsafe extern "C" fn(*const c_void, *mut c_void, *mut *mut c_void, *mut *mut c_void) -> *mut c_void;
type FnClassGetMethods = unsafe extern "C" fn(*mut c_void, *mut *mut c_void) -> *const c_void;
type FnMethodGetName = unsafe extern "C" fn(*const c_void) -> *const c_char;
type FnImageGetClassCount = unsafe extern "C" fn(*const c_void) -> u32;
type FnImageGetClass = unsafe extern "C" fn(*const c_void, u32) -> *mut c_void;

#[repr(C)]
struct Il2CppFieldInfo {
    name: *const c_char,
    _ty: *const c_void,
    parent: *mut c_void,
    offset: i32,
    _token: u32,
}

unsafe fn resolve_il2cpp_symbol(name: &str) -> *mut c_void {
    if API.is_null() { return ptr::null_mut(); }
    match (*API).il2cpp_resolve_symbol_fn {
        Some(resolve) => {
            let cname = to_cstr(name);
            resolve(cname.as_ptr()) as *mut c_void
        }
        None => ptr::null_mut(),
    }
}

// ============================================================
// Call getter method via il2cpp_runtime_invoke
// ============================================================

unsafe fn call_getter_on_instance(
    class: *mut c_void,
    instance: *const c_void,
    method_name: &str,
) -> *mut c_void {
    if class.is_null() || instance.is_null() {
        return ptr::null_mut();
    }

    let get_method_fn: Option<FnClassGetMethodFromName> = {
        let p = resolve_il2cpp_symbol("il2cpp_class_get_method_from_name");
        if p.is_null() { None } else { Some(std::mem::transmute::<*mut c_void, FnClassGetMethodFromName>(p)) }
    };
    let invoke_fn: Option<FnRuntimeInvoke> = {
        let p = resolve_il2cpp_symbol("il2cpp_runtime_invoke");
        if p.is_null() { None } else { Some(std::mem::transmute::<*mut c_void, FnRuntimeInvoke>(p)) }
    };

    if get_method_fn.is_none() || invoke_fn.is_none() {
        return ptr::null_mut();
    }

    let method_name_c = to_cstr(method_name);
    let method_info = get_method_fn.unwrap()(class, method_name_c.as_ptr(), 0);
    if method_info.is_null() {
        ura_log(4, &format!("call_getter: '{}' not found", method_name));
        return ptr::null_mut();
    }

    let mut exc: *mut c_void = ptr::null_mut();
    let result = invoke_fn.unwrap()(
        method_info,
        instance as *mut c_void,
        ptr::null_mut(),
        &mut exc,
    );

    if !exc.is_null() {
        ura_log(1, &format!("call_getter: '{}' threw exception", method_name));
        return ptr::null_mut();
    }

    result
}

/// Call getter that returns a reference type (class instance)
/// Result is a direct Il2CppObject pointer
unsafe fn call_getter_ref(
    class: *mut c_void,
    instance: *const c_void,
    method_name: &str,
) -> *mut c_void {
    call_getter_on_instance(class, instance, method_name)
}

/// Call getter that returns i32 (value type - gets boxed by il2cpp_runtime_invoke)
/// The boxed value is at result_ptr + 16 (after Il2CppObject header on 64-bit)
unsafe fn call_getter_int(
    class: *mut c_void,
    instance: *const c_void,
    method_name: &str,
) -> i32 {
    if class.is_null() || instance.is_null() { return -1; }

    let result = call_getter_on_instance(class, instance, method_name);
    if result.is_null() { return -1; }

    // Value type (int/enum) is boxed: real value at offset +16
    let val_ptr = result as *const u8;
    let int_val = std::ptr::read_unaligned::<i32>(val_ptr.add(16) as *const i32);
    int_val
}

/// Call getter that returns bool (value type - gets boxed)
unsafe fn call_getter_bool(
    class: *mut c_void,
    instance: *const c_void,
    method_name: &str,
) -> bool {
    call_getter_int(class, instance, method_name) != 0
}

/// ★ ObscuredInt getter: The C# property returns ObscuredInt struct,
/// but il2cpp_runtime_invoke boxes it. We need to call the implicit
/// conversion operator to get a plain int.
/// ObscuredInt has an implicit operator that converts to int.
/// Alternative: ObscuredInt struct has fields we can read directly.
/// ObscuredInt layout (from dump.cs struct, 0x20 bytes on 64-bit):
///   offset 0x10: int currentValue (the decrypted value if no crypto)
///   offset 0x14: int fakeValue
///   offset 0x18: int fakeValueActive  
///   offset 0x1C: byte cryptoKey
/// Actually, the getter method get_SkillPoint() returns ObscuredInt,
/// but the C# property SkillPoint has type ObscuredInt.
/// When il2cpp_runtime_invoke calls it, the result is boxed ObscuredInt.
/// We need to read the ObscuredInt struct fields from the boxed result.
///
/// From dump.cs line 1166804:
/// public struct ObscuredInt : IFormattable, IEquatable`1, IComparable`1
/// It has: implicit operator int, explicit operator int
/// The boxed result will have the ObscuredInt data starting at offset 0x10
///
/// Looking at ObscuredInt implementation (Anti-Cheat Toolkit):
/// struct ObscuredInt {
///     int currentValue;   // offset 0x10 in boxed form (after header)
///     int fakeValue;      // offset 0x14
///     int fakeValueActive; // offset 0x18
///     byte cryptoKey;     // offset 0x1C
/// }
/// currentValue = encrypted_value ^ cryptoKey
/// Decrypted = currentValue ^ cryptoKey
///
/// BUT: When we call get_SkillPoint() via il2cpp_runtime_invoke,
/// the return type is ObscuredInt (value type), so it gets boxed.
/// We read the boxed ObscuredInt fields and decrypt manually.
///
/// HOWEVER: There's a simpler approach! The C# property wrapper
/// actually calls the internal get method which returns ObscuredInt.
/// We can try calling the implicit conversion operator instead.
///
/// Simplest approach: Read ObscuredInt fields from boxed result and decrypt.
unsafe fn call_getter_obscured_int(
    class: *mut c_void,
    instance: *const c_void,
    method_name: &str,
) -> i32 {
    if class.is_null() || instance.is_null() { return -1; }

    let result = call_getter_on_instance(class, instance, method_name);
    if result.is_null() { return -1; }

    // Boxed ObscuredInt struct layout (from dump.cs Anti-Cheat Toolkit):
    // offset 0x10: currentCryptoKey (Int32) — the decryption key
    // offset 0x14: hiddenValue (Int32) — the encrypted value
    // offset 0x18: inited (Boolean)
    // offset 0x1C: fakeValue (Int32)
    // offset 0x20: fakeValueActive (Boolean)
    let base = result as *const u8;

    let current_crypto_key = std::ptr::read_unaligned::<i32>(base.add(0x10) as *const i32);
    let hidden_value = std::ptr::read_unaligned::<i32>(base.add(0x14) as *const i32);

    // Decrypt: hiddenValue ^ currentCryptoKey
    let decrypted = hidden_value ^ current_crypto_key;

    ura_log(4, &format!("ObscuredInt {}: hidden={} key={} decrypted={}", 
        method_name, hidden_value, current_crypto_key, decrypted));

    decrypted
}

// ============================================================
// ★ Read ObscuredInt[] array (for charaEffectIdArray etc)
// ============================================================

unsafe fn read_obscured_int_array(
    class: *mut c_void,
    instance: *const c_void,
    method_name: &str,
) -> Vec<i32> {
    let mut result = Vec::new();
    if class.is_null() || instance.is_null() { return result; }

    let arr_obj = call_getter_on_instance(class, instance, method_name);
    if arr_obj.is_null() { return result; }

    // IL2CPP array layout (64-bit):
    // +0x00: Il2CppObject header (16 bytes)
    // +0x10: bounds ptr (8 bytes, null for 1D)
    // +0x18: max_length (8 bytes on 64-bit)
    // +0x20: data start
    let base = arr_obj as *const u8;
    let length = std::ptr::read_unaligned::<usize>(base.add(0x18) as *const usize);
    if length == 0 || length > 1000 { return result; } // sanity check

    // ObscuredInt struct (unboxed) layout:
    // offset 0x00: currentCryptoKey (Int32)
    // offset 0x04: hiddenValue (Int32)
    // offset 0x08: inited (Boolean, padded to 4)
    // offset 0x0C: fakeValue (Int32)
    // offset 0x10: fakeValueActive (Boolean, padded to 4)
    // struct size = 0x14 (20 bytes), aligned to 4
    let struct_size: usize = 0x14;
    let data_start = base.add(0x20);

    for i in 0..length {
        let elem_base = data_start.add(i * struct_size);
        let crypto_key = std::ptr::read_unaligned::<i32>(elem_base as *const i32);
        let hidden_val = std::ptr::read_unaligned::<i32>(elem_base.add(4) as *const i32);
        let decrypted = hidden_val ^ crypto_key;
        result.push(decrypted);
    }

    ura_log(4, &format!("{}: read {} elements", method_name, result.len()));
    result
}

// ============================================================
// ★ Read reference-type array elements with getter calls
// For expanding EnhanceGroupArray, CommandInfoArray, etc.
// ============================================================

unsafe fn read_array_element_details(
    array_obj: *mut c_void,
    element_class: *mut c_void,
    obscured_getters: &[&str],
    plain_getters: &[&str],
) -> Vec<String> {
    let mut results = Vec::new();
    if array_obj.is_null() || element_class.is_null() { return results; }

    let base = array_obj as *const u8;
    let length = std::ptr::read_unaligned::<usize>(base.add(0x18) as *const usize);
    if length == 0 || length > 100 { return results; }

    for i in 0..length {
        let elem_ptr = std::ptr::read_unaligned::<*mut c_void>(base.add(0x20 + i * 8) as *const *mut c_void);
        if elem_ptr.is_null() {
            results.push(r#"{"_null":true}"#.to_string());
            continue;
        }

        let mut fields = Vec::new();
        for getter in obscured_getters {
            let val = call_getter_obscured_int(element_class, elem_ptr, getter);
            let key = getter.strip_prefix("get_").unwrap_or(*getter);
            fields.push(format!(r#""{}":{}"#, key, val));
        }
        for getter in plain_getters {
            let val = call_getter_int(element_class, elem_ptr, getter);
            let key = getter.strip_prefix("get_").unwrap_or(*getter);
            fields.push(format!(r#""{}":{}"#, key, val));
        }
        results.push(format!(r#"{{{}}}"#, fields.join(",")));
    }

    ura_log(4, &format!("read_array_elements: {} elements, {} getters", length, obscured_getters.len() + plain_getters.len()));
    results
}

// ============================================================
// ★ Try to get scenario-specific object from chara
// Based on scenario_id, try multiple possible getter names
// ============================================================

unsafe fn try_get_scenario_obj(
    chara_class: *mut c_void,
    chara_obj: *const c_void,
    scenario_id: i32,
) -> *mut c_void {
    if chara_class.is_null() || chara_obj.is_null() { return ptr::null_mut(); }

    // Map scenario_id to possible getter names
    // From dump.cs, most scenarios use get_ScenarioXxx(), but URA uses get_WorkScenarioURA()
    let getter_names: &[&str] = match scenario_id {
        1 => &["get_WorkScenarioURA", "get_ScenarioURA", "get_Ura"],
        2 => &["get_TeamRace", "get_ScenarioTeamRace"],
        3 => &["get_ScenarioLive", "get_Live"],
        4 => &["get_WorkScenarioFree", "get_ScenarioFree", "get_Free"],
        5 => &["get_ScenarioVenus", "get_Venus"],
        6 => &["get_ScenarioArc", "get_Arc"],
        7 => &["get_ScenarioSport", "get_Sport"],
        8 => &["get_ScenarioCook", "get_Cook"],
        9 => &["get_ScenarioMecha", "get_Mecha"],
        10 => &["get_ScenarioLegend", "get_Legend"],
        11 => &["get_ScenarioPioneer", "get_Pioneer"],
        12 => &["get_ScenarioOnsen", "get_Onsen"],
        13 => &["get_ScenarioBreeders", "get_WorkScenarioBreeders", "get_Breeders"], // ★ 育马者杯
        14 => &["get_ScenarioRamen", "get_WorkScenarioRamen", "get_Ramen"],          // ★ 拉面杯
        _ => &[],
    };

    for name in getter_names {
        let result = call_getter_ref(chara_class, chara_obj, name);
        if !result.is_null() {
            ura_log(3, &format!("★ Scenario {} getter '{}' found at {:p}", scenario_id, name, result));
            return result;
        }
    }

    ura_log(3, &format!("Scenario {} getter: all attempts failed", scenario_id));
    ptr::null_mut()
}

// ============================================================
// ★ Read scenario detail data (/scenario endpoint)
// ============================================================

unsafe fn read_scenario_detail() -> String {
    if API.is_null() { return r#"{"error":"api_null"}"#.to_string(); }

    let image = match get_image() {
        img if !img.is_null() => img,
        _ => return r#"{"error":"image_null"}"#.to_string(),
    };

    let wdm_class = find_class(image, to_cstr("Gallop").as_ptr(), to_cstr("WorkDataManager").as_ptr());
    if wdm_class.is_null() { return r#"{"error":"no_wdm_class"}"#.to_string(); }

    let wdm_instance = get_singleton(wdm_class);
    if wdm_instance.is_null() { return r#"{"error":"no_wdm_singleton"}"#.to_string(); }

    let sm_data_class = find_class(image, to_cstr("Gallop").as_ptr(), to_cstr("WorkSingleModeData").as_ptr());
    let sm_data_obj = call_getter_ref(wdm_class, wdm_instance, "get_SingleMode");
    if sm_data_obj.is_null() { return r#"{"error":"no_single_mode"}"#.to_string(); }

    let chara_data_class = find_class(image, to_cstr("Gallop").as_ptr(), to_cstr("WorkSingleModeCharaData").as_ptr());
    let chara_obj = call_getter_ref(sm_data_class, sm_data_obj, "get_Character");
    if chara_obj.is_null() { return r#"{"error":"no_chara"}"#.to_string(); }

    let scenario_id = call_getter_int(chara_data_class, chara_obj, "get_ScenarioId");
    let scenario_obj = try_get_scenario_obj(chara_data_class, chara_obj, scenario_id);

    if scenario_obj.is_null() {
        return format!(r#"{{"scenario_id":{},"error":"scenario_obj_null","hint":"getter_name_not_found"}}"#, scenario_id);
    }

    // Try to get DataSet from the scenario object
    // First find the scenario class
    let scenario_class_name = match scenario_id {
        1 => "WorkSingleModeScenarioURA",
        2 => "WorkSingleModeScenarioTeamRace",
        3 => "WorkSingleModeScenarioLive",
        4 => "WorkSingleModeScenarioFree",
        5 => "WorkSingleModeScenarioVenus",
        6 => "WorkSingleModeScenarioArc",
        7 => "WorkSingleModeScenarioSport",
        8 => "WorkSingleModeScenarioCook",
        9 => "WorkSingleModeScenarioMecha",
        10 => "WorkSingleModeScenarioLegend",
        11 => "WorkSingleModeScenarioPioneer",
        12 => "WorkSingleModeScenarioOnsen",
        13 => "WorkSingleModeScenarioBreeders",
        14 => "WorkSingleModeScenarioRamen",
        _ => "Unknown",
    };

    let scenario_class = find_class_by_short_name(image, scenario_class_name);

    let mut result_parts = vec![
        format!(r#""scenario_id":{}"#, scenario_id),
        format!(r#""scenario_class":"{}""#, scenario_class_name),
        format!(r#""scenario_obj":"{:p}""#, scenario_obj),
    ];

    // Try get_DataSet()
    if !scenario_class.is_null() {
        let dataset_obj = call_getter_ref(scenario_class, scenario_obj, "get_DataSet");
        if !dataset_obj.is_null() {
            result_parts.push(format!(r#""dataset_obj":"{:p}""#, dataset_obj));

            // Determine DataSet class name for all known scenarios
            let dataset_class_name = match scenario_id {
                1 => "WorkSingleModeScenarioURADataSet",
                2 => "WorkSingleModeScenarioTeamRaceDataSet",
                3 => "WorkSingleModeScenarioLiveDataSet",
                4 => "WorkSingleModeScenarioFreeDataSet",
                5 => "WorkSingleModeScenarioVenusDataSet",
                6 => "WorkSingleModeScenarioArcDataSet",
                7 => "WorkSingleModeScenarioSportDataSet",
                8 => "WorkSingleModeScenarioCookDataSet",
                9 => "WorkSingleModeScenarioMechaDataSet",
                10 => "WorkSingleModeScenarioLegendDataSet",
                11 => "WorkSingleModeScenarioPioneerDataSet",
                12 => "WorkSingleModeScenarioOnsenDataSet",
                13 => "WorkSingleModeScenarioBreedersDataSet",
                14 => "WorkSingleModeScenarioRamenDataSet",
                _ => "UnknownDataSet",
            };
            let dataset_class = find_class_by_short_name(image, dataset_class_name);
            if !dataset_class.is_null() {
                result_parts.push(format!(r#""dataset_class":"{}""#, dataset_class_name));

                // ★ Read int-type DataSet getters (CY uses ObscuredInt for everything)
                let int_getters = [
                    "get_TeamRank", "get_HavingEnhancePoint", "get_PredictEnhancePoint",
                    "get_BcRaceTrackId", "get_DeckId", "get_TeamSpLevelLimit",
                    "get_TeamUnionProgress",
                ];
                let mut ds_ints = Vec::new();
                for getter in &int_getters {
                    // DataSet getters return ObscuredInt - must use obscured_int decoder
                    let val = call_getter_obscured_int(dataset_class, dataset_obj, getter);
                    if val >= 0 {
                        ds_ints.push(format!(r#""{}":{}"#, getter, val));
                    }
                }
                if !ds_ints.is_empty() {
                    result_parts.push(format!(r#""dataset_values":{{{}}}"#, ds_ints.join(",")));
                }

                // ★ Read array-type DataSet getters (report length + element pointers)
                let array_getters = [
                    "get_EnhanceGroupArray", "get_CommandInfoArray",
                    "get_TeamMemberInfoArray", "get_TeamReviewResultArray",
                    "get_BcRaceResultArray", "get_CommandGainExpArray",
                ];
                let mut ds_arrays = Vec::new();
                for getter in &array_getters {
                    let arr_obj = call_getter_on_instance(dataset_class, dataset_obj, getter);
                    if !arr_obj.is_null() {
                        let base = arr_obj as *const u8;
                        let length = std::ptr::read_unaligned::<usize>(base.add(0x18) as *const usize);
                        ds_arrays.push(format!(r#""{}":{{"len":{},"ptr":"{:p}"}}"#, getter, length, arr_obj));
                    }
                }
                if !ds_arrays.is_empty() {
                    result_parts.push(format!(r#""dataset_arrays":{{{}}}"#, ds_arrays.join(",")));
                }

                // ★ Expand EnhanceGroupArray elements (Breeders buff data)
                // Element class: ObscuredSingleModeBreedersEnhanceGroup
                // Getters: get_GroupType (ObscuredInt), get_Level (ObscuredInt)
                if scenario_id == 13 {
                    let enhance_elem_class = find_class_by_short_name(image, "ObscuredSingleModeBreedersEnhanceGroup");
                    if !enhance_elem_class.is_null() {
                        let enhance_arr = call_getter_on_instance(dataset_class, dataset_obj, "get_EnhanceGroupArray");
                        if !enhance_arr.is_null() {
                            let elements = read_array_element_details(
                                enhance_arr, enhance_elem_class,
                                &["get_GroupType", "get_Level"],
                                &[],
                            );
                            result_parts.push(format!(r#""enhance_groups":[{}]"#, elements.join(",")));
                        }
                    }
                }

                // ★ Expand CommandInfoArray elements (Breeders training commands)
                // Element class: ObscuredSingleModeBreedersCommandInfo
                // Getters: CommandType(ObscuredInt), CommandId(ObscuredInt),
                //          RankUpPredict(ObscuredInt), ParamsIncDecInfoArray, TeamMemberInfoArray
                // ★★ v3.8.0 FIX: ParamsIncDecInfoArray uses SingleModeParamsIncDecInfo (plain Int32),
                //    NOT SingleModeParamsIncDecInfoData (ObscuredInt). The Onsen scenario confirms
                //    Obscured wrappers use plain DTOs, not ObscuredInt-wrapped Data classes.
                if scenario_id == 13 {
                    let cmd_elem_class = find_class_by_short_name(image, "ObscuredSingleModeBreedersCommandInfo");
                    if !cmd_elem_class.is_null() {
                        let cmd_arr = call_getter_on_instance(dataset_class, dataset_obj, "get_CommandInfoArray");
                        if !cmd_arr.is_null() {
                            let elements = read_array_element_details(
                                cmd_arr, cmd_elem_class,
                                &["get_CommandType", "get_CommandId", "get_RankUpPredict"],
                                &[],
                            );
                            // ★ Breeders uses SingleModeParamsIncDecInfo (plain Int32 at 0x10, 0x14)
                            //    Confirmed via Onsen scenario's ObscuredSingleModeOnsenCommandInfo
                            //    which uses SingleModeParamsIncDecInfo[] (not Data variant)
                            //    NO auto-detection needed — hardcode to avoid class lookup crashes

                            let base = cmd_arr as *const u8;
                            let cmd_len = std::ptr::read_unaligned::<usize>(base.add(0x18) as *const usize);
                            let mut cmd_details = Vec::new();
                            for i in 0..cmd_len {
                                let elem_ptr = std::ptr::read_unaligned::<*mut c_void>(base.add(0x20 + i * 8) as *const *mut c_void);
                                let mut detail = if i < elements.len() { elements[i].clone() } else { "{}".to_string() };
                                // ★ Add CommandId→training name mapping
                                {
                                    let cmd_id_val = if detail.contains("\"CommandId\":") {
                                        detail.split("\"CommandId\":").nth(1)
                                            .and_then(|s| s.split(',').next())
                                            .and_then(|s| s.trim().parse::<i32>().ok())
                                            .unwrap_or(-1)
                                    } else { -1 };
                                    let cmd_name = match cmd_id_val {
                                        101 => "Speed", 102 => "Stamina", 103 => "Guts",
                                        105 => "Power", 106 => "Wiz",
                                        601 => "Speed", 602 => "Stamina", 603 => "Guts",
                                        604 => "Power", 605 => "Wiz",
                                        304 => "Kakushimi",
                                        _ => "Unknown"
                                    };
                                    if detail.ends_with('}') { detail.pop(); }
                                    detail.push_str(&format!(",\"CommandName\":\"{}\"}}", cmd_name));
                                }
                                if !elem_ptr.is_null() {
                                    let params_arr = call_getter_on_instance(cmd_elem_class, elem_ptr, "get_ParamsIncDecInfoArray");
                                    let mut params_items = Vec::new();
                                    if !params_arr.is_null() {
                                        let p_base = params_arr as *const u8;
                                        let p_len = std::ptr::read_unaligned::<usize>(p_base.add(0x18) as *const usize);
                                        for j in 0..p_len {
                                            let p_elem = std::ptr::read_unaligned::<*mut c_void>(p_base.add(0x20 + j * 8) as *const *mut c_void);
                                            if p_elem.is_null() { continue; }
                                            // ★ Breeders: always plain Int32 (SingleModeParamsIncDecInfo)
                                            // TargetType 实测映射（与dump.cs ParameterType枚举不同！）：
                                            //   枚举定义3=Power 4=Guts，但target_type字段实际3=Guts 4=Power
                                            //   验证：Stamina训练(TT3)加Guts，Power训练(TT4)加Power
                                            //   0=None, 1=Speed, 2=Stamina, 3=Guts, 4=Power, 5=Wiz
                                            //   10=HP, 20=Motivation, 30=SkillPt
                                            let bytes = p_elem as *const u8;
                                            let t = std::ptr::read_unaligned::<i32>(bytes.add(0x10) as *const i32);
                                            let v = std::ptr::read_unaligned::<i32>(bytes.add(0x14) as *const i32);
                                            let (tt, val) = (t, v);
                                            let tt_name = match tt {
                                                0 => "None", 1 => "Speed", 2 => "Stamina",
                                                3 => "Guts", 4 => "Power", 5 => "Wiz",
                                                6 => "Unknown6", 10 => "HP", 20 => "Motivation",
                                                30 => "SkillPt",
                                                _ => "Unknown"
                                            };
                                            params_items.push(format!(r#"{{"TargetType":{},"TargetTypeName":"{}","Value":{}}}"#, tt, tt_name, val));
                                        }
                                    }
                                    // Read TeamMemberInfoArray length
                                    let member_arr = call_getter_on_instance(cmd_elem_class, elem_ptr, "get_TeamMemberInfoArray");
                                    let member_len = if !member_arr.is_null() {
                                        let mbase = member_arr as *const u8;
                                        std::ptr::read_unaligned::<usize>(mbase.add(0x18) as *const usize)
                                    } else { 0 };
                                    // Trim trailing } and add new fields
                                    if detail.ends_with('}') { detail.pop(); }
                                    detail.push_str(&format!(",\"params_inc_dec\":[{}],\"team_member_len\":{}}}",
                                        params_items.join(","), member_len));
                                }
                                cmd_details.push(detail);
                            }
                            result_parts.push(format!(r#""command_info":[{}]"#, cmd_details.join(",")));
                        }
                    }
                }


                // ★ Ramen scenario (scenario_id == 14) specific data
                if scenario_id == 14 {
                    // Read Ramen-specific ObscuredInt fields
                    let ramen_int_getters = [
                        "get_CheckPointPt", "get_ExpectedCheckPointPt",
                        "get_SpecialFeelingNum", "get_RecommendType",
                    ];
                    let mut ramen_ints = Vec::new();
                    for getter in &ramen_int_getters {
                        let val = call_getter_obscured_int(dataset_class, dataset_obj, getter);
                        if val >= 0 {
                            ramen_ints.push(format!(r#""{}":{}"#, getter, val));
                        }
                    }
                    if !ramen_ints.is_empty() {
                        result_parts.push(format!(r#""ramen_values":{{{}}}"#, ramen_ints.join(",")));
                    }

                    // Read Ramen-specific bool fields
                    let ramen_bool_getters = [
                        "get_IsGaugeGained", "get_IsUrafEffectSelectEventChecked",
                        "get_IsNotGainSpecialFeeling",
                    ];
                    let mut ramen_bools = Vec::new();
                    for getter in &ramen_bool_getters {
                        let val = call_getter_bool(dataset_class, dataset_obj, getter);
                        ramen_bools.push(format!(r#""{}":{}"#, getter, val));
                    }
                    if !ramen_bools.is_empty() {
                        result_parts.push(format!(r#""ramen_bools":{{{}}}"#, ramen_bools.join(",")));
                    }

                    // Read ActiveEffectArray (Ramen current buffs)
                    // Element: ObscuredSingleModeRamenActiveEffectInfo
                    // ObscuredInt getters: EffectCategory, EffectId, EffectValue
                    let ae_class = find_class_by_short_name(image, "ObscuredSingleModeRamenActiveEffectInfo");
                    if !ae_class.is_null() {
                        let ae_arr = call_getter_on_instance(dataset_class, dataset_obj, "get_ActiveEffectArray");
                        if !ae_arr.is_null() {
                            let ae_base = ae_arr as *const u8;
                            let ae_len = std::ptr::read_unaligned::<usize>(ae_base.add(0x18) as *const usize);
                            if ae_len > 0 && ae_len < 100 {
                                let mut effects = Vec::new();
                                for i in 0..ae_len {
                                    let ep = std::ptr::read_unaligned::<*mut c_void>(ae_base.add(0x20 + i * 8) as *const *mut c_void);
                                    if ep.is_null() { continue; }
                                    let cat = call_getter_obscured_int(ae_class, ep, "get_EffectCategory");
                                    let eid = call_getter_obscured_int(ae_class, ep, "get_EffectId");
                                    let val = call_getter_obscured_int(ae_class, ep, "get_EffectValue");
                                    effects.push(format!(
                                        r#"{{"EffectCategory":{},"EffectId":{},"EffectValue":{}}}"#,
                                        cat, eid, val
                                    ));
                                }
                                result_parts.push(format!(r#""active_effects":[{}]"#, effects.join(",")));
                            }
                        }
                    }

                    // Read UrafEffectInfo (Ramen uraf effect)
                    // Class: ObscuredSingleModeRamenUrafEffectInfo
                    // ObscuredInt getters: UrafEffectType, UrafEffectState
                    let uraf_class = find_class_by_short_name(image, "ObscuredSingleModeRamenUrafEffectInfo");
                    if !uraf_class.is_null() {
                        let uraf_obj = call_getter_on_instance(dataset_class, dataset_obj, "get_UrafEffectInfo");
                        if !uraf_obj.is_null() {
                            let ut = call_getter_obscured_int(uraf_class, uraf_obj, "get_UrafEffectType");
                            let us = call_getter_obscured_int(uraf_class, uraf_obj, "get_UrafEffectState");
                            result_parts.push(format!(
                                r#""uraf_effect":{{"UrafEffectType":{},"UrafEffectState":{}}}"#,
                                ut, us
                            ));
                        }
                    }

                    // Read SelectedRegionIdArray using read_obscured_int_array
                    let region_ids = read_obscured_int_array(dataset_class, dataset_obj, "get_SelectedRegionIdArray");
                    if !region_ids.is_empty() {
                        result_parts.push(format!(r#""selected_region_ids":[{}]"#, region_ids.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(",")));
                    }
                    let all_region_ids = read_obscured_int_array(dataset_class, dataset_obj, "get_AllSelectedRegionIdArray");
                    if !all_region_ids.is_empty() {
                        result_parts.push(format!(r#""all_selected_region_ids":[{}]"#, all_region_ids.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(",")));
                    }

                    // ★ v3.18.3: Read Ramen Feeling arrays for 隠し味の秘訣 (Kakushimi) tracking
                    // FeelingInfoArray: available Kakushimi items
                    let fi_arr = call_getter_on_instance(dataset_class, dataset_obj, "get_FeelingInfoArray");
                    if !fi_arr.is_null() {
                        let fi_base = fi_arr as *const u8;
                        let fi_len = std::ptr::read_unaligned::<usize>(fi_base.add(0x18) as *const usize);
                        if fi_len > 0 && fi_len < 100 {
                            // Try multiple class name patterns for FeelingInfo
                            let fi_class = find_class_by_short_name(image, "ObscuredSingleModeRamenFeelingInfo");
                            let fi_class = if fi_class.is_null() { find_class_by_short_name(image, "SingleModeRamenFeelingInfo") } else { fi_class };
                            let fi_class = if fi_class.is_null() { find_class_by_short_name(image, "WorkSingleModeRamenFeelingInfo") } else { fi_class };
                            ura_log(3, &format!("★ FeelingInfoArray: len={}, class_found={}", fi_len, !fi_class.is_null()));
                            let fi_elements = if !fi_class.is_null() {
                                read_array_element_details(fi_arr, fi_class, &["get_FeelingType", "get_FeelingValue"], &[])
                            } else {
                                // Fallback: read raw Int32 pairs from memory (FeelingType at 0x10, FeelingValue at 0x14)
                                let mut elems = Vec::new();
                                for fi in 0..fi_len {
                                    let fe_ptr = std::ptr::read_unaligned::<*mut c_void>(fi_base.add(0x20 + fi * 8) as *const *mut c_void);
                                    if fe_ptr.is_null() { elems.push("{}".to_string()); continue; }
                                    let fe_bytes = fe_ptr as *const u8;
                                    let ft = std::ptr::read_unaligned::<i32>(fe_bytes.add(0x10) as *const i32);
                                    let fv = std::ptr::read_unaligned::<i32>(fe_bytes.add(0x14) as *const i32);
                                    elems.push(format!(r#"{{"FeelingType":{},"FeelingValue":{}}}"#, ft, fv));
                                }
                                elems
                            };
                            result_parts.push(format!(r#""feeling_info":[{}]"#, fi_elements.join(",")));
                        }
                    }

                    // FeelingTurnInfoArray: turn-based Kakushimi schedule
                    let ft_arr = call_getter_on_instance(dataset_class, dataset_obj, "get_FeelingTurnInfoArray");
                    if !ft_arr.is_null() {
                        let ft_base = ft_arr as *const u8;
                        let ft_len = std::ptr::read_unaligned::<usize>(ft_base.add(0x18) as *const usize);
                        if ft_len > 0 && ft_len < 100 {
                            let ft_class = find_class_by_short_name(image, "ObscuredSingleModeRamenFeelingTurnInfo");
                            let ft_elements = if !ft_class.is_null() {
                                read_array_element_details(ft_arr, ft_class, &["get_Turn", "get_FeelingType"], &[])
                            } else {
                                (0..ft_len).map(|_| "{}".to_string()).collect()
                            };
                            result_parts.push(format!(r#""feeling_turn_info":[{}]"#, ft_elements.join(",")));
                        }
                    }

                    // CommandFeelingInfoArray: which trainings get Kakushimi boost
                    let cf_arr = call_getter_on_instance(dataset_class, dataset_obj, "get_CommandFeelingInfoArray");
                    if !cf_arr.is_null() {
                        let cf_base = cf_arr as *const u8;
                        let cf_len = std::ptr::read_unaligned::<usize>(cf_base.add(0x18) as *const usize);
                        if cf_len > 0 && cf_len < 100 {
                            let cf_class = find_class_by_short_name(image, "ObscuredSingleModeRamenCommandFeelingInfo");
                            let cf_elements = if !cf_class.is_null() {
                                read_array_element_details(cf_arr, cf_class, &["get_CommandId", "get_FeelingType"], &[])
                            } else {
                                (0..cf_len).map(|_| "{}".to_string()).collect()
                            };
                            result_parts.push(format!(r#""command_feeling_info":[{}]"#, cf_elements.join(",")));
                        }
                    }

                    // FeelingReduceTurnInfoArray: Kakushimi duration reduction
                    let fr_arr = call_getter_on_instance(dataset_class, dataset_obj, "get_FeelingReduceTurnInfoArray");
                    if !fr_arr.is_null() {
                        let fr_base = fr_arr as *const u8;
                        let fr_len = std::ptr::read_unaligned::<usize>(fr_base.add(0x18) as *const usize);
                        if fr_len > 0 && fr_len < 100 {
                            let fr_class = find_class_by_short_name(image, "ObscuredSingleModeRamenFeelingReduceTurnInfo");
                            let fr_elements = if !fr_class.is_null() {
                                read_array_element_details(fr_arr, fr_class, &["get_Turn", "get_FeelingType"], &[])
                            } else {
                                (0..fr_len).map(|_| "{}".to_string()).collect()
                            };
                            result_parts.push(format!(r#""feeling_reduce_turn_info":[{}]"#, fr_elements.join(",")));
                        }
                    }
                }

                // ★ Read object-type DataSet getters
                let obj_getters = [
                    "get_TeamSpTrainingInfo", "get_NotUpParameterInfo",
                    "get_ScenarioDressSetting", "get_TeamUnionEvent",
                ];
                let mut ds_objs = Vec::new();
                for getter in &obj_getters {
                    let obj = call_getter_on_instance(dataset_class, dataset_obj, getter);
                    if !obj.is_null() {
                        ds_objs.push(format!(r#""{}":"{:p}""#, getter, obj));
                    }
                }
                if !ds_objs.is_empty() {
                    result_parts.push(format!(r#""dataset_objects":{{{}}}"#, ds_objs.join(",")));
                }
            }
        } else {
            result_parts.push(r#""dataset_obj":"null""#.to_string());
        }
    }

    format!(r#"{{{}}}"#, result_parts.join(","))
}

// ============================================================
// ★ Enumerate ALL classes in assembly (runtime dump)
// ============================================================

unsafe fn enumerate_all_classes(search: &str) -> String {
    let image = get_image();
    if image.is_null() { return r#"{"error":"image_null"}"#.to_string(); }

    let get_count_fn = resolve_il2cpp_symbol("il2cpp_image_get_class_count");
    let get_class_fn = resolve_il2cpp_symbol("il2cpp_image_get_class");

    if get_count_fn.is_null() || get_class_fn.is_null() {
        return r#"{"error":"class_enum_api_not_found"}"#.to_string();
    }

    let get_count: FnImageGetClassCount = std::mem::transmute(get_count_fn);
    let get_class: FnImageGetClass = std::mem::transmute(get_class_fn);

    let total = get_count(image);
    let get_name_fn = resolve_il2cpp_symbol("il2cpp_class_get_name");
    let get_namespace_fn = resolve_il2cpp_symbol("il2cpp_class_get_namespace");

    let mut results = Vec::new();
    let search_lower = search.to_lowercase();

    for i in 0..total {
        let cls = get_class(image, i);
        if cls.is_null() { continue; }

        let name = if !get_name_fn.is_null() {
            let name_fn: FnClassGetName = std::mem::transmute(get_name_fn);
            let cstr = name_fn(cls);
            if cstr.is_null() { continue; }
            std::ffi::CStr::from_ptr(cstr).to_string_lossy().into_owned()
        } else {
            format!("class_{}", i)
        };

        let namespace = if !get_namespace_fn.is_null() {
            let ns_fn: FnClassGetName = std::mem::transmute(get_namespace_fn);
            let cstr = ns_fn(cls);
            if cstr.is_null() { String::new() } else { std::ffi::CStr::from_ptr(cstr).to_string_lossy().into_owned() }
        } else {
            String::new()
        };

        // Filter by search term if provided
        if !search.is_empty() {
            let full = format!("{}.{}", namespace, name).to_lowercase();
            if !full.contains(&search_lower) { continue; }
        }

        results.push(format!(r#"{{"ns":"{}","name":"{}"}}"#, namespace, name));
    }

    format!(r#"{{"total_classes":{},"matched":{},"search":"{}","classes":[{}]}}"#,
        total, results.len(), search, results.join(","))
}

// ============================================================
// All known classes for scanning
// ============================================================

const KNOWN_CLASSES: &[(&str, &str)] = &[
    ("Gallop", "WorkDataManager"),
    ("Gallop", "WorkSingleModeData"),
    ("Gallop", "WorkSingleModeCharaData"),
    ("Gallop", "WorkSingleModeHomeInfo"),
    ("Gallop", "WorkSingleModeScenarioBreeders"),
    ("Gallop", "WorkSingleModeScenarioLegend"),
    ("Gallop", "WorkSingleModeScenarioMecha"),
    ("Gallop", "WorkSingleModeScenarioOnsen"),
    ("Gallop", "WorkSingleModeScenarioPioneer"),
    ("Gallop", "WorkSingleModeScenarioRamen"),
    ("Gallop", "GameSystem"),
    ("Gallop", "HomeScene"),
    ("Gallop", "SingleModeScene"),
    ("Gallop", "RaceScene"),
    ("Gallop", "SingleModeSceneController"),
];

// ============================================================
// Scan Classes
// ============================================================

unsafe fn scan_il2cpp_classes() -> String {
    if API.is_null() { return r#"{"error":"api_null"}"#.to_string(); }

    let image = match get_image() {
        img if !img.is_null() => img,
        _ => return r#"{"error":"image_null"}"#.to_string(),
    };

    let mut found_list: Vec<String> = Vec::new();
    let mut singleton_list: Vec<String> = Vec::new();

    for (ns, cls) in KNOWN_CLASSES {
        let ns_c = to_cstr(ns);
        let cls_c = to_cstr(cls);
        let class = find_class(image, ns_c.as_ptr(), cls_c.as_ptr());
        if !class.is_null() {
            let full_name = if ns.is_empty() { cls.to_string() } else { format!("{}.{}", ns, cls) };
            if !found_list.contains(&full_name) {
                found_list.push(full_name.clone());
            }
            let inst = get_singleton(class);
            if !inst.is_null() {
                singleton_list.push(full_name);
            }
        }
    }

    format!(
        r#"{{"found_classes":["{}"],"singletons":["{}"],"total":{}}}"#,
        found_list.join("\",\""), singleton_list.join("\",\""), found_list.len()
    )
}

// ============================================================
// /singletons endpoint
// ============================================================

unsafe fn find_all_singletons() -> String {
    if API.is_null() { return r#"{"error":"api_null"}"#.to_string(); }

    let image = match get_image() {
        img if !img.is_null() => img,
        _ => return r#"{"error":"image_null"}"#.to_string(),
    };

    let mut results: Vec<String> = Vec::new();

    for (ns, cls) in KNOWN_CLASSES {
        let ns_c = to_cstr(ns);
        let cls_c = to_cstr(cls);
        let class = find_class(image, ns_c.as_ptr(), cls_c.as_ptr());
        if !class.is_null() {
            let full_name = if ns.is_empty() { cls.to_string() } else { format!("{}.{}", ns, cls) };
            let inst = get_singleton(class);
            let has_singleton = !inst.is_null();
            results.push(format!(r#"{{"class":"{}","singleton":{},"instance":"{:p}"}}"#,
                full_name, has_singleton, inst));
        }
    }

    format!(r#"{{"total":{},"classes":[{}]}}"#, results.len(), results.join(","))
}

// ============================================================
// ★ Read Training Data v3.7.8 — All via getter methods
// ============================================================

unsafe fn read_training_data() -> String {
    if API.is_null() { return r#"{"error":"api_null"}"#.to_string(); }
    ura_log(3, "Reading training data v3.7.8...");

    let image = match get_image() {
        img if !img.is_null() => img,
        _ => return r#"{"error":"image_null"}"#.to_string(),
    };

    let wdm_class = find_class(image, to_cstr("Gallop").as_ptr(), to_cstr("WorkDataManager").as_ptr());
    let sm_data_class = find_class(image, to_cstr("Gallop").as_ptr(), to_cstr("WorkSingleModeData").as_ptr());
    let chara_data_class = find_class(image, to_cstr("Gallop").as_ptr(), to_cstr("WorkSingleModeCharaData").as_ptr());

    ura_log(3, &format!("Classes: WDM={} SMD={} Chara={}",
        if wdm_class.is_null() { "null" } else { "ok" },
        if sm_data_class.is_null() { "null" } else { "ok" },
        if chara_data_class.is_null() { "null" } else { "ok" },
    ));

    // ===== Step 1: Get WorkDataManager singleton =====
    if wdm_class.is_null() {
        return r#"{"error":"WorkDataManager_class_not_found"}"#.to_string();
    }
    let wdm_instance = get_singleton(wdm_class);
    if wdm_instance.is_null() {
        return r#"{"error":"WorkDataManager_no_singleton","hint":"start_a_training_run"}"#.to_string();
    }

    // ===== Step 2: Call get_SingleMode() =====
    let sm_data_obj = call_getter_ref(wdm_class, wdm_instance, "get_SingleMode");
    if sm_data_obj.is_null() {
        // Fallback: read field directly
        let field_obj = read_field_ptr(wdm_instance, wdm_class, "<SingleMode>k__BackingField");
        if field_obj.is_null() {
            return r#"{"error":"SingleMode_null","step":"get_SingleMode"}"#.to_string();
        }
        return process_single_mode_data(field_obj, sm_data_class, chara_data_class);
    }

    process_single_mode_data(sm_data_obj, sm_data_class, chara_data_class)
}

unsafe fn process_single_mode_data(
    sm_data_obj: *const c_void,
    sm_data_class: *mut c_void,
    chara_data_class: *mut c_void,
) -> String {
    // ===== Read metadata via getters =====
    let month = if !sm_data_class.is_null() { call_getter_int(sm_data_class, sm_data_obj, "get_Month") } else { -1 };
    let half = if !sm_data_class.is_null() { call_getter_int(sm_data_class, sm_data_obj, "get_Half") } else { -1 };
    let playing_state = if !sm_data_class.is_null() { call_getter_int(sm_data_class, sm_data_obj, "get_PlayingState") } else { -1 };
    let is_playing = if !sm_data_class.is_null() { call_getter_bool(sm_data_class, sm_data_obj, "get_IsPlaying") } else { false };

    ura_log(3, &format!("SM Data: month={} half={} playingState={} isPlaying={}", month, half, playing_state, is_playing));

    // ===== Call get_Character() =====
    if sm_data_class.is_null() {
        return format!(r#"{{"error":"WorkSingleModeData_class_null","month":{},"half":{}}}"#, month, half);
    }
    let chara_obj = call_getter_ref(sm_data_class, sm_data_obj, "get_Character");
    if chara_obj.is_null() {
        // Fallback: try field
        let chara_field = read_field_ptr(sm_data_obj, sm_data_class, "<Character>k__BackingField");
        if chara_field.is_null() {
            return format!(
                r#"{{"error":"Character_null","month":{},"half":{},"playingState":{},"isPlaying":{}}}"#,
                month, half, playing_state, is_playing
            );
        }
        return read_chara_data(chara_field, chara_data_class, month, half, playing_state, is_playing);
    }

    read_chara_data(chara_obj, chara_data_class, month, half, playing_state, is_playing)
}

unsafe fn read_chara_data(
    chara_obj: *const c_void,
    chara_data_class: *mut c_void,
    month: i32,
    half: i32,
    playing_state: i32,
    is_playing: bool,
) -> String {
    if chara_data_class.is_null() {
        return r#"{"error":"WorkSingleModeCharaData_class_null"}"#.to_string();
    }

    ura_log(3, &format!("WorkSingleModeCharaData: {:p}", chara_obj));

    // ===== ★ ALL FIELDS VIA GETTER METHODS =====
    // These return plain Int32 (auto-decrypted by C# getter):
    //   get_Speed(), get_Stamina(), get_Power(), get_Guts(), get_Wiz()
    //   get_Hp(), get_MaxHp()
    //   get_Motivation() returns Motivation enum (int)
    //   get_ScenarioId() returns Int32
    //   get_FanCount() returns Int32
    // ObscuredInt getters (need special handling):
    //   get_SkillPoint() returns ObscuredInt struct

    let speed = call_getter_int(chara_data_class, chara_obj, "get_Speed");
    let stamina = call_getter_int(chara_data_class, chara_obj, "get_Stamina");
    let power = call_getter_int(chara_data_class, chara_obj, "get_Power");
    let guts = call_getter_int(chara_data_class, chara_obj, "get_Guts");
    let wiz = call_getter_int(chara_data_class, chara_obj, "get_Wiz");
    let hp = call_getter_int(chara_data_class, chara_obj, "get_Hp");
    let max_hp = call_getter_int(chara_data_class, chara_obj, "get_MaxHp");
    let motivation = call_getter_int(chara_data_class, chara_obj, "get_Motivation");
    let scenario_id = call_getter_int(chara_data_class, chara_obj, "get_ScenarioId");
    let fan_count = call_getter_int(chara_data_class, chara_obj, "get_FanCount");

    // SkillPoint returns ObscuredInt - try the ObscuredInt decoder first,
    // fall back to regular int read if it fails
    let skill_point = call_getter_obscured_int(chara_data_class, chara_obj, "get_SkillPoint");

    // ★ Scenario buffs: charaEffectIdArray (ObscuredInt[]) and scenarioProgress (ObscuredInt)
    let chara_effect_ids = read_obscured_int_array(chara_data_class, chara_obj, "get_CharaEffectIdArray");
    let scenario_progress = call_getter_obscured_int(chara_data_class, chara_obj, "get_ScenarioProgress");

    // ★ Try to read scenario-specific object (Breeders, Ramen, etc.)
    let scenario_obj = try_get_scenario_obj(chara_data_class, chara_obj, scenario_id);
    let scenario_info = if !scenario_obj.is_null() {
        format!(r#""scenario_obj":"{:p}""#, scenario_obj)
    } else {
        r#""scenario_obj":"null""#.to_string()
    };

    // Turn is not a direct getter on chara - it's on WorkSingleModeData
    // Actually from dump.cs, WorkSingleModeCharaData doesn't have turn/totalTurn
    // Those are on WorkSingleModeData: _totalTurnNum (offset 68)
    // We'll read turn from the parent data via a separate call

    ura_log(3, &format!("★ Chara: SPD={} STA={} POW={} GUT={} WIZ={} HP={}/{} MOT={} SKPT={} SCID={} FAN={} EFFECTS={:?} SPROG={}",
        speed, stamina, power, guts, wiz, hp, max_hp, motivation, skill_point, scenario_id, fan_count, chara_effect_ids, scenario_progress));

    let any_valid = speed > 0 || stamina > 0 || power > 0 || wiz > 0 || guts > 0 || hp > 0;

    let cache = CharaCache {
        speed, stamina, power, guts, wiz,
        vital: hp, max_vital: max_hp,
        motivation,
        turn: 0, // will be populated from WorkSingleModeData
        skill_point, scenario_id, fan_count,
        month, half,
        playing_state, is_playing,
        valid: any_valid,
    };
    CHARA = cache;

    if any_valid {
        let effect_ids_str: Vec<String> = chara_effect_ids.iter().map(|x| x.to_string()).collect();
        format!(
            r#"{{"ok":true,"chara":{{"speed":{},"stamina":{},"power":{},"guts":{},"wiz":{},"vital":{},"max_vital":{},"motivation":{},"skill_point":{},"scenario_id":{},"fan_count":{},"chara_effect_ids":[{}],"scenario_progress":{}}},"month":{},"half":{},"playing_state":{},"is_playing":{},{},"via":"WorkDataManager->get_SingleMode->get_Character->getters"}}"#,
            speed, stamina, power, guts, wiz,
            hp, max_hp, motivation, skill_point, scenario_id, fan_count,
            effect_ids_str.join(","), scenario_progress,
            month, half, playing_state, is_playing, scenario_info
        )
    } else {
        let effect_ids_str: Vec<String> = chara_effect_ids.iter().map(|x| x.to_string()).collect();
        format!(
            r#"{{"ok":false,"chara":{{"speed":{},"stamina":{},"power":{},"guts":{},"wiz":{},"vital":{},"max_vital":{},"motivation":{},"skill_point":{},"scenario_id":{},"fan_count":{},"chara_effect_ids":[{}],"scenario_progress":{}}},"month":{},"half":{},"warning":"all_fields_negative_or_zero",{},"via":"WorkDataManager->get_SingleMode->get_Character->getters"}}"#,
            speed, stamina, power, guts, wiz,
            hp, max_hp, motivation, skill_point, scenario_id, fan_count,
            effect_ids_str.join(","), scenario_progress,
            month, half, scenario_info
        )
    }
}

// ============================================================
// Enumerate ALL fields including parent classes
// ============================================================

unsafe fn enumerate_class_fields(class: *mut c_void) -> String {
    if class.is_null() || API.is_null() { return r#"{"error":"null_class"}"#.to_string(); }

    let get_fields_fn: Option<FnClassGetFields> = {
        let p = resolve_il2cpp_symbol("il2cpp_class_get_fields");
        if p.is_null() { None } else { Some(std::mem::transmute::<*mut c_void, FnClassGetFields>(p)) }
    };
    let get_parent_fn: Option<FnClassGetParent> = {
        let p = resolve_il2cpp_symbol("il2cpp_class_get_parent");
        if p.is_null() { None } else { Some(std::mem::transmute::<*mut c_void, FnClassGetParent>(p)) }
    };
    let get_class_name_fn: Option<FnClassGetName> = {
        let p = resolve_il2cpp_symbol("il2cpp_class_get_name");
        if p.is_null() { None } else { Some(std::mem::transmute::<*mut c_void, FnClassGetName>(p)) }
    };

    if get_fields_fn.is_none() {
        return r#"{"error":"no_il2cpp_class_get_fields"}"#.to_string();
    }

    let mut all_fields: Vec<String> = Vec::new();
    let mut current_class = class;
    let mut depth = 0;

    loop {
        if current_class.is_null() || depth > 10 { break; }

        let class_name = if let Some(ref get_name) = get_class_name_fn {
            let name_ptr = get_name(current_class);
            if !name_ptr.is_null() {
                let s = std::ffi::CStr::from_ptr(name_ptr);
                s.to_string_lossy().to_string()
            } else { format!("depth{}", depth) }
        } else { format!("depth{}", depth) };

        let mut iter: *mut c_void = ptr::null_mut();
        loop {
            let field_info = get_fields_fn.unwrap()(current_class, &mut iter);
            if field_info.is_null() { break; }

            let field_name = if !(*field_info).name.is_null() {
                let s = std::ffi::CStr::from_ptr((*field_info).name);
                s.to_string_lossy().to_string()
            } else { String::from("?") };

            let offset = (*field_info).offset;
            all_fields.push(format!(r#"{{"name":"{}","offset":{},"class":"{}"}}"#, field_name, offset, class_name));
        }

        if let Some(ref get_parent) = get_parent_fn {
            let parent = get_parent(current_class);
            if parent.is_null() || parent == current_class { break; }
            current_class = parent;
        } else {
            break;
        }
        depth += 1;
    }

    format!(r#"{{"total":{},"fields":[{}]}}"#, all_fields.len(), all_fields.join(","))
}

// ============================================================
// Enumerate methods on a class
// ============================================================

unsafe fn enumerate_class_methods(class: *mut c_void) -> String {
    if class.is_null() || API.is_null() { return r#"{"error":"null_class"}"#.to_string(); }

    let get_methods_fn: Option<FnClassGetMethods> = {
        let p = resolve_il2cpp_symbol("il2cpp_class_get_methods");
        if p.is_null() { None } else { Some(std::mem::transmute::<*mut c_void, FnClassGetMethods>(p)) }
    };
    let get_method_name_fn: Option<FnMethodGetName> = {
        let p = resolve_il2cpp_symbol("il2cpp_method_get_name");
        if p.is_null() { None } else { Some(std::mem::transmute::<*mut c_void, FnMethodGetName>(p)) }
    };
    let get_parent_fn: Option<FnClassGetParent> = {
        let p = resolve_il2cpp_symbol("il2cpp_class_get_parent");
        if p.is_null() { None } else { Some(std::mem::transmute::<*mut c_void, FnClassGetParent>(p)) }
    };
    let get_class_name_fn: Option<FnClassGetName> = {
        let p = resolve_il2cpp_symbol("il2cpp_class_get_name");
        if p.is_null() { None } else { Some(std::mem::transmute::<*mut c_void, FnClassGetName>(p)) }
    };

    if get_methods_fn.is_none() {
        return r#"{"error":"no_il2cpp_class_get_methods"}"#.to_string();
    }

    let mut all_methods: Vec<String> = Vec::new();
    let mut current_class = class;
    let mut depth = 0;
    let max_methods = 500;

    loop {
        if current_class.is_null() || depth > 5 { break; }
        if all_methods.len() >= max_methods { break; }

        let class_name = if let Some(ref get_name) = get_class_name_fn {
            let name_ptr = get_name(current_class);
            if !name_ptr.is_null() {
                let s = std::ffi::CStr::from_ptr(name_ptr);
                s.to_string_lossy().to_string()
            } else { format!("depth{}", depth) }
        } else { format!("depth{}", depth) };

        let mut iter: *mut c_void = ptr::null_mut();
        loop {
            if all_methods.len() >= max_methods { break; }
            let method_info = get_methods_fn.unwrap()(current_class, &mut iter);
            if method_info.is_null() { break; }

            let method_name = if let Some(ref get_name) = get_method_name_fn {
                let name_ptr = get_name(method_info);
                if !name_ptr.is_null() {
                    let s = std::ffi::CStr::from_ptr(name_ptr);
                    s.to_string_lossy().to_string()
                } else { String::from("?") }
            } else { String::from("?") };

            all_methods.push(format!(r#"{{"name":"{}","class":"{}"}}"#, method_name, class_name));
        }

        if let Some(ref get_parent) = get_parent_fn {
            let parent = get_parent(current_class);
            if parent.is_null() || parent == current_class { break; }
            current_class = parent;
        } else {
            break;
        }
        depth += 1;
    }

    format!(r#"{{"total":{},"methods":[{}]}}"#, all_methods.len(), all_methods.join(","))
}

// ============================================================
// /find_method endpoint
// ============================================================

unsafe fn find_method_in_all_classes(method_name: &str) -> String {
    if API.is_null() { return r#"{"error":"api_null"}"#.to_string(); }

    let image = match get_image() {
        img if !img.is_null() => img,
        _ => return r#"{"error":"image_null"}"#.to_string(),
    };

    let get_method_fn: Option<FnClassGetMethodFromName> = {
        let p = resolve_il2cpp_symbol("il2cpp_class_get_method_from_name");
        if p.is_null() { None } else { Some(std::mem::transmute::<*mut c_void, FnClassGetMethodFromName>(p)) }
    };

    if get_method_fn.is_none() {
        return r#"{"error":"no_class_get_method_from_name"}"#.to_string();
    }

    let method_name_c = to_cstr(method_name);
    let mut found: Vec<String> = Vec::new();

    for (ns, cls) in KNOWN_CLASSES {
        let ns_c = to_cstr(ns);
        let cls_c = to_cstr(cls);
        let class = find_class(image, ns_c.as_ptr(), cls_c.as_ptr());
        if class.is_null() { continue; }

        let full_name = if ns.is_empty() { cls.to_string() } else { format!("{}.{}", ns, cls) };

        let method = get_method_fn.unwrap()(class, method_name_c.as_ptr(), 0);
        if !method.is_null() {
            found.push(format!(r#"{{"class":"{}","args":0}}"#, full_name));
        }

        let method1 = get_method_fn.unwrap()(class, method_name_c.as_ptr(), 1);
        if !method1.is_null() && method.is_null() {
            found.push(format!(r#"{{"class":"{}","args":1}}"#, full_name));
        }
    }

    format!(r#"{{"method":"{}","found":{},"results":[{}]}}"#,
        method_name, !found.is_empty(), found.join(","))
}

// ============================================================
// ★ CharaEffectId → human-readable buff mapping (v3.14.2)
// From dump.cs CharaEffectId enum + CharaEffectType enum
fn chara_effect_name(id: i32) -> (&'static str, &'static str) {
    // Returns (name, effect_type) where effect_type is "Good" or "Bad"
    match id {
        1 => ("夜鷹", "Bad"),
        2 => ("怠け", "Bad"),
        3 => ("肌荒れ", "Bad"),
        4 => ("太り気", "Bad"),
        5 => ("頭痛", "Bad"),
        6 => ("練習下手", "Bad"),
        7 => ("Pt割引", "Good"),
        8 => ("愛嬌", "Good"),
        9 => ("注目", "Good"),
        10 => ("練習上手", "Good"),
        11 => ("練習◎", "Good"),
        25 => ("やる気G", "Good"),
        26 => ("調子G", "Good"),
        999 => ("ランダム", "Special"),
        _ => ("", ""), // unknown
    }
}

/// Generate buffs JSON from chara_effect_ids (works for ALL scenarios)
fn effects_to_buffs_json(effect_ids: &[i32]) -> String {
    if effect_ids.is_empty() { return "[]".to_string(); }
    let mut buffs = Vec::new();
    for &id in effect_ids {
        let (name, etype) = chara_effect_name(id);
        if name.is_empty() {
            // Unknown effect — output raw ID for debugging
            buffs.push(format!(r#"{{"name":"Effect#{}","level":0,"desc":"unknown effect","type":"Unknown"}}"#, id));
        } else {
            buffs.push(format!(r#"{{"name":"{}","level":0,"desc":"{}","type":"{}"}}"#, name, name, etype));
        }
    }
    format!("[{}]", buffs.join(","))
}

// ★ Clean summary for floating window app (/summary endpoint)
// v3.10.0: Player-friendly output — stats + training gains in one response
// ============================================================

/// Breeders作戦会議buff (游戏内青・緑・桃三色)
/// GroupType 1=青(フィジカル), 2=緑(テクニック), 3=桃(メンタル)
fn breeders_buff_desc(group_type: i32, level: i32) -> (&'static str, String) {
    match group_type {
        1 => { // 青: 友情ボーナス + サブ能力UP + 体力消費DOWN
            let desc = match level {
                0 => "-".to_string(),
                1 => "友情+10% サブ+15%".to_string(),
                2 => "友情+20% サブ+25% 体消-40%".to_string(),
                3 => "友情+25% サブ+30% 体消-70%".to_string(),
                4 => "友情+35% サブ+35% 体消-100%".to_string(),
                5 => "友情+40% サブ+40% 体消-100%".to_string(),
                6 => "友情+50% サブ+45% 体消-100%".to_string(),
                7 => "友情+55% サブ+50% 体消-100%".to_string(),
                8 => "友情+65%".to_string(),
                _ => format!("Lv{}", level),
            };
            ("青", desc)
        }
        2 => { // 緑: スキルPt効果UP + ヒント発生
            let desc = match level {
                0 => "-".to_string(),
                1 => "Pt+10%".to_string(),
                2 => "Pt+15%".to_string(),
                3 => "Pt+20% ヒント1人".to_string(),
                4 => "Pt+25% ヒント2人".to_string(),
                5 => "Pt+30% ヒント2人 全ヒント".to_string(),
                6 => "Pt+35% ヒント2人 全ヒント".to_string(),
                7 => "Pt+40% ヒント2人 全ヒント".to_string(),
                8 => "Pt+50% ヒント2人 全ヒント".to_string(),
                _ => format!("Lv{}", level),
            };
            ("緑", desc)
        }
        3 => { // 桃: 絆獲得UP + 失敗率DOWN + 獲得上限UP
            let desc = match level {
                0 => "-".to_string(),
                1 => "絆+3 失敗-5%".to_string(),
                2 => "絆+5 失敗-50% 上限+15".to_string(),
                3 => "絆+7 失敗-100% 上限+25 Pt上限+40".to_string(),
                4 => "絆+7 失敗-100% 上限+35 Pt上限+60".to_string(),
                5 => "絆+7 失敗-100% 上限+40 Pt上限+80".to_string(),
                6 => "絆+7 失敗-100% 上限+45 Pt上限+100".to_string(),
                7 => "絆+7 失敗-100% 上限+50 Pt上限+110".to_string(),
                8 => "絆+7 失敗-100% 上限+60 Pt上限+120".to_string(),
                _ => format!("Lv{}", level),
            };
            ("桃", desc)
        }
        _ => ("?", format!("Lv{}", level)),
    }
}

/// Safe wrapper: catches panics from read_summary_inner to prevent game crash

// ============================================================
// ★ AI Evaluation Module (v3.15.1)
// Handwritten evaluation logic ported from UmaAi
// ============================================================

const FIVE_STATUS_FINAL_SCORE: [i32; 2801] = [
0,1,1,2,2,3,3,4,4,5,5,6,6,7,7,8,8,9,9,10,
    10,11,11,12,12,13,13,14,14,15,15,16,16,17,17,18,18,19,19,20,
    20,21,21,22,22,23,23,24,24,25,25,26,27,28,29,29,30,31,32,33,
    33,34,35,36,37,37,38,39,40,41,41,42,43,44,45,45,46,47,48,49,
    49,50,51,52,53,53,54,55,56,57,57,58,59,60,61,61,62,63,64,65,
    66,67,68,69,70,71,72,73,74,75,76,77,78,79,80,81,82,83,84,85,
    86,87,88,89,90,91,92,93,94,95,96,97,98,99,100,101,102,103,104,105,
    106,107,108,109,110,111,112,113,114,115,116,117,118,120,121,122,124,125,126,128,
    129,130,131,133,134,135,137,138,139,141,142,143,144,146,147,148,150,151,152,154,
    155,156,157,159,160,161,163,164,165,167,168,169,170,172,173,174,176,177,178,180,
    181,183,184,186,188,189,191,192,194,196,197,199,200,202,204,205,207,208,210,212,
    213,215,216,218,220,221,223,224,226,228,229,231,232,234,236,237,239,240,242,244,
    245,247,248,250,252,253,255,256,258,260,261,263,265,267,269,270,272,274,276,278,
    279,281,283,285,287,288,290,292,294,296,297,299,301,303,305,306,308,310,312,314,
    315,317,319,321,323,324,326,328,330,332,333,335,337,339,341,342,344,346,348,350,
    352,354,356,358,360,362,364,366,368,371,373,375,377,379,381,383,385,387,389,392,
    394,396,398,400,402,404,406,408,410,413,415,417,419,421,423,425,427,429,431,434,
    436,438,440,442,444,446,448,450,452,455,457,459,462,464,467,469,471,474,476,479,
    481,483,486,488,491,493,495,498,500,503,505,507,510,512,515,517,519,522,524,527,
    529,531,534,536,539,541,543,546,548,551,553,555,558,560,563,565,567,570,572,575,
    577,580,582,585,588,590,593,595,598,601,603,606,608,611,614,616,619,621,624,627,
    629,632,634,637,640,642,645,647,650,653,655,658,660,663,666,668,671,673,676,679,
    681,684,686,689,692,694,697,699,702,705,707,710,713,716,719,721,724,727,730,733,
    735,738,741,744,747,749,752,755,758,761,763,766,769,772,775,777,780,783,786,789,
    791,794,797,800,803,805,808,811,814,817,819,822,825,828,831,833,836,839,842,845,
    847,850,853,856,859,862,865,868,871,874,876,879,882,885,888,891,894,897,900,903,
    905,908,911,914,917,920,923,926,929,932,934,937,940,943,946,949,952,955,958,961,
    963,966,969,972,975,978,981,984,987,990,993,996,999,1002,1005,1008,1011,1014,1017,1020,
    1023,1026,1029,1032,1035,1038,1041,1044,1047,1050,1053,1056,1059,1062,1065,1068,1071,1074,1077,1080,
    1083,1086,1089,1092,1095,1098,1101,1104,1107,1110,1113,1116,1119,1122,1125,1128,1131,1134,1137,1140,
    1143,1146,1149,1152,1155,1158,1161,1164,1167,1171,1174,1177,1180,1183,1186,1189,1192,1195,1198,1202,
    1205,1208,1211,1214,1217,1220,1223,1226,1229,1233,1236,1239,1242,1245,1248,1251,1254,1257,1260,1264,
    1267,1270,1273,1276,1279,1282,1285,1288,1291,1295,1298,1301,1304,1308,1311,1314,1318,1321,1324,1328,
    1331,1334,1337,1341,1344,1347,1351,1354,1357,1361,1364,1367,1370,1374,1377,1380,1384,1387,1390,1394,
    1397,1400,1403,1407,1410,1413,1417,1420,1423,1427,1430,1433,1436,1440,1443,1446,1450,1453,1456,1460,
    1463,1466,1470,1473,1477,1480,1483,1487,1490,1494,1497,1500,1504,1507,1511,1514,1517,1521,1524,1528,
    1531,1534,1538,1541,1545,1548,1551,1555,1558,1562,1565,1568,1572,1575,1579,1582,1585,1589,1592,1596,
    1599,1602,1606,1609,1613,1616,1619,1623,1626,1630,1633,1637,1640,1644,1647,1651,1654,1658,1661,1665,
    1668,1672,1675,1679,1682,1686,1689,1693,1696,1700,1703,1707,1710,1714,1717,1721,1724,1728,1731,1735,
    1738,1742,1745,1749,1752,1756,1759,1763,1766,1770,1773,1777,1780,1784,1787,1791,1794,1798,1801,1805,
    1808,1812,1816,1820,1824,1828,1832,1836,1840,1844,1847,1851,1855,1859,1863,1867,1871,1875,1879,1883,
    1886,1890,1894,1898,1902,1906,1910,1914,1918,1922,1925,1929,1933,1937,1941,1945,1949,1953,1957,1961,
    1964,1968,1972,1976,1980,1984,1988,1992,1996,2000,2004,2008,2012,2016,2020,2024,2028,2032,2036,2041,
    2045,2049,2053,2057,2061,2065,2069,2073,2077,2082,2086,2090,2094,2098,2102,2106,2110,2114,2118,2123,
    2127,2131,2135,2139,2143,2147,2151,2155,2159,2164,2168,2172,2176,2180,2184,2188,2192,2196,2200,2205,
    2209,2213,2217,2221,2226,2230,2234,2238,2242,2247,2251,2255,2259,2263,2268,2272,2276,2280,2284,2289,
    2293,2297,2301,2305,2310,2314,2318,2322,2326,2331,2335,2339,2343,2347,2352,2356,2360,2364,2368,2373,
    2377,2381,2385,2389,2394,2398,2402,2406,2410,2415,2419,2423,2427,2432,2436,2440,2445,2449,2453,2458,
    2462,2466,2470,2475,2479,2483,2488,2492,2496,2501,2505,2509,2513,2518,2522,2526,2531,2535,2539,2544,
    2548,2552,2556,2561,2565,2569,2574,2578,2582,2587,2591,2595,2599,2604,2608,2612,2617,2621,2625,2630,
    2635,2640,2645,2650,2656,2661,2666,2671,2676,2682,2687,2692,2697,2702,2708,2713,2718,2723,2728,2734,
    2739,2744,2749,2754,2760,2765,2770,2775,2780,2786,2791,2796,2801,2806,2812,2817,2822,2827,2832,2838,
    2843,2848,2853,2858,2864,2869,2874,2879,2884,2890,2895,2901,2906,2912,2917,2923,2928,2934,2939,2945,
    2950,2956,2961,2967,2972,2978,2983,2989,2994,3000,3005,3011,3016,3022,3027,3033,3038,3044,3049,3055,
    3060,3066,3071,3077,3082,3088,3093,3099,3104,3110,3115,3121,3126,3132,3137,3143,3148,3154,3159,3165,
    3171,3178,3184,3191,3198,3204,3211,3217,3224,3231,3237,3244,3250,3257,3264,3270,3277,3283,3290,3297,
    3303,3310,3316,3323,3330,3336,3343,3349,3356,3363,3369,3376,3382,3389,3396,3402,3409,3415,3422,3429,
    3435,3442,3448,3455,3462,3468,3475,3481,3488,3495,3501,3508,3515,3522,3529,3535,3542,3549,3556,3563,
    3569,3576,3583,3590,3597,3603,3610,3617,3624,3631,3637,3644,3651,3658,3665,3671,3678,3685,3692,3699,
    3705,3712,3719,3726,3733,3739,3746,3753,3760,3767,3773,3780,3787,3794,3801,3807,3814,3821,3828,3835,
    3841,3841,3849,3849,3857,3857,3865,3865,3873,3873,3881,3881,3889,3889,3897,3897,3905,3905,3912,3912,
    3920,3920,3928,3928,3936,3936,3944,3944,3952,3952,3960,3960,3968,3968,3976,3976,3984,3984,3992,3992,
    4001,4001,4009,4009,4017,4017,4025,4025,4033,4033,4041,4041,4049,4049,4057,4057,4065,4065,4073,4073,
    4082,4082,4090,4090,4098,4098,4107,4107,4115,4115,4123,4123,4132,4132,4140,4140,4148,4148,4156,4156,
    4165,4165,4173,4173,4182,4182,4190,4190,4198,4198,4207,4207,4215,4215,4224,4224,4232,4232,4240,4240,
    4249,4249,4257,4257,4266,4266,4274,4274,4283,4283,4291,4291,4300,4300,4308,4308,4317,4317,4325,4325,
    4334,4334,4343,4343,4351,4351,4360,4360,4368,4368,4377,4377,4386,4386,4394,4394,4403,4403,4411,4411,
    4420,4420,4429,4429,4438,4438,4447,4447,4455,4455,4464,4464,4473,4473,4482,4482,4491,4491,4499,4499,
    4508,4508,4517,4517,4526,4526,4535,4535,4544,4544,4553,4553,4562,4562,4571,4571,4580,4580,4588,4588,
    4597,4597,4606,4606,4615,4615,4624,4624,4633,4633,4642,4642,4651,4651,4660,4660,4669,4669,4678,4678,
    4688,4688,4697,4697,4706,4706,4715,4715,4724,4724,4734,4734,4743,4743,4752,4752,4761,4761,4770,4770,
    4780,4780,4789,4789,4798,4798,4808,4808,4817,4817,4826,4826,4836,4836,4845,4845,4854,4854,4863,4863,
    4873,4873,4882,4882,4892,4892,4901,4901,4910,4910,4920,4920,4929,4929,4939,4939,4948,4948,4957,4957,
    4967,4967,4977,4977,4986,4986,4996,4996,5005,5005,5015,5015,5025,5025,5034,5034,5044,5044,5053,5053,
    5063,5063,5073,5073,5083,5083,5092,5092,5102,5102,5112,5112,5121,5121,5131,5131,5141,5141,5150,5150,
    5160,5160,5170,5170,5180,5180,5190,5190,5199,5199,5209,5209,5219,5219,5229,5229,5239,5239,5248,5248,
    5258,5258,5268,5268,5278,5278,5288,5288,5298,5298,5308,5308,5318,5318,5328,5328,5338,5338,5348,5348,
    5359,5359,5369,5369,5379,5379,5389,5389,5399,5399,5409,5409,5419,5419,5429,5429,5439,5439,5449,5449,
    5460,5460,5470,5470,5480,5480,5490,5490,5500,5500,5511,5511,5521,5521,5531,5531,5541,5541,5551,5551,
    5562,5562,5572,5572,5582,5582,5593,5593,5603,5603,5613,5613,5624,5624,5634,5634,5644,5644,5654,5654,
    5665,5665,5675,5675,5686,5686,5696,5696,5707,5707,5717,5717,5728,5728,5738,5738,5749,5749,5759,5759,
    5770,5770,5781,5781,5791,5791,5802,5802,5812,5812,5823,5823,5834,5834,5844,5844,5855,5855,5865,5865,
    5876,5876,5887,5887,5898,5898,5908,5908,5919,5919,5930,5930,5940,5940,5951,5951,5962,5962,5972,5972,
    5983,5983,5994,5994,6005,6005,6016,6016,6027,6027,6038,6038,6049,6049,6060,6060,6071,6071,6081,6081,
    6092,6092,6103,6103,6114,6114,6125,6125,6136,6136,6147,6147,6158,6158,6169,6169,6180,6180,6191,6191,
    6203,6203,6214,6214,6225,6225,6236,6236,6247,6247,6258,6258,6269,6269,6280,6280,6291,6291,6302,6302,
    6314,6314,6325,6325,6336,6336,6348,6348,6359,6359,6370,6370,6382,6382,6393,6393,6404,6404,6415,6415,
    6427,6427,6438,6438,6450,6450,6461,6461,6472,6472,6484,6484,6495,6495,6507,6507,6518,6518,6529,6529,
    6541,6541,6552,6552,6564,6564,6575,6575,6587,6587,6598,6598,6610,6610,6621,6621,6633,6633,6644,6644,
    6656,6656,6668,6668,6680,6680,6691,6691,6703,6703,6715,6715,6726,6726,6738,6738,6750,6750,6761,6761,
    6773,6773,6785,6785,6797,6797,6809,6809,6820,6820,6832,6832,6844,6844,6856,6856,6868,6868,6879,6879,
    6891,6891,6903,6903,6915,6915,6927,6927,6939,6939,6951,6951,6963,6963,6975,6975,6987,6987,6998,6998,
    7011,7011,7023,7023,7035,7035,7047,7047,7059,7059,7071,7071,7083,7083,7095,7095,7107,7107,7119,7119,
    7132,7132,7144,7144,7156,7156,7168,7168,7180,7180,7193,7193,7205,7205,7217,7217,7229,7229,7241,7241,
    7254,7254,7266,7266,7278,7278,7291,7291,7303,7303,7315,7315,7328,7328,7340,7340,7352,7352,7364,7364,
    7377,7377,7389,7389,7402,7402,7414,7414,7426,7426,7439,7439,7451,7451,7464,7464,7476,7476,7488,7488,
    7501,7501,7514,7514,7526,7526,7539,7539,7551,7551,7564,7564,7577,7577,7589,7589,7602,7602,7614,7614,
    7627,7627,7640,7640,7653,7653,7665,7665,7678,7678,7691,7691,7703,7703,7716,7716,7729,7729,7741,7741,
    7754,7754,7767,7767,7780,7780,7793,7793,7805,7805,7818,7818,7831,7831,7844,7844,7857,7857,7869,7869,
    7882,7882,7895,7895,7908,7908,7921,7921,7934,7934,7947,7947,7960,7960,7973,7973,7986,7986,7999,7999,
    8013,8013,8026,8026,8039,8039,8052,8052,8065,8065,8078,8078,8091,8091,8104,8104,8117,8117,8130,8130,
    8144,8144,8157,8157,8170,8170,8183,8183,8196,8196,8210,8210,8223,8223,8236,8236,8249,8249,8262,8262,
    8276,8276,8289,8289,8303,8303,8316,8316,8329,8329,8343,8343,8356,8356,8370,8370,8383,8383,8396,8396,
    8410,8410,8423,8423,8437,8437,8450,8450,8464,8464,8477,8477,8491,8491,8504,8504,8518,8518,8531,8531,
    8545,8545,8559,8559,8572,8572,8586,8586,8599,8599,8613,8613,8627,8627,8640,8640,8654,8654,8667,8667,
    8681,8681,8695,8695,8709,8709,8723,8723,8736,8736,8750,8750,8764,8764,8778,8778,8792,8792,8805,8805,
    8819,8819,8833,8833,8847,8847,8861,8861,8875,8875,8889,8889,8903,8903,8917,8917,8931,8931,8944,8944,
    8958,8958,8972,8972,8986,8986,9000,9000,9014,9014,9028,9028,9042,9042,9056,9056,9070,9070,9084,9084,
    9099,9099,9113,9113,9127,9127,9141,9141,9155,9155,9169,9169,9183,9183,9197,9197,9211,9211,9225,9225,
    9240,9240,9254,9254,9268,9268,9283,9283,9297,9297,9311,9311,9326,9326,9340,9340,9354,9354,9368,9368,
    9383,9383,9397,9397,9412,9412,9426,9426,9440,9440,9455,9455,9469,9469,9484,9484,9498,9498,9512,9512,
    9527,9527,9541,9541,9556,9556,9570,9570,9585,9585,9599,9599,9614,9614,9628,9628,9643,9643,9657,9657,
    9672,9672,9687,9687,9702,9702,9716,9716,9731,9731,9746,9746,9760,9760,9775,9775,9790,9790,9804,9804,
    9819,9819,9834,9834,9849,9849,9864,9864,9878,9878,9893,9893,9908,9908,9923,9923,9938,9938,9952,9952,
    9967,9967,9982,9982,9997,9997,10012,10012,10027,10027,10042,10042,10057,10057,10072,10072,10087,10087,10101,10101,
    10117,10117,10132,10132,10147,10147,10162,10162,10177,10177,10192,10192,10207,10207,10222,10222,10237,10237,10252,10252,
    10268,10268,10283,10283,10298,10298,10313,10313,10328,10328,10344,10344,10359,10359,10374,10374,10389,10389,10404,10404,
    10420,10420,10435,10435,10450,10450,10466,10466,10481,10481,10496,10496,10512,10512,10527,10527,10542,10542,10557,10557,
    10573,10573,10588,10588,10604,10604,10619,10619,10635,10635,10650,10650,10666,10666,10681,10681,10697,10697,10712,10712,
    10728,10728,10744,10744,10759,10759,10775,10775,10790,10790,10806,10806,10822,10822,10837,10837,10853,10853,10868,10868,
    10884,10884,10900,10900,10916,10916,10931,10931,10947,10947,10963,10963,10978,10978,10994,10994,11010,11010,11025,11025,
    11041,11041,11057,11057,11073,11073,11089,11089,11105,11105,11121,11121,11137,11137,11153,11153,11169,11169,11184,11184,
    11200,11200,11216,11216,11232,11232,11248,11248,11264,11264,11280,11280,11296,11296,11312,11312,11328,11328,11344,11344,
    11361,11361,11377,11377,11393,11393,11409,11409,11425,11425,11441,11441,11457,11457,11473,11473,11489,11489,11505,11505,
    11522,11522,11538,11538,11554,11554,11570,11570,11586,11586,11603,11603,11619,11619,11635,11635,11651,11651,11667,11667,
    11684,11684,11700,11700,11717,11717,11733,11733,11749,11749,11766,11766,11782,11782,11799,11799,11815,11815,11831,11831,
    11848,11848,11864,11864,11881,11881,11897,11897,11914,11914,11930,11930,11947,11947,11963,11963,11980,11980,11996,11996,
    12013,12013,12030,12030,12046,12046,12063,12063,12079,12079,12096,12096,12113,12113,12129,12129,12146,12146,12162,12162,
    12179,12179,12196,12196,12213,12213,12230,12230,12246,12246,12263,12263,12280,12280,12297,12297,12314,12314,12330,12330,
    12347,12347,12364,12364,12381,12381,12398,12398,12415,12415,12432,12432,12449,12449,12466,12466,12483,12483,12499,12499,
    12516,12516,12533,12533,12550,12550,12567,12567,12584,12584,12601,12601,12618,12618,12635,12635,12652,12652,12669,12669,
    12687,12687,12704,12704,12721,12721,12738,12738,12755,12755,12773,12773,12790,12790,12807,12807,12824,12824,12841,12841,
    12859,12859,12876,12876,12893,12893,12911,12911,12928,12928,12945,12945,12963,12963,12980,12980,12997,12997,13014,13014,
    13032,13032,13049,13049,13067,13067,13084,13084,13101,13101,13119,13119,13136,13136,13154,13154,13171,13171,13188,13188,
    13206,13206,13224,13224,13241,13241,13259,13259,13276,13276,13294,13294,13312,13312,13329,13329,13347,13347,13364,13364,
    13382,13382,13400,13400,13418,13418,13435,13435,13453,13453,13471,13471,13488,13488,13506,13506,13524,13524,13541,13541,
    13559,13559,13577,13577,13595,13595,13613,13613,13630,13630,13648,13648,13666,13666,13684,13684,13702,13702,13719,13719,
    13737,13737,13755,13755,13773,13773,13791,13791,13809,13809,13827,13827,13845,13845,13863,13863,13881,13881,13898,13898,
    13917,13917,13935,13935,13953,13953,13971,13971,13989,13989,14007,14007,14025,14025,14043,14043,14061,14061,14079,14079,
    14098,14098,14116,14116,14134,14134,14152,14152,14170,14170,14189,14189,14207,14207,14225,14225,14243,14243,14261,14261,
    14280
];

/// Basic five status upper limit (default, no chara-specific adjustments)
/// [Speed, Stamina, Power, Guts, Wisdom]
const BASIC_FIVE_STATUS_LIMIT: [i32; 5] = [2300, 2200, 1800, 1400, 1400];

/// ReviseOver1200: stats above 1200 have diminishing returns
/// x > 1200 → 2x - 1200, otherwise x
fn revise_over_1200(x: i32) -> i32 {
    if x > 1200 { x * 2 - 1200 } else { x }
}

/// Compute current evaluation score from five stats
fn compute_score(speed: i32, stamina: i32, power: i32, guts: i32, wiz: i32) -> i32 {
    let total = revise_over_1200(speed) + revise_over_1200(stamina)
              + revise_over_1200(power) + revise_over_1200(guts)
              + revise_over_1200(wiz);
    if total < 0 { return 0; }
    let idx = total as usize;
    if idx >= FIVE_STATUS_FINAL_SCORE.len() { return FIVE_STATUS_FINAL_SCORE[FIVE_STATUS_FINAL_SCORE.len() - 1]; }
    FIVE_STATUS_FINAL_SCORE[idx]
}

/// Soft constraint function for stat overflow control
/// When stat gain would exceed remaining space, reduce its effective value
fn status_soft_function(x: f64, reserve: f64) -> f64 {
    if x >= 0.0 { return 0.0; }
    if x > -reserve { return -x * x / (2.0 * reserve); }
    x + 0.5 * reserve
}

/// Vital evaluation: low vital is very valuable, high vital less so
/// ≤50: 2.0x, 50-70: 1.5x, 70+: 1.0x
fn vital_evaluation(vital: i32, max_vital: i32) -> f64 {
    let v = if vital > max_vital { max_vital } else { vital };
    if v <= 50 {
        2.0 * v as f64
    } else if v <= 70 {
        1.5 * (v - 50) as f64 + 100.0  // 2.0 * 50 = 100
    } else {
        1.0 * (v - 70) as f64 + 130.0   // 100 + 1.5*20 = 130
    }
}

/// Calculate max vital equivalent for vital evaluation
/// Late game: less vital needed (fewer turns remain)
fn calculate_max_vital_eq(turn: i32, max_vital: i32) -> i32 {
    if turn >= 76 { return 0; }
    if turn > 71 { return 10; }
    if turn == 71 { return 30; }
    // Assume max 6 non-race turns before URA
    let non_race_turns = std::cmp::min(6, 71 - turn);
    let eq = 30 + 15 * non_race_turns;
    if eq > max_vital { max_vital } else { eq }
}

/// CommandId → training index (0=Speed, 1=Stamina, 2=Power, 3=Guts, 4=Wisdom)
fn cmd_id_to_train_idx(cmd_id: i32) -> Option<usize> {
    match cmd_id {
        101 => Some(0), // Speed
        102 => Some(1), // Stamina
        105 => Some(2), // Power
        103 => Some(3), // Guts
        106 => Some(4), // Wisdom
        _ => None,
    }
}

/// AI evaluation result
struct AiResult {
    score: i32,           // Current evaluation score
    total_stats: i32,     // Total revised stats
    best_action: String,  // Recommended action name
    best_value: f64,      // Best action value
    train_values: Vec<(String, f64)>,  // Per-training values
    rest_value: f64,      // Rest value
    outgoing_value: f64,  // Outgoing value
}

/// Run handwritten AI evaluation for current game state
/// Input: all data from read_summary_inner
fn evaluate_ai(
    turn: i32,
    stats: [i32; 5],     // [speed, stamina, power, guts, wiz]
    vital: i32,
    max_vital: i32,
    motivation: i32,      // 1-5
    scenario_id: i32,
    // Per-training data: (command_id, [5 stat gains], skill_pt_gain, vital_cost, failure_rate, is_enable, shining, heads)
    trainings: &[(i32, [i32; 5], i32, i32, i32, i32, i32, i32)],
    // Buff effects
    _has_ai_jiao: bool,    // 愛嬌 buff (TODO: implement buff effect)
    _has_renshou_jouzu: bool, // 練習上手 buff (TODO: implement buff effect)
) -> AiResult {
    // Total turns per scenario
    let total_turn: i32 = match scenario_id {
        1 => 78,  // URA
        _ => 72,
    };

    let remain_turn = total_turn - turn - 1;
    let remain_turn = if remain_turn < 0 { 0 } else { remain_turn };

    // === Current Score ===
    let score = compute_score(stats[0], stats[1], stats[2], stats[3], stats[4]);
    let total_stats = revise_over_1200(stats[0]) + revise_over_1200(stats[1])
                    + revise_over_1200(stats[2]) + revise_over_1200(stats[3])
                    + revise_over_1200(stats[4]);

    // === Evaluation Parameters ===
    let status_weights = [6.0, 6.0, 6.0, 6.0, 6.0];
    let small_fail_value = -150.0;
    let big_fail_value = -500.0;
    let pt_score_rate = 2.0;

    // Vital factor: increases from 3.5 to 7.0 as game progresses
    let vital_factor = 3.5 + (turn as f64 / total_turn as f64) * 3.5;

    // Reserve for soft constraint: controls stat overflow penalty
    let reserve = 40.0 * remain_turn as f64 * (1.0 - remain_turn as f64 / (total_turn as f64 * 2.0));
    let reserve = if reserve > 0.1 { reserve } else { 0.1 }; // avoid div by zero

    // URA final bonus (events that add stats after training)
    let mut final_bonus = 45 + 30; // URA3 + final event
    if remain_turn >= 1 { final_bonus += 20; } // URA2
    if remain_turn >= 2 { final_bonus += 20; } // URA1

    // Remaining space per stat
    let mut remain = [0.0f64; 5];
    for i in 0..5 {
        remain[i] = (BASIC_FIVE_STATUS_LIMIT[i] - stats[i] - final_bonus) as f64;
    }

    // Vital evaluation baseline
    let max_vital_eq = calculate_max_vital_eq(turn, max_vital);
    let vital_before = vital_evaluation(std::cmp::min(vital, max_vital_eq), max_vital);

    let mut train_values = Vec::new();
    let mut best_value = std::f64::NEG_INFINITY;
    let mut best_action = "Rest".to_string();

    // === Evaluate each training ===
    for &(cmd_id, ref gains, skill_pt, vital_cost, fail_rate, is_enable, shining, heads) in trainings {
        let name = match cmd_id {
            101 => "Speed", 102 => "Stamina", 103 => "Guts",
            105 => "Power", 106 => "Wisdom",
            601 => "Speed", 602 => "Stamina", 603 => "Guts",
            604 => "Power", 605 => "Wisdom",
            304 => "Kakushimi",
            _ => "Unknown",
        };

        if is_enable == 0 || name == "Unknown" {
            train_values.push((name.to_string(), std::f64::NEG_INFINITY));
            continue;
        }

        // Status gain evaluation with soft constraints
        let mut value = 0.0;
        for sta in 0..5 {
            let s0 = status_soft_function(-remain[sta], reserve);
            let s1 = status_soft_function(gains[sta] as f64 - remain[sta], reserve);
            value += status_weights[sta] * (s1 - s0);
        }

        // Skill point value
        value += pt_score_rate * skill_pt as f64;

        // ★ v3.15.3: Motivation factor — higher mood = more future training value
        // Low motivation reduces effective training value; high motivation boosts it
        // mot_factor: 1=0.6, 2=0.75, 3=0.9, 4=1.0, 5=1.1
        let mot_factor = match motivation {
            1 => 0.6,
            2 => 0.75,
            3 => 0.9,
            4 => 1.0,
            _ => 1.1,  // 5 = 絶好調
        };
        value *= mot_factor;

        // Vital change effect
        let vital_after = std::cmp::min(max_vital_eq, vital + vital_cost);
        value += vital_factor * (vital_evaluation(vital_after, max_vital) - vital_before);

        // Failure penalty
        if fail_rate > 0 {
            let big_fail_prob = if fail_rate < 20 { 0.0 } else { fail_rate as f64 };
            let fail_value_avg = 0.01 * big_fail_prob * big_fail_value
                               + (1.0 - 0.01 * big_fail_prob) * small_fail_value;
            value = 0.01 * fail_rate as f64 * fail_value_avg
                  + (1.0 - 0.01 * fail_rate as f64) * value;
        }

        // ★ v3.15.3: Shining (彩圈) bonus — friend/group card event expected value
        // Each 彩圈 partner gives a training event with extra stats + skill hint
        if shining > 0 {
            let shining_bonus = 200.0 * shining as f64;  // ~200 per 彩圈 partner (stats + skill hint + friendship)
            value += shining_bonus;
        }
        // Heads bonus: more partners = faster relationship building
        if heads > 1 {
            let heads_bonus = 20.0 * (heads - 1) as f64;  // small bonus for extra partners beyond first
            value += heads_bonus;
        }
        train_values.push((name.to_string(), value));

        if value > best_value {
            best_value = value;
            best_action = name.to_string();
        }
    }

    // === Evaluate Rest ===
    let rest_vital_gain = 50;
    let vital_after_rest = std::cmp::min(max_vital_eq, vital + rest_vital_gain);
    let rest_value = vital_factor * (vital_evaluation(vital_after_rest, max_vital) - vital_before);

    if rest_value > best_value {
        best_value = rest_value;
        best_action = "Rest".to_string();
    }

    // === Evaluate Outgoing ===
    // ★ v3.15.3: outgoing bonus scales with motivation deficit
    // Only recommend外出 when motivation is low (≤3 = 普通 or worse)
    // mot 1→2: big value (80), mot 2→3: medium (50), mot 3→4: small (25), mot 4→5: negligible (10)
    let outgoing_bonus = match motivation {
        1 => 80.0,   // 絶不調 → 不調: urgent
        2 => 50.0,   // 不調 → 普通: important
        3 => 25.0,   // 普通 → 好調: moderate
        4 => 10.0,   // 好調 → 絶好調: minor gain
        _ => 0.0,    // already 絶好調
    };
    let outgoing_vital_gain = 50;
    let vital_after_outgoing = std::cmp::min(max_vital_eq, vital + outgoing_vital_gain);
    let outgoing_value = vital_factor * (vital_evaluation(vital_after_outgoing, max_vital) - vital_before)
                        + outgoing_bonus;

    if outgoing_value > best_value {
        best_value = outgoing_value;
        best_action = "Outgoing".to_string();
    }

    AiResult {
        score,
        total_stats,
        best_action,
        best_value,
        train_values,
        rest_value,
        outgoing_value,
    }
}

/// Format AI result as compact JSON for /summary output
fn ai_result_to_json(r: &AiResult) -> String {
    let tv: Vec<String> = r.train_values.iter()
        .map(|(n, v)| format!(r#""{}":{}"#, n, (v * 10.0).round() / 10.0))
        .collect();
    format!(
        r#"{{"score":{},"total_stats":{},"best":"{}","best_v":{},"train":{{{}}},"rest":{},"outgoing":{}}}"#,
        r.score,
        r.total_stats,
        r.best_action,
        (r.best_value * 10.0).round() / 10.0,
        tv.join(","),
        (r.rest_value * 10.0).round() / 10.0,
        (r.outgoing_value * 10.0).round() / 10.0,
    )
}

fn read_summary() -> String {
    // ★ v3.15.2: Mutex lock prevents concurrent il2cpp reads from HTTP + push threads
    let _lock = READ_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        unsafe { read_summary_inner() }
    })).unwrap_or_else(|_| r#"{"error":"panic_caught","hint":"read_summary panicked, game protected"}"#.to_string())
}

unsafe fn read_summary_inner() -> String {
    if API.is_null() { return r#"{"error":"api_null"}"#.to_string(); }
    let image = match get_image() {
        img if !img.is_null() => img,
        _ => return r#"{"error":"image_null"}"#.to_string(),
    };

    // --- Chara stats ---
    ura_log(3, "★ read_summary phase1: chara stats");
    let wdm_class = find_class(image, to_cstr("Gallop").as_ptr(), to_cstr("WorkDataManager").as_ptr());
    if wdm_class.is_null() { return r#"{"error":"no_wdm"}"#.to_string(); }
    let wdm_inst = get_singleton(wdm_class);
    if wdm_inst.is_null() { return r#"{"error":"no_wdm_inst"}"#.to_string(); }

    let sm_class = find_class(image, to_cstr("Gallop").as_ptr(), to_cstr("WorkSingleModeData").as_ptr());
    let sm_obj = call_getter_ref(wdm_class, wdm_inst, "get_SingleMode");
    if sm_obj.is_null() { return r#"{"error":"no_sm"}"#.to_string(); }

    let chara_class = find_class(image, to_cstr("Gallop").as_ptr(), to_cstr("WorkSingleModeCharaData").as_ptr());
    let chara_obj = call_getter_ref(sm_class, sm_obj, "get_Character");
    if chara_obj.is_null() { return r#"{"error":"no_chara"}"#.to_string(); }

    // ★ C# property types from dump.cs:
    //   Int32 Speed/Stamina/Power/Guts/Wiz/Hp/MaxHp/FanCount/Motivation/Month/Half/ScenarioId
    //   → getter returns boxed Int32, use call_getter_int
    //   ObscuredInt SkillPoint/ScenarioProgress/CharaEffectIdArray
    //   → getter returns boxed ObscuredInt struct, use call_getter_obscured_int
    // Previous bug: used call_getter_obscured_int for Int32 fields → read wrong offsets → always 0
    let spd = call_getter_int(chara_class, chara_obj, "get_Speed");
    let sta = call_getter_int(chara_class, chara_obj, "get_Stamina");
    let pow_ = call_getter_int(chara_class, chara_obj, "get_Power");
    let gut = call_getter_int(chara_class, chara_obj, "get_Guts");
    let wiz = call_getter_int(chara_class, chara_obj, "get_Wiz");
    let vit = call_getter_int(chara_class, chara_obj, "get_Hp");
    let mvit = call_getter_int(chara_class, chara_obj, "get_MaxHp");
    let mot = call_getter_int(chara_class, chara_obj, "get_Motivation");
    let spt = call_getter_obscured_int(chara_class, chara_obj, "get_SkillPoint");
    let fan = call_getter_int(chara_class, chara_obj, "get_FanCount");
    // ★ v3.18.2 fix: Month/Half are on WorkSingleModeData, not CharaData
    let mon = if !sm_class.is_null() { call_getter_int(sm_class, sm_obj, "get_Month") } else { call_getter_int(chara_class, chara_obj, "get_Month") };
    let half = if !sm_class.is_null() { call_getter_int(sm_class, sm_obj, "get_Half") } else { call_getter_int(chara_class, chara_obj, "get_Half") };
    let sid = call_getter_int(chara_class, chara_obj, "get_ScenarioId");
    let chara_effect_ids = read_obscured_int_array(chara_class, chara_obj, "get_CharaEffectIdArray");
    let effect_ids_str: Vec<String> = chara_effect_ids.iter().map(|x| x.to_string()).collect();

    let mot_s = match mot { 5=>"Best", 4=>"Good", 3=>"Normal", 2=>"Bad", 1=>"Worst", _=>"?" };
    let scn_s = match sid {
        1=>"URA", 2=>"TeamRace", 3=>"Live", 4=>"Free", 5=>"Venus",
        6=>"Arc", 7=>"Sport", 8=>"Cook", 9=>"Mecha", 10=>"Legend",
        11=>"Pioneer", 12=>"Onsen", 13=>"Breeders", 14=>"Ramen", _=>"Unknown"
    };


    // ★ v3.18.2: Pre-read Ramen CommandInfoArray gains (scenario_id == 14)
    // HomeInfoData.ParamsIncDecInfoArray is empty for Ramen scenario.
    // Real gains are in WorkSingleModeScenarioRamenDataSet.CommandInfoArray
    // → ObscuredSingleModeRamenCommandInfo.ParamsIncDecInfoArray
    // Uses same plain Int32 format as Breeders: SingleModeParamsIncDecInfo at 0x10, 0x14
    let mut ramen_gains_map: std::collections::HashMap<i32, String> = std::collections::HashMap::new();
    let mut ramen_stat_gains_map: std::collections::HashMap<i32, [i32; 5]> = std::collections::HashMap::new();
    let mut ramen_skill_pt_map: std::collections::HashMap<i32, i32> = std::collections::HashMap::new();
    let mut ramen_vital_cost_map: std::collections::HashMap<i32, i32> = std::collections::HashMap::new();
    // ★ v3.18.4: Ramen scenario-specific data for /summary
    let mut ramen_checkpoint_pt: i32 = -1;
    let mut ramen_special_feeling_num: i32 = -1;
    let mut ramen_recommend_type: i32 = -1;
    let mut ramen_feeling_info_json = String::new();
    let mut ramen_selected_region_ids_json = String::new();
    let mut ramen_active_effects_raw_json = String::new();
    if sid == 14 {
        let ramen_sc_class = find_class_by_short_name(image, "WorkSingleModeScenarioRamen");
        if !ramen_sc_class.is_null() {
            let ramen_sc_obj = try_get_scenario_obj(chara_class, chara_obj, 14);
            if !ramen_sc_obj.is_null() {
                let ramen_ds_obj = call_getter_ref(ramen_sc_class, ramen_sc_obj, "get_DataSet");
                if !ramen_ds_obj.is_null() {
                    let ramen_ds_class = find_class_by_short_name(image, "WorkSingleModeScenarioRamenDataSet");
                    if !ramen_ds_class.is_null() {
                        let ramen_cmd_arr = call_getter_on_instance(ramen_ds_class, ramen_ds_obj, "get_CommandInfoArray");
                        if !ramen_cmd_arr.is_null() {
                            let ramen_cmd_base = ramen_cmd_arr as *const u8;
                            let ramen_cmd_len = std::ptr::read_unaligned::<usize>(ramen_cmd_base.add(0x18) as *const usize);
                            if ramen_cmd_len > 0 && ramen_cmd_len < 50 {
                                let ramen_cmd_elem_class = find_class_by_short_name(image, "ObscuredSingleModeRamenCommandInfo");
                                for ri in 0..ramen_cmd_len {
                                    let re_ptr = std::ptr::read_unaligned::<*mut c_void>(ramen_cmd_base.add(0x20 + ri * 8) as *const *mut c_void);
                                    if re_ptr.is_null() { continue; }
                                    let r_cmd_id = if !ramen_cmd_elem_class.is_null() {
                                        call_getter_obscured_int(ramen_cmd_elem_class, re_ptr, "get_CommandId")
                                    } else { -1 };
                                    // Read ParamsIncDecInfoArray (plain Int32, same as Breeders)
                                    let mut r_gains = Vec::new();
                                    let mut r_stat_gains = [0i32; 5];
                                    let mut r_skill_pt = 0i32;
                                    let mut r_vital_cost = 0i32;
                                    let r_params_arr = if !ramen_cmd_elem_class.is_null() {
                                        call_getter_on_instance(ramen_cmd_elem_class, re_ptr, "get_ParamsIncDecInfoArray")
                                    } else { std::ptr::null_mut() };
                                    if !r_params_arr.is_null() {
                                        let r_pb = r_params_arr as *const u8;
                                        let r_pl = std::ptr::read_unaligned::<usize>(r_pb.add(0x18) as *const usize);
                                        if r_pl > 0 && r_pl < 100 {
                                            for rj in 0..r_pl {
                                                let r_pe = std::ptr::read_unaligned::<*mut c_void>(r_pb.add(0x20 + rj * 8) as *const *mut c_void);
                                                if r_pe.is_null() { continue; }
                                                // Plain Int32: TargetType at 0x10, Value at 0x14
                                                let r_bytes = r_pe as *const u8;
                                                let r_tt = std::ptr::read_unaligned::<i32>(r_bytes.add(0x10) as *const i32);
                                                let r_val = std::ptr::read_unaligned::<i32>(r_bytes.add(0x14) as *const i32);
                                                if r_val == 0 { continue; }
                                                let r_tn = match r_tt {
                                                    1=>"Speed", 2=>"Stamina", 3=>"Guts",
                                                    4=>"Power", 5=>"Wiz", 10=>"HP",
                                                    20=>"Motivation", 30=>"SkillPt", _=>"Unknown"
                                                };
                                                r_gains.push(format!(r#""{}":{}"#, r_tn, r_val));
                                                match r_tt {
                                                    1 => r_stat_gains[0] += r_val,
                                                    2 => r_stat_gains[1] += r_val,
                                                    4 => r_stat_gains[2] += r_val,
                                                    3 => r_stat_gains[3] += r_val,
                                                    5 => r_stat_gains[4] += r_val,
                                                    10 => r_vital_cost += r_val,
                                                    30 => r_skill_pt += r_val,
                                                    _ => {}
                                                }
                                            }
                                        }
                                    }
                                    if r_cmd_id > 0 && !r_gains.is_empty() {
                                        ramen_gains_map.insert(r_cmd_id, r_gains.join(","));
                                        ramen_stat_gains_map.insert(r_cmd_id, r_stat_gains);
                                        ramen_skill_pt_map.insert(r_cmd_id, r_skill_pt);
                                        ramen_vital_cost_map.insert(r_cmd_id, r_vital_cost);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        ura_log(3, &format!("★ Ramen gains pre-read: {} commands with gains", ramen_gains_map.len()));

        // ★ v3.18.4: Read Ramen-specific fields for /summary
        {
            let rso2 = try_get_scenario_obj(chara_class, chara_obj, 14);
            let rds_obj2 = if !rso2.is_null() { call_getter_ref(ramen_sc_class, rso2, "get_DataSet") } else { std::ptr::null_mut() };
            if !rds_obj2.is_null() {
                let rds_cls2 = find_class_by_short_name(image, "WorkSingleModeScenarioRamenDataSet");
                if !rds_cls2.is_null() {
                    ramen_checkpoint_pt = call_getter_obscured_int(rds_cls2, rds_obj2, "get_CheckPointPt");
                    ramen_special_feeling_num = call_getter_obscured_int(rds_cls2, rds_obj2, "get_SpecialFeelingNum");
                    ramen_recommend_type = call_getter_obscured_int(rds_cls2, rds_obj2, "get_RecommendType");

                    // FeelingInfoArray: コツ inventory
                    let fi_arr = call_getter_on_instance(rds_cls2, rds_obj2, "get_FeelingInfoArray");
                    if !fi_arr.is_null() {
                        let fi_base = fi_arr as *const u8;
                        let fi_len = std::ptr::read_unaligned::<usize>(fi_base.add(0x18) as *const usize);
                        if fi_len > 0 && fi_len < 100 {
                            let fi_cls = find_class_by_short_name(image, "ObscuredSingleModeRamenFeelingInfo");
                            let fi_cls = if fi_cls.is_null() { find_class_by_short_name(image, "SingleModeRamenFeelingInfo") } else { fi_cls };
                            let fi_cls = if fi_cls.is_null() { find_class_by_short_name(image, "WorkSingleModeRamenFeelingInfo") } else { fi_cls };
                            let fi_elems = if !fi_cls.is_null() {
                                read_array_element_details(fi_arr, fi_cls, &["get_FeelingType", "get_FeelingValue"], &[])
                            } else {
                                // Fallback: read raw Int32 pairs from memory
                                let mut elems = Vec::new();
                                for fi in 0..fi_len {
                                    let fe_ptr = std::ptr::read_unaligned::<*mut c_void>(fi_base.add(0x20 + fi * 8) as *const *mut c_void);
                                    if fe_ptr.is_null() { elems.push("{}".to_string()); continue; }
                                    let fe_bytes = fe_ptr as *const u8;
                                    let ft = std::ptr::read_unaligned::<i32>(fe_bytes.add(0x10) as *const i32);
                                    let fv = std::ptr::read_unaligned::<i32>(fe_bytes.add(0x14) as *const i32);
                                    elems.push(format!(r#"{{"FeelingType":{},"FeelingValue":{}}}"#, ft, fv));
                                }
                                elems
                            };
                            ramen_feeling_info_json = fi_elems.join(",");
                        }
                    }

                    // SelectedRegionIdArray
                    let sr_arr = call_getter_on_instance(rds_cls2, rds_obj2, "get_SelectedRegionIdArray");
                    if !sr_arr.is_null() {
                        let sr_base = sr_arr as *const u8;
                        let sr_len = std::ptr::read_unaligned::<usize>(sr_base.add(0x18) as *const usize);
                        if sr_len > 0 && sr_len < 50 {
                            let mut sr_ids = Vec::new();
                            for si in 0..sr_len {
                                // Int32 array: each element is 4 bytes
                                let sr_val = std::ptr::read_unaligned::<i32>(sr_base.add(0x20 + si * 4) as *const i32);
                                sr_ids.push(sr_val.to_string());
                            }
                            ramen_selected_region_ids_json = sr_ids.join(",");
                        }
                    }

                    // ActiveEffectArray (raw data for training)
                    let ae_arr2 = call_getter_on_instance(rds_cls2, rds_obj2, "get_ActiveEffectArray");
                    if !ae_arr2.is_null() {
                        let ae_base2 = ae_arr2 as *const u8;
                        let ae_len2 = std::ptr::read_unaligned::<usize>(ae_base2.add(0x18) as *const usize);
                        if ae_len2 > 0 && ae_len2 < 100 {
                            let ae_cls2 = find_class_by_short_name(image, "ObscuredSingleModeRamenActiveEffectInfo");
                            if !ae_cls2.is_null() {
                                let mut ae_elems2 = Vec::new();
                                for ai2 in 0..ae_len2 {
                                    let ae_ptr2 = std::ptr::read_unaligned::<*mut c_void>(ae_base2.add(0x20 + ai2 * 8) as *const *mut c_void);
                                    if ae_ptr2.is_null() { continue; }
                                    let cat2 = call_getter_obscured_int(ae_cls2, ae_ptr2, "get_EffectCategory");
                                    let eid2 = call_getter_obscured_int(ae_cls2, ae_ptr2, "get_EffectId");
                                    let val2 = call_getter_obscured_int(ae_cls2, ae_ptr2, "get_EffectValue");
                                    ae_elems2.push(format!(r#"{{"category":{},"id":{},"value":{}}}"#, cat2, eid2, val2));
                                }
                                ramen_active_effects_raw_json = ae_elems2.join(",");
                            }
                        }
                    }

                    // ★ UrafEffectInfo pre-read (Ramen裏風) for buffs generation
                    let uraf_cls2 = find_class_by_short_name(image, "ObscuredSingleModeRamenUrafEffectInfo");
                    if !uraf_cls2.is_null() {
                        let uraf_obj2 = call_getter_on_instance(rds_cls2, rds_obj2, "get_UrafEffectInfo");
                        if !uraf_obj2.is_null() {
                            ramen_uraf_type = call_getter_obscured_int(uraf_cls2, uraf_obj2, "get_UrafEffectType");
                            ramen_uraf_state = call_getter_obscured_int(uraf_cls2, uraf_obj2, "get_UrafEffectState");
                        }
                    }
                    ura_log(3, &format!("★ Ramen summary fields: cppt={} sfn={} rt={} fi=[{}] regions=[{}] effects=[{}] uraf={}/{}",
                        ramen_checkpoint_pt, ramen_special_feeling_num, ramen_recommend_type,
                        ramen_feeling_info_json.len(), ramen_selected_region_ids_json.len(), ramen_active_effects_raw_json.len(),
                        ramen_uraf_type, ramen_uraf_state));
            }
        }
    }

    // --- Training data via HomeInfoData (ALL scenarios) ---
    ura_log(3, "★ read_summary phase2: training data");
    let mut tr_json = "[]".to_string();
    // ★ v3.15.1: collect eval_trainings in same pass (eliminate dangerous double-read)
    let mut eval_trainings: Vec<(i32, [i32; 5], i32, i32, i32, i32, i32, i32)> = Vec::new();
    let home_info_obj = call_getter_on_instance(sm_class, sm_obj, "get_HomeInfoData");
    if !home_info_obj.is_null() {
        let hi_class = find_class_by_short_name(image, "WorkSingleModeHomeInfoData");
        if !hi_class.is_null() {
            // CommandInfoArray is a public field (not a getter), at offset 0x10
            let cmd_arr = read_field_value(hi_class, home_info_obj, "CommandInfoArray");
            if !cmd_arr.is_null() {
                let cmd_base = cmd_arr as *const u8;
                let cmd_len = std::ptr::read_unaligned::<usize>(cmd_base.add(0x18) as *const usize);
                if cmd_len > 0 && cmd_len < 100 {
                    let cmd_elem_class = find_class_by_short_name(image, "SingleModeCommandInfoData");
                    let mut trs = Vec::new();
                    for i in 0..cmd_len {
                        let ep = std::ptr::read_unaligned::<*mut c_void>(cmd_base.add(0x20 + i * 8) as *const *mut c_void);
                        if ep.is_null() { continue; }

                        let cid = if !cmd_elem_class.is_null() {
                            call_getter_obscured_int(cmd_elem_class, ep, "get_CommandId")
                        } else { -1 };
                        let cname = match cid {
                            101=>"Speed", 102=>"Stamina", 103=>"Guts",
                            105=>"Power", 106=>"Wiz",
                            601=>"Speed", 602=>"Stamina", 603=>"Guts",
                            604=>"Power", 605=>"Wiz",
                            304=>"Kakushimi",
                            301=>"Outing", 390=>"Rest", 401=>"Outing2",
                            701=>"Outing3", 801=>"Outing4", _=>"Unknown"
                        };
                        let is_enable = if !cmd_elem_class.is_null() {
                            call_getter_obscured_int(cmd_elem_class, ep, "get_IsEnable")
                        } else { -1 };
                        let failure_rate = if !cmd_elem_class.is_null() {
                            call_getter_obscured_int(cmd_elem_class, ep, "get_FailureRate")
                        } else { -1 };

                        // Heads count = TrainingPartnerArray length
                        let heads = if !cmd_elem_class.is_null() {
                            let arr = call_getter_on_instance(cmd_elem_class, ep, "get_TrainingPartnerArray");
                            if !arr.is_null() {
                                let ab = arr as *const u8;
                                let al = std::ptr::read_unaligned::<usize>(ab.add(0x18) as *const usize);
                                al as i32
                            } else { -1 }
                        } else { -1 };

                        // Shining count = TipsEventPartnerArray length
                        let shining = if !cmd_elem_class.is_null() {
                            let arr = call_getter_on_instance(cmd_elem_class, ep, "get_TipsEventPartnerArray");
                            if !arr.is_null() {
                                let ab = arr as *const u8;
                                let al = std::ptr::read_unaligned::<usize>(ab.add(0x18) as *const usize);
                                al as i32
                            } else { -1 }
                        } else { -1 };

                        // Gains from ParamsIncDecInfoArray (ObscuredInt getters)
                        let mut gains = Vec::new();
                        // ★ v3.15.1: also collect for AI eval in same pass
                        let mut stat_gains = [0i32; 5]; // [Speed, Stamina, Power, Guts, Wisdom]
                        let mut skill_pt_gain = 0i32;
                        let mut vital_cost = 0i32;
                        if !cmd_elem_class.is_null() {
                            let pa = call_getter_on_instance(cmd_elem_class, ep, "get_ParamsIncDecInfoArray");
                            if !pa.is_null() {
                                let pb = pa as *const u8;
                                let pl = std::ptr::read_unaligned::<usize>(pb.add(0x18) as *const usize);
                                if pl > 0 && pl < 100 {
                                    let pid_class = find_class_by_short_name(image, "SingleModeParamsIncDecInfoData");
                                    for j in 0..pl {
                                        let pe = std::ptr::read_unaligned::<*mut c_void>(pb.add(0x20 + j * 8) as *const *mut c_void);
                                        if pe.is_null() { continue; }
                                        let tt = if !pid_class.is_null() {
                                            call_getter_obscured_int(pid_class, pe, "get_TargetType")
                                        } else { -1 };
                                        let v = if !pid_class.is_null() {
                                            call_getter_obscured_int(pid_class, pe, "get_Value")
                                        } else { 0 };
                                        if v == 0 { continue; }
                                        let tn = match tt {
                                            1=>"Speed", 2=>"Stamina", 3=>"Guts",
                                            4=>"Power", 5=>"Wiz", 10=>"HP",
                                            20=>"Motivation", 30=>"SkillPt", _=>"Unknown"
                                        };
                                        gains.push(format!(r#""{}":{}"#, tn, v));
                                        // ★ v3.15.1: fill eval data from same read
                                        match tt {
                                            1 => stat_gains[0] += v, // Speed
                                            2 => stat_gains[1] += v, // Stamina
                                            4 => stat_gains[2] += v, // Power
                                            3 => stat_gains[3] += v, // Guts
                                            5 => stat_gains[4] += v, // Wisdom
                                            10 => vital_cost += v,    // HP
                                            30 => skill_pt_gain += v, // SkillPt
                                            _ => {}
                                        }
                                    }
                                }
                            }
                        }

                        // ★ v3.18.2: Ramen gains fallback - use pre-read CommandInfoArray gains
                        if gains.is_empty() {
                            if let Some(rg) = ramen_gains_map.get(&cid) {
                                gains.push(rg.clone());
                            }
                            if let Some(rsg) = ramen_stat_gains_map.get(&cid) {
                                stat_gains = *rsg;
                            }
                            if let Some(rsp) = ramen_skill_pt_map.get(&cid) {
                                skill_pt_gain = *rsp;
                            }
                            if let Some(rvc) = ramen_vital_cost_map.get(&cid) {
                                vital_cost = *rvc;
                            }
                        }

                        trs.push(format!(
                            r#"{{"name":"{}","command_id":{},"is_enable":{},"failure_rate":{},"heads":{},"shining":{},"gains":{{{}}}}}"#,
                            cname, cid, is_enable, failure_rate, heads, shining, gains.join(",")
                        ));

                        // ★ v3.15.1: collect eval training data in same pass
                        if cmd_id_to_train_idx(cid).is_some() {
                            eval_trainings.push((cid, stat_gains, skill_pt_gain, vital_cost, failure_rate, is_enable, shining, heads));
                        }
                    }
                    tr_json = format!("[{}]", trs.join(","));
                }
            }
        }
    }

    // --- Support cards (graceful fallback) ---
    ura_log(3, "★ read_summary phase3: support cards");
    let mut sc_json = "[]".to_string();
    let sc_arr = read_field_value(chara_class, chara_obj, "support_card_array");
    if sc_arr.is_null() {
        // Try getter
        let arr = call_getter_on_instance(chara_class, chara_obj, "get_SupportCardArray");
        if !arr.is_null() {
            let ab = arr as *const u8;
            let al = std::ptr::read_unaligned::<usize>(ab.add(0x18) as *const usize);
            if al > 0 && al < 100 {
                let mut scs = Vec::new();
                for i in 0..al {
                    let ep = std::ptr::read_unaligned::<*mut c_void>(ab.add(0x20 + i * 8) as *const *mut c_void);
                    if ep.is_null() { continue; }
                    let b = ep as *const u8;
                    let position = std::ptr::read_unaligned::<i32>(b.add(0x10) as *const i32);
                    let support_card_id = std::ptr::read_unaligned::<i32>(b.add(0x14) as *const i32);
                    let limit_break_count = std::ptr::read_unaligned::<i32>(b.add(0x18) as *const i32);
                    let training_partner_state = std::ptr::read_unaligned::<i32>(b.add(0x20) as *const i32);
                    scs.push(format!(
                        r#"{{"position":{},"support_card_id":{},"limit_break_count":{},"training_partner_state":{}}}"#,
                        position, support_card_id, limit_break_count, training_partner_state
                    ));
                }
                sc_json = format!("[{}]", scs.join(","));
            }
        }
    } else {
        let ab = sc_arr as *const u8;
        let al = std::ptr::read_unaligned::<usize>(ab.add(0x18) as *const usize);
        if al > 0 && al < 100 {
            let mut scs = Vec::new();
            for i in 0..al {
                let ep = std::ptr::read_unaligned::<*mut c_void>(ab.add(0x20 + i * 8) as *const *mut c_void);
                if ep.is_null() { continue; }
                let b = ep as *const u8;
                let position = std::ptr::read_unaligned::<i32>(b.add(0x10) as *const i32);
                let support_card_id = std::ptr::read_unaligned::<i32>(b.add(0x14) as *const i32);
                let limit_break_count = std::ptr::read_unaligned::<i32>(b.add(0x18) as *const i32);
                let training_partner_state = std::ptr::read_unaligned::<i32>(b.add(0x20) as *const i32);
                scs.push(format!(
                    r#"{{"position":{},"support_card_id":{},"limit_break_count":{},"training_partner_state":{}}}"#,
                    position, support_card_id, limit_break_count, training_partner_state
                ));
            }
            sc_json = format!("[{}]", scs.join(","));
        }
    }

    // ★ v3.18.4: Fallback - try WorkSingleModeData if support_cards still empty
    if sc_json == "[]" {
        let arr2 = read_field_value(sm_class, sm_obj, "support_card_array");
        if arr2.is_null() {
            let arr3 = call_getter_on_instance(sm_class, sm_obj, "get_SupportCardArray");
            if !arr3.is_null() {
                let ab = arr3 as *const u8;
                let al = std::ptr::read_unaligned::<usize>(ab.add(0x18) as *const usize);
                if al > 0 && al < 100 {
                    let mut scs = Vec::new();
                    for i in 0..al {
                        let ep = std::ptr::read_unaligned::<*mut c_void>(ab.add(0x20 + i * 8) as *const *mut c_void);
                        if ep.is_null() { continue; }
                        let b = ep as *const u8;
                        let position = std::ptr::read_unaligned::<i32>(b.add(0x10) as *const i32);
                        let support_card_id = std::ptr::read_unaligned::<i32>(b.add(0x14) as *const i32);
                        let limit_break_count = std::ptr::read_unaligned::<i32>(b.add(0x18) as *const i32);
                        let training_partner_state = std::ptr::read_unaligned::<i32>(b.add(0x20) as *const i32);
                        scs.push(format!(
                            r#"{{"position":{},"support_card_id":{},"limit_break_count":{},"training_partner_state":{}}}"#,
                            position, support_card_id, limit_break_count, training_partner_state
                        ));
                    }
                    if !scs.is_empty() {
                        sc_json = format!("[{}]", scs.join(","));
                        ura_log(3, &format!("★ support_cards fallback (sm_class): {} cards", scs.len()));
                    }
                }
            }
        } else {
            let ab = arr2 as *const u8;
            let al = std::ptr::read_unaligned::<usize>(ab.add(0x18) as *const usize);
            if al > 0 && al < 100 {
                let mut scs = Vec::new();
                for i in 0..al {
                    let ep = std::ptr::read_unaligned::<*mut c_void>(ab.add(0x20 + i * 8) as *const *mut c_void);
                    if ep.is_null() { continue; }
                    let b = ep as *const u8;
                    let position = std::ptr::read_unaligned::<i32>(b.add(0x10) as *const i32);
                    let support_card_id = std::ptr::read_unaligned::<i32>(b.add(0x14) as *const i32);
                    let limit_break_count = std::ptr::read_unaligned::<i32>(b.add(0x18) as *const i32);
                    let training_partner_state = std::ptr::read_unaligned::<i32>(b.add(0x20) as *const i32);
                    scs.push(format!(
                        r#"{{"position":{},"support_card_id":{},"limit_break_count":{},"training_partner_state":{}}}"#,
                        position, support_card_id, limit_break_count, training_partner_state
                    ));
                }
                if !scs.is_empty() {
                    sc_json = format!("[{}]", scs.join(","));
                    ura_log(3, &format!("★ support_cards fallback (sm_class field): {} cards", scs.len()));
                }
            }
        }
    }

    // --- Evaluation info (graceful fallback) ---
    ura_log(3, "★ read_summary phase4: evaluation");
    let mut ev_json = "[]".to_string();
    let ev_arr = read_field_value(chara_class, chara_obj, "evaluation_info_array");
    if ev_arr.is_null() {
        let arr = call_getter_on_instance(chara_class, chara_obj, "get_EvaluationInfoArray");
        if !arr.is_null() {
            let ab = arr as *const u8;
            let al = std::ptr::read_unaligned::<usize>(ab.add(0x18) as *const usize);
            if al > 0 && al < 1000 {
                let mut evs = Vec::new();
                for i in 0..al {
                    let ep = std::ptr::read_unaligned::<*mut c_void>(ab.add(0x20 + i * 8) as *const *mut c_void);
                    if ep.is_null() { continue; }
                    let b = ep as *const u8;
                    let target_id = std::ptr::read_unaligned::<i32>(b.add(0x10) as *const i32);
                    let evaluation = std::ptr::read_unaligned::<i32>(b.add(0x14) as *const i32);
                    let is_appear = std::ptr::read_unaligned::<i32>(b.add(0x20) as *const i32);
                    evs.push(format!(
                        r#"{{"target_id":{},"evaluation":{},"is_appear":{}}}"#,
                        target_id, evaluation, is_appear
                    ));
                }
                ev_json = format!("[{}]", evs.join(","));
            }
        }
    } else {
        let ab = ev_arr as *const u8;
        let al = std::ptr::read_unaligned::<usize>(ab.add(0x18) as *const usize);
        if al > 0 && al < 1000 {
            let mut evs = Vec::new();
            for i in 0..al {
                let ep = std::ptr::read_unaligned::<*mut c_void>(ab.add(0x20 + i * 8) as *const *mut c_void);
                if ep.is_null() { continue; }
                let b = ep as *const u8;
                let target_id = std::ptr::read_unaligned::<i32>(b.add(0x10) as *const i32);
                let evaluation = std::ptr::read_unaligned::<i32>(b.add(0x14) as *const i32);
                let is_appear = std::ptr::read_unaligned::<i32>(b.add(0x20) as *const i32);
                evs.push(format!(
                    r#"{{"target_id":{},"evaluation":{},"is_appear":{}}}"#,
                    target_id, evaluation, is_appear
                ));
            }
            ev_json = format!("[{}]", evs.join(","));
        }
    }

    // --- Training levels (graceful fallback) ---
    ura_log(3, "★ read_summary phase5: training_levels");
    let mut tl_json = "[]".to_string();
    let tl_arr = read_field_value(chara_class, chara_obj, "training_level_info_array");
    if tl_arr.is_null() {
        let arr = call_getter_on_instance(chara_class, chara_obj, "get_TrainingLevelInfoArray");
        if !arr.is_null() {
            let ab = arr as *const u8;
            let al = std::ptr::read_unaligned::<usize>(ab.add(0x18) as *const usize);
            if al > 0 && al < 100 {
                let mut tls = Vec::new();
                for i in 0..al {
                    let ep = std::ptr::read_unaligned::<*mut c_void>(ab.add(0x20 + i * 8) as *const *mut c_void);
                    if ep.is_null() { continue; }
                    let b = ep as *const u8;
                    let command_id = std::ptr::read_unaligned::<i32>(b.add(0x10) as *const i32);
                    let level = std::ptr::read_unaligned::<i32>(b.add(0x14) as *const i32);
                    tls.push(format!(
                        r#"{{"command_id":{},"level":{}}}"#,
                        command_id, level
                    ));
                }
                tl_json = format!("[{}]", tls.join(","));
            }
        }
    } else {
        let ab = tl_arr as *const u8;
        let al = std::ptr::read_unaligned::<usize>(ab.add(0x18) as *const usize);
        if al > 0 && al < 100 {
            let mut tls = Vec::new();
            for i in 0..al {
                let ep = std::ptr::read_unaligned::<*mut c_void>(ab.add(0x20 + i * 8) as *const *mut c_void);
                if ep.is_null() { continue; }
                let b = ep as *const u8;
                let command_id = std::ptr::read_unaligned::<i32>(b.add(0x10) as *const i32);
                let level = std::ptr::read_unaligned::<i32>(b.add(0x14) as *const i32);
                tls.push(format!(
                    r#"{{"command_id":{},"level":{}}}"#,
                    command_id, level
                ));
            }
            tl_json = format!("[{}]", tls.join(","));
        }
    }

    // --- Buffs: chara_effect_ids → readable names (ALL scenarios) + EnhanceGroup (Breeders) ---
    ura_log(3, "★ read_summary phase6: buffs");
    // ★ v3.14.2: Always generate buffs from chara_effect_ids first
    let mut buff_json = effects_to_buffs_json(&chara_effect_ids);
    let scenario_obj = try_get_scenario_obj(chara_class, chara_obj, sid);
    if !scenario_obj.is_null() {
        let sc_name = match sid {
            1=>"WorkSingleModeScenarioURA", 2=>"WorkSingleModeScenarioTeamRace",
            3=>"WorkSingleModeScenarioLive", 4=>"WorkSingleModeScenarioFree",
            5=>"WorkSingleModeScenarioVenus", 6=>"WorkSingleModeScenarioArc",
            7=>"WorkSingleModeScenarioSport", 8=>"WorkSingleModeScenarioCook",
            9=>"WorkSingleModeScenarioMecha", 10=>"WorkSingleModeScenarioLegend",
            11=>"WorkSingleModeScenarioPioneer", 12=>"WorkSingleModeScenarioOnsen",
            13=>"WorkSingleModeScenarioBreeders", 14=>"WorkSingleModeScenarioRamen",
            _=>""
        };
        if !sc_name.is_empty() {
            let sc_class = find_class_by_short_name(image, sc_name);
            if !sc_class.is_null() {
                let ds_obj = call_getter_on_instance(sc_class, scenario_obj, "get_DataSet");
                if !ds_obj.is_null() {
                    let ds_name = format!("{}DataSet", sc_name);
                    let ds_class = find_class_by_short_name(image, &ds_name);
                    if !ds_class.is_null() {
                        // ★ Breeders EnhanceGroups → override chara_effect_ids buffs
                        if sid == 13 {
                            let enhance_cls = find_class_by_short_name(image, "ObscuredSingleModeBreedersEnhanceGroup");
                            if !enhance_cls.is_null() {
                                let enhance_arr = call_getter_on_instance(ds_class, ds_obj, "get_EnhanceGroupArray");
                                if !enhance_arr.is_null() {
                                    let eb = enhance_arr as *const u8;
                                    let el = std::ptr::read_unaligned::<usize>(eb.add(0x18) as *const usize);
                                    if el > 0 && el < 20 {
                                        let mut buffs = Vec::new();
                                        for i in 0..el {
                                            let ep = std::ptr::read_unaligned::<*mut c_void>(eb.add(0x20 + i * 8) as *const *mut c_void);
                                            if ep.is_null() { continue; }
                                            let gt = call_getter_obscured_int(enhance_cls, ep, "get_GroupType");
                                            let lv = call_getter_obscured_int(enhance_cls, ep, "get_Level");
                                            let (gtn, desc) = breeders_buff_desc(gt, lv);
                                            buffs.push(format!(r#"{{"name":"{}","level":{},"desc":"{}","type":"Breeders"}}"#, gtn, lv, desc));
                                        }
                                        if !buffs.is_empty() {
                                            buff_json = format!("[{}]", buffs.join(","));
                                        }
                                    }
                                }
                            }
                        }
                        // ★ Ramen ActiveEffects → generate buffs from pre-read data
                        // (v3.18.6: Use ramen_active_effects_raw_json instead of re-reading from memory,
                        //  because call_getter_on_instance(get_DataSet) can fail in this code path)
                        if sid == 14 && !ramen_active_effects_raw_json.is_empty() {
                            // Convert raw {"category":1,"id":36,"value":50} to named buffs
                            let mut buffs = Vec::new();
                            for ae_part in ramen_active_effects_raw_json.split("},{") {
                                let mut cat: i32 = -1;
                                let mut eid: i32 = 0;
                                let mut val: i32 = 0;
                                // Simple field extraction (avoid full JSON parse in no-std)
                                for field in ae_part.trim_start_matches('{').trim_end_matches('}').split(',') {
                                    let fv: Vec<&str> = field.splitn(2, ':').collect();
                                    if fv.len() == 2 {
                                        if fv[0] == ""category"" { cat = fv[1].parse().unwrap_or(-1); }
                                        else if fv[0] == ""id"" { eid = fv[1].parse().unwrap_or(0); }
                                        else if fv[0] == ""value"" { val = fv[1].parse().unwrap_or(0); }
                                    }
                                }
                                if cat >= 0 {
                                    let cat_name = match cat {
                                        1 => "試食会", 2 => "地域", 4 => "隠し味", _ => "?",
                                    };
                                    buffs.push(format!(
                                        r#"{{"name":"{}","EffectId":{},"EffectValue":{},"type":"Ramen"}}"#,
                                        cat_name, eid, val
                                    ));
                                }
                            }
                            // Add UrafEffect from pre-read
                            if ramen_uraf_type >= 0 {
                                let ut_name = match ramen_uraf_type {
                                    1 => "試食会", 2 => "地域", 4 => "隠し味", _ => "?",
                                };
                                let state_name = match ramen_uraf_state {
                                    0 => "無効", 1 => "有効", _ => "?",
                                };
                                buffs.push(format!(r#"{{"name":"裏風:{}","UrafEffectType":{},"type":"Ramen"}}"#, ut_name, ramen_uraf_type));
                                buffs.push(format!(r#"{{"name":"裏風状態","state":"{}","UrafEffectState":{},"type":"Ramen"}}"#, state_name, ramen_uraf_state));
                            }
                            if !buffs.is_empty() {
                                buff_json = format!("[{}]", buffs.join(","));
                            }
                        }
                    }
                }
            }
        }
    }

    // ★ state field removed: get_State() doesn't exist on WorkSingleModeCharaData
    // Health condition is now detected via chara_effect_ids (top-level array)
    // ★ AI Evaluation (v3.15.1): compute score and training recommendation
    // FIXED: no more double-read of CommandInfoArray — eval_trainings collected in phase2
    let ai_json = {
        let turn = std::cmp::min((mon - 1) * 2 + (half - 1), 71);
        let stats = [spd, sta, pow_, gut, wiz];

        // Detect buffs from chara_effect_ids
        let has_ai_jiao = chara_effect_ids.iter().any(|&id| id == 8);
        let has_renshou_jouzu = chara_effect_ids.iter().any(|&id| id == 10 || id == 11);

        let result = evaluate_ai(
            turn, stats, vit, mvit, mot, sid,
            &eval_trainings, has_ai_jiao, has_renshou_jouzu,
        );
        ai_result_to_json(&result)
    };

    // ★ Breeders team member data (v3.15.4)
    let team_json = if sid == 13 {
        let team_result = read_breeders_team();
        if team_result.contains("\"team_members\"") {
            format!(r#","team_data":{}"#, team_result)
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    // ★ v3.18.4: Ramen scenario data for /summary
    let ramen_json = if sid == 14 && ramen_checkpoint_pt >= 0 {
        format!(r#","ramen":{{"checkpoint_pt":{},"special_feeling_num":{},"recommend_type":{},"feeling_info":[{}],"selected_region_ids":[{}],"active_effects":[{}]}}"#, ramen_checkpoint_pt, ramen_special_feeling_num, ramen_recommend_type, ramen_feeling_info_json, ramen_selected_region_ids_json, ramen_active_effects_raw_json)
    } else {
        String::new()
    };

    format!(
        r#"{{"version":"3.18.6","month":{},"half":{},"scenario":"{}","stats":{{"speed":{},"stamina":{},"power":{},"guts":{},"wiz":{},"vital":{},"max_vital":{},"motivation":"{}","skill_point":{},"fan":{}}},"trainings":{},"support_cards":{},"evaluation":{},"training_levels":{},"buffs":{},"chara_effect_ids":[{}],"ai":{}{}{}}}"#,
        mon, half, scn_s, spd, sta, pow_, gut, wiz, vit, mvit, mot_s, spt, fan, tr_json, sc_json, ev_json, tl_json, buff_json, effect_ids_str.join(","), ai_json, team_json, ramen_json
    )
}

// ============================================================
// HTTP Server
// ============================================================


// ============================================================
// ★ Push-to-app (v3.10.0): auto-push /summary JSON to uma-juece
// When game data changes, POST the /summary JSON to 127.0.0.1:18766/data
// The uma-juece floating window app receives and displays the data
// ============================================================

fn simple_hash(s: &str) -> u64 {
    let mut h: u64 = 5381;
    for b in s.bytes() {
        h = h.wrapping_mul(33).wrapping_add(b as u64);
    }
    h
}

fn push_to_app(json: &str) {
    use std::io::{Read, Write};
    let cfg = unsafe { get_config() };
    if !cfg.push_enabled { return; }
    let addr_str = cfg.push_addr();
    let addr: std::net::SocketAddr = match addr_str.parse() {
        Ok(a) => a,
        Err(_) => return,
    };
    let mut stream = match std::net::TcpStream::connect_timeout(
        &addr, std::time::Duration::from_secs(2)
    ) {
        Ok(s) => s,
        Err(_) => return, // App not running, that's fine
    };
    let body = json.as_bytes();
    let req = format!(
        "POST /data HTTP/1.1\r\nHost: {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        addr_str, body.len()
    );
    let _ = stream.write_all(req.as_bytes());
    let _ = stream.write_all(body);
    let _ = stream.flush();
    let mut buf = [0u8; 256];
    let _ = stream.read(&mut buf);
}

fn push_loop() {
    let interval = std::time::Duration::from_secs(unsafe { get_config() }.push_interval_secs.max(2));
    let mut consecutive_errors: u32 = 0;

    // ★ Initial push: try pushing current data on startup
    // Don't rely solely on GAME_INITIALIZED callback — it may never fire
    // if the game was already initialized before the plugin loaded.
    // Instead, try reading data; if it succeeds, the game is ready.
    for wait_round in 0..60 {
        if GAME_INITIALIZED.load(Ordering::Relaxed) { break; }
        // Try a probe read — if it doesn't error, game is ready
        let probe = read_summary();
        if !probe.contains("\"error\"") {
            GAME_INITIALIZED.store(true, Ordering::Relaxed);
            unsafe { ura_log(3, "Push: game detected via probe (no callback)"); }
            break;
        }
        if wait_round % 10 == 0 {
            unsafe { ura_log(3, &format!("Push: waiting for game... round={}", wait_round)); }
        }
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
    let init_summary = read_summary();
    if !init_summary.contains("\"error\"") {
        unsafe { LAST_PUSH_HASH = simple_hash(&init_summary); }
        push_to_app(&init_summary);
        unsafe { ura_log(3, "Push: initial data pushed"); }
    }

    loop {
        std::thread::sleep(interval);
        // Don't gate on GAME_INITIALIZED — just try reading;
        // if the game isn't ready, read_summary returns error and we skip.
        let summary = read_summary();
        if summary.contains("\"error\"") {
            consecutive_errors += 1;
            // ★ v3.14.2: backoff on consecutive errors to avoid crash loop
            if consecutive_errors > 3 {
                let backoff = std::time::Duration::from_secs((consecutive_errors as u64).min(30));
                unsafe { ura_log(3, &format!("Push: {} consecutive errors, backing off {}s", consecutive_errors, backoff.as_secs())); }
                std::thread::sleep(backoff);
            }
            continue;
        }
        consecutive_errors = 0;
        // If we got here, game is definitely ready
        if !GAME_INITIALIZED.load(Ordering::Relaxed) {
            GAME_INITIALIZED.store(true, Ordering::Relaxed);
        }
        let hash = simple_hash(&summary);
        let should_push = unsafe {
            if hash != LAST_PUSH_HASH {
                LAST_PUSH_HASH = hash;
                true
            } else {
                false
            }
        };
        if should_push {
            unsafe { ura_log(3, "Push: data changed, pushing to app"); }
            push_to_app(&summary);
        }
    }
}

fn start_http_server() {
    if HTTP_RUNNING.load(Ordering::Relaxed) { return; }
    HTTP_RUNNING.store(true, Ordering::Relaxed);
    std::thread::spawn(|| {
        unsafe { ura_log(3, "HTTP starting on 0.0.0.0:18765"); }
        let listener = match std::net::TcpListener::bind("0.0.0.0:18765") {
            Ok(l) => l,
            Err(e) => {
                unsafe { ura_log(1, &format!("HTTP bind failed: {}", e)); }
                HTTP_RUNNING.store(false, Ordering::Relaxed);
                return;
            }
        };
        unsafe { ura_log(3, "HTTP listening on :18765"); }
        unsafe { ura_notify("URA HTTP :18765 ON"); }

        // ★ Start push-to-app loop (v3.10.0)
        std::thread::spawn(|| {
            push_loop();
        });

        for stream in listener.incoming() {
            if !HTTP_RUNNING.load(Ordering::Relaxed) { break; }
            match stream {
                Ok(stream) => handle_http(stream),
                Err(_) => continue,
            }
        }
        HTTP_RUNNING.store(false, Ordering::Relaxed);
    });
}

fn parse_path(req: &str) -> String {
    let first_line = req.lines().next().unwrap_or("");
    let uri = first_line.split(' ').nth(1).unwrap_or("/");
    let path = uri.split('?').next().unwrap_or(uri);
    if path.starts_with("http://") || path.starts_with("https://") {
        if let Some(after_host) = path.splitn(4, '/').nth(3) {
            let result = if after_host.is_empty() { "/".to_string() } else { format!("/{}", after_host) };
            return result.trim_end_matches('/').to_string();
        }
        return "/".to_string();
    }
    if path.len() > 1 && path.ends_with('/') {
        path[..path.len()-1].to_string()
    } else {
        path.to_string()
    }
}


fn handle_http(mut stream: std::net::TcpStream) {
    use std::io::{Read, Write};
    let mut buf = [0u8; 8192];
    let n = match stream.read(&mut buf) { Ok(n) if n > 0 => n, _ => return };
    let req = std::str::from_utf8(&buf[..n]).unwrap_or("");
    let path = parse_path(req);

    let body = if path == "/" || path == "/health" {
        r#"{"status":"ok","version":"3.18.6","endpoints":["/summary","/data","/scenario","/debug/params","/debug/breeders","/carddb","/skilldata","/saddles","/saddles-dl","/log","/status","/health"]}"#.to_string()
    } else if path == "/scan" {
        unsafe { scan_il2cpp_classes() }
    } else if path == "/data" {
        let result = unsafe { read_training_data() };
        unsafe { log_snapshot("data", &result); }
        result
    } else if path == "/status" {
        format!(r#"{{"game_initialized":{},"http_running":{}}}"#,
            GAME_INITIALIZED.load(Ordering::Relaxed),
            HTTP_RUNNING.load(Ordering::Relaxed))
    } else if path == "/singletons" {
        unsafe { find_all_singletons() }
    } else if path.starts_with("/find_method") {
        let method_name = if path == "/find_method" || path == "/find_method/" {
            "get_SingleMode"
        } else {
            path.strip_prefix("/find_method/").unwrap_or("get_SingleMode")
        };
        unsafe { find_method_in_all_classes(method_name) }
    } else if path.starts_with("/fields") {
        let class_name = if path == "/fields" || path == "/fields/" {
            "WorkDataManager"
        } else {
            path.strip_prefix("/fields/").unwrap_or("WorkDataManager")
        };
        unsafe {
            let image = get_image();
            if image.is_null() {
                r#"{"error":"image_null"}"#.to_string()
            } else {
                let cls = find_class_by_short_name(image, class_name);
                if cls.is_null() {
                    format!(r#"{{"error":"class_not_found","name":"{}"}}"#, class_name)
                } else {
                    enumerate_class_fields(cls)
                }
            }
        }
    } else if path.starts_with("/methods") {
        let class_name = if path == "/methods" || path == "/methods/" {
            "WorkDataManager"
        } else {
            path.strip_prefix("/methods/").unwrap_or("WorkDataManager")
        };
        unsafe {
            let image = get_image();
            if image.is_null() {
                r#"{"error":"image_null"}"#.to_string()
            } else {
                let cls = find_class_by_short_name(image, class_name);
                if cls.is_null() {
                    format!(r#"{{"error":"class_not_found","name":"{}"}}"#, class_name)
                } else {
                    enumerate_class_methods(cls)
                }
            }
        }
    } else if path == "/summary" {
        read_summary()
    } else if path == "/scenario" {
        let result = unsafe { read_scenario_detail() };
        unsafe { log_snapshot("scenario", &result); }
        result
    } else if path == "/log" {
        unsafe { get_training_log() }
    } else if path == "/debug/params" {
        unsafe { debug_params_inc_dec() }
    } else if path == "/debug/breeders" {
        unsafe { debug_breeders_team() }
    } else if path == "/carddb" {
        read_carddb()
    } else if path == "/skilldata" {
        read_skilldata()
    } else if path == "/saddles-dl" {
        read_saddles()
    } else if path == "/saddles" {
        read_saddles()
    } else if path == "/config" {
        let is_post = req.starts_with("POST");
        if is_post {
            // Parse body from request
            let body_start = req.find("\r\n\r\n").map(|i| i + 4).unwrap_or(req.len());
            let post_body = &req[body_start..];
            if let Some(new_cfg) = PluginConfig::from_json(post_body) {
                let json = new_cfg.to_json();
                unsafe { update_config(new_cfg); }
                unsafe { ura_log(3, &format!("Config updated: {}", json)); }
                format!(r#"{{"ok":true,"config":{}}}"#, json)
            } else {
                r#"{"ok":false,"error":"invalid_json"}"#.to_string()
            }
        } else {
            format!(r#"{{"ok":true,"config":{}}}"#, unsafe { get_config() }.to_json())
        }
    } else if path == "/config.html" {
        // Serve a simple HTML form for config editing - open in any browser
        let cfg = unsafe { get_config() };
        let html = format!(r#"<!DOCTYPE html><html><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>URA Plugin Config</title><style>body{{font-family:system-ui;max-width:500px;margin:20px auto;padding:0 16px;background:#1a1a2e;color:#e0e0e0}}h1{{color:#4fc3f7;font-size:1.3em}}label{{display:block;margin:12px 0 4px;color:#aaa;font-size:0.85em}}input{{width:100%;padding:8px;background:#16213e;border:1px solid #333;border-radius:4px;color:#fff;box-sizing:border-box}}button{{margin-top:16px;padding:10px 24px;background:#4fc3f7;border:none;border-radius:4px;color:#000;font-weight:bold;cursor:pointer}}.ok{{color:#4caf50;margin-top:8px}}</style></head><body><h1>URA Plugin Config</h1><form id="f"><label>Push Host</label><input id="push_host" value="{}"><label>Push Port</label><input id="push_port" type="number" value="{}"><label>HTTP Port</label><input id="http_port" type="number" value="{}"><label>Push Interval (sec)</label><input id="push_interval_secs" type="number" value="{}" min="1"><label>Push Enabled</label><input id="push_enabled" type="checkbox" {}><label>HTTP Enabled</label><input id="http_enabled" type="checkbox" {}><button type="submit">Save</button></form><div id="r"></div><script>document.getElementById('f').onsubmit=async(e)=>{{e.preventDefault();const d={{push_host:push_host.value,push_port:+push_port.value,http_port:+http_port.value,push_interval_secs:+push_interval_secs.value,push_enabled:push_enabled.checked,http_enabled:http_enabled.checked}};const r=await fetch('/config',{{method:'POST',headers:{{'Content-Type':'application/json'}},body:JSON.stringify(d)}});const j=await r.json();document.getElementById('r').innerHTML=j.ok?'<p class="ok">Saved!</p>':'<p style="color:red">Error: '+j.error+'</p>';}};</script></body></html>"#,
            cfg.push_host, cfg.push_port, cfg.http_port, cfg.push_interval_secs,
            if cfg.push_enabled { "checked" } else { "" },
            if cfg.http_enabled { "checked" } else { "" }
        );
        // Return HTML with text/html content type (handled below)
        html
    } else if path.starts_with("/classes") {
        let search = if path == "/classes" || path == "/classes/" {
            ""
        } else {
            path.strip_prefix("/classes/search/").or_else(|| path.strip_prefix("/classes/")).unwrap_or("")
        };
        unsafe { enumerate_all_classes(search) }
    } else {
        format!(r#"{{"error":"not_found","path":"{}","available":["/scan","/data","/status","/health","/scenario","/log","/debug/params","/fields","/methods","/singletons","/find_method","/classes","/carddb","/skilldata","/debug/breeders","/classes/search/keyword"]}}"#, path)
    };

    if path == "/saddles-dl" {
        let resp = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Disposition: attachment; filename=\"saddles.json\"\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(), body
        );
        let _ = stream.write_all(resp.as_bytes());
    } else {
        let content_type = if body.starts_with("<!DOCTYPE") || body.starts_with("<html") { "text/html; charset=utf-8" } else { "application/json" };
        let resp = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            content_type, body.len(), body
        );
        let _ = stream.write_all(resp.as_bytes());
    }
    let _ = stream.flush();
}

// ============================================================
// Menu Callbacks
// ============================================================

extern "C" fn on_menu_item_click(_userdata: *mut c_void) {
    unsafe { ura_log(3, "URA menu item clicked"); }
}

extern "C" fn on_game_initialized(_userdata: *mut c_void) {
    GAME_INITIALIZED.store(true, Ordering::Relaxed);
    unsafe {
        ura_log(3, "Game initialized");
        ura_notify("URA: Game ready!");
    }
}

extern "C" fn on_menu_section(ui: *mut c_void, _userdata: *mut c_void) {
    unsafe {
        if API.is_null() || ui.is_null() { return; }
        let api = &*API;

        if let Some(f) = api.gui_ui_heading_fn {
            f(ui, to_cstr("URA Assistant v3.18.6").as_ptr());
        }
        if let Some(f) = api.gui_ui_separator_fn { f(ui); }

        if let Some(f) = api.gui_ui_colored_label_fn {
            if GAME_INITIALIZED.load(Ordering::Relaxed) {
                f(ui, 0, 255, 136, 255, to_cstr("Game: Connected").as_ptr());
            } else {
                f(ui, 255, 200, 0, 255, to_cstr("Game: Waiting...").as_ptr());
            }
        }

        if let Some(f) = api.gui_ui_colored_label_fn {
            if HTTP_RUNNING.load(Ordering::Relaxed) {
                f(ui, 0, 255, 136, 255, to_cstr(&format!("HTTP: Running :{}", unsafe { get_config() }.http_port)).as_ptr());
            } else {
                f(ui, 255, 80, 80, 255, to_cstr("HTTP: Failed").as_ptr());
            }
        }

        if let Some(f) = api.gui_ui_label_fn {
            f(ui, to_cstr("Data: WDM->SingleMode->Chara (getters)").as_ptr());
        }

        let c = CHARA;
        if c.valid {
            if let Some(f) = api.gui_ui_separator_fn { f(ui); }

            if let Some(f) = api.gui_ui_colored_label_fn {
                f(ui, 0, 200, 255, 255, to_cstr(&format!("Month {} | Half {} | PS:{}", c.month, c.half, c.playing_state)).as_ptr());
            }

            if let Some(f) = api.gui_ui_colored_label_fn {
                f(ui, 255, 100, 100, 255, to_cstr(&format!("SPD: {}", c.speed)).as_ptr());
            }
            if let Some(f) = api.gui_ui_colored_label_fn {
                f(ui, 100, 255, 100, 255, to_cstr(&format!("STA: {}", c.stamina)).as_ptr());
            }
            if let Some(f) = api.gui_ui_colored_label_fn {
                f(ui, 255, 200, 50, 255, to_cstr(&format!("POW: {}", c.power)).as_ptr());
            }
            if let Some(f) = api.gui_ui_colored_label_fn {
                f(ui, 255, 130, 50, 255, to_cstr(&format!("GUT: {}", c.guts)).as_ptr());
            }
            if let Some(f) = api.gui_ui_colored_label_fn {
                f(ui, 100, 180, 255, 255, to_cstr(&format!("WIZ: {}", c.wiz)).as_ptr());
            }

            if let Some(f) = api.gui_ui_label_fn {
                f(ui, to_cstr(&format!("Vital: {}/{}", c.vital, c.max_vital)).as_ptr());
            }
            if let Some(f) = api.gui_ui_colored_label_fn {
                let mot_text = match c.motivation {
                    5 => "Motivation: Best!!!",
                    4 => "Motivation: Good",
                    3 => "Motivation: Normal",
                    2 => "Motivation: Bad",
                    1 => "Motivation: Worst",
                    _ => "Motivation: ???",
                };
                let color = match c.motivation {
                    5 => (0, 255, 136),
                    4 => (100, 255, 100),
                    3 => (255, 255, 100),
                    2 => (255, 150, 50),
                    1 => (255, 50, 50),
                    _ => (200, 200, 200),
                };
                f(ui, color.0, color.1, color.2, 255, to_cstr(mot_text).as_ptr());
            }

            if let Some(f) = api.gui_ui_label_fn {
                f(ui, to_cstr(&format!("SkillPt: {} | Fan: {}", c.skill_point, c.fan_count)).as_ptr());
            }
        } else {
            if let Some(f) = api.gui_ui_label_fn {
                f(ui, to_cstr("No training data yet").as_ptr());
            }
            if let Some(f) = api.gui_ui_label_fn {
                f(ui, to_cstr("Start a training run first").as_ptr());
            }
        }

        if let Some(f) = api.gui_ui_separator_fn { f(ui); }

        // ★ Config input fields (v3.12.0): editable push_host and push_port
        {
            let cfg = unsafe { get_config() };

            // Initialize buffers from config on first frame or when config changes externally
            unsafe {
                let host_bytes = cfg.push_host.as_bytes();
                let host_len = host_bytes.len().min(63);
                if GUI_HOST_BUF_LEN == 0 && host_len > 0 {
                    GUI_HOST_BUF[..host_len].copy_from_slice(&host_bytes[..host_len]);
                    GUI_HOST_BUF[host_len] = 0;
                    GUI_HOST_BUF_LEN = host_len as i32;
                }
                let port_str = cfg.push_port.to_string();
                let port_bytes = port_str.as_bytes();
                let port_len = port_bytes.len().min(7);
                if GUI_PORT_BUF_LEN == 0 && port_len > 0 {
                    GUI_PORT_BUF[..port_len].copy_from_slice(&port_bytes[..port_len]);
                    GUI_PORT_BUF[port_len] = 0;
                    GUI_PORT_BUF_LEN = port_len as i32;
                }
            }

            // Push Host label + input
            if let Some(f) = api.gui_ui_label_fn {
                f(ui, to_cstr("Push Host:").as_ptr());
            }
            if let Some(f) = api.gui_ui_text_edit_singleline_fn {
                let changed = f(ui, unsafe { GUI_HOST_BUF.as_mut_ptr() }, 64);
                if changed {
                    unsafe {
                        // Find null terminator
                        let mut len = 0;
                        while len < 64 && GUI_HOST_BUF[len] != 0 { len += 1; }
                        GUI_HOST_BUF_LEN = len as i32;
                        if let Ok(s) = std::str::from_utf8(&GUI_HOST_BUF[..len]) {
                            let trimmed = s.trim();
                            if !trimmed.is_empty() {
                                let mut new_cfg = get_config().clone();
                                new_cfg.push_host = trimmed.to_string();
                                update_config(new_cfg);
                                ura_log(3, &format!("Config updated: push_host={}", trimmed));
                            }
                        }
                    }
                }
            }

            // Push Port label + input
            if let Some(f) = api.gui_ui_label_fn {
                f(ui, to_cstr("Push Port:").as_ptr());
            }
            if let Some(f) = api.gui_ui_text_edit_singleline_fn {
                let changed = f(ui, unsafe { GUI_PORT_BUF.as_mut_ptr() }, 8);
                if changed {
                    unsafe {
                        let mut len = 0;
                        while len < 8 && GUI_PORT_BUF[len] != 0 { len += 1; }
                        GUI_PORT_BUF_LEN = len as i32;
                        if let Ok(s) = std::str::from_utf8(&GUI_PORT_BUF[..len]) {
                            if let Ok(port) = s.trim().parse::<u16>() {
                                let mut new_cfg = get_config().clone();
                                new_cfg.push_port = port;
                                update_config(new_cfg);
                                ura_log(3, &format!("Config updated: push_port={}", port));
                            }
                        }
                    }
                }
            }
        }
    }
}

// ============================================================
// resolve_api
// ============================================================

unsafe fn resolve_api(get_api: extern "C" fn(*const c_char) -> *mut c_void) -> Api {
    macro_rules! try_api {
        ($name:expr, $ty:ty) => {{
            let cname = CString::new($name).unwrap();
            let ptr = get_api(cname.as_ptr());
            if ptr.is_null() { None } else { Some(std::mem::transmute::<*mut c_void, $ty>(ptr)) }
        }};
    }
    Api {
        log_fn: try_api!("log", unsafe extern "C" fn(i32, *const c_char, *const c_char)),
        gui_show_notification_fn: try_api!("gui_show_notification", unsafe extern "C" fn(*const c_char) -> bool),
        gui_register_menu_item_fn: try_api!("gui_register_menu_item", unsafe extern "C" fn(*const c_char, Option<extern "C" fn(*mut c_void)>, *mut c_void) -> bool),
        gui_register_menu_section_fn: try_api!("gui_register_menu_section", unsafe extern "C" fn(Option<extern "C" fn(*mut c_void, *mut c_void)>, *mut c_void) -> bool),
        hachimi_register_on_game_initialized_fn: try_api!("hachimi_register_on_game_initialized", unsafe extern "C" fn(Option<extern "C" fn(*mut c_void)>, *mut c_void) -> bool),
        gui_ui_heading_fn: try_api!("gui_ui_heading", unsafe extern "C" fn(*mut c_void, *const c_char) -> bool),
        gui_ui_label_fn: try_api!("gui_ui_label", unsafe extern "C" fn(*mut c_void, *const c_char) -> bool),
        gui_ui_colored_label_fn: try_api!("gui_ui_colored_label", unsafe extern "C" fn(*mut c_void, u8, u8, u8, u8, *const c_char) -> bool),
        gui_ui_separator_fn: try_api!("gui_ui_separator", unsafe extern "C" fn(*mut c_void) -> bool),
        gui_ui_text_edit_singleline_fn: try_api!("gui_ui_text_edit_singleline", unsafe extern "C" fn(*mut c_void, *mut c_char, i32) -> bool),
        il2cpp_get_assembly_image_fn: try_api!("il2cpp_get_assembly_image", unsafe extern "C" fn(*const c_char) -> *const c_void),
        il2cpp_get_class_fn: try_api!("il2cpp_get_class", unsafe extern "C" fn(*const c_void, *const c_char, *const c_char) -> *mut c_void),
        il2cpp_get_field_from_name_fn: try_api!("il2cpp_get_field_from_name", unsafe extern "C" fn(*mut c_void, *const c_char) -> *mut c_void),
        il2cpp_get_field_value_fn: try_api!("il2cpp_get_field_value", unsafe extern "C" fn(*const c_void, *const c_void, *mut c_void)),
        il2cpp_get_static_field_value_fn: try_api!("il2cpp_get_static_field_value", unsafe extern "C" fn(*const c_void, *mut c_void)),
        il2cpp_resolve_symbol_fn: try_api!("il2cpp_resolve_symbol", unsafe extern "C" fn(*const c_char) -> *mut c_void),
        il2cpp_get_singleton_like_instance_fn: try_api!("il2cpp_get_singleton_like_instance", unsafe extern "C" fn(*mut c_void) -> *const c_void),
        il2cpp_string_chars_fn: try_api!("il2cpp_string_chars", unsafe extern "C" fn(*const c_void) -> *mut u16),
        il2cpp_string_length_fn: try_api!("il2cpp_string_length", unsafe extern "C" fn(*const c_void) -> i32),
    }
}

// ============================================================
// hachimi_init_v3
// ============================================================

#[no_mangle]
pub unsafe extern "C" fn hachimi_init_v3(
    get_api: extern "C" fn(*const c_char) -> *mut c_void,
    version: i32,
) -> i32 {
    let api = resolve_api(get_api);
    API = Box::into_raw(Box::new(api));
    ura_log(3, "URA plugin v3.18.6 loaded (Ramen + Kakushimi + AI eval)");

    if let Some(f) = (*API).gui_show_notification_fn {
        f(to_cstr("URA v3.18.6 Loaded!").as_ptr());
    }

    if let Some(f) = (*API).gui_register_menu_item_fn {
        f(to_cstr("URA Assistant").as_ptr(), Some(on_menu_item_click), ptr::null_mut());
    }

    if let Some(f) = (*API).gui_register_menu_section_fn {
        f(Some(on_menu_section), ptr::null_mut());
    }

    if let Some(f) = (*API).hachimi_register_on_game_initialized_fn {
        f(Some(on_game_initialized), ptr::null_mut());
    }

    start_http_server();

    ura_log(3, &format!("hachimi_init_v3 done, api_version={}", version));
    InitResult::Ok as i32
}

// ============================================================
// ★ Debug: dump ParamsIncDecInfo raw memory (v3.8.0)
// Reads the first CommandInfo's ParamsIncDecInfoArray elements
// Auto-detects element class (SingleModeParamsIncDecInfo vs InfoData)
// and reads fields accordingly (plain Int32 vs ObscuredInt)
// ============================================================
unsafe fn debug_params_inc_dec() -> String {
    let image = match get_image() {
        img if !img.is_null() => img,
        _ => return r#"{"error":"image_null"}"#.to_string(),
    };

    // Get chara -> scenario -> dataset -> CommandInfoArray
    let wdm_class = find_class(image, to_cstr("Gallop").as_ptr(), to_cstr("WorkDataManager").as_ptr());
    if wdm_class.is_null() { return r#"{"error":"wdm_class_null"}"#.to_string(); }
    let wdm_inst = get_singleton(wdm_class);
    if wdm_inst.is_null() { return r#"{"error":"wdm_no_singleton"}"#.to_string(); }

    let sm_data_obj = call_getter_ref(wdm_class, wdm_inst, "get_SingleMode");
    if sm_data_obj.is_null() { return r#"{"error":"sm_data_null"}"#.to_string(); }

    let chara_data_class = find_class(image, to_cstr("Gallop").as_ptr(), to_cstr("WorkSingleModeCharaData").as_ptr());
    let chara_obj = call_getter_ref(find_class(image, to_cstr("Gallop").as_ptr(), to_cstr("WorkSingleModeData").as_ptr()), sm_data_obj, "get_Character");
    if chara_obj.is_null() { return r#"{"error":"chara_null"}"#.to_string(); }

    let scenario_id = call_getter_int(chara_data_class, chara_obj, "get_ScenarioId");
    let scenario_obj = try_get_scenario_obj(chara_data_class, chara_obj, scenario_id);
    if scenario_obj.is_null() { return r#"{"error":"scenario_obj_null"}"#.to_string(); }

    let scenario_class_name = match scenario_id {
        1 => "WorkSingleModeScenarioURA",
        2 => "WorkSingleModeScenarioTeamRace",
        3 => "WorkSingleModeScenarioLive",
        4 => "WorkSingleModeScenarioFree",
        5 => "WorkSingleModeScenarioVenus",
        6 => "WorkSingleModeScenarioArc",
        7 => "WorkSingleModeScenarioSport",
        8 => "WorkSingleModeScenarioCook",
        9 => "WorkSingleModeScenarioMecha",
        10 => "WorkSingleModeScenarioLegend",
        11 => "WorkSingleModeScenarioPioneer",
        12 => "WorkSingleModeScenarioOnsen",
        13 => "WorkSingleModeScenarioBreeders",
        14 => "WorkSingleModeScenarioRamen",
        _ => return format!(r#"{{"error":"unknown_scenario","id":{}}}"#, scenario_id),
    };
    let scenario_class = find_class_by_short_name(image, scenario_class_name);
    if scenario_class.is_null() { return format!(r#"{{"error":"scenario_class_null","name":"{}"}}"#, scenario_class_name).to_string(); }

    let dataset_obj = call_getter_on_instance(scenario_class, scenario_obj, "get_DataSet");
    if dataset_obj.is_null() { return r#"{"error":"dataset_null"}"#.to_string(); }

    let dataset_class_name = format!("{}DataSet", scenario_class_name);
    let dataset_class = find_class_by_short_name(image, &dataset_class_name);
    if dataset_class.is_null() { return format!(r#"{{"error":"dataset_class_null","name":"{}"}}"#, dataset_class_name).to_string(); }

    let cmd_elem_class = find_class_by_short_name(image, "ObscuredSingleModeBreedersCommandInfo");
    if cmd_elem_class.is_null() { return r#"{"error":"cmd_elem_class_null"}"#.to_string(); }

    let cmd_arr = call_getter_on_instance(dataset_class, dataset_obj, "get_CommandInfoArray");
    if cmd_arr.is_null() { return r#"{"error":"cmd_arr_null"}"#.to_string(); }

    let cmd_base = cmd_arr as *const u8;
    let cmd_len = std::ptr::read_unaligned::<usize>(cmd_base.add(0x18) as *const usize);
    if cmd_len == 0 { return r#"{"error":"cmd_arr_empty"}"#.to_string(); }

    // ★ Safe element type detection: read klass pointer from first element,
    //   then get class name string via il2cpp_class_get_name (no find_class_by_short_name!)
    let get_name_fn = resolve_il2cpp_symbol("il2cpp_class_get_name");

    let mut actual_elem_class_name = "unknown".to_string();
    let mut elem_is_info_type = true; // default: plain Int32 (safer for small objects)

    // Quick scan: find first command with params to detect element type
    let cmd_limit_detect = std::cmp::min(cmd_len, 5);
    'detect: for i in 0..cmd_limit_detect {
        let elem_ptr = std::ptr::read_unaligned::<*mut c_void>(cmd_base.add(0x20 + i * 8) as *const *mut c_void);
        if elem_ptr.is_null() { continue; }
        let params_arr = call_getter_on_instance(cmd_elem_class, elem_ptr, "get_ParamsIncDecInfoArray");
        if params_arr.is_null() { continue; }
        let p_base = params_arr as *const u8;
        let p_len = std::ptr::read_unaligned::<usize>(p_base.add(0x18) as *const usize);
        if p_len == 0 { continue; }
        // Read first element's klass pointer
        let first_elem = std::ptr::read_unaligned::<*mut c_void>(p_base.add(0x20) as *const *mut c_void);
        if first_elem.is_null() { continue; }
        let elem_klass = std::ptr::read_unaligned::<*mut c_void>(first_elem as *const *mut c_void);
        if elem_klass.is_null() { continue; }
        // Get class name string directly from the klass pointer
        if !get_name_fn.is_null() {
            let gn: FnClassGetName = std::mem::transmute(get_name_fn);
            let np = gn(elem_klass);
            if !np.is_null() {
                let name = std::ffi::CStr::from_ptr(np).to_string_lossy().into_owned();
                actual_elem_class_name = name.clone();
                // Compare by string name — safe, no pointer comparison needed
                if name == "SingleModeParamsIncDecInfo" {
                    elem_is_info_type = true;
                } else if name == "SingleModeParamsIncDecInfoData" {
                    elem_is_info_type = false;
                } else {
                    // Unknown class — default to plain Int32 (safer)
                    elem_is_info_type = true;
                }
            }
        }
        break 'detect;
    }

    let mut debug_items = Vec::new();

    // Only process first 3 commands max
    let cmd_limit = std::cmp::min(cmd_len, 3);
    for i in 0..cmd_limit {
        let elem_ptr = std::ptr::read_unaligned::<*mut c_void>(cmd_base.add(0x20 + i * 8) as *const *mut c_void);
        if elem_ptr.is_null() { continue; }

        let params_arr = call_getter_on_instance(cmd_elem_class, elem_ptr, "get_ParamsIncDecInfoArray");
        if params_arr.is_null() { continue; }

        let p_base = params_arr as *const u8;
        let p_len = std::ptr::read_unaligned::<usize>(p_base.add(0x18) as *const usize);
        if p_len == 0 || p_len > 20 { continue; }

        // Only first 3 params per command
        let p_limit = std::cmp::min(p_len, 3);
        for j in 0..p_limit {
            let p_elem = std::ptr::read_unaligned::<*mut c_void>(p_base.add(0x20 + j * 8) as *const *mut c_void);
            if p_elem.is_null() { continue; }

            let p_elem_bytes = p_elem as *const u8;

            // ★ Method A: ObscuredInt field XOR decryption (Data layout offsets 0x10, 0x24)
            let tt_crypto = std::ptr::read_unaligned::<i32>(p_elem_bytes.add(0x10 + 0x00) as *const i32);
            let tt_hidden = std::ptr::read_unaligned::<i32>(p_elem_bytes.add(0x10 + 0x04) as *const i32);
            let tt_decrypted = tt_hidden ^ tt_crypto;
            let val_crypto = std::ptr::read_unaligned::<i32>(p_elem_bytes.add(0x24 + 0x00) as *const i32);
            let val_hidden = std::ptr::read_unaligned::<i32>(p_elem_bytes.add(0x24 + 0x04) as *const i32);
            let val_decrypted = val_hidden ^ val_crypto;

            // ★ Method B: Plain Int32 read (Info layout: 0x10, 0x14)
            let plain_tt = std::ptr::read_unaligned::<i32>(p_elem_bytes.add(0x10) as *const i32);
            let plain_val = std::ptr::read_unaligned::<i32>(p_elem_bytes.add(0x14) as *const i32);

            // ★ Method C: Auto-detected correct reading based on element class name
            let (auto_tt, auto_val) = if elem_is_info_type {
                (plain_tt, plain_val)
            } else {
                (tt_decrypted, val_decrypted)
            };

            // ★ Raw hex dump of first 0x20 bytes (enough for both layouts)
            let mut hex_dump = String::new();
            for b in 0..0x20 {
                if b > 0 && b % 4 == 0 { hex_dump.push(' '); }
                hex_dump.push_str(&format!("{:02x}", *p_elem_bytes.add(b)));
            }

            debug_items.push(format!(
                r#"{{"cmd_idx":{},"param_idx":{},"actual_class":"{}","elem_is_info_type":{},"auto_tt":{},"auto_val":{},"plain_tt":{},"plain_val":{},"field_tt_xor":{},"field_val_xor":{},"raw":"{}"}}"#,
                i, j, actual_elem_class_name, elem_is_info_type, auto_tt, auto_val, plain_tt, plain_val, tt_decrypted, val_decrypted, hex_dump
            ));
        }
    }

    format!(r#"{{"scenario_id":{},"actual_elem_class":"{}","elem_is_info_type":{},"items":[{}]}}"#,
        scenario_id, actual_elem_class_name, elem_is_info_type, debug_items.join(","))
}

// ============================================================
// ★ Debug: Breeders scenario team member exploration (v3.15.4)
// Explores the Breeders DataSet to find team member fields
// Also reads team member data for the /summary endpoint
// ============================================================

/// Read team member data for the Breeders (Dreams) scenario
/// Returns JSON: {"team_members":[...], "team_rank":N, "dream_training_left":N}
/// Or {"error":"..."} if not available
unsafe fn read_breeders_team() -> String {
    let image = match get_image() {
        img if !img.is_null() => img,
        _ => return r#"{"error":"image_null"}"#.to_string(),
    };

    let wdm_class = find_class(image, to_cstr("Gallop").as_ptr(), to_cstr("WorkDataManager").as_ptr());
    if wdm_class.is_null() { return r#"{"error":"wdm_class_null"}"#.to_string(); }
    let wdm_inst = get_singleton(wdm_class);
    if wdm_inst.is_null() { return r#"{"error":"wdm_no_singleton"}"#.to_string(); }

    let sm_data_obj = call_getter_ref(wdm_class, wdm_inst, "get_SingleMode");
    if sm_data_obj.is_null() { return r#"{"error":"sm_data_null"}"#.to_string(); }

    let chara_class = find_class(image, to_cstr("Gallop").as_ptr(), to_cstr("WorkSingleModeCharaData").as_ptr());
    let chara_obj = call_getter_ref(find_class(image, to_cstr("Gallop").as_ptr(), to_cstr("WorkSingleModeData").as_ptr()), sm_data_obj, "get_Character");
    if chara_obj.is_null() { return r#"{"error":"chara_null"}"#.to_string(); }

    let sid = call_getter_int(chara_class, chara_obj, "get_ScenarioId");
    if sid != 13 { return format!(r#"{{"error":"not_breeders","scenario_id":{}}}"#, sid); }

    let scenario_obj = try_get_scenario_obj(chara_class, chara_obj, sid);
    if scenario_obj.is_null() { return r#"{"error":"scenario_obj_null"}"#.to_string(); }

    let sc_class = find_class_by_short_name(image, "WorkSingleModeScenarioBreeders");
    if sc_class.is_null() { return r#"{"error":"sc_class_null"}"#.to_string(); }

    let ds_obj = call_getter_on_instance(sc_class, scenario_obj, "get_DataSet");
    if ds_obj.is_null() { return r#"{"error":"dataset_null"}"#.to_string(); }

    let ds_class = find_class_by_short_name(image, "WorkSingleModeScenarioBreedersDataSet");
    if ds_class.is_null() { return r#"{"error":"ds_class_null"}"#.to_string(); }

    // ★ Read TeamRank from DataSet (ObscuredInt)
    let team_rank = call_getter_obscured_int(ds_class, ds_obj, "get_TeamRank");

    // ★ Read HavingEnhancePoint (DP) from DataSet (ObscuredInt)
    let having_dp = call_getter_obscured_int(ds_class, ds_obj, "get_HavingEnhancePoint");

    // ★ Read EnhanceGroupArray (team parameter levels)
    let mut enhance_groups_json = Vec::new();
    let enhance_elem_class = find_class_by_short_name(image, "ObscuredSingleModeBreedersEnhanceGroup");
    if !enhance_elem_class.is_null() {
        let enhance_arr = call_getter_on_instance(ds_class, ds_obj, "get_EnhanceGroupArray");
        if !enhance_arr.is_null() {
            let ebase = enhance_arr as *const u8;
            let elen = std::ptr::read_unaligned::<usize>(ebase.add(0x18) as *const usize);
            for i in 0..elen {
                let ep = std::ptr::read_unaligned::<*mut c_void>(ebase.add(0x20 + i * 8) as *const *mut c_void);
                if ep.is_null() { continue; }
                let gt = call_getter_obscured_int(enhance_elem_class, ep, "get_GroupType");
                let lv = call_getter_obscured_int(enhance_elem_class, ep, "get_Level");
                enhance_groups_json.push(format!(r#"{{"group_type":{},"level":{}}}"#, gt, lv));
            }
        }
    }

    // ★ Read TeamSpTrainingInfo for DREAMS training count (confirmed class+getters from debug!)
    let sp_train_obj = call_getter_on_instance(ds_class, ds_obj, "get_TeamSpTrainingInfo");
    let mut dream_left: i32 = -1;
    let mut dream_max: i32 = -1;
    let mut dream_activated: i32 = -1;
    let mut dream_overflow = false;
    if !sp_train_obj.is_null() {
        let sp_train_class = find_class_by_short_name(image, "ObscuredSingleModeBreedersTeamSpTrainingInfo");
        if !sp_train_class.is_null() {
            dream_left = call_getter_obscured_int(sp_train_class, sp_train_obj, "get_StockNum");
            dream_max = call_getter_obscured_int(sp_train_class, sp_train_obj, "get_StockMax");
            dream_activated = call_getter_obscured_int(sp_train_class, sp_train_obj, "get_ActivatedState");
            // v3.15.8: dream_overflow from heuristic StockNum>StockMax
            // TODO: use ChangeParameterInfo.get_IsOverflowTeamSpTrainingStock for authoritative value
        }
    }

    // ★ Read TeamMemberInfoArray from DataSet
    let member_arr = call_getter_on_instance(ds_class, ds_obj, "get_TeamMemberInfoArray");
    if member_arr.is_null() {
        return format!(
            r#"{{"error":"member_arr_null","scenario_id":13,"team_rank":{},"having_dp":{},"dream_left":{},"dream_max":{},"enhance_groups":[{}]}}"#,
            team_rank, having_dp, dream_left, dream_max, enhance_groups_json.join(",")
        );
    }

    let mb = member_arr as *const u8;
    let ml = std::ptr::read_unaligned::<usize>(mb.add(0x18) as *const usize);

    if ml == 0 || ml > 10 {
        return format!(
            r#"{{"error":"member_arr_empty","count":{},"team_rank":{},"having_dp":{},"dream_left":{},"dream_max":{},"enhance_groups":[{}]}}"#,
            ml, team_rank, having_dp, dream_left, dream_max, enhance_groups_json.join(",")
        );
    }

    // ★ Discover member element class name from runtime object header
    // Instead of guessing class names, read the klass pointer from the first element
    // and use il2cpp_class_get_name to get the actual class name
    let mut discovered_member_class_name = String::new();
    let mut member_class: *mut c_void = std::ptr::null_mut();
    {
        let first_ep = std::ptr::read_unaligned::<*mut c_void>(mb.add(0x20) as *const *mut c_void);
        if !first_ep.is_null() {
            discovered_member_class_name = get_object_class_name(first_ep);
            if !discovered_member_class_name.is_empty() {
                member_class = find_class_by_short_name(image, &discovered_member_class_name);
            }
        }
    }

    // ★ Read member data using discovered class
    let mut members_json = Vec::new();
    let mut min_level: i32 = 999;

    for i in 0..ml {
        let ep = std::ptr::read_unaligned::<*mut c_void>(mb.add(0x20 + i * 8) as *const *mut c_void);
        if ep.is_null() { continue; }

        let mut level: i32 = -1;
        let mut gauge: i32 = -1;
        let mut chara_id: i32 = 0;
        let mut exp: i32 = 0;
        let mut burst_ready = false;
        let mut found_data = false;

        if !member_class.is_null() {
            // Try all plausible getter name patterns for member fields
            // Level/Rank
            for &ln in &["get_Level", "get_Rank", "get_Grade", "get_RankLevel"] {
                let v = call_getter_obscured_int(member_class, ep, ln);
                if v >= 0 && v <= 17 { level = v; break; }
                let v2 = call_getter_int(member_class, ep, ln);
                if v2 >= 0 && v2 <= 17 { level = v2; break; }
            }
            // Dream gauge — v3.15.8: TeamMemberInfo has no gauge field (only MemberId/CharaId/Rank/Exp)
            // Gauge data lives in CommandInfo, not TeamMemberInfo; skip reading here
            // gauge stays -1 (will be clamped to 0 below)
            // Chara ID — ObscuredInt field, try obscured decoder first
            // BUG FIX v3.15.8: call_getter_int reads crypto key as plain int (returns 444444),
            // must use call_getter_obscured_int to get decrypted value
            for &cn in &["get_CharaId", "get_CharacterId", "get_CardId"] {
                let v = call_getter_obscured_int(member_class, ep, cn);
                if v > 0 { chara_id = v; break; }
                let v2 = call_getter_int(member_class, ep, cn);
                if v2 > 0 { chara_id = v2; break; }
            }
            // Exp
            for &en in &["get_Exp", "get_Experience", "get_RankExp", "get_DreamExp"] {
                let v = call_getter_obscured_int(member_class, ep, en);
                if v >= 0 { exp = v; break; }
            }
            // Burst ready — BUG FIX v3.15.8: call_getter_bool returns true on -1 (not found)
            // TeamMemberInfo has no burst field, use call_getter_int + explicit >= 0 check
            for &bn in &["get_IsBurstReady", "get_BurstReady", "get_IsBurst", "get_CanBurst"] {
                let v = call_getter_int(member_class, ep, bn);
                if v >= 0 { burst_ready = v != 0; break; }
            }

            found_data = level >= 0;
        }

        // Build hex dump as fallback
        let mut hex = String::new();
        let epb = ep as *const u8;
        for b in 0..0x80 {
            if b > 0 && b % 4 == 0 { hex.push(' '); }
            hex.push_str(&format!("{:02x}", *epb.add(b)));
        }

        if gauge < 0 { gauge = 0; }
        if level < 0 { level = 0; }
        // v3.15.8: removed gauge>=3 burst fallback (gauge not available on TeamMemberInfo)

        if level < min_level && level >= 0 { min_level = level; }

        if found_data {
            members_json.push(format!(
                r#"{{"chara_id":{},"level":{},"dream_gauge":{},"burst_ready":{},"exp":{}}}"#,
                chara_id, level, gauge, burst_ready, exp
            ));
        } else {
            // Fallback: include raw hex dump + discovered class name for analysis
            members_json.push(format!(
                r#"{{"idx":{},"chara_id":{},"level":{},"gauge":{},"burst_ready":{},"exp":{},"klass_name":"{}","raw":"{}"}}"#,
                i, chara_id, level, gauge, burst_ready, exp,
                discovered_member_class_name, hex
            ));
        }
    }

    if min_level == 999 { min_level = 0; }

    format!(
        r#"{{"team_members":[{}],"team_rank":{},"having_dp":{},"dream_left":{},"dream_max":{},"dream_overflow":{},"dream_activated":{},"enhance_groups":[{}],"member_count":{},"member_class":"{}"}}"#,
        members_json.join(","), team_rank, having_dp, dream_left, dream_max, dream_overflow,
        dream_activated, enhance_groups_json.join(","), ml, discovered_member_class_name
    )
}

unsafe fn debug_breeders_team() -> String {
    let image = match get_image() {
        img if !img.is_null() => img,
        _ => return r#"{"error":"image_null"}"#.to_string(),
    };

    let wdm_class = find_class(image, to_cstr("Gallop").as_ptr(), to_cstr("WorkDataManager").as_ptr());
    if wdm_class.is_null() { return r#"{"error":"wdm_class_null"}"#.to_string(); }
    let wdm_inst = get_singleton(wdm_class);
    if wdm_inst.is_null() { return r#"{"error":"wdm_no_singleton"}"#.to_string(); }

    let sm_data_obj = call_getter_ref(wdm_class, wdm_inst, "get_SingleMode");
    if sm_data_obj.is_null() { return r#"{"error":"sm_data_null"}"#.to_string(); }

    let chara_class = find_class(image, to_cstr("Gallop").as_ptr(), to_cstr("WorkSingleModeCharaData").as_ptr());
    let chara_obj = call_getter_ref(find_class(image, to_cstr("Gallop").as_ptr(), to_cstr("WorkSingleModeData").as_ptr()), sm_data_obj, "get_Character");
    if chara_obj.is_null() { return r#"{"error":"chara_null"}"#.to_string(); }

    let sid = call_getter_int(chara_class, chara_obj, "get_ScenarioId");

    // ★ Support both Breeders(13) and Ramen(14)
    let scenario_class_name = match sid {
        13 => "WorkSingleModeScenarioBreeders",
        14 => "WorkSingleModeScenarioRamen",
        _ => return format!(r#"{{"error":"not_supported","scenario_id":{}}}"#, sid),
    };
    let dataset_class_name = match sid {
        13 => "WorkSingleModeScenarioBreedersDataSet",
        14 => "WorkSingleModeScenarioRamenDataSet",
        _ => "Unknown",
    };

    let scenario_obj = try_get_scenario_obj(chara_class, chara_obj, sid);

    // ★ Auto-enumerate scenario-related classes
    let class_names: Vec<&str> = match sid {
        13 => vec![
            "WorkSingleModeScenarioBreeders",
            "WorkSingleModeScenarioBreedersDataSet",
            "ObscuredSingleModeBreedersMemberInfo",
            "SingleModeBreedersMemberInfo",
            "ObscuredSingleModeBreedersUnitInfo",
            "SingleModeBreedersUnitInfo",
            "ObscuredSingleModeBreedersEnhanceGroup",
            "ObscuredSingleModeBreedersCommandInfo",
            "ObscuredSingleModeBreedersTeamSpTrainingInfo",
            "WorkSingleModeChangeParameterInfoScenarioBreeders",
            "SingleModeBreedersPartnerInfo",
            "SingleModeBreedersMemberInfoData",
            "SingleModeBreedersTeamMemberInfo",
            "ObscuredSingleModeBreedersTeamMemberInfo",
        ],
        14 => vec![
            "WorkSingleModeScenarioRamen",
            "WorkSingleModeScenarioRamenDataSet",
            "ObscuredSingleModeRamenMemberInfo",
            "ObscuredSingleModeRamenCommandInfo",
            "WorkSingleModeChangeParameterInfoScenarioRamen",
        ],
        _ => vec![],
    };

    let mut class_details = Vec::new();
    for &cn in &class_names {
        let cls = find_class_by_short_name(image, cn);
        if cls.is_null() {
            class_details.push(format!(r#"{{"name":"{}","found":false}}"#, cn));
        } else {
            let methods = enumerate_class_methods(cls);
            let fields = enumerate_class_fields(cls);
            class_details.push(format!(
                r#"{{"name":"{}","found":true,"methods":{},"fields":{}}}"#,
                cn, methods, fields
            ));
        }
    }

    // ★ Try to read CharaData scenario getter
    let getter_names_map: &[&str] = match sid {
        13 => &["get_ScenarioBreeders", "get_WorkScenarioBreeders", "get_Breeders"],
        14 => &["get_ScenarioRamen", "get_WorkScenarioRamen", "get_Ramen"],
        _ => &[],
    };
    let mut getter_results = Vec::new();
    if !chara_class.is_null() && !chara_obj.is_null() {
        for &gn in getter_names_map {
            let result = call_getter_ref(chara_class, chara_obj, gn);
            getter_results.push(format!(r#"{{"name":"{}","found":{}}}"#, gn, !result.is_null()));
        }
    }

    let team_data = if sid == 13 { read_breeders_team() } else { r#"{"skip":"not_breeders"}"#.to_string() };

    // ★ Runtime member class discovery: read actual class name from TeamMemberInfoArray elements
    let mut member_class_detail = String::new();
    {
        let sc_class = find_class_by_short_name(image, scenario_class_name);
        if !sc_class.is_null() && !scenario_obj.is_null() {
            let ds_obj = call_getter_on_instance(sc_class, scenario_obj, "get_DataSet");
            if !ds_obj.is_null() {
                let ds_cls = find_class_by_short_name(image, dataset_class_name);
                if !ds_cls.is_null() {
                    let arr = call_getter_on_instance(ds_cls, ds_obj, "get_TeamMemberInfoArray");
                    if !arr.is_null() {
                        let ab = arr as *const u8;
                        let al = std::ptr::read_unaligned::<usize>(ab.add(0x18) as *const usize);
                        if al > 0 {
                            let first_ep = std::ptr::read_unaligned::<*mut c_void>(ab.add(0x20) as *const *mut c_void);
                            if !first_ep.is_null() {
                                let disc_name = get_object_class_name(first_ep);
                                if !disc_name.is_empty() {
                                    // Found the real class name! Enumerate its methods and fields
                                    let disc_class = find_class_by_short_name(image, &disc_name);
                                    if !disc_class.is_null() {
                                        let methods = enumerate_class_methods(disc_class);
                                        let fields = enumerate_class_fields(disc_class);
                                        member_class_detail = format!(
                                            r#","member_class_runtime":{{"name":"{}","found":true,"methods":{},"fields":{}}}"#,
                                            disc_name, methods, fields
                                        );
                                    } else {
                                        member_class_detail = format!(
                                            r#","member_class_runtime":{{"name":"{}","found_but_cannot_locate":true}}"#,
                                            disc_name
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    format!(
        r#"{{"scenario_id":{},"team_data":{},"class_details":[{}],"chara_getters":[{}]{} }}"#,
        sid,
        team_data,
        class_details.join(","),
        getter_results.join(","),
        member_class_detail
    )
}

/// Search for IL2CPP classes containing a keyword
unsafe fn search_classes(_keyword: &str) -> String {
    let image = match get_image() {
        img if !img.is_null() => img,
        _ => return r#"[{"error":"image_null"}]"#.to_string(),
    };

    let class_names = [
        "WorkSingleModeScenarioBreeders",
        "WorkSingleModeScenarioBreedersDataSet",
        "ObscuredSingleModeBreedersMemberInfo",
        "SingleModeBreedersMemberInfo",
        "ObscuredSingleModeBreedersUnitInfo",
        "SingleModeBreedersUnitInfo",
        "ObscuredSingleModeBreedersEnhanceGroup",
        "ObscuredSingleModeBreedersCommandInfo",
    ];

    let mut found = Vec::new();
    for &cn in &class_names {
        let cls = find_class_by_short_name(image, cn);
        if !cls.is_null() {
            found.push(format!(r#"{{"name":"{}","found":true}}"#, cn));
        } else {
            found.push(format!(r#"{{"name":"{}","found":false}}"#, cn));
        }
    }

    format!("[{}]", found.join(","))
}

// ============================================================
// ★ v3.16.1: /carddb & /skilldata — Read MasterDB via bundled rusqlite
// ============================================================


/// Find MasterDB file on the device filesystem
fn find_mdb_path() -> Option<String> {
    let paths = [
        "/data/data/jp.pokemon.pokeuma/files/master/master.mdb",
        "/data/user/0/jp.pokemon.pokeuma/files/master/master.mdb",
        "/data/data/jp.pokemon.pokeuma/files/master/master (1).mdb",
        "/data/user/0/jp.pokemon.pokeuma/files/master/master (1).mdb",
        "/storage/emulated/0/Android/data/jp.pokemon.pokeuma/files/master/master.mdb",
    ];

    for p in &paths {
        if std::path::Path::new(p).exists() {
            return Some(p.to_string());
        }
    }

    // Try to discover from /proc/self/cmdline
    if let Ok(bytes) = std::fs::read("/proc/self/cmdline") {
        let pkg = bytes.split(|&b| b == 0).filter(|s| !s.is_empty())
            .next().and_then(|s| std::str::from_utf8(s).ok());
        if let Some(pkg) = pkg {
            if !pkg.is_empty() {
                let alt_paths = [
                    format!("/data/data/{}/files/master/master.mdb", pkg),
                    format!("/data/user/0/{}/files/master/master.mdb", pkg),
                ];
                for p in &alt_paths {
                    if std::path::Path::new(p).exists() {
                        return Some(p.clone());
                    }
                }
            }
        }
    }

    None
}

fn json_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n").replace('\r', "\\r")
}

/// /carddb - Read support card data from MasterDB via rusqlite
fn read_carddb() -> String {
    let mdb_path = match find_mdb_path() {
        Some(p) => p,
        None => return r#"{"error":"mdb_not_found","hint":"MasterDB file not found on device"}"#.to_string(),
    };

    let conn = match Connection::open_with_flags(&mdb_path, OpenFlags::SQLITE_OPEN_READ_ONLY) {
        Ok(c) => c,
        Err(e) => return format!(r#"{{"error":"open_failed","detail":"{}"}}"#, e),
    };

    // Collect all card data (consumes iterator, releases borrow)
    let cards: Vec<String> = match conn.prepare(
        "SELECT id, chara_id, rarity, command_id, effect_table_id, unique_effect_id, support_card_type, outing_max FROM support_card_data ORDER BY id"
    ) {
        Ok(mut stmt) => stmt.query_map([], |row| {
            Ok(format!(
                r#"{{"id":{},"chara_id":{},"rarity":{},"command_id":{},"effect_table_id":{},"unique_effect_id":{},"support_card_type":{},"outing_max":{}}}"#,
                row.get::<_, i32>(0).unwrap_or(0),
                row.get::<_, i32>(1).unwrap_or(0),
                row.get::<_, i32>(2).unwrap_or(0),
                row.get::<_, i32>(3).unwrap_or(0),
                row.get::<_, i32>(4).unwrap_or(0),
                row.get::<_, i32>(5).unwrap_or(0),
                row.get::<_, i32>(6).unwrap_or(0),
                row.get::<_, i32>(7).unwrap_or(0),
            ))
        }).unwrap().filter_map(|r| r.ok()).collect(),
        Err(e) => return format!(r#"{{"error":"card_prepare_failed","detail":"{}"}}"#, e),
    };

    // Collect all effect data
    let effects: Vec<String> = match conn.prepare(
        "SELECT id, type, init, limit_lv5, limit_lv10, limit_lv15, limit_lv20, limit_lv25, limit_lv30, limit_lv35, limit_lv40, limit_lv45, limit_lv50 FROM support_card_effect_table ORDER BY id, type"
    ) {
        Ok(mut stmt) => stmt.query_map([], |row| {
            Ok(format!(
                r#"{{"id":{},"type":{},"init":{},"lv5":{},"lv10":{},"lv15":{},"lv20":{},"lv25":{},"lv30":{},"lv35":{},"lv40":{},"lv45":{},"lv50":{}}}"#,
                row.get::<_, i32>(0).unwrap_or(0),
                row.get::<_, i32>(1).unwrap_or(0),
                row.get::<_, i32>(2).unwrap_or(0),
                row.get::<_, i32>(3).unwrap_or(0),
                row.get::<_, i32>(4).unwrap_or(0),
                row.get::<_, i32>(5).unwrap_or(0),
                row.get::<_, i32>(6).unwrap_or(0),
                row.get::<_, i32>(7).unwrap_or(0),
                row.get::<_, i32>(8).unwrap_or(0),
                row.get::<_, i32>(9).unwrap_or(0),
                row.get::<_, i32>(10).unwrap_or(0),
                row.get::<_, i32>(11).unwrap_or(0),
                row.get::<_, i32>(12).unwrap_or(0),
            ))
        }).unwrap().filter_map(|r| r.ok()).collect(),
        Err(e) => return format!(r#"{{"error":"effect_prepare_failed","detail":"{}"}}"#, e),
    };

    drop(conn);

    format!(
        r#"{{"ok":true,"version":"3.18.6","mdb":"{}","card_count":{},"effect_count":{},"cards":[{}],"effects":[{}]}}"#,
        mdb_path, cards.len(), effects.len(), cards.join(","), effects.join(",")
    )
}

/// /skilldata - Read skill data from MasterDB via rusqlite
fn read_skilldata() -> String {
    let mdb_path = match find_mdb_path() {
        Some(p) => p,
        None => return r#"{"error":"mdb_not_found","hint":"MasterDB file not found on device"}"#.to_string(),
    };

    let conn = match Connection::open_with_flags(&mdb_path, OpenFlags::SQLITE_OPEN_READ_ONLY) {
        Ok(c) => c,
        Err(e) => return format!(r#"{{"error":"open_failed","detail":"{}"}}"#, e),
    };

    // Collect skill data
    let skills: Vec<String> = match conn.prepare(
        "SELECT id, rarity, grade_value, skill_category, condition_1, ability_type_1_1, float_ability_value_1_1, icon_id, disable_singlemode FROM skill_data ORDER BY id"
    ) {
        Ok(mut stmt) => stmt.query_map([], |row| {
            let condition: String = row.get::<_, Option<String>>(4).unwrap_or(None).unwrap_or_default();
            Ok(format!(
                r#"{{"id":{},"rarity":{},"grade_value":{},"skill_category":{},"condition":"{}","ability_type":{},"ability_value":{},"icon_id":{},"disable_sm":{}}}"#,
                row.get::<_, i32>(0).unwrap_or(0),
                row.get::<_, i32>(1).unwrap_or(0),
                row.get::<_, i32>(2).unwrap_or(0),
                row.get::<_, i32>(3).unwrap_or(0),
                json_escape(&condition),
                row.get::<_, i32>(5).unwrap_or(0),
                row.get::<_, i32>(6).unwrap_or(0),
                row.get::<_, i32>(7).unwrap_or(0),
                row.get::<_, i32>(8).unwrap_or(0),
            ))
        }).unwrap().filter_map(|r| r.ok()).collect(),
        Err(e) => return format!(r#"{{"error":"skill_prepare_failed","detail":"{}"}}"#, e),
    };

    // Collect skill names (category=47)
    let names: Vec<String> = match conn.prepare(
        "SELECT id, text FROM text_data WHERE category=47 ORDER BY id"
    ) {
        Ok(mut stmt) => stmt.query_map([], |row| {
            let text: String = row.get::<_, Option<String>>(1).unwrap_or(None).unwrap_or_default();
            Ok(format!(r#"{{"id":{},"name":"{}"}}"#,
                row.get::<_, i32>(0).unwrap_or(0),
                json_escape(&text),
            ))
        }).unwrap().filter_map(|r| r.ok()).collect(),
        Err(e) => return format!(r#"{{"error":"name_prepare_failed","detail":"{}"}}"#, e),
    };

    // Collect skill need points
    let points: Vec<String> = match conn.prepare(
        "SELECT id, need_skill_point, status_type, status_value FROM single_mode_skill_need_point ORDER BY id"
    ) {
        Ok(mut stmt) => stmt.query_map([], |row| {
            Ok(format!(
                r#"{{"id":{},"need_pt":{},"status_type":{},"status_value":{}}}"#,
                row.get::<_, i32>(0).unwrap_or(0),
                row.get::<_, i32>(1).unwrap_or(0),
                row.get::<_, i32>(2).unwrap_or(0),
                row.get::<_, i32>(3).unwrap_or(0),
            ))
        }).unwrap().filter_map(|r| r.ok()).collect(),
        Err(e) => return format!(r#"{{"error":"pt_prepare_failed","detail":"{}"}}"#, e),
    };

    drop(conn);

    format!(
        r#"{{"ok":true,"version":"3.18.6","mdb":"{}","skill_count":{},"name_count":{},"point_count":{},"skills":[{}],"names":[{}],"need_points":[{}]}}"#,
        mdb_path, skills.len(), names.len(), points.len(), skills.join(","), names.join(","), points.join(",")
    )
}

/// /saddles - Read G1 win saddle data from MasterDB for compatibility verification
/// Returns: win saddle groups with relation_group_id (5th anniversary field)
fn read_saddles() -> String {
    let mdb_path = match find_mdb_path() {
        Some(p) => p,
        None => return r#"{"error":"mdb_not_found","hint":"MasterDB file not found on device"}"#.to_string(),
    };

    let conn = match Connection::open_with_flags(&mdb_path, OpenFlags::SQLITE_OPEN_READ_ONLY) {
        Ok(c) => c,
        Err(e) => return format!(r#"{{"error":"open_failed","detail":"{}"}}"#, e),
    };

    // Collect G1 win saddles (win_saddle_type=3)
    let saddles: Vec<String> = match conn.prepare(
        "SELECT id, priority, group_id, relation_group_id, condition, win_saddle_type, race_instance_id_1, race_instance_id_2, race_instance_id_3, race_instance_id_4, race_instance_id_5, race_instance_id_6, race_instance_id_7, race_instance_id_8 FROM single_mode_wins_saddle WHERE win_saddle_type=3 ORDER BY id"
    ) {
        Ok(mut stmt) => stmt.query_map([], |row| {
            Ok(format!(
                r#"{{"id":{},"priority":{},"group_id":{},"relation_group_id":{},"condition":{},"race_ids":[{},{},{},{},{},{},{},{}]}}"#,
                row.get::<_, i32>(0).unwrap_or(0),
                row.get::<_, i32>(1).unwrap_or(0),
                row.get::<_, i32>(2).unwrap_or(0),
                row.get::<_, i32>(3).unwrap_or(0),
                row.get::<_, i32>(4).unwrap_or(0),
                row.get::<_, i32>(6).unwrap_or(0),
                row.get::<_, i32>(7).unwrap_or(0),
                row.get::<_, i32>(8).unwrap_or(0),
                row.get::<_, i32>(9).unwrap_or(0),
                row.get::<_, i32>(10).unwrap_or(0),
                row.get::<_, i32>(11).unwrap_or(0),
                row.get::<_, i32>(12).unwrap_or(0),
                row.get::<_, i32>(13).unwrap_or(0),
            ))
        }).unwrap().filter_map(|r| r.ok()).collect(),
        Err(e) => return format!(r#"{{"error":"saddle_prepare_failed","detail":"{}"}}"#, e),
    };

    // Collect chara_program (which chara runs which program_group)
    let chara_programs: Vec<String> = match conn.prepare(
        "SELECT chara_id, program_group, program_group_2 FROM single_mode_chara_program ORDER BY program_group, chara_id"
    ) {
        Ok(mut stmt) => stmt.query_map([], |row| {
            Ok(format!(
                r#"{{"chara_id":{},"program_group":{},"program_group_2":{}}}"#,
                row.get::<_, i32>(0).unwrap_or(0),
                row.get::<_, i32>(1).unwrap_or(0),
                row.get::<_, i32>(2).unwrap_or(0),
            ))
        }).unwrap().filter_map(|r| r.ok()).collect(),
        Err(e) => return format!(r#"{{"error":"program_prepare_failed","detail":"{}"}}"#, e),
    };

    // Collect program race mapping
    let programs: Vec<String> = match conn.prepare(
        "SELECT id, program_group, race_instance_id, month, half FROM single_mode_program ORDER BY program_group, month, half"
    ) {
        Ok(mut stmt) => stmt.query_map([], |row| {
            Ok(format!(
                r#"{{"id":{},"program_group":{},"race_instance_id":{},"month":{},"half":{}}}"#,
                row.get::<_, i32>(0).unwrap_or(0),
                row.get::<_, i32>(1).unwrap_or(0),
                row.get::<_, i32>(2).unwrap_or(0),
                row.get::<_, i32>(3).unwrap_or(0),
                row.get::<_, i32>(4).unwrap_or(0),
            ))
        }).unwrap().filter_map(|r| r.ok()).collect(),
        Err(e) => return format!(r#"{{"error":"prog_prepare_failed","detail":"{}"}}"#, e),
    };

    // Collect race names (category=32 = race name in text_data)
    let race_names: Vec<String> = match conn.prepare(
        "SELECT \"index\", text FROM text_data WHERE category=32 ORDER BY \"index\""
    ) {
        Ok(mut stmt) => stmt.query_map([], |row| {
            let text: String = row.get::<_, Option<String>>(1).unwrap_or(None).unwrap_or_default();
            Ok(format!(
                r#"{{"race_id":{},"name":"{}"}}"#,
                row.get::<_, i32>(0).unwrap_or(0),
                json_escape(&text),
            ))
        }).unwrap().filter_map(|r| r.ok()).collect(),
        Err(e) => return format!(r#"{{"error":"race_name_prepare_failed","detail":"{}"}}"#, e),
    };

    // Collect chara names (category=6 = chara name in text_data)
    let chara_names: Vec<String> = match conn.prepare(
        "SELECT \"index\", text FROM text_data WHERE category=6 ORDER BY \"index\""
    ) {
        Ok(mut stmt) => stmt.query_map([], |row| {
            let text: String = row.get::<_, Option<String>>(1).unwrap_or(None).unwrap_or_default();
            Ok(format!(
                r#"{{"chara_id":{},"name":"{}"}}"#,
                row.get::<_, i32>(0).unwrap_or(0),
                json_escape(&text),
            ))
        }).unwrap().filter_map(|r| r.ok()).collect(),
        Err(e) => return format!(r#"{{"error":"chara_name_prepare_failed","detail":"{}"}}"#, e),
    };

    // Collect succession_relation (fixed compatibility scores)
    let relations: Vec<String> = match conn.prepare(
        "SELECT relation_type, relation_point FROM succession_relation ORDER BY relation_type, relation_point"
    ) {
        Ok(mut stmt) => stmt.query_map([], |row| {
            Ok(format!(
                r#"{{"relation_type":{},"relation_point":{}}}"#,
                row.get::<_, i32>(0).unwrap_or(0),
                row.get::<_, i32>(1).unwrap_or(0),
            ))
        }).unwrap().filter_map(|r| r.ok()).collect(),
        Err(e) => return format!(r#"{{"error":"relation_prepare_failed","detail":"{}"}}"#, e),
    };

    // Collect succession_relation_member
    let relation_members: Vec<String> = match conn.prepare(
        "SELECT id, relation_type, chara_id FROM succession_relation_member ORDER BY relation_type, chara_id"
    ) {
        Ok(mut stmt) => stmt.query_map([], |row| {
            Ok(format!(
                r#"{{"id":{},"relation_type":{},"chara_id":{}}}"#,
                row.get::<_, i32>(0).unwrap_or(0),
                row.get::<_, i32>(1).unwrap_or(0),
                row.get::<_, i32>(2).unwrap_or(0),
            ))
        }).unwrap().filter_map(|r| r.ok()).collect(),
        Err(e) => return format!(r#"{{"error":"member_prepare_failed","detail":"{}"}}"#, e),
    };

    // Collect race_instance to race_course_set mapping (for venue info)
    let race_instances: Vec<String> = match conn.prepare(
        "SELECT ri.id, ri.race_id, r.grade, r.course_set, cs.race_track_id, cs.distance, cs.ground FROM race_instance ri JOIN race r ON ri.race_id=r.id JOIN race_course_set cs ON r.course_set=cs.id WHERE r.grade=100 ORDER BY ri.id"
    ) {
        Ok(mut stmt) => stmt.query_map([], |row| {
            Ok(format!(
                r#"{{"id":{},"race_id":{},"grade":{},"course_set":{},"race_track_id":{},"distance":{},"ground":{}}}"#,
                row.get::<_, i32>(0).unwrap_or(0),
                row.get::<_, i32>(1).unwrap_or(0),
                row.get::<_, i32>(2).unwrap_or(0),
                row.get::<_, i32>(3).unwrap_or(0),
                row.get::<_, i32>(4).unwrap_or(0),
                row.get::<_, i32>(5).unwrap_or(0),
                row.get::<_, i32>(6).unwrap_or(0),
            ))
        }).unwrap().filter_map(|r| r.ok()).collect(),
        Err(e) => return format!(r#"{{"error":"race_inst_prepare_failed","detail":"{}"}}"#, e),
    };

    drop(conn);

    format!(
        r#"{{"ok":true,"version":"3.18.6","mdb":"{}","saddle_count":{},"program_chara_count":{},"program_count":{},"race_name_count":{},"chara_name_count":{},"relation_count":{},"member_count":{},"race_instance_count":{},"saddles":[{}],"chara_programs":[{}],"programs":[{}],"race_names":[{}],"chara_names":[{}],"relations":[{}],"relation_members":[{}],"race_instances":[{}]}}"#,
        mdb_path, saddles.len(), chara_programs.len(), programs.len(),
        race_names.len(), chara_names.len(), relations.len(), relation_members.len(), race_instances.len(),
        saddles.join(","), chara_programs.join(","), programs.join(","),
        race_names.join(","), chara_names.join(","),
        relations.join(","), relation_members.join(","),
        race_instances.join(","),
    )
}
