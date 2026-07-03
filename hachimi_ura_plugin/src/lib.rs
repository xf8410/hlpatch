//! URA Plugin v3.22.21
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
static PREDICT_STEP: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
static CRASH_SIG: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(0);
static CRASH_STEP: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
static mut LAST_STEP_BUF: [u8; 128] = [0; 128];
static LAST_STEP_LEN: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
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

// ===== Crash logging for /training/predict =====
extern "C" {
    #[link_name = "signal"]
    fn sys_signal(signum: i32, handler: usize) -> usize;
    #[link_name = "open"]
    fn sys_open(pathname: *const i8, flags: i32, mode: i32) -> i32;
    #[link_name = "write"]
    fn sys_write(fd: i32, buf: *const u8, count: usize) -> isize;
    #[link_name = "close"]
    fn sys_close(fd: i32) -> i32;
    #[link_name = "raise"]
    fn sys_raise(sig: i32) -> i32;
    #[link_name = "system"]
    fn sys_system(cmd: *const i8) -> i32;
}

const CRASH_LOG_PATH: &str = "/data/data/jp.pokemon.pokeuma/files/uma_predict.log";

extern "C" fn crash_signal_handler(sig: i32) {
    CRASH_SIG.store(sig, std::sync::atomic::Ordering::Relaxed);
    CRASH_STEP.store(PREDICT_STEP.load(std::sync::atomic::Ordering::Relaxed), std::sync::atomic::Ordering::Relaxed);
    let step = PREDICT_STEP.load(std::sync::atomic::Ordering::Relaxed);
    let mut msg = [0u8; 48];
    let p = b"CRASH at step ";
    msg[..p.len()].copy_from_slice(p);
    let mut len = p.len();
    let mut n = step;
    if n == 0 { msg[len] = b'0'; len += 1; }
    else {
        let mut digits = [0u8; 10];
        let mut dlen = 0;
        while n > 0 { digits[dlen] = b'0' + (n % 10) as u8; n /= 10; dlen += 1; }
        for i in (0..dlen).rev() { msg[len] = digits[i]; len += 1; }
    }
    let s = b" sig=";
    msg[len..len+s.len()].copy_from_slice(s); len += s.len();
    let mut n2 = sig as u32;
    if n2 == 0 { msg[len] = b'0'; len += 1; }
    else {
        let mut digits = [0u8; 10];
        let mut dlen = 0;
        while n2 > 0 { digits[dlen] = b'0' + (n2 % 10) as u8; n2 /= 10; dlen += 1; }
        for i in (0..dlen).rev() { msg[len] = digits[i]; len += 1; }
    }
    msg[len] = b'\n'; len += 1;
    let path = b"/data/data/jp.pokemon.pokeuma/files/uma_predict.log\0";
    let fd = unsafe { sys_open(path.as_ptr() as *const i8, 1 | 64 | 1024, 0o644) };
    if fd >= 0 {
        unsafe { sys_write(fd, msg.as_ptr(), len); sys_close(fd); }
    }
    unsafe { sys_signal(sig, 0); sys_raise(sig); }
}

fn init_crash_handler() {
    unsafe {
        let handler = crash_signal_handler as usize;
        sys_signal(11, handler); // SIGSEGV
        sys_signal(6, handler);  // SIGABRT
        sys_signal(7, handler);  // SIGBUS
        sys_signal(8, handler);  // SIGFPE
    }
    std::panic::set_hook(Box::new(|info| {
        let msg = format!("PANIC: {}\n", info);
        let _ = std::fs::OpenOptions::new().create(true).append(true)
            .open("/data/data/jp.pokemon.pokeuma/files/uma_predict.log")
            .and_then(|mut f| std::io::Write::write_all(&mut f, msg.as_bytes()));
    }));
}

fn log_predict_step(msg: &str) {
    let step = PREDICT_STEP.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
    let line = format!("[{}] {}\n", step, msg);

    // Store last step in static buffer for /debug/laststep
    let bytes = msg.as_bytes();
    let len = bytes.len().min(120);
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), LAST_STEP_BUF.as_mut_ptr(), len);
        LAST_STEP_BUF[len] = 0;
    }
    LAST_STEP_LEN.store(len as u32, std::sync::atomic::Ordering::Relaxed);

    // Write to file using raw libc syscalls (more reliable than std::fs on Android)
    let path1 = b"/data/data/jp.pokemon.pokeuma/files/uma_predict.log\0";
    let path2 = b"/data/local/tmp/uma_predict.log\0";
    let line_bytes = line.as_bytes();
    unsafe {
        let fd = sys_open(path1.as_ptr() as *const i8, 1 | 64 | 1024, 0o644);
        if fd >= 0 { sys_write(fd, line_bytes.as_ptr(), line_bytes.len()); sys_close(fd); }
        let fd2 = sys_open(path2.as_ptr() as *const i8, 1 | 64 | 1024, 0o644);
        if fd2 >= 0 { sys_write(fd2, line_bytes.as_ptr(), line_bytes.len()); sys_close(fd2); }
    // v3.22.21: std::fs fallback
    let _ = std::fs::OpenOptions::new().create(true).append(true)
        .open("/data/data/jp.pokemon.pokeuma/files/uma_predict.log")
        .and_then(|mut f| std::io::Write::write_all(&mut f, line.as_bytes()));
    }
}

fn clear_predict_log() {
    PREDICT_STEP.store(0, std::sync::atomic::Ordering::Relaxed);
    LAST_STEP_LEN.store(0, std::sync::atomic::Ordering::Relaxed);
    let path1 = b"/data/data/jp.pokemon.pokeuma/files/uma_predict.log\0";
    let path2 = b"/data/local/tmp/uma_predict.log\0";
    unsafe {
        let fd = sys_open(path1.as_ptr() as *const i8, 1 | 64 | 512, 0o644);
        if fd >= 0 { sys_close(fd); }
        let fd2 = sys_open(path2.as_ptr() as *const i8, 1 | 64 | 512, 0o644);
        if fd2 >= 0 { sys_close(fd2); }
    }
}

