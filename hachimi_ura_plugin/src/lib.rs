//! URA Plugin v3.4.0
//! - Updated scan class list (real IL2CPP class names from URA source)
//! - New /data endpoint: read training data via GameSystem singleton
//! - Menu panel shows chara stats when available

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

// Cached chara data for menu display
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
    let api = &*API;
    match api.il2cpp_get_assembly_image_fn {
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

// ============================================================
// Scan Classes (updated with real IL2CPP class names)
// ============================================================

unsafe fn scan_il2cpp_classes() -> String {
    if API.is_null() { return r#"{"error":"api_null"}"#.to_string(); }
    ura_log(3, "IL2CPP class scan starting...");

    let image = match get_image() {
        img if !img.is_null() => img,
        _ => return r#"{"error":"image_null"}"#.to_string(),
    };

    // Real IL2CPP class names confirmed from URA C# source + actual scan
    let classes_to_try: &[(&str, &str)] = &[
        // Core - confirmed to exist
        ("Gallop", "GameSystem"),
        ("Gallop", "SingleModeChara"),
        ("Gallop", "SingleModeHomeInfo"),
        ("Gallop", "SingleModeCommandInfo"),
        ("Gallop", "SingleModeParamsIncDecInfo"),
        ("Gallop", "SingleModeSupportCard"),
        ("Gallop", "EvaluationInfo"),
        ("Gallop", "TrainingLevelInfo"),
        ("Gallop", "SkillData"),
        // Scenes - confirmed to exist
        ("Gallop", "HomeScene"),
        ("Gallop", "SingleModeScene"),
        ("Gallop", "RaceScene"),
        // Additional - from URA source, may exist
        ("Gallop", "SingleModeCheckEventResponse"),
        ("Gallop", "SingleModeExecCommandRequest"),
        ("Gallop", "SingleModeChoiceRequest"),
        ("Gallop", "SingleModeCharaTalentLevel"),
        // Try without namespace too
        ("", "GameSystem"),
        ("", "SingleModeChara"),
        ("", "SingleModeHomeInfo"),
    ];

    let mut found_list: Vec<String> = Vec::new();
    let mut singleton_list: Vec<String> = Vec::new();

    for (ns, cls) in classes_to_try {
        let class = find_class(image, ns, cls);
        if !class.is_null() {
            let full_name = if ns.is_empty() { cls.to_string() } else { format!("{}.{}", ns, cls) };
            ura_log(3, &format!("FOUND: {}", full_name));
            found_list.push(full_name.clone());
            let inst = get_singleton(class);
            if !inst.is_null() {
                ura_log(3, &format!("{} [SINGLETON]", full_name));
                singleton_list.push(full_name);
            }
        }
    }

    let result = format!(
        r#"{{"found_classes":["{}"],"singletons":["{}"],"total":{}}}"#,
        found_list.join("\",\""), singleton_list.join("\",\""), found_list.len()
    );
    ura_log(3, &format!("Scan done: {} classes found", found_list.len()));
    result
}

// ============================================================
// Read Training Data via GameSystem singleton
// ============================================================

unsafe fn read_training_data() -> String {
    if API.is_null() { return r#"{"game_system_ok":false,"chara":null,"error":"api_null"}"#.to_string(); }
    ura_log(3, "Reading training data...");

    let image = match get_image() {
        img if !img.is_null() => img,
        _ => return r#"{"game_system_ok":false,"chara":null,"error":"image_null"}"#.to_string(),
    };

    // 1. Get GameSystem class + singleton
    let gs_class = find_class(image, "Gallop", "GameSystem");
    if gs_class.is_null() {
        return r#"{"game_system_ok":false,"chara":null,"error":"no_GameSystem_class"}"#.to_string();
    }

    let gs_inst = get_singleton(gs_class);
    if gs_inst.is_null() {
        return r#"{"game_system_ok":false,"chara":null,"error":"no_GameSystem_singleton"}"#.to_string();
    }

    ura_log(3, "GameSystem singleton OK, scanning fields...");

    // 2. Try to find SingleModeChara reference on GameSystem
    // Try various field name patterns
    let chara_field_candidates: &[&str] = &[
        // Most specific first
        "_singleModeChara", "singleModeChara", "SingleModeChara",
        "_chara", "chara", "Chara",
        "_currentChara", "currentChara",
        "_mainChara", "mainChara",
        // Then controller/manager patterns
        "_singleModeController", "singleModeController",
        "_singleModeManager", "singleModeManager",
        "_singleModeData", "singleModeData",
        // Home info
        "_homeInfo", "homeInfo", "_singleModeHomeInfo", "singleModeHomeInfo",
        // Generic
        "_data", "Data", "data",
    ];

    // Get SingleModeChara class for field reading
    let chara_class = find_class(image, "Gallop", "SingleModeChara");

    let mut found_ref: *const c_void = ptr::null();
    let mut found_via = String::new();
    let mut found_fields: Vec<String> = Vec::new();

    for candidate in chara_field_candidates {
        let field = match (*API).il2cpp_get_field_from_name_fn {
            Some(fn_ptr) => {
                let name_c = to_cstr(candidate);
                fn_ptr(gs_class, name_c.as_ptr())
            }
            None => continue,
        };
        if !field.is_null() {
            found_fields.push(candidate.to_string());
            ura_log(3, &format!("GameSystem field found: {}", candidate));

            // Try to read the reference value
            let mut value: *const c_void = ptr::null();
            if let Some(fn_ptr) = (*API).il2cpp_get_field_value_fn {
                fn_ptr(gs_inst as *mut c_void, field, &mut value as *mut *const c_void as *mut c_void);
            }
            if !value.is_null() && found_ref.is_null() {
                found_ref = value;
                found_via = candidate.to_string();
                ura_log(3, &format!("Found non-null ref via: {} = 0x{:x}", candidate, value as usize));
            }
        }
    }

    // 3. If we found a reference, try to read SingleModeChara fields from it
    if !found_ref.is_null() {
        let ref_class = if !chara_class.is_null() {
            chara_class
        } else {
            // If SingleModeChara class not found, use GameSystem class
            // (the ref might be a different type)
            gs_class
        };

        let int_fields: &[(&str, &str)] = &[
            ("speed", "speed"), ("stamina", "stamina"), ("power", "power"),
            ("wiz", "wiz"), ("guts", "guts"),
            ("vital", "vital"), ("max_vital", "maxVital"),
            ("motivation", "motivation"), ("turn", "turn"),
            ("skill_point", "skillPoint"), ("scenario_id", "scenarioId"),
            ("playing_state", "playingState"),
        ];

        // Try both _prefix and no-prefix variants for each field
        let mut chara_json_parts: Vec<String> = Vec::new();
        let mut cache = CharaCache {
            speed: 0, stamina: 0, power: 0, wiz: 0, guts: 0,
            vital: 0, max_vital: 0, motivation: 0, turn: 0,
            skill_point: 0, scenario_id: 0, playing_state: 0,
            valid: false,
        };

        for (json_key, il_name) in int_fields {
            // Try with underscore prefix first (common IL2CPP convention)
            let val = read_field_int(found_ref, ref_class, &format!("_{}", il_name));
            let val = if val == 0 {
                // Try without prefix
                read_field_int(found_ref, ref_class, il_name)
            } else {
                val
            };
            chara_json_parts.push(format!(r#""{}":{}"#, json_key, val));

            // Update cache
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

        // Check if any value is non-zero (validate the ref is actually useful)
        let any_nonzero = cache.speed > 0 || cache.stamina > 0 || cache.power > 0
            || cache.wiz > 0 || cache.guts > 0 || cache.turn > 0;

        cache.valid = any_nonzero;
        CHARA = cache;

        if any_nonzero {
            ura_log(3, &format!("Chara data valid: SPD={} STA={} POW={} WIZ={} GUT={} VIT={}/{} MOT={} TURN={}",
                cache.speed, cache.stamina, cache.power, cache.wiz, cache.guts,
                cache.vital, cache.max_vital, cache.motivation, cache.turn));
            format!(
                r#"{{"game_system_ok":true,"chara":{{{}}},"found_via":"{}","fields_on_gs":["{}"],"error":null}}"#,
                chara_json_parts.join(","), found_via, found_fields.join("\",\"")
            )
        } else {
            ura_log(2, &format!("Ref found via {} but all fields zero - wrong type?", found_via));
            format!(
                r#"{{"game_system_ok":true,"chara":{{{}}},"found_via":"{}","fields_on_gs":["{}"],"warning":"all_fields_zero","error":null}}"#,
                chara_json_parts.join(","), found_via, found_fields.join("\",\"")
            )
        }
    } else {
        // No reference found - return field list for debugging
        if found_fields.is_empty() {
            // No fields found at all - try enumerating with il2cpp_class_get_methods pattern
            // Since we don't have il2cpp_class_get_fields, try more field names
            let extra_fields: &[&str] = &[
                "Instance", "_instance", "instance",
                "Current", "_current", "current",
                "Value", "_value", "value",
                "Self", "_self",
                "Owner", "_owner",
                "Controller", "_controller",
                "Manager", "_manager",
                "State", "_state",
                "Model", "_model",
                "Context", "_context",
            ];
            for f in extra_fields {
                let field = match (*API).il2cpp_get_field_from_name_fn {
                    Some(fn_ptr) => {
                        let name_c = to_cstr(f);
                        fn_ptr(gs_class, name_c.as_ptr())
                    }
                    None => continue,
                };
                if !field.is_null() {
                    found_fields.push(f.to_string());
                }
            }
        }

        format!(
            r#"{{"game_system_ok":true,"chara":null,"fields_on_gs":["{}"],"error":"no_chara_ref"}}"#,
            found_fields.join("\",\"")
        )
    }
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

fn handle_http(mut stream: std::net::TcpStream) {
    use std::io::{Read, Write};
    let mut buf = [0u8; 4096];
    let n = match stream.read(&mut buf) { Ok(n) if n > 0 => n, _ => return };
    let req = std::str::from_utf8(&buf[..n]).unwrap_or("");
    let path = req.split(' ').nth(1).unwrap_or("/");

    let body = match path {
        "/" | "/health" => r#"{"status":"ok","version":"3.4.0","endpoints":["/scan","/data","/status","/health"]}"#.to_string(),
        "/scan" => {
            unsafe { scan_il2cpp_classes() }
        }
        "/data" => {
            unsafe { read_training_data() }
        }
        "/status" => {
            format!(r#"{{"game_initialized":{},"http_running":{}}}"#,
                GAME_INITIALIZED.load(Ordering::Relaxed),
                HTTP_RUNNING.load(Ordering::Relaxed))
        }
        _ => r#"{"error":"not_found"}"#.to_string(),
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
    // Important: callback params are (ui, userdata), not (userdata, ui)!
    unsafe {
        if API.is_null() || ui.is_null() { return; }
        let api = &*API;

        // Heading
        if let Some(f) = api.gui_ui_heading_fn {
            f(ui, to_cstr("URA Assistant v3.4.0").as_ptr());
        }
        if let Some(f) = api.gui_ui_separator_fn { f(ui); }

        // Game status
        if let Some(f) = api.gui_ui_colored_label_fn {
            if GAME_INITIALIZED.load(Ordering::Relaxed) {
                f(ui, 0, 255, 136, 255, to_cstr("Game: Connected").as_ptr());
            } else {
                f(ui, 255, 200, 0, 255, to_cstr("Game: Waiting...").as_ptr());
            }
        }

        // HTTP status
        if let Some(f) = api.gui_ui_colored_label_fn {
            if HTTP_RUNNING.load(Ordering::Relaxed) {
                f(ui, 0, 255, 136, 255, to_cstr("HTTP: Running :18765").as_ptr());
            } else {
                f(ui, 255, 80, 80, 255, to_cstr("HTTP: Failed").as_ptr());
            }
        }

        // Chara data display (from cache)
        let c = CHARA;
        if c.valid {
            if let Some(f) = api.gui_ui_separator_fn { f(ui); }

            // Five dimensions with colors
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

            // Vital + Motivation
            if let Some(f) = api.gui_ui_label_fn {
                f(ui, to_cstr(&format!("Vital: {}/{}", c.vital, c.max_vital)).as_ptr());
            }
            // Motivation: 1=絶不調 2=不調 3=普通 4=好調 5=絶好調
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

        // Endpoints
        if let Some(f) = api.gui_ui_separator_fn { f(ui); }
        if let Some(f) = api.gui_ui_label_fn {
            f(ui, to_cstr("Scan: 127.0.0.1:18765/scan").as_ptr());
        }
        if let Some(f) = api.gui_ui_label_fn {
            f(ui, to_cstr("Data: 127.0.0.1:18765/data").as_ptr());
        }
    }
}

// ============================================================
// resolve_api - unchanged from v3.3.4 (all signatures verified)
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
// hachimi_init_v3 - Plugin Entry Point
// ============================================================

#[no_mangle]
pub unsafe extern "C" fn hachimi_init_v3(
    get_api: extern "C" fn(*const c_char) -> *mut c_void,
    version: i32,
) -> i32 {
    let api = resolve_api(get_api);
    API = Box::into_raw(Box::new(api));
    ura_log(3, "URA plugin v3.4.0 loaded");

    if let Some(f) = (*API).gui_show_notification_fn {
        f(to_cstr("URA v3.4.0 Loaded!").as_ptr());
    }

    // Register menu_item WITH callback - clickable tab
    if let Some(f) = (*API).gui_register_menu_item_fn {
        f(to_cstr("URA Assistant").as_ptr(), Some(on_menu_item_click), ptr::null_mut());
    }

    // Register menu_section - content panel (only 2 params: callback, userdata)
    if let Some(f) = (*API).gui_register_menu_section_fn {
        f(Some(on_menu_section), ptr::null_mut());
    }

    if let Some(f) = (*API).hachimi_register_on_game_initialized_fn {
        f(Some(on_game_initialized), ptr::null_mut());
    }

    // Start HTTP immediately (don't wait for game_initialized)
    start_http_server();

    ura_log(3, &format!("hachimi_init_v3 done, api_version={}", version));
    I