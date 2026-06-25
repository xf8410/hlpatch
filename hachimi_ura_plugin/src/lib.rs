//! URA Plugin v3.3.0
//! Hachimi Edge V3 API - IL2CPP exploration + HTTP Server

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
    gui_ui_button_fn: Option<unsafe extern "C" fn(*mut c_void, *const c_char) -> bool>,
    gui_ui_checkbox_fn: Option<unsafe extern "C" fn(*mut c_void, *const c_char, *mut bool) -> bool>,
    il2cpp_get_assembly_image_fn: Option<unsafe extern "C" fn(*const c_char) -> *const c_void>,
    il2cpp_get_class_fn: Option<unsafe extern "C" fn(*const c_void, *const c_char, *const c_char) -> *mut c_void>,
    il2cpp_get_method_fn: Option<unsafe extern "C" fn(*mut c_void, *const c_char, i32) -> *const c_void>,
    il2cpp_get_method_addr_fn: Option<unsafe extern "C" fn(*mut c_void, *const c_char, i32) -> *mut c_void>,
    il2cpp_get_field_from_name_fn: Option<unsafe extern "C" fn(*mut c_void, *const c_char) -> *mut c_void>,
    il2cpp_get_field_value_fn: Option<unsafe extern "C" fn(*const c_void, *const c_void, *mut c_void)>,
    il2cpp_get_static_field_value_fn: Option<unsafe extern "C" fn(*const c_void, *mut c_void)>,
    il2cpp_string_new_fn: Option<unsafe extern "C" fn(*const c_char) -> *mut c_void>,
    il2cpp_string_chars_fn: Option<unsafe extern "C" fn(*const c_void) -> *mut u16>,
    il2cpp_string_length_fn: Option<unsafe extern "C" fn(*const c_void) -> i32>,
    il2cpp_resolve_symbol_fn: Option<unsafe extern "C" fn(*const c_char) -> *mut c_void>,
    il2cpp_get_singleton_like_instance_fn: Option<unsafe extern "C" fn(*mut c_void) -> *const c_void>,
    hachimi_instance_fn: Option<unsafe extern "C" fn() -> *const c_void>,
    hachimi_get_interceptor_fn: Option<unsafe extern "C" fn(*const c_void) -> *const c_void>,
    interceptor_hook_fn: Option<unsafe extern "C" fn(*const c_void, *mut c_void, *mut c_void) -> *mut c_void>,
    interceptor_get_trampoline_addr_fn: Option<unsafe extern "C" fn(*const c_void, *mut c_void) -> *mut c_void>,
    hachimi_get_data_path_fn: Option<unsafe extern "C" fn() -> *const c_char>,
}

static mut API: *const Api = ptr::null();
static GAME_INITIALIZED: AtomicBool = AtomicBool::new(false);
static HTTP_RUNNING: AtomicBool = AtomicBool::new(false);
static mut SCAN_RESULT: String = String::new();
static mut HTTP_ENABLED: bool = false;

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
// IL2CPP scan
// ============================================================