fn read_crash_log() -> String {
    let sig = CRASH_SIG.load(std::sync::atomic::Ordering::Relaxed);
    let step = CRASH_STEP.load(std::sync::atomic::Ordering::Relaxed);
    if sig != 0 {
        return format!(r#"{{"crash":true,"signal":{},"step":{}}}"#, sig, step);
    }
    match std::fs::read_to_string("/data/data/jp.pokemon.pokeuma/files/uma_predict.log") {
        Ok(s) if !s.is_empty() => s,
        _ => match std::fs::read_to_string("/data/local/tmp/uma_predict.log") {
            Ok(s) if !s.is_empty() => s,
            _ => r#"{"error":"no_crash_log"}"#.to_string(),
        }
    }
}

fn base64_encode(data: &[u8]) -> String {
    const T: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut r = String::new();
    for ch in data.chunks(3) {
        let b0 = ch[0] as usize;
        let b1 = if ch.len() > 1 { ch[1] as usize } else { 0 };
        let b2 = if ch.len() > 2 { ch[2] as usize } else { 0 };
        r.push(T[b0 >> 2] as char);
        r.push(T[((b0 & 0x03) << 4) | (b1 >> 4)] as char);
        if ch.len() > 1 { r.push(T[((b1 & 0x0f) << 2) | (b2 >> 6)] as char); }
        else { r.push('='); }
        if ch.len() > 2 { r.push(T[b2 & 0x3f] as char); }
        else { r.push('='); }
    }
    r
}

fn check_and_upload_crash_log() {
    let path = "/data/data/jp.pokemon.pokeuma/files/uma_predict.log";
    if !std::path::Path::new(path).exists() { return; }
    let content = match std::fs::read(path) { Ok(c) => c, Err(_) => return };
    if content.is_empty() { return; }
    if content.ends_with(b"DONE\n") {
        let _ = std::fs::remove_file(path);
        return;
    }
    // Base64 encode and upload to GitHub
    let b64 = base64_encode(&content);
    let json = format!(r#"{{"message":"crash log auto-upload","content":"{}"}}"#, b64);
    let _ = std::fs::write("/data/data/jp.pokemon.pokeuma/files/uma_upload.json", &json);
    let cmd = format!("curl -s -X PUT -H 'Authorization: token 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' -H 'Content-Type: application/json' -d @/data/data/jp.pokemon.pokeuma/files/uma_upload.json https://api.github.com/repos/xf8410/uma-hook/contents/crash_log.txt >/dev/null 2>&1");
    if let Ok(cmd_c) = std::ffi::CString::new(cmd) {
        unsafe { sys_system(cmd_c.as_ptr() as *const i8); }
    }
    let _ = std::fs::remove_file("/data/data/jp.pokemon.pokeuma/files/uma_upload.json");
}



fn save_endpoint_log(endpoint: &str, data: &str) {
    let safe_name = endpoint.trim_start_matches('/').replace('/', "_");
    if safe_name.is_empty() || safe_name == "health" || safe_name == "status" 
        || safe_name == "config" || safe_name == "config.html" 
        || safe_name == "debug_upload" || safe_name == "debug_crashlog" {
        return;
    }
    let _ = std::fs::create_dir_all("/data/data/jp.pokemon.pokeuma/files/uma_logs");
    let path = format!("/data/data/jp.pokemon.pokeuma/files/uma_logs/{}.json", safe_name);
    let _ = std::fs::write(&path, data);
}

fn upload_all_logs() -> String {
    let dir = "/data/data/jp.pokemon.pokeuma/files/uma_logs";
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return r#"{"error":"no_logs_dir"}"#.to_string(),
    };

    let mut uploaded = 0;
    let mut file_names: Vec<String> = Vec::new();

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        let content = match std::fs::read(entry.path()) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let b64 = base64_encode(&content);
        let github_path = format!("logs/{}", name);
        let json = format!(r#"{{"message":"upload {}","content":"{}"}}"#, name, b64);
        let tmp_path = "/data/data/jp.pokemon.pokeuma/files/uma_upload_tmp.json";
        let _ = std::fs::write(tmp_path, &json);

        let cmd = format!(
            "curl -s -X PUT -H 'Authorization: token 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' -H 'Content-Type: application/json' -d @{} 'https://api.github.com/repos/xf8410/uma-hook/contents/{}' > /dev/null 2>&1",
            tmp_path, github_path
        );
        if let Ok(cmd_c) = std::ffi::CString::new(cmd) {
            unsafe { sys_system(cmd_c.as_ptr() as *const i8); }
        }

        uploaded += 1;
        file_names.push(name);
    }

    let _ = std::fs::remove_file("/data/data/jp.pokemon.pokeuma/files/uma_upload_tmp.json");

    let files_json = file_names.iter().map(|n| format!(r#""{}""#, n)).collect::<Vec<_>>().join(",");
    format!(r#"{{"uploaded":{},"files":[{}]}}"#, uploaded, files_json)
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
    // v3.22.21: Check CLASS_CACHE first
    if !name.is_null() {
        let name_str = std::ffi::CStr::from_ptr(name).to_string_lossy().to_string();
        if let Ok(guard) = CLASS_CACHE.lock() {
            if let Some(ref map) = *guard {
                if let Some(&cls) = map.get(&name_str) {
                    return cls as *mut c_void;
                }
            }
        }
    }
    // v3.22.21: Block IL2CPP API in read path
    if IN_READ_PATH.load(Ordering::Relaxed) { return ptr::null_mut(); }
    if image.is_null() || API.is_null() { return ptr::null_mut(); }
    match (*API).il2cpp_get_class_fn {
        Some(fn_ptr) => fn_ptr(image, ns, name),
        None => ptr::null_mut(),
    }
}

unsafe fn find_class_by_short_name(image: *const c_void, class_name: &str) -> *mut c_void {
    // v3.22.21: Check CLASS_CACHE first
    if let Ok(guard) = CLASS_CACHE.lock() {
        if let Some(ref map) = *guard {
            if let Some(&cls) = map.get(class_name) {
                return cls as *mut c_void;
            }
        }
    }
    // v3.22.21: Block IL2CPP API in read path
    if IN_READ_PATH.load(Ordering::Relaxed) { return ptr::null_mut(); }
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
    // v3.22.21: Check SINGLETON_CACHE first
    let key = class as usize;
    if let Ok(guard) = SINGLETON_CACHE.lock() {
        if let Some(ref map) = *guard {
            if let Some(&val) = map.get(&key) {
                return val as *const c_void;
            }
        }
    }
    // v3.22.21: Block IL2CPP API in read path
    if IN_READ_PATH.load(Ordering::Relaxed) { return ptr::null(); }
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
    // v3.22.21: In read path, use cached offset + direct memory read
    if IN_READ_PATH.load(Ordering::Relaxed) {
        let offset = cached_find_field_offset(class, field_name);
        if offset >= 0 { return read_ptr_at(obj, offset); }
        return ptr::null_mut();
    }
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
    let field_name = method_name.strip_prefix("get_").unwrap_or(method_name);
    let offset = cached_find_field_offset(class, field_name);
    if offset >= 0 {
        return read_ptr_at(instance, offset);
    }
    // v3.22.21: Block il2cpp_runtime_invoke in read path
    if IN_READ_PATH.load(Ordering::Relaxed) { return ptr::null_mut(); }

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
    let field_name = method_name.strip_prefix("get_").unwrap_or(method_name);
    let offset = cached_find_field_offset(class, field_name);
    if offset >= 0 {
        return read_int_at(instance, offset);
    }
    // v3.22.21: Block il2cpp_runtime_invoke in read path
    if IN_READ_PATH.load(Ordering::Relaxed) { return -1; }

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
    let field_name = method_name.strip_prefix("get_").unwrap_or(method_name);
    let offset = cached_find_field_offset(class, field_name);
    if offset >= 0 {
        return read_obscured_int_at(instance, offset);
    }
    // v3.22.21: Block il2cpp_runtime_invoke in read path
    if IN_READ_PATH.load(Ordering::Relaxed) { return -1; }

    let result = call_getter_on_instance(class, instance, method_name);
    if result.is_null() { return -1; }

    // Boxed ObscuredInt struct layout (from dump.cs Anti-Cheat Toolkit):
    // offset 0x10: currentCryptoKey (Int32) — the decryption key
    // offset 0x14: hiddenValue (Int32) — the encrypted value
    // offset 0x18: inited (Boolean)
    // offset 0x1C: fakeValue (Int32)
    // offset 0x20: fakeValueActive (Boolean)
    let base = result as *const u8;

    let current_crypto_key = std::ptr::read_unaligned::<i32>(base.add(IL2CPP_OBSCURED_INT_KEY_OFF) as *const i32);
    let hidden_value = std::ptr::read_unaligned::<i32>(base.add(IL2CPP_OBSCURED_INT_HIDDEN_OFF) as *const i32);

    // Decrypt: hiddenValue ^ currentCryptoKey
    let decrypted = hidden_value ^ current_crypto_key;

    ura_log(4, &format!("ObscuredInt {}: hidden={} key={} decrypted={}", 
        method_name, hidden_value, current_crypto_key, decrypted));

    decrypted
}

// ============================================================
// ★ v3.22.21: Direct memory read helpers — zero il2cpp calls
// ============================================================

unsafe fn read_obscured_int_at(obj: *const c_void, field_offset: i32) -> i32 {
    if obj.is_null() || field_offset < 0 { return -1; }
    let base = obj as *const u8;
    let off = field_offset as usize;
    let key = std::ptr::read_unaligned::<i32>(base.add(off) as *const i32);
    let hidden = std::ptr::read_unaligned::<i32>(base.add(off + 4) as *const i32);
    hidden ^ key
}

unsafe fn read_ptr_at(obj: *const c_void, field_offset: i32) -> *mut c_void {
    if obj.is_null() || field_offset < 0 { return ptr::null_mut(); }
    std::ptr::read_unaligned::<*mut c_void>(
        (obj as *const u8).add(field_offset as usize) as *const *mut c_void
    )
}

// ★ v3.22.21: Direct int read — zero il2cpp_runtime_invoke
unsafe fn read_int_at(obj: *const c_void, field_offset: i32) -> i32 {
    if obj.is_null() || field_offset < 0 { return -1; }
    let base = obj as *const u8;
    std::ptr::read_unaligned::<i32>(base.add(field_offset as usize) as *const i32)
}

/// v3.22.21: Read Il2CppClass* from object header (offset 0 on 64-bit)
/// This gives us the EXACT class of any object instance at runtime
unsafe fn get_class_from_object(obj: *const c_void) -> *mut c_void {
    if obj.is_null() { return ptr::null_mut(); }
    std::ptr::read_unaligned::<*mut c_void>(obj as *const *mut c_void)
}

/// v3.22.21: Read ObscuredInt field from object using its own class (from header)
/// No need to find class by name — reads it directly from the object
unsafe fn read_obscured_int_from_obj(obj: *const c_void, field_name: &str) -> i32 {
    if obj.is_null() { return -1; }
    let class = get_class_from_object(obj);
    if class.is_null() { return -1; }
    call_getter_obscured_int(class, obj, field_name)
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
    let length = std::ptr::read_unaligned::<usize>(base.add(IL2CPP_LIST_COUNT_OFF) as *const usize);
    if length == 0 || length > 200 { return result; } // ★ v3.22.21: guard rail — lower limit

    // ObscuredInt struct (unboxed) layout:
    // offset 0x00: currentCryptoKey (Int32)
    // offset 0x04: hiddenValue (Int32)
    // offset 0x08: inited (Boolean, padded to 4)
    // offset 0x0C: fakeValue (Int32)
    // offset 0x10: fakeValueActive (Boolean, padded to 4)
    // struct size = 0x14 (20 bytes), aligned to 4
    let struct_size: usize = 0x14;  // ObscuredInt unboxed: 5 fields × 4 bytes = 20 bytes (key+hidden+inited+fake+fakeActive) — fixed: was IL2CPP_OBSCURED_INT_HIDDEN_OFF + IL2CPP_LIST_ITEM_SIZE = 0x1C (wrong, mixed boxed offset with unboxed)
    let data_start = base.add(IL2CPP_LIST_ITEMS_OFF);

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
    let length = std::ptr::read_unaligned::<usize>(base.add(IL2CPP_LIST_COUNT_OFF) as *const usize);
    if length == 0 || length > 100 { return results; }

    for i in 0..length {
        let elem_ptr = std::ptr::read_unaligned::<*mut c_void>(base.add(IL2CPP_LIST_ITEMS_OFF + i * IL2CPP_LIST_ITEM_SIZE) as *const *mut c_void);
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
                        let length = std::ptr::read_unaligned::<usize>(base.add(IL2CPP_LIST_COUNT_OFF) as *const usize);
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
                            let cmd_len = std::ptr::read_unaligned::<usize>(base.add(IL2CPP_LIST_COUNT_OFF) as *const usize);
                            let mut cmd_details = Vec::new();
                            for i in 0..cmd_len {
                                let elem_ptr = std::ptr::read_unaligned::<*mut c_void>(base.add(IL2CPP_LIST_ITEMS_OFF + i * IL2CPP_LIST_ITEM_SIZE) as *const *mut c_void);
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
                                        CMD_SPEED => "Speed", CMD_STAMINA => "Stamina", CMD_GUTS => "Guts",
                                        CMD_POWER => "Power", CMD_WISDOM => "Wiz",
                                        CMD_URA_SPEED => "Speed", CMD_URA_STAMINA => "Stamina", CMD_URA_GUTS => "Guts",
                                        CMD_URA_POWER => "Power", CMD_URA_WISDOM => "Wiz",
                                        CMD_KAKUSHIMI => "Kakushimi",
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
                                        let p_len = std::ptr::read_unaligned::<usize>(p_base.add(IL2CPP_LIST_COUNT_OFF) as *const usize);
                                        for j in 0..p_len {
                                            let p_elem = std::ptr::read_unaligned::<*mut c_void>(p_base.add(IL2CPP_LIST_ITEMS_OFF + j * IL2CPP_LIST_ITEM_SIZE) as *const *mut c_void);
                                            if p_elem.is_null() { continue; }
                                            // ★ Breeders: always plain Int32 (SingleModeParamsIncDecInfo)
                                            // TargetType 实测映射（与dump.cs ParameterType枚举不同！）：
                                            //   枚举定义3=Power 4=Guts，但target_type字段实际3=Guts 4=Power
                                            //   验证：Stamina训练(TT3)加Guts，Power训练(TT4)加Power
                                            //   0=None, 1=Speed, 2=Stamina, 3=Guts, 4=Power, 5=Wiz
                                            //   10=HP, 20=Motivation, 30=SkillPt
                                            let bytes = p_elem as *const u8;
                                            let t = std::ptr::read_unaligned::<i32>(bytes.add(IL2CPP_OBSCURED_INT_KEY_OFF) as *const i32);
                                            let v = std::ptr::read_unaligned::<i32>(bytes.add(IL2CPP_OBSCURED_INT_HIDDEN_OFF) as *const i32);
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
                                        std::ptr::read_unaligned::<usize>(mbase.add(IL2CPP_LIST_COUNT_OFF) as *const usize)
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
                    // ObscuredInt fields: EffectCategory, EffectId, EffectValue
                    let ae_arr = call_getter_on_instance(dataset_class, dataset_obj, "get_ActiveEffectArray");
                    if !ae_arr.is_null() {
                        let ae_base = ae_arr as *const u8;
                        let ae_len = std::ptr::read_unaligned::<usize>(ae_base.add(IL2CPP_LIST_COUNT_OFF) as *const usize);
                        if ae_len > 0 && ae_len < 100 {
                            let mut effects = Vec::new();
                            for i in 0..ae_len {
                                let ep = std::ptr::read_unaligned::<*mut c_void>(ae_base.add(IL2CPP_LIST_ITEMS_OFF + i * IL2CPP_LIST_ITEM_SIZE) as *const *mut c_void);
                                if ep.is_null() { continue; }
                                // v3.22.21: Read class from object header — no more find_class or hardcoded offsets
                                let cat = read_obscured_int_from_obj(ep, "get_EffectCategory");
                                let eid = read_obscured_int_from_obj(ep, "get_EffectId");
                                let val = read_obscured_int_from_obj(ep, "get_EffectValue");
                                effects.push(format!(r#"{{"EffectCategory":{},"EffectId":{},"EffectValue":{}}}"#, cat, eid, val));
                            }
                            result_parts.push(format!(r#""active_effects":[{}]"#, effects.join(",")));
                        }
                    }

                    // Read UrafEffectInfo (Ramen uraf effect)
                    // Class: ObscuredSingleModeRamenUrafEffectInfo
                    // ObscuredInt fields: UrafEffectType, UrafEffectState
                    let uraf_obj = call_getter_on_instance(dataset_class, dataset_obj, "get_UrafEffectInfo");
                    if !uraf_obj.is_null() {
                        // v3.22.21: Read class from object header — no more find_class or hardcoded offsets
                        let ut = read_obscured_int_from_obj(uraf_obj, "get_UrafEffectType");
                        let us = read_obscured_int_from_obj(uraf_obj, "get_UrafEffectState");
                        result_parts.push(format!(r#""uraf_effect":{{"UrafEffectType":{},"UrafEffectState":{}}}"#, ut, us));
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
                        let fi_len = std::ptr::read_unaligned::<usize>(fi_base.add(IL2CPP_LIST_COUNT_OFF) as *const usize);
                        if fi_len > 0 && fi_len < 100 {
                            // v3.22.21: Read class from each element's object header — no more find_class or hardcoded offsets
                            let mut fi_elements = Vec::new();
                            for fi in 0..fi_len {
                                let fe_ptr = std::ptr::read_unaligned::<*mut c_void>(fi_base.add(IL2CPP_LIST_ITEMS_OFF + fi * IL2CPP_LIST_ITEM_SIZE) as *const *mut c_void);
                                if fe_ptr.is_null() { fi_elements.push("{}".to_string()); continue; }
                                let ft = read_obscured_int_from_obj(fe_ptr, "get_FeelingIndex");
                                let fv = read_obscured_int_from_obj(fe_ptr, "get_FeelingId");
                                fi_elements.push(format!(r#"{{"FeelingIndex":{},"FeelingId":{}}}"#, ft, fv));
                            }
                            result_parts.push(format!(r#""feeling_info":[{}]"#, fi_elements.join(",")));
                        }
                    }

                    // FeelingTurnInfoArray: 2 ObscuredInt fields (Turn, FeelingType)
                    // v3.22.21: Read class from object header — no more hardcoded offsets
                    let ft_arr = call_getter_on_instance(dataset_class, dataset_obj, "get_FeelingTurnInfoArray");
                    if !ft_arr.is_null() {
                        let ft_base = ft_arr as *const u8;
                        let ft_len = std::ptr::read_unaligned::<usize>(ft_base.add(IL2CPP_LIST_COUNT_OFF) as *const usize);
                        if ft_len > 0 && ft_len < 100 {
                            let mut ft_elems = Vec::new();
                            for fi in 0..ft_len {
                                let fp = std::ptr::read_unaligned::<*mut c_void>(ft_base.add(IL2CPP_LIST_ITEMS_OFF + fi * IL2CPP_LIST_ITEM_SIZE) as *const *mut c_void);
                                if fp.is_null() { ft_elems.push("{}".to_string()); continue; }
                                let t = read_obscured_int_from_obj(fp, "get_RemainTurn");
                                let fty = read_obscured_int_from_obj(fp, "get_FeelingId");
                                ft_elems.push(format!(r#"{{"RemainTurn":{},"FeelingId":{}}}"#, t, fty));
                            }
                            result_parts.push(format!(r#""feeling_turn_info":[{}]"#, ft_elems.join(",")));
                        }
                    }


                    // CommandFeelingInfoArray: 3 ObscuredInt fields (CommandType, CommandId, FeelingId)
                    // v3.22.21: Read class from object header — no more hardcoded offsets
                    let cf_arr = call_getter_on_instance(dataset_class, dataset_obj, "get_CommandFeelingInfoArray");
                    if !cf_arr.is_null() {
                        let cf_base = cf_arr as *const u8;
                        let cf_len = std::ptr::read_unaligned::<usize>(cf_base.add(IL2CPP_LIST_COUNT_OFF) as *const usize);
                        if cf_len > 0 && cf_len < 100 {
                            let mut cf_elems = Vec::new();
                            for ci in 0..cf_len {
                                let cp = std::ptr::read_unaligned::<*mut c_void>(cf_base.add(IL2CPP_LIST_ITEMS_OFF + ci * IL2CPP_LIST_ITEM_SIZE) as *const *mut c_void);
                                if cp.is_null() { cf_elems.push("{}".to_string()); continue; }
                                let ct = read_obscured_int_from_obj(cp, "get_CommandType");
                                let cid = read_obscured_int_from_obj(cp, "get_CommandId");
                                let fid = read_obscured_int_from_obj(cp, "get_FeelingId");
                                cf_elems.push(format!(r#"{{"CommandType":{},"CommandId":{},"FeelingId":{}}}"#, ct, cid, fid));
                            }
                            result_parts.push(format!(r#""command_feeling_info":[{}]"#, cf_elems.join(",")));
                        }
                    }


                    // FeelingReduceTurnInfoArray: 2 ObscuredInt fields (Turn, FeelingType)
                    // v3.22.21: Read class from object header — no more hardcoded offsets
                    let fr_arr = call_getter_on_instance(dataset_class, dataset_obj, "get_FeelingReduceTurnInfoArray");
                    if !fr_arr.is_null() {
                        let fr_base = fr_arr as *const u8;
                        let fr_len = std::ptr::read_unaligned::<usize>(fr_base.add(IL2CPP_LIST_COUNT_OFF) as *const usize);
                        if fr_len > 0 && fr_len < 100 {
                            let mut fr_elems = Vec::new();
                            for ri in 0..fr_len {
                                let rp = std::ptr::read_unaligned::<*mut c_void>(fr_base.add(IL2CPP_LIST_ITEMS_OFF + ri * IL2CPP_LIST_ITEM_SIZE) as *const *mut c_void);
                                if rp.is_null() { fr_elems.push("{}".to_string()); continue; }
                                let t = read_obscured_int_from_obj(rp, "get_CommandType");
                                let fty = read_obscured_int_from_obj(rp, "get_CommandId");
                                fr_elems.push(format!(r#"{{"CommandType":{},"CommandId":{}}}"#, t, fty));
                            }
                            result_parts.push(format!(r#""feeling_reduce_turn_info":[{}]"#, fr_elems.join(",")));
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
// ★ v3.22.21: find_field_offset — read field offset via il2cpp_class_get_fields
// Thread-safe metadata API, NO il2cpp_runtime_invoke calls
// ============================================================

unsafe fn find_field_offset(class: *mut c_void, field_name: &str) -> i32 {
    if class.is_null() || API.is_null() { return -1; }
    // v3.22.21: Block IL2CPP API in read path
    if IN_READ_PATH.load(Ordering::Relaxed) { return -1; }

    let get_fields_fn: Option<FnClassGetFields> = {
        let p = resolve_il2cpp_symbol("il2cpp_class_get_fields");
        if p.is_null() { None } else { Some(std::mem::transmute::<*mut c_void, FnClassGetFields>(p)) }
    };
    let get_parent_fn: Option<FnClassGetParent> = {
        let p = resolve_il2cpp_symbol("il2cpp_class_get_parent");
        if p.is_null() { None } else { Some(std::mem::transmute::<*mut c_void, FnClassGetParent>(p)) }
    };

    if get_fields_fn.is_none() { return -1; }

    let normalize = |name: &str| -> String {
        let n = if name.starts_with('<') {
            if let Some(end) = name.find('>') { &name[1..end] } else { name }
        } else {
            name
        };
        n.trim_start_matches('_').to_lowercase()
    };
    let target = normalize(field_name);

    // Pass 1: exact match (case-insensitive after normalization)
    let mut current_class = class;
    let mut depth = 0;
    loop {
        if current_class.is_null() || depth > 10 { break; }
        let mut iter: *mut c_void = ptr::null_mut();
        loop {
            let field_info = get_fields_fn.unwrap()(current_class, &mut iter);
            if field_info.is_null() { break; }
            if !(*field_info).name.is_null() {
                let s = std::ffi::CStr::from_ptr((*field_info).name);
                let fname = s.to_string_lossy().to_string();
                if normalize(&fname) == target {
                    return (*field_info).offset;
                }
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

    // Pass 2: substring match (fallback)
    let mut current_class = class;
    let mut depth = 0;
    loop {
        if current_class.is_null() || depth > 10 { break; }
        let mut iter: *mut c_void = ptr::null_mut();
        loop {
            let field_info = get_fields_fn.unwrap()(current_class, &mut iter);
            if field_info.is_null() { break; }
            if !(*field_info).name.is_null() {
                let s = std::ffi::CStr::from_ptr((*field_info).name);
                let fname = s.to_string_lossy().to_string();
                let normalized = normalize(&fname);
                if normalized.contains(&target) || fname.contains(field_name) {
                    return (*field_info).offset;
                }
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

    ura_log(3, &format!("find_field_offset: '{}' not found", field_name));
    -1
}

// ============================================================
// ★ v3.22.21: Field offset cache — avoid repeated il2cpp_class_get_fields calls
// ============================================================
use std::collections::HashMap;
static FIELD_OFFSET_CACHE: std::sync::Mutex<Option<HashMap<String, i32>>> = std::sync::Mutex::new(None);

// v3.22.21: Zero IL2CPP API in read path
static IN_READ_PATH: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static CLASS_CACHE: std::sync::Mutex<Option<HashMap<String, usize>>> = std::sync::Mutex::new(None);
static SINGLETON_CACHE: std::sync::Mutex<Option<HashMap<usize, usize>>> = std::sync::Mutex::new(None);

unsafe fn cached_find_field_offset(class: *mut c_void, field_name: &str) -> i32 {
    let key = format!("{:p}_{}", class, field_name);
    // Check cache
    if let Ok(guard) = FIELD_OFFSET_CACHE.lock() {
        if let Some(ref map) = *guard {
            if let Some(&offset) = map.get(&key) {
                return offset;
            }
        }
    }
    // v3.22.21: Block IL2CPP API in read path
    if IN_READ_PATH.load(Ordering::Relaxed) { return -1; }
    // Not in cache, look up
    let offset = find_field_offset(class, field_name);
    // Store in cache (even -1, to avoid repeated failed lookups)
    if let Ok(mut guard) = FIELD_OFFSET_CACHE.lock() {
        if guard.is_none() { *guard = Some(HashMap::new()); }
        if let Some(ref mut map) = *guard {
            map.insert(key, offset);
        }
    }
    offset
}

// ============================================================
// ★ v3.22.21: read_ramen_scalar_fields — read 5 ObscuredInt fields from DataSet
// Zero il2cpp_runtime_invoke calls (only find_field_offset + read_obscured_int_at)
// ============================================================

unsafe fn read_ramen_scalar_fields(
    ds_class: *mut c_void,
    dataset_obj: *const c_void,
) -> (i32, i32, i32, i32, i32) {
    let checkpoint_pt = {
        let off = cached_find_field_offset(ds_class, "CheckPointPt");
        if off >= 0 { read_obscured_int_at(dataset_obj, off) } else { -1 }
    };
    let special_feeling_num = {
        let off = cached_find_field_offset(ds_class, "SpecialFeelingNum");
        if off >= 0 { read_obscured_int_at(dataset_obj, off) } else { -1 }
    };
    let recommend_type = {
        let off = cached_find_field_offset(ds_class, "RecommendType");
        if off >= 0 { read_obscured_int_at(dataset_obj, off) } else { -1 }
    };
    let (uraf_type, uraf_state) = {
        let uraf_off = cached_find_field_offset(ds_class, "UrafEffectInfo");
        if uraf_off >= 0 {
            let uraf_obj = read_ptr_at(dataset_obj, uraf_off);
            if !uraf_obj.is_null() {
                let uraf_class = std::ptr::read_unaligned::<*mut c_void>(
                    uraf_obj as *const *mut c_void
                );
                let ut_off = cached_find_field_offset(uraf_class, "UrafEffectType");
                let us_off = cached_find_field_offset(uraf_class, "UrafEffectState");
                let ut = if ut_off >= 0 { read_obscured_int_at(uraf_obj, ut_off) } else { -1 };
                let us = if us_off >= 0 { read_obscured_int_at(uraf_obj, us_off) } else { -1 };
                (ut, us)
            } else {
                (-1, -1)
            }
        } else {
            (-1, -1)
        }
    };
    (checkpoint_pt, special_feeling_num, recommend_type, uraf_type, uraf_state)
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

/// Per-stat evaluation score lookup table (stat value → 評価点)
/// 0-1200: gamewith実測値 (https://gamewith.jp/uma-musume/article/show/279308)
/// 1201-2300: cubic外推 f(x)=2.346e-9·x³+6.537e-3·x²-7.890·x+3891.8
const STAT_EVAL_SCORE: [i32; 2301] = [
    0, 1, 1, 2, 2, 3, 3, 4, 4, 5,
    5, 6, 6, 7, 7, 8, 8, 9, 9, 10,
    10, 11, 11, 12, 12, 13, 13, 14, 14, 15,
    15, 16, 16, 17, 17, 18, 18, 19, 19, 20,
    20, 21, 21, 22, 22, 23, 23, 24, 24, 25,
    25, 26, 27, 28, 29, 29, 30, 31, 32, 33,
    33, 34, 35, 36, 37, 37, 38, 39, 40, 41,
    41, 42, 43, 44, 45, 45, 46, 47, 48, 49,
    49, 50, 51, 52, 53, 53, 54, 55, 56, 57,
    57, 58, 59, 60, 61, 61, 62, 63, 64, 65,
    66, 67, 68, 69, 70, 71, 72, 73, 74, 75,
    76, 77, 78, 79, 80, 81, 82, 83, 84, 85,
    86, 87, 88, 89, 90, 91, 92, 93, 94, 95,
    96, 97, 98, 99, 100, 101, 102, 103, 104, 105,
    106, 107, 108, 109, 110, 111, 112, 113, 114, 115,
    116, 117, 118, 120, 121, 122, 124, 125, 126, 128,
    129, 130, 131, 133, 134, 135, 137, 138, 139, 141,
    142, 143, 144, 146, 147, 148, 150, 151, 152, 154,
    155, 156, 157, 159, 160, 161, 163, 164, 165, 167,
    168, 169, 170, 172, 173, 174, 176, 177, 178, 180,
    181, 183, 184, 186, 188, 189, 191, 192, 194, 196,
    197, 199, 200, 202, 204, 205, 207, 208, 210, 212,
    213, 215, 216, 218, 220, 221, 223, 224, 226, 228,
    229, 231, 232, 234, 236, 237, 239, 240, 242, 244,
    245, 247, 248, 250, 252, 253, 255, 256, 258, 260,
    261, 263, 265, 267, 269, 270, 272, 274, 276, 278,
    279, 281, 283, 285, 287, 288, 290, 292, 294, 296,
    297, 299, 301, 303, 305, 306, 308, 310, 312, 314,
    315, 317, 319, 321, 323, 324, 326, 328, 330, 332,
    333, 335, 337, 339, 341, 342, 344, 346, 348, 350,
    352, 354, 356, 358, 360, 362, 364, 366, 368, 371,
    373, 375, 377, 379, 381, 383, 385, 387, 389, 392,
    394, 396, 398, 400, 402, 404, 406, 408, 410, 413,
    415, 417, 419, 422, 423, 425, 427, 429, 431, 434,
    436, 438, 440, 442, 444, 446, 448, 450, 452, 455,
    457, 459, 462, 464, 467, 469, 471, 474, 476, 479,
    481, 483, 486, 488, 491, 493, 495, 498, 500, 503,
    505, 507, 510, 512, 515, 517, 519, 522, 524, 527,
    529, 531, 534, 536, 539, 541, 543, 546, 548, 551,
    553, 555, 558, 560, 563, 565, 567, 570, 572, 575,
    577, 580, 582, 585, 588, 590, 593, 595, 598, 601,
    603, 606, 608, 611, 614, 616, 619, 621, 624, 627,
    629, 632, 634, 637, 640, 642, 645, 647, 650, 653,
    655, 658, 660, 663, 666, 668, 671, 673, 676, 679,
    681, 684, 686, 689, 692, 694, 697, 699, 702, 705,
    707, 710, 713, 716, 719, 721, 724, 727, 730, 733,
    735, 738, 741, 744, 747, 749, 752, 755, 758, 761,
    763, 766, 769, 772, 775, 777, 780, 783, 786, 789,
    791, 794, 797, 800, 803, 805, 808, 811, 814, 817,
    819, 822, 825, 828, 831, 833, 836, 839, 842, 845,
    847, 850, 853, 856, 859, 862, 865, 868, 871, 874,
    876, 879, 882, 885, 888, 891, 894, 897, 900, 903,
    905, 908, 911, 914, 917, 920, 923, 926, 929, 931,
    934, 937, 940, 943, 946, 949, 952, 955, 958, 961,
    963, 966, 969, 972, 975, 978, 981, 984, 987, 990,
    993, 996, 999, 1002, 1005, 1008, 1011, 1014, 1017, 1020,
    1023, 1026, 1029, 1032, 1035, 1038, 1041, 1044, 1047, 1050,
    1053, 1056, 1059, 1062, 1065, 1068, 1071, 1074, 1077, 1080,
    1083, 1086, 1089, 1092, 1095, 1098, 1101, 1104, 1107, 1110,
    1113, 1116, 1119, 1122, 1125, 1128, 1131, 1134, 1137, 1140,
    1143, 1146, 1149, 1152, 1155, 1158, 1161, 1164, 1167, 1171,
    1174, 1177, 1180, 1183, 1186, 1189, 1192, 1195, 1198, 1202,
    1205, 1208, 1211, 1214, 1217, 1220, 1223, 1226, 1229, 1233,
    1236, 1239, 1242, 1245, 1248, 1251, 1254, 1257, 1260, 1264,
    1267, 1270, 1273, 1276, 1279, 1282, 1285, 1288, 1291, 1295,
    1298, 1301, 1304, 1308, 1311, 1314, 1318, 1321, 1324, 1328,
    1331, 1334, 1337, 1341, 1344, 1347, 1351, 1354, 1357, 1361,
    1364, 1367, 1370, 1374, 1377, 1380, 1384, 1387, 1390, 1394,
    1397, 1400, 1403, 1407, 1410, 1413, 1417, 1420, 1423, 1427,
    1430, 1433, 1436, 1440, 1443, 1446, 1450, 1453, 1456, 1460,
    1463, 1466, 1470, 1473, 1477, 1480, 1483, 1487, 1490, 1494,
    1497, 1500, 1504, 1507, 1511, 1514, 1517, 1521, 1524, 1528,
    1531, 1534, 1538, 1541, 1545, 1548, 1551, 1555, 1558, 1562,
    1565, 1568, 1572, 1575, 1579, 1582, 1585, 1589, 1592, 1596,
    1599, 1602, 1606, 1609, 1613, 1616, 1619, 1623, 1626, 1630,
    1633, 1637, 1640, 1644, 1647, 1651, 1654, 1658, 1661, 1665,
    1668, 1672, 1675, 1679, 1682, 1686, 1689, 1693, 1696, 1700,
    1703, 1707, 1710, 1714, 1717, 1721, 1724, 1728, 1731, 1735,
    1738, 1742, 1745, 1749, 1752, 1756, 1759, 1763, 1766, 1770,
    1773, 1777, 1780, 1784, 1787, 1791, 1794, 1798, 1801, 1805,
    1808, 1812, 1816, 1820, 1824, 1828, 1832, 1836, 1840, 1844,
    1847, 1851, 1855, 1859, 1863, 1867, 1871, 1875, 1879, 1883,
    1886, 1890, 1894, 1898, 1902, 1906, 1910, 1914, 1918, 1922,
    1925, 1929, 1933, 1937, 1941, 1945, 1949, 1953, 1957, 1961,
    1964, 1968, 1972, 1976, 1980, 1984, 1988, 1992, 1996, 2000,
    2004, 2008, 2012, 2016, 2020, 2024, 2028, 2032, 2036, 2041,
    2045, 2049, 2053, 2057, 2061, 2065, 2069, 2073, 2077, 2082,
    2086, 2090, 2094, 2098, 2102, 2106, 2110, 2114, 2118, 2123,
    2127, 2131, 2135, 2139, 2143, 2147, 2151, 2155, 2159, 2164,
    2168, 2172, 2176, 2180, 2184, 2188, 2192, 2196, 2200, 2205,
    2209, 2213, 2217, 2221, 2226, 2230, 2234, 2238, 2242, 2247,
    2251, 2255, 2259, 2263, 2268, 2272, 2276, 2280, 2284, 2289,
    2293, 2297, 2301, 2305, 2310, 2314, 2318, 2322, 2326, 2331,
    2335, 2339, 2343, 2347, 2352, 2356, 2360, 2364, 2368, 2373,
    2377, 2381, 2385, 2389, 2394, 2398, 2402, 2406, 2410, 2415,
    2419, 2423, 2427, 2432, 2436, 2440, 2445, 2449, 2453, 2458,
    2462, 2466, 2470, 2475, 2479, 2483, 2488, 2492, 2496, 2501,
    2505, 2509, 2513, 2518, 2522, 2526, 2531, 2535, 2539, 2544,
    2548, 2552, 2556, 2561, 2565, 2569, 2574, 2578, 2582, 2587,
    2591, 2595, 2599, 2604, 2608, 2612, 2617, 2621, 2625, 2630,
    2635, 2640, 2645, 2650, 2656, 2661, 2666, 2671, 2676, 2682,
    2687, 2692, 2697, 2702, 2708, 2713, 2718, 2723, 2728, 2734,
    2739, 2744, 2749, 2754, 2760, 2765, 2770, 2775, 2780, 2786,
    2791, 2796, 2801, 2806, 2812, 2817, 2822, 2827, 2832, 2838,
    2843, 2848, 2853, 2858, 2864, 2869, 2874, 2879, 2884, 2890,
    2895, 2901, 2906, 2912, 2917, 2923, 2928, 2934, 2939, 2945,
    2950, 2956, 2961, 2967, 2972, 2978, 2983, 2989, 2994, 3000,
    3005, 3011, 3016, 3022, 3027, 3033, 3038, 3044, 3049, 3055,
    3060, 3066, 3071, 3077, 3082, 3088, 3093, 3099, 3104, 3110,
    3115, 3121, 3126, 3132, 3137, 3143, 3148, 3154, 3159, 3165,
    3171, 3178, 3184, 3191, 3198, 3204, 3211, 3217, 3224, 3231,
    3237, 3244, 3250, 3257, 3264, 3270, 3277, 3283, 3290, 3297,
    3303, 3310, 3316, 3323, 3330, 3336, 3343, 3349, 3356, 3363,
    3369, 3376, 3382, 3389, 3396, 3402, 3409, 3415, 3422, 3429,
    3435, 3442, 3448, 3455, 3462, 3468, 3475, 3481, 3488, 3495,
    3501, 3508, 3515, 3522, 3529, 3535, 3542, 3549, 3556, 3563,
    3569, 3576, 3583, 3590, 3597, 3603, 3610, 3617, 3624, 3631,
    3637, 3644, 3651, 3658, 3665, 3671, 3678, 3685, 3692, 3699,
    3705, 3712, 3719, 3726, 3733, 3739, 3746, 3753, 3760, 3767,
    3773, 3780, 3787, 3794, 3801, 3807, 3814, 3821, 3828, 3835,
    3841, 3849, 3857, 3865, 3873, 3881, 3889, 3896, 3904, 3912,
    3920, 3928, 3936, 3944, 3952, 3960, 3968, 3976, 3984, 3992,
    4000, 4008, 4016, 4025, 4033, 4041, 4049, 4057, 4065, 4073,
    4082, 4090, 4098, 4106, 4115, 4123, 4131, 4139, 4148, 4156,
    4164, 4173, 4181, 4189, 4198, 4206, 4215, 4223, 4231, 4240,
    4248, 4257, 4265, 4274, 4282, 4291, 4299, 4308, 4316, 4325,
    4334, 4342, 4351, 4359, 4368, 4377, 4385, 4394, 4403, 4411,
    4420, 4429, 4438, 4446, 4455, 4464, 4473, 4482, 4490, 4499,
    4508, 4517, 4526, 4535, 4544, 4553, 4561, 4570, 4579, 4588,
    4597, 4606, 4615, 4624, 4633, 4642, 4651, 4661, 4670, 4679,
    4688, 4697, 4706, 4715, 4724, 4734, 4743, 4752, 4761, 4770,
    4780, 4789, 4798, 4808, 4817, 4826, 4835, 4845, 4854, 4863,
    4873, 4882, 4892, 4901, 4910, 4920, 4929, 4939, 4948, 4958,
    4967, 4977, 4986, 4996, 5005, 5015, 5025, 5034, 5044, 5053,
    5063, 5073, 5082, 5092, 5102, 5111, 5121, 5131, 5141, 5150,
    5160, 5170, 5180, 5190, 5199, 5209, 5219, 5229, 5239, 5249,
    5259, 5268, 5278, 5288, 5298, 5308, 5318, 5328, 5338, 5348,
    5358, 5368, 5378, 5388, 5398, 5409, 5419, 5429, 5439, 5449,
    5459, 5469, 5480, 5490, 5500, 5510, 5520, 5531, 5541, 5551,
    5562, 5572, 5582, 5593, 5603, 5613, 5624, 5634, 5644, 5655,
    5665, 5676, 5686, 5697, 5707, 5717, 5728, 5739, 5749, 5760,
    5770, 5781, 5791, 5802, 5812, 5823, 5834, 5844, 5855, 5866,
    5876, 5887, 5898, 5908, 5919, 5930, 5941, 5952, 5962, 5973,
    5984, 5995, 6006, 6016, 6027, 6038, 6049, 6060, 6071, 6082,
    6093, 6104, 6115, 6126, 6137, 6148, 6159, 6170, 6181, 6192,
    6203, 6214, 6225, 6236, 6247, 6259, 6270, 6281, 6292, 6303,
    6314, 6326, 6337, 6348, 6359, 6371, 6382, 6393, 6405, 6416,
    6427, 6439, 6450, 6461, 6473, 6484, 6496, 6507, 6518, 6530,
    6541, 6553, 6564, 6576, 6587, 6599, 6610, 6622, 6634, 6645,
    6657, 6668, 6680, 6692, 6703, 6715, 6727, 6738, 6750, 6762,
    6773, 6785, 6797, 6809, 6821, 6832, 6844, 6856, 6868, 6880,
    6891, 6903, 6915, 6927, 6939, 6951, 6963, 6975, 6987, 6999,
    7011, 7023, 7035, 7047, 7059, 7071, 7083, 7095, 7107, 7119,
    7131, 7144, 7156, 7168, 7180, 7192, 7204, 7217, 7229, 7241,
    7253, 7266, 7278, 7290, 7303, 7315, 7327, 7340, 7352, 7364,
    7377, 7389, 7402, 7414, 7426, 7439, 7451, 7464, 7476, 7489,
    7501, 7514, 7526, 7539, 7551, 7564, 7577, 7589, 7602, 7615,
    7627, 7640, 7652, 7665, 7678, 7691, 7703, 7716, 7729, 7742,
    7754, 7767, 7780, 7793, 7806, 7818, 7831, 7844, 7857, 7870,
    7883, 7896, 7909, 7922, 7935, 7948, 7961, 7974, 7987, 8000,
    8013, 8026, 8039, 8052, 8065, 8078, 8091, 8104, 8117, 8131,
    8144, 8157, 8170, 8183, 8197, 8210, 8223, 8236, 8250, 8263,
    8276, 8290, 8303, 8316, 8330, 8343, 8356, 8370, 8383, 8397,
    8410, 8423, 8437, 8450, 8464, 8477, 8491, 8504, 8518, 8531,
    8545, 8559, 8572, 8586, 8599, 8613, 8627, 8640, 8654, 8668,
    8681, 8695, 8709, 8723, 8736, 8750, 8764, 8778, 8791, 8805,
    8819, 8833, 8847, 8861, 8874, 8888, 8902, 8916, 8930, 8944,
    8958, 8972, 8986, 9000, 9014, 9028, 9042, 9056, 9070, 9084,
    9098, 9112, 9127, 9141, 9155, 9169, 9183, 9197, 9212, 9226,
    9240, 9254, 9268, 9283, 9297, 9311, 9326, 9340, 9354, 9369,
    9383, 9397, 9412, 9426, 9440, 9455, 9469, 9484, 9498, 9513,
    9527, 9542, 9556, 9571, 9585, 9600, 9614, 9629, 9643, 9658,
    9673, 9687, 9702, 9717, 9731, 9746, 9761, 9775, 9790, 9805,
    9819, 9834, 9849, 9864, 9879, 9893, 9908, 9923, 9938, 9953,
    9968, 9982, 9997, 10012, 10027, 10042, 10057, 10072, 10087, 10102,
    10117, 10132, 10147, 10162, 10177, 10192, 10207, 10222, 10238, 10253,
    10268, 10283, 10298, 10313, 10329, 10344, 10359, 10374, 10389, 10405,
    10420, 10435, 10450, 10466, 10481, 10496, 10512, 10527, 10543, 10558,
    10573, 10589, 10604, 10620, 10635, 10650, 10666, 10681, 10697, 10712,
    10728, 10744, 10759, 10775, 10790, 10806, 10821, 10837, 10853, 10868,
    10884, 10900, 10915, 10931, 10947, 10963, 10978, 10994, 11010, 11026,
    11041, 11057, 11073, 11089, 11105, 11120, 11136, 11152, 11168, 11184,
    11200, 11216, 11232, 11248, 11264, 11280, 11296, 11312, 11328, 11344,
    11360, 11376, 11392, 11408, 11424, 11440, 11457, 11473, 11489, 11505,
    11521, 11537, 11554, 11570, 11586, 11602, 11619, 11635, 11651, 11667,
    11684, 11700, 11716, 11733, 11749, 11765, 11782, 11798, 11815, 11831,
    11848, 11864, 11881, 11897, 11914, 11930, 11947, 11963, 11980, 11996,
    12013, 12029, 12046, 12063, 12079, 12096, 12113, 12129, 12146, 12163,
    12179, 12196, 12213, 12229, 12246, 12263, 12280, 12297, 12313, 12330,
    12347, 12364, 12381, 12398, 12415, 12431, 12448, 12465, 12482, 12499,
    12516, 12533, 12550, 12567, 12584, 12601, 12618, 12635, 12652, 12670,
    12687, 12704, 12721, 12738, 12755, 12772, 12789, 12807, 12824, 12841,
    12858, 12876, 12893, 12910, 12927, 12945, 12962, 12979, 12997, 13014,
    13031, 13049, 13066, 13084, 13101, 13118, 13136, 13153, 13171, 13188,
    13206, 13223, 13241, 13258, 13276, 13293, 13311, 13329, 13346, 13364,
    13381, 13399, 13417, 13434, 13452, 13470, 13487, 13505, 13523, 13541,
    13558, 13576, 13594, 13612, 13630, 13647, 13665, 13683, 13701, 13719,
    13737, 13755, 13772, 13790, 13808, 13826, 13844, 13862, 13880, 13898,
    13916, 13934, 13952, 13970, 13988, 14007, 14025, 14043, 14061, 14079,
    14097, 14115, 14133, 14152, 14170, 14188, 14206, 14225, 14243, 14261,
    14279, 14298, 14316, 14334, 14353, 14371, 14389, 14408, 14426, 14444,
    14463, 14481, 14500, 14518, 14537, 14555, 14574, 14592, 14611, 14629,
    14648, 14666, 14685, 14703, 14722, 14741, 14759, 14778, 14797, 14815,
    14834, 14853, 14871, 14890, 14909, 14927, 14946, 14965, 14984, 15003,
    15021, 15040, 15059, 15078, 15097, 15116, 15134, 15153, 15172, 15191,
    15210, 15229, 15248, 15267, 15286, 15305, 15324, 15343, 15362, 15381,
    15400, 15419, 15438, 15457, 15477, 15496, 15515, 15534, 15553, 15572,
    15592, 15611, 15630, 15649, 15668, 15688, 15707, 15726, 15746, 15765,
    15784, 15804, 15823, 15842, 15862, 15881, 15900, 15920, 15939, 15959,
    15978, 15998, 16017, 16037, 16056, 16076, 16095, 16115, 16134, 16154,
    16174, 16193, 16213, 16232, 16252, 16272, 16291, 16311, 16331, 16350,
    16370, 16390, 16410, 16429, 16449, 16469, 16489, 16509, 16528, 16548,
    16568, 16588, 16608, 16628, 16648, 16668, 16688, 16707, 16727, 16747,
    16767, 16787, 16807, 16827, 16847, 16867, 16888, 16908, 16928, 16948,
    16968, 16988, 17008, 17028, 17049, 17069, 17089, 17109, 17129, 17150,
    17170, 17190, 17210, 17231, 17251, 17271, 17292, 17312, 17332, 17353,
    17373, 17393, 17414, 17434, 17455, 17475, 17496, 17516, 17536, 17557,
    17577, 17598, 17619, 17639, 17660, 17680, 17701, 17721, 17742, 17763,
    17783, 17804, 17825, 17845, 17866, 17887, 17907, 17928, 17949, 17970,
    17990, 18011, 18032, 18053, 18074, 18094, 18115, 18136, 18157, 18178,
    18199, 18220, 18241, 18262, 18283, 18303, 18324, 18345, 18366, 18387,
    18409, 18430, 18451, 18472, 18493, 18514, 18535, 18556, 18577, 18598,
    18620, 18641, 18662, 18683, 18704, 18726, 18747, 18768, 18789, 18811,
    18832, 18853, 18875, 18896, 18917, 18939, 18960, 18981, 19003, 19024,
    19046, 19067, 19088, 19110, 19131, 19153, 19174, 19196, 19217, 19239,
    19261, 19282, 19304, 19325, 19347, 19369, 19390, 19412, 19433, 19455,
    19477, 19499, 19520, 19542, 19564, 19585, 19607, 19629, 19651, 19673,
    19694, 19716, 19738, 19760, 19782, 19804, 19826, 19848, 19869, 19891,
    19913, 19935, 19957, 19979, 20001, 20023, 20045, 20067, 20089, 20111,
    20134, 20156, 20178, 20200, 20222, 20244, 20266, 20288, 20311, 20333,
    20355,
];

const BASIC_FIVE_STATUS_LIMIT: [i32; 5] = [2300, 2200, 1800, 1400, 1400];
// === AI Evaluation Named Constants ===
// Vital evaluation piecewise: slopes and breakpoints
const VITAL_EVAL_LOW_SLOPE: f64 = 2.0;     // vital ≤50: steep slope
const VITAL_EVAL_MID_SLOPE: f64 = 1.5;     // vital 50-70: moderate slope
const VITAL_EVAL_HIGH_SLOPE: f64 = 1.0;    // vital >70: flat slope
const VITAL_EVAL_LOW_THRESH: i32 = 50;     // low→mid breakpoint
const VITAL_EVAL_MID_THRESH: i32 = 70;     // mid→high breakpoint
// Derived intercepts (precomputed to avoid recomputation)
const VITAL_EVAL_MID_INTERCEPT: f64 = 100.0;   // VITAL_EVAL_LOW_SLOPE * VITAL_EVAL_LOW_THRESH = 2.0*50
const VITAL_EVAL_HIGH_INTERCEPT: f64 = 130.0;  // 100.0 + 1.5*(70-50) = mid_intercept + mid_slope*(mid_thresh-low_thresh)

// Vital factor: controls how much we value vitality
const VITAL_FACTOR_BASE: f64 = 3.5;        // starting vital factor
const VITAL_FACTOR_RANGE: f64 = 3.5;       // added over full game (base→7.0 at end)

// Soft constraint: reserve multiplier for stat overflow penalty
const RESERVE_MULTIPLIER: f64 = 40.0;
const RESERVE_MIN: f64 = 0.1;             // avoid division by zero

// URA event final bonus (stats gained from non-training events)
const URA3_BONUS: i32 = 45;               // URA scenario 3rd event
const URA_FINAL_EVENT_BONUS: i32 = 30;    // final event after training
const URA_EVENT_BONUS: i32 = 20;          // URA1/URA2 event bonus

// Training evaluation parameters
const STATUS_WEIGHT: f64 = 6.0;           // per-stat weight (uniform for all 5)
const SMALL_FAIL_VALUE: f64 = -150.0;     // minor failure penalty
const BIG_FAIL_VALUE: f64 = -500.0;       // major failure (大失敗) penalty
const PT_SCORE_RATE: f64 = 2.0;           // skill point → evaluation value rate
const FAIL_RATE_TO_PROB: f64 = 0.01;      // convert percentage (0-100) to probability
const BIG_FAIL_THRESHOLD: i32 = 20;       // fail_rate below this → no 大失敗

// Shining (彩圈) and heads (相伴) bonus
const SHINING_BONUS_PER: f64 = 200.0;     // expected value per 彩圈 partner
const HEADS_BONUS_PER: f64 = 20.0;        // small bonus per extra partner

// Rest/Outgoing vital gain
const REST_VITAL_GAIN: i32 = 50;          // vital gained from rest
const OUTGOING_VITAL_GAIN: i32 = 50;      // vital gained from outgoing

// Motivation factor: scales training value by current mood
// 1=絶不調, 2=不調, 3=普通, 4=好調, 5=絶好調
const MOT_FACTOR_WORST: f64 = 0.6;        // motivation 1
const MOT_FACTOR_BAD: f64 = 0.75;         // motivation 2
const MOT_FACTOR_NORMAL: f64 = 0.9;       // motivation 3
const MOT_FACTOR_GOOD: f64 = 1.0;         // motivation 4
const MOT_FACTOR_BEST: f64 = 1.1;         // motivation 5

// Outgoing motivation bonus (motivation level → value of raising it)
const OUTGOING_BONUS_MOT1: f64 = 80.0;    // 絶不調→不調: urgent
const OUTGOING_BONUS_MOT2: f64 = 50.0;    // 不調→普通: important
const OUTGOING_BONUS_MOT3: f64 = 25.0;    // 普通→好調: moderate
const OUTGOING_BONUS_MOT4: f64 = 10.0;    // 好調→絶好調: minor

// Game scenario total turns
const URA_TOTAL_TURNS: i32 = 78;           // URA scenario has 78 training turns
const DEFAULT_TOTAL_TURNS: i32 = 72;       // Standard scenarios have 72 turns

// Game CommandId constants (IL2CPP method identifiers)
const CMD_SPEED: i32 = 101;
const CMD_STAMINA: i32 = 102;
const CMD_GUTS: i32 = 103;
const CMD_POWER: i32 = 105;
const CMD_WISDOM: i32 = 106;
const CMD_URA_SPEED: i32 = 601;
const CMD_URA_STAMINA: i32 = 602;
const CMD_URA_GUTS: i32 = 603;
const CMD_URA_POWER: i32 = 604;
const CMD_URA_WISDOM: i32 = 605;
const CMD_KAKUSHIMI: i32 = 304;

// URA turn thresholds for max vital equivalent calculation
const URA_LAST_TURN: i32 = 76;            // URA finals: no vital needed
const URA_PRE_FINAL_TURN: i32 = 71;       // just before URA: minimal vital
const URA_PRE_FINAL_VITAL: i32 = 10;      // vital needed at pre-final turn
const URA_FINAL_VITAL: i32 = 30;          // vital needed at final training turn
const URA_MAX_NON_RACE_TURNS: i32 = 6;    // max non-race turns before URA
const URA_VITAL_PER_NON_RACE: i32 = 15;   // vital equivalent per non-race turn
const TEXT_DATA_CATEGORY_CHARA_NAME: i32 = 6;    // text_data.category=6: character name
const TEXT_DATA_CATEGORY_RACE_NAME: i32 = 32;    // text_data.category=32: race name
const TEXT_DATA_CATEGORY_STORY_TITLE: i32 = 45;  // text_data.category=45: single mode story title
const TEXT_DATA_CATEGORY_SKILL_NAME: i32 = 47;   // text_data.category=47: skill name
const IL2CPP_LIST_COUNT_OFF: usize = 0x18;      // Il2CppList._count (il2cpp internal, all List<T>)
const IL2CPP_LIST_ITEMS_OFF: usize = 0x20;      // Il2CppList._items[0] start (il2cpp internal, all List<T>)
const IL2CPP_LIST_ITEM_SIZE: usize = 0x08;      // sizeof(pointer) on aarch64
const IL2CPP_OBSCURED_INT_KEY_OFF: usize = 0x10;  // ObscuredInt.currentCryptoKey (boxed layout)
const IL2CPP_OBSCURED_INT_HIDDEN_OFF: usize = 0x14; // ObscuredInt.hiddenValue (boxed layout)
const IL2CPP_UNBOX_FIRST_FIELD: usize = 0x10;     // Unbox() result: first field offset (after Il2CppObject header 0x10)
const IL2CPP_UNBOX_SECOND_FIELD: usize = 0x14;    // Unbox() result: second field offset
const IL2CPP_SUPPORT_CARD_POSITION_OFF: usize = 0x10;  // SingleModeEquipSupportCard.position (IL2CPP /fields/ offset=16)
const IL2CPP_SUPPORT_CARD_ID_OFF: usize = 0x14;        // SingleModeEquipSupportCard.supportCardId (IL2CPP /fields/ offset=20)
const IL2CPP_SUPPORT_CARD_LIMIT_OFF: usize = 0x18;     // SingleModeEquipSupportCard.limitBreakCount (IL2CPP /fields/ offset=24)
const IL2CPP_TARGET_RACE_ID_OFF: usize = 0x10;         // SingleModeTargetRace.targetId (IL2CPP /fields/ offset=16)
const IL2CPP_TARGET_RACE_EVAL_OFF: usize = 0x14;       // SingleModeTargetRace.evaluation (IL2CPP /fields/ offset=20)
const IL2CPP_COMMAND_ID_OFF: usize = 0x10;             // SingleModeCommandId.commandId (IL2CPP /fields/ offset=16)
const IL2CPP_COMMAND_LEVEL_OFF: usize = 0x14;          // SingleModeCommandId.level (IL2CPP /fields/ offset=20)
const IL2CPP_OBSCURED_INT_UNBOX_KEY_OFF: usize = 0x10; // ObscuredInt unboxed: currentCryptoKey (offset=0x10)
const IL2CPP_OBSCURED_INT_UNBOX_HIDDEN_OFF: usize = 0x14; // ObscuredInt unboxed: hiddenValue (offset=0x14)
const IL2CPP_OBSCURED_INT_PAIR2_KEY_OFF: usize = 0x24;   // Second ObscuredInt in pair: currentCryptoKey (offset=0x24)
const IL2CPP_OBSCURED_INT_PAIR2_HIDDEN_OFF: usize = 0x28; // Second ObscuredInt in pair: hiddenValue (offset=0x28)
const IL2CPP_LIST_ARRAY_OFF: usize = 0x10;            // Il2CppList._items array pointer (offset=0x10)


/// Compute current evaluation score from five stats (per-stat lookup then sum)
/// 評価点 = STAT_EVAL_SCORE[speed] + STAT_EVAL_SCORE[stamina] + ... + STAT_EVAL_SCORE[wiz]
fn compute_score(speed: i32, stamina: i32, power: i32, guts: i32, wiz: i32) -> i32 {
    let lookup = |x: i32| -> i32 {
        if x <= 0 { return 0; }
        let idx = x as usize;
        if idx >= STAT_EVAL_SCORE.len() { return STAT_EVAL_SCORE[STAT_EVAL_SCORE.len() - 1]; }
        STAT_EVAL_SCORE[idx]
    };
    lookup(speed) + lookup(stamina) + lookup(power) + lookup(guts) + lookup(wiz)
}

/// Soft constraint function for stat overflow control
/// When stat gain would exceed remaining space, reduce its effective value
fn status_soft_function(x: f64, reserve: f64) -> f64 {
    if x >= 0.0 { return 0.0; }
    if x > -reserve { return -x * x / (2.0 * reserve); }
    x + 0.5 * reserve
}

/// Vital evaluation: low vital is very valuable, high vital less so
/// ≤VITAL_EVAL_LOW_THRESH: steep, LOW_THRESH-MID_THRESH: moderate, >MID_THRESH: flat
fn vital_evaluation(vital: i32, max_vital: i32) -> f64 {
    let v = if vital > max_vital { max_vital } else { vital };
    if v <= VITAL_EVAL_LOW_THRESH {
        VITAL_EVAL_LOW_SLOPE * v as f64
    } else if v <= VITAL_EVAL_MID_THRESH {
        VITAL_EVAL_MID_SLOPE * (v - VITAL_EVAL_LOW_THRESH) as f64 + VITAL_EVAL_MID_INTERCEPT
    } else {
        VITAL_EVAL_HIGH_SLOPE * (v - VITAL_EVAL_MID_THRESH) as f64 + VITAL_EVAL_HIGH_INTERCEPT
    }
}

/// Calculate max vital equivalent for vital evaluation
/// Late game: less vital needed (fewer turns remain)
fn calculate_max_vital_eq(turn: i32, max_vital: i32) -> i32 {
    if turn >= URA_LAST_TURN { return 0; }
    if turn > URA_PRE_FINAL_TURN { return URA_PRE_FINAL_VITAL; }
    if turn == URA_PRE_FINAL_TURN { return URA_FINAL_VITAL; }
    let non_race_turns = std::cmp::min(URA_MAX_NON_RACE_TURNS, URA_PRE_FINAL_TURN - turn);
    let eq = URA_FINAL_VITAL + URA_VITAL_PER_NON_RACE * non_race_turns;
    if eq > max_vital { max_vital } else { eq }
}

/// CommandId → training index (0=Speed, 1=Stamina, 2=Power, 3=Guts, 4=Wisdom)
fn cmd_id_to_train_idx(cmd_id: i32) -> Option<usize> {
    match cmd_id {
        CMD_SPEED => Some(0),
        CMD_STAMINA => Some(1),
        CMD_POWER => Some(2),
        CMD_GUTS => Some(3),
        CMD_WISDOM => Some(4),
        _ => None,
    }
}

/// AI evaluation result
struct AiResult {
    score: i32,           // Current evaluation score (attribute + skill)
    skill_eval: i32,      // Skill evaluation value
    skill_count: i32,     // Number of learned skills
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
    skill_eval: i32,      // ★ v3.22.0: skill evaluation value
    skill_count: i32,     // ★ v3.22.0: learned skill count
) -> AiResult {
    // Total turns per scenario
    let total_turn: i32 = match scenario_id {
        1 => URA_TOTAL_TURNS,
        _ => DEFAULT_TOTAL_TURNS,
    };

    let remain_turn = total_turn - turn - 1;
    let remain_turn = if remain_turn < 0 { 0 } else { remain_turn };

    // === Current Score ===
    let attr_score = compute_score(stats[0], stats[1], stats[2], stats[3], stats[4]);
    let score = attr_score + skill_eval;  // ★ v3.22.0: attribute + skill evaluation
    let total_stats = stats[0] + stats[1] + stats[2] + stats[3] + stats[4];

    // === Evaluation Parameters ===
    let status_weights = [STATUS_WEIGHT, STATUS_WEIGHT, STATUS_WEIGHT, STATUS_WEIGHT, STATUS_WEIGHT];
    let small_fail_value = SMALL_FAIL_VALUE;
    let big_fail_value = BIG_FAIL_VALUE;
    let pt_score_rate = PT_SCORE_RATE;

    // Vital factor: increases from 3.5 to 7.0 as game progresses
    let vital_factor = VITAL_FACTOR_BASE + (turn as f64 / total_turn as f64) * VITAL_FACTOR_RANGE;

    // Reserve for soft constraint: controls stat overflow penalty
    let reserve = RESERVE_MULTIPLIER * remain_turn as f64 * (1.0 - remain_turn as f64 / (total_turn as f64 * 2.0));
    let reserve = if reserve > RESERVE_MIN { reserve } else { RESERVE_MIN };

    // URA final bonus (events that add stats after training)
    let mut final_bonus = URA3_BONUS + URA_FINAL_EVENT_BONUS;
    if remain_turn >= 1 { final_bonus += URA_EVENT_BONUS; } // URA2
    if remain_turn >= 2 { final_bonus += URA_EVENT_BONUS; } // URA1

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
            CMD_SPEED => "Speed", CMD_STAMINA => "Stamina", CMD_GUTS => "Guts",
            CMD_POWER => "Power", CMD_WISDOM => "Wisdom",
            CMD_URA_SPEED => "Speed", CMD_URA_STAMINA => "Stamina", CMD_URA_GUTS => "Guts",
            CMD_URA_POWER => "Power", CMD_URA_WISDOM => "Wisdom",
            CMD_KAKUSHIMI => "Kakushimi",
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
            1 => MOT_FACTOR_WORST,
            2 => MOT_FACTOR_BAD,
            3 => MOT_FACTOR_NORMAL,
            4 => MOT_FACTOR_GOOD,
            _ => MOT_FACTOR_BEST,
        };
        value *= mot_factor;

        // Vital change effect
        let vital_after = std::cmp::min(max_vital_eq, vital + vital_cost);
        value += vital_factor * (vital_evaluation(vital_after, max_vital) - vital_before);

        // Failure penalty
        if fail_rate > 0 {
            let big_fail_prob = if fail_rate < BIG_FAIL_THRESHOLD { 0.0 } else { fail_rate as f64 };
            let fail_value_avg = FAIL_RATE_TO_PROB * big_fail_prob * big_fail_value
                               + (1.0 - FAIL_RATE_TO_PROB * big_fail_prob) * small_fail_value;
            value = FAIL_RATE_TO_PROB * fail_rate as f64 * fail_value_avg
                  + (1.0 - FAIL_RATE_TO_PROB * fail_rate as f64) * value;
        }

        // ★ v3.15.3: Shining (彩圈) bonus — friend/group card event expected value
        // Each 彩圈 partner gives a training event with extra stats + skill hint
        if shining > 0 {
            let shining_bonus = SHINING_BONUS_PER * shining as f64;
            value += shining_bonus;
        }
        // Heads bonus: more partners = faster relationship building
        if heads > 1 {
            let heads_bonus = HEADS_BONUS_PER * (heads - 1) as f64;
            value += heads_bonus;
        }
        train_values.push((name.to_string(), value));

        if value > best_value {
            best_value = value;
            best_action = name.to_string();
        }
    }

    // === Evaluate Rest ===
    let rest_vital_gain = REST_VITAL_GAIN;
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
        1 => OUTGOING_BONUS_MOT1,
        2 => OUTGOING_BONUS_MOT2,
        3 => OUTGOING_BONUS_MOT3,
        4 => OUTGOING_BONUS_MOT4,
        _ => 0.0,
    };
    let outgoing_vital_gain = OUTGOING_VITAL_GAIN;
    let vital_after_outgoing = std::cmp::min(max_vital_eq, vital + outgoing_vital_gain);
    let outgoing_value = vital_factor * (vital_evaluation(vital_after_outgoing, max_vital) - vital_before)
                        + outgoing_bonus;

    if outgoing_value > best_value {
        best_value = outgoing_value;
        best_action = "Outgoing".to_string();
    }

    AiResult {
        score,
        skill_eval,
        skill_count,
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
        r#"{{"score":{},"skill_eval":{},"skill_count":{},"total_stats":{},"best":"{}","best_v":{},"train":{{{}}},"rest":{},"outgoing":{}}}"#,
        r.score,
        r.skill_eval,
        r.skill_count,
        r.total_stats,
        r.best_action,
        (r.best_value * 10.0).round() / 10.0,
        tv.join(","),
        (r.rest_value * 10.0).round() / 10.0,
        (r.outgoing_value * 10.0).round() / 10.0,
    )
}

// ★ v3.22.0: Skill Evaluation — read learned skills from game + MasterDB

/// Read learned skill IDs and levels from Character object
/// Returns Vec<(skill_id, level)> where level is 1=normal, 2=evolved
unsafe fn read_chara_skills(chara_class: *mut c_void, chara_obj: *const c_void, image: *const c_void) -> Vec<(i32, i32)> {
    let mut skills = Vec::new();
    
    // Approach 1: Try get_SkillDataArray() -> SingleModeSkillData[]
    let arr = call_getter_on_instance(chara_class, chara_obj, "get_SkillDataArray");
    if !arr.is_null() {
        let ab = arr as *const u8;
        let al = std::ptr::read_unaligned::<usize>(ab.add(IL2CPP_LIST_COUNT_OFF) as *const usize);
        if al > 0 && al < 500 {
            let skill_elem_class = find_class_by_short_name(image, "SingleModeSkillData");
            for i in 0..al {
                let ep = std::ptr::read_unaligned::<*mut c_void>(ab.add(IL2CPP_LIST_ITEMS_OFF + i * IL2CPP_LIST_ITEM_SIZE) as *const *mut c_void);
                if ep.is_null() { continue; }
                let skill_id = if !skill_elem_class.is_null() {
                    call_getter_int(skill_elem_class, ep, "get_SkillId")
                } else {
                    std::ptr::read_unaligned::<i32>((ep as *const u8).add(IL2CPP_UNBOX_FIRST_FIELD) as *const i32)
                };
                let level = if !skill_elem_class.is_null() {
                    call_getter_int(skill_elem_class, ep, "get_Level")
                } else { 1 };
                if skill_id > 0 {
                    skills.push((skill_id, if level > 0 { level } else { 1 }));
                }
            }
            if !skills.is_empty() { return skills; }
        }
    }
    
    // Approach 2: Try get_PossessSkillIdArray() -> int[]
    for method_name in &["get_PossessSkillIdArray", "get_SkillIdArray"] {
        let arr2 = call_getter_on_instance(chara_class, chara_obj, method_name);
        if arr2.is_null() { continue; }
        let ab = arr2 as *const u8;
        let al = std::ptr::read_unaligned::<usize>(ab.add(IL2CPP_LIST_COUNT_OFF) as *const usize);
        if al > 0 && al < 500 {
            for i in 0..al {
                let ep = std::ptr::read_unaligned::<*mut c_void>(ab.add(IL2CPP_LIST_ITEMS_OFF + i * IL2CPP_LIST_ITEM_SIZE) as *const *mut c_void);
                if ep.is_null() { continue; }
                let sid = std::ptr::read_unaligned::<i32>(ep as *const i32);
                if sid > 0 { skills.push((sid, 1)); }
            }
        }
        if !skills.is_empty() { return skills; }
    }
    
    // Approach 3: Try reading skill_data_array field directly
    let field_arr = read_field_value(chara_class, chara_obj, "skill_data_array");
    if !field_arr.is_null() {
        let ab = field_arr as *const u8;
        let al = std::ptr::read_unaligned::<usize>(ab.add(IL2CPP_LIST_COUNT_OFF) as *const usize);
        if al > 0 && al < 500 {
            let skill_elem_class = find_class_by_short_name(image, "SingleModeSkillData");
            for i in 0..al {
                let ep = std::ptr::read_unaligned::<*mut c_void>(ab.add(IL2CPP_LIST_ITEMS_OFF + i * IL2CPP_LIST_ITEM_SIZE) as *const *mut c_void);
                if ep.is_null() { continue; }
                let skill_id = if !skill_elem_class.is_null() {
                    call_getter_int(skill_elem_class, ep, "get_SkillId")
                } else {
                    std::ptr::read_unaligned::<i32>((ep as *const u8).add(IL2CPP_UNBOX_FIRST_FIELD) as *const i32)
                };
                let level = if !skill_elem_class.is_null() {
                    call_getter_int(skill_elem_class, ep, "get_Level")
                } else { 1 };
                if skill_id > 0 {
                    skills.push((skill_id, if level > 0 { level } else { 1 }));
                }
            }
        }
    }
    
    skills
}

/// Compute skill evaluation from learned skills using MasterDB
/// Returns (total_skill_eval, skill_count, skills_breakdown_json)
fn compute_skill_eval(skills: &[(i32, i32)]) -> (i32, i32, String) {
    if skills.is_empty() {
        return (0, 0, "[]".to_string());
    }
    
    let mdb_path = match find_mdb_path() {
        Some(p) => p,
        None => return (0, skills.len() as i32, "[]".to_string()),
    };
    
    let conn = match Connection::open_with_flags(&mdb_path, OpenFlags::SQLITE_OPEN_READ_ONLY) {
        Ok(c) => c,
        Err(_) => return (0, skills.len() as i32, "[]".to_string()),
    };
    
    // Build a map of skill_id -> grade_value from MasterDB
    let mut grade_map: std::collections::HashMap<i32, i32> = std::collections::HashMap::new();
    let _ = conn.prepare("SELECT id, grade_value FROM skill_data").map(|mut stmt| {
        let _ = stmt.query_map([], |row| {
            Ok((row.get::<_, i32>(0).unwrap_or(0), row.get::<_, i32>(1).unwrap_or(0)))
        }).map(|rows| {
            rows.filter_map(|r| r.ok()).for_each(|(id, gv)| {
                grade_map.insert(id, gv);
            });
        });
    });
    
    // Also get skill names
    let mut name_map: std::collections::HashMap<i32, String> = std::collections::HashMap::new();
    let _ = conn.prepare(&format!("SELECT id, text FROM text_data WHERE category={}", TEXT_DATA_CATEGORY_SKILL_NAME)).map(|mut stmt| {
        let _ = stmt.query_map([], |row| {
            let text: String = row.get::<_, Option<String>>(1).unwrap_or(None).unwrap_or_default();
            Ok((row.get::<_, i32>(0).unwrap_or(0), text))
        }).map(|rows| {
            rows.filter_map(|r| r.ok()).for_each(|(id, name)| {
                name_map.insert(id, name);
            });
        });
    });
    
    // Compute skill_eval for each learned skill
    let mut total_eval = 0i32;
    let mut breakdown = Vec::new();
    for &(skill_id, level) in skills {
        let grade_value = *grade_map.get(&skill_id).unwrap_or(&0);
        // Level multiplier: level 1 = 1.0x, level 2 (evolved) = 1.2x
        let level_mult = if level >= 2 { 1.2f64 } else { 1.0f64 };
        let eval = (grade_value as f64 * level_mult) as i32;
        total_eval += eval;
        let name = name_map.get(&skill_id).cloned().unwrap_or_else(|| format!("id:{}", skill_id));
        breakdown.push(format!(
            r#"{{"id":{},"name":"{}","gv":{},"lv":{},"ev":{}}}"#,
            skill_id, json_escape(&name), grade_value, level, eval
        ));
    }
    
    (total_eval, skills.len() as i32, format!("[{}]", breakdown.join(",")))
}

// ★ v3.22.21: Summary cache — reduce IL2CPP metadata reads
static CACHED_SUMMARY: std::sync::Mutex<Option<(String, u64)>> = std::sync::Mutex::new(None);
const SUMMARY_CACHE_TTL_SECS: u64 = 3;

fn read_summary() -> String {
    // ★ v3.22.21: Check cache first — avoid IL2CPP calls if data hasn't changed
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    if let Ok(guard) = CACHED_SUMMARY.lock() {
        if let Some((ref cached, ts)) = *guard {
            if now.saturating_sub(ts) < SUMMARY_CACHE_TTL_SECS {
                return cached.clone();
            }
        }
    }
    // ★ v3.15.2: Mutex lock prevents concurrent il2cpp reads from HTTP + push threads
    let _lock = READ_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let summary = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        unsafe { read_summary_inner() }
    })).unwrap_or_else(|_| r#"{"error":"panic_caught","hint":"read_summary panicked, game protected"}"#.to_string());
    // ★ v3.22.21: Update cache
    if let Ok(mut guard) = CACHED_SUMMARY.lock() {
        *guard = Some((summary.clone(), now));
    }
    summary
}

unsafe fn read_summary_inner() -> String {
    // v3.22.21: Set IN_READ_PATH to block ALL IL2CPP API calls
    IN_READ_PATH.store(true, Ordering::Relaxed);
    let result = read_summary_inner_impl();
    IN_READ_PATH.store(false, Ordering::Relaxed);
    result
}

unsafe fn read_summary_inner_impl() -> String {
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
    log_predict_step("S:wdm");

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
    log_predict_step(&format!("S:stats sid={}", sid));

    // ★ v3.22.0: Read learned skills and compute skill evaluation
    ura_log(3, "★ read_summary phase1b: skill eval");
    let (skill_eval, skill_count, skills_json) = {
        let learned_skills = read_chara_skills(chara_class, chara_obj, image);
        compute_skill_eval(&learned_skills)
    };
    ura_log(2, &format!("skill_eval={} count={}", skill_eval, skill_count));
    log_predict_step("S:skills");

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
    let mut ramen_uraf_type: i32 = -1;
    let mut ramen_uraf_state: i32 = -1;
    // ★ v3.22.21: Ramen direct memory read — only 2 il2cpp_runtime_invoke calls
    // (try_get_scenario_obj + get_DataSet), then zero il2cpp calls
    if sid == 14 {
        ura_log(3, "v3.22.21 ramen: direct memory read");
        log_predict_step("S:ramen start");
        let scenario_obj = try_get_scenario_obj(chara_class, chara_obj, 14);
        if !scenario_obj.is_null() {
            let sc_class = std::ptr::read_unaligned::<*mut c_void>(
                scenario_obj as *const *mut c_void
            );
            log_predict_step("S:ramen sc_obj");
            let dataset_obj = call_getter_ref(sc_class, scenario_obj, "get_DataSet");
            if !dataset_obj.is_null() {
                let ds_class = std::ptr::read_unaligned::<*mut c_void>(
                    dataset_obj as *const *mut c_void
                );
                // Read 5 scalar ObscuredInt fields (zero il2cpp calls)
                let (cp_pt, sf_num, rec_type, uraf_t, uraf_s) =
                    read_ramen_scalar_fields(ds_class, dataset_obj);
                log_predict_step("S:ramen ds");
                ramen_checkpoint_pt = cp_pt;
                ramen_special_feeling_num = sf_num;
                ramen_recommend_type = rec_type;
                ramen_uraf_type = uraf_t;
                ramen_uraf_state = uraf_s;
                ura_log(3, &format!(
                    "ramen scalar: cp={} sf={} rec={} uraf_t={} uraf_s={}",
                    cp_pt, sf_num, rec_type, uraf_t, uraf_s
                ));
                // SelectedRegionIdArray (List<ObscuredInt>)
                let sra_off = cached_find_field_offset(ds_class, "SelectedRegionIdArray");
                if sra_off >= 0 {
                    let list_obj = read_ptr_at(dataset_obj, sra_off);
                    if !list_obj.is_null() {
                        let lb = list_obj as *const u8;
                        let llen = std::ptr::read_unaligned::<usize>(
                            lb.add(IL2CPP_LIST_COUNT_OFF) as *const usize
                        );
                        if llen > 0 && llen < 100 {
                            let mut ids: Vec<String> = Vec::new();
                            for i in 0..llen {
                                let elem = lb.add(IL2CPP_LIST_ITEMS_OFF + i * 0x14);
                                let val = read_obscured_int_at(elem as *const c_void, 0);
                                ids.push(val.to_string());
                            }
                            ramen_selected_region_ids_json = ids.join(",");
                        }
                    }
                }
                // ActiveEffectArray (List<ActiveEffectInfo>)
                let ae_off = cached_find_field_offset(ds_class, "ActiveEffectArray");
                if ae_off >= 0 {
                    let list_obj = read_ptr_at(dataset_obj, ae_off);
                    if !list_obj.is_null() {
                        let lb = list_obj as *const u8;
                        let llen = std::ptr::read_unaligned::<usize>(
                            lb.add(IL2CPP_LIST_COUNT_OFF) as *const usize
                        );
                        if llen > 0 && llen < 100 {
                            let first_elem = std::ptr::read_unaligned::<*mut c_void>(
                                lb.add(IL2CPP_LIST_ITEMS_OFF) as *const *mut c_void
                            );
                            if !first_elem.is_null() {
                                let elem_class = std::ptr::read_unaligned::<*mut c_void>(
                                    first_elem as *const *mut c_void
                                );
                                let cat_off = cached_find_field_offset(elem_class, "EffectCategory");
                                let eid_off = cached_find_field_offset(elem_class, "EffectId");
                                let val_off = cached_find_field_offset(elem_class, "EffectValue");
                                let mut effects: Vec<String> = Vec::new();
                                for i in 0..llen {
                                    let ep = std::ptr::read_unaligned::<*mut c_void>(
                                        lb.add(IL2CPP_LIST_ITEMS_OFF + i * IL2CPP_LIST_ITEM_SIZE) as *const *mut c_void
                                    );
                                    if ep.is_null() { continue; }
                                    let cat = if cat_off >= 0 { read_obscured_int_at(ep, cat_off) } else { -1 };
                                    let eid = if eid_off >= 0 { read_obscured_int_at(ep, eid_off) } else { -1 };
                                    let val = if val_off >= 0 { read_obscured_int_at(ep, val_off) } else { -1 };
                                    effects.push(format!(
                                        r#"{{"category":{},"id":{},"value":{}}}"#,
                                        cat, eid, val
                                    ));
                                }
                                ramen_active_effects_raw_json = effects.join(",");
                            }
                        }
                    }
                }
                // FeelingInfoArray (List<FeelingInfo>)
                let fi_off = cached_find_field_offset(ds_class, "FeelingInfoArray");
                if fi_off >= 0 {
                    let list_obj = read_ptr_at(dataset_obj, fi_off);
                    if !list_obj.is_null() {
                        let lb = list_obj as *const u8;
                        let llen = std::ptr::read_unaligned::<usize>(
                            lb.add(IL2CPP_LIST_COUNT_OFF) as *const usize
                        );
                        if llen > 0 && llen < 100 {
                            let first_elem = std::ptr::read_unaligned::<*mut c_void>(
                                lb.add(IL2CPP_LIST_ITEMS_OFF) as *const *mut c_void
                            );
                            if !first_elem.is_null() {
                                let elem_class = std::ptr::read_unaligned::<*mut c_void>(
                                    first_elem as *const *mut c_void
                                );
                                let ft_off = cached_find_field_offset(elem_class, "FeelingIndex");
                                let fv_off = cached_find_field_offset(elem_class, "FeelingId");
                                let mut feelings: Vec<String> = Vec::new();
                                for i in 0..llen {
                                    let ep = std::ptr::read_unaligned::<*mut c_void>(
                                        lb.add(IL2CPP_LIST_ITEMS_OFF + i * IL2CPP_LIST_ITEM_SIZE) as *const *mut c_void
                                    );
                                    if ep.is_null() { continue; }
                                    let ft = if ft_off >= 0 { read_obscured_int_at(ep, ft_off) } else { -1 };
                                    let fv = if fv_off >= 0 { read_obscured_int_at(ep, fv_off) } else { -1 };
                                    feelings.push(format!(
                                        r#"{{"FeelingIndex":{},"FeelingId":{}}}"#,
                                        ft, fv
                                    ));
                                }
                                ramen_feeling_info_json = feelings.join(",");
                            }
                        }
                    }
                }
                ura_log(3, &format!(
                    "ramen arrays: regions={} effects={} feelings={}",
                    !ramen_selected_region_ids_json.is_empty(),
                    !ramen_active_effects_raw_json.is_empty(),
                    !ramen_feeling_info_json.is_empty()
                ));
                log_predict_step("S:ramen arrays");
            } else {
                ura_log(2, "ramen: dataset_obj null");
            }
        } else {
            ura_log(2, "ramen: scenario_obj null");
        }
    }

    // --- Training data via HomeInfoData (ALL scenarios) ---
    log_predict_step("S:ramen end");
    ura_log(3, "★ read_summary phase2: training data");
    log_predict_step("S:p2 training");
    let mut tr_json = "[]".to_string();
    // ★ v3.15.1: collect eval_trainings in same pass (eliminate dangerous double-read)
    let mut eval_trainings: Vec<(i32, [i32; 5], i32, i32, i32, i32, i32, i32)> = Vec::new();
    let home_info_obj = call_getter_on_instance(sm_class, sm_obj, "get_HomeInfoData");
    if !home_info_obj.is_null() {
        let hi_class = find_class(image, to_cstr("Gallop").as_ptr(), to_cstr("WorkSingleModeHomeInfoData").as_ptr());
        if !hi_class.is_null() {
            // CommandInfoArray is a public field (not a getter), at offset 0x10
            let cmd_arr = read_field_value(hi_class, home_info_obj, "CommandInfoArray");
            if !cmd_arr.is_null() {
                let cmd_base = cmd_arr as *const u8;
                let cmd_len = std::ptr::read_unaligned::<usize>(cmd_base.add(IL2CPP_LIST_COUNT_OFF) as *const usize);
                if cmd_len > 0 && cmd_len < 100 {
                    let cmd_elem_class = find_class_by_short_name(image, "SingleModeCommandInfoData");
                    let mut trs = Vec::new();
                    for i in 0..cmd_len {
                        let ep = std::ptr::read_unaligned::<*mut c_void>(cmd_base.add(IL2CPP_LIST_ITEMS_OFF + i * IL2CPP_LIST_ITEM_SIZE) as *const *mut c_void);
                        if ep.is_null() { continue; }

                        let cid = if !cmd_elem_class.is_null() {
                            call_getter_obscured_int(cmd_elem_class, ep, "get_CommandId")
                        } else { -1 };
                        let cname = match cid {
                            CMD_SPEED=>"Speed", CMD_STAMINA=>"Stamina", CMD_GUTS=>"Guts",
                            CMD_POWER=>"Power", CMD_WISDOM=>"Wiz",
                            CMD_URA_SPEED=>"Speed", CMD_URA_STAMINA=>"Stamina", CMD_URA_GUTS=>"Guts",
                            CMD_URA_POWER=>"Power", CMD_URA_WISDOM=>"Wiz",
                            CMD_KAKUSHIMI=>"Kakushimi",
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
                                let al = std::ptr::read_unaligned::<usize>(ab.add(IL2CPP_LIST_COUNT_OFF) as *const usize);
                                al as i32
                            } else { -1 }
                        } else { -1 };

                        // Shining count = TipsEventPartnerArray length
                        let shining = if !cmd_elem_class.is_null() {
                            let arr = call_getter_on_instance(cmd_elem_class, ep, "get_TipsEventPartnerArray");
                            if !arr.is_null() {
                                let ab = arr as *const u8;
                                let al = std::ptr::read_unaligned::<usize>(ab.add(IL2CPP_LIST_COUNT_OFF) as *const usize);
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
                                let pl = std::ptr::read_unaligned::<usize>(pb.add(IL2CPP_LIST_COUNT_OFF) as *const usize);
                                if pl > 0 && pl < 100 {
                                    let pid_class = find_class_by_short_name(image, "SingleModeParamsIncDecInfoData");
                                    for j in 0..pl {
                                        let pe = std::ptr::read_unaligned::<*mut c_void>(pb.add(IL2CPP_LIST_ITEMS_OFF + j * IL2CPP_LIST_ITEM_SIZE) as *const *mut c_void);
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
    log_predict_step("S:p2 done");
    ura_log(3, "★ read_summary phase3: support cards");
    log_predict_step("S:p3 cards");
    let mut sc_json = "[]".to_string();
    let sc_arr = read_field_value(chara_class, chara_obj, "support_card_array");
    if sc_arr.is_null() {
        // Try getter
        let arr = call_getter_on_instance(chara_class, chara_obj, "get_SupportCardArray");
        if !arr.is_null() {
            let ab = arr as *const u8;
            let al = std::ptr::read_unaligned::<usize>(ab.add(IL2CPP_LIST_COUNT_OFF) as *const usize);
            if al > 0 && al < 100 {
                let mut scs = Vec::new();
                for i in 0..al {
                    let ep = std::ptr::read_unaligned::<*mut c_void>(ab.add(IL2CPP_LIST_ITEMS_OFF + i * IL2CPP_LIST_ITEM_SIZE) as *const *mut c_void);
                    if ep.is_null() { continue; }
                    let b = ep as *const u8;
                    let position = std::ptr::read_unaligned::<i32>(b.add(IL2CPP_SUPPORT_CARD_POSITION_OFF) as *const i32);
                    let support_card_id = std::ptr::read_unaligned::<i32>(b.add(IL2CPP_SUPPORT_CARD_ID_OFF) as *const i32);
                    let limit_break_count = std::ptr::read_unaligned::<i32>(b.add(IL2CPP_SUPPORT_CARD_LIMIT_OFF) as *const i32);
                    let training_partner_state = std::ptr::read_unaligned::<i32>(b.add(IL2CPP_LIST_ITEMS_OFF) as *const i32);
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
        let al = std::ptr::read_unaligned::<usize>(ab.add(IL2CPP_LIST_COUNT_OFF) as *const usize);
        if al > 0 && al < 100 {
            let mut scs = Vec::new();
            for i in 0..al {
                let ep = std::ptr::read_unaligned::<*mut c_void>(ab.add(IL2CPP_LIST_ITEMS_OFF + i * IL2CPP_LIST_ITEM_SIZE) as *const *mut c_void);
                if ep.is_null() { continue; }
                let b = ep as *const u8;
                let position = std::ptr::read_unaligned::<i32>(b.add(IL2CPP_SUPPORT_CARD_POSITION_OFF) as *const i32);
                let support_card_id = std::ptr::read_unaligned::<i32>(b.add(IL2CPP_SUPPORT_CARD_ID_OFF) as *const i32);
                let limit_break_count = std::ptr::read_unaligned::<i32>(b.add(IL2CPP_SUPPORT_CARD_LIMIT_OFF) as *const i32);
                let training_partner_state = std::ptr::read_unaligned::<i32>(b.add(IL2CPP_LIST_ITEMS_OFF) as *const i32);
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
                let al = std::ptr::read_unaligned::<usize>(ab.add(IL2CPP_LIST_COUNT_OFF) as *const usize);
                if al > 0 && al < 100 {
                    let mut scs = Vec::new();
                    for i in 0..al {
                        let ep = std::ptr::read_unaligned::<*mut c_void>(ab.add(IL2CPP_LIST_ITEMS_OFF + i * IL2CPP_LIST_ITEM_SIZE) as *const *mut c_void);
                        if ep.is_null() { continue; }
                        let b = ep as *const u8;
                        let position = std::ptr::read_unaligned::<i32>(b.add(IL2CPP_SUPPORT_CARD_POSITION_OFF) as *const i32);
                        let support_card_id = std::ptr::read_unaligned::<i32>(b.add(IL2CPP_SUPPORT_CARD_ID_OFF) as *const i32);
                        let limit_break_count = std::ptr::read_unaligned::<i32>(b.add(IL2CPP_SUPPORT_CARD_LIMIT_OFF) as *const i32);
                        let training_partner_state = std::ptr::read_unaligned::<i32>(b.add(IL2CPP_LIST_ITEMS_OFF) as *const i32);
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
            let al = std::ptr::read_unaligned::<usize>(ab.add(IL2CPP_LIST_COUNT_OFF) as *const usize);
            if al > 0 && al < 100 {
                let mut scs = Vec::new();
                for i in 0..al {
                    let ep = std::ptr::read_unaligned::<*mut c_void>(ab.add(IL2CPP_LIST_ITEMS_OFF + i * IL2CPP_LIST_ITEM_SIZE) as *const *mut c_void);
                    if ep.is_null() { continue; }
                    let b = ep as *const u8;
                    let position = std::ptr::read_unaligned::<i32>(b.add(IL2CPP_SUPPORT_CARD_POSITION_OFF) as *const i32);
                    let support_card_id = std::ptr::read_unaligned::<i32>(b.add(IL2CPP_SUPPORT_CARD_ID_OFF) as *const i32);
                    let limit_break_count = std::ptr::read_unaligned::<i32>(b.add(IL2CPP_SUPPORT_CARD_LIMIT_OFF) as *const i32);
                    let training_partner_state = std::ptr::read_unaligned::<i32>(b.add(IL2CPP_LIST_ITEMS_OFF) as *const i32);
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
    log_predict_step("S:p3 done");
    ura_log(3, "★ read_summary phase4: evaluation");
    log_predict_step("S:p4 eval");
    let mut ev_json = "[]".to_string();
    let ev_arr = read_field_value(chara_class, chara_obj, "evaluation_info_array");
    if ev_arr.is_null() {
        let arr = call_getter_on_instance(chara_class, chara_obj, "get_EvaluationInfoArray");
        if !arr.is_null() {
            let ab = arr as *const u8;
            let al = std::ptr::read_unaligned::<usize>(ab.add(IL2CPP_LIST_COUNT_OFF) as *const usize);
            if al > 0 && al < 1000 {
                let mut evs = Vec::new();
                for i in 0..al {
                    let ep = std::ptr::read_unaligned::<*mut c_void>(ab.add(IL2CPP_LIST_ITEMS_OFF + i * IL2CPP_LIST_ITEM_SIZE) as *const *mut c_void);
                    if ep.is_null() { continue; }
                    let b = ep as *const u8;
                    let target_id = std::ptr::read_unaligned::<i32>(b.add(IL2CPP_TARGET_RACE_ID_OFF) as *const i32);
                    let evaluation = std::ptr::read_unaligned::<i32>(b.add(IL2CPP_TARGET_RACE_EVAL_OFF) as *const i32);
                    let is_appear = std::ptr::read_unaligned::<i32>(b.add(IL2CPP_LIST_ITEMS_OFF) as *const i32);
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
        let al = std::ptr::read_unaligned::<usize>(ab.add(IL2CPP_LIST_COUNT_OFF) as *const usize);
        if al > 0 && al < 1000 {
            let mut evs = Vec::new();
            for i in 0..al {
                let ep = std::ptr::read_unaligned::<*mut c_void>(ab.add(IL2CPP_LIST_ITEMS_OFF + i * IL2CPP_LIST_ITEM_SIZE) as *const *mut c_void);
                if ep.is_null() { continue; }
                let b = ep as *const u8;
                let target_id = std::ptr::read_unaligned::<i32>(b.add(IL2CPP_TARGET_RACE_ID_OFF) as *const i32);
                let evaluation = std::ptr::read_unaligned::<i32>(b.add(IL2CPP_TARGET_RACE_EVAL_OFF) as *const i32);
                let is_appear = std::ptr::read_unaligned::<i32>(b.add(IL2CPP_LIST_ITEMS_OFF) as *const i32);
                evs.push(format!(
                    r#"{{"target_id":{},"evaluation":{},"is_appear":{}}}"#,
                    target_id, evaluation, is_appear
                ));
            }
            ev_json = format!("[{}]", evs.join(","));
        }
    }

    // --- Training levels (graceful fallback) ---
    log_predict_step("S:p4 done");
    ura_log(3, "★ read_summary phase5: training_levels");
    log_predict_step("S:p5 levels");
    let mut tl_json = "[]".to_string();
    let tl_arr = read_field_value(chara_class, chara_obj, "training_level_info_array");
    if tl_arr.is_null() {
        let arr = call_getter_on_instance(chara_class, chara_obj, "get_TrainingLevelInfoArray");
        if !arr.is_null() {
            let ab = arr as *const u8;
            let al = std::ptr::read_unaligned::<usize>(ab.add(IL2CPP_LIST_COUNT_OFF) as *const usize);
            if al > 0 && al < 100 {
                let mut tls = Vec::new();
                for i in 0..al {
                    let ep = std::ptr::read_unaligned::<*mut c_void>(ab.add(IL2CPP_LIST_ITEMS_OFF + i * IL2CPP_LIST_ITEM_SIZE) as *const *mut c_void);
                    if ep.is_null() { continue; }
                    let b = ep as *const u8;
                    let command_id = std::ptr::read_unaligned::<i32>(b.add(IL2CPP_COMMAND_ID_OFF) as *const i32);
                    let level = std::ptr::read_unaligned::<i32>(b.add(IL2CPP_COMMAND_LEVEL_OFF) as *const i32);
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
        let al = std::ptr::read_unaligned::<usize>(ab.add(IL2CPP_LIST_COUNT_OFF) as *const usize);
        if al > 0 && al < 100 {
            let mut tls = Vec::new();
            for i in 0..al {
                let ep = std::ptr::read_unaligned::<*mut c_void>(ab.add(IL2CPP_LIST_ITEMS_OFF + i * IL2CPP_LIST_ITEM_SIZE) as *const *mut c_void);
                if ep.is_null() { continue; }
                let b = ep as *const u8;
                let command_id = std::ptr::read_unaligned::<i32>(b.add(IL2CPP_COMMAND_ID_OFF) as *const i32);
                let level = std::ptr::read_unaligned::<i32>(b.add(IL2CPP_COMMAND_LEVEL_OFF) as *const i32);
                tls.push(format!(
                    r#"{{"command_id":{},"level":{}}}"#,
                    command_id, level
                ));
            }
            tl_json = format!("[{}]", tls.join(","));
        }
    }

    // --- Buffs: chara_effect_ids → readable names (ALL scenarios) + EnhanceGroup (Breeders) ---
    log_predict_step("S:p5 done");
    ura_log(3, "★ read_summary phase6: buffs");
    log_predict_step("S:p6 buffs");
    // ★ v3.14.2: Always generate buffs from chara_effect_ids first
    let mut buff_json = effects_to_buffs_json(&chara_effect_ids);
    // ★ v3.22.21: sid==14 skips try_get_scenario_obj (data pre-read in ramen section)
    let scenario_obj = if sid == 14 {
        ptr::null_mut()
    } else {
        try_get_scenario_obj(chara_class, chara_obj, sid)
    };
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
                                    let el = std::ptr::read_unaligned::<usize>(eb.add(IL2CPP_LIST_COUNT_OFF) as *const usize);
                                    if el > 0 && el < 20 {
                                        let mut buffs = Vec::new();
                                        for i in 0..el {
                                            let ep = std::ptr::read_unaligned::<*mut c_void>(eb.add(IL2CPP_LIST_ITEMS_OFF + i * IL2CPP_LIST_ITEM_SIZE) as *const *mut c_void);
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
                        // (v3.18.7: Use ramen_active_effects_raw_json instead of re-reading from memory,
                        //  because call_getter_on_instance(get_DataSet) can fail in this code path)
                        if sid == 14 && !ramen_active_effects_raw_json.is_empty() {
                            // Convert raw {"category":1,"id":36,"value":50} to named buffs
                            let mut buffs = Vec::new();
                            for ae_part in ramen_active_effects_raw_json.split("},{") {
                                let mut cat: i32 = -1;
                                let mut eid: i32 = 0;
                                let mut val: i32 = 0;
                                // Simple field extraction from {"category":1,"id":36,"value":50}
                                for field in ae_part.trim_start_matches('{').trim_end_matches('}').split(',') {
                                    let fv: Vec<&str> = field.splitn(2, ':').collect();
                                    if fv.len() == 2 {
                                        let key = fv[0].trim();
                                        if key.contains("category") { cat = fv[1].parse().unwrap_or(-1); }
                                        else if key.contains("id") && !key.contains("Eff") { eid = fv[1].parse().unwrap_or(0); }
                                        else if key.contains("value") { val = fv[1].parse().unwrap_or(0); }
                                    }
                                }
                                if cat >= 0 {
                                    // ★ v3.18.8: readable effect name with ID + desc
                                    let cat_name = match cat {
                                        1 => "試食会", 2 => "地域", 4 => "隠し味", _ => "他",
                                    };
                                    let name = format!("{}#{}", cat_name, eid);
                                    // EffectValue = 加成率(%)，试食会/地域効果是训练效果UP百分比
                                    let desc = format!("+{}%", val);
                                    buffs.push(format!(
                                        r#"{{"name":"{}","EffectId":{},"EffectValue":{},"desc":"{}","type":"Ramen"}}"#,
                                        name, eid, val, desc
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

    // ★ v3.22.21: Ramen buffs — extracted outside nested block (uses pre-read data only)
    if sid == 14 && !ramen_active_effects_raw_json.is_empty() {
        let mut buffs = Vec::new();
        for ae_part in ramen_active_effects_raw_json.split("},{") {
            let mut cat: i32 = -1;
            let mut eid: i32 = 0;
            let mut val: i32 = 0;
            for field in ae_part.trim_start_matches('{').trim_end_matches('}').split(',') {
                let fv: Vec<&str> = field.splitn(2, ':').collect();
                if fv.len() == 2 {
                    let key = fv[0].trim();
                    if key.contains("category") { cat = fv[1].parse().unwrap_or(-1); }
                    else if key.contains("id") && !key.contains("Eff") { eid = fv[1].parse().unwrap_or(0); }
                    else if key.contains("value") { val = fv[1].parse().unwrap_or(0); }
                }
            }
            if cat >= 0 {
                let cat_name = match cat {
                    1 => "試食会", 2 => "地域", 4 => "隠し味", _ => "他",
                };
                let name = format!("{}#{}", cat_name, eid);
                let desc = format!("+{}%", val);
                buffs.push(format!(
                    r#"{{"name":"{}","EffectId":{},"EffectValue":{},"desc":"{}","type":"Ramen"}}"#,
                    name, eid, val, desc
                ));
            }
        }
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

    // ★ state field removed: get_State() doesn't exist on WorkSingleModeCharaData
    // Health condition is now detected via chara_effect_ids (top-level array)
    // ★ AI Evaluation (v3.15.1): compute score and training recommendation
    // FIXED: no more double-read of CommandInfoArray — eval_trainings collected in phase2
    log_predict_step("S:buffs done");
    let ai_json = {
        let turn = std::cmp::min((mon - 1) * 2 + (half - 1), 71);
        let stats = [spd, sta, pow_, gut, wiz];

        // Detect buffs from chara_effect_ids
        let has_ai_jiao = chara_effect_ids.iter().any(|&id| id == 8);
        let has_renshou_jouzu = chara_effect_ids.iter().any(|&id| id == 10 || id == 11);

        let result = evaluate_ai(
            turn, stats, vit, mvit, mot, sid,
            &eval_trainings, has_ai_jiao, has_renshou_jouzu,
            skill_eval, skill_count,  // ★ v3.22.0
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

    log_predict_step("S:json");
    format!(
        r#"{{"version":"3.22.21","month":{},"half":{},"scenario":"{}","stats":{{"speed":{},"stamina":{},"power":{},"guts":{},"wiz":{},"vital":{},"max_vital":{},"motivation":"{}","skill_point":{},"fan":{}}},"trainings":{},"support_cards":{},"evaluation":{},"training_levels":{},"buffs":{},"chara_effect_ids":[{}],"skills":{{"eval":{},"count":{},"list":{}}},"ai":{}{}{}}}"#,
        mon, half, scn_s, spd, sta, pow_, gut, wiz, vit, mvit, mot_s, spt, fan, tr_json, sc_json, ev_json, tl_json, buff_json, effect_ids_str.join(","), skill_eval, skill_count, skills_json, ai_json, team_json, ramen_json
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
                Ok(stream) => {
                    // ★ v3.18.8: spawn thread per request — prevents slow endpoint from blocking others
                    let _ = stream.set_read_timeout(Some(std::time::Duration::from_secs(5)));
                    let _ = stream.set_write_timeout(Some(std::time::Duration::from_secs(10)));
                    std::thread::spawn(move || handle_http(stream));
                }
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
        r#"{"status":"ok","version":"3.22.21","endpoints":["/summary","/data","/scenario","/training/predict","/debug/rameninfo","/debug/laststep","/event/recommend","/inherit/compat","/log/turn","/debug/params","/debug/breeders","/debug/cmdinfo","/debug/crashlog","/debug/upload","/debug/dumpclass","/debug/ramenfields","/carddb","/skilldata","/hall","/saddles","/saddles-dl","/log","/status","/health"]}"#.to_string()
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
    } else if path == "/debug/rameninfo" {
        // Dump ramen DataSet raw memory for field layout analysis
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            unsafe { read_ramen_info() }
        })).unwrap_or_else(|_| r#"{"error":"rameninfo_panic"}"#.to_string())
    } else if path == "/debug/laststep" {
        let step = PREDICT_STEP.load(std::sync::atomic::Ordering::Relaxed);
        let len = LAST_STEP_LEN.load(std::sync::atomic::Ordering::Relaxed) as usize;
        let msg = if len > 0 && len < 128 {
            unsafe {
                let buf_ptr = LAST_STEP_BUF.as_ptr();
                std::ffi::CStr::from_ptr(buf_ptr).to_string_lossy().into_owned()
            }
        } else { String::new() };
        format!(r#"{{"step":{},"last_step":"{}"}}"#, step, msg)

    } else if path == "/debug/crashlog" {
        read_crash_log()
    } else if path == "/debug/upload" {
        upload_all_logs()
    } else if path == "/debug/cmdinfo" {
        unsafe { debug_cmdinfo() }
    } else if path.starts_with("/debug/dumpclass") {
        // v3.22.21: Dump all fields of any IL2CPP class by name
        // Usage: /debug/dumpclass?name=WorkSingleModeData
        let class_name = if let Some(q) = path.find("?name=") {
            &path[q+6..]
        } else { "" };
        unsafe { debug_dumpclass(class_name) }
    } else if path == "/debug/ramenfields" {
        // v3.22.21: Dump all ramen array element classes + their fields at runtime
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            unsafe { debug_ramenfields() }
        })).unwrap_or_else(|_| r#"{"error":"ramenfields_panic"}"#.to_string())

    } else if path == "/events" {
        read_events_data()
    } else if path == "/tables" {
        read_mdb_tables()
    } else if path == "/carddb" {

        read_carddb()
    } else if path == "/skilldata" {
        read_skilldata()
    } else if path == "/hall" {
        unsafe { read_hall_data() }
    } else if path == "/training/predict" {
        clear_predict_log();
        let _lock = READ_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            unsafe { read_training_predict() }
        })).unwrap_or_else(|_| r#"{"error":"panic_caught","hint":"read_training_predict panicked"}"#.to_string())
    } else if path == "/event/recommend" {
        unsafe { read_event_recommend() }
    } else if path == "/inherit/compat" {
        unsafe { read_inherit_compat() }
    } else if path == "/log/turn" {
        unsafe { read_turn_log() }
    } else if path == "/ranking" {
        unsafe { read_ranking_data() }
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
        format!(r#"{{"error":"not_found","path":"{}","available":["/scan","/data","/status","/health","/scenario","/debug/upload","/training/predict","/debug/rameninfo","/debug/laststep","/event/recommend","/inherit/compat","/log/turn","/log","/debug/params","/fields","/methods","/singletons","/find_method","/classes","/carddb","/skilldata","/hall","/debug/breeders","/debug/cmdinfo","/debug/dumpclass","/debug/ramenfields","/classes/search/keyword"]}}"#, path)
    };

    save_endpoint_log(&path, &body);

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
// v3.22.21: Pre-cache all class metadata on game thread
// ============================================================

/// Convert PascalCase to snake_case for cache key matching
fn to_snake_case(name: &str) -> String {
    let mut result = String::new();
    for (i, c) in name.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            result.push('_');
        }
        result.extend(c.to_lowercase());
    }
    result
}

/// Pre-cache ALL field offsets for a class (including parent classes)
/// Called on game thread — safe to use IL2CPP API
unsafe fn precache_all_fields(class: *mut c_void) {
    if class.is_null() { return; }
    let get_fields_fn: Option<FnClassGetFields> = {
        let p = resolve_il2cpp_symbol("il2cpp_class_get_fields");
        if p.is_null() { None } else { Some(std::mem::transmute::<*mut c_void, FnClassGetFields>(p)) }
    };
    let get_parent_fn: Option<FnClassGetParent> = {
        let p = resolve_il2cpp_symbol("il2cpp_class_get_parent");
        if p.is_null() { None } else { Some(std::mem::transmute::<*mut c_void, FnClassGetParent>(p)) }
    };
    if get_fields_fn.is_none() { return; }

    let mut current_class = class;
    let mut depth = 0;
    loop {
        if current_class.is_null() || depth > 10 { break; }
        let mut iter: *mut c_void = ptr::null_mut();
        loop {
            let field_info = get_fields_fn.unwrap()(current_class, &mut iter);
            if field_info.is_null() { break; }
            if !(*field_info).name.is_null() {
                let s = std::ffi::CStr::from_ptr((*field_info).name);
                let fname = s.to_string_lossy().to_string();
                let offset = (*field_info).offset;
                // Extract property name from <PropName>k__BackingField
                let prop_name = if fname.starts_with('<') {
                    if let Some(end) = fname.find('>') { &fname[1..end] } else { &fname }
                } else {
                    &fname
                };
                // Store multiple cache keys for robust lookup
                let keys = [
                    format!("{:p}_{}", class, prop_name),
                    format!("{:p}_{}", class, prop_name.to_lowercase()),
                    format!("{:p}_{}", class, to_snake_case(prop_name)),
                ];
                if let Ok(mut guard) = FIELD_OFFSET_CACHE.lock() {
                    if guard.is_none() { *guard = Some(HashMap::new()); }
                    if let Some(ref mut map) = *guard {
                        for k in &keys {
                            map.insert(k.clone(), offset);
                        }
                    }
                }
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
}

/// Pre-cache all known classes and field offsets on game thread
unsafe fn precache_metadata() {
    ura_log(2, "v3.22.21 precache_metadata: starting");
    let image = match get_image() {
        img if !img.is_null() => img,
        _ => { ura_log(1, "precache_metadata: image null"); return; }
    };

    // Classes found via find_class(image, "Gallop", X)
    let gallop_classes = [
        "WorkDataManager", "WorkSingleModeData", "WorkSingleModeCharaData",
        "WorkSingleModeHomeInfoData", "WorkSingleModeScenarioRamen",
        "WorkSingleModeScenarioURA", "WorkSingleModeScenarioTeamRace",
        "WorkSingleModeScenarioLive", "WorkSingleModeScenarioFree",
        "WorkSingleModeScenarioVenus", "WorkSingleModeScenarioArc",
        "WorkSingleModeScenarioSport", "WorkSingleModeScenarioCook",
        "WorkSingleModeScenarioMecha", "WorkSingleModeScenarioLegend",
        "WorkSingleModeScenarioPioneer", "WorkSingleModeScenarioOnsen",
        "WorkSingleModeScenarioBreeders",
    ];

    // Classes found via find_class_by_short_name
    let short_name_classes = [
        "SingleModeSkillData", "SingleModeCommandInfoData", "SingleModeParamsIncDecInfoData",
        "ObscuredSingleModeBreedersEnhanceGroup", "ObscuredSingleModeBreedersCommandInfo",
        "WorkSingleModeScenarioRamenDataSet",
        "ObscuredSingleModeRamenFeeling", "ObscuredSingleModeRamenFeelingTurnInfo",
        "ObscuredSingleModeRamenCommandFeelingInfo", "ObscuredSingleModeRamenFeelingReduceTurnInfo",
        "ObscuredSingleModeRamenUrafEffectInfo", "ObscuredSingleModeRamenActiveEffectInfo",
        "WorkTrainedCharaData", "TrainedCharaData", "SuccessionCharaInfo",
        "WorkSingleModeScenarioURADataSet", "WorkSingleModeScenarioTeamRaceDataSet",
        "WorkSingleModeScenarioLiveDataSet", "WorkSingleModeScenarioFreeDataSet",
        "WorkSingleModeScenarioVenusDataSet", "WorkSingleModeScenarioArcDataSet",
        "WorkSingleModeScenarioSportDataSet", "WorkSingleModeScenarioCookDataSet",
        "WorkSingleModeScenarioMechaDataSet", "WorkSingleModeScenarioLegendDataSet",
        "WorkSingleModeScenarioPioneerDataSet", "WorkSingleModeScenarioOnsenDataSet",
        "WorkSingleModeScenarioBreedersDataSet",
    ];

    let mut cached_count = 0i32;

    // Cache Gallop namespace classes
    for name in &gallop_classes {
        let cls = find_class(image, to_cstr("Gallop").as_ptr(), to_cstr(name).as_ptr());
        if !cls.is_null() {
            if let Ok(mut guard) = CLASS_CACHE.lock() {
                if guard.is_none() { *guard = Some(HashMap::new()); }
                if let Some(ref mut map) = *guard {
                    map.insert(name.to_string(), cls as usize);
                }
            }
            precache_all_fields(cls);
            cached_count += 1;
        }
    }

    // Cache short-name classes
    for name in &short_name_classes {
        let cls = find_class_by_short_name(image, name);
        if !cls.is_null() {
            if let Ok(mut guard) = CLASS_CACHE.lock() {
                if guard.is_none() { *guard = Some(HashMap::new()); }
                if let Some(ref mut map) = *guard {
                    map.insert(name.to_string(), cls as usize);
                }
            }
            precache_all_fields(cls);
            cached_count += 1;
        }
    }

    // Cache WorkDataManager singleton
    if let Some(wdm_cls) = CLASS_CACHE.lock().ok().and_then(|g| g.as_ref().and_then(|m| m.get("WorkDataManager").copied())) {
        let wdm_ptr = wdm_cls as *mut c_void;
        let inst = get_singleton(wdm_ptr);
        if !inst.is_null() {
            if let Ok(mut guard) = SINGLETON_CACHE.lock() {
                if guard.is_none() { *guard = Some(HashMap::new()); }
                if let Some(ref mut map) = *guard {
                    map.insert(wdm_cls, inst as usize);
                }
            }
            ura_log(2, &format!("precache_metadata: WDM singleton cached at {:p}", inst));
        }
    }

    // Count cached field offsets
    let field_count = FIELD_OFFSET_CACHE.lock()
        .ok().and_then(|g| g.as_ref().map(|m| m.len())).unwrap_or(0);

    ura_log(2, &format!(
        "v3.22.21 precache_metadata: done — {} classes, {} field offsets cached",
        cached_count, field_count
    ));
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
        // v3.22.21: Pre-cache all IL2CPP metadata on game thread
        precache_metadata();
    }
}

extern "C" fn on_menu_section(ui: *mut c_void, _userdata: *mut c_void) {
    unsafe {
        if API.is_null() || ui.is_null() { return; }
        let api = &*API;

        if let Some(f) = api.gui_ui_heading_fn {
            f(ui, to_cstr("URA Assistant v3.22.21").as_ptr());
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
    init_crash_handler();
    check_and_upload_crash_log();
    ura_log(3, "URA plugin v3.22.21 loaded (Ramen + Kakushimi + AI eval)");

    if let Some(f) = (*API).gui_show_notification_fn {
        f(to_cstr("URA v3.22.21 Loaded!").as_ptr());
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
    let cmd_len = std::ptr::read_unaligned::<usize>(cmd_base.add(IL2CPP_LIST_COUNT_OFF) as *const usize);
    if cmd_len == 0 { return r#"{"error":"cmd_arr_empty"}"#.to_string(); }

    // ★ Safe element type detection: read klass pointer from first element,
    //   then get class name string via il2cpp_class_get_name (no find_class_by_short_name!)
    let get_name_fn = resolve_il2cpp_symbol("il2cpp_class_get_name");

    let mut actual_elem_class_name = "unknown".to_string();
    let mut elem_is_info_type = true; // default: plain Int32 (safer for small objects)

    // Quick scan: find first command with params to detect element type
    let cmd_limit_detect = std::cmp::min(cmd_len, 5);
    'detect: for i in 0..cmd_limit_detect {
        let elem_ptr = std::ptr::read_unaligned::<*mut c_void>(cmd_base.add(IL2CPP_LIST_ITEMS_OFF + i * IL2CPP_LIST_ITEM_SIZE) as *const *mut c_void);
        if elem_ptr.is_null() { continue; }
        let params_arr = call_getter_on_instance(cmd_elem_class, elem_ptr, "get_ParamsIncDecInfoArray");
        if params_arr.is_null() { continue; }
        let p_base = params_arr as *const u8;
        let p_len = std::ptr::read_unaligned::<usize>(p_base.add(IL2CPP_LIST_COUNT_OFF) as *const usize);
        if p_len == 0 { continue; }
        // Read first element's klass pointer
        let first_elem = std::ptr::read_unaligned::<*mut c_void>(p_base.add(IL2CPP_LIST_ITEMS_OFF) as *const *mut c_void);
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
        let elem_ptr = std::ptr::read_unaligned::<*mut c_void>(cmd_base.add(IL2CPP_LIST_ITEMS_OFF + i * IL2CPP_LIST_ITEM_SIZE) as *const *mut c_void);
        if elem_ptr.is_null() { continue; }

        let params_arr = call_getter_on_instance(cmd_elem_class, elem_ptr, "get_ParamsIncDecInfoArray");
        if params_arr.is_null() { continue; }

        let p_base = params_arr as *const u8;
        let p_len = std::ptr::read_unaligned::<usize>(p_base.add(IL2CPP_LIST_COUNT_OFF) as *const usize);
        if p_len == 0 || p_len > 20 { continue; }

        // Only first 3 params per command
        let p_limit = std::cmp::min(p_len, 3);
        for j in 0..p_limit {
            let p_elem = std::ptr::read_unaligned::<*mut c_void>(p_base.add(IL2CPP_LIST_ITEMS_OFF + j * IL2CPP_LIST_ITEM_SIZE) as *const *mut c_void);
            if p_elem.is_null() { continue; }

            let p_elem_bytes = p_elem as *const u8;

            // ★ Method A: ObscuredInt field XOR decryption (Data layout offsets 0x10, 0x24)
            let tt_crypto = std::ptr::read_unaligned::<i32>(p_elem_bytes.add(IL2CPP_OBSCURED_INT_UNBOX_KEY_OFF) as *const i32);
            let tt_hidden = std::ptr::read_unaligned::<i32>(p_elem_bytes.add(IL2CPP_OBSCURED_INT_UNBOX_HIDDEN_OFF) as *const i32);
            let tt_decrypted = tt_hidden ^ tt_crypto;
            let val_crypto = std::ptr::read_unaligned::<i32>(p_elem_bytes.add(IL2CPP_OBSCURED_INT_PAIR2_KEY_OFF) as *const i32);
            let val_hidden = std::ptr::read_unaligned::<i32>(p_elem_bytes.add(IL2CPP_OBSCURED_INT_PAIR2_HIDDEN_OFF) as *const i32);
            let val_decrypted = val_hidden ^ val_crypto;

            // ★ Method B: Plain Int32 read (Info layout: 0x10, 0x14)
            let plain_tt = std::ptr::read_unaligned::<i32>(p_elem_bytes.add(IL2CPP_OBSCURED_INT_KEY_OFF) as *const i32);
            let plain_val = std::ptr::read_unaligned::<i32>(p_elem_bytes.add(IL2CPP_OBSCURED_INT_HIDDEN_OFF) as *const i32);

            // ★ Method C: Auto-detected correct reading based on element class name
            let (auto_tt, auto_val) = if elem_is_info_type {
                (plain_tt, plain_val)
            } else {
                (tt_decrypted, val_decrypted)
            };

            // ★ Raw hex dump of first 0x20 bytes (enough for both layouts)
            let mut hex_dump = String::new();
            for b in 0..0x20 {  // dump first 32 bytes for debug
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
            let elen = std::ptr::read_unaligned::<usize>(ebase.add(IL2CPP_LIST_COUNT_OFF) as *const usize);
            for i in 0..elen {
                let ep = std::ptr::read_unaligned::<*mut c_void>(ebase.add(IL2CPP_LIST_ITEMS_OFF + i * IL2CPP_LIST_ITEM_SIZE) as *const *mut c_void);
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
    let ml = std::ptr::read_unaligned::<usize>(mb.add(IL2CPP_LIST_COUNT_OFF) as *const usize);

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
        let first_ep = std::ptr::read_unaligned::<*mut c_void>(mb.add(IL2CPP_LIST_ITEMS_OFF) as *const *mut c_void);
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
        let ep = std::ptr::read_unaligned::<*mut c_void>(mb.add(IL2CPP_LIST_ITEMS_OFF + i * IL2CPP_LIST_ITEM_SIZE) as *const *mut c_void);
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
        for b in 0..0x80 {  // dump first 128 bytes for debug
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
                        let al = std::ptr::read_unaligned::<usize>(ab.add(IL2CPP_LIST_COUNT_OFF) as *const usize);
                        if al > 0 {
                            let first_ep = std::ptr::read_unaligned::<*mut c_void>(ab.add(IL2CPP_LIST_ITEMS_OFF) as *const *mut c_void);
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


/// /tables - List all tables in MasterDB for discovery
/// /tables - List all tables in MasterDB for discovery
fn read_mdb_tables() -> String {
    let mdb_path = match find_mdb_path() {
        Some(p) => p,
        None => return r#"{"error":"mdb_not_found"}"#.to_string(),
    };

    let conn = match Connection::open_with_flags(&mdb_path, OpenFlags::SQLITE_OPEN_READ_ONLY) {
        Ok(c) => c,
        Err(e) => return format!(r#"{{"error":"open_failed","detail":"{}"}}"#, e),
    };

    // Collect table names first (can't query_row while stmt is active)
    let single_mode_names: Vec<String> = match conn.prepare(
        "SELECT name FROM sqlite_master WHERE type='table' AND name LIKE '%single_mode%' ORDER BY name"
    ) {
        Ok(mut stmt) => stmt.query_map([], |row| {
            Ok(row.get::<_, String>(0).unwrap_or_default())
        }).unwrap().filter_map(|r| r.ok()).collect(),
        Err(e) => return format!(r#"{{"error":"table_list_failed","detail":"{}"}}"#, e),
    };

    let event_names: Vec<String> = match conn.prepare(
        "SELECT name FROM sqlite_master WHERE type='table' AND (name LIKE '%event%' OR name LIKE '%story%' OR name LIKE '%choice%' OR name LIKE '%gain%' OR name LIKE '%condition%') ORDER BY name"
    ) {
        Ok(mut stmt) => stmt.query_map([], |row| {
            Ok(row.get::<_, String>(0).unwrap_or_default())
        }).unwrap().filter_map(|r| r.ok()).collect(),
        Err(_) => Vec::new(),
    };

    // Now get row counts separately (no active stmt borrowing conn)
    let mut tables_json: Vec<String> = Vec::new();
    for name in &single_mode_names {
        let safe_name = name.replace("]", "]]");
        let count: i32 = conn.query_row(
            &format!("SELECT COUNT(*) FROM [{}]", safe_name), [], |r| r.get(0)
        ).unwrap_or(0);
        tables_json.push(format!(r#"{{"name":"{}","rows":{}}}"#, json_escape(name), count));
    }

    let mut event_json: Vec<String> = Vec::new();
    for name in &event_names {
        let safe_name = name.replace("]", "]]");
        let count: i32 = conn.query_row(
            &format!("SELECT COUNT(*) FROM [{}]", safe_name), [], |r| r.get(0)
        ).unwrap_or(0);
        event_json.push(format!(r#"{{"name":"{}","rows":{}}}"#, json_escape(name), count));
    }

    drop(conn);

    format!(
        r#"{{"ok":true,"single_mode_tables":[{}],"event_tables":[{}]}}"#,
        tables_json.join(","), event_json.join(","),
    )
}

/// /events - Read event data from MasterDB
/// Supports: ?card_id=XXX (filter by support_card_id)
fn read_events_data() -> String {
    let mdb_path = match find_mdb_path() {
        Some(p) => p,
        None => return r#"{"error":"mdb_not_found"}"#.to_string(),
    };

    let conn = match Connection::open_with_flags(&mdb_path, OpenFlags::SQLITE_OPEN_READ_ONLY) {
        Ok(c) => c,
        Err(e) => return format!(r#"{{"error":"open_failed","detail":"{}"}}"#, e),
    };

    // 1. Read single_mode_story_data (event metadata)
    let stories: Vec<String> = match conn.prepare(
        "SELECT id, story_id, short_story_id, card_id, card_chara_id, support_card_id, support_chara_id, show_progress1, show_progress2, show_progress3, show_clear, show_succession, ending_type, race_event_flag, event_category FROM single_mode_story_data ORDER BY id"
    ) {
        Ok(mut stmt) => stmt.query_map([], |row| {
            Ok(format!(
                r#"{{"id":{},"story_id":{},"short_story_id":{},"card_id":{},"card_chara_id":{},"support_card_id":{},"support_chara_id":{},"show_progress1":{},"show_progress2":{},"show_progress3":{},"show_clear":{},"show_succession":{},"ending_type":{},"race_event_flag":{},"event_category":{}}}"#,
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
                row.get::<_, i32>(13).unwrap_or(0),
                row.get::<_, i32>(14).unwrap_or(0),
            ))
        }).unwrap().filter_map(|r| r.ok()).collect(),
        Err(e) => return format!(r#"{{"error":"story_query_failed","detail":"{}"}}"#, e),
    };

    // 2. Read single_mode_event_choice_reward (choice rewards)
    let choices: Vec<String> = match conn.prepare(
        "SELECT id, disp_type, effect_value_type0, effect_value_type1, effect_value_type2 FROM single_mode_event_choice_reward ORDER BY id"
    ) {
        Ok(mut stmt) => stmt.query_map([], |row| {
            Ok(format!(
                r#"{{"id":{},"disp_type":{},"evt0":{},"evt1":{},"evt2":{}}}"#,
                row.get::<_, i32>(0).unwrap_or(0),
                row.get::<_, i32>(1).unwrap_or(0),
                row.get::<_, i32>(2).unwrap_or(0),
                row.get::<_, i32>(3).unwrap_or(0),
                row.get::<_, i32>(4).unwrap_or(0),
            ))
        }).unwrap().filter_map(|r| r.ok()).collect(),
        Err(e) => return format!(r#"{{"error":"choice_query_failed","detail":"{}"}}"#, e),
    };

    // 3. Read event_choice_reward_gain_param (actual stat gains - decrypted from ObscuredInt)
    let gains: Vec<String> = match conn.prepare(
        "SELECT id, display_id, effect_value0, effect_value1, effect_value2 FROM event_choice_reward_gain_param ORDER BY id"
    ) {
        Ok(mut stmt) => stmt.query_map([], |row| {
            Ok(format!(
                r#"{{"id":{},"display_id":{},"ev0":{},"ev1":{},"ev2":{}}}"#,
                row.get::<_, i32>(0).unwrap_or(0),
                row.get::<_, i32>(1).unwrap_or(0),
                row.get::<_, i32>(2).unwrap_or(0),
                row.get::<_, i32>(3).unwrap_or(0),
                row.get::<_, i32>(4).unwrap_or(0),
            ))
        }).unwrap().filter_map(|r| r.ok()).collect(),
        Err(e) => return format!(r#"{{"error":"gain_query_failed","detail":"{}"}}"#, e),
    };

    // 4. Read event titles from text_data
    // Category 45 = single mode story title (guessed, verified via /tables)
    // Use [index] instead of "index" to avoid Rust string escaping issues
    let titles: Vec<String> = match conn.prepare(
        &format!("SELECT [index], text FROM text_data WHERE category={} ORDER BY [index]", TEXT_DATA_CATEGORY_STORY_TITLE)
    ) {
        Ok(mut stmt) => stmt.query_map([], |row| {
            let text: String = row.get::<_, Option<String>>(1).unwrap_or(None).unwrap_or_default();
            Ok(format!(r#"{{"id":{},"title":"{}"}}"#,
                row.get::<_, i32>(0).unwrap_or(0),
                json_escape(&text),
            ))
        }).unwrap().filter_map(|r| r.ok()).collect(),
        Err(_) => Vec::new(),
    };

    drop(conn);

    format!(
        r#"{{"ok":true,"version":"3.22.21","story_count":{},"choice_count":{},"gain_count":{},"title_count":{},"stories":[{}],"choices":[{}],"gains":[{}],"titles":[{}]}}"#,
        stories.len(), choices.len(), gains.len(), titles.len(),
        stories.join(","), choices.join(","), gains.join(","), titles.join(","),
    )
}

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
        r#"{{"ok":true,"version":"3.22.21","mdb":"{}","card_count":{},"effect_count":{},"cards":[{}],"effects":[{}]}}"#,
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
        &format!("SELECT id, text FROM text_data WHERE category={} ORDER BY id", TEXT_DATA_CATEGORY_SKILL_NAME)
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
        r#"{{"ok":true,"version":"3.22.21","mdb":"{}","skill_count":{},"name_count":{},"point_count":{},"skills":[{}],"names":[{}],"need_points":[{}]}}"#,
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
        &format!("SELECT [index], text FROM text_data WHERE category={} ORDER BY [index]", TEXT_DATA_CATEGORY_RACE_NAME)
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
        &format!("SELECT [index], text FROM text_data WHERE category={} ORDER BY [index]", TEXT_DATA_CATEGORY_CHARA_NAME)
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
        r#"{{"ok":true,"version":"3.22.21","mdb":"{}","saddle_count":{},"program_chara_count":{},"program_count":{},"race_name_count":{},"chara_name_count":{},"relation_count":{},"member_count":{},"race_instance_count":{},"saddles":[{}],"chara_programs":[{}],"programs":[{}],"race_names":[{}],"chara_names":[{}],"relations":[{}],"relation_members":[{}],"race_instances":[{}]}}"#,
        mdb_path, saddles.len(), chara_programs.len(), programs.len(),
        race_names.len(), chara_names.len(), relations.len(), relation_members.len(), race_instances.len(),
        saddles.join(","), chara_programs.join(","), programs.join(","),
        race_names.join(","), chara_names.join(","),
        relations.join(","), relation_members.join(","),
        race_instances.join(","),
    )
}

/// /hall - Read 殿堂 (Hall of Fame) data via TrainedCharaData
/// Path: WDM -> get_TrainedCharaData -> WorkTrainedCharaData -> get_List -> List<TrainedCharaData>
/// Each TrainedCharaData has get_RankScore (評価点), get_Speed/Stamina/Power/Guts/Wiz, etc.
/// _rankScore is the game's own calculated評価点 (gold standard for verification)
unsafe fn read_hall_data() -> String {
    if API.is_null() { return r#"{"error":"api_null"}"#.to_string(); }
    let image = match get_image() {
        img if !img.is_null() => img,
        _ => return r#"{"error":"image_null"}"#.to_string(),
    };

    // 1. Get WDM singleton
    let wdm_class = find_class(image, to_cstr("Gallop").as_ptr(), to_cstr("WorkDataManager").as_ptr());
    if wdm_class.is_null() { return r#"{"error":"no_wdm"}"#.to_string(); }
    let wdm_inst = get_singleton(wdm_class);
    if wdm_inst.is_null() { return r#"{"error":"no_wdm_inst"}"#.to_string(); }

    // 2. Get WorkTrainedCharaData from WDM
    let wtcd_inst = call_getter_ref(wdm_class, wdm_inst, "get_TrainedCharaData");
    if wtcd_inst.is_null() {
        ura_log(1, "/hall: get_TrainedCharaData returned null");
        return r#"{"error":"no_trained_chara_data"}"#.to_string();
    }
    ura_log(2, "/hall: got WorkTrainedCharaData instance");

    // 3. Find WorkTrainedCharaData class for calling get_List
    let wtcd_class = find_class_by_short_name(image, "WorkTrainedCharaData");

    // 4. Get List<TrainedCharaData> from WorkTrainedCharaData
    let list_obj = call_getter_ref(wtcd_class, wtcd_inst, "get_List");
    if list_obj.is_null() {
        ura_log(1, "/hall: get_List returned null");
        return r#"{"error":"no_list"}"#.to_string();
    }

    // 5. Read List<TrainedCharaData> internals
    // List<T> IL2CPP layout (64-bit):
    //   +0x00: Il2CppObject header (16 bytes)
    //   +0x10: _items (Il2CppArray* pointer, 8 bytes)
    //   +0x18: _size (int32, 4 bytes)
    let list_base = list_obj as *const u8;
    let items_arr = std::ptr::read_unaligned::<*mut c_void>(list_base.add(IL2CPP_LIST_ARRAY_OFF) as *const *mut c_void);
    let list_size = std::ptr::read_unaligned::<usize>(list_base.add(IL2CPP_LIST_COUNT_OFF) as *const usize) as i32;

    if items_arr.is_null() || list_size <= 0 {
        ura_log(1, &format!("/hall: List null or empty, size={}", list_size));
        return format!(r#"{{"error":"empty_list","list_size":{}}}"#, list_size);
    }
    ura_log(2, &format!("/hall: List has {} entries", list_size));

    // 6. Find TrainedCharaData class
    let tcd_class = find_class_by_short_name(image, "TrainedCharaData");
    if tcd_class.is_null() {
        ura_log(1, "/hall: TrainedCharaData class not found");
        return r#"{"error":"no_tcd_class"}"#.to_string();
    }

    // 7. Read array elements from List._items
    // Il2CppArray layout: +0x18: max_length (usize), +0x20: data[0]
    let arr_base = items_arr as *const u8;
    let arr_len = std::ptr::read_unaligned::<usize>(arr_base.add(IL2CPP_LIST_COUNT_OFF) as *const usize);

    let mut entries = Vec::new();
    let count = std::cmp::min(list_size as usize, std::cmp::min(arr_len, 200));

    for i in 0..count {
        let elem_ptr = std::ptr::read_unaligned::<*mut c_void>(arr_base.add(IL2CPP_LIST_ITEMS_OFF + i * IL2CPP_LIST_ITEM_SIZE) as *const *mut c_void);
        if elem_ptr.is_null() { continue; }

        // Read fields via getter methods
        let card_id = call_getter_int(tcd_class, elem_ptr, "get_CardId");
        let speed = call_getter_int(tcd_class, elem_ptr, "get_Speed");
        let stamina = call_getter_int(tcd_class, elem_ptr, "get_Stamina");
        let power = call_getter_int(tcd_class, elem_ptr, "get_Power");
        let guts = call_getter_int(tcd_class, elem_ptr, "get_Guts");
        let wiz = call_getter_int(tcd_class, elem_ptr, "get_Wiz");
        let rank_score = call_getter_int(tcd_class, elem_ptr, "get_RankScore");
        let rank = call_getter_int(tcd_class, elem_ptr, "get_Rank");
        let scenario_id = call_getter_obscured_int(tcd_class, elem_ptr, "get_ScenarioId");
        let fans = call_getter_int(tcd_class, elem_ptr, "get_Fans");
        let rarity = call_getter_obscured_int(tcd_class, elem_ptr, "get_Rarity");

        // Skip entries with no meaningful data
        if speed <= 0 && stamina <= 0 && rank_score <= 0 { continue; }

        entries.push(format!(
            r#"{{"idx":{},"card_id":{},"speed":{},"stamina":{},"power":{},"guts":{},"wiz":{},"rank_score":{},"rank":{},"scenario_id":{},"fans":{},"rarity":{}}}"#,
            i, card_id, speed, stamina, power, guts, wiz, rank_score, rank, scenario_id, fans, rarity
        ));
    }

    if entries.is_empty() {
        return r#"{"error":"no_valid_entries"}"#.to_string();
    }

    ura_log(2, &format!("/hall: {} valid entries", entries.len()));
    format!(r#"{{"count":{},"entries":[{}]}}"#, entries.len(), entries.join(","))
}

/// /ranking - Ranking data is server-side (not in local IL2CPP memory)
/// Verified: Sprint class doesn't exist (search=0 results), ranking fetched from game server API
unsafe fn read_ranking_data() -> String {
    r#"{"error":"server_side_data","hint":"ランキング data is fetched from game server, not stored locally"}"#.to_string()
}

// ============================================================
// ★ v3.22.0: 4 new endpoints for training prediction, event recommendation,
//   inheritance compatibility, and turn log
// ============================================================


/// v3.22.21: /debug/dumpclass?name=ClassName — Dump all fields of any IL2CPP class
/// Uses il2cpp_class_get_fields (metadata only, no runtime_invoke)
unsafe fn debug_dumpclass(class_name: &str) -> String {
    if class_name.is_empty() {
        return r#"{"error":"missing ?name= parameter"}"#.to_string();
    }
    if API.is_null() { return r#"{"error":"api_null"}"#.to_string(); }
    let image = match get_image() {
        img if !img.is_null() => img,
        _ => return r#"{"error":"image_null"}"#.to_string(),
    };

    // Try to find class by short name
    let class = find_class_by_short_name(image, class_name);
    if class.is_null() {
        return format!(r#"{{"error":"class_not_found","name":"{}"}}"#, class_name);
    }

    // Get class name from IL2CPP
    let get_name_fn = resolve_il2cpp_symbol("il2cpp_class_get_name");
    let real_name = if !get_name_fn.is_null() {
        let get_name: FnClassGetName = std::mem::transmute(get_name_fn);
        let name_ptr = get_name(class);
        if !name_ptr.is_null() {
            std::ffi::CStr::from_ptr(name_ptr).to_string_lossy().to_string()
        } else { String::new() }
    } else { String::new() };

    // Enumerate all fields (including parent classes)
    let fields_json = enumerate_class_fields(class);

    // Also enumerate methods (for debugging)
    let methods_json = enumerate_class_methods(class);

    format!(
        r#"{{"requested":"{}","found":"{}","fields":{},"methods":{}}}"#,
        class_name, real_name, fields_json, methods_json
    )
}

/// v3.22.21: /debug/ramenfields — Walk all ramen arrays, dump element class + fields
/// For each array: read first element, get class from object header, dump all fields + hex
unsafe fn debug_ramenfields() -> String {
    if API.is_null() { return r#"{"error":"api_null"}"#.to_string(); }
    let image = match get_image() {
        img if !img.is_null() => img,
        _ => return r#"{"error":"image_null"}"#.to_string(),
    };
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
    let ramen_sc_class = find_class(image, to_cstr("Gallop").as_ptr(), to_cstr("WorkSingleModeScenarioRamen").as_ptr());
    if ramen_sc_class.is_null() { return r#"{"error":"no_ramen_sc_class"}"#.to_string(); }
    let ramen_sc_obj = try_get_scenario_obj(chara_class, chara_obj, 14);
    if ramen_sc_obj.is_null() { return r#"{"error":"no_ramen_sc_obj"}"#.to_string(); }
    let ramen_ds_obj = call_getter_ref(ramen_sc_class, ramen_sc_obj, "get_DataSet");
    if ramen_ds_obj.is_null() { return r#"{"error":"no_ramen_ds"}"#.to_string(); }

    // Read DataSet class from object header
    let ds_class = get_class_from_object(ramen_ds_obj);
    let ds_class_name = get_class_name_from_pointer(ds_class);

    // Arrays to dump
    let array_getters = [
        "get_ActiveEffectArray",
        "get_FeelingInfoArray",
        "get_FeelingTurnInfoArray",
        "get_CommandFeelingInfoArray",
        "get_FeelingReduceTurnInfoArray",
    ];

    let mut arrays_json = Vec::new();
    for getter in &array_getters {
        let arr = call_getter_on_instance(ds_class, ramen_ds_obj, getter);
        if arr.is_null() {
            arrays_json.push(format!(r#"{{"getter":"{}","status":"null"}}"#, getter));
            continue;
        }
        let base = arr as *const u8;
        let length = std::ptr::read_unaligned::<usize>(base.add(IL2CPP_LIST_COUNT_OFF) as *const usize);
        if length == 0 || length > 200 {
            arrays_json.push(format!(r#"{{"getter":"{}","status":"empty_or_too_long","len":{}}}"#, getter, length));
            continue;
        }

        // Read first element
        let first_elem = std::ptr::read_unaligned::<*mut c_void>(
            base.add(IL2CPP_LIST_ITEMS_OFF) as *const *mut c_void
        );
        if first_elem.is_null() {
            arrays_json.push(format!(r#"{{"getter":"{}","status":"null_first_elem"}}"#, getter));
            continue;
        }

        // Read class from object header
        let elem_class = get_class_from_object(first_elem);
        let elem_class_name = get_class_name_from_pointer(elem_class);
        let fields_json = enumerate_class_fields(elem_class);

        // Hex dump first 0x80 bytes
        let ep_base = first_elem as *const u8;
        let mut hex_parts: Vec<String> = Vec::new();
        for off in (0..0x80).step_by(4) {
            let val = std::ptr::read_unaligned::<i32>(ep_base.add(off) as *const i32);
            hex_parts.push(format!(r#""0x{:02x}":{}"#, off, val));
        }

        arrays_json.push(format!(
            r#"{{"getter":"{}","length":{},"elem_class":"{}","fields":{},"hex":{{{}}}}}"#,
            getter, length, elem_class_name, fields_json, hex_parts.join(",")
        ));
    }

    // Also dump UrafEffectInfo (single object, not array)
    let uraf_obj = call_getter_on_instance(ds_class, ramen_ds_obj, "get_UrafEffectInfo");
    let uraf_json = if !uraf_obj.is_null() {
        let uraf_class = get_class_from_object(uraf_obj);
        let uraf_class_name = get_class_name_from_pointer(uraf_class);
        let uraf_fields = enumerate_class_fields(uraf_class);
        let ub = uraf_obj as *const u8;
        let mut hex_parts: Vec<String> = Vec::new();
        for off in (0..0x80).step_by(4) {
            let val = std::ptr::read_unaligned::<i32>(ub.add(off) as *const i32);
            hex_parts.push(format!(r#""0x{:02x}":{}"#, off, val));
        }
        format!(
            r#""uraf_effect":{{"class":"{}","fields":{},"hex":{{{}}}}}"#,
            uraf_class_name, uraf_fields, hex_parts.join(",")
        )
    } else {
        r#""uraf_effect":{"status":"null"}"#.to_string()
    };

    format!(
        r#"{{"dataset_class":"{}","arrays":[{}],{}}}"#,
        ds_class_name, arrays_json.join(","), uraf_json
    )
}

/// /debug/cmdinfo — Dump command element class info WITHOUT runtime_invoke on command elements
/// Reads class from object header (offset 0), enumerates fields + methods + hex dump
/// Safe: only uses il2cpp_class_get_fields / il2cpp_class_get_methods (no runtime_invoke on cmd elements)
unsafe fn debug_cmdinfo() -> String {
    if API.is_null() { return r#"{"error":"api_null"}"#.to_string(); }
    let image = match get_image() {
        img if !img.is_null() => img,
        _ => return r#"{"error":"image_null"}"#.to_string(),
    };

    let wdm_class = find_class(image, to_cstr("Gallop").as_ptr(), to_cstr("WorkDataManager").as_ptr());
    if wdm_class.is_null() { return r#"{"error":"no_wdm"}"#.to_string(); }
    let wdm_inst = get_singleton(wdm_class);
    if wdm_inst.is_null() { return r#"{"error":"no_wdm_inst"}"#.to_string(); }
    log_predict_step("P:wdm");

    let sm_class = find_class(image, to_cstr("Gallop").as_ptr(), to_cstr("WorkSingleModeData").as_ptr());
    let sm_obj = call_getter_ref(wdm_class, wdm_inst, "get_SingleMode");
    if sm_obj.is_null() { return r#"{"error":"no_sm"}"#.to_string(); }

    let home_info_obj = call_getter_on_instance(sm_class, sm_obj, "get_HomeInfoData");
    if home_info_obj.is_null() { return r#"{"error":"no_home_info"}"#.to_string(); }
    log_predict_step("got home_info");
    let hi_class = find_class(image, to_cstr("Gallop").as_ptr(), to_cstr("WorkSingleModeHomeInfoData").as_ptr());
    if hi_class.is_null() { return r#"{"error":"no_home_info_class"}"#.to_string(); }

    let cmd_arr = read_field_value(hi_class, home_info_obj, "CommandInfoArray");
    if cmd_arr.is_null() { return r#"{"error":"no_cmd_arr"}"#.to_string(); }

    let cmd_base = cmd_arr as *const u8;
    let cmd_len = std::ptr::read_unaligned::<usize>(cmd_base.add(IL2CPP_LIST_COUNT_OFF) as *const usize);
    if cmd_len == 0 { return r#"{"error":"empty_cmd_arr"}"#.to_string(); }

    // Read first element
    let ep = std::ptr::read_unaligned::<*mut c_void>(cmd_base.add(IL2CPP_LIST_ITEMS_OFF) as *const *mut c_void);
    if ep.is_null() { return r#"{"error":"null_elem"}"#.to_string(); }

    // Read class from object header (offset 0 = Il2CppClass*)
    let elem_class = std::ptr::read_unaligned::<*mut c_void>(ep as *const *mut c_void);
    let class_name = get_class_name_from_pointer(elem_class);

    // Enumerate fields and methods on this class
    let fields_json = enumerate_class_fields(elem_class);
    let methods_json = enumerate_class_methods(elem_class);

    // Hex dump first 0x80 bytes (32 x i32 values)
    let ep_base = ep as *const u8;
    let mut hex_parts: Vec<String> = Vec::new();
    for off in (0..0x80).step_by(4) {
        let val = std::ptr::read_unaligned::<i32>(ep_base.add(off) as *const i32);
        hex_parts.push(format!(r#""0x{:02x}":{}"#, off, val));
    }

    // Also check 2nd element class name (verify all elements are same type)
    let ep2_class_name = if cmd_len > 1 {
        let ep2 = std::ptr::read_unaligned::<*mut c_void>(cmd_base.add(IL2CPP_LIST_ITEMS_OFF + IL2CPP_LIST_ITEM_SIZE) as *const *mut c_void);
        if !ep2.is_null() {
            let ep2_class = std::ptr::read_unaligned::<*mut c_void>(ep2 as *const *mut c_void);
            get_class_name_from_pointer(ep2_class)
        } else { String::new() }
    } else { String::new() };

    format!(
        r#"{{"cmd_len":{},"elem0_class":"{}","elem1_class":"{}","fields":{},"methods":{},"hex":{{{}}}}}"#,
        cmd_len, class_name, ep2_class_name, fields_json, methods_json, hex_parts.join(",")
    )
}

/// /training/predict — Detailed training prediction with NPC partner breakdown
/// Returns per-command: gains, partner details (support card vs NPC), buffs, failure risk
/// Key data sources:
///   - WorkSingleModeData -> get_HomeInfoData -> CommandInfoArray (training layout + partners)
///   - WorkSingleModeCharaData -> CharaEffectBuffArray (active buffs)
///   - WorkSingleModeScenarioRamenDataSet (ramen-specific data, scenario_id==14)
unsafe fn read_ramen_info() -> String {
    if API.is_null() { return r#"{"error":"api_null"}"#.to_string(); }
    let image = match get_image() {
        img if !img.is_null() => img,
        _ => return r#"{"error":"image_null"}"#.to_string(),
    };
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

    let ramen_sc_class = find_class(image, to_cstr("Gallop").as_ptr(), to_cstr("WorkSingleModeScenarioRamen").as_ptr());
    if ramen_sc_class.is_null() { return r#"{"error":"no_ramen_sc_class"}"#.to_string(); }
    let ramen_sc_obj = try_get_scenario_obj(chara_class, chara_obj, 14);
    if ramen_sc_obj.is_null() { return r#"{"error":"no_ramen_sc_obj"}"#.to_string(); }
    let ramen_ds_obj = call_getter_ref(ramen_sc_class, ramen_sc_obj, "get_DataSet");
    if ramen_ds_obj.is_null() { return r#"{"error":"no_ramen_ds"}"#.to_string(); }

    // Read class pointer from object header (offset 0 on 64-bit = Il2CppObject.klass)
    let ds_base = ramen_ds_obj as *const u8;
    let ds_class_ptr = std::ptr::read_unaligned::<*mut c_void>(ds_base as *const *mut c_void);

    // Hex dump first 256 bytes
    let mut hex = String::new();
    for i in 0..256usize {
        let b = std::ptr::read_unaligned::<u8>(ds_base.add(i));
        hex.push_str(&format!("{:02x}", b));
        if (i + 1) % 16 == 0 { hex.push('\n'); } else if (i + 1) % 8 == 0 { hex.push(' '); }
    }

    // Try to read class name via il2cpp class API
    let mut class_name = String::new();
    if !ds_class_ptr.is_null() {
        let get_name_fn = resolve_il2cpp_symbol("il2cpp_class_get_name");
        if !get_name_fn.is_null() {
            let fn_ptr: unsafe extern "C" fn(*mut c_void) -> *const u8 = std::mem::transmute(get_name_fn);
            let name_ptr = fn_ptr(ds_class_ptr);
            if !name_ptr.is_null() {
                let cstr = std::ffi::CStr::from_ptr(name_ptr);
                class_name = cstr.to_string_lossy().into_owned();
            }
        }
    }

    format!(
        r#"{{"ds_ptr":"0x{:x}","ds_class":"0x{:x}","class_name":"{}","hex_dump":"{}"}}"#,
        ramen_ds_obj as usize, ds_class_ptr as usize, class_name, hex
    )
}

unsafe fn read_training_predict() -> String {
    if API.is_null() { return r#"{"error":"api_null"}"#.to_string(); }
    clear_predict_log();
    log_predict_step("P:start");
    let image = match get_image() {
        img if !img.is_null() => img,
        _ => return r#"{"error":"image_null"}"#.to_string(),
    };
    log_predict_step("got image");

    // 1. Get WDM -> SingleMode -> CharaData (standard path)
    let wdm_class = find_class(image, to_cstr("Gallop").as_ptr(), to_cstr("WorkDataManager").as_ptr());
    if wdm_class.is_null() { return r#"{"error":"no_wdm"}"#.to_string(); }
    let wdm_inst = get_singleton(wdm_class);
    if wdm_inst.is_null() { return r#"{"error":"no_wdm_inst"}"#.to_string(); }
    log_predict_step("P:wdm");

    let sm_class = find_class(image, to_cstr("Gallop").as_ptr(), to_cstr("WorkSingleModeData").as_ptr());
    let sm_obj = call_getter_ref(wdm_class, wdm_inst, "get_SingleMode");
    if sm_obj.is_null() { return r#"{"error":"no_sm"}"#.to_string(); }

    let chara_class = find_class(image, to_cstr("Gallop").as_ptr(), to_cstr("WorkSingleModeCharaData").as_ptr());
    let chara_obj = call_getter_ref(sm_class, sm_obj, "get_Character");
    if chara_obj.is_null() { return r#"{"error":"no_chara"}"#.to_string(); }

    let sid = call_getter_int(chara_class, chara_obj, "get_ScenarioId");
    let spd = call_getter_int(chara_class, chara_obj, "get_Speed");
    let sta = call_getter_int(chara_class, chara_obj, "get_Stamina");
    let pow_ = call_getter_int(chara_class, chara_obj, "get_Power");
    let gut = call_getter_int(chara_class, chara_obj, "get_Guts");
    let wiz = call_getter_int(chara_class, chara_obj, "get_Wiz");
    let vit = call_getter_int(chara_class, chara_obj, "get_Hp");
    let mvit = call_getter_int(chara_class, chara_obj, "get_MaxHp");
    let mot = call_getter_int(chara_class, chara_obj, "get_Motivation");
    let spt = call_getter_obscured_int(chara_class, chara_obj, "get_SkillPoint");
    log_predict_step(&format!("chara stats sid={} spd={} sta={}", sid, spd, sta));

    // 2. Get HomeInfoData -> CommandInfoArray
    let home_info_obj = call_getter_on_instance(sm_class, sm_obj, "get_HomeInfoData");
    if home_info_obj.is_null() { return r#"{"error":"no_home_info"}"#.to_string(); }
    log_predict_step("got home_info");
    let hi_class = find_class(image, to_cstr("Gallop").as_ptr(), to_cstr("WorkSingleModeHomeInfoData").as_ptr());
    if hi_class.is_null() { return r#"{"error":"no_home_info_class"}"#.to_string(); }

    // CommandInfoArray is a public field at offset 0x10
    let cmd_arr = read_field_value(hi_class, home_info_obj, "CommandInfoArray");
    if cmd_arr.is_null() { return r#"{"error":"no_cmd_arr"}"#.to_string(); }

    let cmd_base = cmd_arr as *const u8;
    let cmd_len = std::ptr::read_unaligned::<usize>(cmd_base.add(IL2CPP_LIST_COUNT_OFF) as *const usize);
    if cmd_len == 0 || cmd_len > 100 {
        return format!(r#"{{"error":"cmd_len_invalid","len":{}}}"#, cmd_len);
    }

    log_predict_step(&format!("cmd_arr len={}", cmd_len));

    // 3. Read CharaEffectBuffArray for active buffs
    // WorkSingleModeCharaData._charaEffectIdArray + EvaluationList -> CharaEffectBuff
    let effect_ids = read_obscured_int_array(chara_class, chara_obj, "get_CharaEffectIdArray");
    let buffs_from_effects = effects_to_buffs_json(&effect_ids);
    log_predict_step("effects done");

    // 4. Iterate commands — read class from each object's header (offset 0 = Il2CppClass*)
    // This avoids find_class_by_short_name matching wrong class -> runtime_invoke crash
    let mut commands_json: Vec<String> = Vec::new();

    for i in 0..cmd_len {
        let ep = std::ptr::read_unaligned::<*mut c_void>(cmd_base.add(IL2CPP_LIST_ITEMS_OFF + i * IL2CPP_LIST_ITEM_SIZE) as *const *mut c_void);
        if ep.is_null() { continue; }

        // Read class from object header — guaranteed correct class for this object
        let cmd_elem_class = std::ptr::read_unaligned::<*mut c_void>(ep as *const *mut c_void);
        log_predict_step(&format!("cmd[{}] class read", i));

        // Read command fields directly from memory (offsets confirmed by /debug/cmdinfo)
        // SingleModeCommandInfoData: CommandId@0x24/0x28, IsEnable@0x38/0x3c, FailureRate@0x68/0x6c
        let epb = ep as *const u8;
        let cid = {
            let k = std::ptr::read_unaligned::<i32>(epb.add(0x24) as *const i32);
            let h = std::ptr::read_unaligned::<i32>(epb.add(0x28) as *const i32);
            h ^ k
        };
        let is_enable = {
            let k = std::ptr::read_unaligned::<i32>(epb.add(0x38) as *const i32);
            let h = std::ptr::read_unaligned::<i32>(epb.add(0x3c) as *const i32);
            h ^ k
        };
        let failure_rate = {
            let k = std::ptr::read_unaligned::<i32>(epb.add(0x68) as *const i32);
            let h = std::ptr::read_unaligned::<i32>(epb.add(0x6c) as *const i32);
            h ^ k
        };
        log_predict_step(&format!("cmd[{}] cid={} enable={} fail={}", i, cid, is_enable, failure_rate));

        let cname = match cid {
            CMD_SPEED=>"Speed", CMD_STAMINA=>"Stamina", CMD_GUTS=>"Guts",
            CMD_POWER=>"Power", CMD_WISDOM=>"Wiz",
            CMD_URA_SPEED=>"Speed", CMD_URA_STAMINA=>"Stamina", CMD_URA_GUTS=>"Guts",
            CMD_URA_POWER=>"Power", CMD_URA_WISDOM=>"Wiz",
            CMD_KAKUSHIMI=>"Kakushimi",
            301=>"Outing", 390=>"Rest", 401=>"Outing2",
            701=>"Outing3", 801=>"Outing4", _=>"Unknown"
        };

        log_predict_step(&format!("cmd[{}] reading partners", i));
        // Read TrainingPartnerArray — distinguish support cards vs NPCs
        let mut partners_json: Vec<String> = Vec::new();
        let mut support_count: i32 = 0;
        let mut npc_count: i32 = 0;

        {
            let pa = std::ptr::read_unaligned::<*mut c_void>(epb.add(0x50) as *const *mut c_void);
            if !pa.is_null() {
                let pb = pa as *const u8;
                let pl = std::ptr::read_unaligned::<usize>(pb.add(IL2CPP_LIST_COUNT_OFF) as *const usize);
                if pl > 0 && pl < 50 {
                    for j in 0..pl {
                        let pp = std::ptr::read_unaligned::<*mut c_void>(pb.add(IL2CPP_LIST_ITEMS_OFF + j * IL2CPP_LIST_ITEM_SIZE) as *const *mut c_void);
                        if pp.is_null() { continue; }

                        // Read partner class from object header (same pattern as cmd_elem_class above)
                        let pp_class = std::ptr::read_unaligned::<*mut c_void>(pp as *const *mut c_void);
                        let obj_class_name = get_object_class_name(pp);
                        let is_support_card = obj_class_name.contains("SingleModeTrainingPartnerEntity")
                            && !obj_class_name.contains("EtcChara")
                            && !obj_class_name.contains("UniqueChara")
                            && !obj_class_name.contains("Scout");

                        if is_support_card {
                            partners_json.push(r#"{"type":"support_card"}"#.to_string());
                            support_count += 1;
                        } else {
                            partners_json.push(r#"{"type":"npc"}"#.to_string());
                            npc_count += 1;
                        }
                    }
                }
            }
        }

        log_predict_step(&format!("cmd[{}] partners done s={} n={}", i, support_count, npc_count));

        // Read TipsEventPartnerArray (shining partners)
        let shining_count = {
            let arr = std::ptr::read_unaligned::<*mut c_void>(epb.add(0x58) as *const *mut c_void);
            if !arr.is_null() {
                let ab = arr as *const u8;
                let al = std::ptr::read_unaligned::<usize>(ab.add(IL2CPP_LIST_COUNT_OFF) as *const usize);
                al as i32
            } else { 0 }
        };

        log_predict_step(&format!("cmd[{}] reading params", i));
        // Read ParamsIncDecInfoArray (training gains)
        let mut gains_json: Vec<String> = Vec::new();
        let mut stat_gains = [0i32; 5]; // [Speed, Stamina, Power, Guts, Wisdom]
        let mut skill_pt_gain: i32 = 0;
        let mut vital_cost: i32 = 0;

        {
            let pa = std::ptr::read_unaligned::<*mut c_void>(epb.add(0x60) as *const *mut c_void);
            if !pa.is_null() {
                let pb = pa as *const u8;
                let pl = std::ptr::read_unaligned::<usize>(pb.add(IL2CPP_LIST_COUNT_OFF) as *const usize);
                if pl > 0 && pl < 100 {
                    for j in 0..pl {
                        let pe = std::ptr::read_unaligned::<*mut c_void>(pb.add(IL2CPP_LIST_ITEMS_OFF + j * IL2CPP_LIST_ITEM_SIZE) as *const *mut c_void);
                        if pe.is_null() { continue; }
                        // Read ObscuredInt fields directly: TargetType@0x10/0x14, Value@0x24/0x28
                        let peb = pe as *const u8;
                        let tt = {
                            let k = std::ptr::read_unaligned::<i32>(peb.add(0x10) as *const i32);
                            let h = std::ptr::read_unaligned::<i32>(peb.add(0x14) as *const i32);
                            h ^ k
                        };
                        let v = {
                            let k = std::ptr::read_unaligned::<i32>(peb.add(0x24) as *const i32);
                            let h = std::ptr::read_unaligned::<i32>(peb.add(0x28) as *const i32);
                            h ^ k
                        };
                        if v == 0 { continue; }
                        let tn = match tt {
                            1=>"Speed", 2=>"Stamina", 3=>"Guts",
                            4=>"Power", 5=>"Wiz", 10=>"HP",
                            20=>"Motivation", 30=>"SkillPt", _=>"Unknown"
                        };
                        gains_json.push(format!(r#""{}":{}"#, tn, v));
                        match tt {
                            1 => stat_gains[0] += v,
                            2 => stat_gains[1] += v,
                            4 => stat_gains[2] += v,
                            3 => stat_gains[3] += v,
                            5 => stat_gains[4] += v,
                            10 => vital_cost += v,
                            30 => skill_pt_gain += v,
                            _ => {}
                        }
                    }
                }
            }
        }

        log_predict_step(&format!("cmd[{}] params done", i));
        commands_json.push(format!(
            r#"{{"name":"{}","command_id":{},"is_enable":{},"failure_rate":{},"shining":{},"support_count":{},"npc_count":{},"partners":[{}],"gains":{{{}}},"stat_gains":{{"speed":{},"stamina":{},"power":{},"guts":{},"wiz":{}}},"skill_pt":{},"vital_cost":{}}}"#,
            cname, cid, is_enable, failure_rate, shining_count, support_count, npc_count,
            partners_json.join(","),
            gains_json.join(","),
            stat_gains[0], stat_gains[1], stat_gains[2], stat_gains[3], stat_gains[4],
            skill_pt_gain, vital_cost
        ));
    }

    log_predict_step(&format!("commands done, ramen sid={}", sid));

    // 5. Ramen scenario data — v3.22.21: Direct memory read (only 2 il2cpp_runtime_invoke)
    let mut ramen_json = String::new();
    if sid == 14 {
        log_predict_step("ramen direct read (v3.22.21)");
        let scenario_obj = try_get_scenario_obj(chara_class, chara_obj, 14);
        if !scenario_obj.is_null() {
            let sc_class = std::ptr::read_unaligned::<*mut c_void>(
                scenario_obj as *const *mut c_void
            );
            let dataset_obj = call_getter_ref(sc_class, scenario_obj, "get_DataSet");
            if !dataset_obj.is_null() {
                let ds_class = std::ptr::read_unaligned::<*mut c_void>(
                    dataset_obj as *const *mut c_void
                );
                let (cp_pt, sf_num, rec_type, uraf_t, uraf_s) =
                    read_ramen_scalar_fields(ds_class, dataset_obj);
                log_predict_step(&format!(
                    "ramen: cp={} sf={} rec={} uraf_t={} uraf_s={}",
                    cp_pt, sf_num, rec_type, uraf_t, uraf_s
                ));
                ramen_json = format!(
                    r#","ramen":{{"checkpoint_pt":{},"special_feeling_num":{},"recommend_type":{},"uraf_type":{},"uraf_state":{}}}"#,
                    cp_pt, sf_num, rec_type, uraf_t, uraf_s
                );
            } else {
                log_predict_step("ramen: dataset null");
            }
        } else {
            log_predict_step("ramen: scenario null");
        }
        if ramen_json.is_empty() {
            ramen_json = r#","ramen":{"available":false,"error":"dataset_null"}"#.to_string();
        }
    }

    log_predict_step("DONE");
    log_predict_step("building json");

    let result = format!(
        r#"{{"version":"3.22.21","scenario_id":{},"stats":{{"speed":{},"stamina":{},"power":{},"guts":{},"wiz":{},"vital":{},"max_vital":{},"motivation":{},"skill_point":{}}},"commands":[{}]{},"buffs":{}}}"#,
        sid, spd, sta, pow_, gut, wiz, vit, mvit, mot, spt,
        commands_json.join(","),
        ramen_json,
        buffs_from_effects
    );
    log_predict_step("json built ok");
    result
}

/// /inherit/compat — Inheritance compatibility calculation
/// Shows exact compatibility values (not just ○△×), split by parent gender,
/// and detects target race overlap
/// Data sources:
///   - SuccessionCharaInfo (parent chara IDs)
///   - SuccessionRelationMember + SuccessionRelation (compatibility data)
///   - mdb succession_relation tables
///   - SingleModeTargetRace (current target races)
unsafe fn read_inherit_compat() -> String {
    if API.is_null() { return r#"{"error":"api_null"}"#.to_string(); }
    let image = match get_image() {
        img if !img.is_null() => img,
        _ => return r#"{"error":"image_null"}"#.to_string(),
    };

    let wdm_class = find_class(image, to_cstr("Gallop").as_ptr(), to_cstr("WorkDataManager").as_ptr());
    if wdm_class.is_null() { return r#"{"error":"no_wdm"}"#.to_string(); }
    let wdm_inst = get_singleton(wdm_class);
    if wdm_inst.is_null() { return r#"{"error":"no_wdm_inst"}"#.to_string(); }
    log_predict_step("P:wdm");

    let sm_class = find_class(image, to_cstr("Gallop").as_ptr(), to_cstr("WorkSingleModeData").as_ptr());
    let sm_obj = call_getter_ref(wdm_class, wdm_inst, "get_SingleMode");
    if sm_obj.is_null() { return r#"{"error":"no_sm"}"#.to_string(); }

    let chara_class = find_class(image, to_cstr("Gallop").as_ptr(), to_cstr("WorkSingleModeCharaData").as_ptr());
    let chara_obj = call_getter_ref(sm_class, sm_obj, "get_Character");
    if chara_obj.is_null() { return r#"{"error":"no_chara"}"#.to_string(); }

    // 1. Read succession parent info
    // WorkSingleModeCharaData.SuccessionTrainedCharaInfoFirst (offset 0x48)
    // WorkSingleModeCharaData.SuccessionTrainedCharaInfoSecond (offset 0x50)
    let sci_class = find_class_by_short_name(image, "SuccessionCharaInfo");
    let first_sci = call_getter_ref(chara_class, chara_obj, "get_SuccessionTrainedCharaInfoFirst");
    let second_sci = call_getter_ref(chara_class, chara_obj, "get_SuccessionTrainedCharaInfoSecond");

    let mut first_chara_id: i32 = -1;
    let mut second_chara_id: i32 = -1;
    if !first_sci.is_null() && !sci_class.is_null() {
        first_chara_id = call_getter_int(sci_class, first_sci, "get_TrainedCharaId");
    }
    if !second_sci.is_null() && !sci_class.is_null() {
        second_chara_id = call_getter_int(sci_class, second_sci, "get_TrainedCharaId");
    }

    // 2. Read SuccessionFactor (offset 0x448 on CharaData) — factor count for compatibility
    let factor_arr = call_getter_on_instance(chara_class, chara_obj, "get_SuccessionFactor");
    let mut factor_count: i32 = 0;
    if !factor_arr.is_null() {
        let fb = factor_arr as *const u8;
        factor_count = std::ptr::read_unaligned::<usize>(fb.add(IL2CPP_LIST_COUNT_OFF) as *const usize) as i32;
    }

    // 3. Read relation data from mdb
    let mut relations_json: Vec<String> = Vec::new();
    let mut relation_members_json: Vec<String> = Vec::new();
    let mut relation_ranks_json: Vec<String> = Vec::new();

    if let Some(mdb_path) = find_mdb_path() {
        if let Ok(conn) = Connection::open_with_flags(&mdb_path, OpenFlags::SQLITE_OPEN_READ_ONLY) {
            // succession_relation: type + point pairs
            if let Ok(mut stmt) = conn.prepare("SELECT relation_type, relation_point FROM succession_relation ORDER BY relation_type") {
                let rows: Vec<String> = stmt.query_map([], |row| {
                    Ok(format!(r#"{{"type":{},"point":{}}}"#,
                        row.get::<_, i32>(0).unwrap_or(0),
                        row.get::<_, i32>(1).unwrap_or(0)))
                }).unwrap().filter_map(|r| r.ok()).collect();
                relations_json = rows;
            }

            // succession_relation_member: id + type + chara_id
            if let Ok(mut stmt) = conn.prepare("SELECT id, relation_type, chara_id FROM succession_relation_member ORDER BY id") {
                let rows: Vec<String> = stmt.query_map([], |row| {
                    Ok(format!(r#"{{"id":{},"type":{},"chara_id":{}}}"#,
                        row.get::<_, i32>(0).unwrap_or(0),
                        row.get::<_, i32>(1).unwrap_or(0),
                        row.get::<_, i32>(2).unwrap_or(0)))
                }).unwrap().filter_map(|r| r.ok()).collect();
                relation_members_json = rows;
            }

            // succession_relation_rank: rank + min + max
            if let Ok(mut stmt) = conn.prepare("SELECT relation_rank, rank_value_min, rank_value_max FROM succession_relation_rank ORDER BY relation_rank") {
                let rows: Vec<String> = stmt.query_map([], |row| {
                    Ok(format!(r#"{{"rank":{},"min":{},"max":{}}}"#,
                        row.get::<_, i32>(0).unwrap_or(0),
                        row.get::<_, i32>(1).unwrap_or(0),
                        row.get::<_, i32>(2).unwrap_or(0)))
                }).unwrap().filter_map(|r| r.ok()).collect();
                relation_ranks_json = rows;
            }

            drop(conn);
        }
    }

    // 4. Read target races for overlap detection
    let mut target_races_json: Vec<String> = Vec::new();
    let tr_arr = call_getter_on_instance(chara_class, chara_obj, "get_TargetRaceArray");
    if !tr_arr.is_null() {
        let trb = tr_arr as *const u8;
        let trl = std::ptr::read_unaligned::<usize>(trb.add(IL2CPP_LIST_COUNT_OFF) as *const usize);
        if trl > 0 && trl < 50 {
            for ti in 0..trl {
                let tp = std::ptr::read_unaligned::<*mut c_void>(trb.add(IL2CPP_LIST_ITEMS_OFF + ti * IL2CPP_LIST_ITEM_SIZE) as *const *mut c_void);
                if tp.is_null() { continue; }
                // TargetRace: targetId at offset 0x10, evaluation at 0x14
                let bytes = tp as *const u8;
                let tid = std::ptr::read_unaligned::<i32>(bytes.add(IL2CPP_TARGET_RACE_ID_OFF) as *const i32);
                let teval = std::ptr::read_unaligned::<i32>(bytes.add(IL2CPP_TARGET_RACE_EVAL_OFF) as *const i32);
                target_races_json.push(format!(r#"{{"target_id":{},"evaluation":{}}}"#, tid, teval));
            }
        }
    }

    // 5. Read route_race from mdb for race name resolution
    let mut race_names_json: Vec<String> = Vec::new();
    if let Some(mdb_path) = find_mdb_path() {
        if let Ok(conn) = Connection::open_with_flags(&mdb_path, OpenFlags::SQLITE_OPEN_READ_ONLY) {
            if let Ok(mut stmt) = conn.prepare("SELECT id, race_id, race_grade FROM single_mode_route_race ORDER BY id LIMIT 200") {
                let rows: Vec<String> = stmt.query_map([], |row| {
                    Ok(format!(r#"{{"id":{},"race_id":{},"grade":{}}}"#,
                        row.get::<_, i32>(0).unwrap_or(0),
                        row.get::<_, i32>(1).unwrap_or(0),
                        row.get::<_, i32>(2).unwrap_or(0)))
                }).unwrap().filter_map(|r| r.ok()).collect();
                race_names_json = rows;
            }
            drop(conn);
        }
    }

    format!(
        r#"{{"version":"3.22.21","parents":{{"first_chara_id":{},"second_chara_id":{}}},"factor_count":{},"relations":[{}],"relation_members":[{}],"relation_ranks":[{}],"target_races":[{}],"route_races":[{}]}}"#,
        first_chara_id, second_chara_id, factor_count,
        relations_json.join(","), relation_members_json.join(","),
        relation_ranks_json.join(","), target_races_json.join(","),
        race_names_json.join(",")
    )
}

/// /log/turn — Turn-by-turn game state log
/// Returns current turn info + history from training log
/// Data sources:
///   - WorkSingleModeData: Month, Half, Turn
///   - WorkSingleModeCharaData: all stats, motivation
///   - SingleModeTurn (mdb): turn config (year, period, training set)
///   - Training log snapshots
unsafe fn read_turn_log() -> String {
    if API.is_null() { return r#"{"error":"api_null"}"#.to_string(); }
    let image = match get_image() {
        img if !img.is_null() => img,
        _ => return r#"{"error":"image_null"}"#.to_string(),
    };

    let wdm_class = find_class(image, to_cstr("Gallop").as_ptr(), to_cstr("WorkDataManager").as_ptr());
    if wdm_class.is_null() { return r#"{"error":"no_wdm"}"#.to_string(); }
    let wdm_inst = get_singleton(wdm_class);
    if wdm_inst.is_null() { return r#"{"error":"no_wdm_inst"}"#.to_string(); }
    log_predict_step("P:wdm");

    let sm_class = find_class(image, to_cstr("Gallop").as_ptr(), to_cstr("WorkSingleModeData").as_ptr());
    let sm_obj = call_getter_ref(wdm_class, wdm_inst, "get_SingleMode");
    if sm_obj.is_null() { return r#"{"error":"no_sm"}"#.to_string(); }

    let chara_class = find_class(image, to_cstr("Gallop").as_ptr(), to_cstr("WorkSingleModeCharaData").as_ptr());
    let chara_obj = call_getter_ref(sm_class, sm_obj, "get_Character");
    if chara_obj.is_null() { return r#"{"error":"no_chara"}"#.to_string(); }

    // Current state
    let mon = call_getter_int(sm_class, sm_obj, "get_Month");
    let half = call_getter_int(sm_class, sm_obj, "get_Half");
    let sid = call_getter_int(chara_class, chara_obj, "get_ScenarioId");
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

    // Turn config from mdb
    let mut turn_config_json = String::new();
    if let Some(mdb_path) = find_mdb_path() {
        if let Ok(conn) = Connection::open_with_flags(&mdb_path, OpenFlags::SQLITE_OPEN_READ_ONLY) {
            if let Ok(mut stmt) = conn.prepare("SELECT id, turn, year, month, half, period, unique_command, training_set_id, race_entry_type FROM single_mode_turn ORDER BY id") {
                let rows: Vec<String> = stmt.query_map([], |row| {
                    Ok(format!(
                        r#"{{"id":{},"turn":{},"year":{},"month":{},"half":{},"period":{},"unique_cmd":{},"training_set":{},"race_entry":{}}}"#,
                        row.get::<_, i32>(0).unwrap_or(0),
                        row.get::<_, i32>(1).unwrap_or(0),
                        row.get::<_, i32>(2).unwrap_or(0),
                        row.get::<_, i32>(3).unwrap_or(0),
                        row.get::<_, i32>(4).unwrap_or(0),
                        row.get::<_, i32>(5).unwrap_or(0),
                        row.get::<_, i32>(6).unwrap_or(0),
                        row.get::<_, i32>(7).unwrap_or(0),
                        row.get::<_, i32>(8).unwrap_or(0),
                    ))
                }).unwrap().filter_map(|r| r.ok()).collect();
                turn_config_json = rows.join(",");
            }
            drop(conn);
        }
    }

    // Training log history
    let log_json = get_training_log();

    // Training levels
    let mut tl_json = "[]".to_string();
    let tl_arr = call_getter_on_instance(chara_class, chara_obj, "get_TrainingLevelInfoArray");
    if !tl_arr.is_null() {
        let tb = tl_arr as *const u8;
        let tl = std::ptr::read_unaligned::<usize>(tb.add(IL2CPP_LIST_COUNT_OFF) as *const usize);
        if tl > 0 && tl < 50 {
            let mut tls = Vec::new();
            for i in 0..tl {
                let tp = std::ptr::read_unaligned::<*mut c_void>(tb.add(IL2CPP_LIST_ITEMS_OFF + i * IL2CPP_LIST_ITEM_SIZE) as *const *mut c_void);
                if tp.is_null() { continue; }
                let bytes = tp as *const u8;
                // TrainingLevelInfo: commandId at 0x10, level at 0x14 (IL2CPP_COMMAND_ID_OFF/IL2CPP_COMMAND_LEVEL_OFF)
                let cmd_id = std::ptr::read_unaligned::<i32>(bytes.add(IL2CPP_COMMAND_ID_OFF) as *const i32);
                let level = std::ptr::read_unaligned::<i32>(bytes.add(IL2CPP_COMMAND_LEVEL_OFF) as *const i32);
                tls.push(format!(r#"{{"command_id":{},"level":{}}}"#, cmd_id, level));
            }
            tl_json = format!("[{}]", tls.join(","));
        }
    }

    format!(
        r#"{{"version":"3.22.21","current":{{"month":{},"half":{},"scenario_id":{},"stats":{{"speed":{},"stamina":{},"power":{},"guts":{},"wiz":{}}},"vital":{},"max_vital":{},"motivation":{},"skill_point":{},"fan":{}}},"training_levels":{},"turn_config":[{}],"history":{}}}"#,
        mon, half, sid, spd, sta, pow_, gut, wiz, vit, mvit, mot, spt, fan,
        tl_json, turn_config_json, log_json
    )
}

/// /event/recommend — Event recommendation based on current game state + event data
/// Reads mdb event data and matches against current support cards + chara
/// Returns: matching events with choice evaluations
unsafe fn read_event_recommend() -> String {
    // Event data is all from mdb (like /events), plus current game state
    if API.is_null() { return r#"{"error":"api_null"}"#.to_string(); }
    let image = match get_image() {
        img if !img.is_null() => img,
        _ => return r#"{"error":"image_null"}"#.to_string(),
    };

    let wdm_class = find_class(image, to_cstr("Gallop").as_ptr(), to_cstr("WorkDataManager").as_ptr());
    if wdm_class.is_null() { return r#"{"error":"no_wdm"}"#.to_string(); }
    let wdm_inst = get_singleton(wdm_class);
    if wdm_inst.is_null() { return r#"{"error":"no_wdm_inst"}"#.to_string(); }
    log_predict_step("P:wdm");

    let sm_class = find_class(image, to_cstr("Gallop").as_ptr(), to_cstr("WorkSingleModeData").as_ptr());
    let sm_obj = call_getter_ref(wdm_class, wdm_inst, "get_SingleMode");
    if sm_obj.is_null() { return r#"{"error":"no_sm"}"#.to_string(); }

    let chara_class = find_class(image, to_cstr("Gallop").as_ptr(), to_cstr("WorkSingleModeCharaData").as_ptr());
    let chara_obj = call_getter_ref(sm_class, sm_obj, "get_Character");
    if chara_obj.is_null() { return r#"{"error":"no_chara"}"#.to_string(); }

    // Current game state
    let card_id = call_getter_int(chara_class, chara_obj, "get_CardId");
    let sid = call_getter_int(chara_class, chara_obj, "get_ScenarioId");
    let spd = call_getter_int(chara_class, chara_obj, "get_Speed");
    let sta = call_getter_int(chara_class, chara_obj, "get_Stamina");
    let pow_ = call_getter_int(chara_class, chara_obj, "get_Power");
    let gut = call_getter_int(chara_class, chara_obj, "get_Guts");
    let wiz = call_getter_int(chara_class, chara_obj, "get_Wiz");
    let spt = call_getter_obscured_int(chara_class, chara_obj, "get_SkillPoint");
    let vit = call_getter_int(chara_class, chara_obj, "get_Hp");
    let mvit = call_getter_int(chara_class, chara_obj, "get_MaxHp");
    let mon = call_getter_int(sm_class, sm_obj, "get_Month");
    let half = call_getter_int(sm_class, sm_obj, "get_Half");

    // Read equipped support cards to match events
    let sc_arr = read_field_value(chara_class, chara_obj, "support_card_array");
    let mut support_card_ids: Vec<i32> = Vec::new();
    if sc_arr.is_null() {
        let arr = call_getter_on_instance(chara_class, chara_obj, "get_SupportCardArray");
        if !arr.is_null() {
            let ab = arr as *const u8;
            let al = std::ptr::read_unaligned::<usize>(ab.add(IL2CPP_LIST_COUNT_OFF) as *const usize);
            if al > 0 && al < 100 {
                for i in 0..al {
                    let ep = std::ptr::read_unaligned::<*mut c_void>(ab.add(IL2CPP_LIST_ITEMS_OFF + i * IL2CPP_LIST_ITEM_SIZE) as *const *mut c_void);
                    if ep.is_null() { continue; }
                    let bytes = ep as *const u8;
                    let sc_id = std::ptr::read_unaligned::<i32>(bytes.add(IL2CPP_SUPPORT_CARD_ID_OFF) as *const i32);
                    support_card_ids.push(sc_id);
                }
            }
        }
    } else {
        let ab = sc_arr as *const u8;
        let al = std::ptr::read_unaligned::<usize>(ab.add(IL2CPP_LIST_COUNT_OFF) as *const usize);
        if al > 0 && al < 100 {
            for i in 0..al {
                let ep = std::ptr::read_unaligned::<*mut c_void>(ab.add(IL2CPP_LIST_ITEMS_OFF + i * IL2CPP_LIST_ITEM_SIZE) as *const *mut c_void);
                if ep.is_null() { continue; }
                let bytes = ep as *const u8;
                let sc_id = std::ptr::read_unaligned::<i32>(bytes.add(IL2CPP_SUPPORT_CARD_ID_OFF) as *const i32);
                support_card_ids.push(sc_id);
            }
        }
    }

    // Read evaluation list for partner chara IDs (needed for NPC event matching)
    let mut eval_chara_ids: Vec<i32> = Vec::new();
    let eval_arr = call_getter_on_instance(chara_class, chara_obj, "get_EvaluationList");
    if !eval_arr.is_null() {
        let eb = eval_arr as *const u8;
        let el = std::ptr::read_unaligned::<usize>(eb.add(IL2CPP_LIST_COUNT_OFF) as *const usize);
        if el > 0 && el < 200 {
            let eval_class = find_class_by_short_name(image, "Evaluation");
            if !eval_class.is_null() {
                for ei in 0..el {
                    let ep = std::ptr::read_unaligned::<*mut c_void>(eb.add(IL2CPP_LIST_ITEMS_OFF + ei * IL2CPP_LIST_ITEM_SIZE) as *const *mut c_void);
                    if ep.is_null() { continue; }
                    let tid = call_getter_int(eval_class, ep, "get_TargetId");
                    eval_chara_ids.push(tid);
                }
            }
        }
    }

    // Read events from mdb with matching support cards
    let mut matching_events: Vec<String> = Vec::new();
    let mut all_events_count: i32 = 0;
    let mut matching_events_count: i32 = 0;

    if let Some(mdb_path) = find_mdb_path() {
        if let Ok(conn) = Connection::open_with_flags(&mdb_path, OpenFlags::SQLITE_OPEN_READ_ONLY) {
            // Read all story data, filter by current chara + support cards
            let sc_ids_str: Vec<String> = support_card_ids.iter().map(|id| id.to_string()).collect();
            let sc_in_clause = sc_ids_str.join(",");

            // Chara events (card_id matches) + Support card events (support_card_id matches)
            let query = if !sc_in_clause.is_empty() {
                format!(
                    "SELECT id, story_id, card_id, support_card_id, event_category FROM single_mode_story_data WHERE card_id={} OR support_card_id IN ({}) ORDER BY id",
                    card_id, sc_in_clause
                )
            } else {
                format!(
                    "SELECT id, story_id, card_id, support_card_id, event_category FROM single_mode_story_data WHERE card_id={} ORDER BY id",
                    card_id
                )
            };

            if let Ok(mut stmt) = conn.prepare(&query) {
                let rows: Vec<String> = stmt.query_map([], |row| {
                    Ok(format!(
                        r#"{{"id":{},"story_id":{},"card_id":{},"support_card_id":{},"event_category":{}}}"#,
                        row.get::<_, i32>(0).unwrap_or(0),
                        row.get::<_, i32>(1).unwrap_or(0),
                        row.get::<_, i32>(2).unwrap_or(0),
                        row.get::<_, i32>(3).unwrap_or(0),
                        row.get::<_, i32>(4).unwrap_or(0),
                    ))
                }).unwrap().filter_map(|r| r.ok()).collect();
                matching_events = rows;
                matching_events_count = matching_events.len() as i32;
            }

            // Total events count
            all_events_count = conn.query_row(
                "SELECT COUNT(*) FROM single_mode_story_data", [], |r| r.get(0)
            ).unwrap_or(0);

            // Read choice rewards for matching stories
            let mut choice_rewards: Vec<String> = Vec::new();
            if let Ok(mut stmt) = conn.prepare(
                "SELECT id, disp_type, effect_value_type0, effect_value_type1, effect_value_type2 FROM single_mode_event_choice_reward ORDER BY id"
            ) {
                let rows: Vec<String> = stmt.query_map([], |row| {
                    Ok(format!(
                        r#"{{"id":{},"disp_type":{},"evt0":{},"evt1":{},"evt2":{}}}"#,
                        row.get::<_, i32>(0).unwrap_or(0),
                        row.get::<_, i32>(1).unwrap_or(0),
                        row.get::<_, i32>(2).unwrap_or(0),
                        row.get::<_, i32>(3).unwrap_or(0),
                        row.get::<_, i32>(4).unwrap_or(0),
                    ))
                }).unwrap().filter_map(|r| r.ok()).collect();
                choice_rewards = rows;
            }

            drop(conn);

            format!(
                r#"{{"version":"3.22.21","current_state":{{"card_id":{},"scenario_id":{},"month":{},"half":{},"stats":{{"speed":{},"stamina":{},"power":{},"guts":{},"wiz":{}}},"vital":{},"max_vital":{},"skill_point":{}}},"support_card_ids":[{}],"eval_chara_ids":[{}],"total_events":{},"matching_events":{},"events":[{}],"choice_rewards":[{}]}}"#,
                card_id, sid, mon, half, spd, sta, pow_, gut, wiz, vit, mvit, spt,
                support_card_ids.iter().map(|id| id.to_string()).collect::<Vec<_>>().join(","),
                eval_chara_ids.iter().map(|id| id.to_string()).collect::<Vec<_>>().join(","),
                all_events_count, matching_events_count,
                matching_events.join(","),
                choice_rewards.join(",")
            )
        } else {
            format!(
                r#"{{"version":"3.22.21","error":"mdb_open_failed","current_state":{{"card_id":{},"scenario_id":{}}}}}"#,
                card_id, sid
            )
        }
    } else {
        format!(
            r#"{{"version":"3.22.21","error":"mdb_not_found","current_state":{{"card_id":{},"scenario_id":{}}}}}"#,
            card_id, sid
        )
    }
}
