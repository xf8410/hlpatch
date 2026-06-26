//! URA Plugin v3.4.1
//! - FIXED: Real IL2CPP class names (WorkSingleModeCharaData, not SingleModeChara!)
//! - FIXED: URL path parsing (handle full URL and query params)
//! - Updated scan class list with all Work* data classes
//! - /data endpoint reads training data via GameSystem singleton

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

// Read field int, trying both _prefix and no-prefix variants
unsafe fn read_field_int_auto(obj: *const c_void, class: *mut c_void, base_name: &str) -> i32 {
    let v1 = read_field_int(obj, class, &format!("_{}", base_name));
    if v1 != 0 { return v1; }
    read_field_int(obj, class, base_name)
}

// Read field ptr, trying both _prefix and no-prefix variants
unsafe fn read_field_ptr_auto(obj: *const c_void, class: *mut c_void, base_name: &str) -> *const c_void {
    let v1 = read_field_ptr(obj, class, &format!("_{}", base_name));
    if !v1.is_null() { return v1; }
    read_field_ptr(obj, class, base_name)
}

// ============================================================
// Scan Classes - REAL IL2CPP class names from metadata
// ============================================================

unsafe fn scan_il2cpp_classes() -> String {
    if API.is_null() { return r#"{"error":"api_null"}"#.to_string(); }
    ura_log(3, "IL2CPP class scan starting...");

    let image = match get_image() {
        img if !img.is_null() => img,
        _ => return r#"{"error":"image_null"}"#.to_string(),
    };

    // Real IL2CPP class names confirmed from global-metadata.dat
    // Key discovery: game uses "Work" prefix for data classes!
    let classes_to_try: &[(&str, &str)] = &[
        // Core singletons
        ("Gallop", "GameSystem"),
        // *** THE KEY CLASS: WorkSingleModeCharaData (NOT SingleModeChara!) ***
        ("Gallop", "WorkSingleModeCharaData"),
        // Training commands
        ("Gallop", "WorkSingleModeHomeInfo"),
        // Parent data container
        ("Gallop", "WorkSingleModeData"),
        // Scenario-specific data classes
        ("Gallop", "WorkSingleModeScenarioBreeders"),
        ("Gallop", "WorkSingleModeScenarioLegend"),
        ("Gallop", "WorkSingleModeScenarioMecha"),
        ("Gallop", "WorkSingleModeScenarioOnsen"),
        ("Gallop", "WorkSingleModeScenarioPioneer"),
        ("Gallop", "WorkSingleModeScenarioRamen"),
        // Nested types in WorkSingleModeCharaData
        ("Gallop.WorkSingleModeCharaData", "EquipSupportCard"),
        ("Gallop.WorkSingleModeCharaData", "SkillTips"),
        ("Gallop.WorkSingleModeCharaData", "Evaluation"),
        ("Gallop.WorkSingleModeCharaData", "SuccessionFactorInfo"),
        ("Gallop.WorkSingleModeCharaData", "SuccessionCharaInfo"),
        ("Gallop.WorkSingleModeCharaData", "GuestOutingInfo"),
        // Lightweight version
        ("Gallop", "SingleModeCharaLight"),
        // Scenes
        ("Gallop", "HomeScene"),
        ("Gallop", "SingleModeScene"),
        ("Gallop", "RaceScene"),
        // Controller
        ("Gallop", "SingleModeSceneController"),
        // Playing state
        ("Gallop", "SingleModePlayingState"),
        // Also try without namespace (in case namespace is empty)
        ("", "WorkSingleModeCharaData"),
        ("", "WorkSingleModeData"),
        ("", "WorkSingleModeHomeInfo"),
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
// Read Training Data
// ============================================================

unsafe fn read_training_data() -> String {
    if API.is_null() { return r#"{"game_system_ok":false,"chara":null,"error":"api_null"}"#.to_string(); }
    ura_log(3, "Reading training data...");

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

    ura_log(3, "GameSystem singleton OK, looking for chara data...");

    // 2. Try to find WorkSingleModeCharaData reference through various paths
    // Path A: GameSystem -> _workSingleModeCharaData (direct field)
    // Path B: GameSystem -> _singleModeData -> _workSingleModeCharaData
    // Path C: GameSystem -> some controller -> _workSingleModeCharaData

    let chara_data_class = find_class(image, "Gallop", "WorkSingleModeCharaData");
    let sm_data_class = find_class(image, "Gallop", "WorkSingleModeData");
    let home_info_class = find_class(image, "Gallop", "WorkSingleModeHomeInfo");

    // Field name candidates for finding chara data on GameSystem
    let chara_field_candidates: &[&str] = &[
        // Direct reference to WorkSingleModeCharaData
        "_workSingleModeCharaData", "workSingleModeCharaData",
        // Reference via WorkSingleModeData
        "_workSingleModeData", "workSingleModeData",
        "_singleModeData", "singleModeData",
        // SingleModeCharaData (non-Work variant)
        "_singleModeCharaData", "singleModeCharaData",
        // Controller/manager paths
        "_singleModeSceneController", "singleModeSceneController",
        "_singleModeController", "singleModeController",
        "_singleModeManager", "singleModeManager",
        // Generic
        "_charaData", "charaData",
        "_data", "data",
        "_currentChara", "currentChara",
        "_chara", "chara",
    ];

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

    // 3. If we found a reference, try to read fields from it
    if !found_ref.is_null() {
        // Try reading as WorkSingleModeCharaData first
        let ref_class = if !chara_data_class.is_null() {
            chara_data_class
        } else {
            gs_class // fallback
        };

        // WorkSingleModeCharaData fields (from metadata analysis)
        // speed, stamina, power, guts, wiz - five dimensions
        // vital, maxVital - stamina gauge
        // motivation - 干劲
        // turn - current turn
        // skillPoint - skill points
        // scenarioId - current scenario
        // playingState - playing state
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
            let val = read_field_int_auto(found_ref, ref_class, il_name);
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

        // Also try to drill into WorkSingleModeData if that's what we found
        // WorkSingleModeData might have _workSingleModeCharaData inside it
        if found_via.contains("singleModeData") && !chara_data_class.is_null() {
            let inner_ref = read_field_ptr_auto(found_ref, ref_class, "workSingleModeCharaData");
            if !inner_ref.is_null() {
                ura_log(3, "Found WorkSingleModeCharaData inside WorkSingleModeData!");
                // Re-read all fields from the inner object
                for (json_key, il_name) in int_fields {
                    let val = read_field_int_auto(inner_ref, chara_data_class, il_name);
                    chara_json_parts = chara_json_parts.into_iter().enumerate().map(|(i, s)| {
                        if i / 2 < int_fields.len() && int_fields[i / 2].0 == *json_key {
                            format!(r#""{}":{}"#, json_key, val)
                        } else {
                            s
                        }
                    }).collect();

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
                found_ref = inner_ref;
                found_via = format!("{}->workSingleModeCharaData", found_via);
            }
        }

        let any_nonzero = cache.speed > 0 || cache.stamina > 0 || cache.power > 0
            || cache.wiz > 0 || cache.guts > 0 || cache.turn > 0;

        cache.valid = any_nonzero;
        CHARA = cache;

        if any_nonzero {
            ura_log(3, &format!("Chara data valid: SPD={} STA={} POW={} WIZ={} GUT={} VIT={}/{} MOT={} TURN={}",
                cache.speed, cache.stamina, cache.power, cache.wiz, cache.guts,
                cache.vital, cache.max_vital, cache.motivation, cache.turn));
            format!(
                r#"{{"game_system_ok":true,"chara":{{{}}},"found_via":"{}","ref_class":"{}","fields_on_gs":["{}"],"error":null}}"#,
                chara_json_parts.join(","),
                found_via,
                if !chara_data_class.is_null() { "WorkSingleModeCharaData" } else { "GameSystem" },
                found_fields.join("\",\"")
            )
        } else {
            ura_log(2, &format!("Ref found via {} but all fields zero", found_via));
            format!(
                r#"{{"game_system_ok":true,"chara":{{{}}},"found_via":"{}","ref_class":"{}","fields_on_gs":["{}"],"warning":"all_fields_zero","error":null}}"#,
                chara_json_parts.join(","),
                found_via,
                if !chara_data_class.is_null() { "WorkSingleModeCharaData" } else { "GameSystem" },
                found_fields.join("\",\"")
            )
        }
    } else {
        // No reference found - return diagnostic info
        // Also try scanning SingleModeScene singleton
        let sm_scene_class = find_class(image, "Gallop", "SingleModeScene");
        let mut scene_fields: Vec<String> = Vec::new();
        if !sm_scene_class.is_null() {
            let sm_scene_inst = get_singleton(sm_scene_class);
            if !sm_scene_inst.is_null() {
                let scene_candidates: &[&str] = &[
                    "_workSingleModeCharaData", "workSingleModeCharaData",
                    "_workSingleModeData", "workSingleModeData",
                    "_charaData", "charaData",
                    "_data", "data",
                    "_controller", "controller",
                ];
                for c in scene_candidates {
                    let field = match (*API).il2cpp_get_field_from_name_fn {
                        Some(fn_ptr) => {
                            let name_c = to_cstr(c);
                            fn_ptr(sm_scene_class, name_c.as_ptr())
                        }
                        None => continue,
                    };
                    if !field.is_null() {
                        scene_fields.push(c.to_string());
                    }
                }
            }
        }

        format!(
            r#"{{"game_system_ok":true,"chara":null,"gs_fields":["{}"],"scene_fields":["{}"],"sm_data_class":{},"home_info_class":{},"chara_data_class":{},"error":"no_chara_ref"}}"#,
            found_fields.join("\",\""),
            scene_fields.join("\",\""),
            if !sm_data_class.is_null() { "true" } else { "false" },
            if !home_info_class.is_null() { "true" } else { "false" },
            if !chara_data_class.is_null() { "true" } else { "false" },
        )
    }
}

// ============================================================
// HTTP Server - with fixed path parsing
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

/// Parse the path from an HTTP request, handling:
/// - "GET /data HTTP/1.1" -> "/data"
/// - "GET http://127.0.0.1:18765/data HTTP/1.1" -> "/data" (proxy-style)
/// - Strips query params: "/data?foo=bar" -> "/data"
fn parse_path(req: &str) -> &str {
    // Get the request line (first line)
    let first_line = req.lines().next().unwrap_or("");
    // Split into parts: METHOD URI PROTOCOL
    let uri = first_line.split(' ').nth(1).unwrap_or("/");
    // Strip query params
    let path = uri.split('?').next().unwrap_or(uri);
    // Handle full URL (proxy-style requests)
    if path.starts_with("http://") || path.starts_with("https://") {
        // Extract path from full URL
        if let Some(after_host) = path.splitn(4, '/').nth(3) {
            return after_host;
        }
    }
    path
}

fn handle_http(mut stream: std::net::TcpStream) {
    use std::io::{Read, Write};
    let mut buf = [0u8; 4096];
    let n = match stream.read(&mut buf) { Ok(n) if n > 0 => n, _ => return };
    let req = std::str::from_utf8(&buf[..n]).unwrap_or("");
    let path = parse_path(req);

    let body = match path {
        "/" | "/health" => r#"{"status":"ok","version":"3.4.1","endpoints":["/scan","/data","/status","/health"]}"#.to_string(),
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
        _ => r#"{"error":"not_found","available":["/scan","/data","/status","/health"]}"#.to_string(),
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
            f(ui, to_cstr("URA Assistant v3.4.1").as_ptr());
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

        // Chara data from cache
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
    ura_log(3, "URA plugin v3.4.1 loaded");

    if let Some(f) = (*API).gui_show_notification_fn {
        f(to_cstr("URA v3.4.1 Loaded!").as_ptr());
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
