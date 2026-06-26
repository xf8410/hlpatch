//! URA Plugin v3.7.9
//! ★ v3.7.9: Fix Value=115 bug + training log + debug/params raw memory dump
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
                                            let bytes = p_elem as *const u8;
                                            let t = std::ptr::read_unaligned::<i32>(bytes.add(0x10) as *const i32);
                                            let v = std::ptr::read_unaligned::<i32>(bytes.add(0x14) as *const i32);
                                            let (tt, val) = (t, v);
                                            params_items.push(format!(r#"{{"TargetType":{},"Value":{}}}"#, tt, val));
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
// HTTP Server
// ============================================================

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
        r#"{"status":"ok","version":"3.8.1","fix":"Breeders_hardcode_plain_Int32+safe_class_name_detect","data_path":"WorkDataManager->get_SingleMode->get_Character->get_Speed()","endpoints":["/scan","/data","/status","/health","/scenario","/log","/debug/params","/fields","/fields/ClassName","/methods","/methods/ClassName","/singletons","/find_method/methodName","/classes","/classes/search/keyword"]}"#.to_string()
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
    } else if path == "/scenario" {
        let result = unsafe { read_scenario_detail() };
        unsafe { log_snapshot("scenario", &result); }
        result
    } else if path == "/log" {
        unsafe { get_training_log() }
    } else if path == "/debug/params" {
        unsafe { debug_params_inc_dec() }
    } else if path.starts_with("/classes") {
        let search = if path == "/classes" || path == "/classes/" {
            ""
        } else {
            path.strip_prefix("/classes/search/").or_else(|| path.strip_prefix("/classes/")).unwrap_or("")
        };
        unsafe { enumerate_all_classes(search) }
    } else {
        format!(r#"{{"error":"not_found","path":"{}","available":["/scan","/data","/status","/health","/scenario","/log","/debug/params","/fields","/methods","/singletons","/find_method","/classes","/classes/search/keyword"]}}"#, path)
    };

    let resp = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(), body
    );
    let _ = stream.write_all(resp.as_bytes());
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
            f(ui, to_cstr("URA Assistant v3.7.9").as_ptr());
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
                f(ui, 0, 255, 136, 255, to_cstr("HTTP: Running :18765").as_ptr());
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
        if let Some(f) = api.gui_ui_label_fn {
            f(ui, to_cstr("127.0.0.1:18765/data").as_ptr());
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
    ura_log(3, "URA plugin v3.7.8 loaded (scenario data + export)");

    if let Some(f) = (*API).gui_show_notification_fn {
        f(to_cstr("URA v3.7.8 Loaded!").as_ptr());
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