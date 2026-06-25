//! URA Plugin v3.2.0-minimal
//! Hachimi Edge V3 API — 修正版
//!
//! 修复: gui_register_menu_section 实际签名为 (callback, userdata)
//!       之前错误地传了3个参数(label, callback, userdata)导致闪退

#![allow(dead_code)]

use std::ffi::{c_char, c_void, CString};
use std::ptr;

// ============================================================
// InitResult
// ============================================================
#[repr(i32)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum InitResult {
    Error = 0,
    Ok = 1,
}

// ============================================================
// V3 API 函数指针（全部 Option，解析失败不阻止初始化）
// ============================================================
struct Api {
    log_fn: Option<unsafe extern "C" fn(i32, *const c_char, *const c_char)>,
    gui_show_notification_fn: Option<unsafe extern "C" fn(*const c_char) -> bool>,
    // gui_register_menu_item(label, callback, userdata) — 有label
    gui_register_menu_item_fn: Option<unsafe extern "C" fn(*const c_char, Option<extern "C" fn(*mut c_void)>, *mut c_void) -> bool>,
    // gui_register_menu_section(callback, userdata) — 无label！
    gui_register_menu_section_fn: Option<unsafe extern "C" fn(Option<extern "C" fn(*mut c_void, *mut c_void)>, *mut c_void) -> bool>,
    hachimi_register_on_game_initialized_fn: Option<unsafe extern "C" fn(Option<extern "C" fn(*mut c_void)>, *mut c_void) -> bool>,
    gui_ui_heading_fn: Option<unsafe extern "C" fn(*mut c_void, *const c_char) -> bool>,
    gui_ui_label_fn: Option<unsafe extern "C" fn(*mut c_void, *const c_char) -> bool>,
    gui_ui_colored_label_fn: Option<unsafe extern "C" fn(*mut c_void, u8, u8, u8, u8, *const c_char) -> bool>,
    gui_ui_separator_fn: Option<unsafe extern "C" fn(*mut c_void) -> bool>,
}

// 全局 API 指针
static mut API: *const Api = ptr::null();

fn to_cstr(s: &str) -> CString {
    CString::new(s).unwrap_or_else(|_| CString::new("<err>").unwrap())
}

unsafe fn ura_log(level: i32, msg: &str) {
    if API.is_null() { return; }
    let api = &*API;
    if let Some(log_fn) = api.log_fn {
        let tag = to_cstr("URA");
        let text = to_cstr(msg);
        log_fn(level, tag.as_ptr(), text.as_ptr());
    }
}

// ============================================================
// 回调函数
// ============================================================

extern "C" fn on_game_initialized(_userdata: *mut c_void) {
    unsafe {
        ura_log(3, "game initialized callback fired");
    }
}

extern "C" fn on_menu_section(_userdata: *mut c_void, ui: *mut c_void) {
    unsafe {
        if API.is_null() || ui.is_null() { return; }
        let api = &*API;

        if let Some(heading_fn) = api.gui_ui_heading_fn {
            let t = to_cstr("URA Assistant");
            heading_fn(ui, t.as_ptr());
        }

        if let Some(sep_fn) = api.gui_ui_separator_fn {
            sep_fn(ui);
        }

        if let Some(label_fn) = api.gui_ui_label_fn {
            let t = to_cstr("v3.2.0-minimal (fixed)");
            label_fn(ui, t.as_ptr());
        }

        if let Some(colored_fn) = api.gui_ui_colored_label_fn {
            let t = to_cstr("Plugin loaded successfully!");
            colored_fn(ui, 0, 255, 136, 255, t.as_ptr());
        }

        if let Some(sep_fn) = api.gui_ui_separator_fn {
            sep_fn(ui);
        }

        if let Some(label_fn) = api.gui_ui_label_fn {
            let t = to_cstr("Minimal build - no HTTP/IL2CPP");
            label_fn(ui, t.as_ptr());
        }
    }
}

// ============================================================
// API 解析
// ============================================================

unsafe fn resolve_api(get_api: extern "C" fn(*const c_char) -> *mut c_void) -> Api {
    macro_rules! try_api {
        ($name:expr, $ty:ty) => {{
            let cname = CString::new($name).unwrap();
            let ptr = get_api(cname.as_ptr());
            if ptr.is_null() {
                None
            } else {
                Some(std::mem::transmute::<*mut c_void, $ty>(ptr))
            }
        }};
    }

    Api {
        log_fn: try_api!("log", unsafe extern "C" fn(i32, *const c_char, *const c_char)),
        gui_show_notification_fn: try_api!("gui_show_notification", unsafe extern "C" fn(*const c_char) -> bool),
        gui_register_menu_item_fn: try_api!("gui_register_menu_item", unsafe extern "C" fn(*const c_char, Option<extern "C" fn(*mut c_void)>, *mut c_void) -> bool),
        // 修正: 只2个参数，无label
        gui_register_menu_section_fn: try_api!("gui_register_menu_section", unsafe extern "C" fn(Option<extern "C" fn(*mut c_void, *mut c_void)>, *mut c_void) -> bool),
        hachimi_register_on_game_initialized_fn: try_api!("hachimi_register_on_game_initialized", unsafe extern "C" fn(Option<extern "C" fn(*mut c_void)>, *mut c_void) -> bool),
        gui_ui_heading_fn: try_api!("gui_ui_heading", unsafe extern "C" fn(*mut c_void, *const c_char) -> bool),
        gui_ui_label_fn: try_api!("gui_ui_label", unsafe extern "C" fn(*mut c_void, *const c_char) -> bool),
        gui_ui_colored_label_fn: try_api!("gui_ui_colored_label", unsafe extern "C" fn(*mut c_void, u8, u8, u8, u8, *const c_char) -> bool),
        gui_ui_separator_fn: try_api!("gui_ui_separator", unsafe extern "C" fn(*mut c_void) -> bool),
    }
}

// ============================================================
// 插件入口
// ============================================================

#[no_mangle]
pub unsafe extern "C" fn hachimi_init_v3(
    get_api: extern "C" fn(*const c_char) -> *mut c_void,
    version: i32,
) -> i32 {
    let api = resolve_api(get_api);
    API = Box::into_raw(Box::new(api));

    ura_log(3, "URA plugin v3.2.0-minimal loaded");

    // gui_show_notification
    if let Some(notify_fn) = (*API).gui_show_notification_fn {
        let msg = to_cstr("URA Plugin Loaded!");
        notify_fn(msg.as_ptr());
    }

    // gui_register_menu_item(label, callback, userdata) — 有label
    if let Some(reg_item_fn) = (*API).gui_register_menu_item_fn {
        let label = to_cstr("URA Assistant");
        reg_item_fn(label.as_ptr(), None, ptr::null_mut());
    }

    // gui_register_menu_section(callback, userdata) — 修正：无label！
    if let Some(reg_section_fn) = (*API).gui_register_menu_section_fn {
        reg_section_fn(Some(on_menu_section), ptr::null_mut());
    }

    // hachimi_register_on_game_initialized
    if let Some(reg_init_fn) = (*API).hachimi_register_on_game_initialized_fn {
        reg_init_fn(Some(on_game_initialized), ptr::null_mut());
    }

    ura_log(3, &format!("hachimi_init_v3 complete, api_version={}", version));

    InitResult::Ok as i32
}
