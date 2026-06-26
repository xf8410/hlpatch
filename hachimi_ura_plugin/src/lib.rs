//! URA Plugin v3.4.4
//! - BREAKTHROUGH: Use il2cpp_runtime_invoke to call getter methods!
//! - /fields confirmed GameSystem has NO _workSingleModeCharaData field
//! - Data is accessed via get_WorkSingleModeCharaData() getter method
//! - Added /methods endpoint to enumerate class methods
//! - Generalized /fields to support any class name
//! - Falls back to field approach if method invocation fails

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

#[derive(Copy, Clone)]
struct CharaCache {
    speed: i32, stamina: i32, power: i32, wiz: i32, guts: i32,
    vital: i32, max_vital: i32, motivation: i32, turn: i32,
    skill_point: i32, scenario_id: i32, playing_state: i32,
    valid: bool,
}

static mut CHARA: CharaCache = CharaCache {
    speed: 0, stamina: 0, power: 0, wiz: 0, guts: 0,
    vital: 0, max_vital: 0, motivation: 0, turn: 0,
    skill_point: 0, scenario_id: 0, playing_state: 0,
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

unsafe fn find_class(image: *const c_void, ns: &str, name: &str) -> *mut c_void {
    if image.is_null() || API.is_null() { return ptr::null_mut(); }
    match (*API).il2cpp_get_class_fn {
        Some(fn_ptr) => {
            let ns_c = to_cstr(ns);
            let name_c = to_cstr(name);
            fn_ptr(image, ns_c.as_ptr(), name_c.as_ptr())
        }
        None => ptr::null_mut(),
    }
}

/// Find a class by trying multiple known namespaces
unsafe fn find_class_by_short_name(image: *const c_void, class_name: &str) -> *mut c_void {
    let namespaces: &[&str] = &[
        "Gallop",
        "Gallop.WorkSingleModeCharaData",
        "",
    ];
    for ns in namespaces {
        let cls = find_class(image, ns, class_name);
        if !cls.is_null() { return cls; }
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

unsafe fn read_field_int(obj: *const c_void, class: *mut c_void, field_name: &str) -> i32 {
    if obj.is_null() || class.is_null() || API.is_null() { return 0; }
    let field = match (*API).il2cpp_get_field_from_name_fn {
        Some(fn_ptr) => {
            let name_c = to_cstr(field_name);
            fn_ptr(class, name_c.as_ptr())
        }
        None => return 0,
    };
    if field.is_null() { return 0; }
    let mut value: i32 = 0;
    match (*API).il2cpp_get_field_value_fn {
        Some(fn_ptr) => fn_ptr(obj as *mut c_void, field, &mut value as *mut i32 as *mut c_void),
        None => return 0,
    }
    value
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

unsafe fn read_field_int_auto(obj: *const c_void, class: *mut c_void, base_name: &str) -> i32 {
    let v1 = read_field_int(obj, class, &format!("_{}", base_name));
    if v1 != 0 { return v1; }
    read_field_int(obj, class, base_name)
}

unsafe fn read_field_ptr_auto(obj: *const c_void, class: *mut c_void, base_name: &str) -> *const c_void {
    let v1 = read_field_ptr(obj, class, &format!("_{}", base_name));
    if !v1.is_null() { return v1; }
    read_field_ptr(obj, class, base_name)
}

// ============================================================
// IL2CPP Runtime API types for method invocation
// ============================================================

type FnClassGetFields = unsafe extern "C" fn(*mut c_void, *mut *mut c_void) -> *mut Il2CppFieldInfo;
type FnClassGetParent = unsafe extern "C" fn(*mut c_void) -> *mut c_void;
type FnClassGetName = unsafe extern "C" fn(*mut c_void) -> *const c_char;
type FnClassGetMethodFromName = unsafe extern "C" fn(*mut c_void, *const c_char, i32) -> *const c_void;
type FnRuntimeInvoke = unsafe extern "C" fn(*const c_void, *mut c_void, *mut *mut c_void, *mut *mut c_void) -> *mut c_void;
type FnClassGetMethods = unsafe extern "C" fn(*mut c_void, *mut *mut c_void) -> *const c_void;
type FnMethodGetName = unsafe extern "C" fn(*const c_void) -> *const c_char;

#[repr(C)]
struct Il2CppFieldInfo {
    name: *const c_char,
    _ty: *const c_void,
    parent: *mut c_void,
    offset: i32,
    _token: u32,
}

/// Resolve an IL2CPP runtime function by symbol name
unsafe fn resolve_il2cpp_fn<T>(name: &str) -> Option<T> {
    if API.is_null() { return None; }
    match (*API).il2cpp_resolve_symbol_fn {
        Some(resolve) => {
            let cname = to_cstr(name);
            let ptr = resolve(cname.as_ptr());
            if ptr.is_null() {
                ura_log(2, &format!("resolve_il2cpp_fn: {} not found", name));
                None
            } else {
                ura_log(3, &format!("resolve_il2cpp_fn: {} OK", name));
                Some(std::mem::transmute::<*mut c_void, T>(ptr))
            }
        }
        None => None,
    }
}

// ============================================================
// Call getter method via il2cpp_runtime_invoke
// ============================================================

/// Call a getter method (0 params) on an object instance.
/// Returns the result object pointer, or null if failed.
unsafe fn call_getter_on_instance(
    class: *mut c_void,
    instance: *const c_void,
    method_name: &str,
) -> *mut c_void {
    if class.is_null() || instance.is_null() {
        ura_log(1, "call_getter: null class or instance");
        return ptr::null_mut();
    }

    let get_method_fn: Option<FnClassGetMethodFromName> = resolve_il2cpp_fn("il2cpp_class_get_method_from_name");
    let invoke_fn: Option<FnRuntimeInvoke> = resolve_il2cpp_fn("il2cpp_runtime_invoke");

    if get_method_fn.is_none() {
        ura_log(1, "call_getter: il2cpp_class_get_method_from_name unavailable");
        return ptr::null_mut();
    }
    if invoke_fn.is_none() {
        ura_log(1, "call_getter: il2cpp_runtime_invoke unavailable");
        return ptr::null_mut();
    }

    // Find the method
    let method_name_c = to_cstr(method_name);
    let method_info = get_method_fn.unwrap()(class, method_name_c.as_ptr(), 0);
    if method_info.is_null() {
        ura_log(2, &format!("call_getter: method '{}' not found on class", method_name));
        return ptr::null_mut();
    }
    ura_log(3, &format!("call_getter: found method '{}'", method_name));

    // Invoke the method
    let mut exc: *mut c_void = ptr::null_mut();
    let result = invoke_fn.unwrap()(
        method_info,
        instance as *mut c_void,
        ptr::null_mut(),
        &mut exc,
    );

    if !exc.is_null() {
        ura_log(1, &format!("call_getter: method '{}' threw exception", method_name));
        return ptr::null_mut();
    }

    if result.is_null() {
        ura_log(2, &format!("call_getter: method '{}' returned null (not in training mode?)", method_name));
    } else {
        ura_log(3, &format!("call_getter: '{}' returned {:p}", method_name, result));
    }

    result
}

// ============================================================
// Scan Classes
// ============================================================

unsafe fn scan_il2cpp_classes() -> String {
    if API.is_null() { return r#"{"error":"api_null"}"#.to_string(); }

    let image = match get_image() {
        img if !img.is_null() => img,
        _ => return r#"{"error":"image_null"}"#.to_string(),
    };

    let classes_to_try: &[(&str, &str)] = &[
        ("Gallop", "GameSystem"),
        ("Gallop", "WorkSingleModeCharaData"),
        ("Gallop", "WorkSingleModeHomeInfo"),
        ("Gallop", "WorkSingleModeData"),
        ("Gallop", "WorkSingleModeScenarioBreeders"),
        ("Gallop", "WorkSingleModeScenarioLegend"),
        ("Gallop", "WorkSingleModeScenarioMecha"),
        ("Gallop", "WorkSingleModeScenarioOnsen"),
        ("Gallop", "WorkSingleModeScenarioPioneer"),
        ("Gallop", "WorkSingleModeScenarioRamen"),
        ("Gallop", "SingleModeCharaLight"),
        ("Gallop", "HomeScene"),
        ("Gallop", "SingleModeScene"),
        ("Gallop", "RaceScene"),
        ("Gallop", "SingleModeSceneController"),
        ("Gallop", "SingleModePlayingState"),
        ("", "WorkSingleModeCharaData"),
        ("", "WorkSingleModeData"),
    ];

    let mut found_list: Vec<String> = Vec::new();
    let mut singleton_list: Vec<String> = Vec::new();

    for (ns, cls) in classes_to_try {
        let class = find_class(image, ns, cls);
        if !class.is_null() {
            let full_name = if ns.is_empty() { cls.to_string() } else { format!("{}.{}", ns, cls) };
            found_list.push(full_name.clone());
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
// Read Training Data - v3.4.4 with method invocation
// ============================================================

unsafe fn read_training_data() -> String {
    if API.is_null() { return r#"{"game_system_ok":false,"chara":null,"error":"api_null"}"#.to_string(); }
    ura_log(3, "Reading training data v3.4.4...");

    let image = match get_image() {
        img if !img.is_null() => img,
        _ => return r#"{"game_system_ok":false,"chara":null,"error":"image_null"}"#.to_string(),
    };

    // 1. Get GameSystem singleton
    let gs_class = find_class(image, "Gallop", "GameSystem");
    if gs_class.is_null() {
        return r#"{"game_system_ok":false,"chara":null,"error":"no_GameSystem_class"}"#.to_string();
    }
    let gs_inst = get_singleton(gs_class);
    if gs_inst.is_null() {
        return r#"{"game_system_ok":false,"chara":null,"error":"no_GameSystem_singleton"}"#.to_string();
    }

    let chara_data_class = find_class(image, "Gallop", "WorkSingleModeCharaData");
    let sm_data_class = find_class(image, "Gallop", "WorkSingleModeData");
    let home_info_class = find_class(image, "Gallop", "WorkSingleModeHomeInfo");

    let mut chara_obj: *mut c_void = ptr::null_mut();
    let mut chara_via = String::new();
    let mut methods_tried: Vec<String> = Vec::new();

    // ===== Strategy A: Call get_WorkSingleModeCharaData directly =====
    let getter_names: &[&str] = &[
        "get_WorkSingleModeCharaData",
        "GetWorkSingleModeCharaData",
        "get_WorkSingleModeData",
        "GetWorkSingleModeData",
        "get_WorkSingleMode",
        "GetWorkSingleMode",
    ];

    for getter_name in getter_names {
        methods_tried.push(getter_name.to_string());
        let result = call_getter_on_instance(gs_class, gs_inst, getter_name);
        if !result.is_null() {
            if getter_name.contains("CharaData") {
                // Direct hit!
                chara_obj = result;
                chara_via = getter_name.to_string();
                ura_log(3, &format!("GOT chara data via {}!", getter_name));
                break;
            } else if getter_name.contains("SingleModeData") || getter_name.contains("SingleMode") {
                // Got WorkSingleMode or WorkSingleModeData - try drilling into it
                ura_log(3, &format!("Got {} result, trying to find chara data inside...", getter_name));
                let drill_class = if !sm_data_class.is_null() { sm_data_class } else { gs_class };
                let inner = read_field_ptr_auto(result as *const c_void, drill_class, "workSingleModeCharaData");
                if !inner.is_null() {
                    chara_obj = inner as *mut c_void;
                    chara_via = format!("{}->workSingleModeCharaData", getter_name);
                    ura_log(3, "Got chara data via indirect path!");
                    break;
                }
                // Also try _charaData
                let inner2 = read_field_ptr_auto(result as *const c_void, drill_class, "charaData");
                if !inner2.is_null() {
                    chara_obj = inner2 as *mut c_void;
                    chara_via = format!("{}->charaData", getter_name);
                    ura_log(3, "Got chara data via charaData path!");
                    break;
                }
            }
        }
    }

    // ===== Strategy B: Field-based fallback (old approach) =====
    if chara_obj.is_null() {
        ura_log(2, "Getter approach failed, trying field-based fallback...");
        let field_candidates: &[&str] = &[
            "_workSingleModeCharaData", "workSingleModeCharaData",
            "_workSingleModeData", "workSingleModeData",
            "_singleModeData", "singleModeData",
        ];
        for c in field_candidates {
            let val = read_field_ptr_auto(gs_inst, gs_class, c);
            if !val.is_null() {
                chara_obj = val as *mut c_void;
                chara_via = format!("field:{}", c);
                ura_log(3, &format!("Found via field fallback: {}", c));
                break;
            }
        }
    }

    // ===== Strategy C: Try SingleModeScene controller =====
    if chara_obj.is_null() {
        let sm_scene_class = find_class(image, "Gallop", "SingleModeScene");
        if !sm_scene_class.is_null() {
            let sm_scene_inst = call_getter_on_instance(gs_class, gs_inst, "get_SingleModeScene");
            if sm_scene_inst.is_null() {
                // Try singleton
                let inst = get_singleton(sm_scene_class);
                if !inst.is_null() {
                    let inner = read_field_ptr_auto(inst, sm_scene_class, "workSingleModeCharaData");
                    if !inner.is_null() {
                        chara_obj = inner as *mut c_void;
                        chara_via = "SingleModeScene->workSingleModeCharaData".to_string();
                    }
                }
            } else {
                let inner = read_field_ptr_auto(sm_scene_inst as *const c_void, sm_scene_class, "workSingleModeCharaData");
                if !inner.is_null() {
                    chara_obj = inner as *mut c_void;
                    chara_via = "get_SingleModeScene->workSingleModeCharaData".to_string();
                }
            }
        }
    }

    // ===== Read fields from chara data object =====
    if !chara_obj.is_null() {
        let ref_class = if !chara_data_class.is_null() { chara_data_class } else { gs_class };

        let int_fields: &[(&str, &str)] = &[
            ("speed", "speed"),
            ("stamina", "stamina"),
            ("power", "power"),
            ("wiz", "wiz"),
            ("guts", "guts"),
            ("vital", "vital"),
            ("max_vital", "maxVital"),
            ("motivation", "motivation"),
            ("turn", "turn"),
            ("skill_point", "skillPoint"),
            ("scenario_id", "scenarioId"),
            ("playing_state", "playingState"),
        ];

        let mut chara_json_parts: Vec<String> = Vec::new();
        let mut cache = CharaCache {
            speed: 0, stamina: 0, power: 0, wiz: 0, guts: 0,
            vital: 0, max_vital: 0, motivation: 0, turn: 0,
            skill_point: 0, scenario_id: 0, playing_state: 0,
            valid: false,
        };

        for (json_key, il_name) in int_fields {
            let val = read_field_int_auto(chara_obj as *const c_void, ref_class, il_name);
            chara_json_parts.push(format!(r#""{}":{}"#, json_key, val));
            match *json_key {
                "speed" => cache.speed = val,
                "stamina" => cache.stamina = val,
                "power" => cache.power = val,
                "wiz" => cache.wiz = val,
                "guts" => cache.guts = val,
                "vital" => cache.vital = val,
                "max_vital" => cache.max_vital = val,
                "motivation" => cache.motivation = val,
                "turn" => cache.turn = val,
                "skill_point" => cache.skill_point = val,
                "scenario_id" => cache.scenario_id = val,
                "playing_state" => cache.playing_state = val,
                _ => {}
            }
        }

        let any_nonzero = cache.speed > 0 || cache.stamina > 0 || cache.power > 0
            || cache.wiz > 0 || cache.guts > 0 || cache.turn > 0;

        cache.valid = any_nonzero;
        CHARA = cache;

        if any_nonzero {
            ura_log(3, &format!("Chara OK: SPD={} STA={} POW={} WIZ={} GUT={} VIT={}/{} MOT={} TURN={}",
                cache.speed, cache.stamina, cache.power, cache.wiz, cache.guts,
                cache.vital, cache.max_vital, cache.motivation, cache.turn));
            format!(
                r#"{{"game_system_ok":true,"chara":{{{}}},"found_via":"{}","chara_class":{},"home_info_class":{},"error":null}}"#,
                chara_json_parts.join(","),
                chara_via,
                if !chara_data_class.is_null() { "true" } else { "false" },
                if !home_info_class.is_null() { "true" } else { "false" },
            )
        } else {
            ura_log(2, &format!("Got chara obj via {} but all fields zero", chara_via));
            format!(
                r#"{{"game_system_ok":true,"chara":{{{}}},"found_via":"{}","warning":"all_fields_zero","chara_class":{},"error":null}}"#,
                chara_json_parts.join(","),
                chara_via,
                if !chara_data_class.is_null() { "true" } else { "false" },
            )
        }
    } else {
        // Nothing found
        format!(
            r#"{{"game_system_ok":true,"chara":null,"methods_tried":["{}"],"chara_class":{},"sm_data_class":{},"home_info_class":{},"error":"no_chara_ref"}}"#,
            methods_tried.join("\",\""),
            if !chara_data_class.is_null() { "true" } else { "false" },
            if !sm_data_class.is_null() { "true" } else { "false" },
            if !home_info_class.is_null() { "true" } else { "false" },
        )
    }
}

// ============================================================
// Enumerate ALL fields including parent classes
// ============================================================

unsafe fn enumerate_class_fields(class: *mut c_void) -> String {
    if class.is_null() || API.is_null() { return r#"{"error":"null_class"}"#.to_string(); }

    let get_fields_fn: Option<FnClassGetFields> = resolve_il2cpp_fn("il2cpp_class_get_fields");
    let get_parent_fn: Option<FnClassGetParent> = resolve_il2cpp_fn("il2cpp_class_get_parent");
    let get_class_name_fn: Option<FnClassGetName> = resolve_il2cpp_fn("il2cpp_class_get_name");

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

    let get_methods_fn: Option<FnClassGetMethods> = resolve_il2cpp_fn("il2cpp_class_get_methods");
    let get_method_name_fn: Option<FnMethodGetName> = resolve_il2cpp_fn("il2cpp_method_get_name");
    let get_parent_fn: Option<FnClassGetParent> = resolve_il2cpp_fn("il2cpp_class_get_parent");
    let get_class_name_fn: Option<FnClassGetName> = resolve_il2cpp_fn("il2cpp_class_get_name");

    if get_methods_fn.is_none() {
        return r#"{"error":"no_il2cpp_class_get_methods"}"#.to_string();
    }

    let mut all_methods: Vec<String> = Vec::new();
    let mut current_class = class;
    let mut depth = 0;
    let max_methods = 200;

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

            // Filter: only show get_/set_/.ctor methods to keep output manageable
            if method_name.starts_with("get_") || method_name.starts_with("set_") || method_name == ".ctor" || method_name.contains("SingleMode") || method_name.contains("Work") {
                all_methods.push(format!(r#"{{"name":"{}","class":"{}"}}"#, method_name, class_name));
            }
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
    let mut buf = [0u8; 4096];
    let n = match stream.read(&mut buf) { Ok(n) if n > 0 => n, _ => return };
    let req = std::str::from_utf8(&buf[..n]).unwrap_or("");
    let path = parse_path(req);

    let body = if path == "/" || path == "/health" {
        r#"{"status":"ok","version":"3.4.4","endpoints":["/scan","/data","/status","/health","/fields","/fields/ClassName","/methods","/methods/ClassName"]}"#.to_string()
    } else if path == "/scan" {
        unsafe { scan_il2cpp_classes() }
    } else if path == "/data" {
        unsafe { read_training_data() }
    } else if path == "/status" {
        format!(r#"{{"game_initialized":{},"http_running":{}}}"#,
            GAME_INITIALIZED.load(Ordering::Relaxed),
            HTTP_RUNNING.load(Ordering::Relaxed))
    } else if path.starts_with("/fields") {
        let class_name = if path == "/fields" || path == "/fields/" {
            "GameSystem"
        } else {
            path.strip_prefix("/fields/").unwrap_or("GameSystem")
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
            "GameSystem"
        } else {
            path.strip_prefix("/methods/").unwrap_or("GameSystem")
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
    } else {
        format!(r#"{{"error":"not_found","path":"{}","available":["/scan","/data","/status","/health","/fields","/methods"]}}"#, path)
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
            f(ui, to_cstr("URA Assistant v3.4.4").as_ptr());
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

        let c = CHARA;
        if c.valid {
            if let Some(f) = api.gui_ui_separator_fn { f(ui); }

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
                f(ui, 100, 180, 255, 255, to_cstr(&format!("WIZ: {}", c.wiz)).as_ptr());
            }
            if let Some(f) = api.gui_ui_colored_label_fn {
                f(ui, 255, 130, 50, 255, to_cstr(&format!("GUT: {}", c.guts)).as_ptr());
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
                f(ui, to_cstr(&format!("Turn: {}  SkillPt: {}", c.turn, c.skill_point)).as_ptr());
            }
        }

        if let Some(f) = api.gui_ui_separator_fn { f(ui); }
        if let Some(f) = api.gui_ui_label_fn {
            f(ui, to_cstr("Data: 127.0.0.1:18765/data").as_ptr());
        }
        if let Some(f) = api.gui_ui_label_fn {
            f(ui, to_cstr("Methods: 127.0.0.1:18765/methods").as_ptr());
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
    ura_log(3, "URA plugin v3.4.4 loaded");

    if let Some(f) = (*API).gui_show_notification_fn {
        f(to_cstr("URA v3.4.4 Loaded!").as_ptr());
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