unsafe fn scan_il2cpp_classes() -> String {
    if API.is_null() { return r#"{"error":"api_null"}"#.to_string(); }
    let api = &*API;

    ura_log(3, "IL2CPP class scan starting...");

    let image = match api.il2cpp_get_assembly_image_fn {
        Some(fn_ptr) => {
            let name = to_cstr("umamusume.dll");
            let img = fn_ptr(name.as_ptr());
            if img.is_null() {
                ura_log(1, "umamusume.dll image = null");
                return r#"{"error":"image_null"}"#.to_string();
            }
            ura_log(3, "umamusume.dll image OK");
            img
        }
        None => return r#"{"error":"no_get_assembly_image"}"#.to_string(),
    };

    let classes_to_try: &[(&str, &str)] = &[
        ("Gallop", "GameSystem"),
        ("Gallop", "GameManager"),
        ("", "GameSystem"),
        ("Gallop", "TrainingInfo"),
        ("Gallop", "TrainingResultData"),
        ("Gallop", "TrainingScene"),
        ("Gallop", "TrainingExecutor"),
        ("Gallop", "StatusData"),
        ("Gallop", "GameData"),
        ("Gallop", "CharaDataSet"),
        ("", "StatusData"),
        ("", "GameData"),
        ("Gallop", "HomeData"),
        ("Gallop", "HomeScene"),
        ("Gallop", "SingleModeData"),
        ("Gallop", "SingleModeScene"),
        ("", "SingleModeData"),
        ("Gallop", "TurnInfo"),
        ("Gallop", "TurnData"),
        ("Gallop", "MotivationData"),
        ("Gallop", "CondData"),
        ("Gallop", "RaceData"),
        ("Gallop", "RaceScene"),
        ("Gallop", "RaceResultData"),
        ("Gallop", "GallopData"),
        ("Gallop", "GallopScene"),
        ("Gallop", "CharacterData"),
        ("Gallop", "CharaData"),
        ("Gallop", "SkillData"),
        ("Gallop", "SkillPointData"),
    ];

    let mut found_list: Vec<String> = Vec::new();
    let mut singleton_list: Vec<String> = Vec::new();

    for (ns, cls) in classes_to_try {
        let class = match api.il2cpp_get_class_fn {
            Some(fn_ptr) => {
                let ns_c = to_cstr(ns);
                let cls_c = to_cstr(cls);
                fn_ptr(image, ns_c.as_ptr(), cls_c.as_ptr())
            }
            None => continue,
        };

        if !class.is_null() {
            let full_name = if ns.is_empty() {
                cls.to_string()
            } else {
                format!("{}.{}", ns, cls)
            };
            ura_log(3, &format!("FOUND: {}", full_name));
            found_list.push(full_name.clone());

            if let Some(singleton_fn) = api.il2cpp_get_singleton_like_instance_fn {
                let instance = singleton_fn(class);
                if !instance.is_null() {
                    let msg = format!("{} [SINGLETON]", full_name);
                    ura_log(3, &msg);
                    singleton_list.push(full_name.clone());
                }
            }
        }
    }

    let symbols_to_try: &[&str] = &[
        "GameSystem_get_Instance",
        "GameSystem_get_Data",
        "StatusData_get_Speed",
        "StatusData_get_Stamina",
        "SingleModeData_get_StatusData",
    ];

    let mut symbol_list: Vec<String> = Vec::new();
    if let Some(resolve_fn) = api.il2cpp_resolve_symbol_fn {
        for sym in symbols_to_try {
            let sym_c = to_cstr(sym);
            let addr = resolve_fn(sym_c.as_ptr());
            if !addr.is_null() {
                let msg = format!("{} -> 0x{:x}", sym, addr as usize);
                ura_log(3, &msg);
                symbol_list.push(msg);
            }
        }
    }

    let result = format!(
        r#"{{"found_classes":["{}"],"singletons":["{}"],"symbols":["{}"],"total":{}}}"#,
        found_list.join("\",\""),
        singleton_list.join("\",\""),
        symbol_list.join("\",\""),
        found_list.len()
    );

    ura_log(3, &format!("Scan done: {} classes found", found_list.len()));
    result
}

// ============================================================
// HTTP Server
// ============================================================

fn start_http_server() {
    if HTTP_RUNNING.load(Ordering::Relaxed) { return; }
    HTTP_RUNNING.store(true, Ordering::Relaxed);

    std::thread::spawn(|| {
        unsafe { ura_log(3, "HTTP starting on :18765"); }

        let listener = match std::net::TcpListener::bind("127.0.0.1:18765") {
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
    let n = match stream.read(&mut buf) {
        Ok(n) if n > 0 => n,
        _ => return,
    };

    let req = std::str::from_utf8(&buf[..n]).unwrap_or("");
    let path = req.split(' ').nth(1).unwrap_or("/");

    let body = match path {
        "/" | "/health" => r#"{"status":"ok","version":"3.3.0","endpoints":["/scan","/status","/health"]}"#.to_string(),
        "/scan" => {
            let result = unsafe { scan_il2cpp_classes() };
            unsafe { SCAN_RESULT = result.clone(); }
            result
        }
        "/status" => {
            let g = GAME_INITIALIZED.load(Ordering::Relaxed);
            let h = HTTP_RUNNING.load(Ordering::Relaxed);
            format!(r#"{{"game_initialized":{},"http_running":{}}}"#, g, h)
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
// Callbacks
// ============================================================

extern "C" fn on_game_initialized(_userdata: *mut c_void) {
    GAME_INITIALIZED.store(true, Ordering::Relaxed);
    unsafe {
        ura_log(3, "Game initialized");
        ura_notify("URA: Game ready!");
        if HTTP_ENABLED && !HTTP_RUNNING.load(Ordering::Relaxed) {
            start_http_server();
        }
    }
}

extern "C" fn on_menu_section(_userdata: *mut c_void, ui: *mut c_void) {
    unsafe {
        if API.is_null() || ui.is_null() { return; }
        let api = &*API;

        if let Some(f) = api.gui_ui_heading_fn {
            f(ui, to_cstr("URA Assistant").as_ptr());
        }
        if let Some(f) = api.gui_ui_separator_fn { f(ui); }
        if let Some(f) = api.gui_ui_label_fn {
            f(ui, to_cstr("v3.3.0 - IL2CPP + HTTP").as_ptr());
        }

        if let Some(f) = api.gui_ui_colored_label_fn {
            if GAME_INITIALIZED.load(Ordering::Relaxed) {
                f(ui, 0, 255, 136, 255, to_cstr("Game: Connected").as_ptr());
            } else {
                f(ui, 255, 200, 0, 255, to_cstr("Game: Waiting...").as_ptr());
            }
        }

        if let Some(f) = api.gui_ui_separator_fn { f(ui); }

        if let Some(f) = api.gui_ui_checkbox_fn {
            f(ui, to_cstr("HTTP Server :18765").as_ptr(), &mut HTTP_ENABLED as *mut bool);
        }

        if let Some(f) = api.gui_ui_colored_label_fn {
            if HTTP_RUNNING.load(Ordering::Relaxed) {
                f(ui, 0, 255, 136, 255, to_cstr("HTTP: Running").as_ptr());
            } else if HTTP_ENABLED {
                f(ui, 255, 200, 0, 255, to_cstr("HTTP: Waiting...").as_ptr());
            } else {
                f(ui, 128, 128, 128, 255, to_cstr("HTTP: Off").as_ptr());
            }
        }

        if let Some(f) = api.gui_ui_separator_fn { f(ui); }

        if let Some(f) = api.gui_ui_button_fn {
            if f(ui, to_cstr("Scan IL2CPP Classes").as_ptr()) {
                std::thread::spawn(|| { unsafe { scan_il2cpp_classes(); } });
            }
        }

        if let Some(f) = api.gui_ui_label_fn {
            if !SCAN_RESULT.is_empty() {
                f(ui, to_cstr("See: 127.0.0.1:18765/scan").as_ptr());
            }
        }

        if let Some(f) = api.gui_ui_separator_fn { f(ui); }

        if let Some(f) = api.gui_ui_label_fn {
            f(ui, to_cstr("1. Check HTTP ON").as_ptr());
            f(ui, to_cstr("2. Enter game").as_ptr());
            f(ui, to_cstr("3. Browser: 127.0.0.1:18765/scan").as_ptr());
        }
    }
}

// ============================================================
// API resolve
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
        gui_ui_button_fn: try_api!("gui_ui_button", unsafe extern "C" fn(*mut c_void, *const c_char) -> bool),
        gui_ui_checkbox_fn: try_api!("gui_ui_checkbox", unsafe extern "C" fn(*mut c_void, *const c_char, *mut bool) -> bool),
        il2cpp_get_assembly_image_fn: try_api!("il2cpp_get_assembly_image", unsafe extern "C" fn(*const c_char) -> *const c_void),
        il2cpp_get_class_fn: try_api!("il2cpp_get_class", unsafe extern "C" fn(*const c_void, *const c_char, *const c_char) -> *mut c_void),
        il2cpp_get_method_fn: try_api!("il2cpp_get_method", unsafe extern "C" fn(*mut c_void, *const c_char, i32) -> *const c_void),
        il2cpp_get_method_addr_fn: try_api!("il2cpp_get_method_addr", unsafe extern "C" fn(*mut c_void, *const c_char, i32) -> *mut c_void),
        il2cpp_get_field_from_name_fn: try_api!("il2cpp_get_field_from_name", unsafe extern "C" fn(*mut c_void, *const c_char) -> *mut c_void),
        il2cpp_get_field_value_fn: try_api!("il2cpp_get_field_value", unsafe extern "C" fn(*const c_void, *const c_void, *mut c_void)),
        il2cpp_get_static_field_value_fn: try_api!("il2cpp_get_static_field_value", unsafe extern "C" fn(*const c_void, *mut c_void)),
        il2cpp_string_new_fn: try_api!("il2cpp_string_new", unsafe extern "C" fn(*const c_char) -> *mut c_void),
        il2cpp_string_chars_fn: try_api!("il2cpp_string_chars", unsafe extern "C" fn(*const c_void) -> *mut u16),
        il2cpp_string_length_fn: try_api!("il2cpp_string_length", unsafe extern "C" fn(*const c_void) -> i32),
        il2cpp_resolve_symbol_fn: try_api!("il2cpp_resolve_symbol", unsafe extern "C" fn(*const c_char) -> *mut c_void),
        il2cpp_get_singleton_like_instance_fn: try_api!("il2cpp_get_singleton_like_instance", unsafe extern "C" fn(*mut c_void) -> *const c_void),
        hachimi_instance_fn: try_api!("hachimi_instance", unsafe extern "C" fn() -> *const c_void),
        hachimi_get_interceptor_fn: try_api!("hachimi_get_interceptor", unsafe extern "C" fn(*const c_void) -> *const c_void),
        interceptor_hook_fn: try_api!("interceptor_hook", unsafe extern "C" fn(*const c_void, *mut c_void, *mut c_void) -> *mut c_void),
        interceptor_get_trampoline_addr_fn: try_api!("interceptor_get_trampoline_addr", unsafe extern "C" fn(*const c_void, *mut c_void) -> *mut c_void),
        hachimi_get_data_path_fn: try_api!("hachimi_get_data_path", unsafe extern "C" fn() -> *const c_char),
    }
}

// ============================================================
// Entry point
// ============================================================

#[no_mangle]
pub unsafe extern "C" fn hachimi_init_v3(
    get_api: extern "C" fn(*const c_char) -> *mut c_void,
    version: i32,
) -> i32 {
    let api = resolve_api(get_api);
    API = Box::into_raw(Box::new(api));

    ura_log(3, "URA plugin v3.3.0 loaded");

    if let Some(f) = (*API).gui_show_notification_fn {
        f(to_cstr("URA v3.3.0 Loaded!").as_ptr());
    }

    if let Some(f) = (*API).gui_register_menu_item_fn {
        f(to_cstr("URA Assistant").as_ptr(), None, ptr::null_mut());
    }

    if let Some(f) = (*API).gui_register_menu_section_fn {
        f(Some(on_menu_section), ptr::null_mut());
    }

    if let Some(f) = (*API).hachimi_register_on_game_initialized_fn {
        f(Some(on_game_initialized), ptr::null_mut());
    }

    ura_log(3, &format!("hachimi_init_v3 done, api_version={}", version));

    InitResult::Ok as i32
}
