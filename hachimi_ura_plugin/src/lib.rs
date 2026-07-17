//! URA Plugin v3.24.15
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
const PLUGIN_VERSION: &str = env!("CARGO_PKG_VERSION");

use rusqlite::{Connection, OpenFlags};
use std::ffi::{c_char, c_void, CStr, CString};
use std::io::{Read, Write};
use std::ptr;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Mutex;

#[repr(i32)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum InitResult {
    Error = 0,
    Ok = 1,
}

struct Api {
    log_fn: Option<unsafe extern "C" fn(i32, *const c_char, *const c_char)>,
    gui_show_notification_fn: Option<unsafe extern "C" fn(*const c_char) -> bool>,
    gui_register_menu_item_fn: Option<
        unsafe extern "C" fn(
            *const c_char,
            Option<extern "C" fn(*mut c_void)>,
            *mut c_void,
        ) -> bool,
    >,
    gui_register_menu_section_fn: Option<
        unsafe extern "C" fn(Option<extern "C" fn(*mut c_void, *mut c_void)>, *mut c_void) -> bool,
    >,
    hachimi_register_on_game_initialized_fn:
        Option<unsafe extern "C" fn(Option<extern "C" fn(*mut c_void)>, *mut c_void) -> bool>,
    gui_ui_heading_fn: Option<unsafe extern "C" fn(*mut c_void, *const c_char) -> bool>,
    gui_ui_label_fn: Option<unsafe extern "C" fn(*mut c_void, *const c_char) -> bool>,
    gui_ui_colored_label_fn:
        Option<unsafe extern "C" fn(*mut c_void, u8, u8, u8, u8, *const c_char) -> bool>,
    gui_ui_separator_fn: Option<unsafe extern "C" fn(*mut c_void) -> bool>,
    gui_ui_text_edit_singleline_fn:
        Option<unsafe extern "C" fn(*mut c_void, *mut c_char, i32) -> bool>,
    il2cpp_get_assembly_image_fn: Option<unsafe extern "C" fn(*const c_char) -> *const c_void>,
    il2cpp_get_class_fn:
        Option<unsafe extern "C" fn(*const c_void, *const c_char, *const c_char) -> *mut c_void>,
    il2cpp_get_field_from_name_fn:
        Option<unsafe extern "C" fn(*mut c_void, *const c_char) -> *mut c_void>,
    il2cpp_get_field_value_fn:
        Option<unsafe extern "C" fn(*const c_void, *const c_void, *mut c_void)>,
    il2cpp_get_static_field_value_fn: Option<unsafe extern "C" fn(*const c_void, *mut c_void)>,
    il2cpp_resolve_symbol_fn: Option<unsafe extern "C" fn(*const c_char) -> *mut c_void>,
    il2cpp_get_singleton_like_instance_fn:
        Option<unsafe extern "C" fn(*mut c_void) -> *const c_void>,
    il2cpp_string_chars_fn: Option<unsafe extern "C" fn(*const c_void) -> *mut u16>,
    il2cpp_string_length_fn: Option<unsafe extern "C" fn(*const c_void) -> i32>,
    // ★ v3.23.3: Hachimi-Edge V3 Interceptor API
    hachimi_instance_fn: Option<unsafe extern "C" fn() -> usize>,
    hachimi_get_interceptor_fn: Option<unsafe extern "C" fn(usize) -> usize>,
    interceptor: usize,
    interceptor_hook_fn:
        Option<unsafe extern "C" fn(usize, *mut c_void, *mut c_void) -> *mut c_void>,
    interceptor_get_trampoline_addr_fn:
        Option<unsafe extern "C" fn(usize, *mut c_void) -> *mut c_void>,
    il2cpp_get_method_addr_fn: Option<unsafe extern "C" fn(usize, *const c_char, i32) -> usize>,
}

static mut API: *mut Api = ptr::null_mut();
static GAME_INITIALIZED: AtomicBool = AtomicBool::new(false);
static HTTP_RUNNING: AtomicBool = AtomicBool::new(false);
static PREDICT_STEP: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
static CRASH_SIG: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(0);
static CRASH_STEP: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
static mut LAST_STEP_BUF: [u8; 128] = [0; 128];
static LAST_STEP_LEN: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
static AUTO_UPDATE_STATUS: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);
// ★ v3.22.94: Training result hook — intercept OnSuccessSendCommand to read resultType
static mut LAST_TRAINING_RESULT: i32 = -1;
static mut LAST_TRAINING_SUB_ID: i32 = -1;
// ★ v2.2: 真实 CommandId 捕获 — 从 training_hook_handler 记录
static mut LAST_ACTION_COMMAND_ID: i32 = -1;
static mut LAST_ACTION_SEQUENCE: u64 = 0;
static LAST_ACTION_MUTEX: Mutex<()> = Mutex::new(());
static mut TRAINING_HOOK_INSTALLED: bool = false;
static mut ORIG_ON_SUCCESS_PROLOGUE: [u8; 16] = [0; 16];
static mut ON_SUCCESS_ADDR: usize = 0;
// ★ v3.23.3: API sniffing — use Hachimi Interceptor API (hook+trampoline) + WWWRequest.Post for URL (replaces _Send+SetHeader)
static SNIFF_ENABLED: AtomicBool = AtomicBool::new(false);
static SNIFF_MUTEX: Mutex<()> = Mutex::new(());
// SniffEntry: (id, url, headers_json, body)
static mut SNIFF_REQUESTS: Vec<(u64, String, String, Vec<u8>)> = Vec::new();
static mut SNIFF_RESPONSES: Vec<(u64, Vec<u8>)> = Vec::new();
static SNIFF_MAX: usize = 20;
static SNIFF_REQ_ID: AtomicU64 = AtomicU64::new(0);
static mut PENDING_URL: String = String::new();
static mut PENDING_HEADERS: Vec<(String, String)> = Vec::new();
static mut PENDING_REQ_ID: u64 = 0;
// CompressRequest/DecompressResponse/Post hook addresses (via Interceptor API)
static mut COMPRESS_REQUEST_ADDR: usize = 0;
static mut DECOMPRESS_RESPONSE_ADDR: usize = 0;
static mut POST_ADDR: usize = 0;
// Pending request body parking (CompressRequest → Post matching)
static mut PENDING_REQ_BODY: Option<Vec<u8>> = None;
static mut PENDING_COMPRESSED: usize = 0;
// ★ Mutex to prevent concurrent read_summary_inner calls from HTTP + push threads
static READ_MUTEX: Mutex<()> = Mutex::new(());

// ★ v3.24.2: Story event choice hook — capture career event choices (options, effects, branches)
static mut EVENT_CHOICE_HOOK_INSTALLED: bool = false;
static mut EVENT_CHOICE_ADDR: usize = 0; // StoryChoiceController.Choice
static mut EVENT_ADD_BTN_ADDR: usize = 0; // StoryChoiceController.AddChoiceButton
static mut ORIG_EVENT_CHOICE_PROLOGUE: [u8; 16] = [0; 16];
static mut ORIG_EVENT_ADD_BTN_PROLOGUE: [u8; 16] = [0; 16];
// ★ v3.24.2: StoryManager.SetStory hook — capture story_id and chara_id for event type identification
static mut STORY_SET_HOOK_INSTALLED: bool = false;
static mut STORY_SET_ADDR: usize = 0;
static mut ORIG_STORY_SET_PROLOGUE: [u8; 16] = [0; 16];
// Event state: accumulated choices for current event
static EVENT_STATE_MUTEX: Mutex<()> = Mutex::new(());
static mut EVENT_CHOICES: Vec<EventChoice> = Vec::new();
static mut EVENT_SELECTED_IDX: i32 = -1;
static mut EVENT_STORY_ID: i32 = 0;
static mut EVENT_CHARA_ID: i32 = 0;

#[derive(Clone)]
struct EventChoice {
    label: String,
    gain_id: i32,
    next_block_idx: i32,
    loop_exit_gain_id: i32,
}

// ★ v3.24.2: Read C# string from IL2CPP String object
unsafe fn read_il2cpp_string(s: *const c_void) -> String {
    if s.is_null() {
        return String::new();
    }
    let len = std::ptr::read::<i32>((s as *const u8).offset(16) as *const i32);
    if len <= 0 || len > 4096 {
        return String::new();
    }
    let chars_ptr = (s as *const u8).offset(20);
    let chars_slice = std::slice::from_raw_parts(chars_ptr as *const u16, len as usize);
    String::from_utf16_lossy(chars_slice)
}

// ★ Push-to-app state (v3.10.0): auto-push /summary to uma-juece when data changes
static mut LAST_PUSH_HASH: u64 = 0;
static PUSH_INTERVAL_SECS: u64 = 1;

// ★ Config (v3.11.0): runtime config updated via POST /config from App
// No file editing needed — App settings page sends config to plugin HTTP endpoint
#[derive(Clone)]
struct PluginConfig {
    push_host: String,       // default: "127.0.0.1"
    push_port: u16,          // default: 18766
    http_port: u16,          // default: 18765
    push_interval_secs: u64, // default: 1
    push_enabled: bool,      // default: true
    http_enabled: bool,      // default: true
}

impl PluginConfig {
    fn defaults() -> Self {
        Self {
            push_host: "127.0.0.1".to_string(),
            push_port: 18766,
            http_port: 18765,
            push_interval_secs: 5,
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
            if l.is_empty() || l == "{" || l == "}" {
                continue;
            }
            if let Some((k, v)) = l.split_once(':') {
                let k = k.trim().trim_matches('"');
                let v = v.trim().trim_matches('"');
                match k {
                    "push_host" => {
                        cfg.push_host = v.to_string();
                        changed = true;
                    }
                    "push_port" => {
                        if let Ok(n) = v.parse::<u16>() {
                            cfg.push_port = n;
                            changed = true;
                        }
                    }
                    "http_port" => {
                        if let Ok(n) = v.parse::<u16>() {
                            cfg.http_port = n;
                            changed = true;
                        }
                    }
                    "push_interval_secs" => {
                        if let Ok(n) = v.parse::<u64>() {
                            cfg.push_interval_secs = n.max(1);
                            changed = true;
                        }
                    }
                    "push_enabled" => {
                        cfg.push_enabled = v == "true";
                        changed = true;
                    }
                    "http_enabled" => {
                        cfg.http_enabled = v == "true";
                        changed = true;
                    }
                    _ => {}
                }
            }
        }
        if changed {
            Some(cfg)
        } else {
            None
        }
    }

    fn to_json(&self) -> String {
        format!(
            r#"{{"push_host":"{}","push_port":{},"http_port":{},"push_interval_secs":{},"push_enabled":{},"http_enabled":{}}}"#,
            self.push_host,
            self.push_port,
            self.http_port,
            self.push_interval_secs,
            self.push_enabled,
            self.http_enabled
        )
    }
}

static mut PLUGIN_CONFIG: Option<PluginConfig> = None;

// ★ Text edit buffers for GUI config (v3.12.0): persist across frames for egui immediate mode
static mut GUI_HOST_BUF: [u8; 64] = [0u8; 64]; // push_host input buffer
static mut GUI_HOST_BUF_LEN: i32 = 0;
static mut GUI_PORT_BUF: [u8; 8] = [0u8; 8]; // push_port input buffer
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
    format!(
        r#"{{"entries":{},"log":[{}]}}"#,
        TRAINING_LOG.len(),
        TRAINING_LOG.join(",")
    )
}

#[derive(Copy, Clone)]
struct CharaCache {
    speed: i32,
    stamina: i32,
    power: i32,
    guts: i32,
    wiz: i32,
    vital: i32,
    max_vital: i32,
    motivation: i32,
    turn: i32,
    skill_point: i32,
    scenario_id: i32,
    fan_count: i32,
    month: i32,
    half: i32,
    playing_state: i32,
    is_playing: bool,
    valid: bool,
}

static mut CHARA: CharaCache = CharaCache {
    speed: 0,
    stamina: 0,
    power: 0,
    wiz: 0,
    guts: 0,
    vital: 0,
    max_vital: 0,
    motivation: 0,
    turn: 0,
    skill_point: 0,
    scenario_id: 0,
    fan_count: 0,
    month: 0,
    half: 0,
    playing_state: 0,
    is_playing: false,
    valid: false,
};

fn to_cstr(s: &str) -> CString {
    CString::new(s).unwrap_or_else(|_| CString::new("<err>").unwrap())
}

unsafe fn ura_log(level: i32, msg: &str) {
    if API.is_null() {
        return;
    }
    if let Some(log_fn) = (*API).log_fn {
        let tag = to_cstr("URA");
        let text = to_cstr(msg);
        log_fn(level, tag.as_ptr(), text.as_ptr());
    }
}

unsafe fn ura_notify(msg: &str) {
    if API.is_null() {
        return;
    }
    if let Some(notify_fn) = (*API).gui_show_notification_fn {
        let text = to_cstr(msg);
        notify_fn(text.as_ptr());
    }
}

// ===== Crash logging =====
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
    #[link_name = "sigsetjmp"]
    fn sys_sigsetjmp(env: *mut u8, savemask: i32) -> i32;
    #[link_name = "siglongjmp"]
    fn sys_siglongjmp(env: *const u8, val: i32) -> !;
}

const CRASH_LOG_PATH: &str = "/data/data/jp.pokemon.pokeuma/files/uma_predict.log";

// ★ v3.22.35: SIGSEGV recovery for push thread
// sigsetjmp buffer: 200 bytes is enough for jmp_buf on aarch64 (typically 24 x 8 = 192 bytes)
static mut SIGSEGV_JMP_BUF: [u8; 200] = [0u8; 200];
static SIGSEGV_RECOVERY: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
// Cooldown: after SIGSEGV recovery, skip reads for N seconds
static SIGSEGV_COOLDOWN_UNTIL: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

extern "C" fn crash_signal_handler(sig: i32) {
    CRASH_SIG.store(sig, std::sync::atomic::Ordering::Relaxed);
    CRASH_STEP.store(
        PREDICT_STEP.load(std::sync::atomic::Ordering::Relaxed),
        std::sync::atomic::Ordering::Relaxed,
    );
    // Log the crash
    let step = PREDICT_STEP.load(std::sync::atomic::Ordering::Relaxed);
    let mut msg = [0u8; 64];
    let p = b"CRASH at step ";
    msg[..p.len()].copy_from_slice(p);
    let mut len = p.len();
    let mut n = step;
    if n == 0 {
        msg[len] = b'0';
        len += 1;
    } else {
        let mut digits = [0u8; 10];
        let mut dlen = 0;
        while n > 0 {
            digits[dlen] = b'0' + (n % 10) as u8;
            n /= 10;
            dlen += 1;
        }
        for i in (0..dlen).rev() {
            msg[len] = digits[i];
            len += 1;
        }
    }
    let s = b" sig=";
    msg[len..len + s.len()].copy_from_slice(s);
    len += s.len();
    let mut n2 = sig as u32;
    if n2 == 0 {
        msg[len] = b'0';
        len += 1;
    } else {
        let mut digits = [0u8; 10];
        let mut dlen = 0;
        while n2 > 0 {
            digits[dlen] = b'0' + (n2 % 10) as u8;
            n2 /= 10;
            dlen += 1;
        }
        for i in (0..dlen).rev() {
            msg[len] = digits[i];
            len += 1;
        }
    }
    let r = b" RECOVERED";
    msg[len..len + r.len()].copy_from_slice(r);
    len += r.len();
    msg[len] = b'\n';
    len += 1;
    let path = b"/data/data/jp.pokemon.pokeuma/files/uma_predict.log\0";
    let fd = unsafe { sys_open(path.as_ptr() as *const i8, 1 | 64 | 1024, 0o644) };
    if fd >= 0 {
        unsafe {
            sys_write(fd, msg.as_ptr(), len);
            sys_close(fd);
        }
    }
    // ★ v3.22.35: If sigsetjmp was set (push thread), longjmp back instead of killing process
    if SIGSEGV_RECOVERY.load(std::sync::atomic::Ordering::Relaxed) {
        // Set cooldown: skip reads for 60 seconds
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        SIGSEGV_COOLDOWN_UNTIL.store(now + 60, std::sync::atomic::Ordering::Relaxed);
        SIGSEGV_RECOVERY.store(false, std::sync::atomic::Ordering::Relaxed);
        unsafe {
            sys_siglongjmp(SIGSEGV_JMP_BUF.as_ptr(), 1);
        }
    }
    // Not in recovery context — re-raise signal to kill process (unrecoverable)
    unsafe {
        sys_signal(sig, 0);
        sys_raise(sig);
    }
}

fn init_crash_handler() {
    unsafe {
        let handler = crash_signal_handler as usize;
        sys_signal(11, handler); // SIGSEGV
        sys_signal(6, handler); // SIGABRT
        sys_signal(7, handler); // SIGBUS
        sys_signal(8, handler); // SIGFPE
    }
    std::panic::set_hook(Box::new(|info| {
        let msg = format!("PANIC: {}\n", info);
        let _ = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
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
        if fd >= 0 {
            sys_write(fd, line_bytes.as_ptr(), line_bytes.len());
            sys_close(fd);
        }
        let fd2 = sys_open(path2.as_ptr() as *const i8, 1 | 64 | 1024, 0o644);
        if fd2 >= 0 {
            sys_write(fd2, line_bytes.as_ptr(), line_bytes.len());
            sys_close(fd2);
        }
        // v3.22.51: std::fs fallback
        let _ = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
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
        if fd >= 0 {
            sys_close(fd);
        }
        let fd2 = sys_open(path2.as_ptr() as *const i8, 1 | 64 | 512, 0o644);
        if fd2 >= 0 {
            sys_close(fd2);
        }
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
        },
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
        if ch.len() > 1 {
            r.push(T[((b1 & 0x0f) << 2) | (b2 >> 6)] as char);
        } else {
            r.push('=');
        }
        if ch.len() > 2 {
            r.push(T[b2 & 0x3f] as char);
        } else {
            r.push('=');
        }
    }
    r
}

fn read_github_token() -> String {
    let token_path = "/data/data/jp.pokemon.pokeuma/files/uma_github_token.txt";
    match std::fs::read_to_string(token_path) {
        Ok(t) => t.trim().to_string(),
        Err(_) => String::new(),
    }
}

fn check_and_upload_crash_log() {
    let path = "/data/data/jp.pokemon.pokeuma/files/uma_predict.log";
    if !std::path::Path::new(path).exists() {
        return;
    }
    let content = match std::fs::read(path) {
        Ok(c) => c,
        Err(_) => return,
    };
    if content.is_empty() {
        return;
    }
    if content.ends_with(b"DONE\n") {
        let _ = std::fs::remove_file(path);
        return;
    }
    // Base64 encode and upload to GitHub
    let b64 = base64_encode(&content);
    let json = format!(
        r#"{{"message":"crash log auto-upload","content":"{}"}}"#,
        b64
    );
    let _ = std::fs::write("/data/data/jp.pokemon.pokeuma/files/uma_upload.json", &json);
    let gh_token = read_github_token();
    if gh_token.is_empty() {
        return;
    }
    let cmd = format!("curl -s -X PUT -H 'Authorization: token {}' -H 'Content-Type: application/json' -d @/data/data/jp.pokemon.pokeuma/files/uma_upload.json https://api.github.com/repos/xf8410/hlpatch/contents/crash_log.txt >/dev/null 2>&1", gh_token);
    if let Ok(cmd_c) = std::ffi::CString::new(cmd) {
        unsafe {
            sys_system(cmd_c.as_ptr() as *const i8);
        }
    }
    let _ = std::fs::remove_file("/data/data/jp.pokemon.pokeuma/files/uma_upload.json");
}

fn save_endpoint_log(endpoint: &str, data: &str) {
    let safe_name = endpoint.trim_start_matches('/').replace('/', "_");
    if safe_name.is_empty()
        || safe_name == "health"
        || safe_name == "status"
        || safe_name == "config"
        || safe_name == "config.html"
        || safe_name == "debug_upload"
        || safe_name == "debug_crashlog"
    {
        return;
    }
    let _ = std::fs::create_dir_all("/data/data/jp.pokemon.pokeuma/files/uma_logs");
    let path = format!(
        "/data/data/jp.pokemon.pokeuma/files/uma_logs/{}.json",
        safe_name
    );
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
            "curl -s -X PUT -H 'Authorization: token {}' -H 'Content-Type: application/json' -d @{} 'https://api.github.com/repos/xf8410/hlpatch/contents/{}' > /dev/null 2>&1",
            read_github_token(), tmp_path, github_path
        );
        if let Ok(cmd_c) = std::ffi::CString::new(cmd) {
            unsafe {
                sys_system(cmd_c.as_ptr() as *const i8);
            }
        }

        uploaded += 1;
        file_names.push(name);
    }

    let _ = std::fs::remove_file("/data/data/jp.pokemon.pokeuma/files/uma_upload_tmp.json");

    let files_json = file_names
        .iter()
        .map(|n| format!(r#""{}""#, n))
        .collect::<Vec<_>>()
        .join(",");
    format!(r#"{{"uploaded":{},"files":[{}]}}"#, uploaded, files_json)
}

// ============================================================
// IL2CPP Helpers
// ============================================================

unsafe fn get_image() -> *const c_void {
    if API.is_null() {
        return ptr::null();
    }
    match (*API).il2cpp_get_assembly_image_fn {
        Some(fn_ptr) => {
            let name = to_cstr("umamusume.dll");
            let img = fn_ptr(name.as_ptr());
            if img.is_null() {
                ura_log(1, "get_image: umamusume.dll image = null");
            }
            img
        }
        None => {
            ura_log(1, "get_image: no get_assembly_image_fn");
            ptr::null()
        }
    }
}

unsafe fn find_class(image: *const c_void, ns: *const c_char, name: *const c_char) -> *mut c_void {
    if image.is_null() || API.is_null() {
        return ptr::null_mut();
    }
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
        if !cls.is_null() {
            return cls;
        }
    }
    // Fallback: iterate all classes to find by name (slow but handles any namespace)
    find_class_by_iteration(image, class_name)
}

/// Slow fallback: iterate all classes in the assembly to find one by name
unsafe fn find_class_by_iteration(image: *const c_void, class_name: &str) -> *mut c_void {
    let get_count_fn = resolve_il2cpp_symbol("il2cpp_image_get_class_count");
    let get_class_fn = resolve_il2cpp_symbol("il2cpp_image_get_class");
    if get_count_fn.is_null() || get_class_fn.is_null() {
        return ptr::null_mut();
    }

    let get_count: FnImageGetClassCount = std::mem::transmute(get_count_fn);
    let get_class: FnImageGetClass = std::mem::transmute(get_class_fn);
    let get_name_fn = resolve_il2cpp_symbol("il2cpp_class_get_name");

    let count = get_count(image);
    for i in 0..count {
        let cls = get_class(image, i);
        if cls.is_null() {
            continue;
        }
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
    if klass.is_null() {
        return String::new();
    }
    let get_name_fn = resolve_il2cpp_symbol("il2cpp_class_get_name");
    if get_name_fn.is_null() {
        return String::new();
    }
    let get_name: FnClassGetName = std::mem::transmute(get_name_fn);
    let name_ptr = get_name(klass);
    if name_ptr.is_null() {
        return String::new();
    }
    std::ffi::CStr::from_ptr(name_ptr)
        .to_string_lossy()
        .into_owned()
}

/// ★ Get class name from an object instance by reading its klass pointer from the object header
/// IL2CPP object layout: offset 0 = Il2CppClass* klass (8 bytes on 64-bit)
unsafe fn get_object_class_name(obj: *const c_void) -> String {
    if obj.is_null() {
        return String::new();
    }
    let klass = std::ptr::read_unaligned::<*mut c_void>(obj as *const *mut c_void);
    get_class_name_from_pointer(klass)
}

unsafe fn get_singleton(class: *mut c_void) -> *const c_void {
    if class.is_null() || API.is_null() {
        return ptr::null();
    }
    match (*API).il2cpp_get_singleton_like_instance_fn {
        Some(fn_ptr) => fn_ptr(class),
        None => ptr::null(),
    }
}

unsafe fn read_field_ptr(
    obj: *const c_void,
    class: *mut c_void,
    field_name: &str,
) -> *const c_void {
    if obj.is_null() || class.is_null() || API.is_null() {
        return ptr::null();
    }
    let field = match (*API).il2cpp_get_field_from_name_fn {
        Some(fn_ptr) => {
            let name_c = to_cstr(field_name);
            fn_ptr(class, name_c.as_ptr())
        }
        None => return ptr::null(),
    };
    if field.is_null() {
        return ptr::null();
    }
    let mut value: *const c_void = ptr::null();
    match (*API).il2cpp_get_field_value_fn {
        Some(fn_ptr) => fn_ptr(
            obj as *mut c_void,
            field,
            &mut value as *mut *const c_void as *mut c_void,
        ),
        None => return ptr::null(),
    }
    value
}

// ★ Read a field value from an object by class + field name (returns *mut c_void)
// Used for reading public fields (not getter properties) like CommandInfoArray
unsafe fn read_field_value(
    class: *mut c_void,
    obj: *const c_void,
    field_name: &str,
) -> *mut c_void {
    if class.is_null() || obj.is_null() || API.is_null() {
        return ptr::null_mut();
    }
    let field_info = match (*API).il2cpp_get_field_from_name_fn {
        Some(f) => f(class, to_cstr(field_name).as_ptr()),
        None => return ptr::null_mut(),
    };
    if field_info.is_null() {
        return ptr::null_mut();
    }
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
type FnClassGetMethodFromName =
    unsafe extern "C" fn(*mut c_void, *const c_char, i32) -> *const c_void;
type FnRuntimeInvoke = unsafe extern "C" fn(
    *const c_void,
    *mut c_void,
    *mut *mut c_void,
    *mut *mut c_void,
) -> *mut c_void;
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
    if API.is_null() {
        return ptr::null_mut();
    }
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
        if p.is_null() {
            None
        } else {
            Some(std::mem::transmute::<*mut c_void, FnClassGetMethodFromName>(p))
        }
    };
    let invoke_fn: Option<FnRuntimeInvoke> = {
        let p = resolve_il2cpp_symbol("il2cpp_runtime_invoke");
        if p.is_null() {
            None
        } else {
            Some(std::mem::transmute::<*mut c_void, FnRuntimeInvoke>(p))
        }
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
        ura_log(
            1,
            &format!("call_getter: '{}' threw exception", method_name),
        );
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
unsafe fn call_getter_int(class: *mut c_void, instance: *const c_void, method_name: &str) -> i32 {
    if class.is_null() || instance.is_null() {
        return -1;
    }

    let result = call_getter_on_instance(class, instance, method_name);
    if result.is_null() {
        return -1;
    }

    // Value type (int/enum) is boxed: real value at offset +16
    let val_ptr = result as *const u8;
    let int_val = std::ptr::read_unaligned::<i32>(val_ptr.add(16) as *const i32);
    int_val
}

/// Call getter that returns bool (value type - gets boxed)
unsafe fn call_getter_bool(class: *mut c_void, instance: *const c_void, method_name: &str) -> bool {
    call_getter_int(class, instance, method_name) != 0
}

/// ★ v3.22.39: Call method with 1 int arg that returns int (value type - boxed by il2cpp_runtime_invoke)
/// Used for TrainingFeelingEntity.GetGainCount(int FeelingId)
/// IMPORTANT: il2cpp_runtime_invoke needs properly boxed args.
/// We find Int32 klass, box our arg into it, then invoke.
unsafe fn call_getter_int_with_arg(
    class: *mut c_void,
    instance: *const c_void,
    method_name: &str,
    int_arg: i32,
) -> i32 {
    if class.is_null() || instance.is_null() {
        return -1;
    }

    let get_method_fn: Option<FnClassGetMethodFromName> = {
        let p = resolve_il2cpp_symbol("il2cpp_class_get_method_from_name");
        if p.is_null() {
            None
        } else {
            Some(std::mem::transmute::<*mut c_void, FnClassGetMethodFromName>(p))
        }
    };
    let invoke_fn: Option<FnRuntimeInvoke> = {
        let p = resolve_il2cpp_symbol("il2cpp_runtime_invoke");
        if p.is_null() {
            None
        } else {
            Some(std::mem::transmute::<*mut c_void, FnRuntimeInvoke>(p))
        }
    };
    if get_method_fn.is_none() || invoke_fn.is_none() {
        return -1;
    }

    let method_name_c = to_cstr(method_name);
    let method_info = get_method_fn.unwrap()(class, method_name_c.as_ptr(), 1); // 1 parameter
    if method_info.is_null() {
        ura_log(
            4,
            &format!("call_int_with_arg: '{}' not found", method_name),
        );
        return -1;
    }

    // Find System.Int32 klass to properly box the argument
    let image = match get_image() {
        img if !img.is_null() => img,
        _ => return -1,
    };
    let int32_class = find_class(image, to_cstr("System").as_ptr(), to_cstr("Int32").as_ptr());
    if int32_class.is_null() {
        ura_log(2, "call_int_with_arg: Int32 class not found");
        return -1;
    }

    // Use il2cpp_object_new to allocate a proper boxed Int32
    let object_new_fn: Option<unsafe extern "C" fn(*mut c_void) -> *mut c_void> = {
        let p = resolve_il2cpp_symbol("il2cpp_object_new");
        if p.is_null() {
            None
        } else {
            Some(std::mem::transmute(p))
        }
    };
    if object_new_fn.is_none() {
        ura_log(2, "call_int_with_arg: il2cpp_object_new not found");
        return -1;
    }

    let boxed_arg = object_new_fn.unwrap()(int32_class);
    if boxed_arg.is_null() {
        ura_log(2, "call_int_with_arg: failed to allocate boxed int");
        return -1;
    }
    // Write int value at offset +16 (after Il2CppObject header)
    std::ptr::write_unaligned::<i32>((boxed_arg as *mut u8).add(16) as *mut i32, int_arg);

    let mut args: [*mut c_void; 1] = [boxed_arg];
    let mut exc: *mut c_void = ptr::null_mut();
    let result = invoke_fn.unwrap()(
        method_info,
        instance as *mut c_void,
        args.as_mut_ptr(),
        &mut exc,
    );
    if !exc.is_null() {
        ura_log(
            2,
            &format!("call_int_with_arg: '{}' threw exception", method_name),
        );
        return -1;
    }
    if result.is_null() {
        return -1;
    }
    // Result is boxed int, value at +16
    std::ptr::read_unaligned::<i32>((result as *const u8).add(16) as *const i32)
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
    if class.is_null() || instance.is_null() {
        return -1;
    }

    let result = call_getter_on_instance(class, instance, method_name);
    if result.is_null() {
        return -1;
    }

    // Boxed ObscuredInt struct layout (from dump.cs Anti-Cheat Toolkit):
    // offset 0x10: currentCryptoKey (Int32) — the decryption key
    // offset 0x14: hiddenValue (Int32) — the encrypted value
    // offset 0x18: inited (Boolean)
    // offset 0x1C: fakeValue (Int32)
    // offset 0x20: fakeValueActive (Boolean)
    let base = result as *const u8;

    let current_crypto_key =
        std::ptr::read_unaligned::<i32>(base.add(IL2CPP_OBSCURED_INT_KEY_OFF) as *const i32);
    let hidden_value =
        std::ptr::read_unaligned::<i32>(base.add(IL2CPP_OBSCURED_INT_HIDDEN_OFF) as *const i32);

    // Decrypt: hiddenValue ^ currentCryptoKey
    let decrypted = hidden_value ^ current_crypto_key;

    ura_log(
        4,
        &format!(
            "ObscuredInt {}: hidden={} key={} decrypted={}",
            method_name, hidden_value, current_crypto_key, decrypted
        ),
    );

    decrypted
}

// ============================================================
// ★ v3.22.51: Direct memory read helpers — zero il2cpp calls
// ============================================================

unsafe fn read_obscured_int_at(obj: *const c_void, field_offset: i32) -> i32 {
    if obj.is_null() || field_offset < 0 {
        return -1;
    }
    let base = obj as *const u8;
    let off = field_offset as usize;
    let key = std::ptr::read_unaligned::<i32>(base.add(off) as *const i32);
    let hidden = std::ptr::read_unaligned::<i32>(base.add(off + 4) as *const i32);
    hidden ^ key
}

unsafe fn read_ptr_at(obj: *const c_void, field_offset: i32) -> *mut c_void {
    if obj.is_null() || field_offset < 0 {
        return ptr::null_mut();
    }
    std::ptr::read_unaligned::<*mut c_void>(
        (obj as *const u8).add(field_offset as usize) as *const *mut c_void
    )
}

// ★ v3.22.51: Direct int read — zero il2cpp_runtime_invoke
unsafe fn read_int_at(obj: *const c_void, field_offset: i32) -> i32 {
    if obj.is_null() || field_offset < 0 {
        return -1;
    }
    let base = obj as *const u8;
    std::ptr::read_unaligned::<i32>(base.add(field_offset as usize) as *const i32)
}

/// ★ v3.24.9: Read IL2CPP List<int> via pointer — zero runtime_invoke
unsafe fn read_il2cpp_int_list(list_ptr: *const c_void) -> Vec<i32> {
    if list_ptr.is_null() {
        return Vec::new();
    }
    let base = list_ptr as *const u8;
    let count = std::ptr::read_unaligned::<usize>(base.add(IL2CPP_LIST_COUNT_OFF) as *const usize);
    if count > 10000 {
        return Vec::new();
    }
    let mut result = Vec::with_capacity(count);
    for i in 0..count {
        let elem = read_obscured_int_at(
            std::ptr::read_unaligned::<*mut c_void>(
                base.add(IL2CPP_LIST_ITEMS_OFF + i * IL2CPP_LIST_ITEM_SIZE) as *const *mut c_void,
            ) as *const c_void,
            IL2CPP_OBSCURED_INT_KEY_OFF as i32,
        );
        result.push(elem);
    }
    result
}

/// v3.22.51: Read Il2CppClass* from object header (offset 0 on 64-bit)
/// This gives us the EXACT class of any object instance at runtime
unsafe fn get_class_from_object(obj: *const c_void) -> *mut c_void {
    if obj.is_null() {
        return ptr::null_mut();
    }
    std::ptr::read_unaligned::<*mut c_void>(obj as *const *mut c_void)
}

/// v3.22.51: Read ObscuredInt field from object using its own class (from header)
/// No need to find class by name — reads it directly from the object
unsafe fn read_obscured_int_from_obj(obj: *const c_void, field_name: &str) -> i32 {
    if obj.is_null() {
        return -1;
    }
    let class = get_class_from_object(obj);
    if class.is_null() {
        return -1;
    }
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
    if class.is_null() || instance.is_null() {
        return result;
    }

    let arr_obj = call_getter_on_instance(class, instance, method_name);
    if arr_obj.is_null() {
        return result;
    }

    // IL2CPP array layout (64-bit):
    // +0x00: Il2CppObject header (16 bytes)
    // +0x10: bounds ptr (8 bytes, null for 1D)
    // +0x18: max_length (8 bytes on 64-bit)
    // +0x20: data start
    let base = arr_obj as *const u8;
    let length = std::ptr::read_unaligned::<usize>(base.add(IL2CPP_LIST_COUNT_OFF) as *const usize);
    if length == 0 || length > 200 {
        return result;
    } // ★ v3.22.51: guard rail — lower limit

    // ObscuredInt struct (unboxed) layout:
    // offset 0x00: currentCryptoKey (Int32)
    // offset 0x04: hiddenValue (Int32)
    // offset 0x08: inited (Boolean, padded to 4)
    // offset 0x0C: fakeValue (Int32)
    // offset 0x10: fakeValueActive (Boolean, padded to 4)
    // struct size = 0x14 (20 bytes), aligned to 4
    let struct_size: usize = 0x14; // ObscuredInt unboxed: 5 fields × 4 bytes = 20 bytes (key+hidden+inited+fake+fakeActive) — fixed: was IL2CPP_OBSCURED_INT_HIDDEN_OFF + IL2CPP_LIST_ITEM_SIZE = 0x1C (wrong, mixed boxed offset with unboxed)
    let data_start = base.add(IL2CPP_LIST_ITEMS_OFF);

    for i in 0..length {
        let elem_base = data_start.add(i * struct_size);
        let crypto_key = std::ptr::read_unaligned::<i32>(elem_base as *const i32);
        let hidden_val = std::ptr::read_unaligned::<i32>(elem_base.add(4) as *const i32);
        let decrypted = hidden_val ^ crypto_key;
        result.push(decrypted);
    }

    ura_log(
        4,
        &format!("{}: read {} elements", method_name, result.len()),
    );
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
    if array_obj.is_null() || element_class.is_null() {
        return results;
    }

    let base = array_obj as *const u8;
    let length = std::ptr::read_unaligned::<usize>(base.add(IL2CPP_LIST_COUNT_OFF) as *const usize);
    if length == 0 || length > 100 {
        return results;
    }

    for i in 0..length {
        let elem_ptr = std::ptr::read_unaligned::<*mut c_void>(
            base.add(IL2CPP_LIST_ITEMS_OFF + i * IL2CPP_LIST_ITEM_SIZE) as *const *mut c_void,
        );
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

    ura_log(
        4,
        &format!(
            "read_array_elements: {} elements, {} getters",
            length,
            obscured_getters.len() + plain_getters.len()
        ),
    );
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
    if chara_class.is_null() || chara_obj.is_null() {
        return ptr::null_mut();
    }

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
        13 => &[
            "get_ScenarioBreeders",
            "get_WorkScenarioBreeders",
            "get_Breeders",
        ], // ★ 育马者杯
        14 => &["get_ScenarioRamen", "get_WorkScenarioRamen", "get_Ramen"], // ★ 拉面杯
        _ => &[],
    };

    for name in getter_names {
        let result = call_getter_ref(chara_class, chara_obj, name);
        if !result.is_null() {
            ura_log(
                3,
                &format!(
                    "★ Scenario {} getter '{}' found at {:p}",
                    scenario_id, name, result
                ),
            );
            return result;
        }
    }

    ura_log(
        3,
        &format!("Scenario {} getter: all attempts failed", scenario_id),
    );
    ptr::null_mut()
}

// ============================================================
// ★ Read scenario detail data (/scenario endpoint)
// ============================================================

unsafe fn read_scenario_detail() -> String {
    if API.is_null() {
        return r#"{"error":"api_null"}"#.to_string();
    }

    let image = match get_image() {
        img if !img.is_null() => img,
        _ => return r#"{"error":"image_null"}"#.to_string(),
    };

    let wdm_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkDataManager").as_ptr(),
    );
    if wdm_class.is_null() {
        return r#"{"error":"no_wdm_class"}"#.to_string();
    }

    let wdm_instance = get_singleton(wdm_class);
    if wdm_instance.is_null() {
        return r#"{"error":"no_wdm_singleton"}"#.to_string();
    }

    let sm_data_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkSingleModeData").as_ptr(),
    );
    let sm_data_obj = call_getter_ref(wdm_class, wdm_instance, "get_SingleMode");
    if sm_data_obj.is_null() {
        return r#"{"error":"no_single_mode"}"#.to_string();
    }

    let chara_data_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkSingleModeCharaData").as_ptr(),
    );
    let chara_obj = call_getter_ref(sm_data_class, sm_data_obj, "get_Character");
    if chara_obj.is_null() {
        return r#"{"error":"no_chara"}"#.to_string();
    }

    let scenario_id = call_getter_int(chara_data_class, chara_obj, "get_ScenarioId");
    let scenario_obj = try_get_scenario_obj(chara_data_class, chara_obj, scenario_id);

    if scenario_obj.is_null() {
        return format!(
            r#"{{"scenario_id":{},"error":"scenario_obj_null","hint":"getter_name_not_found"}}"#,
            scenario_id
        );
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
                    "get_TeamRank",
                    "get_HavingEnhancePoint",
                    "get_PredictEnhancePoint",
                    "get_BcRaceTrackId",
                    "get_DeckId",
                    "get_TeamSpLevelLimit",
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
                    "get_EnhanceGroupArray",
                    "get_CommandInfoArray",
                    "get_TeamMemberInfoArray",
                    "get_TeamReviewResultArray",
                    "get_BcRaceResultArray",
                    "get_CommandGainExpArray",
                ];
                let mut ds_arrays = Vec::new();
                for getter in &array_getters {
                    let arr_obj = call_getter_on_instance(dataset_class, dataset_obj, getter);
                    if !arr_obj.is_null() {
                        let base = arr_obj as *const u8;
                        let length = std::ptr::read_unaligned::<usize>(
                            base.add(IL2CPP_LIST_COUNT_OFF) as *const usize,
                        );
                        ds_arrays.push(format!(
                            r#""{}":{{"len":{},"ptr":"{:p}"}}"#,
                            getter, length, arr_obj
                        ));
                    }
                }
                if !ds_arrays.is_empty() {
                    result_parts.push(format!(r#""dataset_arrays":{{{}}}"#, ds_arrays.join(",")));
                }

                // ★ Expand EnhanceGroupArray elements (Breeders buff data)
                // Element class: ObscuredSingleModeBreedersEnhanceGroup
                // Getters: get_GroupType (ObscuredInt), get_Level (ObscuredInt)
                if scenario_id == 13 {
                    let enhance_elem_class =
                        find_class_by_short_name(image, "ObscuredSingleModeBreedersEnhanceGroup");
                    if !enhance_elem_class.is_null() {
                        let enhance_arr = call_getter_on_instance(
                            dataset_class,
                            dataset_obj,
                            "get_EnhanceGroupArray",
                        );
                        if !enhance_arr.is_null() {
                            let elements = read_array_element_details(
                                enhance_arr,
                                enhance_elem_class,
                                &["get_GroupType", "get_Level"],
                                &[],
                            );
                            result_parts
                                .push(format!(r#""enhance_groups":[{}]"#, elements.join(",")));
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
                    let cmd_elem_class =
                        find_class_by_short_name(image, "ObscuredSingleModeBreedersCommandInfo");
                    if !cmd_elem_class.is_null() {
                        let cmd_arr = call_getter_on_instance(
                            dataset_class,
                            dataset_obj,
                            "get_CommandInfoArray",
                        );
                        if !cmd_arr.is_null() {
                            let elements = read_array_element_details(
                                cmd_arr,
                                cmd_elem_class,
                                &["get_CommandType", "get_CommandId", "get_RankUpPredict"],
                                &[],
                            );
                            // ★ Breeders uses SingleModeParamsIncDecInfo (plain Int32 at 0x10, 0x14)
                            //    Confirmed via Onsen scenario's ObscuredSingleModeOnsenCommandInfo
                            //    which uses SingleModeParamsIncDecInfo[] (not Data variant)
                            //    NO auto-detection needed — hardcode to avoid class lookup crashes

                            let base = cmd_arr as *const u8;
                            let cmd_len = std::ptr::read_unaligned::<usize>(
                                base.add(IL2CPP_LIST_COUNT_OFF) as *const usize,
                            );
                            let mut cmd_details = Vec::new();
                            for i in 0..cmd_len {
                                let elem_ptr = std::ptr::read_unaligned::<*mut c_void>(
                                    base.add(IL2CPP_LIST_ITEMS_OFF + i * IL2CPP_LIST_ITEM_SIZE)
                                        as *const *mut c_void,
                                );
                                let mut detail = if i < elements.len() {
                                    elements[i].clone()
                                } else {
                                    "{}".to_string()
                                };
                                // ★ Add CommandId→training name mapping
                                {
                                    let cmd_id_val = if detail.contains("\"CommandId\":") {
                                        detail
                                            .split("\"CommandId\":")
                                            .nth(1)
                                            .and_then(|s| s.split(',').next())
                                            .and_then(|s| s.trim().parse::<i32>().ok())
                                            .unwrap_or(-1)
                                    } else {
                                        -1
                                    };
                                    let cmd_name = match cmd_id_val {
                                        CMD_SPEED => "Speed",
                                        CMD_STAMINA => "Stamina",
                                        CMD_GUTS => "Guts",
                                        CMD_POWER => "Power",
                                        CMD_WISDOM => "Wiz",
                                        CMD_URA_SPEED => "Speed",
                                        CMD_URA_STAMINA => "Stamina",
                                        CMD_URA_GUTS => "Guts",
                                        CMD_URA_POWER => "Power",
                                        CMD_URA_WISDOM => "Wiz",
                                        CMD_KAKUSHIMI => "Kakushimi",
                                        _ => "Unknown",
                                    };
                                    if detail.ends_with('}') {
                                        detail.pop();
                                    }
                                    detail
                                        .push_str(&format!(",\"CommandName\":\"{}\"}}", cmd_name));
                                }
                                if !elem_ptr.is_null() {
                                    let params_arr = call_getter_on_instance(
                                        cmd_elem_class,
                                        elem_ptr,
                                        "get_ParamsIncDecInfoArray",
                                    );
                                    let mut params_items = Vec::new();
                                    if !params_arr.is_null() {
                                        let p_base = params_arr as *const u8;
                                        let p_len = std::ptr::read_unaligned::<usize>(
                                            p_base.add(IL2CPP_LIST_COUNT_OFF) as *const usize,
                                        );
                                        for j in 0..p_len {
                                            let p_elem = std::ptr::read_unaligned::<*mut c_void>(
                                                p_base.add(
                                                    IL2CPP_LIST_ITEMS_OFF
                                                        + j * IL2CPP_LIST_ITEM_SIZE,
                                                )
                                                    as *const *mut c_void,
                                            );
                                            if p_elem.is_null() {
                                                continue;
                                            }
                                            // ★ Breeders: always plain Int32 (SingleModeParamsIncDecInfo)
                                            // TargetType 实测映射（与dump.cs ParameterType枚举不同！）：
                                            //   枚举定义3=Power 4=Guts，但target_type字段实际3=Guts 4=Power
                                            //   验证：Stamina训练(TT3)加Guts，Power训练(TT4)加Power
                                            //   0=None, 1=Speed, 2=Stamina, 3=Guts, 4=Power, 5=Wiz
                                            //   10=HP, 20=Motivation, 30=SkillPt
                                            let bytes = p_elem as *const u8;
                                            let t = std::ptr::read_unaligned::<i32>(
                                                bytes.add(IL2CPP_OBSCURED_INT_KEY_OFF)
                                                    as *const i32,
                                            );
                                            let v = std::ptr::read_unaligned::<i32>(
                                                bytes.add(IL2CPP_OBSCURED_INT_HIDDEN_OFF)
                                                    as *const i32,
                                            );
                                            let (tt, val) = (t, v);
                                            let tt_name = match tt {
                                                0 => "None",
                                                1 => "Speed",
                                                2 => "Stamina",
                                                3 => "Power",
                                                4 => "Guts",
                                                5 => "Wiz",
                                                6 => "Unknown6",
                                                10 => "HP",
                                                20 => "Motivation",
                                                30 => "SkillPt",
                                                _ => "Unknown",
                                            };
                                            params_items.push(format!(r#"{{"TargetType":{},"TargetTypeName":"{}","Value":{}}}"#, tt, tt_name, val));
                                        }
                                    }
                                    // Read TeamMemberInfoArray length
                                    let member_arr = call_getter_on_instance(
                                        cmd_elem_class,
                                        elem_ptr,
                                        "get_TeamMemberInfoArray",
                                    );
                                    let member_len = if !member_arr.is_null() {
                                        let mbase = member_arr as *const u8;
                                        std::ptr::read_unaligned::<usize>(
                                            mbase.add(IL2CPP_LIST_COUNT_OFF) as *const usize,
                                        )
                                    } else {
                                        0
                                    };
                                    // Trim trailing } and add new fields
                                    if detail.ends_with('}') {
                                        detail.pop();
                                    }
                                    detail.push_str(&format!(
                                        ",\"params_inc_dec\":[{}],\"team_member_len\":{}}}",
                                        params_items.join(","),
                                        member_len
                                    ));
                                }
                                cmd_details.push(detail);
                            }
                            result_parts
                                .push(format!(r#""command_info":[{}]"#, cmd_details.join(",")));
                        }
                    }
                }

                // ★ Ramen scenario (scenario_id == 14) specific data
                if scenario_id == 14 {
                    // Read Ramen-specific ObscuredInt fields
                    let ramen_int_getters = [
                        "get_CheckPointPt",
                        "get_ExpectedCheckPointPt",
                        "get_SpecialFeelingNum",
                        "get_RecommendType",
                    ];
                    let mut ramen_ints = Vec::new();
                    for getter in &ramen_int_getters {
                        let val = call_getter_obscured_int(dataset_class, dataset_obj, getter);
                        if val >= 0 {
                            ramen_ints.push(format!(r#""{}":{}"#, getter, val));
                        }
                    }
                    if !ramen_ints.is_empty() {
                        result_parts
                            .push(format!(r#""ramen_values":{{{}}}"#, ramen_ints.join(",")));
                    }

                    // Read Ramen-specific bool fields
                    let ramen_bool_getters = [
                        "get_IsGaugeGained",
                        "get_IsUrafEffectSelectEventChecked",
                        "get_IsNotGainSpecialFeeling",
                    ];
                    let mut ramen_bools = Vec::new();
                    for getter in &ramen_bool_getters {
                        let val = call_getter_bool(dataset_class, dataset_obj, getter);
                        ramen_bools.push(format!(r#""{}":{}"#, getter, val));
                    }
                    if !ramen_bools.is_empty() {
                        result_parts
                            .push(format!(r#""ramen_bools":{{{}}}"#, ramen_bools.join(",")));
                    }

                    // Read ActiveEffectArray (Ramen current buffs)
                    // Element: ObscuredSingleModeRamenActiveEffectInfo
                    // ObscuredInt fields: EffectCategory, EffectId, EffectValue
                    let ae_arr = call_getter_on_instance(
                        dataset_class,
                        dataset_obj,
                        "get_ActiveEffectArray",
                    );
                    if !ae_arr.is_null() {
                        let ae_base = ae_arr as *const u8;
                        let ae_len = std::ptr::read_unaligned::<usize>(
                            ae_base.add(IL2CPP_LIST_COUNT_OFF) as *const usize,
                        );
                        if ae_len > 0 && ae_len < 100 {
                            let mut effects = Vec::new();
                            for i in 0..ae_len {
                                let ep = std::ptr::read_unaligned::<*mut c_void>(
                                    ae_base.add(IL2CPP_LIST_ITEMS_OFF + i * IL2CPP_LIST_ITEM_SIZE)
                                        as *const *mut c_void,
                                );
                                if ep.is_null() {
                                    continue;
                                }
                                // v3.22.51: Read class from object header — no more find_class or hardcoded offsets
                                let cat = read_obscured_int_from_obj(ep, "get_EffectCategory");
                                let eid = read_obscured_int_from_obj(ep, "get_EffectId");
                                let val = read_obscured_int_from_obj(ep, "get_EffectValue");
                                effects.push(format!(
                                    r#"{{"EffectCategory":{},"EffectId":{},"EffectValue":{}}}"#,
                                    cat, eid, val
                                ));
                            }
                            result_parts
                                .push(format!(r#""active_effects":[{}]"#, effects.join(",")));
                        }
                    }

                    // Read UrafEffectInfo (Ramen uraf effect)
                    // Class: ObscuredSingleModeRamenUrafEffectInfo
                    // ObscuredInt fields: UrafEffectType, UrafEffectState
                    let uraf_obj =
                        call_getter_on_instance(dataset_class, dataset_obj, "get_UrafEffectInfo");
                    if !uraf_obj.is_null() {
                        // v3.22.51: Read class from object header — no more find_class or hardcoded offsets
                        let ut = read_obscured_int_from_obj(uraf_obj, "get_UrafEffectType");
                        let us = read_obscured_int_from_obj(uraf_obj, "get_UrafEffectState");
                        result_parts.push(format!(
                            r#""uraf_effect":{{"UrafEffectType":{},"UrafEffectState":{}}}"#,
                            ut, us
                        ));
                    }

                    // Read SelectedRegionIdArray using read_obscured_int_array
                    let region_ids = read_obscured_int_array(
                        dataset_class,
                        dataset_obj,
                        "get_SelectedRegionIdArray",
                    );
                    if !region_ids.is_empty() {
                        result_parts.push(format!(
                            r#""selected_region_ids":[{}]"#,
                            region_ids
                                .iter()
                                .map(|x| x.to_string())
                                .collect::<Vec<_>>()
                                .join(",")
                        ));
                    }
                    let all_region_ids = read_obscured_int_array(
                        dataset_class,
                        dataset_obj,
                        "get_AllSelectedRegionIdArray",
                    );
                    if !all_region_ids.is_empty() {
                        result_parts.push(format!(
                            r#""all_selected_region_ids":[{}]"#,
                            all_region_ids
                                .iter()
                                .map(|x| x.to_string())
                                .collect::<Vec<_>>()
                                .join(",")
                        ));
                    }

                    // ★ v3.18.3: Read Ramen Feeling arrays for 隠し味の秘訣 (Kakushimi) tracking
                    // FeelingInfoArray: available Kakushimi items
                    let fi_arr =
                        call_getter_on_instance(dataset_class, dataset_obj, "get_FeelingInfoArray");
                    if !fi_arr.is_null() {
                        let fi_base = fi_arr as *const u8;
                        let fi_len = std::ptr::read_unaligned::<usize>(
                            fi_base.add(IL2CPP_LIST_COUNT_OFF) as *const usize,
                        );
                        if fi_len > 0 && fi_len < 100 {
                            // v3.22.51: Read class from each element's object header — no more find_class or hardcoded offsets
                            let mut fi_elements = Vec::new();
                            for fi in 0..fi_len {
                                let fe_ptr = std::ptr::read_unaligned::<*mut c_void>(
                                    fi_base.add(IL2CPP_LIST_ITEMS_OFF + fi * IL2CPP_LIST_ITEM_SIZE)
                                        as *const *mut c_void,
                                );
                                if fe_ptr.is_null() {
                                    fi_elements.push("{}".to_string());
                                    continue;
                                }
                                let ft = read_obscured_int_from_obj(fe_ptr, "get_FeelingIndex");
                                let fv = read_obscured_int_from_obj(fe_ptr, "get_FeelingId");
                                fi_elements.push(format!(
                                    r#"{{"FeelingIndex":{},"FeelingId":{}}}"#,
                                    ft, fv
                                ));
                            }
                            result_parts
                                .push(format!(r#""feeling_info":[{}]"#, fi_elements.join(",")));
                        }
                    }

                    // FeelingTurnInfoArray: 2 ObscuredInt fields (Turn, FeelingType)
                    // v3.22.51: Read class from object header — no more hardcoded offsets
                    let ft_arr = call_getter_on_instance(
                        dataset_class,
                        dataset_obj,
                        "get_FeelingTurnInfoArray",
                    );
                    if !ft_arr.is_null() {
                        let ft_base = ft_arr as *const u8;
                        let ft_len = std::ptr::read_unaligned::<usize>(
                            ft_base.add(IL2CPP_LIST_COUNT_OFF) as *const usize,
                        );
                        if ft_len > 0 && ft_len < 100 {
                            let mut ft_elems = Vec::new();
                            for fi in 0..ft_len {
                                let fp = std::ptr::read_unaligned::<*mut c_void>(
                                    ft_base.add(IL2CPP_LIST_ITEMS_OFF + fi * IL2CPP_LIST_ITEM_SIZE)
                                        as *const *mut c_void,
                                );
                                if fp.is_null() {
                                    ft_elems.push("{}".to_string());
                                    continue;
                                }
                                let t = read_obscured_int_from_obj(fp, "get_RemainTurn");
                                let fty = read_obscured_int_from_obj(fp, "get_FeelingId");
                                ft_elems
                                    .push(format!(r#"{{"RemainTurn":{},"FeelingId":{}}}"#, t, fty));
                            }
                            result_parts
                                .push(format!(r#""feeling_turn_info":[{}]"#, ft_elems.join(",")));
                        }
                    }

                    // CommandFeelingInfoArray: 3 ObscuredInt fields (CommandType, CommandId, FeelingId)
                    // v3.22.51: Read class from object header — no more hardcoded offsets
                    let cf_arr = call_getter_on_instance(
                        dataset_class,
                        dataset_obj,
                        "get_CommandFeelingInfoArray",
                    );
                    if !cf_arr.is_null() {
                        let cf_base = cf_arr as *const u8;
                        let cf_len = std::ptr::read_unaligned::<usize>(
                            cf_base.add(IL2CPP_LIST_COUNT_OFF) as *const usize,
                        );
                        if cf_len > 0 && cf_len < 100 {
                            let mut cf_elems = Vec::new();
                            for ci in 0..cf_len {
                                let cp = std::ptr::read_unaligned::<*mut c_void>(
                                    cf_base.add(IL2CPP_LIST_ITEMS_OFF + ci * IL2CPP_LIST_ITEM_SIZE)
                                        as *const *mut c_void,
                                );
                                if cp.is_null() {
                                    cf_elems.push("{}".to_string());
                                    continue;
                                }
                                let ct = read_obscured_int_from_obj(cp, "get_CommandType");
                                let cid = read_obscured_int_from_obj(cp, "get_CommandId");
                                let fid = read_obscured_int_from_obj(cp, "get_FeelingId");
                                cf_elems.push(format!(
                                    r#"{{"CommandType":{},"CommandId":{},"FeelingId":{}}}"#,
                                    ct, cid, fid
                                ));
                            }
                            result_parts.push(format!(
                                r#""command_feeling_info":[{}]"#,
                                cf_elems.join(",")
                            ));
                        }
                    }

                    // FeelingReduceTurnInfoArray: 2 ObscuredInt fields (Turn, FeelingType)
                    // v3.22.51: Read class from object header — no more hardcoded offsets
                    let fr_arr = call_getter_on_instance(
                        dataset_class,
                        dataset_obj,
                        "get_FeelingReduceTurnInfoArray",
                    );
                    if !fr_arr.is_null() {
                        let fr_base = fr_arr as *const u8;
                        let fr_len = std::ptr::read_unaligned::<usize>(
                            fr_base.add(IL2CPP_LIST_COUNT_OFF) as *const usize,
                        );
                        if fr_len > 0 && fr_len < 100 {
                            let mut fr_elems = Vec::new();
                            for ri in 0..fr_len {
                                let rp = std::ptr::read_unaligned::<*mut c_void>(
                                    fr_base.add(IL2CPP_LIST_ITEMS_OFF + ri * IL2CPP_LIST_ITEM_SIZE)
                                        as *const *mut c_void,
                                );
                                if rp.is_null() {
                                    fr_elems.push("{}".to_string());
                                    continue;
                                }
                                let t = read_obscured_int_from_obj(rp, "get_CommandType");
                                let fty = read_obscured_int_from_obj(rp, "get_CommandId");
                                fr_elems.push(format!(
                                    r#"{{"CommandType":{},"CommandId":{}}}"#,
                                    t, fty
                                ));
                            }
                            result_parts.push(format!(
                                r#""feeling_reduce_turn_info":[{}]"#,
                                fr_elems.join(",")
                            ));
                        }
                    }
                }

                // ★ Read object-type DataSet getters
                let obj_getters = [
                    "get_TeamSpTrainingInfo",
                    "get_NotUpParameterInfo",
                    "get_ScenarioDressSetting",
                    "get_TeamUnionEvent",
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
    if image.is_null() {
        return r#"{"error":"image_null"}"#.to_string();
    }

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
        if cls.is_null() {
            continue;
        }

        let name = if !get_name_fn.is_null() {
            let name_fn: FnClassGetName = std::mem::transmute(get_name_fn);
            let cstr = name_fn(cls);
            if cstr.is_null() {
                continue;
            }
            std::ffi::CStr::from_ptr(cstr)
                .to_string_lossy()
                .into_owned()
        } else {
            format!("class_{}", i)
        };

        let namespace = if !get_namespace_fn.is_null() {
            let ns_fn: FnClassGetName = std::mem::transmute(get_namespace_fn);
            let cstr = ns_fn(cls);
            if cstr.is_null() {
                String::new()
            } else {
                std::ffi::CStr::from_ptr(cstr)
                    .to_string_lossy()
                    .into_owned()
            }
        } else {
            String::new()
        };

        // Filter by search term if provided
        if !search.is_empty() {
            let full = format!("{}.{}", namespace, name).to_lowercase();
            if !full.contains(&search_lower) {
                continue;
            }
        }

        results.push(format!(r#"{{"ns":"{}","name":"{}"}}"#, namespace, name));
    }

    format!(
        r#"{{"total_classes":{},"matched":{},"search":"{}","classes":[{}]}}"#,
        total,
        results.len(),
        search,
        results.join(",")
    )
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
    if API.is_null() {
        return r#"{"error":"api_null"}"#.to_string();
    }

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
            let full_name = if ns.is_empty() {
                cls.to_string()
            } else {
                format!("{}.{}", ns, cls)
            };
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
        found_list.join("\",\""),
        singleton_list.join("\",\""),
        found_list.len()
    )
}

// ============================================================
// /singletons endpoint
// ============================================================

unsafe fn find_all_singletons() -> String {
    if API.is_null() {
        return r#"{"error":"api_null"}"#.to_string();
    }

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
            let full_name = if ns.is_empty() {
                cls.to_string()
            } else {
                format!("{}.{}", ns, cls)
            };
            let inst = get_singleton(class);
            let has_singleton = !inst.is_null();
            results.push(format!(
                r#"{{"class":"{}","singleton":{},"instance":"{:p}"}}"#,
                full_name, has_singleton, inst
            ));
        }
    }

    format!(
        r#"{{"total":{},"classes":[{}]}}"#,
        results.len(),
        results.join(",")
    )
}

// ============================================================
// ★ Read Training Data v3.7.8 — All via getter methods
// ============================================================

unsafe fn read_training_data() -> String {
    if API.is_null() {
        return r#"{"error":"api_null"}"#.to_string();
    }
    ura_log(3, "Reading training data v3.7.8...");

    let image = match get_image() {
        img if !img.is_null() => img,
        _ => return r#"{"error":"image_null"}"#.to_string(),
    };

    let wdm_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkDataManager").as_ptr(),
    );
    let sm_data_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkSingleModeData").as_ptr(),
    );
    let chara_data_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkSingleModeCharaData").as_ptr(),
    );

    ura_log(
        3,
        &format!(
            "Classes: WDM={} SMD={} Chara={}",
            if wdm_class.is_null() { "null" } else { "ok" },
            if sm_data_class.is_null() {
                "null"
            } else {
                "ok"
            },
            if chara_data_class.is_null() {
                "null"
            } else {
                "ok"
            },
        ),
    );

    // ===== Step 1: Get WorkDataManager singleton =====
    if wdm_class.is_null() {
        return r#"{"error":"WorkDataManager_class_not_found"}"#.to_string();
    }
    let wdm_instance = get_singleton(wdm_class);
    if wdm_instance.is_null() {
        return r#"{"error":"WorkDataManager_no_singleton","hint":"start_a_training_run"}"#
            .to_string();
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
    let month = if !sm_data_class.is_null() {
        call_getter_int(sm_data_class, sm_data_obj, "get_Month")
    } else {
        -1
    };
    let half = if !sm_data_class.is_null() {
        call_getter_int(sm_data_class, sm_data_obj, "get_Half")
    } else {
        -1
    };
    let playing_state = if !sm_data_class.is_null() {
        call_getter_int(sm_data_class, sm_data_obj, "get_PlayingState")
    } else {
        -1
    };
    let is_playing = if !sm_data_class.is_null() {
        call_getter_bool(sm_data_class, sm_data_obj, "get_IsPlaying")
    } else {
        false
    };

    ura_log(
        3,
        &format!(
            "SM Data: month={} half={} playingState={} isPlaying={}",
            month, half, playing_state, is_playing
        ),
    );

    // ===== Call get_Character() =====
    if sm_data_class.is_null() {
        return format!(
            r#"{{"error":"WorkSingleModeData_class_null","month":{},"half":{}}}"#,
            month, half
        );
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
        return read_chara_data(
            chara_field,
            chara_data_class,
            month,
            half,
            playing_state,
            is_playing,
        );
    }

    read_chara_data(
        chara_obj,
        chara_data_class,
        month,
        half,
        playing_state,
        is_playing,
    )
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
    // ★ v3.22.39: Read chara_id (card_id) for model inference input
    let chara_id = call_getter_int(chara_data_class, chara_obj, "get_CardId");

    // SkillPoint returns ObscuredInt - try the ObscuredInt decoder first,
    // fall back to regular int read if it fails
    let skill_point = call_getter_obscured_int(chara_data_class, chara_obj, "get_SkillPoint");

    // ★ Scenario buffs: charaEffectIdArray (ObscuredInt[]) and scenarioProgress (ObscuredInt)
    let chara_effect_ids =
        read_obscured_int_array(chara_data_class, chara_obj, "get_CharaEffectIdArray");
    let scenario_progress =
        call_getter_obscured_int(chara_data_class, chara_obj, "get_ScenarioProgress");

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
        speed,
        stamina,
        power,
        guts,
        wiz,
        vital: hp,
        max_vital: max_hp,
        motivation,
        turn: 0, // will be populated from WorkSingleModeData
        skill_point,
        scenario_id,
        fan_count,
        month,
        half,
        playing_state,
        is_playing,
        valid: any_valid,
    };
    CHARA = cache;

    if any_valid {
        let effect_ids_str: Vec<String> = chara_effect_ids.iter().map(|x| x.to_string()).collect();
        format!(
            r#"{{"ok":true,"chara":{{"speed":{},"stamina":{},"power":{},"guts":{},"wiz":{},"vital":{},"max_vital":{},"motivation":{},"skill_point":{},"scenario_id":{},"fan_count":{},"chara_effect_ids":[{}],"scenario_progress":{}}},"month":{},"half":{},"playing_state":{},"is_playing":{},{},"via":"WorkDataManager->get_SingleMode->get_Character->getters"}}"#,
            speed,
            stamina,
            power,
            guts,
            wiz,
            hp,
            max_hp,
            motivation,
            skill_point,
            scenario_id,
            fan_count,
            effect_ids_str.join(","),
            scenario_progress,
            month,
            half,
            playing_state,
            is_playing,
            scenario_info
        )
    } else {
        let effect_ids_str: Vec<String> = chara_effect_ids.iter().map(|x| x.to_string()).collect();
        format!(
            r#"{{"ok":false,"chara":{{"speed":{},"stamina":{},"power":{},"guts":{},"wiz":{},"vital":{},"max_vital":{},"motivation":{},"skill_point":{},"scenario_id":{},"fan_count":{},"chara_effect_ids":[{}],"scenario_progress":{}}},"month":{},"half":{},"warning":"all_fields_negative_or_zero",{},"via":"WorkDataManager->get_SingleMode->get_Character->getters"}}"#,
            speed,
            stamina,
            power,
            guts,
            wiz,
            hp,
            max_hp,
            motivation,
            skill_point,
            scenario_id,
            fan_count,
            effect_ids_str.join(","),
            scenario_progress,
            month,
            half,
            scenario_info
        )
    }
}

// ============================================================
// Enumerate ALL fields including parent classes
// ============================================================

unsafe fn enumerate_class_fields(class: *mut c_void) -> String {
    if class.is_null() || API.is_null() {
        return r#"{"error":"null_class"}"#.to_string();
    }

    let get_fields_fn: Option<FnClassGetFields> = {
        let p = resolve_il2cpp_symbol("il2cpp_class_get_fields");
        if p.is_null() {
            None
        } else {
            Some(std::mem::transmute::<*mut c_void, FnClassGetFields>(p))
        }
    };
    let get_parent_fn: Option<FnClassGetParent> = {
        let p = resolve_il2cpp_symbol("il2cpp_class_get_parent");
        if p.is_null() {
            None
        } else {
            Some(std::mem::transmute::<*mut c_void, FnClassGetParent>(p))
        }
    };
    let get_class_name_fn: Option<FnClassGetName> = {
        let p = resolve_il2cpp_symbol("il2cpp_class_get_name");
        if p.is_null() {
            None
        } else {
            Some(std::mem::transmute::<*mut c_void, FnClassGetName>(p))
        }
    };

    if get_fields_fn.is_none() {
        return r#"{"error":"no_il2cpp_class_get_fields"}"#.to_string();
    }

    let mut all_fields: Vec<String> = Vec::new();
    let field_get_type_fn: Option<unsafe extern "C" fn(*const Il2CppFieldInfo) -> *const c_void> = {
        let p = resolve_il2cpp_symbol("il2cpp_field_get_type");
        if p.is_null() { None } else { Some(std::mem::transmute(p)) }
    };
    let type_get_name_fn: Option<unsafe extern "C" fn(*const c_void) -> *const c_char> = {
        let p = resolve_il2cpp_symbol("il2cpp_type_get_name");
        if p.is_null() { None } else { Some(std::mem::transmute(p)) }
    };

    let mut current_class = class;
    let mut depth = 0;

    loop {
        if current_class.is_null() || depth > 10 {
            break;
        }

        let class_name = if let Some(ref get_name) = get_class_name_fn {
            let name_ptr = get_name(current_class);
            if !name_ptr.is_null() {
                let s = std::ffi::CStr::from_ptr(name_ptr);
                s.to_string_lossy().to_string()
            } else {
                format!("depth{}", depth)
            }
        } else {
            format!("depth{}", depth)
        };

        let mut iter: *mut c_void = ptr::null_mut();
        loop {
            let field_info = get_fields_fn.unwrap()(current_class, &mut iter);
            if field_info.is_null() {
                break;
            }

            let field_name = if !(*field_info).name.is_null() {
                let s = std::ffi::CStr::from_ptr((*field_info).name);
                s.to_string_lossy().to_string()
            } else {
                String::from("?")
            };

            let offset = (*field_info).offset;
            let type_ptr = field_get_type_fn
                .map(|f| f(field_info))
                .unwrap_or((*field_info)._ty);
            let type_enum = il2cpp_type_get_type_enum(type_ptr);
            let type_name = type_get_name_fn
                .and_then(|f| {
                    let p = f(type_ptr);
                    if p.is_null() { None } else { Some(CStr::from_ptr(p).to_string_lossy().into_owned()) }
                })
                .unwrap_or_else(|| type_enum_to_name(type_enum));
            all_fields.push(format!(
                r#"{{"name":"{}","offset":{},"class":"{}","type_enum":{},"type_name":"{}"}}"#,
                json_escape(&field_name), offset, json_escape(&class_name), type_enum,
                json_escape(&type_name)
            ));

        }
        if let Some(ref get_parent) = get_parent_fn {
            let parent = get_parent(current_class);
            if parent.is_null() || parent == current_class {
                break;
            }
            current_class = parent;
        } else {
            break;
        }
        depth += 1;
    }

    format!(
        r#"{{"total":{},"fields":[{}]}}"#,
        all_fields.len(),
        all_fields.join(",")
    )
}

// ============================================================
// ★ v3.22.51: find_field_offset — read field offset via il2cpp_class_get_fields
// Thread-safe metadata API, NO il2cpp_runtime_invoke calls
// ============================================================

unsafe fn find_field_offset(class: *mut c_void, field_name: &str) -> i32 {
    if class.is_null() || API.is_null() {
        return -1;
    }
    let get_fields_fn: Option<FnClassGetFields> = {
        let p = resolve_il2cpp_symbol("il2cpp_class_get_fields");
        if p.is_null() {
            None
        } else {
            Some(std::mem::transmute::<*mut c_void, FnClassGetFields>(p))
        }
    };
    let get_parent_fn: Option<FnClassGetParent> = {
        let p = resolve_il2cpp_symbol("il2cpp_class_get_parent");
        if p.is_null() {
            None
        } else {
            Some(std::mem::transmute::<*mut c_void, FnClassGetParent>(p))
        }
    };

    if get_fields_fn.is_none() {
        return -1;
    }

    let normalize = |name: &str| -> String {
        let n = if name.starts_with('<') {
            if let Some(end) = name.find('>') {
                &name[1..end]
            } else {
                name
            }
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
        if current_class.is_null() || depth > 10 {
            break;
        }
        let mut iter: *mut c_void = ptr::null_mut();
        loop {
            let field_info = get_fields_fn.unwrap()(current_class, &mut iter);
            if field_info.is_null() {
                break;
            }
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
            if parent.is_null() || parent == current_class {
                break;
            }
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
        if current_class.is_null() || depth > 10 {
            break;
        }
        let mut iter: *mut c_void = ptr::null_mut();
        loop {
            let field_info = get_fields_fn.unwrap()(current_class, &mut iter);
            if field_info.is_null() {
                break;
            }
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
            if parent.is_null() || parent == current_class {
                break;
            }
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
// ★ v3.22.51: Field offset cache — avoid repeated il2cpp_class_get_fields calls
// ============================================================
use std::collections::HashMap;
static FIELD_OFFSET_CACHE: std::sync::Mutex<Option<HashMap<String, i32>>> =
    std::sync::Mutex::new(None);

// v3.22.51: Zero IL2CPP API in read path
static IN_READ_PATH: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static CLASS_CACHE: std::sync::Mutex<Option<HashMap<String, usize>>> = std::sync::Mutex::new(None);
static SINGLETON_CACHE: std::sync::Mutex<Option<HashMap<usize, usize>>> =
    std::sync::Mutex::new(None);

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
    // Not in cache, look up
    let offset = find_field_offset(class, field_name);
    // Store in cache (even -1, to avoid repeated failed lookups)
    if let Ok(mut guard) = FIELD_OFFSET_CACHE.lock() {
        if guard.is_none() {
            *guard = Some(HashMap::new());
        }
        if let Some(ref mut map) = *guard {
            map.insert(key, offset);
        }
    }
    offset
}

// ============================================================
// ★ v3.22.51: read_ramen_scalar_fields — read 5 ObscuredInt fields from DataSet
// Zero il2cpp_runtime_invoke calls (only find_field_offset + read_obscured_int_at)
// ============================================================

unsafe fn read_ramen_scalar_fields(
    ds_class: *mut c_void,
    dataset_obj: *const c_void,
) -> (i32, i32, i32, i32, i32) {
    let checkpoint_pt = {
        let off = cached_find_field_offset(ds_class, "CheckPointPt");
        if off >= 0 {
            read_obscured_int_at(dataset_obj, off)
        } else {
            -1
        }
    };
    let special_feeling_num = {
        let off = cached_find_field_offset(ds_class, "SpecialFeelingNum");
        if off >= 0 {
            read_obscured_int_at(dataset_obj, off)
        } else {
            -1
        }
    };
    let recommend_type = {
        let off = cached_find_field_offset(ds_class, "RecommendType");
        if off >= 0 {
            read_obscured_int_at(dataset_obj, off)
        } else {
            -1
        }
    };
    let (uraf_type, uraf_state) = {
        let uraf_off = cached_find_field_offset(ds_class, "UrafEffectInfo");
        if uraf_off >= 0 {
            let uraf_obj = read_ptr_at(dataset_obj, uraf_off);
            if !uraf_obj.is_null() {
                let uraf_class =
                    std::ptr::read_unaligned::<*mut c_void>(uraf_obj as *const *mut c_void);
                let ut_off = cached_find_field_offset(uraf_class, "UrafEffectType");
                let us_off = cached_find_field_offset(uraf_class, "UrafEffectState");
                let ut = if ut_off >= 0 {
                    read_obscured_int_at(uraf_obj, ut_off)
                } else {
                    -1
                };
                let us = if us_off >= 0 {
                    read_obscured_int_at(uraf_obj, us_off)
                } else {
                    -1
                };
                (ut, us)
            } else {
                (-1, -1)
            }
        } else {
            (-1, -1)
        }
    };
    (
        checkpoint_pt,
        special_feeling_num,
        recommend_type,
        uraf_type,
        uraf_state,
    )
}

// ============================================================
// Enumerate methods on a class
// ============================================================

unsafe fn enumerate_class_methods(class: *mut c_void) -> String {
    if class.is_null() || API.is_null() {
        return r#"{"error":"null_class"}"#.to_string();
    }

    let get_methods_fn: Option<FnClassGetMethods> = {
        let p = resolve_il2cpp_symbol("il2cpp_class_get_methods");
        if p.is_null() {
            None
        } else {
            Some(std::mem::transmute::<*mut c_void, FnClassGetMethods>(p))
        }
    };
    let get_method_name_fn: Option<FnMethodGetName> = {
        let p = resolve_il2cpp_symbol("il2cpp_method_get_name");
        if p.is_null() {
            None
        } else {
            Some(std::mem::transmute::<*mut c_void, FnMethodGetName>(p))
        }
    };
    let get_parent_fn: Option<FnClassGetParent> = {
        let p = resolve_il2cpp_symbol("il2cpp_class_get_parent");
        if p.is_null() {
            None
        } else {
            Some(std::mem::transmute::<*mut c_void, FnClassGetParent>(p))
        }
    };
    let get_class_name_fn: Option<FnClassGetName> = {
        let p = resolve_il2cpp_symbol("il2cpp_class_get_name");
        if p.is_null() {
            None
        } else {
            Some(std::mem::transmute::<*mut c_void, FnClassGetName>(p))
        }
    };

    if get_methods_fn.is_none() {
        return r#"{"error":"no_il2cpp_class_get_methods"}"#.to_string();
    }

    let mut all_methods: Vec<String> = Vec::new();
    let mut current_class = class;
    let mut depth = 0;
    let max_methods = 500;

    loop {
        if current_class.is_null() || depth > 5 {
            break;
        }
        if all_methods.len() >= max_methods {
            break;
        }

        let class_name = if let Some(ref get_name) = get_class_name_fn {
            let name_ptr = get_name(current_class);
            if !name_ptr.is_null() {
                let s = std::ffi::CStr::from_ptr(name_ptr);
                s.to_string_lossy().to_string()
            } else {
                format!("depth{}", depth)
            }
        } else {
            format!("depth{}", depth)
        };

        let mut iter: *mut c_void = ptr::null_mut();
        loop {
            if all_methods.len() >= max_methods {
                break;
            }
            let method_info = get_methods_fn.unwrap()(current_class, &mut iter);
            if method_info.is_null() {
                break;
            }

            let method_name = if let Some(ref get_name) = get_method_name_fn {
                let name_ptr = get_name(method_info);
                if !name_ptr.is_null() {
                    let s = std::ffi::CStr::from_ptr(name_ptr);
                    s.to_string_lossy().to_string()
                } else {
                    String::from("?")
                }
            } else {
                String::from("?")
            };

            all_methods.push(format!(
                r#"{{"name":"{}","class":"{}"}}"#,
                method_name, class_name
            ));
        }

        if let Some(ref get_parent) = get_parent_fn {
            let parent = get_parent(current_class);
            if parent.is_null() || parent == current_class {
                break;
            }
            current_class = parent;
        } else {
            break;
        }
        depth += 1;
    }

    format!(
        r#"{{"total":{},"methods":[{}]}}"#,
        all_methods.len(),
        all_methods.join(",")
    )
}

// ============================================================
// /find_method endpoint
// ============================================================

unsafe fn find_method_in_all_classes(method_name: &str) -> String {
    if API.is_null() {
        return r#"{"error":"api_null"}"#.to_string();
    }

    let image = match get_image() {
        img if !img.is_null() => img,
        _ => return r#"{"error":"image_null"}"#.to_string(),
    };

    let get_method_fn: Option<FnClassGetMethodFromName> = {
        let p = resolve_il2cpp_symbol("il2cpp_class_get_method_from_name");
        if p.is_null() {
            None
        } else {
            Some(std::mem::transmute::<*mut c_void, FnClassGetMethodFromName>(p))
        }
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
        if class.is_null() {
            continue;
        }

        let full_name = if ns.is_empty() {
            cls.to_string()
        } else {
            format!("{}.{}", ns, cls)
        };

        let method = get_method_fn.unwrap()(class, method_name_c.as_ptr(), 0);
        if !method.is_null() {
            found.push(format!(r#"{{"class":"{}","args":0}}"#, full_name));
        }

        let method1 = get_method_fn.unwrap()(class, method_name_c.as_ptr(), 1);
        if !method1.is_null() && method.is_null() {
            found.push(format!(r#"{{"class":"{}","args":1}}"#, full_name));
        }
    }

    format!(
        r#"{{"method":"{}","found":{},"results":[{}]}}"#,
        method_name,
        !found.is_empty(),
        found.join(",")
    )
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
    if effect_ids.is_empty() {
        return "[]".to_string();
    }
    let mut buffs = Vec::new();
    for &id in effect_ids {
        let (name, etype) = chara_effect_name(id);
        if name.is_empty() {
            // Unknown effect — output raw ID for debugging
            buffs.push(format!(
                r#"{{"name":"Effect#{}","level":0,"desc":"unknown effect","type":"Unknown"}}"#,
                id
            ));
        } else {
            buffs.push(format!(
                r#"{{"name":"{}","level":0,"desc":"{}","type":"{}"}}"#,
                name, name, etype
            ));
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
        1 => {
            // 青: 友情ボーナス + サブ能力UP + 体力消費DOWN
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
        2 => {
            // 緑: スキルPt効果UP + ヒント発生
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
        3 => {
            // 桃: 絆獲得UP + 失敗率DOWN + 獲得上限UP
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
    0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13, 13, 14,
    14, 15, 15, 16, 16, 17, 17, 18, 18, 19, 19, 20, 20, 21, 21, 22, 22, 23, 23, 24, 24, 25, 25, 26,
    27, 28, 29, 29, 30, 31, 32, 33, 33, 34, 35, 36, 37, 37, 38, 39, 40, 41, 41, 42, 43, 44, 45, 45,
    46, 47, 48, 49, 49, 50, 51, 52, 53, 53, 54, 55, 56, 57, 57, 58, 59, 60, 61, 61, 62, 63, 64, 65,
    66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89,
    90, 91, 92, 93, 94, 95, 96, 97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110,
    111, 112, 113, 114, 115, 116, 117, 118, 120, 121, 122, 124, 125, 126, 128, 129, 130, 131, 133,
    134, 135, 137, 138, 139, 141, 142, 143, 144, 146, 147, 148, 150, 151, 152, 154, 155, 156, 157,
    159, 160, 161, 163, 164, 165, 167, 168, 169, 170, 172, 173, 174, 176, 177, 178, 180, 181, 183,
    184, 186, 188, 189, 191, 192, 194, 196, 197, 199, 200, 202, 204, 205, 207, 208, 210, 212, 213,
    215, 216, 218, 220, 221, 223, 224, 226, 228, 229, 231, 232, 234, 236, 237, 239, 240, 242, 244,
    245, 247, 248, 250, 252, 253, 255, 256, 258, 260, 261, 263, 265, 267, 269, 270, 272, 274, 276,
    278, 279, 281, 283, 285, 287, 288, 290, 292, 294, 296, 297, 299, 301, 303, 305, 306, 308, 310,
    312, 314, 315, 317, 319, 321, 323, 324, 326, 328, 330, 332, 333, 335, 337, 339, 341, 342, 344,
    346, 348, 350, 352, 354, 356, 358, 360, 362, 364, 366, 368, 371, 373, 375, 377, 379, 381, 383,
    385, 387, 389, 392, 394, 396, 398, 400, 402, 404, 406, 408, 410, 413, 415, 417, 419, 422, 423,
    425, 427, 429, 431, 434, 436, 438, 440, 442, 444, 446, 448, 450, 452, 455, 457, 459, 462, 464,
    467, 469, 471, 474, 476, 479, 481, 483, 486, 488, 491, 493, 495, 498, 500, 503, 505, 507, 510,
    512, 515, 517, 519, 522, 524, 527, 529, 531, 534, 536, 539, 541, 543, 546, 548, 551, 553, 555,
    558, 560, 563, 565, 567, 570, 572, 575, 577, 580, 582, 585, 588, 590, 593, 595, 598, 601, 603,
    606, 608, 611, 614, 616, 619, 621, 624, 627, 629, 632, 634, 637, 640, 642, 645, 647, 650, 653,
    655, 658, 660, 663, 666, 668, 671, 673, 676, 679, 681, 684, 686, 689, 692, 694, 697, 699, 702,
    705, 707, 710, 713, 716, 719, 721, 724, 727, 730, 733, 735, 738, 741, 744, 747, 749, 752, 755,
    758, 761, 763, 766, 769, 772, 775, 777, 780, 783, 786, 789, 791, 794, 797, 800, 803, 805, 808,
    811, 814, 817, 819, 822, 825, 828, 831, 833, 836, 839, 842, 845, 847, 850, 853, 856, 859, 862,
    865, 868, 871, 874, 876, 879, 882, 885, 888, 891, 894, 897, 900, 903, 905, 908, 911, 914, 917,
    920, 923, 926, 929, 931, 934, 937, 940, 943, 946, 949, 952, 955, 958, 961, 963, 966, 969, 972,
    975, 978, 981, 984, 987, 990, 993, 996, 999, 1002, 1005, 1008, 1011, 1014, 1017, 1020, 1023,
    1026, 1029, 1032, 1035, 1038, 1041, 1044, 1047, 1050, 1053, 1056, 1059, 1062, 1065, 1068, 1071,
    1074, 1077, 1080, 1083, 1086, 1089, 1092, 1095, 1098, 1101, 1104, 1107, 1110, 1113, 1116, 1119,
    1122, 1125, 1128, 1131, 1134, 1137, 1140, 1143, 1146, 1149, 1152, 1155, 1158, 1161, 1164, 1167,
    1171, 1174, 1177, 1180, 1183, 1186, 1189, 1192, 1195, 1198, 1202, 1205, 1208, 1211, 1214, 1217,
    1220, 1223, 1226, 1229, 1233, 1236, 1239, 1242, 1245, 1248, 1251, 1254, 1257, 1260, 1264, 1267,
    1270, 1273, 1276, 1279, 1282, 1285, 1288, 1291, 1295, 1298, 1301, 1304, 1308, 1311, 1314, 1318,
    1321, 1324, 1328, 1331, 1334, 1337, 1341, 1344, 1347, 1351, 1354, 1357, 1361, 1364, 1367, 1370,
    1374, 1377, 1380, 1384, 1387, 1390, 1394, 1397, 1400, 1403, 1407, 1410, 1413, 1417, 1420, 1423,
    1427, 1430, 1433, 1436, 1440, 1443, 1446, 1450, 1453, 1456, 1460, 1463, 1466, 1470, 1473, 1477,
    1480, 1483, 1487, 1490, 1494, 1497, 1500, 1504, 1507, 1511, 1514, 1517, 1521, 1524, 1528, 1531,
    1534, 1538, 1541, 1545, 1548, 1551, 1555, 1558, 1562, 1565, 1568, 1572, 1575, 1579, 1582, 1585,
    1589, 1592, 1596, 1599, 1602, 1606, 1609, 1613, 1616, 1619, 1623, 1626, 1630, 1633, 1637, 1640,
    1644, 1647, 1651, 1654, 1658, 1661, 1665, 1668, 1672, 1675, 1679, 1682, 1686, 1689, 1693, 1696,
    1700, 1703, 1707, 1710, 1714, 1717, 1721, 1724, 1728, 1731, 1735, 1738, 1742, 1745, 1749, 1752,
    1756, 1759, 1763, 1766, 1770, 1773, 1777, 1780, 1784, 1787, 1791, 1794, 1798, 1801, 1805, 1808,
    1812, 1816, 1820, 1824, 1828, 1832, 1836, 1840, 1844, 1847, 1851, 1855, 1859, 1863, 1867, 1871,
    1875, 1879, 1883, 1886, 1890, 1894, 1898, 1902, 1906, 1910, 1914, 1918, 1922, 1925, 1929, 1933,
    1937, 1941, 1945, 1949, 1953, 1957, 1961, 1964, 1968, 1972, 1976, 1980, 1984, 1988, 1992, 1996,
    2000, 2004, 2008, 2012, 2016, 2020, 2024, 2028, 2032, 2036, 2041, 2045, 2049, 2053, 2057, 2061,
    2065, 2069, 2073, 2077, 2082, 2086, 2090, 2094, 2098, 2102, 2106, 2110, 2114, 2118, 2123, 2127,
    2131, 2135, 2139, 2143, 2147, 2151, 2155, 2159, 2164, 2168, 2172, 2176, 2180, 2184, 2188, 2192,
    2196, 2200, 2205, 2209, 2213, 2217, 2221, 2226, 2230, 2234, 2238, 2242, 2247, 2251, 2255, 2259,
    2263, 2268, 2272, 2276, 2280, 2284, 2289, 2293, 2297, 2301, 2305, 2310, 2314, 2318, 2322, 2326,
    2331, 2335, 2339, 2343, 2347, 2352, 2356, 2360, 2364, 2368, 2373, 2377, 2381, 2385, 2389, 2394,
    2398, 2402, 2406, 2410, 2415, 2419, 2423, 2427, 2432, 2436, 2440, 2445, 2449, 2453, 2458, 2462,
    2466, 2470, 2475, 2479, 2483, 2488, 2492, 2496, 2501, 2505, 2509, 2513, 2518, 2522, 2526, 2531,
    2535, 2539, 2544, 2548, 2552, 2556, 2561, 2565, 2569, 2574, 2578, 2582, 2587, 2591, 2595, 2599,
    2604, 2608, 2612, 2617, 2621, 2625, 2630, 2635, 2640, 2645, 2650, 2656, 2661, 2666, 2671, 2676,
    2682, 2687, 2692, 2697, 2702, 2708, 2713, 2718, 2723, 2728, 2734, 2739, 2744, 2749, 2754, 2760,
    2765, 2770, 2775, 2780, 2786, 2791, 2796, 2801, 2806, 2812, 2817, 2822, 2827, 2832, 2838, 2843,
    2848, 2853, 2858, 2864, 2869, 2874, 2879, 2884, 2890, 2895, 2901, 2906, 2912, 2917, 2923, 2928,
    2934, 2939, 2945, 2950, 2956, 2961, 2967, 2972, 2978, 2983, 2989, 2994, 3000, 3005, 3011, 3016,
    3022, 3027, 3033, 3038, 3044, 3049, 3055, 3060, 3066, 3071, 3077, 3082, 3088, 3093, 3099, 3104,
    3110, 3115, 3121, 3126, 3132, 3137, 3143, 3148, 3154, 3159, 3165, 3171, 3178, 3184, 3191, 3198,
    3204, 3211, 3217, 3224, 3231, 3237, 3244, 3250, 3257, 3264, 3270, 3277, 3283, 3290, 3297, 3303,
    3310, 3316, 3323, 3330, 3336, 3343, 3349, 3356, 3363, 3369, 3376, 3382, 3389, 3396, 3402, 3409,
    3415, 3422, 3429, 3435, 3442, 3448, 3455, 3462, 3468, 3475, 3481, 3488, 3495, 3501, 3508, 3515,
    3522, 3529, 3535, 3542, 3549, 3556, 3563, 3569, 3576, 3583, 3590, 3597, 3603, 3610, 3617, 3624,
    3631, 3637, 3644, 3651, 3658, 3665, 3671, 3678, 3685, 3692, 3699, 3705, 3712, 3719, 3726, 3733,
    3739, 3746, 3753, 3760, 3767, 3773, 3780, 3787, 3794, 3801, 3807, 3814, 3821, 3828, 3835, 3841,
    3849, 3857, 3865, 3873, 3881, 3889, 3896, 3904, 3912, 3920, 3928, 3936, 3944, 3952, 3960, 3968,
    3976, 3984, 3992, 4000, 4008, 4016, 4025, 4033, 4041, 4049, 4057, 4065, 4073, 4082, 4090, 4098,
    4106, 4115, 4123, 4131, 4139, 4148, 4156, 4164, 4173, 4181, 4189, 4198, 4206, 4215, 4223, 4231,
    4240, 4248, 4257, 4265, 4274, 4282, 4291, 4299, 4308, 4316, 4325, 4334, 4342, 4351, 4359, 4368,
    4377, 4385, 4394, 4403, 4411, 4420, 4429, 4438, 4446, 4455, 4464, 4473, 4482, 4490, 4499, 4508,
    4517, 4526, 4535, 4544, 4553, 4561, 4570, 4579, 4588, 4597, 4606, 4615, 4624, 4633, 4642, 4651,
    4661, 4670, 4679, 4688, 4697, 4706, 4715, 4724, 4734, 4743, 4752, 4761, 4770, 4780, 4789, 4798,
    4808, 4817, 4826, 4835, 4845, 4854, 4863, 4873, 4882, 4892, 4901, 4910, 4920, 4929, 4939, 4948,
    4958, 4967, 4977, 4986, 4996, 5005, 5015, 5025, 5034, 5044, 5053, 5063, 5073, 5082, 5092, 5102,
    5111, 5121, 5131, 5141, 5150, 5160, 5170, 5180, 5190, 5199, 5209, 5219, 5229, 5239, 5249, 5259,
    5268, 5278, 5288, 5298, 5308, 5318, 5328, 5338, 5348, 5358, 5368, 5378, 5388, 5398, 5409, 5419,
    5429, 5439, 5449, 5459, 5469, 5480, 5490, 5500, 5510, 5520, 5531, 5541, 5551, 5562, 5572, 5582,
    5593, 5603, 5613, 5624, 5634, 5644, 5655, 5665, 5676, 5686, 5697, 5707, 5717, 5728, 5739, 5749,
    5760, 5770, 5781, 5791, 5802, 5812, 5823, 5834, 5844, 5855, 5866, 5876, 5887, 5898, 5908, 5919,
    5930, 5941, 5952, 5962, 5973, 5984, 5995, 6006, 6016, 6027, 6038, 6049, 6060, 6071, 6082, 6093,
    6104, 6115, 6126, 6137, 6148, 6159, 6170, 6181, 6192, 6203, 6214, 6225, 6236, 6247, 6259, 6270,
    6281, 6292, 6303, 6314, 6326, 6337, 6348, 6359, 6371, 6382, 6393, 6405, 6416, 6427, 6439, 6450,
    6461, 6473, 6484, 6496, 6507, 6518, 6530, 6541, 6553, 6564, 6576, 6587, 6599, 6610, 6622, 6634,
    6645, 6657, 6668, 6680, 6692, 6703, 6715, 6727, 6738, 6750, 6762, 6773, 6785, 6797, 6809, 6821,
    6832, 6844, 6856, 6868, 6880, 6891, 6903, 6915, 6927, 6939, 6951, 6963, 6975, 6987, 6999, 7011,
    7023, 7035, 7047, 7059, 7071, 7083, 7095, 7107, 7119, 7131, 7144, 7156, 7168, 7180, 7192, 7204,
    7217, 7229, 7241, 7253, 7266, 7278, 7290, 7303, 7315, 7327, 7340, 7352, 7364, 7377, 7389, 7402,
    7414, 7426, 7439, 7451, 7464, 7476, 7489, 7501, 7514, 7526, 7539, 7551, 7564, 7577, 7589, 7602,
    7615, 7627, 7640, 7652, 7665, 7678, 7691, 7703, 7716, 7729, 7742, 7754, 7767, 7780, 7793, 7806,
    7818, 7831, 7844, 7857, 7870, 7883, 7896, 7909, 7922, 7935, 7948, 7961, 7974, 7987, 8000, 8013,
    8026, 8039, 8052, 8065, 8078, 8091, 8104, 8117, 8131, 8144, 8157, 8170, 8183, 8197, 8210, 8223,
    8236, 8250, 8263, 8276, 8290, 8303, 8316, 8330, 8343, 8356, 8370, 8383, 8397, 8410, 8423, 8437,
    8450, 8464, 8477, 8491, 8504, 8518, 8531, 8545, 8559, 8572, 8586, 8599, 8613, 8627, 8640, 8654,
    8668, 8681, 8695, 8709, 8723, 8736, 8750, 8764, 8778, 8791, 8805, 8819, 8833, 8847, 8861, 8874,
    8888, 8902, 8916, 8930, 8944, 8958, 8972, 8986, 9000, 9014, 9028, 9042, 9056, 9070, 9084, 9098,
    9112, 9127, 9141, 9155, 9169, 9183, 9197, 9212, 9226, 9240, 9254, 9268, 9283, 9297, 9311, 9326,
    9340, 9354, 9369, 9383, 9397, 9412, 9426, 9440, 9455, 9469, 9484, 9498, 9513, 9527, 9542, 9556,
    9571, 9585, 9600, 9614, 9629, 9643, 9658, 9673, 9687, 9702, 9717, 9731, 9746, 9761, 9775, 9790,
    9805, 9819, 9834, 9849, 9864, 9879, 9893, 9908, 9923, 9938, 9953, 9968, 9982, 9997, 10012,
    10027, 10042, 10057, 10072, 10087, 10102, 10117, 10132, 10147, 10162, 10177, 10192, 10207,
    10222, 10238, 10253, 10268, 10283, 10298, 10313, 10329, 10344, 10359, 10374, 10389, 10405,
    10420, 10435, 10450, 10466, 10481, 10496, 10512, 10527, 10543, 10558, 10573, 10589, 10604,
    10620, 10635, 10650, 10666, 10681, 10697, 10712, 10728, 10744, 10759, 10775, 10790, 10806,
    10821, 10837, 10853, 10868, 10884, 10900, 10915, 10931, 10947, 10963, 10978, 10994, 11010,
    11026, 11041, 11057, 11073, 11089, 11105, 11120, 11136, 11152, 11168, 11184, 11200, 11216,
    11232, 11248, 11264, 11280, 11296, 11312, 11328, 11344, 11360, 11376, 11392, 11408, 11424,
    11440, 11457, 11473, 11489, 11505, 11521, 11537, 11554, 11570, 11586, 11602, 11619, 11635,
    11651, 11667, 11684, 11700, 11716, 11733, 11749, 11765, 11782, 11798, 11815, 11831, 11848,
    11864, 11881, 11897, 11914, 11930, 11947, 11963, 11980, 11996, 12013, 12029, 12046, 12063,
    12079, 12096, 12113, 12129, 12146, 12163, 12179, 12196, 12213, 12229, 12246, 12263, 12280,
    12297, 12313, 12330, 12347, 12364, 12381, 12398, 12415, 12431, 12448, 12465, 12482, 12499,
    12516, 12533, 12550, 12567, 12584, 12601, 12618, 12635, 12652, 12670, 12687, 12704, 12721,
    12738, 12755, 12772, 12789, 12807, 12824, 12841, 12858, 12876, 12893, 12910, 12927, 12945,
    12962, 12979, 12997, 13014, 13031, 13049, 13066, 13084, 13101, 13118, 13136, 13153, 13171,
    13188, 13206, 13223, 13241, 13258, 13276, 13293, 13311, 13329, 13346, 13364, 13381, 13399,
    13417, 13434, 13452, 13470, 13487, 13505, 13523, 13541, 13558, 13576, 13594, 13612, 13630,
    13647, 13665, 13683, 13701, 13719, 13737, 13755, 13772, 13790, 13808, 13826, 13844, 13862,
    13880, 13898, 13916, 13934, 13952, 13970, 13988, 14007, 14025, 14043, 14061, 14079, 14097,
    14115, 14133, 14152, 14170, 14188, 14206, 14225, 14243, 14261, 14279, 14298, 14316, 14334,
    14353, 14371, 14389, 14408, 14426, 14444, 14463, 14481, 14500, 14518, 14537, 14555, 14574,
    14592, 14611, 14629, 14648, 14666, 14685, 14703, 14722, 14741, 14759, 14778, 14797, 14815,
    14834, 14853, 14871, 14890, 14909, 14927, 14946, 14965, 14984, 15003, 15021, 15040, 15059,
    15078, 15097, 15116, 15134, 15153, 15172, 15191, 15210, 15229, 15248, 15267, 15286, 15305,
    15324, 15343, 15362, 15381, 15400, 15419, 15438, 15457, 15477, 15496, 15515, 15534, 15553,
    15572, 15592, 15611, 15630, 15649, 15668, 15688, 15707, 15726, 15746, 15765, 15784, 15804,
    15823, 15842, 15862, 15881, 15900, 15920, 15939, 15959, 15978, 15998, 16017, 16037, 16056,
    16076, 16095, 16115, 16134, 16154, 16174, 16193, 16213, 16232, 16252, 16272, 16291, 16311,
    16331, 16350, 16370, 16390, 16410, 16429, 16449, 16469, 16489, 16509, 16528, 16548, 16568,
    16588, 16608, 16628, 16648, 16668, 16688, 16707, 16727, 16747, 16767, 16787, 16807, 16827,
    16847, 16867, 16888, 16908, 16928, 16948, 16968, 16988, 17008, 17028, 17049, 17069, 17089,
    17109, 17129, 17150, 17170, 17190, 17210, 17231, 17251, 17271, 17292, 17312, 17332, 17353,
    17373, 17393, 17414, 17434, 17455, 17475, 17496, 17516, 17536, 17557, 17577, 17598, 17619,
    17639, 17660, 17680, 17701, 17721, 17742, 17763, 17783, 17804, 17825, 17845, 17866, 17887,
    17907, 17928, 17949, 17970, 17990, 18011, 18032, 18053, 18074, 18094, 18115, 18136, 18157,
    18178, 18199, 18220, 18241, 18262, 18283, 18303, 18324, 18345, 18366, 18387, 18409, 18430,
    18451, 18472, 18493, 18514, 18535, 18556, 18577, 18598, 18620, 18641, 18662, 18683, 18704,
    18726, 18747, 18768, 18789, 18811, 18832, 18853, 18875, 18896, 18917, 18939, 18960, 18981,
    19003, 19024, 19046, 19067, 19088, 19110, 19131, 19153, 19174, 19196, 19217, 19239, 19261,
    19282, 19304, 19325, 19347, 19369, 19390, 19412, 19433, 19455, 19477, 19499, 19520, 19542,
    19564, 19585, 19607, 19629, 19651, 19673, 19694, 19716, 19738, 19760, 19782, 19804, 19826,
    19848, 19869, 19891, 19913, 19935, 19957, 19979, 20001, 20023, 20045, 20067, 20089, 20111,
    20134, 20156, 20178, 20200, 20222, 20244, 20266, 20288, 20311, 20333, 20355,
];

const BASIC_FIVE_STATUS_LIMIT: [i32; 5] = [2300, 2200, 1800, 1400, 1400];
// === AI Evaluation Named Constants ===
// Vital evaluation piecewise: slopes and breakpoints
const VITAL_EVAL_LOW_SLOPE: f64 = 2.0; // vital ≤50: steep slope
const VITAL_EVAL_MID_SLOPE: f64 = 1.5; // vital 50-70: moderate slope
const VITAL_EVAL_HIGH_SLOPE: f64 = 1.0; // vital >70: flat slope
const VITAL_EVAL_LOW_THRESH: i32 = 50; // low→mid breakpoint
const VITAL_EVAL_MID_THRESH: i32 = 70; // mid→high breakpoint
                                       // Derived intercepts (precomputed to avoid recomputation)
const VITAL_EVAL_MID_INTERCEPT: f64 = 100.0; // VITAL_EVAL_LOW_SLOPE * VITAL_EVAL_LOW_THRESH = 2.0*50
const VITAL_EVAL_HIGH_INTERCEPT: f64 = 130.0; // 100.0 + 1.5*(70-50) = mid_intercept + mid_slope*(mid_thresh-low_thresh)

// Vital factor: controls how much we value vitality
const VITAL_FACTOR_BASE: f64 = 3.5; // starting vital factor
const VITAL_FACTOR_RANGE: f64 = 3.5; // added over full game (base→7.0 at end)

// Soft constraint: reserve multiplier for stat overflow penalty
const RESERVE_MULTIPLIER: f64 = 40.0;
const RESERVE_MIN: f64 = 0.1; // avoid division by zero

// URA event final bonus (stats gained from non-training events)
const URA3_BONUS: i32 = 45; // URA scenario 3rd event
const URA_FINAL_EVENT_BONUS: i32 = 30; // final event after training
const URA_EVENT_BONUS: i32 = 20; // URA1/URA2 event bonus

// Training evaluation parameters
const STATUS_WEIGHT: f64 = 6.0; // per-stat weight (uniform for all 5)
const SMALL_FAIL_VALUE: f64 = -150.0; // minor failure penalty
const BIG_FAIL_VALUE: f64 = -500.0; // major failure (大失敗) penalty
const PT_SCORE_RATE: f64 = 2.0; // skill point → evaluation value rate
const FAIL_RATE_TO_PROB: f64 = 0.01; // convert percentage (0-100) to probability
const BIG_FAIL_THRESHOLD: i32 = 20; // fail_rate below this → no 大失敗

// Shining (彩圈) and heads (相伴) bonus
const SHINING_BONUS_PER: f64 = 200.0; // expected value per 彩圈 partner
const HEADS_BONUS_PER: f64 = 20.0; // small bonus per extra partner

// Rest/Outgoing vital gain
const REST_VITAL_GAIN: i32 = 50; // vital gained from rest
const OUTGOING_VITAL_GAIN: i32 = 50; // vital gained from outgoing

// Motivation factor: scales training value by current mood
// 1=絶不調, 2=不調, 3=普通, 4=好調, 5=絶好調
const MOT_FACTOR_WORST: f64 = 0.6; // motivation 1
const MOT_FACTOR_BAD: f64 = 0.75; // motivation 2
const MOT_FACTOR_NORMAL: f64 = 0.9; // motivation 3
const MOT_FACTOR_GOOD: f64 = 1.0; // motivation 4
const MOT_FACTOR_BEST: f64 = 1.1; // motivation 5

// Outgoing motivation bonus (motivation level → value of raising it)
const OUTGOING_BONUS_MOT1: f64 = 80.0; // 絶不調→不調: urgent
const OUTGOING_BONUS_MOT2: f64 = 50.0; // 不調→普通: important
const OUTGOING_BONUS_MOT3: f64 = 25.0; // 普通→好調: moderate
const OUTGOING_BONUS_MOT4: f64 = 10.0; // 好調→絶好調: minor

// Game scenario total turns
const URA_TOTAL_TURNS: i32 = 78; // URA scenario has 78 training turns
const DEFAULT_TOTAL_TURNS: i32 = 72; // Standard scenarios have 72 turns

// Game CommandId constants (IL2CPP method identifiers)
// 2026-07-17 修正: 102=力量(Power), 105=耐力(Stamina) — 游戏内ID与UI顺序速耐力根智相反
// 证据: support_card_data 名卡 command_id — 小海湾SSR 30016=105, 麦昆SSR 30022/30139=105,
// 小栗帽SSR 30024=102, 北黑SSR 30028=101, 诗歌剧SSR 30030=103
// Ramen(剧本14)指令 601-605 经 MDB single_mode_training.base_command_id 实证: 601→101速度, 602→105耐力, 603→102力量, 604→103根性, 605→106智力
const CMD_SPEED: i32 = 101;
const CMD_STAMINA: i32 = 105;
const CMD_GUTS: i32 = 103;
const CMD_POWER: i32 = 102;
const CMD_WISDOM: i32 = 106;
const CMD_URA_SPEED: i32 = 601;
const CMD_URA_STAMINA: i32 = 602;
const CMD_URA_GUTS: i32 = 604;
const CMD_URA_POWER: i32 = 603;
const CMD_URA_WISDOM: i32 = 605;
const CMD_KAKUSHIMI: i32 = 304;

/// Validate the ordinary training specialty stored in
/// support_card_data.command_id.
///
/// MasterDB v2.28.5 contains the five normal command IDs below plus 0.
/// A value of 0 belongs to friend/group/special cards, whose shining rules
/// cannot use the ordinary attribute-card rule.
fn support_card_command_id_to_training_id(command_id: i32) -> Option<i32> {
    match command_id {
        CMD_SPEED | CMD_STAMINA | CMD_POWER | CMD_GUTS | CMD_WISDOM => Some(command_id),
        _ => None,
    }
}

/// Normalize normal and scenario-specific training IDs to the same
/// five standard training IDs.
fn normalize_training_command_id(command_id: i32) -> Option<i32> {
    match command_id {
        CMD_SPEED | CMD_URA_SPEED => Some(CMD_SPEED),
        CMD_STAMINA | CMD_URA_STAMINA => Some(CMD_STAMINA),
        CMD_POWER | CMD_URA_POWER => Some(CMD_POWER),
        CMD_GUTS | CMD_URA_GUTS => Some(CMD_GUTS),
        CMD_WISDOM | CMD_URA_WISDOM => Some(CMD_WISDOM),
        _ => None,
    }
}

// URA turn thresholds for max vital equivalent calculation
const URA_LAST_TURN: i32 = 76; // URA finals: no vital needed
const URA_PRE_FINAL_TURN: i32 = 71; // just before URA: minimal vital
const URA_PRE_FINAL_VITAL: i32 = 10; // vital needed at pre-final turn
const URA_FINAL_VITAL: i32 = 30; // vital needed at final training turn
const URA_MAX_NON_RACE_TURNS: i32 = 6; // max non-race turns before URA
const URA_VITAL_PER_NON_RACE: i32 = 15; // vital equivalent per non-race turn
const TEXT_DATA_CATEGORY_CHARA_NAME: i32 = 6; // text_data.category=6: character name
const TEXT_DATA_CATEGORY_RACE_NAME: i32 = 32; // text_data.category=32: race name
const TEXT_DATA_CATEGORY_STORY_TITLE: i32 = 45; // text_data.category=45: single mode story title
const TEXT_DATA_CATEGORY_SKILL_NAME: i32 = 47; // text_data.category=47: skill name
const IL2CPP_LIST_COUNT_OFF: usize = 0x18; // Il2CppList._count (il2cpp internal, all List<T>)
const IL2CPP_LIST_ITEMS_OFF: usize = 0x20; // Il2CppList._items[0] start (il2cpp internal, all List<T>)
const IL2CPP_LIST_ITEM_SIZE: usize = 0x08; // sizeof(pointer) on aarch64
const IL2CPP_OBSCURED_INT_KEY_OFF: usize = 0x10; // ObscuredInt.currentCryptoKey (boxed layout)
const IL2CPP_OBSCURED_INT_HIDDEN_OFF: usize = 0x14; // ObscuredInt.hiddenValue (boxed layout)
const IL2CPP_UNBOX_FIRST_FIELD: usize = 0x10; // Unbox() result: first field offset (after Il2CppObject header 0x10)
const IL2CPP_UNBOX_SECOND_FIELD: usize = 0x14; // Unbox() result: second field offset
const IL2CPP_SUPPORT_CARD_POSITION_OFF: usize = 0x10; // SingleModeEquipSupportCard.position (IL2CPP /fields/ offset=16)
const IL2CPP_SUPPORT_CARD_ID_OFF: usize = 0x14; // SingleModeEquipSupportCard.supportCardId (IL2CPP /fields/ offset=20)
const IL2CPP_SUPPORT_CARD_LIMIT_OFF: usize = 0x18; // SingleModeEquipSupportCard.limitBreakCount (IL2CPP /fields/ offset=24)
const IL2CPP_TARGET_RACE_ID_OFF: usize = 0x10; // SingleModeTargetRace.targetId (IL2CPP /fields/ offset=16)
const IL2CPP_TARGET_RACE_EVAL_OFF: usize = 0x14; // SingleModeTargetRace.evaluation (IL2CPP /fields/ offset=20)

// ★ v3.24.9: Named offsets for ObscuredSingleModeRamenCommandInfo (confirmed by /debug/dumpclass)
const RAMEN_CMD_COMMAND_TYPE_OFF: usize = 0x10; // CommandType (ObscuredInt inline, 20 bytes)
const RAMEN_CMD_COMMAND_ID_OFF: usize = 0x24; // CommandId (ObscuredInt inline, 20 bytes) — key at 0x24, hidden at 0x28
const RAMEN_CMD_PARAMS_ARRAY_OFF: usize = 0x38; // ParamsIncDecInfoArray (List ptr)
const RAMEN_DATASET_CMD_ARRAY_OFF: usize = 0x10; // DataSet.CommandInfoArray (List ptr at offset 16)
                                                 // SingleModeParamsIncDecInfo: two plain Int32 fields.
                                                 // These constants are retained for Breeders/Ramen DataSet diagnostic paths.
const PARAMS_INCDEC_TARGET_TYPE_OFF: usize = 0x10;
const PARAMS_INCDEC_VALUE_OFF: usize = 0x14;

// SingleModeParamsIncDecInfoData: two inline ObscuredInt fields.
// HomeInfoData.CommandInfoArray uses this layout.
const PARAMS_INCDEC_DATA_TARGET_TYPE_OFF: i32 = 0x10;
const PARAMS_INCDEC_DATA_VALUE_OFF: i32 = 0x24;
// WorkSingleModeCharaData._evaluationList contains Evaluation objects.
// Runtime diagnostic 2026-07-10 confirmed both fields are inline ObscuredInt values.
const EVALUATION_PARTNER_ID_OFF: i32 = 0x10;
const EVALUATION_VALUE_OFF: i32 = 0x24;
const TRAINING_PARTNER_ARRAY_OFF: i32 = 0x50;  // = 80
const TIPS_EVENT_PARTNER_ARRAY_OFF: i32 = 0x58;  // = 88
const EVALUATION_LIST_OFF: i32 = 0x3f8;
const OBSCURED_INT_SIZE: usize = 20;
const IL2CPP_COMMAND_ID_OFF: usize = 0x10; // SingleModeCommandId.commandId (IL2CPP /fields/ offset=16)
const IL2CPP_COMMAND_LEVEL_OFF: usize = 0x14; // SingleModeCommandId.level (IL2CPP /fields/ offset=20)
const IL2CPP_OBSCURED_INT_UNBOX_KEY_OFF: usize = 0x10; // ObscuredInt unboxed: currentCryptoKey (offset=0x10)
const IL2CPP_OBSCURED_INT_UNBOX_HIDDEN_OFF: usize = 0x14; // ObscuredInt unboxed: hiddenValue (offset=0x14)
const IL2CPP_OBSCURED_INT_PAIR2_KEY_OFF: usize = 0x24; // Second ObscuredInt in pair: currentCryptoKey (offset=0x24)
const IL2CPP_OBSCURED_INT_PAIR2_HIDDEN_OFF: usize = 0x28; // Second ObscuredInt in pair: hiddenValue (offset=0x28)
const IL2CPP_LIST_ARRAY_OFF: usize = 0x10; // Il2CppList._items array pointer (offset=0x10)

/// Compute current evaluation score from five stats (per-stat lookup then sum)
/// 評価点 = STAT_EVAL_SCORE[speed] + STAT_EVAL_SCORE[stamina] + ... + STAT_EVAL_SCORE[wiz]
fn compute_score(speed: i32, stamina: i32, power: i32, guts: i32, wiz: i32) -> i32 {
    let lookup = |x: i32| -> i32 {
        if x <= 0 {
            return 0;
        }
        let idx = x as usize;
        if idx >= STAT_EVAL_SCORE.len() {
            return STAT_EVAL_SCORE[STAT_EVAL_SCORE.len() - 1];
        }
        STAT_EVAL_SCORE[idx]
    };
    lookup(speed) + lookup(stamina) + lookup(power) + lookup(guts) + lookup(wiz)
}

/// Soft constraint function for stat overflow control
/// When stat gain would exceed remaining space, reduce its effective value
fn status_soft_function(x: f64, reserve: f64) -> f64 {
    if x >= 0.0 {
        return 0.0;
    }
    if x > -reserve {
        return -x * x / (2.0 * reserve);
    }
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
    if turn >= URA_LAST_TURN {
        return 0;
    }
    if turn > URA_PRE_FINAL_TURN {
        return URA_PRE_FINAL_VITAL;
    }
    if turn == URA_PRE_FINAL_TURN {
        return URA_FINAL_VITAL;
    }
    let non_race_turns = std::cmp::min(URA_MAX_NON_RACE_TURNS, URA_PRE_FINAL_TURN - turn);
    let eq = URA_FINAL_VITAL + URA_VITAL_PER_NON_RACE * non_race_turns;
    if eq > max_vital {
        max_vital
    } else {
        eq
    }
}

/// CommandId → training index (0=Speed, 1=Stamina, 2=Power, 3=Guts, 4=Wisdom)
fn cmd_id_to_train_idx(cmd_id: i32) -> Option<usize> {
    match cmd_id {
        CMD_SPEED | CMD_URA_SPEED => Some(0),
        CMD_STAMINA | CMD_URA_STAMINA => Some(1),
        CMD_POWER | CMD_URA_POWER => Some(2),
        CMD_GUTS | CMD_URA_GUTS => Some(3),
        CMD_WISDOM | CMD_URA_WISDOM => Some(4),
        _ => None,
    }
}

/// AI evaluation result
struct AiResult {
    score: i32,                       // Current evaluation score (attribute + skill)
    skill_eval: i32,                  // Skill evaluation value
    skill_count: i32,                 // Number of learned skills
    total_stats: i32,                 // Total revised stats
    best_action: String,              // Recommended action name
    best_value: f64,                  // Best action value
    train_values: Vec<(String, f64)>, // Per-training values
    rest_value: f64,                  // Rest value
    outgoing_value: f64,              // Outgoing value
}

/// Run handwritten AI evaluation for current game state
/// Input: all data from read_summary_inner
fn evaluate_ai(
    turn: i32,
    stats: [i32; 5], // [speed, stamina, power, guts, wiz]
    vital: i32,
    max_vital: i32,
    motivation: i32, // 1-5
    scenario_id: i32,
    // Per-training data: (command_id, [5 stat gains], skill_pt_gain, vital_cost, failure_rate, is_enable, shining, heads)
    trainings: &[(i32, [i32; 5], i32, i32, i32, i32, i32, i32)],
    // Buff effects
    _has_ai_jiao: bool,       // 愛嬌 buff (TODO: implement buff effect)
    _has_renshou_jouzu: bool, // 練習上手 buff (TODO: implement buff effect)
    skill_eval: i32,          // ★ v3.22.0: skill evaluation value
    skill_count: i32,         // ★ v3.22.0: learned skill count
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
    let score = attr_score + skill_eval; // ★ v3.22.0: attribute + skill evaluation
    let total_stats = stats[0] + stats[1] + stats[2] + stats[3] + stats[4];

    // === Evaluation Parameters ===
    let status_weights = [
        STATUS_WEIGHT,
        STATUS_WEIGHT,
        STATUS_WEIGHT,
        STATUS_WEIGHT,
        STATUS_WEIGHT,
    ];
    let small_fail_value = SMALL_FAIL_VALUE;
    let big_fail_value = BIG_FAIL_VALUE;
    let pt_score_rate = PT_SCORE_RATE;

    // Vital factor: increases from 3.5 to 7.0 as game progresses
    let vital_factor = VITAL_FACTOR_BASE + (turn as f64 / total_turn as f64) * VITAL_FACTOR_RANGE;

    // Reserve for soft constraint: controls stat overflow penalty
    let reserve = RESERVE_MULTIPLIER
        * remain_turn as f64
        * (1.0 - remain_turn as f64 / (total_turn as f64 * 2.0));
    let reserve = if reserve > RESERVE_MIN {
        reserve
    } else {
        RESERVE_MIN
    };

    // URA final bonus (events that add stats after training)
    let mut final_bonus = URA3_BONUS + URA_FINAL_EVENT_BONUS;
    if remain_turn >= 1 {
        final_bonus += URA_EVENT_BONUS;
    } // URA2
    if remain_turn >= 2 {
        final_bonus += URA_EVENT_BONUS;
    } // URA1

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
    for &(cmd_id, ref gains, skill_pt, vital_cost, fail_rate, is_enable, shining, heads) in
        trainings
    {
        let name = match cmd_id {
            CMD_SPEED => "Speed",
            CMD_STAMINA => "Stamina",
            CMD_GUTS => "Guts",
            CMD_POWER => "Power",
            CMD_WISDOM => "Wisdom",
            CMD_URA_SPEED => "Speed",
            CMD_URA_STAMINA => "Stamina",
            CMD_URA_GUTS => "Guts",
            CMD_URA_POWER => "Power",
            CMD_URA_WISDOM => "Wisdom",
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
            let big_fail_prob = if fail_rate < BIG_FAIL_THRESHOLD {
                0.0
            } else {
                fail_rate as f64
            };
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
    let outgoing_value = vital_factor
        * (vital_evaluation(vital_after_outgoing, max_vital) - vital_before)
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
    let tv: Vec<String> = r
        .train_values
        .iter()
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
unsafe fn read_chara_skills(
    chara_class: *mut c_void,
    chara_obj: *const c_void,
    image: *const c_void,
) -> Vec<(i32, i32)> {
    let mut skills = Vec::new();

    // Approach 1: Try get_SkillDataArray() -> SingleModeSkillData[]
    let arr = call_getter_on_instance(chara_class, chara_obj, "get_SkillDataArray");
    if !arr.is_null() {
        let ab = arr as *const u8;
        let al = std::ptr::read_unaligned::<usize>(ab.add(IL2CPP_LIST_COUNT_OFF) as *const usize);
        if al > 0 && al < 500 {
            let skill_elem_class = find_class_by_short_name(image, "SingleModeSkillData");
            for i in 0..al {
                let ep = std::ptr::read_unaligned::<*mut c_void>(
                    ab.add(IL2CPP_LIST_ITEMS_OFF + i * IL2CPP_LIST_ITEM_SIZE) as *const *mut c_void,
                );
                if ep.is_null() {
                    continue;
                }
                let skill_id = if !skill_elem_class.is_null() {
                    call_getter_int(skill_elem_class, ep, "get_SkillId")
                } else {
                    std::ptr::read_unaligned::<i32>(
                        (ep as *const u8).add(IL2CPP_UNBOX_FIRST_FIELD) as *const i32
                    )
                };
                let level = if !skill_elem_class.is_null() {
                    call_getter_int(skill_elem_class, ep, "get_Level")
                } else {
                    1
                };
                if skill_id > 0 {
                    skills.push((skill_id, if level > 0 { level } else { 1 }));
                }
            }
            if !skills.is_empty() {
                return skills;
            }
        }
    }

    // Approach 2: Try get_PossessSkillIdArray() -> int[]
    for method_name in &["get_PossessSkillIdArray", "get_SkillIdArray"] {
        let arr2 = call_getter_on_instance(chara_class, chara_obj, method_name);
        if arr2.is_null() {
            continue;
        }
        let ab = arr2 as *const u8;
        let al = std::ptr::read_unaligned::<usize>(ab.add(IL2CPP_LIST_COUNT_OFF) as *const usize);
        if al > 0 && al < 500 {
            for i in 0..al {
                let ep = std::ptr::read_unaligned::<*mut c_void>(
                    ab.add(IL2CPP_LIST_ITEMS_OFF + i * IL2CPP_LIST_ITEM_SIZE) as *const *mut c_void,
                );
                if ep.is_null() {
                    continue;
                }
                let sid = std::ptr::read_unaligned::<i32>(ep as *const i32);
                if sid > 0 {
                    skills.push((sid, 1));
                }
            }
        }
        if !skills.is_empty() {
            return skills;
        }
    }

    // Approach 3: Try reading skill_data_array field directly
    let field_arr = read_field_value(chara_class, chara_obj, "skill_data_array");
    if !field_arr.is_null() {
        let ab = field_arr as *const u8;
        let al = std::ptr::read_unaligned::<usize>(ab.add(IL2CPP_LIST_COUNT_OFF) as *const usize);
        if al > 0 && al < 500 {
            let skill_elem_class = find_class_by_short_name(image, "SingleModeSkillData");
            for i in 0..al {
                let ep = std::ptr::read_unaligned::<*mut c_void>(
                    ab.add(IL2CPP_LIST_ITEMS_OFF + i * IL2CPP_LIST_ITEM_SIZE) as *const *mut c_void,
                );
                if ep.is_null() {
                    continue;
                }
                let skill_id = if !skill_elem_class.is_null() {
                    call_getter_int(skill_elem_class, ep, "get_SkillId")
                } else {
                    std::ptr::read_unaligned::<i32>(
                        (ep as *const u8).add(IL2CPP_UNBOX_FIRST_FIELD) as *const i32
                    )
                };
                let level = if !skill_elem_class.is_null() {
                    call_getter_int(skill_elem_class, ep, "get_Level")
                } else {
                    1
                };
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
    let _ = conn
        .prepare("SELECT id, grade_value FROM skill_data")
        .map(|mut stmt| {
            let _ = stmt
                .query_map([], |row| {
                    Ok((
                        row.get::<_, i32>(0).unwrap_or(0),
                        row.get::<_, i32>(1).unwrap_or(0),
                    ))
                })
                .map(|rows| {
                    rows.filter_map(|r| r.ok()).for_each(|(id, gv)| {
                        grade_map.insert(id, gv);
                    });
                });
        });

    // Also get skill names
    let mut name_map: std::collections::HashMap<i32, String> = std::collections::HashMap::new();
    let _ = conn
        .prepare(&format!(
            "SELECT id, text FROM text_data WHERE category={}",
            TEXT_DATA_CATEGORY_SKILL_NAME
        ))
        .map(|mut stmt| {
            let _ = stmt
                .query_map([], |row| {
                    let text: String = row
                        .get::<_, Option<String>>(1)
                        .unwrap_or(None)
                        .unwrap_or_default();
                    Ok((row.get::<_, i32>(0).unwrap_or(0), text))
                })
                .map(|rows| {
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
        let name = name_map
            .get(&skill_id)
            .cloned()
            .unwrap_or_else(|| format!("id:{}", skill_id));
        breakdown.push(format!(
            r#"{{"id":{},"name":"{}","gv":{},"lv":{},"ev":{}}}"#,
            skill_id,
            json_escape(&name),
            grade_value,
            level,
            eval
        ));
    }

    (
        total_eval,
        skills.len() as i32,
        format!("[{}]", breakdown.join(",")),
    )
}

// ★ v3.22.51: Summary cache — reduce IL2CPP metadata reads
static CACHED_SUMMARY: std::sync::Mutex<Option<(String, u64)>> = std::sync::Mutex::new(None);
const SUMMARY_CACHE_TTL_SECS: u64 = 3;

fn read_summary() -> String {
    // ★ v3.22.35: SIGSEGV cooldown — if we recently recovered from a crash, skip reads
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let cooldown = SIGSEGV_COOLDOWN_UNTIL.load(std::sync::atomic::Ordering::Relaxed);
    if now < cooldown {
        return format!(
            r#"{{"error":"sigsegv_cooldown","retry_after":{}}}"#,
            cooldown - now
        );
    }
    // ★ v3.22.51: Check cache first — avoid IL2CPP calls if data hasn't changed
    if let Ok(guard) = CACHED_SUMMARY.lock() {
        if let Some((ref cached, ts)) = *guard {
            if now.saturating_sub(ts) < SUMMARY_CACHE_TTL_SECS {
                return cached.clone();
            }
        }
    }
    // ★ v3.15.2: Mutex lock prevents concurrent il2cpp reads from HTTP + push threads
    let _lock = READ_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    // ★ v3.22.35: sigsetjmp recovery — catch SIGSEGV from il2cpp_runtime_invoke
    // If SIGSEGV fires during read_summary_inner, signal handler will longjmp back here
    let jmp_result = unsafe { sys_sigsetjmp(SIGSEGV_JMP_BUF.as_mut_ptr(), 1) };
    if jmp_result != 0 {
        // We jumped back from SIGSEGV handler — read_summary_inner crashed
        unsafe {
            ura_log(1, "★ SIGSEGV recovered in read_summary — skipping for 60s");
        };
        let err =
            r#"{"error":"sigsegv_recovered","hint":"read_summary hit native crash, cooling down"}"#
                .to_string();
        if let Ok(mut guard) = CACHED_SUMMARY.lock() {
            *guard = Some((err.clone(), now));
        }
        return err;
    }
    // Set recovery flag so signal handler knows to longjmp instead of killing process
    SIGSEGV_RECOVERY.store(true, std::sync::atomic::Ordering::Relaxed);
    let summary = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
        read_summary_inner()
    }))
    .unwrap_or_else(|_| {
        r#"{"error":"panic_caught","hint":"read_summary panicked, game protected"}"#.to_string()
    });
    // Clear recovery flag — normal return, no crash
    SIGSEGV_RECOVERY.store(false, std::sync::atomic::Ordering::Relaxed);
    // ★ v3.22.51: Update cache
    if let Ok(mut guard) = CACHED_SUMMARY.lock() {
        *guard = Some((summary.clone(), now));
    }
    summary
}

unsafe fn read_summary_inner() -> String {
    // v3.22.51: IN_READ_PATH disabled - /debug/ramenfields proves IL2CPP APIs are safe from HTTP thread
    // Keep the wrapper for potential future use, but don't block any APIs
    read_summary_inner_impl()
}

/// ★ INVOKE INVENTORY — read_summary_inner_impl 热路径所有 il2cpp_runtime_invoke 调用点
///
/// 每次 call_getter_* 都是一次 il2cpp_runtime_invoke（非主线程）。
/// 安全阈值 ~130 次/次调用。加新功能前先数总数，不要重复调同一个 getter。
///
/// ┌────────┬──────────────────────────────────────┬───────────────────────────────┐
/// │ 行号   │ 调用                                 │ 备注                          │
/// ├────────┼──────────────────────────────────────┼───────────────────────────────┤
/// │ 4108   │ get_SingleMode (wdm) [INVOKE-01]     │ 唯一调用                      │
/// │ 4119   │ get_Character (sm) [INVOKE-02]       │ 唯一调用                      │
/// │ 4146   │ get_Month (sm) [INVOKE-03]           │ 唯一调用                      │
/// │ 4151   │ get_Half (sm) [INVOKE-04]            │ 唯一调用                      │
/// │ 4274   │ get_DataSet (scenario) [INVOKE-05]   │ ★ 与 INVOKE-09 重复，待去重   │
/// │ 4695   │ get_EquipSupportCardArray [INVOKE-06]│ ★ 结果复用到 support cards 段 │
/// │ 4763   │ get_HomeInfoData (sm) [INVOKE-07]    │ 唯一调用                      │
/// │ 5194   │ get_TrainingLevelInfoArray [INV-08]  │ 唯一调用                      │
/// │ 5282   │ get_DataSet (scenario) [INVOKE-09]   │ ★ 与 INVOKE-05 重复，待去重   │
/// │ 5294   │ get_EnhanceGroupArray [INVOKE-10]    │ 唯一调用                      │
/// │ 5316   │ get_GainTotal (obscured) [INVOKE-11] │ 循环内，按 enhance 数量倍增   │
/// │ 5321   │ get_Level (obscured) [INVOKE-12]     │ 循环内，按 enhance 数量倍增   │
/// └────────┴──────────────────────────────────────┴───────────────────────────────┘
///
/// 历史：v3.24.3 因 invoke 过多 SIGSEGV；v3.24.5 削减后稳定；
///       v3.24.12 加 shining 检测新增一次 get_EquipSupportCardArray → 回归；
///       v3.24.13 复用 shining 结果消除重复调用 → 修复。
/// 维护规则：新增 call_getter_* 前，先在本表登记行号和是否重复。
unsafe fn read_summary_inner_impl() -> String {
    if API.is_null() {
        return r#"{"error":"api_null"}"#.to_string();
    }
    let image = match get_image() {
        img if !img.is_null() => img,
        _ => return r#"{"error":"image_null"}"#.to_string(),
    };

    // --- Chara stats ---
    ura_log(3, "★ read_summary phase1: chara stats");
    let wdm_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkDataManager").as_ptr(),
    );
    if wdm_class.is_null() {
        return r#"{"error":"no_wdm"}"#.to_string();
    }
    let wdm_inst = get_singleton(wdm_class);
    if wdm_inst.is_null() {
        return r#"{"error":"no_wdm_inst"}"#.to_string();
    }
    log_predict_step("S:wdm");

    log_predict_step("S:before_sm_class");
    let sm_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkSingleModeData").as_ptr(),
    );
    log_predict_step("S:after_sm_class");
    if sm_class.is_null() {
        return r#"{"error":"no_sm_class"}"#.to_string();
    }

    log_predict_step("S:before_get_single_mode");
    let sm_obj = call_getter_ref(wdm_class, wdm_inst, "get_SingleMode"); // [INVOKE-01]
    log_predict_step("S:after_get_single_mode");
    if sm_obj.is_null() {
        return r#"{"error":"no_sm"}"#.to_string();
    }

    log_predict_step("S:before_chara_class");
    let chara_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkSingleModeCharaData").as_ptr(),
    );
    log_predict_step("S:after_chara_class");
    if chara_class.is_null() {
        return r#"{"error":"no_chara_class"}"#.to_string();
    }

    log_predict_step("S:before_get_character");
    let chara_obj = call_getter_ref(sm_class, sm_obj, "get_Character"); // [INVOKE-02] get_Character — 唯一调用
    log_predict_step("S:after_get_character");
    if chara_obj.is_null() {
        return r#"{"error":"no_chara"}"#.to_string();
    }

    log_predict_step("S:before_read_speed");
    let spd = read_obscured_int_at(chara_obj, 248); // _speed
    log_predict_step("S:after_read_speed");
    let sta = read_obscured_int_at(chara_obj, 268); // _stamina
    let pow_ = read_obscured_int_at(chara_obj, 288); // _power
    let gut = read_obscured_int_at(chara_obj, 308); // _guts
    let wiz = read_obscured_int_at(chara_obj, 328); // _wiz
    let vit = read_obscured_int_at(chara_obj, 208); // _hp
    let mvit = read_obscured_int_at(chara_obj, 228); // _maxHp
    let mot = read_obscured_int_at(chara_obj, 1056); // _motivation
    let spt = read_obscured_int_at(chara_obj, 704); // _skillPoint
    let fan = read_obscured_int_at(chara_obj, 996); // _fanCount
                                                    // ★ v3.24.9: Month/Half are computed properties — must use getter (no offset available)
    let mon = if !sm_class.is_null() {
        call_getter_int(sm_class, sm_obj, "get_Month")
    } else {
        1
    }; // [INVOKE-03] get_Month — 唯一调用
    let half = if !sm_class.is_null() {
        call_getter_int(sm_class, sm_obj, "get_Half")
    } else {
        1
    }; // [INVOKE-04] get_Half — 唯一调用
    // ★ v2.2: Read year and compute cumulative turn
    // WorkSingleModeData._totalTurnNum at offset 68 (confirmed by /debug/dumpclass)
    let total_turn_num = read_int_at(sm_obj as *const c_void, 68); // _totalTurnNum
    // Year from totalTurnNum: year 1 = turn 1-18 (month 4-12), year 2 = turn 19-42, year 3 = turn 43-66
    let year = if total_turn_num > 0 {
        if total_turn_num <= 18 { 1 }
        else if total_turn_num <= 42 { 2 }
        else if total_turn_num <= 66 { 3 }
        else { 4 }
    } else {
        // Fallback: estimate from month (less reliable)
        if mon >= 4 { 1 } else { 2 }
    };
    let cumulative_turn = if total_turn_num > 0 {
        total_turn_num
    } else {
        (year - 1) * 24 + (mon - 1) * 2 + half
    };
    let sid = read_obscured_int_at(chara_obj, 568); // _scenarioId
    let chara_id = read_obscured_int_at(chara_obj, 36); // _cardId

    // ★ v3.24.9: New fields — attribute caps + scenario progress + running style
    let max_spd = read_obscured_int_at(chara_obj, 348); // MaxSpeed
    let max_sta = read_obscured_int_at(chara_obj, 368); // MaxStamina
    let max_pow = read_obscured_int_at(chara_obj, 388); // MaxPower
    let max_gut = read_obscured_int_at(chara_obj, 408); // MaxGuts
    let max_wiz = read_obscured_int_at(chara_obj, 428); // MaxWiz
    let scenario_progress = read_obscured_int_at(chara_obj, 1116); // ScenarioProgress
    let running_style = read_obscured_int_at(chara_obj, 944); // RunningStyle
    let training_event_type = read_obscured_int_at(chara_obj, 672); // TrainingEventType

    // ★ v3.24.9: Static info (read every time, but rarely changes)
    let talent_level = read_obscured_int_at(chara_obj, 88); // TalentLevel
    let limit_break = read_obscured_int_at(chara_obj, 108); // LimitBreakCount
    let chara_grade = read_obscured_int_at(chara_obj, 168); // CharaGrade
    let difficulty = read_obscured_int_at(chara_obj, 608); // Difficulty

    // ★ v3.24.9: Proper (适性) — A=6,B=5,C=4,D=3,E=2,F=1,G=0
    let proper_dist_short = read_obscured_int_at(chara_obj, 744);
    let proper_dist_mile = read_obscured_int_at(chara_obj, 764);
    let proper_dist_mid = read_obscured_int_at(chara_obj, 784);
    let proper_dist_long = read_obscured_int_at(chara_obj, 804);
    let proper_ground_turf = read_obscured_int_at(chara_obj, 904);
    let proper_ground_dirt = read_obscured_int_at(chara_obj, 924);

    // Runtime reflection confirms offset 0x198 is ObscuredInt _fixedTurnCharaSeed.
    // This is a named game field, not a complete PRNG state.
    let fixed_turn_chara_seed = if !sm_obj.is_null() {
        read_obscured_int_at(sm_obj, 408)
    } else {
        0
    };
    let chara_effect_ids_arr = read_ptr_at(chara_obj, 1080); // _charaEffectIdArray
    let chara_effect_ids: Vec<i32> = if !chara_effect_ids_arr.is_null() {
        read_il2cpp_int_list(chara_effect_ids_arr)
    } else {
        Vec::new()
    };
    let effect_ids_str: Vec<String> = chara_effect_ids.iter().map(|x| x.to_string()).collect();
    log_predict_step(&format!("S:stats sid={}", sid));

    // ★ v3.22.0: Read learned skills and compute skill evaluation
    ura_log(3, "★ read_summary phase1b: skill eval");
    let (skill_eval, skill_count, skills_json) = {
        let learned_skills = read_chara_skills(chara_class, chara_obj, image);
        compute_skill_eval(&learned_skills)
    };
    ura_log(
        2,
        &format!("skill_eval={} count={}", skill_eval, skill_count),
    );
    log_predict_step("S:skills");

    let mot_s = match mot {
        5 => "Best",
        4 => "Good",
        3 => "Normal",
        2 => "Bad",
        1 => "Worst",
        _ => "?",
    };
    let scn_s = match sid {
        1 => "URA",
        2 => "TeamRace",
        3 => "Live",
        4 => "Free",
        5 => "Venus",
        6 => "Arc",
        7 => "Sport",
        8 => "Cook",
        9 => "Mecha",
        10 => "Legend",
        11 => "Pioneer",
        12 => "Onsen",
        13 => "Breeders",
        14 => "Ramen",
        _ => "Unknown",
    };

    // ★ v3.18.2: Pre-read Ramen CommandInfoArray gains (scenario_id == 14)
    // HomeInfoData.ParamsIncDecInfoArray is empty for Ramen scenario.
    // Real gains are in WorkSingleModeScenarioRamenDataSet.CommandInfoArray
    // → ObscuredSingleModeRamenCommandInfo.ParamsIncDecInfoArray
    // Uses same plain Int32 format as Breeders: SingleModeParamsIncDecInfo at 0x10, 0x14
    let mut ramen_gains_map: std::collections::HashMap<i32, String> =
        std::collections::HashMap::new();
    let mut ramen_stat_gains_map: std::collections::HashMap<i32, [i32; 5]> =
        std::collections::HashMap::new();
    let mut ramen_skill_pt_map: std::collections::HashMap<i32, i32> =
        std::collections::HashMap::new();
    let mut ramen_vital_cost_map: std::collections::HashMap<i32, i32> =
        std::collections::HashMap::new();
    let mut ramen_gauge_gains_map: std::collections::HashMap<i32, i32> =
        std::collections::HashMap::new();
    // ★ v3.18.4: Ramen scenario-specific data for /summary
    let mut ramen_checkpoint_pt: i32 = -1;
    let mut ramen_special_feeling_num: i32 = -1;
    let mut ramen_recommend_type: i32 = -1;
    let mut ramen_feeling_info_json = String::new();
    // ★ v3.22.39: Aggregate sozai counts while reading FeelingInfo
    let mut ramen_sozai_counts: [i32; 3] = [0, 0, 0]; // [麺=1, スープ=2, トッピング=3]
    let mut ramen_selected_region_ids_json = String::new();
    let mut ramen_active_effects_raw_json = String::new();
    let mut ramen_uraf_type: i32 = -1;
    let mut ramen_uraf_state: i32 = -1;
    // ★ v3.22.89: Gauge gains per training command (from DataSet CommandInfoArray, target_type=30)
    let mut ramen_gauge_gains_json = String::new();
    // ★ v3.22.51: Ramen direct memory read — only 2 il2cpp_runtime_invoke calls
    // (try_get_scenario_obj + get_DataSet), then zero il2cpp calls
    if sid == 14 {
        ura_log(3, "v3.22.51 ramen: direct memory read");
        log_predict_step("S:ramen start");
        log_predict_step("S:ramen dataset before scenario");
        let scenario_obj = try_get_scenario_obj(chara_class, chara_obj, 14);
        if !scenario_obj.is_null() {
            let sc_class =
                std::ptr::read_unaligned::<*mut c_void>(scenario_obj as *const *mut c_void);
            log_predict_step("S:ramen sc_obj");
            log_predict_step("S:ramen dataset before getter");
            let dataset_obj = call_getter_ref(sc_class, scenario_obj, "get_DataSet"); // [INVOKE-05] get_DataSet (Ramen) — ★ 与 INVOKE-09 重复，待去重
            log_predict_step("S:ramen dataset after getter");
            if !dataset_obj.is_null() {
                let ds_class =
                    std::ptr::read_unaligned::<*mut c_void>(dataset_obj as *const *mut c_void);
                // Read 5 scalar ObscuredInt fields (zero il2cpp calls)
                let (cp_pt, sf_num, rec_type, uraf_t, uraf_s) =
                    read_ramen_scalar_fields(ds_class, dataset_obj);
                log_predict_step("S:ramen ds");
                ramen_checkpoint_pt = cp_pt;
                ramen_special_feeling_num = sf_num;
                ramen_recommend_type = rec_type;
                ramen_uraf_type = uraf_t;
                ramen_uraf_state = uraf_s;
                ura_log(
                    3,
                    &format!(
                        "ramen scalar: cp={} sf={} rec={} uraf_t={} uraf_s={}",
                        cp_pt, sf_num, rec_type, uraf_t, uraf_s
                    ),
                );
                log_predict_step("S:ramen dataset scalars done");
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
                log_predict_step("S:ramen regions done");
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
                                lb.add(IL2CPP_LIST_ITEMS_OFF) as *const *mut c_void,
                            );
                            if !first_elem.is_null() {
                                let elem_class = std::ptr::read_unaligned::<*mut c_void>(
                                    first_elem as *const *mut c_void,
                                );
                                let cat_off =
                                    cached_find_field_offset(elem_class, "EffectCategory");
                                let eid_off = cached_find_field_offset(elem_class, "EffectId");
                                let val_off = cached_find_field_offset(elem_class, "EffectValue");
                                let mut effects: Vec<String> = Vec::new();
                                for i in 0..llen {
                                    let ep = std::ptr::read_unaligned::<*mut c_void>(
                                        lb.add(IL2CPP_LIST_ITEMS_OFF + i * IL2CPP_LIST_ITEM_SIZE)
                                            as *const *mut c_void,
                                    );
                                    if ep.is_null() {
                                        continue;
                                    }
                                    let cat = if cat_off >= 0 {
                                        read_obscured_int_at(ep, cat_off)
                                    } else {
                                        -1
                                    };
                                    let eid = if eid_off >= 0 {
                                        read_obscured_int_at(ep, eid_off)
                                    } else {
                                        -1
                                    };
                                    let val = if val_off >= 0 {
                                        read_obscured_int_at(ep, val_off)
                                    } else {
                                        -1
                                    };
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
                log_predict_step("S:ramen effects done");
                // ★ v3.22.39: CommandFeelingInfoArray — dump element class name + gauge data
                // Skip in /summary for now, use /debug/gauge for safe testing
                // TODO: re-enable after /debug/gauge confirms element type and GetGainCount works
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
                                lb.add(IL2CPP_LIST_ITEMS_OFF) as *const *mut c_void,
                            );
                            if !first_elem.is_null() {
                                let elem_class = std::ptr::read_unaligned::<*mut c_void>(
                                    first_elem as *const *mut c_void,
                                );
                                let ft_off = cached_find_field_offset(elem_class, "FeelingIndex");
                                let fv_off = cached_find_field_offset(elem_class, "FeelingId");
                                let mut feelings: Vec<String> = Vec::new();
                                for i in 0..llen {
                                    let ep = std::ptr::read_unaligned::<*mut c_void>(
                                        lb.add(IL2CPP_LIST_ITEMS_OFF + i * IL2CPP_LIST_ITEM_SIZE)
                                            as *const *mut c_void,
                                    );
                                    if ep.is_null() {
                                        continue;
                                    }
                                    let ft = if ft_off >= 0 {
                                        read_obscured_int_at(ep, ft_off)
                                    } else {
                                        -1
                                    };
                                    let fv = if fv_off >= 0 {
                                        read_obscured_int_at(ep, fv_off)
                                    } else {
                                        -1
                                    };
                                    // ★ v3.22.39: Count sozai by FeelingId (1=麺, 2=スープ, 3=トッピング)
                                    if fv >= 1 && fv <= 3 {
                                        ramen_sozai_counts[(fv - 1) as usize] += 1;
                                    }
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
                // ★ v3.22.52: Read CommandInfoArray from DataSet for Ramen gains
                // HomeInfoData.ParamsIncDecInfoArray is empty for Ramen,
                // real gains are in DataSet.CommandInfoArray[].ParamsIncDecInfoArray
                // Same direct memory read as /debug/paramsincdec
                // ★ v3.24.9: Reverted to read_ptr_at — call_getter_ref caused crash during loading
                // The offset 16 is confirmed correct by /debug/dumpclass
                // Original code worked in v3.24.2, crash was introduced by getter call
                log_predict_step("S:ramen feelings done");
                let cmd_list = read_ptr_at(dataset_obj, RAMEN_DATASET_CMD_ARRAY_OFF as i32);
                if !cmd_list.is_null() {
                    let cmd_lb = cmd_list as *const u8;
                    let cmd_count = std::ptr::read_unaligned::<usize>(
                        cmd_lb.add(IL2CPP_LIST_COUNT_OFF) as *const usize,
                    );
                    if cmd_count > 0 && cmd_count < 100 {
                        // ObscuredSingleModeRamenCommandInfo field offsets (confirmed by /debug/dumpclass):
                        //   0x10 (16): CommandType (ObscuredInt, 20 bytes inline)
                        //   0x24 (36): CommandId (ObscuredInt, 20 bytes inline)
                        //   0x38 (56): ParamsIncDecInfoArray (List ptr)
                        // read_obscured_int_at reads key^hidden at the given offset
                        for ci in 0..cmd_count {
                            let ce = std::ptr::read_unaligned::<*mut c_void>(
                                cmd_lb.add(IL2CPP_LIST_ITEMS_OFF + ci * IL2CPP_LIST_ITEM_SIZE)
                                    as *const *mut c_void,
                            );
                            if ce.is_null() {
                                continue;
                            }
                            let cmd_id = read_obscured_int_at(ce, RAMEN_CMD_COMMAND_ID_OFF as i32);
                            let ce_params = read_ptr_at(ce, RAMEN_CMD_PARAMS_ARRAY_OFF as i32);
                            if cmd_id < 0 || ce_params.is_null() {
                                continue;
                            }
                            let ce_plb = ce_params as *const u8;
                            let ce_plen = std::ptr::read_unaligned::<usize>(
                                ce_plb.add(IL2CPP_LIST_COUNT_OFF) as *const usize,
                            );
                            if ce_plen == 0 || ce_plen > 1000 {
                                continue;
                            }
                            let mut gains_parts: Vec<String> = Vec::new();
                            let mut sg = [0i32; 5]; // [Speed, Stamina, Power, Guts, Wisdom]
                            let mut spt = 0i32;
                            let mut vc = 0i32;
                            // ★ v3.22.89: Merged gauge_gain into single loop (was separate redundant loop)
                            let mut gauge_gain = 0i32;
                            // ★ Confirmed by /debug/params: elements are SingleModeParamsIncDecInfo (plain int32)
                            // NOT SingleModeParamsIncDecInfoData (ObscuredInt)
                            // Plain int32 read at 0x10/0x14 gives correct values
                            for pi in 0..ce_plen {
                                let pe = std::ptr::read_unaligned::<*mut c_void>(
                                    ce_plb.add(IL2CPP_LIST_ITEMS_OFF + pi * IL2CPP_LIST_ITEM_SIZE)
                                        as *const *mut c_void,
                                );
                                if pe.is_null() {
                                    continue;
                                }
                                let tt = std::ptr::read_unaligned::<i32>(
                                    (pe as *const u8).add(PARAMS_INCDEC_TARGET_TYPE_OFF)
                                        as *const i32,
                                );
                                let vv = std::ptr::read_unaligned::<i32>(
                                    (pe as *const u8).add(PARAMS_INCDEC_VALUE_OFF) as *const i32,
                                );
                                if tt == 30 {
                                    gauge_gain += vv;
                                }
                                if vv == 0 {
                                    continue;
                                }
                                let tn = match tt {
                                    1 => "Speed",
                                    2 => "Stamina",
                                    3 => "Power",
                                    4 => "Guts",
                                    5 => "Wiz",
                                    10 => "HP",
                                    20 => "Motivation",
                                    30 => "Gauge",
                                    40 => "SkillPt",
                                    _ => "Unknown",
                                };
                                gains_parts.push(format!(r#""{}":{}"#, tn, vv));
                                match tt {
                                    1 => sg[0] += vv,
                                    2 => sg[1] += vv,
                                    4 => sg[2] += vv,
                                    3 => sg[3] += vv,
                                    5 => sg[4] += vv,
                                    10 => vc += vv,
                                    40 => spt += vv,
                                    _ => {}
                                }
                            }
                            if !gains_parts.is_empty() {
                                // ★ FIX: Store under both cmd_id variants (601→101, 602→102, etc.)
                                // so lookup works regardless of which command_id space HomeInfoData uses
                                ramen_gains_map.insert(cmd_id, gains_parts.join(","));
                                ramen_stat_gains_map.insert(cmd_id, sg);
                                ramen_skill_pt_map.insert(cmd_id, spt);
                                ramen_vital_cost_map.insert(cmd_id, vc);
                                let alt_id = match cmd_id {
                                    601 => Some(101),
                                    602 => Some(105),
                                    603 => Some(102),
                                    604 => Some(103),
                                    605 => Some(106),
                                    101 => Some(601),
                                    102 => Some(603),
                                    103 => Some(604),
                                    105 => Some(602),
                                    106 => Some(605),
                                    _ => None,
                                };
                                if let Some(aid) = alt_id {
                                    ramen_gains_map
                                        .insert(aid, ramen_gains_map.get(&cmd_id).unwrap().clone());
                                    ramen_stat_gains_map.insert(aid, sg);
                                    ramen_skill_pt_map.insert(aid, spt);
                                    ramen_vital_cost_map.insert(aid, vc);
                                }
                                ura_log(
                                    4,
                                    &format!(
                                        "ramen gains: cmd_id={} gains={} alt={:?}",
                                        cmd_id,
                                        gains_parts.join(","),
                                        alt_id
                                    ),
                                );
                            }
                            if gauge_gain > 0 {
                                ramen_gauge_gains_map.insert(cmd_id, gauge_gain);
                            }
                        }
                    }
                }
                // ★ v3.22.89: Build gauge_gains JSON from ramen_gauge_gains_map
                log_predict_step("S:ramen commands done");
                if !ramen_gauge_gains_map.is_empty() {
                    let mut gg_parts: Vec<String> = Vec::new();
                    for (&cmd_id, &gauge_val) in &ramen_gauge_gains_map {
                        let cname = match cmd_id {
                            101 | 601 => "Speed",
                            102 | 603 => "Power",
                            103 | 604 => "Guts",
                            105 | 602 => "Stamina",
                            106 | 605 => "Wiz",
                            _ => "Unknown",
                        };
                        gg_parts.push(format!(
                            r#"{{"command_id":{},"name":"{}","gauge":{}}}"#,
                            cmd_id, cname, gauge_val
                        ));
                    }
                    ramen_gauge_gains_json = gg_parts.join(",");
                }
                ura_log(
                    3,
                    &format!(
                    "ramen arrays: regions={} effects={} feelings={} gains_map={} gauge_gains={}",
                    !ramen_selected_region_ids_json.is_empty(),
                    !ramen_active_effects_raw_json.is_empty(),
                    !ramen_feeling_info_json.is_empty(),
                    !ramen_gains_map.is_empty(),
                    !ramen_gauge_gains_map.is_empty()
                ),
                );
                log_predict_step("S:ramen arrays");
            } else {
                ura_log(2, "ramen: dataset_obj null");
            }
        } else {
            ura_log(2, "ramen: scenario_obj null");
        }
    }

    // Partner ID -> current bond.
    //
    // _evaluationList is List<Evaluation>, not Evaluation[]:
    //
    // List<Evaluation> + 0x10 = _items (Evaluation[])
    // List<Evaluation> + 0x18 = _size  (Int32)
    // Evaluation[]     + 0x20 = first element
    //
    // Each Evaluation object contains two inline ObscuredInt fields:
    //
    // Evaluation + 0x10 = partner ID
    // Evaluation + 0x24 = current bond
    let mut partner_evaluation: std::collections::HashMap<i32, i32> =
        std::collections::HashMap::new();

    let evaluation_list = read_ptr_at(chara_obj as *const c_void, EVALUATION_LIST_OFF);

    if !evaluation_list.is_null() {
        // List<T>._size is Int32 at +0x18. Do not read this as usize,
        // because +0x1c contains List<T>._version.
        let count = read_int_at(evaluation_list as *const c_void, 0x18);

        // List<T>._items is the backing T[] array at +0x10.
        let evaluation_items = read_ptr_at(evaluation_list as *const c_void, 0x10);

        if count > 0 && count < 1000 && !evaluation_items.is_null() {
            let items_base = evaluation_items as *const u8;

            for i in 0..count as usize {
                // Evaluation is a reference type, so the backing array
                // contains object pointers beginning at array + 0x20.
                let item = std::ptr::read_unaligned::<*mut c_void>(
                    items_base.add(IL2CPP_LIST_ITEMS_OFF + i * IL2CPP_LIST_ITEM_SIZE)
                        as *const *mut c_void,
                );

                if item.is_null() {
                    continue;
                }

                let partner_id =
                    read_obscured_int_at(item as *const c_void, EVALUATION_PARTNER_ID_OFF);

                let current_bond =
                    read_obscured_int_at(item as *const c_void, EVALUATION_VALUE_OFF);

                // Guard against corrupt or misread entries.
                if partner_id > 0
                    && partner_id < 100_000
                    && current_bond >= 0
                    && current_bond <= 100
                {
                    partner_evaluation.insert(partner_id, current_bond);
                }
            }

            ura_log(
                3,
                &format!(
                    "evaluation_list: size={}, decoded={} entries",
                    count,
                    partner_evaluation.len()
                ),
            );
        } else {
            ura_log(
                2,
                &format!(
                    "evaluation_list unavailable: size={}, items_null={}",
                    count,
                    evaluation_items.is_null()
                ),
            );
        }
    } else {
        ura_log(2, "evaluation_list is null");
    }

    // Runtime support-card position -> support_card_data.command_id.
    //
    // Position is read dynamically from the equipped-card object.
    // It is not assumed that a particular card is always in a fixed slot.
    let mut support_command_by_position: std::collections::HashMap<i32, i32> =
        std::collections::HashMap::new();
    /// ★ v3.24.14: position → bond_threshold from MasterDB unique_effect
    let mut bond_threshold_by_position: std::collections::HashMap<i32, i32> =
        std::collections::HashMap::new();
    /// ★ v3.24.15: position → support_card_type (1=普通, 2=友人, 3=团体)
    let mut support_card_type_by_position: std::collections::HashMap<i32, i32> =
        std::collections::HashMap::new();

    // First collect equipped (position, support_card_id) pairs.
    let mut equipped_support_cards: Vec<(i32, i32)> = Vec::new();

    log_predict_step("S:support equip before getter");
    let support_array_for_shining =
        call_getter_on_instance(chara_class, chara_obj, "get_EquipSupportCardArray"); // [INVOKE-06] get_EquipSupportCardArray — ★ 结果复用到 support cards 段
    log_predict_step("S:support equip after getter");

    if !support_array_for_shining.is_null() {
        let support_array_base = support_array_for_shining as *const u8;

        let support_count = std::ptr::read_unaligned::<usize>(
            support_array_base.add(IL2CPP_LIST_COUNT_OFF) as *const usize,
        );

        if support_count > 0 && support_count <= 6 {
            for i in 0..support_count {
                let support = std::ptr::read_unaligned::<*mut c_void>(
                    support_array_base.add(IL2CPP_LIST_ITEMS_OFF + i * IL2CPP_LIST_ITEM_SIZE)
                        as *const *mut c_void,
                );

                if support.is_null() {
                    continue;
                }

                // SingleModeEquipSupportCard:
                //   +0x10 Position      (inline ObscuredInt)
                //   +0x24 SupportCardId (inline ObscuredInt)
                let position = read_obscured_int_at(support as *const c_void, 0x10);

                let support_card_id = read_obscured_int_at(support as *const c_void, 0x24);

                if (1..=6).contains(&position) && support_card_id > 0 {
                    equipped_support_cards.push((position, support_card_id));
                }
            }
        }
    }

    // Resolve each ordinary card's training specialty from MasterDB:
    //
    // command_id=0 is intentionally retained as an unclassified special card.
    //
    // ★ v3.24.14: Also read bond_threshold from support_card_unique_effect.
    //   type_0=101 → value_0 = bond threshold for unique effect / friendship training.
    //   Cards without unique_effect_id get threshold = i32::MAX (never shines).
    log_predict_step("S:support mdb before");
    /// position → (command_id, bond_threshold, support_card_type)
    static SUPPORT_CARD_INFO_CACHE: std::sync::Mutex<
        Option<std::collections::HashMap<i32, (i32, i32, i32)>>,
    > = std::sync::Mutex::new(None);

    // Try cache first; rebuild if empty.
    let mut info_map: std::collections::HashMap<i32, (i32, i32, i32)> =
        SUPPORT_CARD_INFO_CACHE.lock().unwrap().clone().unwrap_or_default();

    if info_map.is_empty() {
        if let Some(mdb_path) = find_mdb_path() {
            if let Ok(connection) =
                Connection::open_with_flags(&mdb_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
            {
                // Query command_id, unique effect threshold, and support_card_type in one JOIN.
                // type_0=101 is the bond-threshold marker in support_card_unique_effect.
                if let Ok(mut statement) = connection.prepare(
                    "SELECT sc.id, sc.command_id, sc.support_card_type, \
                     COALESCE(ue.value_0, 999999) AS threshold \
                     FROM support_card_data sc \
                     LEFT JOIN support_card_unique_effect ue \
                       ON sc.unique_effect_id = ue.id AND ue.type_0 = 101 \
                     WHERE sc.id = ?1",
                ) {
                    for &(position, support_card_id) in &equipped_support_cards {
                        let result = statement.query_row([support_card_id], |row| {
                            Ok((
                                row.get::<_, i32>(0)?, // id
                                row.get::<_, i32>(1)?, // command_id
                                row.get::<_, i32>(2)?, // support_card_type
                                row.get::<_, i32>(3)?, // threshold
                            ))
                        });

                        if let Ok((_id, support_command_id, sc_type, threshold)) = result {
                            support_command_by_position.insert(position, support_command_id);
                            bond_threshold_by_position.insert(position, threshold);
                            support_card_type_by_position.insert(position, sc_type);
                            info_map.insert(support_card_id, (support_command_id, threshold, sc_type));
                        }
                    }
                }

                // Cache for next call.
                if !info_map.is_empty() {
                    *SUPPORT_CARD_INFO_CACHE.lock().unwrap() = Some(info_map.clone());
                }
            }
        }
    } else {
        // Cache hit — populate from cache without DB access.
        for &(position, support_card_id) in &equipped_support_cards {
            if let Some(&(cmd_id, threshold, sc_type)) = info_map.get(&support_card_id) {
                support_command_by_position.insert(position, cmd_id);
                bond_threshold_by_position.insert(position, threshold);
                support_card_type_by_position.insert(position, sc_type);
            }
        }
    }

    log_predict_step("S:support mdb done");
    // --- Training data via HomeInfoData (ALL scenarios) ---
    log_predict_step("S:ramen end");
    ura_log(3, "★ read_summary phase2: training data");
    log_predict_step("S:p2 training");
    let mut tr_json = "[]".to_string();
    // ★ v3.15.1: collect eval_trainings in same pass (eliminate dangerous double-read)
    let mut eval_trainings: Vec<(i32, [i32; 5], i32, i32, i32, i32, i32, i32)> = Vec::new();
    log_predict_step("S:homeinfo before getter");
    let home_info_obj = call_getter_on_instance(sm_class, sm_obj, "get_HomeInfoData"); // [INVOKE-07] get_HomeInfoData — 唯一调用
    log_predict_step("S:homeinfo after getter");
    if !home_info_obj.is_null() {
        let hi_class = find_class(
            image,
            to_cstr("Gallop").as_ptr(),
            to_cstr("WorkSingleModeHomeInfoData").as_ptr(),
        );
        if !hi_class.is_null() {
            // CommandInfoArray is a public field (not a getter), at offset 0x10
            log_predict_step("S:homeinfo commands before");
            let cmd_arr = read_field_value(hi_class, home_info_obj, "CommandInfoArray");
            log_predict_step("S:homeinfo commands after");
            if !cmd_arr.is_null() {
                let cmd_base = cmd_arr as *const u8;
                let cmd_len = std::ptr::read_unaligned::<usize>(
                    cmd_base.add(IL2CPP_LIST_COUNT_OFF) as *const usize,
                );
                if cmd_len > 0 && cmd_len < 100 {
                    let mut trs = Vec::new();
                    for i in 0..cmd_len {
                        let ep = std::ptr::read_unaligned::<*mut c_void>(
                            cmd_base.add(IL2CPP_LIST_ITEMS_OFF + i * IL2CPP_LIST_ITEM_SIZE)
                                as *const *mut c_void,
                        );
                        if ep.is_null() {
                            continue;
                        }

                        // ★ v3.24.9: Direct memory read — zero il2cpp_runtime_invoke
                        // SingleModeCommandInfoData offsets (confirmed by /debug/dumpclass):
                        //   CommandType=16  CommandId=36  IsEnable=56
                        //   TrainingPartnerArray=80  TipsEventPartnerArray=88  FailureRate=104
                        let cid = read_obscured_int_at(ep as *const c_void, 36); // CommandId
                        let cname = match cid {
                            CMD_SPEED => "Speed",
                            CMD_STAMINA => "Stamina",
                            CMD_GUTS => "Guts",
                            CMD_POWER => "Power",
                            CMD_WISDOM => "Wiz",
                            CMD_URA_SPEED => "Speed",
                            CMD_URA_STAMINA => "Stamina",
                            CMD_URA_GUTS => "Guts",
                            CMD_URA_POWER => "Power",
                            CMD_URA_WISDOM => "Wiz",
                            CMD_KAKUSHIMI => "Kakushimi",
                            301 => "Outing",
                            390 => "Rest",
                            401 => "Outing2",
                            701 => "Outing3",
                            801 => "Outing4",
                            _ => "Unknown",
                        };
                        let is_enable = read_obscured_int_at(ep as *const c_void, 56); // IsEnable
                        let failure_rate = read_obscured_int_at(ep as *const c_void, 104); // FailureRate

                        // TrainingPartnerArray is ObscuredInt[] with
                        // inline 20-byte values.
                        let tp_arr = read_ptr_at(ep as *const c_void, TRAINING_PARTNER_ARRAY_OFF);

                        // ★ v3.24.15: Read TipsEventPartnerArray for group card shining.
                        // Group cards (support_card_type=3) shine when they trigger a
                        // special tips event, not based on bond threshold.
                        // TipsEventPartnerArray is at offset 0x58 (88), same ObscuredInt[] format.
                        let tips_arr = read_ptr_at(ep as *const c_void, TIPS_EVENT_PARTNER_ARRAY_OFF);
                        let mut tips_partner_ids: std::collections::HashSet<i32> =
                            std::collections::HashSet::new();
                        if !tips_arr.is_null() {
                            let tips_base = tips_arr as *const u8;
                            let tips_len = std::ptr::read_unaligned::<usize>(
                                tips_base.add(IL2CPP_LIST_COUNT_OFF) as *const usize,
                            );
                            if tips_len <= 100 {
                                for ti in 0..tips_len {
                                    let tval = tips_base
                                        .add(IL2CPP_LIST_ITEMS_OFF + ti * OBSCURED_INT_SIZE);
                                    let tips_id =
                                        read_obscured_int_at(tval as *const c_void, 0);
                                    if tips_id > 0 {
                                        tips_partner_ids.insert(tips_id);
                                    }
                                }
                            }
                        }

                        let mut partner_ids: Vec<i32> = Vec::new();
                        let mut partners_json: Vec<String> = Vec::new();

                        // Number of confirmed shining support cards.
                        let mut shining_count = 0i32;

                        // This remains true when every present support
                        // partner can be classified conclusively.
                        //
                        // NPC/scenario partners do not affect completeness.
                        let mut shining_complete = true;

                        if !tp_arr.is_null() {
                            let array_base = tp_arr as *const u8;

                            let partner_count = std::ptr::read_unaligned::<usize>(
                                array_base.add(IL2CPP_LIST_COUNT_OFF) as *const usize,
                            );

                            if partner_count <= 100 {
                                for pi in 0..partner_count {
                                    let value = array_base
                                        .add(IL2CPP_LIST_ITEMS_OFF + pi * OBSCURED_INT_SIZE);

                                    let partner_id =
                                        read_obscured_int_at(value as *const c_void, 0);

                                    if partner_id <= 0 {
                                        continue;
                                    }

                                    partner_ids.push(partner_id);

                                    let current_bond = partner_evaluation.get(&partner_id).copied();

                                    // Classify support cards only through the actual equipped
                                    // position map. A numeric partner_id range alone is not proof.
                                    let support_card_id = equipped_support_cards
                                        .iter()
                                        .find(|&&(position, _)| position == partner_id)
                                        .map(|&(_, card_id)| card_id);
                                    let is_support_card = support_card_id.is_some();

                                    let support_position = if is_support_card {
                                        partner_id.to_string()
                                    } else {
                                        "null".to_string()
                                    };
                                    let support_card_id_json = support_card_id
                                        .map(|value| value.to_string())
                                        .unwrap_or_else(|| "null".to_string());

                                    let bond_json = current_bond
                                        .map(|value| value.to_string())
                                        .unwrap_or_else(|| "null".to_string());

                                    // ★ v3.24.15: Card-type-aware shining logic.
                                    //
                                    //   support_card_type=1 (普通卡): bond >= threshold && training match
                                    //   support_card_type=2 (友人卡): always false (友人卡不彩圈)
                                    //   support_card_type=3 (团体卡): partner_id in TipsEventPartnerArray
                                    //     (触发特殊启示事件就彩圈，不管 bond)
                                    //   Unknown type: null (conservative)
                                    let sc_type = support_card_type_by_position
                                        .get(&partner_id)
                                        .copied();

                                    let bond_threshold = bond_threshold_by_position
                                        .get(&partner_id)
                                        .copied()
                                        .unwrap_or(999999);

                                    let is_shining: Option<bool> = if is_support_card {
                                        match sc_type {
                                            // 普通卡: bond >= threshold && training match
                                            Some(1) => {
                                                match (
                                                    current_bond,
                                                    support_command_by_position.get(&partner_id).copied(),
                                                    normalize_training_command_id(cid),
                                                ) {
                                                    (Some(bond), Some(support_command_id), Some(current_training)) => {
                                                        match support_card_command_id_to_training_id(support_command_id) {
                                                            Some(card_training) => Some(
                                                                bond >= bond_threshold
                                                                    && card_training == current_training,
                                                            ),
                                                            None => None,
                                                        }
                                                    }
                                                    _ => None,
                                                }
                                            }
                                            // 友人卡: 永远不彩圈
                                            Some(2) => Some(false),
                                            // 团体卡: 启示事件触发就彩圈
                                            Some(3) => Some(tips_partner_ids.contains(&partner_id)),
                                            // 未知类型: 保守 null
                                            _ => {
                                                // Fallback to old logic for untyped cards
                                                match (
                                                    current_bond,
                                                    support_command_by_position.get(&partner_id).copied(),
                                                    normalize_training_command_id(cid),
                                                ) {
                                                    (Some(bond), Some(support_command_id), Some(current_training)) => {
                                                        match support_card_command_id_to_training_id(support_command_id) {
                                                            Some(card_training) => Some(
                                                                bond >= bond_threshold
                                                                    && card_training == current_training,
                                                            ),
                                                            None => None,
                                                        }
                                                    }
                                                    _ => None,
                                                }
                                            }
                                        }
                                    } else {
                                        // NPC and scenario partners are not equipped support cards.
                                        None
                                    };

                                    // ★ v3.24.14: Unique effect active = bond >= threshold.
                                    //   Triggers on ANY training, not just得意训练.
                                    //   友人卡: threshold=60, 团体卡: threshold=80/100
                                    let is_unique_active: Option<bool> = if is_support_card {
                                        if sc_type == Some(2) {
                                            // 友人卡固有: bond >= 60
                                            current_bond.map(|bond| bond >= bond_threshold)
                                        } else if sc_type == Some(3) {
                                            // 团体卡固有: bond >= threshold (80 or 100)
                                            current_bond.map(|bond| bond >= bond_threshold)
                                        } else if sc_type == Some(1) {
                                            // 普通卡固有: bond >= threshold (80 or 100)
                                            current_bond.map(|bond| bond >= bond_threshold)
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    };

                                    if is_support_card && is_shining.is_none() {
                                        shining_complete = false;
                                    }

                                    if is_shining == Some(true) {
                                        shining_count += 1;
                                    }

                                    let is_shining_json = match is_shining {
                                        Some(true) => "true",
                                        Some(false) => "false",
                                        None => "null",
                                    };

                                    let is_unique_json = match is_unique_active {
                                        Some(true) => "true",
                                        Some(false) => "false",
                                        None => "null",
                                    };

                                    let sc_type_json = match sc_type {
                                        Some(t) => t.to_string(),
                                        None => "null".to_string(),
                                    };

                                    let is_tips_event = tips_partner_ids.contains(&partner_id);

                                    // ★ v2.3: partner_type 和 name（照 PC 版小黑板 personType 映射）
                                    // personType: 0=未加载, 1=友人卡, 2=普通支援卡, 3=NPC, 4=理事长, 5=记者, 6=不带卡佐岳
                                    let (partner_type, partner_name) = if is_support_card {
                                        let sc_type_val = sc_type.unwrap_or(1);
                                        let ptype = match sc_type_val {
                                            2 => 1, // 友人卡
                                            3 => 2, // 团体卡 → 当普通支援卡显示
                                            _ => 2,  // 普通支援卡
                                        };
                                        // 名称从 MDB 查（后续优化），暂时用位置
                                        let name = format!("支援位{}", partner_id);
                                        (ptype, name)
                                    } else {
                                        // NPC/理事长/记者 — 按常见 ID 范围判断
                                        // 暂时全部标为 NPC
                                        (0, format!("伙伴{}", partner_id))
                                    };

                                    partners_json.push(format!(
                                        r#"{{"partner_id":{},"support_position":{},"support_card_id":{},"current_bond":{},"is_shining":{},"is_unique_active":{},"bond_threshold":{},"support_card_type":{},"is_tips_event":{},"partner_type":{},"name":"{}","bond_gain":null}}"#,
                                        partner_id,
                                        support_position,
                                        support_card_id_json,
                                        bond_json,
                                        is_shining_json,
                                        is_unique_json,
                                        bond_threshold,
                                        sc_type_json,
                                        is_tips_event,
                                        partner_type,
                                        json_escape(&partner_name),
                                    ));
                                }
                            }
                        }

                        let heads = partner_ids.len() as i32;

                        // Training-level shining count:
                        //
                        //   >= 0: confirmed number of shining cards
                        //     -1: unknown because a present support card could
                        //         not be classified safely
                        //
                        // TipsEventPartnerArray is intentionally not used.
                        let is_attribute_training = normalize_training_command_id(cid).is_some();

                        let shining = if !is_attribute_training {
                            // Rest, outing and other non-training commands do not have
                            // an ordinary friendship-training count.
                            -1
                        } else if shining_complete {
                            shining_count
                        } else {
                            -1
                        };

                        let shining_json = if shining >= 0 {
                            shining.to_string()
                        } else {
                            "null".to_string()
                        };

                        // Training gains from HomeInfoData.
                        //
                        // Runtime capture confirmed that each
                        // SingleModeParamsIncDecInfoData object contains:
                        //
                        //   +0x10 TargetType (inline ObscuredInt)
                        //   +0x24 Value      (inline ObscuredInt)
                        //
                        // The array itself is an IL2CPP reference array.
                        let mut gains = Vec::new();
                        let mut stat_gains = [0i32; 5];
                        let mut skill_pt_gain = 0i32;
                        let mut vital_cost = 0i32;

                        let params_array = read_ptr_at(ep as *const c_void, 96);

                        if !params_array.is_null() {
                            let array_base = params_array as *const u8;

                            let params_len = std::ptr::read_unaligned::<usize>(
                                array_base.add(IL2CPP_LIST_COUNT_OFF) as *const usize,
                            );

                            if params_len > 0 && params_len < 100 {
                                for j in 0..params_len {
                                    let param = std::ptr::read_unaligned::<*mut c_void>(
                                        array_base
                                            .add(IL2CPP_LIST_ITEMS_OFF + j * IL2CPP_LIST_ITEM_SIZE)
                                            as *const *mut c_void,
                                    );

                                    if param.is_null() {
                                        continue;
                                    }

                                    let target_type = read_obscured_int_at(
                                        param as *const c_void,
                                        PARAMS_INCDEC_DATA_TARGET_TYPE_OFF,
                                    );

                                    let value = read_obscured_int_at(
                                        param as *const c_void,
                                        PARAMS_INCDEC_DATA_VALUE_OFF,
                                    );

                                    if value == 0 {
                                        continue;
                                    }

                                    let target_name = match target_type {
                                        1 => "Speed",
                                        2 => "Stamina",
                                        3 => "Power",
                                        4 => "Guts",
                                        5 => "Wiz",
                                        10 => "HP",
                                        20 => "Motivation",
                                        30 => "SkillPt",
                                        _ => "Unknown",
                                    };

                                    // Include the numeric type in unknown keys
                                    // so malformed/unrecognised entries cannot
                                    // produce duplicate "Unknown" JSON keys.
                                    if target_name == "Unknown" {
                                        gains.push(format!(
                                            r#""Unknown_{}":{}"#,
                                            target_type, value
                                        ));
                                    } else {
                                        gains.push(format!(r#""{}":{}"#, target_name, value));
                                    }

                                    match target_type {
                                        1 => stat_gains[0] += value,
                                        2 => stat_gains[1] += value,
                                        3 => stat_gains[3] += value,
                                        4 => stat_gains[2] += value,
                                        5 => stat_gains[4] += value,
                                        10 => vital_cost += value,
                                        30 => skill_pt_gain += value,
                                        _ => {}
                                    }
                                }
                            }
                        }

                        // ★ v3.24.10: Ramen gains 直接用 HomeInfoData 读到的值
                        // 诊断确认: DataSet.CommandInfoArray.ParamsIncDecInfoArray 为空
                        // HomeInfoData.ParamsIncDecInfoArray 有数据 (params_len=4)

                        trs.push(format!(
                            r#"{{"name":"{}","command_id":{},"is_enable":{},"failure_rate":{},"heads":{},"shining":{},"partner_ids":[{}],"partners":[{}],"gains":{{{}}}}}"#,
                            cname,
                            cid,
                            is_enable,
                            failure_rate,
                            heads,
                            shining_json,
                            partner_ids
                                .iter()
                                .map(|value| value.to_string())
                                .collect::<Vec<_>>()
                                .join(","),
                            partners_json.join(","),
                            gains.join(","),
                        ));

                        // ★ v3.15.1: collect eval training data in same pass
                        if cmd_id_to_train_idx(cid).is_some() {
                            eval_trainings.push((
                                cid,
                                stat_gains,
                                skill_pt_gain,
                                vital_cost,
                                failure_rate,
                                is_enable,
                                shining,
                                heads,
                            ));
                        }
                    }
                    tr_json = format!("[{}]", trs.join(","));
                }
            }
        }
    }

    log_predict_step("S:training partners done");
    // --- Support cards (graceful fallback) ---
    log_predict_step("S:p2 done");
    ura_log(3, "★ read_summary phase3: support cards");
    log_predict_step("S:p3 cards");
    let mut sc_json = "[]".to_string();
    // ★ v3.22.89: Fix support_cards — use get_EquipSupportCardArray getter
    // Root cause: field name is "EquipSupportCardArray" not "SupportCardArray"
    // v3.22.89's cached_find_field_offset("SupportCardArray") hit wrong field via substring match
    // Also: position/supportCardId/limitBreakCount are ObscuredInt, not plain int
    // ★ v3.24.13: Reuse the array already fetched for shining detection —
    // eliminates a duplicate il2cpp_runtime_invoke that caused SIGSEGV.
    let mut sc_arr: *mut c_void = support_array_for_shining;
    ura_log(3, &format!("sc: reused shining array ptr={}", !sc_arr.is_null()));
    // Method 2: direct field offset on chara_class (fallback)
    if sc_arr.is_null() {
        let sc_off = cached_find_field_offset(chara_class, "EquipSupportCardArray");
        if sc_off >= 0 {
            sc_arr = read_ptr_at(chara_obj as *const c_void, sc_off);
            ura_log(
                3,
                &format!("sc: offset={} ptr={}", sc_off, !sc_arr.is_null()),
            );
        }
    }
    // Parse the List<SingleModeEquipSupportCard>
    if !sc_arr.is_null() {
        let ab = sc_arr as *const u8;
        let al = std::ptr::read_unaligned::<usize>(ab.add(IL2CPP_LIST_COUNT_OFF) as *const usize);
        if al > 0 && al < 100 {
            let mut scs = Vec::new();
            for i in 0..al {
                let ep = std::ptr::read_unaligned::<*mut c_void>(
                    ab.add(IL2CPP_LIST_ITEMS_OFF + i * IL2CPP_LIST_ITEM_SIZE) as *const *mut c_void,
                );
                if ep.is_null() {
                    continue;
                }
                // ★ v3.24.9: Direct memory read for EquipSupportCard (zero invoke)
                // Offsets confirmed by /debug/dumpclass:
                //   Position=16  SupportCardId=36  LimitBreakCount=56  Exp=76  RentalType=136
                let sc_ep = ep as *const c_void;
                let position = read_obscured_int_at(sc_ep, 16);
                let support_card_id = read_obscured_int_at(sc_ep, 36);
                let limit_break_count = read_obscured_int_at(sc_ep, 56);
                let sc_exp = read_obscured_int_at(sc_ep, 76); // ★ 新增: 支援卡经验值
                let rental_type = read_obscured_int_at(sc_ep, 136);
                // TrainingPartnerState is not in EquipSupportCard fields — it's on a different object
                // Skip it (set to -1) to avoid invoke
                let training_partner_state = -1;
                // CharaId is a computed property. The app can resolve it
                // through support_card_id and card_db.json.
                let sc_chara_id = -1;

                // Runtime capture confirmed that support-card positions
                // 1..=6 are also the corresponding partner IDs.
                let kizuna = partner_evaluation.get(&position).copied().unwrap_or(-1);
                scs.push(format!(
                    r#"{{"position":{},"support_card_id":{},"limit_break_count":{},"training_partner_state":{},"chara_id":{},"kizuna":{},"exp":{},"rental_type":{}}}"#,
                    position, support_card_id, limit_break_count, training_partner_state, sc_chara_id, kizuna, sc_exp, rental_type
                ));
            }
            sc_json = format!("[{}]", scs.join(","));
            ura_log(
                3,
                &format!(
                    "sc: {} cards found, partner_evaluation: {} entries",
                    scs.len(),
                    partner_evaluation.len()
                ),
            );
        }
    }

    // --- Partner evaluation/bond (confirmed Evaluation layout) ---
    log_predict_step("S:p3 done");
    ura_log(3, "★ read_summary phase4: partner evaluation");
    log_predict_step("S:p4 eval");

    let mut evaluation_entries: Vec<(i32, i32)> = partner_evaluation
        .iter()
        .map(|(&partner_id, &evaluation)| (partner_id, evaluation))
        .collect();

    // HashMap iteration order is undefined, so sort by partner ID
    // to keep /summary stable between requests.
    evaluation_entries.sort_unstable_by_key(|&(partner_id, _)| partner_id);

    let ev_json = format!(
        "[{}]",
        evaluation_entries
            .iter()
            .map(|&(partner_id, evaluation)| {
                format!(
                    r#"{{"target_id":{},"partner_id":{},"evaluation":{},"current_bond":{}}}"#,
                    partner_id, partner_id, evaluation, evaluation
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    );

    // --- Training levels (graceful fallback) ---
    log_predict_step("S:p4 done");
    ura_log(3, "★ read_summary phase5: training_levels");
    log_predict_step("S:p5 levels");
    let mut tl_json = "[]".to_string();
    let tl_arr = read_field_value(chara_class, chara_obj, "training_level_info_array");
    if tl_arr.is_null() {
        let arr = call_getter_on_instance(chara_class, chara_obj, "get_TrainingLevelInfoArray"); // [INVOKE-08] get_TrainingLevelInfoArray — 唯一调用
        if !arr.is_null() {
            let ab = arr as *const u8;
            let al =
                std::ptr::read_unaligned::<usize>(ab.add(IL2CPP_LIST_COUNT_OFF) as *const usize);
            if al > 0 && al < 100 {
                let mut tls = Vec::new();
                for i in 0..al {
                    let ep = std::ptr::read_unaligned::<*mut c_void>(
                        ab.add(IL2CPP_LIST_ITEMS_OFF + i * IL2CPP_LIST_ITEM_SIZE)
                            as *const *mut c_void,
                    );
                    if ep.is_null() {
                        continue;
                    }
                    let b = ep as *const u8;
                    let command_id =
                        std::ptr::read_unaligned::<i32>(b.add(IL2CPP_COMMAND_ID_OFF) as *const i32);
                    let level = std::ptr::read_unaligned::<i32>(
                        b.add(IL2CPP_COMMAND_LEVEL_OFF) as *const i32
                    );
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
                let ep = std::ptr::read_unaligned::<*mut c_void>(
                    ab.add(IL2CPP_LIST_ITEMS_OFF + i * IL2CPP_LIST_ITEM_SIZE) as *const *mut c_void,
                );
                if ep.is_null() {
                    continue;
                }
                let b = ep as *const u8;
                let command_id =
                    std::ptr::read_unaligned::<i32>(b.add(IL2CPP_COMMAND_ID_OFF) as *const i32);
                let level =
                    std::ptr::read_unaligned::<i32>(b.add(IL2CPP_COMMAND_LEVEL_OFF) as *const i32);
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
    // ★ v3.22.51: sid==14 skips try_get_scenario_obj (data pre-read in ramen section)
    let scenario_obj = if sid == 14 {
        ptr::null_mut()
    } else {
        try_get_scenario_obj(chara_class, chara_obj, sid)
    };
    if !scenario_obj.is_null() {
        let sc_name = match sid {
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
            _ => "",
        };
        if !sc_name.is_empty() {
            let sc_class = find_class_by_short_name(image, sc_name);
            if !sc_class.is_null() {
                let ds_obj = call_getter_on_instance(sc_class, scenario_obj, "get_DataSet"); // [INVOKE-09] get_DataSet — ★ 与 INVOKE-05 重复，待去重
                if !ds_obj.is_null() {
                    let ds_name = format!("{}DataSet", sc_name);
                    let ds_class = find_class_by_short_name(image, &ds_name);
                    if !ds_class.is_null() {
                        // ★ Breeders EnhanceGroups → override chara_effect_ids buffs
                        if sid == 13 {
                            let enhance_cls = find_class_by_short_name(
                                image,
                                "ObscuredSingleModeBreedersEnhanceGroup",
                            );
                            if !enhance_cls.is_null() {
                                let enhance_arr = call_getter_on_instance( // [INVOKE-10] get_EnhanceGroupArray — 循环外
                                    ds_class,
                                    ds_obj,
                                    "get_EnhanceGroupArray",
                                );
                                if !enhance_arr.is_null() {
                                    let eb = enhance_arr as *const u8;
                                    let el = std::ptr::read_unaligned::<usize>(
                                        eb.add(IL2CPP_LIST_COUNT_OFF) as *const usize,
                                    );
                                    if el > 0 && el < 20 {
                                        let mut buffs = Vec::new();
                                        for i in 0..el {
                                            let ep =
                                                std::ptr::read_unaligned::<*mut c_void>(eb.add(
                                                    IL2CPP_LIST_ITEMS_OFF
                                                        + i * IL2CPP_LIST_ITEM_SIZE,
                                                )
                                                    as *const *mut c_void);
                                            if ep.is_null() {
                                                continue;
                                            }
                                            let gt = call_getter_obscured_int( // [INVOKE-11] get_GainTotal (obscured) — 循环内倍增
                                                enhance_cls,
                                                ep,
                                                "get_GroupType",
                                            );
                                            let lv = call_getter_obscured_int( // [INVOKE-12] get_Level (obscured) — 循环内倍增
                                                enhance_cls,
                                                ep,
                                                "get_Level",
                                            );
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
                        // ★ v3.22.89: Removed dead Ramen buffs code here
                        // (sid==14 sets scenario_obj=null, so this block never executes for Ramen.
                        //  Ramen buffs are handled below after the scenario_obj block.)
                    }
                }
            }
        }
    }

    // ★ v3.22.51: Ramen buffs — extracted outside nested block (uses pre-read data only)
    if sid == 14 && !ramen_active_effects_raw_json.is_empty() {
        let mut buffs = Vec::new();
        for ae_part in ramen_active_effects_raw_json.split("},{") {
            let mut cat: i32 = -1;
            let mut eid: i32 = 0;
            let mut val: i32 = 0;
            for field in ae_part
                .trim_start_matches('{')
                .trim_end_matches('}')
                .split(',')
            {
                let fv: Vec<&str> = field.splitn(2, ':').collect();
                if fv.len() == 2 {
                    let key = fv[0].trim();
                    if key.contains("category") {
                        cat = fv[1].parse().unwrap_or(-1);
                    } else if key.contains("id") && !key.contains("Eff") {
                        eid = fv[1].parse().unwrap_or(0);
                    } else if key.contains("value") {
                        val = fv[1].parse().unwrap_or(0);
                    }
                }
            }
            if cat >= 0 {
                let cat_name = match cat {
                    1 => "試食会",
                    2 => "地域",
                    4 => "隠し味",
                    _ => "他",
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
                1 => "試食会",
                2 => "地域",
                4 => "隠し味",
                _ => "?",
            };
            let state_name = match ramen_uraf_state {
                0 => "無効",
                1 => "有効",
                _ => "?",
            };
            buffs.push(format!(
                r#"{{"name":"裏風:{}","UrafEffectType":{},"type":"Ramen"}}"#,
                ut_name, ramen_uraf_type
            ));
            buffs.push(format!(
                r#"{{"name":"裏風状態","state":"{}","UrafEffectState":{},"type":"Ramen"}}"#,
                state_name, ramen_uraf_state
            ));
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
            turn,
            stats,
            vit,
            mvit,
            mot,
            sid,
            &eval_trainings,
            has_ai_jiao,
            has_renshou_jouzu,
            skill_eval,
            skill_count, // ★ v3.22.0
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

    // ★ v3.22.39: Ramen scenario data — sozai counts aggregated during read
    let ramen_json = if sid == 14 && ramen_checkpoint_pt >= 0 {
        // Compute moriagari_level from checkpoint_pt thresholds
        let moriagari_level = if ramen_checkpoint_pt >= 480 {
            5
        } else if ramen_checkpoint_pt >= 330 {
            4
        } else if ramen_checkpoint_pt >= 210 {
            3
        } else if ramen_checkpoint_pt >= 120 {
            2
        } else if ramen_checkpoint_pt >= 50 {
            1
        } else {
            0
        };
        format!(
            r#","ramen":{{"checkpoint_pt":{},"moriagari_level":{},"special_feeling_num":{},"recommend_type":{},"sozai":[{},{},{}],"feeling_info":[{}],"selected_region_ids":[{}],"active_effects":[{}],"gauge_gains":[{}]}}"#,
            ramen_checkpoint_pt,
            moriagari_level,
            ramen_special_feeling_num,
            ramen_recommend_type,
            ramen_sozai_counts[0],
            ramen_sozai_counts[1],
            ramen_sozai_counts[2],
            ramen_feeling_info_json,
            ramen_selected_region_ids_json,
            ramen_active_effects_raw_json,
            ramen_gauge_gains_json
        )
    } else {
        String::new()
    };

    // ★ v2.2: last_action 字段 — 从缓存读取，不调用 IL2CPP
    let last_action_json = {
        let _lock = LAST_ACTION_MUTEX.lock();
        let cmd_id = unsafe { LAST_ACTION_COMMAND_ID };
        let seq = unsafe { LAST_ACTION_SEQUENCE };
        drop(_lock);
        if cmd_id >= 0 {
            let (action, normalized) = match cmd_id {
                101 => ("Speed", 101),
                102 => ("Power", 102),
                103 => ("Guts", 103),
                105 => ("Stamina", 105),
                106 => ("Wiz", 106),
                601 => ("Speed", 101),
                602 => ("Stamina", 105),
                603 => ("Power", 102),
                604 => ("Guts", 103),
                605 => ("Wiz", 106),
                _ => ("Unknown", cmd_id),
            };
            format!(
                r#","last_action":{{"sequence":{},"raw_command_id":{},"normalized_command_id":{},"action":"{}","source":"training_hook"}}"#,
                seq, cmd_id, normalized, action
            )
        } else {
            String::new()
        }
    };

    log_predict_step("S:json");
    format!(
        r#"{{"version":"{}","year":{},"turn":{},"month":{},"half":{},"scenario":"{}","chara_id":{},"stats":{{"speed":{},"stamina":{},"power":{},"guts":{},"wiz":{},"vital":{},"max_vital":{},"motivation":"{}","skill_point":{},"fan":{}}},"max_stats":{{"speed":{},"stamina":{},"power":{},"guts":{},"wiz":{}}},"proper":{{"dist_short":{},"dist_mile":{},"dist_mid":{},"dist_long":{},"ground_turf":{},"ground_dirt":{}}},"running_style":{},"scenario_progress":{},"training_event_type":{},"talent_level":{},"chara_grade":{},"difficulty":{},"fixed_turn_chara_seed":{},"trainings":{},"support_cards":{},"evaluation":{},"training_levels":{},"buffs":{},"chara_effect_ids":[{}],"skills":{{"eval":{},"count":{},"list":{}}},"ai":{}{}{}{} }}"#,
        PLUGIN_VERSION,
        year,
        cumulative_turn,
        mon,
        half,
        scn_s,
        chara_id,
        spd,
        sta,
        pow_,
        gut,
        wiz,
        vit,
        mvit,
        mot_s,
        spt,
        fan,
        max_spd,
        max_sta,
        max_pow,
        max_gut,
        max_wiz,
        proper_dist_short,
        proper_dist_mile,
        proper_dist_mid,
        proper_dist_long,
        proper_ground_turf,
        proper_ground_dirt,
        running_style,
        scenario_progress,
        training_event_type,
        talent_level,
        chara_grade,
        difficulty,
        fixed_turn_chara_seed,
        tr_json,
        sc_json,
        ev_json,
        tl_json,
        buff_json,
        effect_ids_str.join(","),
        skill_eval,
        skill_count,
        skills_json,
        ai_json,
        team_json,
        ramen_json,
        last_action_json
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
    if !cfg.push_enabled {
        return;
    }
    let addr_str = cfg.push_addr();
    let addr: std::net::SocketAddr = match addr_str.parse() {
        Ok(a) => a,
        Err(_) => return,
    };
    let mut stream =
        match std::net::TcpStream::connect_timeout(&addr, std::time::Duration::from_secs(2)) {
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
    let interval =
        std::time::Duration::from_secs(unsafe { get_config() }.push_interval_secs.max(2));
    let mut consecutive_errors: u32 = 0;

    // ★ Initial push: try pushing current data on startup
    // Don't rely solely on GAME_INITIALIZED callback — it may never fire
    // if the game was already initialized before the plugin loaded.
    // Instead, try reading data; if it succeeds, the game is ready.
    for wait_round in 0..60 {
        if GAME_INITIALIZED.load(Ordering::Relaxed) {
            break;
        }
        // Try a probe read — if it doesn't error, game is ready
        let probe = read_summary();
        if !probe.contains("\"error\"") {
            GAME_INITIALIZED.store(true, Ordering::Relaxed);
            unsafe {
                ura_log(3, "Push: game detected via probe (no callback)");
                // v3.22.98: Install hooks in fallback (on_game_initialized may never fire)
                install_training_hook();
                install_exec_training_hook();
                install_failure_rate_hook();
                install_event_choice_hook();
            }
            break;
        }
        if wait_round % 10 == 0 {
            unsafe {
                ura_log(
                    3,
                    &format!("Push: waiting for game... round={}", wait_round),
                );
            }
        }
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
    let init_summary = read_summary();
    if !init_summary.contains("\"error\"") {
        unsafe {
            LAST_PUSH_HASH = simple_hash(&init_summary);
        }
        push_to_app(&init_summary);
        unsafe {
            ura_log(3, "Push: initial data pushed");
        }
    }

    loop {
        std::thread::sleep(interval);
        // Don't gate on GAME_INITIALIZED — just try reading;
        // if the game isn't ready, read_summary returns error and we skip.
        let summary = read_summary();
        if summary.contains("\"error\"") {
            consecutive_errors += 1;
            // ★ v3.22.89: Extra cooldown for SIGSEGV recovery — game state transition
            if summary.contains("sigsegv") {
                let cool = std::time::Duration::from_secs(60);
                unsafe {
                    ura_log(
                        2,
                        "Push: SIGSEGV recovered, cooling 60s for game state transition",
                    );
                }
                std::thread::sleep(cool);
            }
            // ★ v3.14.2: backoff on consecutive errors to avoid crash loop
            if consecutive_errors >= 1 {
                let backoff =
                    std::time::Duration::from_secs((consecutive_errors as u64 * 5).min(60));
                unsafe {
                    ura_log(
                        3,
                        &format!(
                            "Push: {} consecutive errors, backing off {}s",
                            consecutive_errors,
                            backoff.as_secs()
                        ),
                    );
                }
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
            unsafe {
                ura_log(3, "Push: data changed, pushing to app");
            }
            push_to_app(&summary);
        }
    }
}

fn start_http_server() {
    if HTTP_RUNNING.load(Ordering::Relaxed) {
        return;
    }
    HTTP_RUNNING.store(true, Ordering::Relaxed);
    std::thread::spawn(|| {
        unsafe {
            ura_log(3, "HTTP starting on 0.0.0.0:18765");
        }
        let listener = match std::net::TcpListener::bind("0.0.0.0:18765") {
            Ok(l) => l,
            Err(e) => {
                unsafe {
                    ura_log(1, &format!("HTTP bind failed: {}", e));
                }
                HTTP_RUNNING.store(false, Ordering::Relaxed);
                return;
            }
        };
        unsafe {
            ura_log(3, "HTTP listening on :18765");
        }
        unsafe {
            ura_notify("URA HTTP :18765 ON");
        }

        // ★ Start push-to-app loop (v3.10.0)
        std::thread::spawn(|| {
            push_loop();
        });

        for stream in listener.incoming() {
            if !HTTP_RUNNING.load(Ordering::Relaxed) {
                break;
            }
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
            let result = if after_host.is_empty() {
                "/".to_string()
            } else {
                format!("/{}", after_host)
            };
            return result.trim_end_matches('/').to_string();
        }
        return "/".to_string();
    }
    if path.len() > 1 && path.ends_with('/') {
        path[..path.len() - 1].to_string()
    } else {
        path.to_string()
    }
}

/// ★ v3.24.9: Debug Ramen gains — trace every step to find why gains is empty
unsafe fn debug_ramengains() -> String {
    if API.is_null() {
        return r#"{"error":"api_null"}"#.to_string();
    }
    let image = match get_image() {
        img if !img.is_null() => img,
        _ => return r#"{"error":"image_null"}"#.to_string(),
    };

    let mut parts: Vec<String> = Vec::new();

    // Step 1: Get WorkDataManager
    let wdm_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkDataManager").as_ptr(),
    );
    parts.push(format!(
        r#""wdm_class":{}"#,
        if wdm_class.is_null() { "null" } else { "ok" }
    ));
    if wdm_class.is_null() {
        return format!("{{{}}}", parts.join(","));
    }

    let wdm_inst = get_singleton(wdm_class);
    parts.push(format!(r#""wdm_inst":"0x{:x}""#, wdm_inst as usize));
    if wdm_inst.is_null() {
        return format!("{{{}}}", parts.join(","));
    }

    // Step 2: Get SingleMode
    let sm_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkSingleModeData").as_ptr(),
    );
    let sm_obj = call_getter_ref(wdm_class, wdm_inst, "get_SingleMode");
    parts.push(format!(r#""sm_obj":"0x{:x}""#, sm_obj as usize));
    if sm_obj.is_null() {
        return format!("{{{}}}", parts.join(","));
    }

    // Step 3: Get Character
    let chara_obj = call_getter_ref(sm_class, sm_obj, "get_Character");
    parts.push(format!(r#""chara_obj":"0x{:x}""#, chara_obj as usize));
    if chara_obj.is_null() {
        return format!("{{{}}}", parts.join(","));
    }

    // Step 4: Get scenario_id
    let chara_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkSingleModeCharaData").as_ptr(),
    );
    let sid = read_obscured_int_at(chara_obj, 568);
    parts.push(format!(r#""scenario_id":{}"#, sid));

    // Step 5: Get scenario_obj
    let scenario_obj = try_get_scenario_obj(chara_class, chara_obj, 14);
    parts.push(format!(r#""scenario_obj":"0x{:x}""#, scenario_obj as usize));
    if scenario_obj.is_null() {
        parts.push(r#""error":"scenario_obj_null""#.to_string());
        return format!("{{{}}}", parts.join(","));
    }

    // Step 6: Get DataSet
    let sc_class = std::ptr::read_unaligned::<*mut c_void>(scenario_obj as *const *mut c_void);
    let dataset_obj = call_getter_ref(sc_class, scenario_obj, "get_DataSet");
    parts.push(format!(r#""dataset_obj":"0x{:x}""#, dataset_obj as usize));
    if dataset_obj.is_null() {
        parts.push(r#""error":"dataset_obj_null""#.to_string());
        return format!("{{{}}}", parts.join(","));
    }

    // Step 7: Read CommandInfoArray at offset 16
    let cmd_list = read_ptr_at(dataset_obj, 16);
    parts.push(format!(r#""cmd_list_ptr":"0x{:x}""#, cmd_list as usize));
    if cmd_list.is_null() {
        parts.push(r#""error":"cmd_list_null""#.to_string());
        return format!("{{{}}}", parts.join(","));
    }

    let cmd_lb = cmd_list as *const u8;
    let cmd_count =
        std::ptr::read_unaligned::<usize>(cmd_lb.add(IL2CPP_LIST_COUNT_OFF) as *const usize);
    parts.push(format!(r#""cmd_count":{}"#, cmd_count));

    // Step 8: Dump each command element
    let mut cmd_parts: Vec<String> = Vec::new();
    for ci in 0..cmd_count.min(10) {
        let ce = std::ptr::read_unaligned::<*mut c_void>(
            cmd_lb.add(IL2CPP_LIST_ITEMS_OFF + ci * IL2CPP_LIST_ITEM_SIZE) as *const *mut c_void,
        );
        if ce.is_null() {
            cmd_parts.push(format!(r#"{{"idx":{},"null":true}}"#, ci));
            continue;
        }

        let ce_b = ce as *const u8;

        // Read raw bytes at offset 36 (CommandId ObscuredInt) and 56 (ParamsIncDecInfoArray ptr)
        let raw_36 = std::ptr::read_unaligned::<i32>(ce_b.add(36) as *const i32);
        let raw_40 = std::ptr::read_unaligned::<i32>(ce_b.add(40) as *const i32);
        let cmd_id = raw_36 ^ raw_40; // ObscuredInt XOR

        let params_ptr = read_ptr_at(ce, 56);

        // Also try offset 16 and 20 (CommandType)
        let raw_16 = std::ptr::read_unaligned::<i32>(ce_b.add(16) as *const i32);
        let raw_20 = std::ptr::read_unaligned::<i32>(ce_b.add(20) as *const i32);
        let cmd_type = raw_16 ^ raw_20;

        // Read raw hex at 0x10-0x40 for debugging
        let mut hex_parts = Vec::new();
        for off in (16..72).step_by(4) {
            let val = std::ptr::read_unaligned::<i32>(ce_b.add(off) as *const i32);
            hex_parts.push(format!(r#""0x{:02x}":{}"#, off, val));
        }

        let params_info = if !params_ptr.is_null() {
            let plb = params_ptr as *const u8;
            let plen =
                std::ptr::read_unaligned::<usize>(plb.add(IL2CPP_LIST_COUNT_OFF) as *const usize);
            if plen > 0 && plen < 100 {
                let mut gain_parts: Vec<String> = Vec::new();
                for pi in 0..plen {
                    let pe = std::ptr::read_unaligned::<*mut c_void>(
                        plb.add(IL2CPP_LIST_ITEMS_OFF + pi * IL2CPP_LIST_ITEM_SIZE)
                            as *const *mut c_void,
                    );
                    if pe.is_null() {
                        continue;
                    }
                    let tt =
                        std::ptr::read_unaligned::<i32>((pe as *const u8).add(16) as *const i32);
                    let vv =
                        std::ptr::read_unaligned::<i32>((pe as *const u8).add(20) as *const i32);
                    gain_parts.push(format!(r#"{{"tt":{},"vv":{}}}"#, tt, vv));
                }
                format!(
                    r#""params_len":{},"items":[{}]"#,
                    plen,
                    gain_parts.join(",")
                )
            } else {
                format!(r#""params_len":{}"#, plen)
            }
        } else {
            r#""params_ptr":"null""#.to_string()
        };

        cmd_parts.push(format!(
            r#"{{"idx":{},"cmd_id":{},"cmd_type":{},"params_ptr":"0x{:x}",{},"raw":{{{}}}}}"#,
            ci,
            cmd_id,
            cmd_type,
            params_ptr as usize,
            params_info,
            hex_parts.join(",")
        ));
    }

    parts.push(format!(r#""commands":[{}]"#, cmd_parts.join(",")));

    // Step 9: Also check HomeInfoData path
    let home_info_obj = call_getter_ref(sm_class, sm_obj, "get_HomeInfoData");
    parts.push(format!(
        r#""home_info_obj":"0x{:x}""#,
        home_info_obj as usize
    ));
    if !home_info_obj.is_null() {
        let hi_class = find_class(
            image,
            to_cstr("Gallop").as_ptr(),
            to_cstr("WorkSingleModeHomeInfoData").as_ptr(),
        );
        if !hi_class.is_null() {
            let cmd_arr = read_field_value(hi_class, home_info_obj, "CommandInfoArray");
            parts.push(format!(r#""home_cmd_arr":"0x{:x}""#, cmd_arr as usize));
            if !cmd_arr.is_null() {
                let ab = cmd_arr as *const u8;
                let al = std::ptr::read_unaligned::<usize>(
                    ab.add(IL2CPP_LIST_COUNT_OFF) as *const usize
                );
                parts.push(format!(r#""home_cmd_count":{}"#, al));
                // Check first element's ParamsIncDecInfoArray
                if al > 0 {
                    let ep = std::ptr::read_unaligned::<*mut c_void>(
                        ab.add(IL2CPP_LIST_ITEMS_OFF) as *const *mut c_void
                    );
                    if !ep.is_null() {
                        let pa = read_ptr_at(ep as *const c_void, 96);
                        parts.push(format!(r#""home_elem0_params":"0x{:x}""#, pa as usize));
                        if !pa.is_null() {
                            let pl = std::ptr::read_unaligned::<usize>(
                                (pa as *const u8).add(IL2CPP_LIST_COUNT_OFF) as *const usize,
                            );
                            parts.push(format!(r#""home_elem0_params_len":{}"#, pl));
                        }
                    }
                }
            }
        }
    }

    format!("{{{}}}", parts.join(","))
}

fn handle_http(mut stream: std::net::TcpStream) {
    use std::io::{Read, Write};
    let mut buf = [0u8; 8192];
    let n = match stream.read(&mut buf) {
        Ok(n) if n > 0 => n,
        _ => return,
    };
    let req = std::str::from_utf8(&buf[..n]).unwrap_or("");
    let path = parse_path(req);
    let full_uri = req
        .lines()
        .next()
        .unwrap_or("")
        .split(' ')
        .nth(1)
        .unwrap_or("/");

    // ★ 白名单下载开关：名单内端点追加 ?dl=1 即以附件形式返回（解决手机复制长度上限）
    //    ?dl=1&name=xxx 可自定义文件名（仅保留字母数字和下划线/连字符）
    //    大文件仍走各专用流式 _dl 端点，避免此路径内存翻倍
    const DL_ALLOWED: &[&str] = &[
        "/summary", "/scenario", "/data",
        "/api/sniff", "/api/sniff/diag", "/api/event/choices",
        "/debug/all", "/debug/params", "/debug/cmdinfo", "/debug/breeders",
        "/debug/training_partners", "/debug/rameninfo", "/debug/laststep",
        "/debug/storydata", "/debug/ramenfields", "/debug/gauge", "/debug/gauge2",
        "/debug/ramengains", "/debug/paramsincdec", "/debug/training_seed",
        "/debug/unique_skills", "/debug/hint_gain", "/debug/sc_effect",
        "/debug/unique_detail", "/classes",
    ];
    let dl_flag = parse_query(&full_uri, "dl");
    let dl_name = parse_query(&full_uri, "name");
    let dl_enabled = !dl_flag.is_empty() && dl_flag != "0"
        && DL_ALLOWED.iter().any(|p| path == *p);

    let body = if path == "/" || path == "/health" {
        format!(r#"{{"status":"ok","version":"{}","endpoints":["/summary","/data","/scenario","/debug/rameninfo","/debug/laststep","/event/recommend","/inherit/compat","/saddle-analysis","/log/turn","/debug/params","/debug/breeders","/debug/cmdinfo","/debug/training_partners","/debug/crashlog","/debug/upload","/debug/dumpclass","/debug/storydata","/debug/ramenfields","/debug/gauge","/debug/gauge2","/debug/ramengains","/debug/paramsincdec","/debug/training_seed","/debug/training_log","/debug/training_log_dl","/update","/update/status","/debug/all","/debug/unique_skills","/debug/mdb_all_tables","/debug/mdb_schema_dump","/debug/hint_gain","/debug/sc_effect","/debug/unique_detail","/debug/table","/debug/push_table","/debug/download_table","/mdb","/carddb","/skilldata","/hall","/saddles","/saddles-dl","/log","/status","/health","/mdb/schema","/mdb/search","/mdb/raw","/mdb/dl_batch","/il2cpp/dump","/il2cpp/call","/il2cpp/tree","/il2cpp/field","/il2cpp/classes","/il2cpp/static","/il2cpp/methods","/il2cpp/disassemble","/il2cpp/disassemble_dl","/il2cpp/disassemble_addr","/il2cpp/disassemble_addr_dl","/il2cpp/dump_all_methods","/il2cpp/dump_all_methods_dl","/il2cpp/search_float","/il2cpp/search_float_dl","/il2cpp/search_int","/il2cpp/search_int_dl","/il2cpp/search_methods","/il2cpp/search_methods_dl","/il2cpp/read_mem","/il2cpp/read_mem_dl","/training/result","/api/sniff","/api/sniff/toggle","/api/sniff/clear","/api/sniff/diag","/api/event/choices","/api/event/clear","/action/latest","/seed/history","/seed/stats"]}}"#, PLUGIN_VERSION)
    } else if path == "/scan" {
        unsafe { scan_il2cpp_classes() }
    } else if path == "/data" {
        let result = unsafe { read_training_data() };
        unsafe {
            log_snapshot("data", &result);
        }
        result
    } else if path == "/status" {
        format!(
            r#"{{"game_initialized":{},"http_running":{}}}"#,
            GAME_INITIALIZED.load(Ordering::Relaxed),
            HTTP_RUNNING.load(Ordering::Relaxed)
        )
    } else if path == "/singletons" {
        unsafe { find_all_singletons() }
    } else if path.starts_with("/find_method") {
        let method_name = if path == "/find_method" || path == "/find_method/" {
            "get_SingleMode"
        } else {
            path.strip_prefix("/find_method/")
                .unwrap_or("get_SingleMode")
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
        unsafe {
            log_snapshot("scenario", &result);
        }
        result
    } else if path == "/log" {
        unsafe { get_training_log() }
    } else if path == "/debug/params" {
        unsafe { debug_params_inc_dec() }
    } else if path == "/debug/breeders" {
        unsafe { debug_breeders_team() }
    } else if path == "/debug/rameninfo" {
        // Dump ramen DataSet raw memory for field layout analysis
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
            read_ramen_info()
        }))
        .unwrap_or_else(|_| r#"{"error":"rameninfo_panic"}"#.to_string())
    } else if path == "/debug/laststep" {
        let step = PREDICT_STEP.load(std::sync::atomic::Ordering::Relaxed);
        let len = LAST_STEP_LEN.load(std::sync::atomic::Ordering::Relaxed) as usize;
        let msg = if len > 0 && len < 128 {
            unsafe {
                let buf_ptr = LAST_STEP_BUF.as_ptr();
                std::ffi::CStr::from_ptr(buf_ptr)
                    .to_string_lossy()
                    .into_owned()
            }
        } else {
            String::new()
        };
        format!(r#"{{"step":{},"last_step":"{}"}}"#, step, msg)
    } else if path == "/debug/crashlog" {
        read_crash_log()
    } else if path == "/debug/upload" {
        upload_all_logs()
    } else if path == "/debug/cmdinfo" {
        unsafe { debug_cmdinfo() }
    } else if path == "/debug/training_partners" {
        debug_training_partners()
    } else if path == "/training/result" {
        // v3.22.94: Read latest training result from hook
        let result = unsafe { LAST_TRAINING_RESULT };
        let sub_id = unsafe { LAST_TRAINING_SUB_ID };
        let hooked = unsafe { TRAINING_HOOK_INSTALLED };
        format!(
            r#"{{"result_type":{},"sub_id":{},"hooked":{},"result_name":"{}"}}"#,
            result,
            sub_id,
            hooked,
            match result {
                0 => "GreatSuccess",
                1 => "Success",
                2 => "Failure",
                _ => "Unknown",
            }
        )
    } else if path == "/api/sniff/toggle" {
        let new_val = !SNIFF_ENABLED.load(Ordering::Relaxed);
        SNIFF_ENABLED.store(new_val, Ordering::Relaxed);
        let req_hooked = unsafe { COMPRESS_REQUEST_ADDR != 0 };
        let resp_hooked = unsafe { DECOMPRESS_RESPONSE_ADDR != 0 };
        let post_hooked = unsafe { POST_ADDR != 0 };
        format!(
            r#"{{"sniff_enabled":{},"compress_hooked":{},"decompress_hooked":{},"post_hooked":{}}}"#,
            new_val, req_hooked, resp_hooked, post_hooked
        )
    } else if path == "/api/sniff/clear" {
        let _lock = SNIFF_MUTEX.lock();
        unsafe {
            SNIFF_REQUESTS.clear();
            SNIFF_RESPONSES.clear();
        }
        r#"{"ok":true}"#.to_string()
    } else if path == "/api/sniff/diag" {
        // v3.23.3: Diagnostic endpoint for hook installation (Interceptor API)
        let req_hooked = unsafe { COMPRESS_REQUEST_ADDR != 0 };
        let resp_hooked = unsafe { DECOMPRESS_RESPONSE_ADDR != 0 };
        let post_hooked = unsafe { POST_ADDR != 0 };
        let req_addr = unsafe { COMPRESS_REQUEST_ADDR };
        let resp_addr = unsafe { DECOMPRESS_RESPONSE_ADDR };
        let post_addr = unsafe { POST_ADDR };
        let interceptor_available = unsafe { !API.is_null() && (*API).interceptor != 0 };
        let has_get_method_addr =
            unsafe { !API.is_null() && (*API).il2cpp_get_method_addr_fn.is_some() };
        format!(
            r#"{{"sniff_enabled":{},"compress_hooked":{},"decompress_hooked":{},"post_hooked":{},"compress_addr":"0x{:x}","decompress_addr":"0x{:x}","post_addr":"0x{:x}","interceptor_available":{},"get_method_addr_available":{}}}"#,
            SNIFF_ENABLED.load(Ordering::Relaxed),
            req_hooked,
            resp_hooked,
            post_hooked,
            req_addr,
            resp_addr,
            post_addr,
            interceptor_available,
            has_get_method_addr
        )
    } else if path == "/api/sniff" {
        let _lock = SNIFF_MUTEX.lock();
        unsafe {
            let reqs: Vec<String> = SNIFF_REQUESTS
                .iter()
                .map(|(rid, url, headers, data)| {
                    let preview = String::from_utf8_lossy(&data[..data.len().min(2048)]);
                    let preview = preview
                        .replace('\\', "\\\\")
                        .replace('"', "\\\"")
                        .replace('\n', "\\n")
                        .replace('\r', "");
                    let url_escaped = url.replace('\\', "\\\\").replace('"', "\\\"");
                    format!(
                        r#"{{"id":{},"url":"{}","headers":{},"size":{},"hex":"{}","text":"{}"}}"#,
                        rid,
                        url_escaped,
                        headers,
                        data.len(),
                        hex_encode(&data[..data.len().min(256)]),
                        preview
                    )
                })
                .collect();
            let resps: Vec<String> = SNIFF_RESPONSES
                .iter()
                .map(|(rid, data)| {
                    let preview = String::from_utf8_lossy(&data[..data.len().min(2048)]);
                    let preview = preview
                        .replace('\\', "\\\\")
                        .replace('"', "\\\"")
                        .replace('\n', "\\n")
                        .replace('\r', "");
                    format!(
                        r#"{{"id":{},"size":{},"hex":"{}","text":"{}"}}"#,
                        rid,
                        data.len(),
                        hex_encode(&data[..data.len().min(256)]),
                        preview
                    )
                })
                .collect();
            format!(
                r#"{{"enabled":{},"requests":[{}],"responses":[{}]}}"#,
                SNIFF_ENABLED.load(Ordering::Relaxed),
                reqs.join(","),
                resps.join(",")
            )
        }
    } else if path == "/api/event/choices" {
        // v3.24.2: Return captured event choices
        let _lock = EVENT_STATE_MUTEX.lock();
        unsafe {
            let choices_json: Vec<String> = EVENT_CHOICES.iter().map(|c| {
                format!(r#"{{"label":"{}","gain_id":{},"next_block_idx":{},"loop_exit_gain_id":{}}}"#,
                    c.label.replace('\\', "\\\\").replace('"', "\\\""),
                    c.gain_id, c.next_block_idx, c.loop_exit_gain_id)
            }).collect();
            let result = format!(
                r#"{{"story_id":{},"chara_id":{},"selected_idx":{},"choices":[{}]}}"#,
                EVENT_STORY_ID,
                EVENT_CHARA_ID,
                EVENT_SELECTED_IDX,
                choices_json.join(",")
            );
            drop(_lock);
            result
        }
    } else if path == "/api/event/clear" {
        let _lock = EVENT_STATE_MUTEX.lock();
        unsafe {
            EVENT_CHOICES.clear();
            EVENT_SELECTED_IDX = -1;
            EVENT_STORY_ID = 0;
            EVENT_CHARA_ID = 0;
        }
        drop(_lock);
        r#"{"ok":true}"#.to_string()
    } else if path == "/action/latest" {
        // ★ v2.2: 返回最新动作记录（只读缓存，不调用 IL2CPP）
        let _lock = LAST_ACTION_MUTEX.lock();
        let cmd_id = unsafe { LAST_ACTION_COMMAND_ID };
        let seq = unsafe { LAST_ACTION_SEQUENCE };
        let result_type = unsafe { LAST_TRAINING_RESULT };
        drop(_lock);
        let (action, normalized) = match cmd_id {
            101 => ("Speed", 101),
            102 => ("Power", 102),
            103 => ("Guts", 103),
            105 => ("Stamina", 105),
            106 => ("Wiz", 106),
            601 => ("Speed", 101),
            602 => ("Stamina", 105),
            603 => ("Power", 102),
            604 => ("Guts", 103),
            605 => ("Wiz", 106),
            -1 => ("None", -1),
            _ => ("Unknown", cmd_id),
        };
        format!(
            r#"{{"sequence":{},"raw_command_id":{},"normalized_command_id":{},"action":"{}","result_type":{},"source":"training_hook"}}"#,
            seq, cmd_id, normalized, action, result_type
        )
    } else if path == "/seed/history" || path == "/seed/stats" {
        r#"{"ok":false,"deprecated":true,"rng_observation_valid":false,"rng_invalid_reason":"offset_0x198_is_ObscuredInt_not_u32x4"}"#.to_string()
    } else if path == "/debug/training_log" {
        let hooked = unsafe { EXEC_TRAINING_HOOK_INSTALLED };
        let addr = unsafe { EXEC_TRAINING_ADDR };
        format!(
            r#"{{"hooked":{},"addr":"0x{:x}","rng_observation_valid":false,"rng_invalid_reason":"offset_0x198_is_ObscuredInt_not_u32x4"}}"#,
            hooked, addr
        )
    } else if path == "/debug/training_log_dl" {
        // Do not re-export legacy files containing the invalid u32x4 interpretation.
        r#"{"ok":false,"deprecated":true,"rng_observation_valid":false,"rng_invalid_reason":"offset_0x198_is_ObscuredInt_not_u32x4"}"#.to_string()
    } else if path.starts_with("/debug/dumpclass") {
        // v3.22.51: Dump all fields of any IL2CPP class by name
        // Usage: /debug/dumpclass?name=WorkSingleModeData
        let class_name = if let Some(q) = full_uri.find("?name=") {
            &full_uri[q + 6..]
        } else {
            ""
        };
        unsafe { debug_dumpclass(class_name) }
    } else if path == "/debug/storydata" {
        // v3.22.35: Discover all DataSet getters, find story/event related arrays
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
            debug_storydata()
        }))
        .unwrap_or_else(|_| r#"{"error":"storydata_panic"}"#.to_string())
    } else if path == "/debug/all" {
        // ★ v3.22.35: Aggregate all debug data in one call — summary + scenario + storydata + cmdinfo + rameninfo
        unsafe { debug_all() }
    } else if path == "/debug/ramenfields" {
        // v3.22.51: Dump all ramen array element classes + their fields at runtime
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
            debug_ramenfields()
        }))
        .unwrap_or_else(|_| r#"{"error":"ramenfields_panic"}"#.to_string())
    } else if path == "/debug/gauge" {
        // ★ v3.22.39: sigsetjmp + READ_MUTEX protection — prevent game crash on SIGSEGV
        let _lock = READ_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let jmp_result = unsafe { sys_sigsetjmp(SIGSEGV_JMP_BUF.as_mut_ptr(), 1) };
        if jmp_result != 0 {
            r#"{"error":"sigsegv_recovered","hint":"/debug/gauge hit native crash, game protected"}"#.to_string()
        } else {
            SIGSEGV_RECOVERY.store(true, std::sync::atomic::Ordering::Relaxed);
            let result =
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe { debug_gauge() }))
                    .unwrap_or_else(|_| r#"{"error":"gauge_panic"}"#.to_string());
            SIGSEGV_RECOVERY.store(false, std::sync::atomic::Ordering::Relaxed);
            result
        }
    } else if path == "/debug/gauge2" {
        // v3.22.39: Scan all DataSet array fields for element class names
        let _lock = READ_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let jmp_result = unsafe { sys_sigsetjmp(SIGSEGV_JMP_BUF.as_mut_ptr(), 1) };
        if jmp_result != 0 {
            r#"{"error":"sigsegv_recovered","hint":"/debug/gauge2 hit native crash, game protected"}"#.to_string()
        } else {
            SIGSEGV_RECOVERY.store(true, std::sync::atomic::Ordering::Relaxed);
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
                debug_gauge2()
            }))
            .unwrap_or_else(|_| r#"{"error":"gauge2_panic"}"#.to_string());
            SIGSEGV_RECOVERY.store(false, std::sync::atomic::Ordering::Relaxed);
            result
        }
    } else if path == "/debug/ramengains" {
        // ★ v3.24.9: Diagnose Ramen gains reading — trace every step
        let _lock = READ_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let jmp_result = unsafe { sys_sigsetjmp(SIGSEGV_JMP_BUF.as_mut_ptr(), 1) };
        if jmp_result != 0 {
            r#"{"error":"sigsegv_recovered"}"#.to_string()
        } else {
            SIGSEGV_RECOVERY.store(true, std::sync::atomic::Ordering::Relaxed);
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
                debug_ramengains()
            }))
            .unwrap_or_else(|_| r#"{"error":"ramengains_panic"}"#.to_string());
            SIGSEGV_RECOVERY.store(false, std::sync::atomic::Ordering::Relaxed);
            result
        }
    } else if path == "/debug/paramsincdec" {
        // v3.22.40: Read DataSet CommandInfo ParamsIncDecInfoArray element class names
        let _lock = READ_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let jmp_result = unsafe { sys_sigsetjmp(SIGSEGV_JMP_BUF.as_mut_ptr(), 1) };
        if jmp_result != 0 {
            r#"{"error":"sigsegv_recovered","hint":"/debug/paramsincdec hit native crash, game protected"}"#.to_string()
        } else {
            SIGSEGV_RECOVERY.store(true, std::sync::atomic::Ordering::Relaxed);
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
                debug_paramsincdec()
            }))
            .unwrap_or_else(|_| r#"{"error":"paramsincdec_panic"}"#.to_string());
            SIGSEGV_RECOVERY.store(false, std::sync::atomic::Ordering::Relaxed);
            result
        }
    } else if path == "/debug/training_seed" {
        // 一键查找训练种子：自动完成 WorkDataManager → WorkSingleModeData → _fixedTurnCharaSeed
        let _lock = READ_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let jmp_result = unsafe { sys_sigsetjmp(SIGSEGV_JMP_BUF.as_mut_ptr(), 1) };
        if jmp_result != 0 {
            r#"{"error":"sigsegv_recovered","hint":"/debug/training_seed hit native crash, game protected"}"#.to_string()
        } else {
            SIGSEGV_RECOVERY.store(true, std::sync::atomic::Ordering::Relaxed);
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
                debug_training_seed()
            }))
            .unwrap_or_else(|_| r#"{"error":"training_seed_panic"}"#.to_string());
            SIGSEGV_RECOVERY.store(false, std::sync::atomic::Ordering::Relaxed);
            result
        }
    } else if path == "/update" {
        // v3.22.51: Self-update SO from GitHub Release
        update_so()
    } else if path == "/update/status" {
        // v3.22.51: Return auto-update status
        match AUTO_UPDATE_STATUS.lock() {
            Ok(guard) => match guard.as_ref() {
                Some(s) => s.clone(),
                None => {
                    r#"{"status":"pending","hint":"auto_update_check_not_started_yet"}"#.to_string()
                }
            },
            Err(_) => r#"{"status":"error","detail":"lock_failed"}"#.to_string(),
        }
    } else if path == "/events" {
        read_events_data()
    } else if path == "/debug/unique_skills" {
        debug_unique_skills()
    } else if path == "/debug/mdb_all_tables" {
        debug_mdb_all_tables()
    } else if path == "/debug/mdb_schema_dump" {
        debug_mdb_schema_dump()
    } else if path == "/debug/hint_gain" {
        debug_hint_gain()
    } else if path == "/debug/sc_effect" {
        debug_sc_effect()
    } else if path == "/debug/unique_detail" {
        debug_unique_detail()
    } else if path == "/debug/table" {
        let table_name = if let Some(q) = full_uri.find("?name=") {
            let rest = &full_uri[q + 6..];
            rest.split('&').next().unwrap_or(rest)
        } else {
            ""
        };
        let limit = if let Some(q) = full_uri.find("limit=") {
            full_uri[q + 6..]
                .split('&')
                .next()
                .unwrap_or("100")
                .parse::<usize>()
                .unwrap_or(100)
        } else {
            100usize
        };
        let offset = if let Some(q) = full_uri.find("offset=") {
            full_uri[q + 7..]
                .split("&")
                .next()
                .unwrap_or("0")
                .parse::<usize>()
                .unwrap_or(0)
        } else {
            0usize
        };
        debug_table_query(table_name, limit.min(1000).max(1), offset)
    } else if path == "/debug/download_table" {
        let table_name = if let Some(q) = full_uri.find("?name=") {
            let rest = &full_uri[q + 6..];
            rest.split('&').next().unwrap_or(rest)
        } else {
            ""
        };
        let batch = if let Some(q) = full_uri.find("batch=") {
            full_uri[q + 6..]
                .split('&')
                .next()
                .unwrap_or("500")
                .parse::<usize>()
                .unwrap_or(500)
        } else {
            500usize
        };
        debug_download_table(table_name, batch.min(1000).max(1))
    } else if path == "/debug/push_table" {
        let table_name = if let Some(q) = full_uri.find("?name=") {
            let rest = &full_uri[q + 6..];
            rest.split('&').next().unwrap_or(rest)
        } else {
            ""
        };
        let batch = if let Some(q) = full_uri.find("batch=") {
            full_uri[q + 6..]
                .split('&')
                .next()
                .unwrap_or("500")
                .parse::<usize>()
                .unwrap_or(500)
        } else {
            500usize
        };
        let offset = if let Some(q) = full_uri.find("offset=") {
            full_uri[q + 7..]
                .split('&')
                .next()
                .unwrap_or("0")
                .parse::<usize>()
                .unwrap_or(0)
        } else {
            0usize
        };
        debug_push_table(table_name, batch.min(1000).max(1), offset)
    } else if path == "/tables" {
        read_mdb_tables()
    } else if path == "/carddb" {
        read_carddb()
    } else if path == "/skilldata" {
        read_skilldata()
    } else if path == "/hall" {
        unsafe { read_hall_data() }
    } else if path == "/event/recommend" {
        unsafe { read_event_recommend() }
    } else if path == "/inherit/compat" {
        unsafe { read_inherit_compat() }
    } else if path == "/saddle-analysis" {
        unsafe { read_win_saddle_analysis() }
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
                unsafe {
                    update_config(new_cfg);
                }
                unsafe {
                    ura_log(3, &format!("Config updated: {}", json));
                }
                format!(r#"{{"ok":true,"config":{}}}"#, json)
            } else {
                r#"{"ok":false,"error":"invalid_json"}"#.to_string()
            }
        } else {
            format!(
                r#"{{"ok":true,"config":{}}}"#,
                unsafe { get_config() }.to_json()
            )
        }
    } else if path == "/debug/dump" {
        // v3.22.89: Dump tool - group tables by first letter, one file per group
        let html = r#"<html><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Dump</title><style>body{font-family:system-ui;max-width:600px;margin:12px auto;padding:0 8px;background:#1a1a2e;color:#e0e0e0}h1{color:#4fc3f7;font-size:1.2em;margin:8px 0}.g{display:inline-block;margin:4px 2px;padding:8px 12px;background:#16213e;border:1px solid #333;border-radius:4px;color:#fff;cursor:pointer;font-size:14px;min-width:36px;text-align:center}.g:disabled{background:#555;color:#333;cursor:default}.g.ok{background:#2e7d32;border-color:#4caf50}.g.err{background:#b71c1c;border-color:#ff5252}.g.run{background:#e65100;border-color:#ff9800}select{padding:8px;background:#16213e;border:1px solid #333;border-radius:4px;color:#fff;font-size:16px;width:100%}button{padding:12px 24px;border:none;border-radius:4px;color:#000;font-weight:bold;cursor:pointer;font-size:16px;margin:4px}#btn{background:#4fc3f7}#btn:disabled{background:#555;color:#333}.p{margin:8px 0;font-size:0.95em}.ok{color:#4caf50}.err{color:#ff5252}progress{width:100%;height:20px;margin:8px 0}#lst{margin:8px 0;font-size:0.8em;color:#aaa;max-height:300px;overflow-y:auto}</style></head><body><h1>MDB Dump Tool</h1><div class="p" id="pg2">Loading table list...</div><div id="groups"></div><hr><select id="tn"><option value="">-- loading --</option></select><button id="btn" onclick="goOne()" disabled>Dump 1 Table</button><div class="p" id="pg">Press a letter group to dump all tables in that group as one file</div><progress id="pb" value="0" max="100"></progress><div id="lst"></div><script>function safeJson(t){try{return JSON.parse(t)}catch(e){return JSON.parse(t.replace(/[\x00-\x1f]/g,function(c){return"\\u"+("0000"+c.charCodeAt(0).toString(16)).slice(-4)}))}}var tables=[];var groups={};async function loadTables(){try{var r=await fetch("/debug/mdb_all_tables");var j=safeJson(await r.text());if(!j.ok){document.getElementById("pg2").innerHTML=`<span class="err">Error: ${j.error||"unknown"}</span>`;return;}tables=j.all_tables||[];var sel=document.getElementById("tn");sel.innerHTML="";groups={};for(var i=0;i<tables.length;i++){var t=tables[i];var o=document.createElement("option");o.value=t.name;o.textContent=t.name+" ("+t.rows+")";sel.appendChild(o);var fl=t.name[0].toUpperCase();if(!groups[fl])groups[fl]=[];groups[fl].push(t);}document.getElementById("btn").disabled=false;document.getElementById("pg2").innerHTML=`<span class="ok">${tables.length} tables in ${Object.keys(groups).length} groups</span>`;renderGroups();}catch(e){document.getElementById("pg2").innerHTML=`<span class="err">Fetch error: ${e}</span>`;}}function renderGroups(){var div=document.getElementById("groups");div.innerHTML="";var keys=Object.keys(groups).sort();for(var k=0;k<keys.length;k++){var key=keys[k];var btn=document.createElement("button");btn.className="g";btn.textContent=key+" ("+groups[key].length+")";btn.setAttribute("data-key",key);btn.onclick=function(){goGroup(this.getAttribute("data-key"),this);};div.appendChild(btn);}}async function dumpTable(n,onProgress){var allRows=[];var off=0;var total=0;var batch=100;var done=false;while(!done){try{var r=await fetch("/debug/table?name="+n+"&limit="+batch+"&offset="+off);var j=safeJson(await r.text());if(!j.ok){return{ok:false,error:j.error||"unknown"};}total=j.row_count||total;var nr=j.rows?j.rows.length:0;if(nr===0){done=true;break;}allRows=allRows.concat(j.rows);off+=nr;if(onProgress)onProgress(off,total);done=off>=total||nr<batch;}catch(e){return{ok:false,error:""+e};}}return{ok:true,table:n,row_count:total,rows_merged:allRows.length,rows:allRows};}function downloadJson(data,filename){var result=JSON.stringify(data);var blob=new Blob([result],{type:"application/json"});var url=URL.createObjectURL(blob);var a=document.createElement("a");a.href=url;a.download=filename;a.click();URL.revokeObjectURL(url);}async function goGroup(key,btn){btn.disabled=true;btn.className="g run";var tbls=groups[key];var result={group:key,tables:{}};var log=document.getElementById("lst");log.innerHTML="";var ok=0,fail=0;for(var i=0;i<tbls.length;i++){var t=tbls[i];document.getElementById("pg").innerHTML=`<span class="ok">[${key}] ${(i+1)}/${tbls.length} ${t.name} (${t.rows} rows)...</span>`;document.getElementById("pb").value=Math.round((i+1)/tbls.length*100);if(t.rows===0){result.tables[t.name]={ok:true,rows:0,data:[]};log.innerHTML+=t.name+": skip (0)<br>";ok++;continue;}var res=await dumpTable(t.name);if(res.ok&&res.rows_merged>0){result.tables[t.name]={ok:true,row_count:res.row_count,rows_merged:res.rows_merged,rows:res.rows};log.innerHTML+=t.name+`: <span class="ok">${res.rows_merged}</span><br>`;ok++;}else{result.tables[t.name]={ok:false,error:res.error||"no rows"};log.innerHTML+=t.name+`: <span class="err">${res.error||"no rows"}</span><br>`;fail++;}}var fname="mdb_"+key.toLowerCase()+".json";downloadJson(result,fname);btn.className=ok>0&&fail===0?"g ok":"g err";btn.disabled=false;document.getElementById("pg").innerHTML=`<span class="ok">${key}: ${ok} OK, ${fail} fail -> ${fname}</span>`;document.getElementById("pb").value=0;}async function goOne(){var b=document.getElementById("btn");var n=document.getElementById("tn").value;if(!n)return;b.disabled=true;document.getElementById("pg").innerHTML=`<span class="ok">Dumping ${n}...</span>`;var res=await dumpTable(n,function(off,total){var pct=total>0?Math.round(off/total*100):0;document.getElementById("pb").value=pct;document.getElementById("pg").innerHTML="Dumping "+n+": "+off+"/"+total+" ("+pct+"%)";});if(res.ok&&res.rows_merged>0){downloadJson(res,n+".json");document.getElementById("pg").innerHTML=`<span class="ok">Done! ${res.rows_merged}/${res.row_count} -> ${n}.json</span>`;}else{document.getElementById("pg").innerHTML=`<span class="err">${res.error?"Error: "+res.error:"No rows found"}</span>`;}document.getElementById("pb").value=0;b.disabled=false;}loadTables();</script></body></html>"#.to_string();
        html
    } else if path == "/config.html" {
        // Serve a simple HTML form for config editing - open in any browser
        let cfg = unsafe { get_config() };
        let html = format!(
            r#"<!DOCTYPE html><html><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>URA Plugin Config</title><style>body{{font-family:system-ui;max-width:500px;margin:20px auto;padding:0 16px;background:#1a1a2e;color:#e0e0e0}}h1{{color:#4fc3f7;font-size:1.3em}}label{{display:block;margin:12px 0 4px;color:#aaa;font-size:0.85em}}input{{width:100%;padding:8px;background:#16213e;border:1px solid #333;border-radius:4px;color:#fff;box-sizing:border-box}}button{{margin-top:16px;padding:10px 24px;background:#4fc3f7;border:none;border-radius:4px;color:#000;font-weight:bold;cursor:pointer}}.ok{{color:#4caf50;margin-top:8px}}</style></head><body><h1>URA Plugin Config</h1><form id="f"><label>Push Host</label><input id="push_host" value="{}"><label>Push Port</label><input id="push_port" type="number" value="{}"><label>HTTP Port</label><input id="http_port" type="number" value="{}"><label>Push Interval (sec)</label><input id="push_interval_secs" type="number" value="{}" min="1"><label>Push Enabled</label><input id="push_enabled" type="checkbox" {}><label>HTTP Enabled</label><input id="http_enabled" type="checkbox" {}><button type="submit">Save</button></form><div id="r"></div><script>document.getElementById('f').onsubmit=async(e)=>{{e.preventDefault();const d={{push_host:push_host.value,push_port:+push_port.value,http_port:+http_port.value,push_interval_secs:+push_interval_secs.value,push_enabled:push_enabled.checked,http_enabled:http_enabled.checked}};const r=await fetch('/config',{{method:'POST',headers:{{'Content-Type':'application/json'}},body:JSON.stringify(d)}});const j=await r.json();document.getElementById('r').innerHTML=j.ok?'<p class="ok">Saved!</p>':'<p style="color:red">Error: '+j.error+'</p>';}};</script></body></html>"#,
            cfg.push_host,
            cfg.push_port,
            cfg.http_port,
            cfg.push_interval_secs,
            if cfg.push_enabled { "checked" } else { "" },
            if cfg.http_enabled { "checked" } else { "" }
        );
        // Return HTML with text/html content type (handled below)
        html
    } else if path.starts_with("/classes") {
        let search = if path == "/classes" || path == "/classes/" {
            ""
        } else {
            path.strip_prefix("/classes/search/")
                .or_else(|| path.strip_prefix("/classes/"))
                .unwrap_or("")
        };
        unsafe { enumerate_all_classes(search) }
    } else if path.starts_with("/mdb/schema") {
        // v3.22.89: 表结构
        let table_name = parse_query(&full_uri, "name");
        mdb_schema(&table_name)
    } else if path.starts_with("/mdb/search") {
        // v3.22.89: 搜索表名和列名
        let keyword = parse_query(&full_uri, "keyword");
        mdb_search(&keyword)
    } else if path.starts_with("/mdb/raw") {
        // v3.22.89: 执行只读SQL
        let sql = parse_query(&full_uri, "sql");
        mdb_raw_query(&sql)
    } else if path.starts_with("/mdb/dl_batch") {
        // ★ 按首字母批量下载 MDB 表数据为 JSON 文件
        // /mdb/dl_batch?prefix=a  → 下载所有 a 开头的表
        // /mdb/dl_batch?prefix=all → 下载全部表（可能很大）
        let prefix = parse_query(&full_uri, "prefix");
        let body = mdb_dl_batch(&prefix);
        let safe_prefix: String = prefix.chars().filter(|c| c.is_alphanumeric()).collect();
        let fname = format!("mdb_{}.json", if safe_prefix.is_empty() { "ALL" } else { &safe_prefix });
        let resp = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Disposition: attachment; filename=\"{}\"\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            fname, body.len(), body
        );
        let _ = stream.write_all(resp.as_bytes());
        return;
    } else if path.starts_with("/il2cpp/dump_all_methods_dl") {
        // v3.22.91: 暴力dump全部类方法目录（下载JSON，修复：内联下载包装）
        let letter = parse_query(&full_uri, "letter");
        let body = unsafe { il2cpp_dump_all_methods(&letter) };
        let safe_letter: String = letter.chars().filter(|c| c.is_alphanumeric()).collect();
        let fname = format!(
            "dump_all_methods_{}.json",
            if safe_letter.is_empty() {
                "ALL"
            } else {
                &safe_letter
            }
        );
        let resp = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Disposition: attachment; filename=\"{}\"\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            fname, body.len(), body
        );
        let _ = stream.write_all(resp.as_bytes());
        return;
    } else if path.starts_with("/il2cpp/dump_all_methods") {
        // v3.22.89: 暴力dump全部类方法目录（按letter分组避免手机卡死）
        let letter = parse_query(&full_uri, "letter");
        unsafe { il2cpp_dump_all_methods(&letter) }
    } else if path.starts_with("/il2cpp/dump") {
        // v3.22.89: dump单例实例（带运行时值）
        let class_name = parse_query(&full_uri, "name");
        unsafe { il2cpp_dump_singleton(&class_name) }
    } else if path.starts_with("/il2cpp/call") {
        // v3.22.89: 调用单例上的getter方法
        let class_name = parse_query(&full_uri, "class");
        let method_name = parse_query(&full_uri, "method");
        unsafe { il2cpp_call_method(&class_name, &method_name) }
    } else if path.starts_with("/il2cpp/tree") {
        // v3.22.89: 递归dump引用类型字段
        let class_name = parse_query(&full_uri, "name");
        let depth_str = parse_query(&full_uri, "depth");
        let depth = depth_str.parse::<usize>().unwrap_or(2);
        unsafe { il2cpp_tree_dump(&class_name, depth) }
    } else if path.starts_with("/il2cpp/field") {
        // v3.22.89: 读取单例的指定字段值
        let class_name = parse_query(&full_uri, "class");
        let field_name = parse_query(&full_uri, "field");
        unsafe { il2cpp_read_single_field(&class_name, &field_name) }
    } else if path.starts_with("/il2cpp/classes") {
        // v3.22.89: 搜索IL2CPP类名（方案A）
        let keyword = parse_query(&full_uri, "keyword");
        unsafe { il2cpp_search_classes(&keyword) }
    } else if path.starts_with("/il2cpp/static") {
        // v3.22.89: 读取静态类常量值（方案B）
        let class_name = parse_query(&full_uri, "name");
        unsafe { il2cpp_read_static_fields(&class_name) }
    } else if path.starts_with("/il2cpp/methods") {
        // v3.22.89: 列出类的所有方法名和参数数量
        let class_name = parse_query(&full_uri, "name");
        unsafe { il2cpp_list_methods(&class_name) }
    } else if path.starts_with("/il2cpp/disassemble_dl") {
        // v3.22.89: 反汇编结果下载JSON文件（手机浏览器复制上限对策）
        let class_name = parse_query(&full_uri, "class");
        let method_name = parse_query(&full_uri, "method");
        let bytes_str = parse_query(&full_uri, "bytes");
        let bytes_limit = bytes_str.parse::<usize>().unwrap_or(2048);
        unsafe { il2cpp_disassemble(&class_name, &method_name, bytes_limit) }
    } else if path.starts_with("/il2cpp/disassemble_addr_dl") {
        // v3.22.91: 按地址反汇编结果下载JSON文件（修复：内联下载包装，避免被starts_with截胡）
        let addr_str = parse_query(&full_uri, "addr");
        let bytes_str = parse_query(&full_uri, "bytes");
        let bytes_limit = bytes_str.parse::<usize>().unwrap_or(2048);
        let body = unsafe { il2cpp_disassemble_addr(&addr_str, bytes_limit) };
        let safe_addr: String = addr_str.chars().filter(|c| c.is_alphanumeric()).collect();
        let fname = format!(
            "disassemble_addr_{}.json",
            if safe_addr.is_empty() {
                "output"
            } else {
                &safe_addr
            }
        );
        let resp = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Disposition: attachment; filename=\"{}\"\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            fname, body.len(), body
        );
        let _ = stream.write_all(resp.as_bytes());
        return;
    } else if path.starts_with("/il2cpp/disassemble_addr") {
        // v3.22.89: 按地址反汇编ARM64指令体（分析ExecTraining等方法的子函数调用目标）
        let addr_str = parse_query(&full_uri, "addr");
        let bytes_str = parse_query(&full_uri, "bytes");
        let bytes_limit = bytes_str.parse::<usize>().unwrap_or(2048);
        unsafe { il2cpp_disassemble_addr(&addr_str, bytes_limit) }
    } else if path.starts_with("/il2cpp/disassemble") {
        // v3.22.89: 反汇编IL2CPP方法的ARM64指令体
        let class_name = parse_query(&full_uri, "class");
        let method_name = parse_query(&full_uri, "method");
        let bytes_str = parse_query(&full_uri, "bytes");
        let bytes_limit = bytes_str.parse::<usize>().unwrap_or(2048);
        unsafe { il2cpp_disassemble(&class_name, &method_name, bytes_limit) }
    } else if path.starts_with("/il2cpp/search_int_dl") {
        // v3.22.91: 搜索整数千分比（下载JSON，修复：内联下载包装）
        let values_str = parse_query(&full_uri, "values");
        let body = unsafe { il2cpp_search_int(&values_str) };
        let safe_vals: String = values_str
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == ',')
            .collect();
        let fname = format!(
            "search_int_{}.json",
            if safe_vals.is_empty() {
                "all".into()
            } else {
                safe_vals
            }
        );
        let resp = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Disposition: attachment; filename=\"{}\"\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            fname, body.len(), body
        );
        let _ = stream.write_all(resp.as_bytes());
        return;
    } else if path.starts_with("/il2cpp/search_int") {
        // v3.22.89: 搜索整数千分比
        let values_str = parse_query(&full_uri, "values");
        unsafe { il2cpp_search_int(&values_str) }
    } else if path.starts_with("/il2cpp/search_float_dl") {
        // v3.22.93: 搜索浮点常量（下载JSON，与search_int_dl对称）
        let value_str = parse_query(&full_uri, "value");
        let body = unsafe { il2cpp_search_float(&value_str) };
        let safe_val: String = value_str
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '.' || *c == '-')
            .collect();
        let fname = format!(
            "search_float_{}.json",
            if safe_val.is_empty() {
                "all".into()
            } else {
                safe_val
            }
        );
        let resp = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Disposition: attachment; filename=\"{}\"\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            fname, body.len(), body
        );
        let _ = stream.write_all(resp.as_bytes());
        return;
    } else if path.starts_with("/il2cpp/search_float") {
        // v3.22.89: 在代码段搜索浮点常量（方案D）
        let value_str = parse_query(&full_uri, "value");
        unsafe { il2cpp_search_float(&value_str) }
    } else if path.starts_with("/il2cpp/read_mem_dl") {
        // v3.22.91: 读取原始内存（下载hex dump，修复：内联下载包装）
        let addr_str = parse_query(&full_uri, "addr");
        let size_str = parse_query(&full_uri, "size");
        let body = il2cpp_read_mem(&addr_str, &size_str);
        let safe_addr: String = addr_str.chars().filter(|c| c.is_alphanumeric()).collect();
        let fname = format!(
            "read_mem_{}.txt",
            if safe_addr.is_empty() {
                "output"
            } else {
                &safe_addr
            }
        );
        let resp = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Disposition: attachment; filename=\"{}\"\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            fname, body.len(), body
        );
        let _ = stream.write_all(resp.as_bytes());
        return;
    } else if path.starts_with("/il2cpp/read_mem") {
        // v3.22.89: 读取原始内存（hex dump）
        let addr_str = parse_query(&full_uri, "addr");
        let size_str = parse_query(&full_uri, "size");
        il2cpp_read_mem(&addr_str, &size_str)
    } else if path == "/il2cpp/search_methods_page" {
        // v3.22.89: 搜索方法名HTML页面（A-Z分组）
        search_methods_page()
    } else if path.starts_with("/il2cpp/search_methods_dl") {
        // v3.22.91: 跨类搜索方法名（下载JSON，修复：内联下载包装）
        let keyword = parse_query(&full_uri, "keyword");
        let letter = parse_query(&full_uri, "letter");
        let body = unsafe { il2cpp_search_methods(&keyword, &letter) };
        let kw = &keyword;
        let safe_kw: String = kw
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        let fname = format!(
            "search_methods_{}.json",
            if safe_kw.is_empty() {
                "all".into()
            } else {
                safe_kw
            }
        );
        let resp = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Disposition: attachment; filename=\"{}\"\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            fname, body.len(), body
        );
        let _ = stream.write_all(resp.as_bytes());
        return;
    } else if path.starts_with("/il2cpp/search_methods") {
        // v3.22.89: 跨类搜索方法名关键词
        let keyword = parse_query(&full_uri, "keyword");
        let letter = parse_query(&full_uri, "letter");
        unsafe { il2cpp_search_methods(&keyword, &letter) }
    } else if path == "/mdb" {
        // v3.22.51: Serve raw MasterDB file for client-side processing
        // Uses marker string; binary file sent in response handler below
        match find_mdb_path() {
            Some(mdb_path) => format!("__MDB_BINARY__{}", mdb_path),
            None => r#"{"error":"mdb_not_found"}"#.to_string(),
        }
    } else {
        format!(
            r#"{{"error":"not_found","path":"{}","available":["/scan","/data","/status","/health","/scenario","/debug/upload","/debug/rameninfo","/debug/laststep","/event/recommend","/inherit/compat","/saddle-analysis","/log/turn","/log","/debug/params","/fields","/methods","/singletons","/find_method","/classes","/carddb","/skilldata","/hall","/debug/breeders","/debug/cmdinfo","/debug/training_partners","/debug/ramengains","/debug/paramsincdec","/debug/training_seed","/debug/training_log","/debug/training_log_dl","/update","/update/status","/debug/dumpclass","/debug/storydata","/debug/ramenfields","/debug/all","/mdb","/debug/push_table","/debug/download_table","/classes/search/keyword","/mdb/schema","/mdb/search","/mdb/raw","/mdb/dl_batch","/il2cpp/dump","/il2cpp/call","/il2cpp/tree","/il2cpp/field","/il2cpp/classes","/il2cpp/static","/il2cpp/methods","/il2cpp/search_float","/il2cpp/search_float_dl","/il2cpp/search_int","/il2cpp/search_int_dl","/il2cpp/search_methods","/il2cpp/search_methods_dl","/il2cpp/search_methods_page","/il2cpp/read_mem","/il2cpp/read_mem_dl","/training/result","/api/sniff","/api/sniff/toggle","/api/sniff/clear","/api/sniff/diag","/api/event/choices","/api/event/clear"]}}"#,
            path
        )
    };

    save_endpoint_log(&path, &body);

    if body.starts_with("__MDB_BINARY__") {
        // v3.22.51: Serve raw mdb file as binary response
        let mdb_path = &body[14..]; // skip "__MDB_BINARY__"
        match std::fs::read(mdb_path) {
            Ok(data) => {
                let header = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Disposition: attachment; filename=\"master.mdb\"\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    data.len()
                );
                let _ = stream.write_all(header.as_bytes());
                // Write in chunks to avoid memory spike
                for chunk in data.chunks(65536) {
                    let _ = stream.write_all(chunk);
                }
            }
            Err(e) => {
                let err_json = format!(r#"{{"error":"mdb_read_failed","detail":"{}"}}"#, e);
                let resp = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    err_json.len(), err_json
                );
                let _ = stream.write_all(resp.as_bytes());
            }
        }
    } else if path == "/saddles-dl" {
        let resp = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Disposition: attachment; filename=\"saddles.json\"\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(), body
        );
        let _ = stream.write_all(resp.as_bytes());
    } else if path == "/il2cpp/disassemble_dl" {
        // v3.22.89: 反汇编结果下载为JSON文件
        let cn = parse_query(&full_uri, "class");
        let mn = parse_query(&full_uri, "method");
        let safe_name: String = format!(
            "{}_{}",
            cn.chars()
                .filter(|c| c.is_alphanumeric() || *c == '_')
                .collect::<String>(),
            mn.chars()
                .filter(|c| c.is_alphanumeric() || *c == '_')
                .collect::<String>()
        );
        let fname = format!(
            "disassemble_{}.json",
            if safe_name.is_empty() {
                "output"
            } else {
                &safe_name
            }
        );
        let resp = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Disposition: attachment; filename=\"{}\"\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            fname, body.len(), body
        );
    } else {
        let content_type = if body.starts_with("<!DOCTYPE") || body.starts_with("<html") {
            "text/html; charset=utf-8"
        } else {
            "application/json"
        };
        if dl_enabled {
            // 下载模式：默认按路由生成文件名，?name= 可覆盖
            let safe: String = dl_name
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
                .take(64)
                .collect();
            let fallback = path.trim_matches('/').replace('/', "_");
            let base = if safe.is_empty() { fallback } else { safe };
            let base = if base.is_empty() { "download".to_string() } else { base };
            let ext = if content_type.starts_with("text/html") { "html" } else { "json" };
            let fname = format!("{}.{}", base, ext);
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Disposition: attachment; filename=\"{}\"\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                fname, body.len(), body
            );
            let _ = stream.write_all(resp.as_bytes());
        } else {
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                content_type, body.len(), body
            );
            let _ = stream.write_all(resp.as_bytes());
        }
    }
    let _ = stream.flush();
}

// ============================================================
// v3.22.51: Pre-cache all class metadata on game thread
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
    if class.is_null() {
        return;
    }
    let get_fields_fn: Option<FnClassGetFields> = {
        let p = resolve_il2cpp_symbol("il2cpp_class_get_fields");
        if p.is_null() {
            None
        } else {
            Some(std::mem::transmute::<*mut c_void, FnClassGetFields>(p))
        }
    };
    let get_parent_fn: Option<FnClassGetParent> = {
        let p = resolve_il2cpp_symbol("il2cpp_class_get_parent");
        if p.is_null() {
            None
        } else {
            Some(std::mem::transmute::<*mut c_void, FnClassGetParent>(p))
        }
    };
    if get_fields_fn.is_none() {
        return;
    }

    let mut current_class = class;
    let mut depth = 0;
    loop {
        if current_class.is_null() || depth > 10 {
            break;
        }
        let mut iter: *mut c_void = ptr::null_mut();
        loop {
            let field_info = get_fields_fn.unwrap()(current_class, &mut iter);
            if field_info.is_null() {
                break;
            }
            if !(*field_info).name.is_null() {
                let s = std::ffi::CStr::from_ptr((*field_info).name);
                let fname = s.to_string_lossy().to_string();
                let offset = (*field_info).offset;
                // Extract property name from <PropName>k__BackingField
                let prop_name = if fname.starts_with('<') {
                    if let Some(end) = fname.find('>') {
                        &fname[1..end]
                    } else {
                        &fname
                    }
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
                    if guard.is_none() {
                        *guard = Some(HashMap::new());
                    }
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
            if parent.is_null() || parent == current_class {
                break;
            }
            current_class = parent;
        } else {
            break;
        }
        depth += 1;
    }
}

/// Pre-cache all known classes and field offsets on game thread
unsafe fn precache_metadata() {
    ura_log(2, "v3.22.51 precache_metadata: starting");
    let image = match get_image() {
        img if !img.is_null() => img,
        _ => {
            ura_log(1, "precache_metadata: image null");
            return;
        }
    };

    // Classes found via find_class(image, "Gallop", X)
    let gallop_classes = [
        "WorkDataManager",
        "WorkSingleModeData",
        "WorkSingleModeCharaData",
        "WorkSingleModeHomeInfoData",
        "WorkSingleModeScenarioRamen",
        "WorkSingleModeScenarioURA",
        "WorkSingleModeScenarioTeamRace",
        "WorkSingleModeScenarioLive",
        "WorkSingleModeScenarioFree",
        "WorkSingleModeScenarioVenus",
        "WorkSingleModeScenarioArc",
        "WorkSingleModeScenarioSport",
        "WorkSingleModeScenarioCook",
        "WorkSingleModeScenarioMecha",
        "WorkSingleModeScenarioLegend",
        "WorkSingleModeScenarioPioneer",
        "WorkSingleModeScenarioOnsen",
        "WorkSingleModeScenarioBreeders",
    ];

    // Classes found via find_class_by_short_name
    let short_name_classes = [
        "SingleModeSkillData",
        "SingleModeCommandInfoData",
        "SingleModeParamsIncDecInfoData",
        "ObscuredSingleModeBreedersEnhanceGroup",
        "ObscuredSingleModeBreedersCommandInfo",
        "WorkSingleModeScenarioRamenDataSet",
        "ObscuredSingleModeRamenFeeling",
        "ObscuredSingleModeRamenFeelingTurnInfo",
        "ObscuredSingleModeRamenCommandFeelingInfo",
        "ObscuredSingleModeRamenFeelingReduceTurnInfo",
        "ObscuredSingleModeRamenUrafEffectInfo",
        "ObscuredSingleModeRamenActiveEffectInfo",
        "WorkTrainedCharaData",
        "TrainedCharaData",
        "SuccessionCharaInfo",
        "WorkSingleModeScenarioURADataSet",
        "WorkSingleModeScenarioTeamRaceDataSet",
        "WorkSingleModeScenarioLiveDataSet",
        "WorkSingleModeScenarioFreeDataSet",
        "WorkSingleModeScenarioVenusDataSet",
        "WorkSingleModeScenarioArcDataSet",
        "WorkSingleModeScenarioSportDataSet",
        "WorkSingleModeScenarioCookDataSet",
        "WorkSingleModeScenarioMechaDataSet",
        "WorkSingleModeScenarioLegendDataSet",
        "WorkSingleModeScenarioPioneerDataSet",
        "WorkSingleModeScenarioOnsenDataSet",
        "WorkSingleModeScenarioBreedersDataSet",
    ];

    let mut cached_count = 0i32;

    // Cache Gallop namespace classes
    for name in &gallop_classes {
        let cls = find_class(image, to_cstr("Gallop").as_ptr(), to_cstr(name).as_ptr());
        if !cls.is_null() {
            if let Ok(mut guard) = CLASS_CACHE.lock() {
                if guard.is_none() {
                    *guard = Some(HashMap::new());
                }
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
                if guard.is_none() {
                    *guard = Some(HashMap::new());
                }
                if let Some(ref mut map) = *guard {
                    map.insert(name.to_string(), cls as usize);
                }
            }
            precache_all_fields(cls);
            cached_count += 1;
        }
    }

    // Cache WorkDataManager singleton
    if let Some(wdm_cls) = CLASS_CACHE
        .lock()
        .ok()
        .and_then(|g| g.as_ref().and_then(|m| m.get("WorkDataManager").copied()))
    {
        let wdm_ptr = wdm_cls as *mut c_void;
        let inst = get_singleton(wdm_ptr);
        if !inst.is_null() {
            if let Ok(mut guard) = SINGLETON_CACHE.lock() {
                if guard.is_none() {
                    *guard = Some(HashMap::new());
                }
                if let Some(ref mut map) = *guard {
                    map.insert(wdm_cls, inst as usize);
                }
            }
            ura_log(
                2,
                &format!("precache_metadata: WDM singleton cached at {:p}", inst),
            );
        }
    }

    // Count cached field offsets
    let field_count = FIELD_OFFSET_CACHE
        .lock()
        .ok()
        .and_then(|g| g.as_ref().map(|m| m.len()))
        .unwrap_or(0);

    ura_log(
        2,
        &format!(
            "v3.22.51 precache_metadata: done — {} classes, {} field offsets cached",
            cached_count, field_count
        ),
    );
}

// ============================================================
// Menu Callbacks
// ============================================================

extern "C" fn on_menu_item_click(_userdata: *mut c_void) {
    unsafe {
        ura_log(3, "URA menu item clicked");
    }
}

// ★ v3.22.94: Training result hook — intercept OnSuccessSendCommand to read resultType
// Write ARM64 hook bytes (LDR X16, [PC, #8]; BR X16; .quad handler)
// ★ v3.24.9: Flush I-Cache after writing — ARM64 has separate I/D cache
unsafe fn write_hook_bytes(target_addr: usize, handler_addr: usize) {
    let page_size = 4096;
    let page_addr = target_addr & !(page_size - 1);
    libc::mprotect(
        page_addr as *mut libc::c_void,
        page_size,
        libc::PROT_READ | libc::PROT_WRITE | libc::PROT_EXEC,
    );
    let hook: [u32; 4] = [
        0x58000050,                  // LDR X16, [PC, #8]
        0xD61F0200,                  // BR X16
        handler_addr as u32,         // .quad (low 32 bits)
        (handler_addr >> 32) as u32, // .quad (high 32 bits)
    ];
    std::ptr::copy_nonoverlapping(hook.as_ptr(), target_addr as *mut u32, 4);

    // ★ v3.24.9: Flush I-Cache for the modified region
    // ARM64 has separate L1 I-Cache and D-Cache. After writing to .text via D-Cache,
    // we must flush I-Cache or CPU may execute stale instructions.
    // Use libc::syscall to call __ARM_NR_cacheflush (ARM-specific syscall)
    // On ARM64 the syscall number for cache flush is 0x0b (0xf0002 in Linux)
    // Alternatively use inline assembly as fallback
    #[cfg(target_arch = "aarch64")]
    {
        unsafe {
            // ARM64: use inline asm to flush I-cache
            // DSB ISH ensures data writes are visible to all cores
            // IC IALLU invalidates entire I-cache (to Point of Unification)
            // ISB forces pipeline reload
            ::std::arch::asm!("dsb ish", "ic iallu", "dsb ish", "isb", options(nostack),);
        }
    }
}

// ★ v3.24.9: Training result hook — rewritten to use interceptor API
// Old write_hook_bytes was thread-unsafe (unhook→call orig→rehook race condition)
// New version uses interceptor_hook/interceptor_get_trampoline (trampoline-based, thread-safe)
// ★ v3.24.9: catch_unwind on all hook handlers — panic must never cross FFI boundary
extern "C" fn training_hook_handler(
    this: *mut c_void,
    turn_info: *mut c_void,
    sub_id: i32,
    result_type: i32,
) -> *mut c_void {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        unsafe {
            LAST_TRAINING_RESULT = result_type;
            LAST_TRAINING_SUB_ID = sub_id;

            // ★ v2.2: 记录真实 command_id 和 sequence
            // sub_id 可能是训练子命令 ID，记录用于动作识别
            {
                let _lock = LAST_ACTION_MUTEX.lock();
                LAST_ACTION_COMMAND_ID = sub_id;
                LAST_ACTION_SEQUENCE += 1;
            }

            // Use trampoline — no unhook/rehook needed
            let trampoline = interceptor_get_trampoline(training_hook_handler as usize);
            if trampoline == 0 {
                ura_log(1, "training_hook: trampoline not found");
                return std::ptr::null_mut();
            }
            type FnType = unsafe extern "C" fn(*mut c_void, *mut c_void, i32, i32) -> *mut c_void;
            let original: FnType = std::mem::transmute(trampoline);
            original(this, turn_info, sub_id, result_type)
        }
    }))
    .unwrap_or_else(|e| {
        unsafe {
            ura_log(1, &format!("training_hook: panic caught: {:?}", e));
        }
        std::ptr::null_mut()
    })
}

unsafe fn find_method_addr(class: *mut c_void, method_name: &str, _param_count: i32) -> usize {
    let get_methods_fn: Option<
        unsafe extern "C" fn(*mut c_void, *mut *mut c_void) -> *const c_void,
    > = {
        let p = resolve_il2cpp_symbol("il2cpp_class_get_methods");
        if p.is_null() {
            None
        } else {
            Some(std::mem::transmute(p))
        }
    };
    let method_get_name_fn: Option<unsafe extern "C" fn(*const c_void) -> *const c_char> = {
        let p = resolve_il2cpp_symbol("il2cpp_method_get_name");
        if p.is_null() {
            None
        } else {
            Some(std::mem::transmute(p))
        }
    };
    if get_methods_fn.is_none() || method_get_name_fn.is_none() {
        return 0;
    }

    let mut iter: *mut c_void = std::ptr::null_mut();
    loop {
        let mi = get_methods_fn.unwrap()(class, &mut iter);
        if mi.is_null() {
            break;
        }
        let name_ptr = method_get_name_fn.unwrap()(mi);
        if !name_ptr.is_null() {
            let name = CStr::from_ptr(name_ptr).to_string_lossy();
            if name == method_name {
                // v3.23.3: (legacy fallback - unused) (official API)
                let method_get_ptr_fn: Option<
                    unsafe extern "C" fn(*const c_void) -> *const c_void,
                > = {
                    let p = resolve_il2cpp_symbol("il2cpp_method_get_pointer");
                    if p.is_null() {
                        None
                    } else {
                        Some(std::mem::transmute(p))
                    }
                };
                if let Some(get_ptr) = method_get_ptr_fn {
                    let ptr = get_ptr(mi);
                    if !ptr.is_null() {
                        let addr = ptr as usize;
                        ura_log(
                            3,
                            &format!(
                                "find_method_addr: {} via get_pointer -> 0x{:x}",
                                method_name, addr
                            ),
                        );
                        return addr;
                    }
                }
                // Fallback: manual offset reading
                let method_ptr =
                    std::ptr::read_unaligned::<*const c_void>(mi as *const *const c_void);
                let addr = method_ptr as usize;
                if addr == 0 {
                    let union_ptr = std::ptr::read_unaligned::<*const c_void>(
                        (mi as *const u8).offset(48) as *const *const c_void,
                    );
                    let union_addr = union_ptr as usize;
                    ura_log(
                        3,
                        &format!(
                            "find_method_addr: {} offset0=0, offset48=0x{:x}",
                            method_name, union_addr
                        ),
                    );
                    return union_addr;
                }
                ura_log(
                    3,
                    &format!(
                        "find_method_addr: {} manual offset -> 0x{:x}",
                        method_name, addr
                    ),
                );
                return addr;
            }
        }
    }
    ura_log(
        3,
        &format!("find_method_addr: {} NOT FOUND in class", method_name),
    );
    0
}

unsafe fn install_training_hook() {
    if TRAINING_HOOK_INSTALLED {
        return;
    }
    if API.is_null() {
        return;
    }

    let image = match get_image() {
        img if !img.is_null() => img,
        _ => return,
    };

    let class = find_class_by_short_name(image, "SingleModeMainTrainingCuttController");
    if class.is_null() {
        return;
    }

    let method_addr = find_method_addr(class, "OnSuccessSendCommand", 3);
    if method_addr == 0 {
        return;
    }

    ON_SUCCESS_ADDR = method_addr;

    // ★ v3.24.9: Use interceptor API instead of write_hook_bytes
    if interceptor_hook(method_addr, training_hook_handler as usize) {
        TRAINING_HOOK_INSTALLED = true;
        ura_log(
            3,
            &format!(
                "Training hook installed at 0x{:x} (interceptor)",
                method_addr
            ),
        );
    } else {
        ura_log(
            1,
            "Training hook: interceptor_hook failed, falling back to write_hook_bytes",
        );
        // Fallback: old write_hook_bytes method (less safe but works without interceptor)
        std::ptr::copy_nonoverlapping(
            method_addr as *const u8,
            ORIG_ON_SUCCESS_PROLOGUE.as_mut_ptr(),
            16,
        );
        write_hook_bytes(method_addr, training_hook_handler as usize);
        TRAINING_HOOK_INSTALLED = true;
    }
}

// ★ v3.23.3: API sniffing — read IL2CPP byte array
// IL2CPP array layout: klass(8) + monitor(8) + bounds(8) + max_length(8) + data
unsafe fn read_il2cpp_byte_array(arr: *const c_void) -> Vec<u8> {
    if arr.is_null() {
        return vec![];
    }
    let len = std::ptr::read::<u64>((arr as *const u8).offset(24) as *const u64) as usize;
    if len == 0 || len > 2 * 1024 * 1024 {
        return vec![];
    }
    let cap = len.min(65536);
    let data_ptr = (arr as *const u8).offset(32);
    std::slice::from_raw_parts(data_ptr, cap).to_vec()
}

fn sniff_timestamp() -> u64 {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

fn hex_encode(data: &[u8]) -> String {
    let mut s = String::with_capacity(data.len() * 2);
    for b in data {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

// ★ v3.23.3: Interceptor helpers — use Hachimi-Edge V3 interceptor API
unsafe fn interceptor_hook(orig_addr: usize, hook_addr: usize) -> bool {
    if API.is_null() || orig_addr == 0 || hook_addr == 0 {
        return false;
    }
    let api = &*API;
    if api.interceptor == 0 {
        return false;
    }
    if let Some(f) = api.interceptor_hook_fn {
        !f(
            api.interceptor,
            orig_addr as *mut c_void,
            hook_addr as *mut c_void,
        )
        .is_null()
    } else {
        false
    }
}

unsafe fn interceptor_get_trampoline(hook_addr: usize) -> usize {
    if API.is_null() || hook_addr == 0 {
        return 0;
    }
    let api = &*API;
    if api.interceptor == 0 {
        return 0;
    }
    if let Some(f) = api.interceptor_get_trampoline_addr_fn {
        f(api.interceptor, hook_addr as *mut c_void) as usize
    } else {
        0
    }
}

/// ★ v3.24.9: Unified hook installer — tries interceptor first, falls back to write_hook_bytes
unsafe fn install_hook_safe(
    name: &str,
    method_addr: usize,
    handler_addr: usize,
    orig_prologue: &mut [u8; 16],
) -> bool {
    if method_addr == 0 {
        return false;
    }
    if interceptor_hook(method_addr, handler_addr) {
        ura_log(
            3,
            &format!("{}: hooked at 0x{:x} (interceptor)", name, method_addr),
        );
        true
    } else {
        ura_log(
            2,
            &format!("{}: interceptor failed, fallback to write_hook_bytes", name),
        );
        std::ptr::copy_nonoverlapping(method_addr as *const u8, orig_prologue.as_mut_ptr(), 16);
        write_hook_bytes(method_addr, handler_addr);
        true
    }
}

// ★ v3.23.3: Hook handler for CompressRequest(byte[] data) -> byte[]
// Parks the uncompressed request body, keyed by the compressed byte array returned by the original.
// WWWRequest.Post will match it later.
extern "C" fn compress_request_hook_handler(data: *mut c_void) -> *mut c_void {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
        let body = read_il2cpp_byte_array(data);
        let trampoline = interceptor_get_trampoline(compress_request_hook_handler as usize);
        if trampoline == 0 {
            return std::ptr::null_mut();
        }
        type FnType = unsafe extern "C" fn(*mut c_void) -> *mut c_void;
        let original: FnType = std::mem::transmute(trampoline);
        let compressed = original(data);
        if !body.is_empty() && POST_ADDR != 0 {
            PENDING_REQ_BODY = Some(body);
            PENDING_COMPRESSED = compressed as usize;
        }
        compressed
    }));
    result.unwrap_or_else(|e| {
        unsafe {
            ura_log(1, &format!("compress_hook panic: {:?}", e));
        }
        std::ptr::null_mut()
    })
}

// ★ v3.23.3: Hook handler for DecompressResponse(byte[] data) -> byte[]
// Forwards the decompressed response body with the matching request's URL + headers.
extern "C" fn decompress_response_hook_handler(data: *mut c_void) -> *mut c_void {
    unsafe {
        let trampoline = interceptor_get_trampoline(decompress_response_hook_handler as usize);
        if trampoline == 0 {
            return std::ptr::null_mut();
        }
        type FnType = unsafe extern "C" fn(*mut c_void) -> *mut c_void;
        let original: FnType = std::mem::transmute(trampoline);
        let decompressed = original(data);
        if SNIFF_ENABLED.load(Ordering::Relaxed) {
            let bytes = read_il2cpp_byte_array(decompressed);
            if !bytes.is_empty() {
                let _lock = SNIFF_MUTEX.lock();
                let rid = PENDING_REQ_ID;
                SNIFF_RESPONSES.push((rid, bytes));
                if SNIFF_RESPONSES.len() > SNIFF_MAX {
                    SNIFF_RESPONSES.remove(0);
                }
            }
        }
        decompressed
    }
}

// ★ v3.23.3: Hook handler for WWWRequest.Post(this, url, postData, headers)
// Captures URL + headers directly, and matches the parked request body from CompressRequest.
// This replaces the old _Send + SetHeader approach.
extern "C" fn post_hook_handler(
    this: *mut c_void,
    url: *const c_void,
    post_data: *mut c_void,
    headers: *mut c_void,
) -> *mut c_void {
    unsafe {
        let trampoline = interceptor_get_trampoline(post_hook_handler as usize);
        if trampoline == 0 {
            return std::ptr::null_mut();
        }
        type FnType = unsafe extern "C" fn(
            *mut c_void,
            *const c_void,
            *mut c_void,
            *mut c_void,
        ) -> *mut c_void;
        let original: FnType = std::mem::transmute(trampoline);

        // Capture URL
        let game_url = if !url.is_null() {
            read_il2cpp_string(url)
        } else {
            String::new()
        };
        let game_url = if game_url.is_empty() {
            None
        } else {
            Some(game_url)
        };

        // Capture headers from Dictionary<string,string>
        let req_headers = read_string_dict(headers);

        if SNIFF_ENABLED.load(Ordering::Relaxed) {
            let rid = SNIFF_REQ_ID.fetch_add(1, Ordering::Relaxed);
            PENDING_REQ_ID = rid;
            // Try to match parked request body
            if let Some(body) = PENDING_REQ_BODY.take() {
                let headers_json = format_headers_json(&req_headers);
                let url_str = game_url.clone().unwrap_or_default();
                let _lock = SNIFF_MUTEX.lock();
                SNIFF_REQUESTS.push((rid, url_str, headers_json, body));
                if SNIFF_REQUESTS.len() > SNIFF_MAX {
                    SNIFF_REQUESTS.remove(0);
                }
            }
            PENDING_URL = game_url.clone().unwrap_or_default();
            PENDING_HEADERS = req_headers.clone();
        }

        let _ = this;
        original(this, url, post_data, headers)
    }
}

// ★ v3.23.3: Read IL2CPP Dictionary<string,string> into Vec<(String,String)>
// Layout: [hdr 0x10][fields...]; _entries @+0x18, _count @+0x20
// Entry: [hashCode:i32][next:i32][key:ptr][value:ptr] = 24B per entry
unsafe fn read_string_dict(dict: *mut c_void) -> Vec<(String, String)> {
    let mut out = Vec::new();
    if dict.is_null() {
        return out;
    }
    let count = std::ptr::read_unaligned::<i32>((dict as *const u8).add(0x20) as *const i32);
    if count <= 0 {
        return out;
    }
    let entries = std::ptr::read_unaligned::<usize>((dict as *const u8).add(0x18) as *const usize);
    if entries == 0 {
        return out;
    }
    // Il2CppArray header: 0x20 bytes, then entries
    let capacity =
        std::ptr::read_unaligned::<usize>((entries as *const u8).add(0x18) as *const usize);
    let entries_base = entries + 0x20;
    for i in 0..capacity {
        let entry_addr = entries_base + i * 24;
        let hash_code = std::ptr::read_unaligned::<i32>((entry_addr as *const u8) as *const i32);
        if hash_code < 0 {
            continue;
        } // free entry
        let key =
            std::ptr::read_unaligned::<usize>((entry_addr as *const u8).add(8) as *const usize);
        let value =
            std::ptr::read_unaligned::<usize>((entry_addr as *const u8).add(16) as *const usize);
        let key_str = read_il2cpp_string(key as *const c_void);
        let val_str = read_il2cpp_string(value as *const c_void);
        out.push((key_str, val_str));
        if out.len() >= count as usize {
            break;
        }
    }
    out
}

// Format headers Vec to JSON string: {"key1":"val1","key2":"val2"}
unsafe fn format_headers_json(headers: &[(String, String)]) -> String {
    if headers.is_empty() {
        return "{}".to_string();
    }
    let mut s = String::from("{");
    for (i, (k, v)) in headers.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        let v_escaped = v.replace('\\', "\\\\").replace('"', "\\\"");
        s.push_str(&format!("\"{}\":\"{}\"", k, v_escaped));
    }
    s.push('}');
    s
}

unsafe fn install_api_sniff_hooks() {
    let all_hooked = COMPRESS_REQUEST_ADDR != 0 && DECOMPRESS_RESPONSE_ADDR != 0 && POST_ADDR != 0;
    if all_hooked {
        return;
    }
    if API.is_null() {
        ura_log(3, "API sniff: API is null");
        return;
    }
    let api = &*API;
    if api.interceptor == 0 {
        ura_log(3, "API sniff: interceptor not available");
        return;
    }

    // Get umamusume.dll assembly image
    let get_asm = match api.il2cpp_get_assembly_image_fn {
        Some(f) => f,
        None => {
            ura_log(3, "API sniff: get_assembly_image not available");
            return;
        }
    };
    let get_class = match api.il2cpp_get_class_fn {
        Some(f) => f,
        None => {
            ura_log(3, "API sniff: get_class not available");
            return;
        }
    };
    let get_method_addr = match api.il2cpp_get_method_addr_fn {
        Some(f) => f,
        None => {
            ura_log(3, "API sniff: get_method_addr not available");
            return;
        }
    };

    let umamusume = get_asm(to_cstr("umamusume.dll").as_ptr());
    if umamusume.is_null() {
        ura_log(3, "API sniff: umamusume.dll image not found");
        return;
    }

    // HttpHelper class
    let http_helper = get_class(
        umamusume,
        to_cstr("Gallop").as_ptr(),
        to_cstr("HttpHelper").as_ptr(),
    );
    if http_helper.is_null() {
        ura_log(3, "API sniff: HttpHelper class not found");
        return;
    }
    ura_log(3, "API sniff: HttpHelper class found");

    // Hook CompressRequest
    if COMPRESS_REQUEST_ADDR == 0 {
        let addr = get_method_addr(http_helper as usize, to_cstr("CompressRequest").as_ptr(), 1);
        if addr != 0 {
            if interceptor_hook(addr, compress_request_hook_handler as usize) {
                COMPRESS_REQUEST_ADDR = addr;
                ura_log(
                    3,
                    &format!("API sniff: CompressRequest hooked at 0x{:x}", addr),
                );
            } else {
                ura_log(
                    3,
                    &format!("API sniff: CompressRequest hook FAILED at 0x{:x}", addr),
                );
            }
        } else {
            ura_log(3, "API sniff: CompressRequest NOT FOUND");
        }
    }

    // Hook DecompressResponse
    if DECOMPRESS_RESPONSE_ADDR == 0 {
        let addr = get_method_addr(
            http_helper as usize,
            to_cstr("DecompressResponse").as_ptr(),
            1,
        );
        if addr != 0 {
            if interceptor_hook(addr, decompress_response_hook_handler as usize) {
                DECOMPRESS_RESPONSE_ADDR = addr;
                ura_log(
                    3,
                    &format!("API sniff: DecompressResponse hooked at 0x{:x}", addr),
                );
            } else {
                ura_log(
                    3,
                    &format!("API sniff: DecompressResponse hook FAILED at 0x{:x}", addr),
                );
            }
        } else {
            ura_log(3, "API sniff: DecompressResponse NOT FOUND");
        }
    }

    // Hook WWWRequest.Post (from Cute.Http.Assembly.dll)
    if POST_ADDR == 0 {
        let cute_http = get_asm(to_cstr("Cute.Http.Assembly.dll").as_ptr());
        if !cute_http.is_null() {
            let www_request = get_class(
                cute_http,
                to_cstr("Cute.Http").as_ptr(),
                to_cstr("WWWRequest").as_ptr(),
            );
            if !www_request.is_null() {
                let addr = get_method_addr(www_request as usize, to_cstr("Post").as_ptr(), 3);
                if addr != 0 {
                    if interceptor_hook(addr, post_hook_handler as usize) {
                        POST_ADDR = addr;
                        ura_log(
                            3,
                            &format!("API sniff: WWWRequest.Post hooked at 0x{:x}", addr),
                        );
                    } else {
                        ura_log(
                            3,
                            &format!("API sniff: WWWRequest.Post hook FAILED at 0x{:x}", addr),
                        );
                    }
                } else {
                    ura_log(3, "API sniff: WWWRequest.Post NOT FOUND");
                }
            } else {
                ura_log(3, "API sniff: Cute.Http.WWWRequest class not found");
            }
        } else {
            ura_log(3, "API sniff: Cute.Http.Assembly.dll image not found");
        }
    }
}

// ★ v3.24.2: Story event choice hook — capture career event choices
// StoryChoiceController.Choice(int choiceIndex, ???)
// ARM64: X0=this, W1=choiceIndex, X2=???
extern "C" fn event_choice_hook_handler(
    this: *mut c_void,
    choice_index: i32,
    _param2: *mut c_void,
) {
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
        EVENT_SELECTED_IDX = choice_index;
        ura_log(
            3,
            &format!(
                "Event choice: index={} choices_count={}",
                choice_index,
                EVENT_CHOICES.len()
            ),
        );

        let trampoline = interceptor_get_trampoline(event_choice_hook_handler as usize);
        if trampoline == 0 {
            ura_log(1, "event_choice_hook: trampoline not found");
            return;
        }
        type FnChoice = unsafe extern "C" fn(*mut c_void, i32, *mut c_void);
        let original: FnChoice = std::mem::transmute(trampoline);
        original(this, choice_index, _param2);
    }));
}

// StoryChoiceController.AddChoiceButton(StoryChoiceParam param)
// ARM64: X0=this, X1=param
// Read StoryChoiceParam fields: LabelText, GainId, NextBlockIndex, LoopExitGainId
extern "C" fn event_add_choice_hook_handler(this: *mut c_void, param: *mut c_void) {
    unsafe {
        if param.is_null() {
            return;
        }

        // Read StoryChoiceParam fields via IL2CPP getter methods
        let label = read_il2cpp_string_from_obj(param, "get_LabelText");
        let gain_id = call_getter_int_raw(param, "get_GainId");
        let next_block_idx = call_getter_int_raw(param, "get_GetNextBlockIndex");
        let loop_exit_gain_id = call_getter_int_raw(param, "get_LoopExitGainId");

        ura_log(
            3,
            &format!(
                "Event choice added: label='{}' gain={} next={} loop_exit={}",
                label, gain_id, next_block_idx, loop_exit_gain_id
            ),
        );

        let _lock = EVENT_STATE_MUTEX.lock();
        EVENT_CHOICES.push(EventChoice {
            label,
            gain_id,
            next_block_idx: if next_block_idx > 0 {
                next_block_idx
            } else {
                -1
            },
            loop_exit_gain_id: if loop_exit_gain_id > 0 {
                loop_exit_gain_id
            } else {
                -1
            },
        });

        drop(_lock);

        if !EVENT_CHOICE_HOOK_INSTALLED || EVENT_ADD_BTN_ADDR == 0 {
            return;
        }

        // ★ v3.24.9: Use trampoline — no unhook/rehook
        let trampoline = interceptor_get_trampoline(event_add_choice_hook_handler as usize);
        if trampoline == 0 {
            ura_log(1, "add_choice_hook: trampoline not found");
            return;
        }
        type FnAddBtn = unsafe extern "C" fn(*mut c_void, *mut c_void);
        let original: FnAddBtn = std::mem::transmute(trampoline);
        original(this, param);
    }
}

// Helper: call getter on an IL2CPP object (returns i32)
unsafe fn call_getter_int_raw(obj: *const c_void, method_name: &str) -> i32 {
    if obj.is_null() || API.is_null() {
        return 0;
    }
    // We need to find the class and method. For simplicity, we use the method pointer approach.
    // Since we don't know the class, we try to read directly from known offsets.
    // StoryChoiceParam is a simple struct, we can try field offsets.
    // Actually, let's use the proper IL2CPP approach.
    let api = &*API;
    let get_class_fn: Option<unsafe extern "C" fn(*const c_void) -> *mut c_void> = {
        let p = resolve_il2cpp_symbol("il2cpp_object_get_class");
        if p.is_null() {
            None
        } else {
            Some(std::mem::transmute(p))
        }
    };
    if get_class_fn.is_none() {
        return 0;
    }
    let class = get_class_fn.unwrap()(obj);
    if class.is_null() {
        return 0;
    }
    call_getter_int(class, obj, method_name)
}

// Helper: read IL2CPP string from object via getter
unsafe fn read_il2cpp_string_from_obj(obj: *const c_void, method_name: &str) -> String {
    if obj.is_null() || API.is_null() {
        return String::new();
    }
    let api = &*API;
    let get_class_fn: Option<unsafe extern "C" fn(*const c_void) -> *mut c_void> = {
        let p = resolve_il2cpp_symbol("il2cpp_object_get_class");
        if p.is_null() {
            None
        } else {
            Some(std::mem::transmute(p))
        }
    };
    if get_class_fn.is_none() {
        return String::new();
    }
    let class = get_class_fn.unwrap()(obj);
    if class.is_null() {
        return String::new();
    }
    let s = call_getter_string(class, obj, method_name);
    let result = read_il2cpp_string(s);
    result
}

// Helper: call a getter that returns a string (IL2CPP String*)
unsafe fn call_getter_string(
    class: *mut c_void,
    obj: *const c_void,
    method_name: &str,
) -> *const c_void {
    if class.is_null() || obj.is_null() || API.is_null() {
        return std::ptr::null();
    }
    let get_method_fn: Option<
        unsafe extern "C" fn(*mut c_void, *const c_char, i32) -> *const c_void,
    > = {
        let p = resolve_il2cpp_symbol("il2cpp_class_get_method_from_name");
        if p.is_null() {
            None
        } else {
            Some(std::mem::transmute(p))
        }
    };
    if get_method_fn.is_none() {
        return std::ptr::null();
    }
    let method = get_method_fn.unwrap()(class, to_cstr(method_name).as_ptr(), 0);
    if method.is_null() {
        return std::ptr::null();
    }

    let get_ptr_fn: Option<unsafe extern "C" fn(*const c_void) -> *const c_void> = {
        let p = resolve_il2cpp_symbol("il2cpp_method_get_pointer");
        if p.is_null() {
            None
        } else {
            Some(std::mem::transmute(p))
        }
    };
    if get_ptr_fn.is_none() {
        return std::ptr::null();
    }
    let ptr = get_ptr_fn.unwrap()(method);
    if ptr.is_null() {
        return std::ptr::null();
    }

    type FnGet = unsafe extern "C" fn(*const c_void) -> *const c_void;
    let getter: FnGet = std::mem::transmute(ptr);
    getter(obj)
}

// ★ v3.24.2: StoryManager.SetStory hook — capture story_id for event type identification
// StoryManager.SetStory(this, story_id, ???, ???, ???)
// ARM64: X0=this, X1=story_id, X2-X4=other params
// ★ v3.24.2 FIX: Don't call getters or ura_log in hook context — these run on the game's
// main thread without SIGSEGV recovery. If the IL2CPP object is in a transitional state,
// calling getters can crash the game process. We only store story_id (passed as parameter).
// chara_id is read from summary data via get_CardId instead.
extern "C" fn story_set_hook_handler(this: *mut c_void, story_id: i32, p2: i64, p3: i64, p4: i64) {
    unsafe {
        if !this.is_null() {
            let _lock = EVENT_STATE_MUTEX.lock();
            EVENT_STORY_ID = story_id;
            drop(_lock);
        }

        if !STORY_SET_HOOK_INSTALLED || STORY_SET_ADDR == 0 {
            return;
        }

        // ★ v3.24.9: Use trampoline — no unhook/rehook
        let trampoline = interceptor_get_trampoline(story_set_hook_handler as usize);
        if trampoline == 0 {
            ura_log(1, "story_set_hook: trampoline not found");
            return;
        }
        type FnSetStory = unsafe extern "C" fn(*mut c_void, i32, i64, i64, i64);
        let original: FnSetStory = std::mem::transmute(trampoline);
        original(this, story_id, p2, p3, p4);
    }
}

unsafe fn install_event_choice_hook() {
    if EVENT_CHOICE_HOOK_INSTALLED {
        return;
    }
    if API.is_null() {
        return;
    }

    let image = match get_image() {
        img if !img.is_null() => img,
        _ => return,
    };

    let class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("StoryChoiceController").as_ptr(),
    );
    if class.is_null() {
        ura_log(3, "Event hook: StoryChoiceController class not found");
        return;
    }

    // Hook AddChoiceButton (1 param: StoryChoiceParam)
    let add_btn_addr = find_method_addr(class, "AddChoiceButton", 1);
    if add_btn_addr != 0 {
        EVENT_ADD_BTN_ADDR = add_btn_addr;
        install_hook_safe(
            "EventAddBtn",
            add_btn_addr,
            event_add_choice_hook_handler as usize,
            &mut ORIG_EVENT_ADD_BTN_PROLOGUE,
        );
    } else {
        ura_log(3, "Event hook: AddChoiceButton NOT FOUND");
    }

    // Hook Choice (2 params: index, ???)
    let choice_addr = find_method_addr(class, "Choice", 2);
    if choice_addr != 0 {
        EVENT_CHOICE_ADDR = choice_addr;
        install_hook_safe(
            "EventChoice",
            choice_addr,
            event_choice_hook_handler as usize,
            &mut ORIG_EVENT_CHOICE_PROLOGUE,
        );
    } else {
        ura_log(3, "Event hook: Choice NOT FOUND");
    }

    // ★ v3.24.2: Hook StoryManager.SetStory to capture story_id and chara_id
    let story_mgr_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("StoryManager").as_ptr(),
    );
    if !story_mgr_class.is_null() {
        let set_story_addr = find_method_addr(story_mgr_class, "SetStory", 4);
        if set_story_addr != 0 {
            STORY_SET_ADDR = set_story_addr;
            STORY_SET_HOOK_INSTALLED = true;
            install_hook_safe(
                "StorySet",
                set_story_addr,
                story_set_hook_handler as usize,
                &mut ORIG_STORY_SET_PROLOGUE,
            );
            ura_log(
                3,
                &format!(
                    "Event hook: StoryManager.SetStory hooked at 0x{:x}",
                    set_story_addr
                ),
            );
        } else {
            ura_log(3, "Event hook: StoryManager.SetStory NOT FOUND");
        }
    } else {
        ura_log(3, "Event hook: StoryManager class NOT FOUND");
    }

    EVENT_CHOICE_HOOK_INSTALLED = true;
}

extern "C" fn on_game_initialized(_userdata: *mut c_void) {
    GAME_INITIALIZED.store(true, Ordering::Relaxed);
    unsafe {
        ura_log(3, "Game initialized");
        ura_notify("URA: Game ready!");
        // v3.22.98: Install hooks FIRST (before precache, which may panic)
        install_training_hook();
        install_exec_training_hook();
        install_failure_rate_hook();
        install_api_sniff_hooks();
        install_event_choice_hook();
        // v3.22.51: Pre-cache all IL2CPP metadata on game thread
        precache_metadata();
    }
}

extern "C" fn on_menu_section(ui: *mut c_void, _userdata: *mut c_void) {
    unsafe {
        if API.is_null() || ui.is_null() {
            return;
        }
        let api = &*API;

        if let Some(f) = api.gui_ui_heading_fn {
            f(ui, to_cstr(&format!("URA Assistant v{}", PLUGIN_VERSION)).as_ptr());
        }
        if let Some(f) = api.gui_ui_separator_fn {
            f(ui);
        }

        if let Some(f) = api.gui_ui_colored_label_fn {
            if GAME_INITIALIZED.load(Ordering::Relaxed) {
                f(ui, 0, 255, 136, 255, to_cstr("Game: Connected").as_ptr());
            } else {
                f(ui, 255, 200, 0, 255, to_cstr("Game: Waiting...").as_ptr());
            }
        }

        if let Some(f) = api.gui_ui_colored_label_fn {
            if HTTP_RUNNING.load(Ordering::Relaxed) {
                f(
                    ui,
                    0,
                    255,
                    136,
                    255,
                    to_cstr(&format!(
                        "HTTP: Running :{}",
                        unsafe { get_config() }.http_port
                    ))
                    .as_ptr(),
                );
            } else {
                f(ui, 255, 80, 80, 255, to_cstr("HTTP: Failed").as_ptr());
            }
        }

        if let Some(f) = api.gui_ui_label_fn {
            f(
                ui,
                to_cstr("Data: WDM->SingleMode->Chara (getters)").as_ptr(),
            );
        }

        let c = CHARA;
        if c.valid {
            if let Some(f) = api.gui_ui_separator_fn {
                f(ui);
            }

            if let Some(f) = api.gui_ui_colored_label_fn {
                f(
                    ui,
                    0,
                    200,
                    255,
                    255,
                    to_cstr(&format!(
                        "Month {} | Half {} | PS:{}",
                        c.month, c.half, c.playing_state
                    ))
                    .as_ptr(),
                );
            }

            if let Some(f) = api.gui_ui_colored_label_fn {
                f(
                    ui,
                    255,
                    100,
                    100,
                    255,
                    to_cstr(&format!("SPD: {}", c.speed)).as_ptr(),
                );
            }
            if let Some(f) = api.gui_ui_colored_label_fn {
                f(
                    ui,
                    100,
                    255,
                    100,
                    255,
                    to_cstr(&format!("STA: {}", c.stamina)).as_ptr(),
                );
            }
            if let Some(f) = api.gui_ui_colored_label_fn {
                f(
                    ui,
                    255,
                    200,
                    50,
                    255,
                    to_cstr(&format!("POW: {}", c.power)).as_ptr(),
                );
            }
            if let Some(f) = api.gui_ui_colored_label_fn {
                f(
                    ui,
                    255,
                    130,
                    50,
                    255,
                    to_cstr(&format!("GUT: {}", c.guts)).as_ptr(),
                );
            }
            if let Some(f) = api.gui_ui_colored_label_fn {
                f(
                    ui,
                    100,
                    180,
                    255,
                    255,
                    to_cstr(&format!("WIZ: {}", c.wiz)).as_ptr(),
                );
            }

            if let Some(f) = api.gui_ui_label_fn {
                f(
                    ui,
                    to_cstr(&format!("Vital: {}/{}", c.vital, c.max_vital)).as_ptr(),
                );
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
                f(
                    ui,
                    color.0,
                    color.1,
                    color.2,
                    255,
                    to_cstr(mot_text).as_ptr(),
                );
            }

            if let Some(f) = api.gui_ui_label_fn {
                f(
                    ui,
                    to_cstr(&format!(
                        "SkillPt: {} | Fan: {}",
                        c.skill_point, c.fan_count
                    ))
                    .as_ptr(),
                );
            }
        } else {
            if let Some(f) = api.gui_ui_label_fn {
                f(ui, to_cstr("No training data yet").as_ptr());
            }
            if let Some(f) = api.gui_ui_label_fn {
                f(ui, to_cstr("Start a training run first").as_ptr());
            }
        }

        if let Some(f) = api.gui_ui_separator_fn {
            f(ui);
        }

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
                        while len < 64 && GUI_HOST_BUF[len] != 0 {
                            len += 1;
                        }
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
                        while len < 8 && GUI_PORT_BUF[len] != 0 {
                            len += 1;
                        }
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
            if ptr.is_null() {
                None
            } else {
                Some(std::mem::transmute::<*mut c_void, $ty>(ptr))
            }
        }};
    }
    Api {
        log_fn: try_api!(
            "log",
            unsafe extern "C" fn(i32, *const c_char, *const c_char)
        ),
        gui_show_notification_fn: try_api!(
            "gui_show_notification",
            unsafe extern "C" fn(*const c_char) -> bool
        ),
        gui_register_menu_item_fn: try_api!(
            "gui_register_menu_item",
            unsafe extern "C" fn(
                *const c_char,
                Option<extern "C" fn(*mut c_void)>,
                *mut c_void,
            ) -> bool
        ),
        gui_register_menu_section_fn: try_api!(
            "gui_register_menu_section",
            unsafe extern "C" fn(
                Option<extern "C" fn(*mut c_void, *mut c_void)>,
                *mut c_void,
            ) -> bool
        ),
        hachimi_register_on_game_initialized_fn: try_api!(
            "hachimi_register_on_game_initialized",
            unsafe extern "C" fn(Option<extern "C" fn(*mut c_void)>, *mut c_void) -> bool
        ),
        gui_ui_heading_fn: try_api!(
            "gui_ui_heading",
            unsafe extern "C" fn(*mut c_void, *const c_char) -> bool
        ),
        gui_ui_label_fn: try_api!(
            "gui_ui_label",
            unsafe extern "C" fn(*mut c_void, *const c_char) -> bool
        ),
        gui_ui_colored_label_fn: try_api!(
            "gui_ui_colored_label",
            unsafe extern "C" fn(*mut c_void, u8, u8, u8, u8, *const c_char) -> bool
        ),
        gui_ui_separator_fn: try_api!(
            "gui_ui_separator",
            unsafe extern "C" fn(*mut c_void) -> bool
        ),
        gui_ui_text_edit_singleline_fn: try_api!(
            "gui_ui_text_edit_singleline",
            unsafe extern "C" fn(*mut c_void, *mut c_char, i32) -> bool
        ),
        il2cpp_get_assembly_image_fn: try_api!(
            "il2cpp_get_assembly_image",
            unsafe extern "C" fn(*const c_char) -> *const c_void
        ),
        il2cpp_get_class_fn: try_api!(
            "il2cpp_get_class",
            unsafe extern "C" fn(*const c_void, *const c_char, *const c_char) -> *mut c_void
        ),
        il2cpp_get_field_from_name_fn: try_api!(
            "il2cpp_get_field_from_name",
            unsafe extern "C" fn(*mut c_void, *const c_char) -> *mut c_void
        ),
        il2cpp_get_field_value_fn: try_api!(
            "il2cpp_get_field_value",
            unsafe extern "C" fn(*const c_void, *const c_void, *mut c_void)
        ),
        il2cpp_get_static_field_value_fn: try_api!(
            "il2cpp_get_static_field_value",
            unsafe extern "C" fn(*const c_void, *mut c_void)
        ),
        il2cpp_resolve_symbol_fn: try_api!(
            "il2cpp_resolve_symbol",
            unsafe extern "C" fn(*const c_char) -> *mut c_void
        ),
        il2cpp_get_singleton_like_instance_fn: try_api!(
            "il2cpp_get_singleton_like_instance",
            unsafe extern "C" fn(*mut c_void) -> *const c_void
        ),
        il2cpp_string_chars_fn: try_api!(
            "il2cpp_string_chars",
            unsafe extern "C" fn(*const c_void) -> *mut u16
        ),
        il2cpp_string_length_fn: try_api!(
            "il2cpp_string_length",
            unsafe extern "C" fn(*const c_void) -> i32
        ),
        // ★ v3.23.3: Hachimi-Edge V3 Interceptor API
        hachimi_instance_fn: try_api!("hachimi_instance", unsafe extern "C" fn() -> usize),
        hachimi_get_interceptor_fn: try_api!(
            "hachimi_get_interceptor",
            unsafe extern "C" fn(usize) -> usize
        ),
        interceptor: 0,
        interceptor_hook_fn: try_api!(
            "interceptor_hook",
            unsafe extern "C" fn(usize, *mut c_void, *mut c_void) -> *mut c_void
        ),
        interceptor_get_trampoline_addr_fn: try_api!(
            "interceptor_get_trampoline_addr",
            unsafe extern "C" fn(usize, *mut c_void) -> *mut c_void
        ),
        il2cpp_get_method_addr_fn: try_api!(
            "il2cpp_get_method_addr",
            unsafe extern "C" fn(usize, *const c_char, i32) -> usize
        ),
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
    // ★ v3.23.3: Initialize interceptor for hook API
    let interceptor = if let (Some(instance_fn), Some(get_interceptor_fn)) =
        (api.hachimi_instance_fn, api.hachimi_get_interceptor_fn)
    {
        let hachimi = unsafe { instance_fn() };
        if hachimi != 0 {
            unsafe { get_interceptor_fn(hachimi) }
        } else {
            0
        }
    } else {
        0
    };
    API = Box::into_raw(Box::new(api));
    if interceptor != 0 {
        unsafe {
            (*API).interceptor = interceptor;
        }
    }
    init_crash_handler();
    check_and_upload_crash_log();
    ura_log(3, "URA plugin v3.24.9 loaded (Interceptor API hooks)");

    if let Some(f) = (*API).gui_show_notification_fn {
        f(to_cstr(&format!("URA v{} Loaded!", PLUGIN_VERSION)).as_ptr());
    }

    if let Some(f) = (*API).gui_register_menu_item_fn {
        f(
            to_cstr("URA Assistant").as_ptr(),
            Some(on_menu_item_click),
            ptr::null_mut(),
        );
    }

    if let Some(f) = (*API).gui_register_menu_section_fn {
        f(Some(on_menu_section), ptr::null_mut());
    }

    if let Some(f) = (*API).hachimi_register_on_game_initialized_fn {
        f(Some(on_game_initialized), ptr::null_mut());
    }

    start_http_server();
    start_auto_update_thread();

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
    let wdm_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkDataManager").as_ptr(),
    );
    if wdm_class.is_null() {
        return r#"{"error":"wdm_class_null"}"#.to_string();
    }
    let wdm_inst = get_singleton(wdm_class);
    if wdm_inst.is_null() {
        return r#"{"error":"wdm_no_singleton"}"#.to_string();
    }

    let sm_data_obj = call_getter_ref(wdm_class, wdm_inst, "get_SingleMode");
    if sm_data_obj.is_null() {
        return r#"{"error":"sm_data_null"}"#.to_string();
    }

    let chara_data_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkSingleModeCharaData").as_ptr(),
    );
    let chara_obj = call_getter_ref(
        find_class(
            image,
            to_cstr("Gallop").as_ptr(),
            to_cstr("WorkSingleModeData").as_ptr(),
        ),
        sm_data_obj,
        "get_Character",
    );
    if chara_obj.is_null() {
        return r#"{"error":"chara_null"}"#.to_string();
    }

    let scenario_id = call_getter_int(chara_data_class, chara_obj, "get_ScenarioId");
    let scenario_obj = try_get_scenario_obj(chara_data_class, chara_obj, scenario_id);
    if scenario_obj.is_null() {
        return r#"{"error":"scenario_obj_null"}"#.to_string();
    }

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
    if scenario_class.is_null() {
        return format!(
            r#"{{"error":"scenario_class_null","name":"{}"}}"#,
            scenario_class_name
        )
        .to_string();
    }

    let dataset_obj = call_getter_on_instance(scenario_class, scenario_obj, "get_DataSet");
    if dataset_obj.is_null() {
        return r#"{"error":"dataset_null"}"#.to_string();
    }

    let dataset_class_name = format!("{}DataSet", scenario_class_name);
    let dataset_class = find_class_by_short_name(image, &dataset_class_name);
    if dataset_class.is_null() {
        return format!(
            r#"{{"error":"dataset_class_null","name":"{}"}}"#,
            dataset_class_name
        )
        .to_string();
    }

    let cmd_elem_class = find_class_by_short_name(image, "ObscuredSingleModeBreedersCommandInfo");
    if cmd_elem_class.is_null() {
        return r#"{"error":"cmd_elem_class_null"}"#.to_string();
    }

    let cmd_arr = call_getter_on_instance(dataset_class, dataset_obj, "get_CommandInfoArray");
    if cmd_arr.is_null() {
        return r#"{"error":"cmd_arr_null"}"#.to_string();
    }

    let cmd_base = cmd_arr as *const u8;
    let cmd_len =
        std::ptr::read_unaligned::<usize>(cmd_base.add(IL2CPP_LIST_COUNT_OFF) as *const usize);
    if cmd_len == 0 {
        return r#"{"error":"cmd_arr_empty"}"#.to_string();
    }

    // ★ Safe element type detection: read klass pointer from first element,
    //   then get class name string via il2cpp_class_get_name (no find_class_by_short_name!)
    let get_name_fn = resolve_il2cpp_symbol("il2cpp_class_get_name");

    let mut actual_elem_class_name = "unknown".to_string();
    let mut elem_is_info_type = true; // default: plain Int32 (safer for small objects)

    // Quick scan: find first command with params to detect element type
    let cmd_limit_detect = std::cmp::min(cmd_len, 5);
    'detect: for i in 0..cmd_limit_detect {
        let elem_ptr = std::ptr::read_unaligned::<*mut c_void>(
            cmd_base.add(IL2CPP_LIST_ITEMS_OFF + i * IL2CPP_LIST_ITEM_SIZE) as *const *mut c_void,
        );
        if elem_ptr.is_null() {
            continue;
        }
        let params_arr =
            call_getter_on_instance(cmd_elem_class, elem_ptr, "get_ParamsIncDecInfoArray");
        if params_arr.is_null() {
            continue;
        }
        let p_base = params_arr as *const u8;
        let p_len =
            std::ptr::read_unaligned::<usize>(p_base.add(IL2CPP_LIST_COUNT_OFF) as *const usize);
        if p_len == 0 {
            continue;
        }
        // Read first element's klass pointer
        let first_elem = std::ptr::read_unaligned::<*mut c_void>(
            p_base.add(IL2CPP_LIST_ITEMS_OFF) as *const *mut c_void
        );
        if first_elem.is_null() {
            continue;
        }
        let elem_klass = std::ptr::read_unaligned::<*mut c_void>(first_elem as *const *mut c_void);
        if elem_klass.is_null() {
            continue;
        }
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
        let elem_ptr = std::ptr::read_unaligned::<*mut c_void>(
            cmd_base.add(IL2CPP_LIST_ITEMS_OFF + i * IL2CPP_LIST_ITEM_SIZE) as *const *mut c_void,
        );
        if elem_ptr.is_null() {
            continue;
        }

        let params_arr =
            call_getter_on_instance(cmd_elem_class, elem_ptr, "get_ParamsIncDecInfoArray");
        if params_arr.is_null() {
            continue;
        }

        let p_base = params_arr as *const u8;
        let p_len =
            std::ptr::read_unaligned::<usize>(p_base.add(IL2CPP_LIST_COUNT_OFF) as *const usize);
        if p_len == 0 || p_len > 20 {
            continue;
        }

        // Only first 3 params per command
        let p_limit = std::cmp::min(p_len, 3);
        for j in 0..p_limit {
            let p_elem = std::ptr::read_unaligned::<*mut c_void>(
                p_base.add(IL2CPP_LIST_ITEMS_OFF + j * IL2CPP_LIST_ITEM_SIZE) as *const *mut c_void,
            );
            if p_elem.is_null() {
                continue;
            }

            let p_elem_bytes = p_elem as *const u8;

            // ★ Method A: ObscuredInt field XOR decryption (Data layout offsets 0x10, 0x24)
            let tt_crypto = std::ptr::read_unaligned::<i32>(
                p_elem_bytes.add(IL2CPP_OBSCURED_INT_UNBOX_KEY_OFF) as *const i32,
            );
            let tt_hidden = std::ptr::read_unaligned::<i32>(
                p_elem_bytes.add(IL2CPP_OBSCURED_INT_UNBOX_HIDDEN_OFF) as *const i32,
            );
            let tt_decrypted = tt_hidden ^ tt_crypto;
            let val_crypto = std::ptr::read_unaligned::<i32>(
                p_elem_bytes.add(IL2CPP_OBSCURED_INT_PAIR2_KEY_OFF) as *const i32,
            );
            let val_hidden = std::ptr::read_unaligned::<i32>(
                p_elem_bytes.add(IL2CPP_OBSCURED_INT_PAIR2_HIDDEN_OFF) as *const i32,
            );
            let val_decrypted = val_hidden ^ val_crypto;

            // ★ Method B: Plain Int32 read (Info layout: 0x10, 0x14)
            let plain_tt = std::ptr::read_unaligned::<i32>(
                p_elem_bytes.add(IL2CPP_OBSCURED_INT_KEY_OFF) as *const i32,
            );
            let plain_val = std::ptr::read_unaligned::<i32>(
                p_elem_bytes.add(IL2CPP_OBSCURED_INT_HIDDEN_OFF) as *const i32,
            );

            // ★ Method C: Auto-detected correct reading based on element class name
            let (auto_tt, auto_val) = if elem_is_info_type {
                (plain_tt, plain_val)
            } else {
                (tt_decrypted, val_decrypted)
            };

            // ★ Raw hex dump of first 0x20 bytes (enough for both layouts)
            let mut hex_dump = String::new();
            for b in 0..0x20 {
                // dump first 32 bytes for debug
                if b > 0 && b % 4 == 0 {
                    hex_dump.push(' ');
                }
                hex_dump.push_str(&format!("{:02x}", *p_elem_bytes.add(b)));
            }

            debug_items.push(format!(
                r#"{{"cmd_idx":{},"param_idx":{},"actual_class":"{}","elem_is_info_type":{},"auto_tt":{},"auto_val":{},"plain_tt":{},"plain_val":{},"field_tt_xor":{},"field_val_xor":{},"raw":"{}"}}"#,
                i, j, actual_elem_class_name, elem_is_info_type, auto_tt, auto_val, plain_tt, plain_val, tt_decrypted, val_decrypted, hex_dump
            ));
        }
    }

    format!(
        r#"{{"scenario_id":{},"actual_elem_class":"{}","elem_is_info_type":{},"items":[{}]}}"#,
        scenario_id,
        actual_elem_class_name,
        elem_is_info_type,
        debug_items.join(",")
    )
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

    let wdm_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkDataManager").as_ptr(),
    );
    if wdm_class.is_null() {
        return r#"{"error":"wdm_class_null"}"#.to_string();
    }
    let wdm_inst = get_singleton(wdm_class);
    if wdm_inst.is_null() {
        return r#"{"error":"wdm_no_singleton"}"#.to_string();
    }

    let sm_data_obj = call_getter_ref(wdm_class, wdm_inst, "get_SingleMode");
    if sm_data_obj.is_null() {
        return r#"{"error":"sm_data_null"}"#.to_string();
    }

    let chara_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkSingleModeCharaData").as_ptr(),
    );
    let chara_obj = call_getter_ref(
        find_class(
            image,
            to_cstr("Gallop").as_ptr(),
            to_cstr("WorkSingleModeData").as_ptr(),
        ),
        sm_data_obj,
        "get_Character",
    );
    if chara_obj.is_null() {
        return r#"{"error":"chara_null"}"#.to_string();
    }

    let sid = call_getter_int(chara_class, chara_obj, "get_ScenarioId");
    if sid != 13 {
        return format!(r#"{{"error":"not_breeders","scenario_id":{}}}"#, sid);
    }

    let scenario_obj = try_get_scenario_obj(chara_class, chara_obj, sid);
    if scenario_obj.is_null() {
        return r#"{"error":"scenario_obj_null"}"#.to_string();
    }

    let sc_class = find_class_by_short_name(image, "WorkSingleModeScenarioBreeders");
    if sc_class.is_null() {
        return r#"{"error":"sc_class_null"}"#.to_string();
    }

    let ds_obj = call_getter_on_instance(sc_class, scenario_obj, "get_DataSet");
    if ds_obj.is_null() {
        return r#"{"error":"dataset_null"}"#.to_string();
    }

    let ds_class = find_class_by_short_name(image, "WorkSingleModeScenarioBreedersDataSet");
    if ds_class.is_null() {
        return r#"{"error":"ds_class_null"}"#.to_string();
    }

    // ★ Read TeamRank from DataSet (ObscuredInt)
    let team_rank = call_getter_obscured_int(ds_class, ds_obj, "get_TeamRank");

    // ★ Read HavingEnhancePoint (DP) from DataSet (ObscuredInt)
    let having_dp = call_getter_obscured_int(ds_class, ds_obj, "get_HavingEnhancePoint");

    // ★ Read EnhanceGroupArray (team parameter levels)
    let mut enhance_groups_json = Vec::new();
    let enhance_elem_class =
        find_class_by_short_name(image, "ObscuredSingleModeBreedersEnhanceGroup");
    if !enhance_elem_class.is_null() {
        let enhance_arr = call_getter_on_instance(ds_class, ds_obj, "get_EnhanceGroupArray");
        if !enhance_arr.is_null() {
            let ebase = enhance_arr as *const u8;
            let elen =
                std::ptr::read_unaligned::<usize>(ebase.add(IL2CPP_LIST_COUNT_OFF) as *const usize);
            for i in 0..elen {
                let ep = std::ptr::read_unaligned::<*mut c_void>(
                    ebase.add(IL2CPP_LIST_ITEMS_OFF + i * IL2CPP_LIST_ITEM_SIZE)
                        as *const *mut c_void,
                );
                if ep.is_null() {
                    continue;
                }
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
        let sp_train_class =
            find_class_by_short_name(image, "ObscuredSingleModeBreedersTeamSpTrainingInfo");
        if !sp_train_class.is_null() {
            dream_left = call_getter_obscured_int(sp_train_class, sp_train_obj, "get_StockNum");
            dream_max = call_getter_obscured_int(sp_train_class, sp_train_obj, "get_StockMax");
            dream_activated =
                call_getter_obscured_int(sp_train_class, sp_train_obj, "get_ActivatedState");
            // v3.15.8: dream_overflow from heuristic StockNum>StockMax
            // TODO: use ChangeParameterInfo.get_IsOverflowTeamSpTrainingStock for authoritative value
        }
    }

    // ★ Read TeamMemberInfoArray from DataSet
    let member_arr = call_getter_on_instance(ds_class, ds_obj, "get_TeamMemberInfoArray");
    if member_arr.is_null() {
        return format!(
            r#"{{"error":"member_arr_null","scenario_id":13,"team_rank":{},"having_dp":{},"dream_left":{},"dream_max":{},"enhance_groups":[{}]}}"#,
            team_rank,
            having_dp,
            dream_left,
            dream_max,
            enhance_groups_json.join(",")
        );
    }

    let mb = member_arr as *const u8;
    let ml = std::ptr::read_unaligned::<usize>(mb.add(IL2CPP_LIST_COUNT_OFF) as *const usize);

    if ml == 0 || ml > 10 {
        return format!(
            r#"{{"error":"member_arr_empty","count":{},"team_rank":{},"having_dp":{},"dream_left":{},"dream_max":{},"enhance_groups":[{}]}}"#,
            ml,
            team_rank,
            having_dp,
            dream_left,
            dream_max,
            enhance_groups_json.join(",")
        );
    }

    // ★ Discover member element class name from runtime object header
    // Instead of guessing class names, read the klass pointer from the first element
    // and use il2cpp_class_get_name to get the actual class name
    let mut discovered_member_class_name = String::new();
    let mut member_class: *mut c_void = std::ptr::null_mut();
    {
        let first_ep = std::ptr::read_unaligned::<*mut c_void>(
            mb.add(IL2CPP_LIST_ITEMS_OFF) as *const *mut c_void
        );
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
        let ep = std::ptr::read_unaligned::<*mut c_void>(
            mb.add(IL2CPP_LIST_ITEMS_OFF + i * IL2CPP_LIST_ITEM_SIZE) as *const *mut c_void,
        );
        if ep.is_null() {
            continue;
        }

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
                if v >= 0 && v <= 17 {
                    level = v;
                    break;
                }
                let v2 = call_getter_int(member_class, ep, ln);
                if v2 >= 0 && v2 <= 17 {
                    level = v2;
                    break;
                }
            }
            // Dream gauge — v3.15.8: TeamMemberInfo has no gauge field (only MemberId/CharaId/Rank/Exp)
            // Gauge data lives in CommandInfo, not TeamMemberInfo; skip reading here
            // gauge stays -1 (will be clamped to 0 below)
            // Chara ID — ObscuredInt field, try obscured decoder first
            // BUG FIX v3.15.8: call_getter_int reads crypto key as plain int (returns 444444),
            // must use call_getter_obscured_int to get decrypted value
            for &cn in &["get_CharaId", "get_CharacterId", "get_CardId"] {
                let v = call_getter_obscured_int(member_class, ep, cn);
                if v > 0 {
                    chara_id = v;
                    break;
                }
                let v2 = call_getter_int(member_class, ep, cn);
                if v2 > 0 {
                    chara_id = v2;
                    break;
                }
            }
            // Exp
            for &en in &["get_Exp", "get_Experience", "get_RankExp", "get_DreamExp"] {
                let v = call_getter_obscured_int(member_class, ep, en);
                if v >= 0 {
                    exp = v;
                    break;
                }
            }
            // Burst ready — BUG FIX v3.15.8: call_getter_bool returns true on -1 (not found)
            // TeamMemberInfo has no burst field, use call_getter_int + explicit >= 0 check
            for &bn in &[
                "get_IsBurstReady",
                "get_BurstReady",
                "get_IsBurst",
                "get_CanBurst",
            ] {
                let v = call_getter_int(member_class, ep, bn);
                if v >= 0 {
                    burst_ready = v != 0;
                    break;
                }
            }

            found_data = level >= 0;
        }

        // Build hex dump as fallback
        let mut hex = String::new();
        let epb = ep as *const u8;
        for b in 0..0x80 {
            // dump first 128 bytes for debug
            if b > 0 && b % 4 == 0 {
                hex.push(' ');
            }
            hex.push_str(&format!("{:02x}", *epb.add(b)));
        }

        if gauge < 0 {
            gauge = 0;
        }
        if level < 0 {
            level = 0;
        }
        // v3.15.8: removed gauge>=3 burst fallback (gauge not available on TeamMemberInfo)

        if level < min_level && level >= 0 {
            min_level = level;
        }

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

    if min_level == 999 {
        min_level = 0;
    }

    format!(
        r#"{{"team_members":[{}],"team_rank":{},"having_dp":{},"dream_left":{},"dream_max":{},"dream_overflow":{},"dream_activated":{},"enhance_groups":[{}],"member_count":{},"member_class":"{}"}}"#,
        members_json.join(","),
        team_rank,
        having_dp,
        dream_left,
        dream_max,
        dream_overflow,
        dream_activated,
        enhance_groups_json.join(","),
        ml,
        discovered_member_class_name
    )
}

unsafe fn debug_breeders_team() -> String {
    let image = match get_image() {
        img if !img.is_null() => img,
        _ => return r#"{"error":"image_null"}"#.to_string(),
    };

    let wdm_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkDataManager").as_ptr(),
    );
    if wdm_class.is_null() {
        return r#"{"error":"wdm_class_null"}"#.to_string();
    }
    let wdm_inst = get_singleton(wdm_class);
    if wdm_inst.is_null() {
        return r#"{"error":"wdm_no_singleton"}"#.to_string();
    }

    let sm_data_obj = call_getter_ref(wdm_class, wdm_inst, "get_SingleMode");
    if sm_data_obj.is_null() {
        return r#"{"error":"sm_data_null"}"#.to_string();
    }

    let chara_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkSingleModeCharaData").as_ptr(),
    );
    let chara_obj = call_getter_ref(
        find_class(
            image,
            to_cstr("Gallop").as_ptr(),
            to_cstr("WorkSingleModeData").as_ptr(),
        ),
        sm_data_obj,
        "get_Character",
    );
    if chara_obj.is_null() {
        return r#"{"error":"chara_null"}"#.to_string();
    }

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
        13 => &[
            "get_ScenarioBreeders",
            "get_WorkScenarioBreeders",
            "get_Breeders",
        ],
        14 => &["get_ScenarioRamen", "get_WorkScenarioRamen", "get_Ramen"],
        _ => &[],
    };
    let mut getter_results = Vec::new();
    if !chara_class.is_null() && !chara_obj.is_null() {
        for &gn in getter_names_map {
            let result = call_getter_ref(chara_class, chara_obj, gn);
            getter_results.push(format!(
                r#"{{"name":"{}","found":{}}}"#,
                gn,
                !result.is_null()
            ));
        }
    }

    let team_data = if sid == 13 {
        read_breeders_team()
    } else {
        r#"{"skip":"not_breeders"}"#.to_string()
    };

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
                        let al = std::ptr::read_unaligned::<usize>(
                            ab.add(IL2CPP_LIST_COUNT_OFF) as *const usize
                        );
                        if al > 0 {
                            let first_ep = std::ptr::read_unaligned::<*mut c_void>(
                                ab.add(IL2CPP_LIST_ITEMS_OFF) as *const *mut c_void,
                            );
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
        let pkg = bytes
            .split(|&b| b == 0)
            .filter(|s| !s.is_empty())
            .next()
            .and_then(|s| std::str::from_utf8(s).ok());
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
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

/// v3.22.89: 简易URL解码（处理+和%XX）
fn url_decode(s: &str) -> String {
    let mut result = String::new();
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'+' {
            result.push(' ');
            i += 1;
        } else if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hex = &s[i + 1..i + 3];
            if let Ok(n) = u8::from_str_radix(hex, 16) {
                result.push(n as char);
                i += 3;
            } else {
                result.push(bytes[i] as char);
                i += 1;
            }
        } else {
            result.push(bytes[i] as char);
            i += 1;
        }
    }
    result
}

/// v3.22.89: 解析query参数值
fn parse_query(full_uri: &str, key: &str) -> String {
    let pattern = format!("{}=", key);
    if let Some(q) = full_uri.find(&format!("?{}", pattern)) {
        let start = q + 1 + pattern.len();
        let end = full_uri[start..]
            .find('&')
            .map(|e| start + e)
            .unwrap_or(full_uri.len());
        url_decode(&full_uri[start..end])
    } else if let Some(q) = full_uri.find(&format!("&{}", pattern)) {
        let start = q + 1 + pattern.len();
        let end = full_uri[start..]
            .find('&')
            .map(|e| start + e)
            .unwrap_or(full_uri.len());
        url_decode(&full_uri[start..end])
    } else {
        String::new()
    }
}

/// /tables - List all tables in MasterDB for discovery
/// /tables - List all tables in MasterDB for discovery
/// /debug/unique_skills - Explore mdb tables related to unique skill unlock conditions
/// Dumps table names matching "unique"/"acquisition"/"condition" and their first few rows
fn debug_unique_skills() -> String {
    let mdb_path = match find_mdb_path() {
        Some(p) => p,
        None => return r#"{"error":"mdb_not_found"}"#.to_string(),
    };
    let conn = match Connection::open_with_flags(&mdb_path, OpenFlags::SQLITE_OPEN_READ_ONLY) {
        Ok(c) => c,
        Err(e) => return format!(r#"{{"error":"open_failed","detail":"{}"}}"#, e),
    };

    // Step 1: Find all tables that might relate to unique skills
    let all_tables: Vec<String> =
        match conn.prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name") {
            Ok(mut stmt) => stmt
                .query_map([], |row| Ok(row.get::<_, String>(0).unwrap_or_default()))
                .unwrap()
                .filter_map(|r| r.ok())
                .collect(),
            Err(e) => return format!(r#"{{"error":"table_list_failed","detail":"{}"}}"#, e),
        };

    let keywords = [
        "unique",
        "acquisition",
        "skill_cond",
        "skill_unlock",
        "support_card_skill",
        "skill_learn",
        "unique_effect",
    ];
    let matched_tables: Vec<String> = all_tables
        .iter()
        .filter(|t| {
            let tl = t.to_lowercase();
            keywords.iter().any(|k| tl.contains(k))
        })
        .cloned()
        .collect();

    // Step 2: For each matched table, dump schema + first 3 rows as raw text
    let mut results: Vec<String> = Vec::new();
    for table_name in &matched_tables {
        let safe_name = table_name.replace("]", "]]");

        // Get column names
        let col_names: Vec<String> =
            match conn.prepare(&format!("PRAGMA table_info([{}])", safe_name)) {
                Ok(mut stmt) => stmt
                    .query_map([], |row| Ok(row.get::<_, String>(1).unwrap_or_default()))
                    .unwrap()
                    .filter_map(|r| r.ok())
                    .collect(),
                Err(_) => Vec::new(),
            };

        // Get first 3 rows as raw tab-separated text (avoid JSON nesting issues)
        let sample_rows: Vec<String> =
            match conn.prepare(&format!("SELECT * FROM [{}] LIMIT 3", safe_name)) {
                Ok(mut stmt) => stmt
                    .query_map([], |row| {
                        let col_count = col_names.len();
                        let mut pairs: Vec<String> = Vec::new();
                        for ci in 0..col_count {
                            let cn = col_names.get(ci).unwrap_or(&String::new()).clone();
                            // Try string first, then integer, then null
                            let val_str: String = row
                                .get::<_, Option<String>>(ci)
                                .unwrap_or(None)
                                .or_else(|| {
                                    row.get::<_, Option<i64>>(ci)
                                        .unwrap_or(None)
                                        .map(|i| i.to_string())
                                })
                                .or_else(|| {
                                    row.get::<_, Option<f64>>(ci)
                                        .unwrap_or(None)
                                        .map(|f| format!("{:.4}", f))
                                })
                                .unwrap_or_else(|| "NULL".to_string());
                            pairs.push(format!("{}={}", cn, val_str));
                        }
                        Ok(pairs.join("|"))
                    })
                    .unwrap()
                    .filter_map(|r| r.ok())
                    .collect(),
                Err(_) => Vec::new(),
            };

        results.push(format!(
            r#"{{"table":"{}","columns":[{}],"rows":[{}]}}"#,
            json_escape(table_name),
            col_names
                .iter()
                .map(|c| format!(r#""{}""#, json_escape(c)))
                .collect::<Vec<_>>()
                .join(","),
            sample_rows
                .iter()
                .map(|r| format!(r#""{}""#, json_escape(r)))
                .collect::<Vec<_>>()
                .join(",")
        ));
    }

    // Step 3: support_card_data columns
    let sc_columns: Vec<String> = match conn.prepare("PRAGMA table_info(support_card_data)") {
        Ok(mut stmt) => stmt
            .query_map([], |row| {
                let name: String = row.get::<_, String>(1).unwrap_or_default();
                let typ: String = row.get::<_, String>(2).unwrap_or_default();
                Ok(format!(
                    r#"{{"name":"{}","type":"{}"}}"#,
                    json_escape(&name),
                    json_escape(&typ)
                ))
            })
            .unwrap()
            .filter_map(|r| r.ok())
            .collect(),
        Err(_) => Vec::new(),
    };

    drop(conn);

    format!(
        r#"{{"ok":true,"version":"3.22.91","matched_tables":{},"table_details":[{}],"support_card_data_columns":[{}]}}"#,
        matched_tables.len(),
        results.join(","),
        sc_columns.join(",")
    )
}

/// /debug/table?name=<table_name>&limit=<N>&offset=<M> - Query any mdb table by name
fn debug_table_query(table_name: &str, limit: usize, offset: usize) -> String {
    if table_name.is_empty() {
        return r#"{"ok":false,"error":"missing_name"}"#.to_string();
    }
    // Validate table name: only alphanumeric, underscore, hyphen
    if table_name
        .chars()
        .any(|c| !c.is_alphanumeric() && c != '_' && c != '-')
    {
        return format!(
            r#"{{"ok":false,"error":"invalid_table_name","table":"{}"}}"#,
            json_escape(table_name)
        );
    }
    // Safe bracket escaping for SQL
    let safe_name = table_name.replace("]", "]]");

    let mdb_path = match find_mdb_path() {
        Some(p) => p,
        None => return r#"{"ok":false,"error":"mdb_not_found"}"#.to_string(),
    };
    let conn = match Connection::open_with_flags(&mdb_path, OpenFlags::SQLITE_OPEN_READ_ONLY) {
        Ok(c) => c,
        Err(e) => return format!(r#"{{"ok":false,"error":"open_failed","detail":"{}"}}"#, e),
    };

    // 1. Get column names via PRAGMA table_info
    let cols: Vec<String> = match conn.prepare(&format!("PRAGMA table_info([{}])", safe_name)) {
        Ok(mut stmt) => {
            let rows_result =
                stmt.query_map([], |row| Ok(row.get::<_, String>(1).unwrap_or_default()));
            match rows_result {
                Ok(rows) => rows.filter_map(|r| r.ok()).collect(),
                Err(_) => {
                    return format!(
                        r#"{{"ok":false,"error":"table_not_found","table":"{}"}}"#,
                        json_escape(table_name)
                    )
                }
            }
        }
        Err(_) => {
            return format!(
                r#"{{"ok":false,"error":"table_not_found","table":"{}"}}"#,
                json_escape(table_name)
            )
        }
    };
    if cols.is_empty() {
        return format!(
            r#"{{"ok":false,"error":"table_not_found","table":"{}"}}"#,
            json_escape(table_name)
        );
    }

    // 2. Get total row count
    let total: i64 = match conn.prepare(&format!("SELECT COUNT(*) FROM [{}]", safe_name)) {
        Ok(mut stmt) => stmt.query_row([], |r| r.get(0)).unwrap_or(0),
        Err(_) => 0,
    };

    // 3. Get rows
    let col_list = cols
        .iter()
        .map(|c| format!("[{}]", c.replace("]", "]]")))
        .collect::<Vec<_>>()
        .join(",");
    let sql = format!(
        "SELECT {} FROM [{}] LIMIT {} OFFSET {}",
        col_list, safe_name, limit, offset
    );
    let n_cols = cols.len();
    let rows: Vec<String> = match conn.prepare(&sql) {
        Ok(mut stmt) => {
            let rows_result = stmt.query_map([], |row| {
                let mut pairs: Vec<String> = Vec::new();
                for ci in 0..n_cols {
                    let cn = cols.get(ci).unwrap_or(&String::new()).clone();
                    let int_val = row.get::<_, Option<i64>>(ci).unwrap_or(None);
                    let val = if let Some(v) = int_val {
                        v.to_string()
                    } else {
                        let str_val = row.get::<_, Option<String>>(ci).unwrap_or(None);
                        match str_val {
                            Some(s) => format!(r#""{}""#, json_escape(&s)),
                            None => "null".to_string(),
                        }
                    };
                    pairs.push(format!(r#""{}":{}"#, json_escape(&cn), val));
                }
                Ok(format!(r#"{{{}}}"#, pairs.join(",")))
            });
            match rows_result {
                Ok(rows) => rows.filter_map(|r| r.ok()).collect(),
                Err(_) => Vec::new(),
            }
        }
        Err(e) => return format!(r#"{{"ok":false,"error":"query_failed","detail":"{}"}}"#, e),
    };

    let col_json: Vec<String> = cols
        .iter()
        .map(|c| format!(r#""{}""#, json_escape(c)))
        .collect();
    format!(
        r#"{{"ok":true,"version":"3.22.91","table":"{}","columns":[{}],"row_count":{},"limit":{},"offset":{},"rows":[{}]}}"#,
        json_escape(table_name),
        col_json.join(","),
        total,
        limit,
        offset,
        rows.join(",")
    )
}

/// /debug/push_table?name=<table_name>&batch=<N>&offset=<M>\n/// /debug/download_table?name=<table_name>
/// Single-batch query: each call processes one batch of rows, appends to local file.
/// On the final batch, base64-encodes the whole file and pushes to GitHub.
fn debug_push_table(table_name: &str, batch: usize, offset: usize) -> String {
    if table_name.is_empty() {
        return r#"{"ok":false,"error":"missing_name"}"#.to_string();
    }
    if table_name
        .chars()
        .any(|c| !c.is_alphanumeric() && c != '_' && c != '-')
    {
        return format!(
            r#"{{"ok":false,"error":"invalid_table_name","table":"{}"}}"#,
            json_escape(table_name)
        );
    }
    let safe_name = table_name.replace("]", "]]");
    let mdb_path = match find_mdb_path() {
        Some(p) => p,
        None => return r#"{"ok":false,"error":"mdb_not_found"}"#.to_string(),
    };
    let conn = match Connection::open_with_flags(&mdb_path, OpenFlags::SQLITE_OPEN_READ_ONLY) {
        Ok(c) => c,
        Err(e) => return format!(r#"{{"ok":false,"error":"open_failed","detail":"{}"}}"#, e),
    };

    // Get columns
    let cols: Vec<String> = match conn.prepare(&format!("PRAGMA table_info([{}])", safe_name)) {
        Ok(mut stmt) => {
            let rows_result =
                stmt.query_map([], |row| Ok(row.get::<_, String>(1).unwrap_or_default()));
            match rows_result {
                Ok(rows) => rows.filter_map(|r| r.ok()).collect(),
                Err(_) => {
                    return format!(
                        r#"{{"ok":false,"error":"table_not_found","table":"{}"}}"#,
                        json_escape(table_name)
                    )
                }
            }
        }
        Err(_) => {
            return format!(
                r#"{{"ok":false,"error":"table_not_found","table":"{}"}}"#,
                json_escape(table_name)
            )
        }
    };
    if cols.is_empty() {
        return format!(
            r#"{{"ok":false,"error":"table_not_found","table":"{}"}}"#,
            json_escape(table_name)
        );
    }

    // Get total row count
    let total: i64 = match conn.prepare(&format!("SELECT COUNT(*) FROM [{}]", safe_name)) {
        Ok(mut stmt) => stmt.query_row([], |r| r.get(0)).unwrap_or(0),
        Err(_) => 0,
    };

    let tmp_dir = "/data/data/jp.pokemon.pokeuma/files";
    let tmp_path = format!("{}/uma_push_{}.json", tmp_dir, table_name);

    // If offset > 0 but tmp file doesn't exist, user needs to restart from offset=0
    if offset > 0 && !std::path::Path::new(&tmp_path).exists() {
        return format!(
            r#"{{"ok":false,"error":"no_tmp_file","message":"restart from offset=0"}}"#
        );
    }

    // If offset == 0, start fresh: write header
    if offset == 0 {
        let col_json: Vec<String> = cols
            .iter()
            .map(|c| format!(r#""{}""#, json_escape(c)))
            .collect();
        let header = format!(
            r#"{{"table":"{}","columns":[{}],"row_count":{},"rows":["#,
            json_escape(table_name),
            col_json.join(","),
            total
        );
        if let Err(e) = std::fs::write(&tmp_path, header.as_bytes()) {
            return format!(r#"{{"ok":false,"error":"write_failed","detail":"{}"}}"#, e);
        }
    }

    // Query ONE batch
    let col_list = cols
        .iter()
        .map(|c| format!("[{}]", c.replace("]", "]]")))
        .collect::<Vec<_>>()
        .join(",");
    let n_cols = cols.len();
    let sql = format!(
        "SELECT {} FROM [{}] LIMIT {} OFFSET {}",
        col_list, safe_name, batch, offset
    );
    let batch_rows: Vec<String> = match conn.prepare(&sql) {
        Ok(mut stmt) => {
            let rows_result = stmt.query_map([], |row| {
                let mut pairs: Vec<String> = Vec::new();
                for ci in 0..n_cols {
                    let cn = cols.get(ci).unwrap_or(&String::new()).clone();
                    let int_val = row.get::<_, Option<i64>>(ci).unwrap_or(None);
                    let val = if let Some(v) = int_val {
                        v.to_string()
                    } else {
                        let str_val = row.get::<_, Option<String>>(ci).unwrap_or(None);
                        match str_val {
                            Some(s) => format!(r#""{}""#, json_escape(&s)),
                            None => "null".to_string(),
                        }
                    };
                    pairs.push(format!(r#""{}":{}"#, json_escape(&cn), val));
                }
                Ok(format!(r#"{{{}}}"#, pairs.join(",")))
            });
            match rows_result {
                Ok(rows) => rows.filter_map(|r| r.ok()).collect(),
                Err(_) => Vec::new(),
            }
        }
        Err(e) => return format!(r#"{{"ok":false,"error":"query_failed","detail":"{}"}}"#, e),
    };

    if batch_rows.is_empty() {
        // No more rows at this offset - close JSON and push
        {
            if let Ok(mut f) = std::fs::OpenOptions::new().append(true).open(&tmp_path) {
                let _ = f.write_all(b"]}");
            }
        }
        // Close JSON - file stays for download
        {
            if let Ok(mut f) = std::fs::OpenOptions::new().append(true).open(&tmp_path) {
                let _ = f.write_all(b"]}");
            }
        }
        return format!(
            r#"{{"ok":true,"version":"3.22.91","table":"{}","total_rows":{},"offset":{},"rows_queried":0,"complete":true,"download_url":"/debug/download_table?name={}"}}"#,
            json_escape(table_name),
            total,
            offset,
            json_escape(table_name)
        );
    }

    // Append rows to file
    let mut append_data = String::new();
    // If offset > 0, we need a comma before the first row of this batch
    if offset > 0 {
        append_data.push(',');
    }
    append_data.push_str(&batch_rows.join(","));

    {
        if let Ok(mut f) = std::fs::OpenOptions::new().append(true).open(&tmp_path) {
            let _ = f.write_all(append_data.as_bytes());
        }
    }

    let rows_queried = batch_rows.len();
    let next_offset = offset + rows_queried;
    let is_last_batch = (next_offset as i64) >= total || rows_queried < batch;

    if !is_last_batch {
        // Not done yet - return progress
        return format!(
            r#"{{"ok":true,"version":"3.22.91","table":"{}","total_rows":{},"offset":{},"rows_queried":{},"next_offset":{},"complete":false}}"#,
            json_escape(table_name),
            total,
            offset,
            rows_queried,
            next_offset
        );
    }

    // Last batch! Close JSON - file stays for download
    {
        if let Ok(mut f) = std::fs::OpenOptions::new().append(true).open(&tmp_path) {
            let _ = f.write_all(b"]}");
        }
    }

    format!(
        r#"{{"ok":true,"version":"3.22.91","table":"{}","total_rows":{},"offset":{},"rows_queried":{},"complete":true,"download_url":"/debug/download_table?name={}"}}"#,
        json_escape(table_name),
        total,
        offset,
        rows_queried,
        json_escape(table_name)
    )
}

/// /debug/mdb_all_tables - Dump ALL table names from mdb with row counts,
/// plus search for tables related to skill unlock conditions (bond thresholds, prerequisites)
/// /debug/hint_gain - Dump single_mode_hint_gain table (support card skill hint acquisition conditions)
/// Plus resolve condition_set_id -> single_mode_story_condition_set details
/// /debug/sc_effect - Dump support_card_effect_table + effect_filter + effect_filter_group
/// These tables likely contain the activation conditions for support card unique effects
/// /debug/unique_detail - Join support_card_data + support_card_unique_effect
/// Shows each card with its unique effect types and values for decoding

/// /debug/download_table?name=<table_name>&batch=<N>
/// Auto-batch build + download: queries all rows in batches, writes to local file, returns full JSON.
/// If file already exists (from previous push_table calls), returns it directly.
/// Set batch=30 for large-row tables (skill_data ~2KB/row), batch=500 for small rows.
fn debug_download_table(table_name: &str, batch: usize) -> String {
    if table_name.is_empty() {
        return r#"{"ok":false,"error":"missing_name"}"#.to_string();
    }
    if table_name
        .chars()
        .any(|c| !c.is_alphanumeric() && c != '_' && c != '-')
    {
        return format!(
            r#"{{"ok":false,"error":"invalid_table_name","table":"{}"}}"#,
            json_escape(table_name)
        );
    }
    let tmp_dir = "/data/data/jp.pokemon.pokeuma/files";
    let tmp_path = format!("{}/uma_push_{}.json", tmp_dir, table_name);

    // If file already exists from a previous successful build, return it directly
    // (check if it ends with "]}" to confirm it's complete)
    if std::path::Path::new(&tmp_path).exists() {
        if let Ok(content) = std::fs::read_to_string(&tmp_path) {
            if content.ends_with("]}") {
                return content;
            }
        }
        // Incomplete or corrupt file, delete and rebuild
        let _ = std::fs::remove_file(&tmp_path);
    }

    // Auto-batch: query all rows and build JSON file
    let mdb_path = match find_mdb_path() {
        Some(p) => p,
        None => return r#"{"ok":false,"error":"mdb_not_found"}"#.to_string(),
    };
    let conn = match Connection::open_with_flags(&mdb_path, OpenFlags::SQLITE_OPEN_READ_ONLY) {
        Ok(c) => c,
        Err(e) => {
            return format!(
                r#"{{"ok":false,"error":"db_open_failed","detail":"{}"}}"#,
                e
            )
        }
    };
    let total: i64 =
        match conn.query_row(&format!("SELECT COUNT(*) FROM {}", table_name), [], |row| {
            row.get(0)
        }) {
            Ok(t) => t,
            Err(e) => return format!(r#"{{"ok":false,"error":"count_failed","detail":"{}"}}"#, e),
        };

    // Write JSON header
    let mut f = match std::fs::File::create(&tmp_path) {
        Ok(file) => file,
        Err(e) => {
            return format!(
                r#"{{"ok":false,"error":"file_create_failed","detail":"{}"}}"#,
                e
            )
        }
    };
    if let Err(e) = f.write_all(
        format!(
            r#"{{"table":"{}","total_rows":{},"rows":["#,
            json_escape(table_name),
            total
        )
        .as_bytes(),
    ) {
        let _ = std::fs::remove_file(&tmp_path);
        return format!(
            r#"{{"ok":false,"error":"header_write_failed","detail":"{}"}}"#,
            e
        );
    }

    let mut offset = 0usize;
    let mut need_comma = false;
    loop {
        let query = format!(
            "SELECT * FROM [{}] LIMIT {} OFFSET {}",
            table_name, batch, offset
        );
        let rows = match conn.prepare(&query) {
            Ok(mut stmt) => {
                let column_count = stmt.column_count();
                let mut batch_rows: Vec<String> = Vec::new();
                let rows_result = stmt.query_map([], |row| {
                    let mut parts: Vec<String> = Vec::new();
                    for ci in 0..column_count {
                        let int_val = row.get::<_, Option<i64>>(ci).unwrap_or(None);
                        let val = if let Some(v) = int_val {
                            v.to_string()
                        } else {
                            let str_val = row.get::<_, Option<String>>(ci).unwrap_or(None);
                            match str_val {
                                Some(s) => format!(r#""{}""#, json_escape(&s)),
                                None => "null".to_string(),
                            }
                        };
                        parts.push(val);
                    }
                    Ok(format!("[{}]", parts.join(",")))
                });
                match rows_result {
                    Ok(mapped) => {
                        for r in mapped.flatten() {
                            batch_rows.push(r);
                        }
                    }
                    Err(e) => {
                        let _ = std::fs::remove_file(&tmp_path);
                        return format!(
                            r#"{{"ok":false,"error":"row_iter_failed","detail":"{}"}}"#,
                            e
                        );
                    }
                }
                batch_rows
            }
            Err(e) => {
                let _ = std::fs::remove_file(&tmp_path);
                return format!(r#"{{"ok":false,"error":"query_failed","detail":"{}"}}"#, e);
            }
        };

        if rows.is_empty() {
            break;
        }

        // Append rows
        let mut append_data = String::new();
        if need_comma {
            append_data.push(',');
        }
        append_data.push_str(&rows.join(","));
        {
            let mut f = match std::fs::OpenOptions::new().append(true).open(&tmp_path) {
                Ok(file) => file,
                Err(e) => {
                    let _ = std::fs::remove_file(&tmp_path);
                    return format!(
                        r#"{{"ok":false,"error":"append_open_failed","detail":"{}"}}"#,
                        e
                    );
                }
            };
            if let Err(e) = f.write_all(append_data.as_bytes()) {
                let _ = std::fs::remove_file(&tmp_path);
                return format!(
                    r#"{{"ok":false,"error":"append_write_failed","detail":"{}"}}"#,
                    e
                );
            }
        }
        need_comma = true;

        offset += rows.len();
        if offset as i64 >= total || rows.len() < batch {
            break;
        }
    }

    // Close JSON
    {
        if let Ok(mut f) = std::fs::OpenOptions::new().append(true).open(&tmp_path) {
            let _ = f.write_all(b"]}");
        }
    }
    // For very large tables, don't try to read the whole file into memory.
    // Return a pointer instead - the user can access it via the file path.
    let file_size = match std::fs::metadata(&tmp_path) {
        Ok(m) => m.len() as usize,
        Err(e) => return format!(r#"{{"ok":false,"error":"stat_failed","detail":"{}"}}"#, e),
    };
    // If file > 2MB, return metadata instead of reading into memory
    if file_size > 2_000_000 {
        return format!(
            r#"{{"ok":true,"version":"3.22.91","table":"{}","total_rows":{},"file_size":{},"file_path":"{}","hint":"file too large for HTTP response, use push_table batch mode instead"}}"#,
            json_escape(table_name),
            total,
            file_size,
            tmp_path
        );
    }

    // Return the file
    match std::fs::read_to_string(&tmp_path) {
        Ok(content) => content,
        Err(e) => format!(r#"{{"ok":false,"error":"read_failed","detail":"{}"}}"#, e),
    }
}

fn debug_unique_detail() -> String {
    let mdb_path = match find_mdb_path() {
        Some(p) => p,
        None => return r#"{"error":"mdb_not_found"}"#.to_string(),
    };
    let conn = match Connection::open_with_flags(&mdb_path, OpenFlags::SQLITE_OPEN_READ_ONLY) {
        Ok(c) => c,
        Err(e) => return format!(r#"{{"error":"open_failed","detail":"{}"}}"#, e),
    };

    // 1. Get all support_card_data rows with unique_effect_id > 0
    let mut cards: Vec<String> = Vec::new();
    if let Ok(mut stmt) = conn.prepare(
        "SELECT id, chara_id, rarity, command_id, unique_effect_id, support_card_type \
         FROM support_card_data WHERE unique_effect_id > 0 ORDER BY id",
    ) {
        cards = stmt
            .query_map([], |row| {
                let id: i64 = row.get(0).unwrap_or(0);
                let cid: i64 = row.get(1).unwrap_or(0);
                let rar: i64 = row.get(2).unwrap_or(0);
                let cmd: i64 = row.get(3).unwrap_or(0);
                let ueid: i64 = row.get(4).unwrap_or(0);
                let sct: i64 = row.get(5).unwrap_or(0);
                Ok(format!(
                    r#"{{"id":{},"chara":{},"rarity":{},"cmd":{},"ueid":{},"sct":{}}}"#,
                    id, cid, rar, cmd, ueid, sct
                ))
            })
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
    }

    // 2. Get all unique_effect rows, grouped by id
    let mut effects: Vec<String> = Vec::new();
    if let Ok(mut stmt) = conn.prepare(
        "SELECT id, lv, type_0, value_0, value_0_1, value_0_2, value_0_3, value_0_4, type_1, value_1, value_1_1, value_1_2, value_1_3, value_1_4, idle_mode_sub_rate FROM support_card_unique_effect ORDER BY id, lv"
    ) {
        effects = stmt.query_map([], |row| {
            let id: i64 = row.get(0).unwrap_or(0);
            let lv: i64 = row.get(1).unwrap_or(0);
            let t0: i64 = row.get(2).unwrap_or(0);
            let v0: i64 = row.get(3).unwrap_or(0);
            let v0_1: i64 = row.get(4).unwrap_or(0);
            let v0_2: i64 = row.get(5).unwrap_or(0);
            let v0_3: i64 = row.get(6).unwrap_or(0);
            let v0_4: i64 = row.get(7).unwrap_or(0);
            let t1: i64 = row.get(8).unwrap_or(0);
            let v1: i64 = row.get(9).unwrap_or(0);
            let v1_1: i64 = row.get(10).unwrap_or(0);
            let v1_2: i64 = row.get(11).unwrap_or(0);
            let v1_3: i64 = row.get(12).unwrap_or(0);
            let v1_4: i64 = row.get(13).unwrap_or(0);
            let idle: i64 = row.get(14).unwrap_or(0);
            Ok(format!(
                r#"{{"id":{},"lv":{},"t0":{},"v0":{},"v0_1":{},"v0_2":{},"v0_3":{},"v0_4":{},"t1":{},"v1":{},"v1_1":{},"v1_2":{},"v1_3":{},"v1_4":{},"idle":{}}}"#,
                id, lv, t0, v0, v0_1, v0_2, v0_3, v0_4, t1, v1, v1_1, v1_2, v1_3, v1_4, idle
            ))
        }).unwrap().filter_map(|r| r.ok()).collect();
    }

    // 3. type_0 + type_1 combo distribution (the key to decoding)
    let mut combo_dist: Vec<String> = Vec::new();
    if let Ok(mut stmt) = conn.prepare(
        "SELECT type_0, type_1, COUNT(*) as cnt FROM support_card_unique_effect GROUP BY type_0, type_1 ORDER BY cnt DESC"
    ) {
        combo_dist = stmt.query_map([], |row| {
            let t0: i64 = row.get(0).unwrap_or(0);
            let t1: i64 = row.get(1).unwrap_or(0);
            let c: i64 = row.get(2).unwrap_or(0);
            Ok(format!(r#"{{"t0":{},"t1":{},"count":{}}}"#, t0, t1, c))
        }).unwrap().filter_map(|r| r.ok()).collect();
    }

    // 4. Also dump effect_filter for reference (type -> group mapping)
    let mut filters: Vec<String> = Vec::new();
    if let Ok(mut stmt) =
        conn.prepare("SELECT type, group_id, sort_id FROM support_card_effect_filter ORDER BY type")
    {
        filters = stmt
            .query_map([], |row| {
                let t: i64 = row.get(0).unwrap_or(0);
                let g: i64 = row.get(1).unwrap_or(0);
                let s: i64 = row.get(2).unwrap_or(0);
                Ok(format!(r#"{{"type":{},"grp":{},"sort":{}}}"#, t, g, s))
            })
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
    }

    // 5. Sample: cards with type_0=101 (the most common "conditional effect" type)
    // These are likely "per X skill, +Y" effects like Mejiro Top Roman
    let mut t101_samples: Vec<String> = Vec::new();
    if let Ok(mut stmt) = conn.prepare(
        "SELECT ue.id, ue.lv, ue.type_0, ue.value_0, ue.value_0_1, ue.value_0_2, ue.value_0_3, ue.value_0_4, ue.type_1, ue.value_1, ue.value_1_1, ue.value_1_2, ue.value_1_3, ue.value_1_4, ue.idle_mode_sub_rate, sc.id as sc_id, sc.chara_id \
         FROM support_card_unique_effect ue \
         JOIN support_card_data sc ON ue.id = sc.unique_effect_id \
         WHERE ue.type_0 = 101 ORDER BY sc.id LIMIT 10"
    ) {
        t101_samples = stmt.query_map([], |row| {
            let eid: i64 = row.get(0).unwrap_or(0);
            let lv: i64 = row.get(1).unwrap_or(0);
            let t0: i64 = row.get(2).unwrap_or(0);
            let v0: i64 = row.get(3).unwrap_or(0);
            let v0_1: i64 = row.get(4).unwrap_or(0);
            let v0_2: i64 = row.get(5).unwrap_or(0);
            let v0_3: i64 = row.get(6).unwrap_or(0);
            let v0_4: i64 = row.get(7).unwrap_or(0);
            let t1: i64 = row.get(8).unwrap_or(0);
            let v1: i64 = row.get(9).unwrap_or(0);
            let v1_1: i64 = row.get(10).unwrap_or(0);
            let v1_2: i64 = row.get(11).unwrap_or(0);
            let v1_3: i64 = row.get(12).unwrap_or(0);
            let v1_4: i64 = row.get(13).unwrap_or(0);
            let idle: i64 = row.get(14).unwrap_or(0);
            let scid: i64 = row.get(15).unwrap_or(0);
            let chid: i64 = row.get(16).unwrap_or(0);
            Ok(format!(
                r#"{{"eid":{},"lv":{},"t0":{},"v0":{},"v0_1":{},"v0_2":{},"v0_3":{},"v0_4":{},"t1":{},"v1":{},"v1_1":{},"v1_2":{},"v1_3":{},"v1_4":{},"idle":{},"sc_id":{},"chara":{}}}"#,
                eid, lv, t0, v0, v0_1, v0_2, v0_3, v0_4, t1, v1, v1_1, v1_2, v1_3, v1_4, idle, scid, chid
            ))
        }).unwrap().filter_map(|r| r.ok()).collect();
    }

    // 6. Sample: cards with type_0=116 (Mejiro Top Roman etc - "per skill count" effects)
    let mut t116_samples: Vec<String> = Vec::new();
    if let Ok(mut stmt) = conn.prepare(
        "SELECT ue.id, ue.lv, ue.type_0, ue.value_0, ue.value_0_1, ue.value_0_2, ue.value_0_3, ue.value_0_4, ue.type_1, ue.value_1, ue.value_1_1, ue.value_1_2, ue.value_1_3, ue.value_1_4, ue.idle_mode_sub_rate, sc.id as sc_id, sc.chara_id \
         FROM support_card_unique_effect ue \
         JOIN support_card_data sc ON ue.id = sc.unique_effect_id \
         WHERE ue.type_0 = 116 ORDER BY sc.id LIMIT 10"
    ) {
        t116_samples = stmt.query_map([], |row| {
            let eid: i64 = row.get(0).unwrap_or(0);
            let lv: i64 = row.get(1).unwrap_or(0);
            let t0: i64 = row.get(2).unwrap_or(0);
            let v0: i64 = row.get(3).unwrap_or(0);
            let v0_1: i64 = row.get(4).unwrap_or(0);
            let v0_2: i64 = row.get(5).unwrap_or(0);
            let v0_3: i64 = row.get(6).unwrap_or(0);
            let v0_4: i64 = row.get(7).unwrap_or(0);
            let t1: i64 = row.get(8).unwrap_or(0);
            let v1: i64 = row.get(9).unwrap_or(0);
            let v1_1: i64 = row.get(10).unwrap_or(0);
            let v1_2: i64 = row.get(11).unwrap_or(0);
            let v1_3: i64 = row.get(12).unwrap_or(0);
            let v1_4: i64 = row.get(13).unwrap_or(0);
            let idle: i64 = row.get(14).unwrap_or(0);
            let scid: i64 = row.get(15).unwrap_or(0);
            let chid: i64 = row.get(16).unwrap_or(0);
            Ok(format!(
                r#"{{"eid":{},"lv":{},"t0":{},"v0":{},"v0_1":{},"v0_2":{},"v0_3":{},"v0_4":{},"t1":{},"v1":{},"v1_1":{},"v1_2":{},"v1_3":{},"v1_4":{},"idle":{},"sc_id":{},"chara":{}}}"#,
                eid, lv, t0, v0, v0_1, v0_2, v0_3, v0_4, t1, v1, v1_1, v1_2, v1_3, v1_4, idle, scid, chid
            ))
        }).unwrap().filter_map(|r| r.ok()).collect();
    }

    drop(conn);

    format!(
        r#"{{"ok":true,"version":"3.22.91","cards_with_unique":[{}],"all_effects":[{}],"combo_dist":[{}],"effect_filter":[{}],"t101_samples":[{}],"t116_samples":[{}]}}"#,
        cards.join(","),
        effects.join(","),
        combo_dist.join(","),
        filters.join(","),
        t101_samples.join(","),
        t116_samples.join(",")
    )
}

fn debug_sc_effect() -> String {
    let mdb_path = match find_mdb_path() {
        Some(p) => p,
        None => return r#"{"error":"mdb_not_found"}"#.to_string(),
    };
    let conn = match Connection::open_with_flags(&mdb_path, OpenFlags::SQLITE_OPEN_READ_ONLY) {
        Ok(c) => c,
        Err(e) => return format!(r#"{{"error":"open_failed","detail":"{}"}}"#, e),
    };

    // Helper: dump table schema as column names
    fn get_columns(conn: &Connection, table: &str) -> Vec<String> {
        let safe = table.replace("]", "]]");
        match conn.prepare(&format!("PRAGMA table_info([{}])", safe)) {
            Ok(mut stmt) => stmt
                .query_map([], |row| Ok(row.get::<_, String>(1).unwrap_or_default()))
                .unwrap()
                .filter_map(|r| r.ok())
                .collect(),
            Err(_) => Vec::new(),
        }
    }

    // Helper: dump first N rows as JSON array
    fn get_rows(conn: &Connection, table: &str, cols: &[String], limit: usize) -> Vec<String> {
        let safe = table.replace("]", "]]");
        let col_list = cols
            .iter()
            .map(|c| format!("[{}]", c.replace("]", "]]")))
            .collect::<Vec<_>>()
            .join(",");
        let sql = format!("SELECT {} FROM [{}] LIMIT {}", col_list, safe, limit);
        match conn.prepare(&sql) {
            Ok(mut stmt) => {
                let n_cols = cols.len();
                stmt.query_map([], |row| {
                    let mut pairs: Vec<String> = Vec::new();
                    for ci in 0..n_cols {
                        let cn = cols.get(ci).unwrap_or(&String::new()).clone();
                        let val = row
                            .get::<_, Option<i64>>(ci)
                            .unwrap_or(None)
                            .map(|v| v.to_string())
                            .or_else(|| row.get::<_, Option<String>>(ci).unwrap_or(None))
                            .unwrap_or_else(|| "null".to_string());
                        pairs.push(format!(
                            r#""{}":{}"#,
                            json_escape(&cn),
                            if val == "null" {
                                val
                            } else {
                                format!(r#""{}""#, json_escape(&val))
                            }
                        ));
                    }
                    Ok(format!(r#"{{{}}}"#, pairs.join(",")))
                })
                .unwrap()
                .filter_map(|r| r.ok())
                .collect()
            }
            Err(_) => Vec::new(),
        }
    }

    // 1. support_card_effect_table (4931 rows) - schema + first 5 rows + 5 rows with unique_effect_id
    let scet_cols = get_columns(&conn, "support_card_effect_table");
    let scet_rows = get_rows(&conn, "support_card_effect_table", &scet_cols, 5);
    // Get rows where effect_table_id matches a known unique card
    let mut scet_unique: Vec<String> = Vec::new();
    if let Ok(mut stmt) = conn.prepare(
        "SELECT * FROM support_card_effect_table WHERE effect_table_id IN (SELECT unique_effect_id FROM support_card_data WHERE unique_effect_id > 0 LIMIT 5) LIMIT 20"
    ) {
        scet_unique = stmt.query_map([], |row| {
            let n = scet_cols.len();
            let mut pairs: Vec<String> = Vec::new();
            for ci in 0..n {
                let cn = scet_cols.get(ci).unwrap_or(&String::new()).clone();
                let val = row.get::<_, Option<i64>>(ci).unwrap_or(None)
                    .map(|v| v.to_string())
                    .or_else(|| row.get::<_, Option<String>>(ci).unwrap_or(None))
                    .unwrap_or_else(|| "null".to_string());
                pairs.push(format!(r#""{}":{}"#, json_escape(&cn), if val == "null" { val } else { format!(r#""{}""#, json_escape(&val)) }));
            }
            Ok(format!(r#"{{{}}}"#, pairs.join(",")))
        }).unwrap().filter_map(|r| r.ok()).collect();
    }

    // 2. support_card_effect_filter (26 rows) - full dump
    let scef_cols = get_columns(&conn, "support_card_effect_filter");
    let scef_rows = get_rows(&conn, "support_card_effect_filter", &scef_cols, 30);

    // 3. support_card_effect_filter_group (4 rows) - full dump
    let scefg_cols = get_columns(&conn, "support_card_effect_filter_group");
    let scefg_rows = get_rows(&conn, "support_card_effect_filter_group", &scefg_cols, 10);

    // 4. support_card_unique_effect - dump more rows to see type patterns
    let scue_cols = get_columns(&conn, "support_card_unique_effect");
    let mut scue_dist: Vec<String> = Vec::new();
    // type_0 distribution
    if let Ok(mut stmt) = conn.prepare(
        "SELECT type_0, COUNT(*) as cnt FROM support_card_unique_effect GROUP BY type_0 ORDER BY cnt DESC"
    ) {
        scue_dist = stmt.query_map([], |row| {
            let t: i64 = row.get(0).unwrap_or(0);
            let c: i64 = row.get(1).unwrap_or(0);
            Ok(format!(r#"{{"type_0":{},"count":{}}}"#, t, c))
        }).unwrap().filter_map(|r| r.ok()).collect();
    }
    let mut scue_type1_dist: Vec<String> = Vec::new();
    if let Ok(mut stmt) = conn.prepare(
        "SELECT type_1, COUNT(*) as cnt FROM support_card_unique_effect GROUP BY type_1 ORDER BY cnt DESC"
    ) {
        scue_type1_dist = stmt.query_map([], |row| {
            let t: i64 = row.get(0).unwrap_or(0);
            let c: i64 = row.get(1).unwrap_or(0);
            Ok(format!(r#"{{"type_1":{},"count":{}}}"#, t, c))
        }).unwrap().filter_map(|r| r.ok()).collect();
    }
    // Sample rows with type_0 or type_1 matching potential condition types
    let mut scue_cond_rows: Vec<String> = Vec::new();
    if let Ok(mut stmt) = conn.prepare(
        "SELECT * FROM support_card_unique_effect WHERE type_0 IN (1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20) OR type_1 IN (1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20) LIMIT 30"
    ) {
        scue_cond_rows = stmt.query_map([], |row| {
            let n = scue_cols.len();
            let mut pairs: Vec<String> = Vec::new();
            for ci in 0..n {
                let cn = scue_cols.get(ci).unwrap_or(&String::new()).clone();
                let val = row.get::<_, Option<i64>>(ci).unwrap_or(None)
                    .map(|v| v.to_string())
                    .or_else(|| row.get::<_, Option<String>>(ci).unwrap_or(None))
                    .unwrap_or_else(|| "null".to_string());
                pairs.push(format!(r#""{}":{}"#, json_escape(&cn), if val == "null" { val } else { format!(r#""{}""#, json_escape(&val)) }));
            }
            Ok(format!(r#"{{{}}}"#, pairs.join(",")))
        }).unwrap().filter_map(|r| r.ok()).collect();
    }

    drop(conn);

    let scet_col_json: Vec<String> = scet_cols
        .iter()
        .map(|c| format!(r#""{}""#, json_escape(c)))
        .collect();
    let scef_col_json: Vec<String> = scef_cols
        .iter()
        .map(|c| format!(r#""{}""#, json_escape(c)))
        .collect();
    let scefg_col_json: Vec<String> = scefg_cols
        .iter()
        .map(|c| format!(r#""{}""#, json_escape(c)))
        .collect();
    let scue_col_json: Vec<String> = scue_cols
        .iter()
        .map(|c| format!(r#""{}""#, json_escape(c)))
        .collect();

    format!(
        r#"{{"ok":true,"version":"3.22.91","effect_table":{{"columns":[{}],"sample":[{}],"unique_match":[{}]}},"effect_filter":{{"columns":[{}],"rows":[{}]}},"effect_filter_group":{{"columns":[{}],"rows":[{}]}},"unique_effect":{{"columns":[{}],"type_0_dist":[{}],"type_1_dist":[{}],"cond_rows":[{}]}}}}"#,
        scet_col_json.join(","),
        scet_rows.join(","),
        scet_unique.join(","),
        scef_col_json.join(","),
        scef_rows.join(","),
        scefg_col_json.join(","),
        scefg_rows.join(","),
        scue_col_json.join(","),
        scue_dist.join(","),
        scue_type1_dist.join(","),
        scue_cond_rows.join(",")
    )
}

fn debug_hint_gain() -> String {
    let mdb_path = match find_mdb_path() {
        Some(p) => p,
        None => return r#"{"error":"mdb_not_found"}"#.to_string(),
    };
    let conn = match Connection::open_with_flags(&mdb_path, OpenFlags::SQLITE_OPEN_READ_ONLY) {
        Ok(c) => c,
        Err(e) => return format!(r#"{{"error":"open_failed","detail":"{}"}}"#, e),
    };

    // 1. Dump single_mode_hint_gain: first 20 rows
    let mut hint_rows: Vec<String> = Vec::new();
    if let Ok(mut stmt) = conn.prepare(
        "SELECT id, hint_id, support_card_id, hint_group, hint_gain_type,          hint_value_1, hint_value_2, group_id, condition_set_id, priority          FROM single_mode_hint_gain ORDER BY id LIMIT 20"
    ) {
        hint_rows = stmt.query_map([], |row| {
            let id: i64 = row.get(0).unwrap_or(0);
            let hint_id: i64 = row.get(1).unwrap_or(0);
            let sc_id: i64 = row.get(2).unwrap_or(0);
            let hint_group: i64 = row.get(3).unwrap_or(0);
            let hint_gain_type: i64 = row.get(4).unwrap_or(0);
            let hv1: i64 = row.get(5).unwrap_or(0);
            let hv2: i64 = row.get(6).unwrap_or(0);
            let gid: i64 = row.get(7).unwrap_or(0);
            let csid: i64 = row.get(8).unwrap_or(-1);
            let pri: i64 = row.get(9).unwrap_or(0);
            Ok(format!(
                r#"{{"id":{},"hint_id":{},"sc_id":{},"hint_grp":{},"gain_type":{},"hv1":{},"hv2":{},"grp_id":{},"cond_set_id":{},"pri":{}}}"#,
                id, hint_id, sc_id, hint_group, hint_gain_type, hv1, hv2, gid, csid, pri
            ))
        }).unwrap().filter_map(|r| r.ok()).collect();
    }

    // 2. hint_gain_type distribution
    let mut type_dist: Vec<String> = Vec::new();
    if let Ok(mut stmt) = conn.prepare(
        "SELECT hint_gain_type, COUNT(*) as cnt FROM single_mode_hint_gain GROUP BY hint_gain_type ORDER BY cnt DESC"
    ) {
        type_dist = stmt.query_map([], |row| {
            let t: i64 = row.get(0).unwrap_or(0);
            let c: i64 = row.get(1).unwrap_or(0);
            Ok(format!(r#"{{"type":{},"count":{}}}"#, t, c))
        }).unwrap().filter_map(|r| r.ok()).collect();
    }

    // 3. Rows with non-zero condition_set_id (these have bond/skill prerequisites)
    let mut hint_with_cond: Vec<String> = Vec::new();
    if let Ok(mut stmt) = conn.prepare(
        "SELECT id, hint_id, support_card_id, hint_group, hint_gain_type, hint_value_1, hint_value_2, group_id, condition_set_id, priority FROM single_mode_hint_gain WHERE condition_set_id > 0 ORDER BY id LIMIT 30"
    ) {
        hint_with_cond = stmt.query_map([], |row| {
            let id: i64 = row.get(0).unwrap_or(0);
            let hint_id: i64 = row.get(1).unwrap_or(0);
            let sc_id: i64 = row.get(2).unwrap_or(0);
            let hint_group: i64 = row.get(3).unwrap_or(0);
            let hint_gain_type: i64 = row.get(4).unwrap_or(0);
            let hv1: i64 = row.get(5).unwrap_or(0);
            let hv2: i64 = row.get(6).unwrap_or(0);
            let gid: i64 = row.get(7).unwrap_or(0);
            let csid: i64 = row.get(8).unwrap_or(-1);
            let pri: i64 = row.get(9).unwrap_or(0);
            Ok(format!(
                r#"{{"id":{},"hint_id":{},"sc_id":{},"hint_grp":{},"gain_type":{},"hv1":{},"hv2":{},"grp_id":{},"cond_set_id":{},"pri":{}}}"#,
                id, hint_id, sc_id, hint_group, hint_gain_type, hv1, hv2, gid, csid, pri
            ))
        }).unwrap().filter_map(|r| r.ok()).collect();
    }

    // 4. Resolve condition_set_ids from hint_gain -> single_mode_story_condition_set
    let cs_ids: Vec<i64> = match conn.prepare(
        "SELECT DISTINCT condition_set_id FROM single_mode_hint_gain WHERE condition_set_id > 0 ORDER BY condition_set_id LIMIT 50"
    ) {
        Ok(mut stmt) => stmt.query_map([], |row| {
            Ok(row.get::<_, i64>(0).unwrap_or(0))
        }).unwrap().filter_map(|r| r.ok()).collect(),
        Err(_) => Vec::new(),
    };

    let mut cond_details: Vec<String> = Vec::new();
    for csid in &cs_ids {
        let mut entries: Vec<String> = Vec::new();
        if let Ok(mut stmt2) = conn.prepare(
            "SELECT id, group_id, story_id, condition_type, condition_story_id_1, gain_select_1, select_index_1, condition_story_id_2, gain_select_2, select_index_2 FROM single_mode_story_condition_set WHERE group_id = ?"
        ) {
            entries = stmt2.query_map([csid], |row| {
                let id: i64 = row.get(0).unwrap_or(0);
                let gid: i64 = row.get(1).unwrap_or(0);
                let sid: i64 = row.get(2).unwrap_or(0);
                let ctype: i64 = row.get(3).unwrap_or(0);
                let cs1: i64 = row.get(4).unwrap_or(0);
                let gs1: i64 = row.get(5).unwrap_or(0);
                let si1: i64 = row.get(6).unwrap_or(0);
                let cs2: i64 = row.get(7).unwrap_or(0);
                let gs2: i64 = row.get(8).unwrap_or(0);
                let si2: i64 = row.get(9).unwrap_or(0);
                Ok(format!(
                    r#"{{"id":{},"gid":{},"sid":{},"ctype":{},"cs1":{},"gs1":{},"si1":{},"cs2":{},"gs2":{},"si2":{}}}"#,
                    id, gid, sid, ctype, cs1, gs1, si1, cs2, gs2, si2
                ))
            }).unwrap().filter_map(|r| r.ok()).collect();
        }
        cond_details.push(format!(
            r#"{{"cond_set_id":{},"entries":[{}]}}"#,
            csid,
            entries.join(",")
        ));
    }

    // 5. Also check single_mode_unique_chara which links partner_id to unique skills
    let mut unique_chara: Vec<String> = Vec::new();
    if let Ok(mut stmt) = conn.prepare(
        "SELECT id, partner_id, scenario_id, chara_id, period, training_placement, gain_flag_id, is_support_featured_stock, gain_role_id FROM single_mode_unique_chara LIMIT 10"
    ) {
        unique_chara = stmt.query_map([], |row| {
            let id: i64 = row.get(0).unwrap_or(0);
            let pid: i64 = row.get(1).unwrap_or(0);
            let ssid: i64 = row.get(2).unwrap_or(0);
            let cid: i64 = row.get(3).unwrap_or(0);
            let per: i64 = row.get(4).unwrap_or(0);
            let tp: i64 = row.get(5).unwrap_or(0);
            let gfi: i64 = row.get(6).unwrap_or(0);
            let isfs: i64 = row.get(7).unwrap_or(0);
            let grid: i64 = row.get(8).unwrap_or(0);
            Ok(format!(
                r#"{{"id":{},"partner":{},"scenario":{},"chara":{},"period":{},"train_place":{},"gain_flag":{},"featured":{},"gain_role":{}}}"#,
                id, pid, ssid, cid, per, tp, gfi, isfs, grid
            ))
        }).unwrap().filter_map(|r| r.ok()).collect();
    }

    drop(conn);

    format!(
        r#"{{"ok":true,"version":"3.22.91","hint_gain_sample":[{}],"hint_gain_with_cond":[{}],"hint_gain_type_dist":[{}],"condition_set_resolved":[{}],"unique_chara_sample":[{}]}}"#,
        hint_rows.join(","),
        hint_with_cond.join(","),
        type_dist.join(","),
        cond_details.join(","),
        unique_chara.join(",")
    )
}

fn debug_mdb_all_tables() -> String {
    let mdb_path = match find_mdb_path() {
        Some(p) => p,
        None => return r#"{"error":"mdb_not_found"}"#.to_string(),
    };
    let conn = match Connection::open_with_flags(&mdb_path, OpenFlags::SQLITE_OPEN_READ_ONLY) {
        Ok(c) => c,
        Err(e) => return format!(r#"{{"error":"open_failed","detail":"{}"}}"#, e),
    };

    // Get ALL table names
    let all_tables: Vec<String> =
        match conn.prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name") {
            Ok(mut stmt) => stmt
                .query_map([], |row| Ok(row.get::<_, String>(0).unwrap_or_default()))
                .unwrap()
                .filter_map(|r| r.ok())
                .collect(),
            Err(e) => return format!(r#"{{"error":"table_list_failed","detail":"{}"}}"#, e),
        };

    // Get row counts for all tables
    let mut tables_json: Vec<String> = Vec::new();
    for name in &all_tables {
        let safe_name = name.replace("]", "]]");
        let count: i32 = conn
            .query_row(&format!("SELECT COUNT(*) FROM [{}]", safe_name), [], |r| {
                r.get(0)
            })
            .unwrap_or(0);
        tables_json.push(format!(
            r#"{{"name":"{}","rows":{}}}"#,
            json_escape(name),
            count
        ));
    }

    // Also search for condition/bond/prerequisite related tables
    let cond_keywords = [
        "bond",
        "cond",
        "acquire",
        "unlock",
        "require",
        "threshold",
        "prerequisite",
        "activate",
        "trigger",
        "learn",
        "skill_set",
        "skill_data",
        "skill_effect",
        "hint",
        "talent",
        "gain",
    ];
    let cond_tables: Vec<&String> = all_tables
        .iter()
        .filter(|t| {
            let tl = t.to_lowercase();
            cond_keywords.iter().any(|k| tl.contains(k))
        })
        .collect();

    // For condition-related tables, dump schema
    let mut cond_details: Vec<String> = Vec::new();
    for table_name in &cond_tables {
        let safe_name = table_name.replace("]", "]]");
        let col_names: Vec<String> =
            match conn.prepare(&format!("PRAGMA table_info([{}])", safe_name)) {
                Ok(mut stmt) => stmt
                    .query_map([], |row| Ok(row.get::<_, String>(1).unwrap_or_default()))
                    .unwrap()
                    .filter_map(|r| r.ok())
                    .collect(),
                Err(_) => Vec::new(),
            };
        cond_details.push(format!(
            r#"{{"table":"{}","columns":[{}]}}"#,
            json_escape(table_name),
            col_names
                .iter()
                .map(|c| format!(r#""{}""#, json_escape(c)))
                .collect::<Vec<_>>()
                .join(",")
        ));
    }

    drop(conn);

    format!(
        r#"{{"ok":true,"version":"3.22.91","total_tables":{},"all_tables":[{}],"cond_keyword_tables":{},"cond_table_schemas":[{}]}}"#,
        all_tables.len(),
        tables_json.join(","),
        cond_tables.len(),
        cond_details.join(",")
    )
}

/// /debug/mdb_schema_dump — 一键扒取所有表名+schema+关键表前20行样本
/// 专门为"纯安卓端无 Termux"场景设计，juece 调一次拿到全部信息
fn debug_mdb_schema_dump() -> String {
    let mdb_path = match find_mdb_path() {
        Some(p) => p,
        None => return r#"{"error":"mdb_not_found"}"#.to_string(),
    };
    let conn = match Connection::open_with_flags(&mdb_path, OpenFlags::SQLITE_OPEN_READ_ONLY) {
        Ok(c) => c,
        Err(e) => return format!(r#"{{"error":"open_failed","detail":"{}"}}"#, e),
    };

    // 1. 获取所有表名
    let all_tables: Vec<String> =
        match conn.prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name") {
            Ok(mut stmt) => stmt
                .query_map([], |row| Ok(row.get::<_, String>(0).unwrap_or_default()))
                .unwrap()
                .filter_map(|r| r.ok())
                .collect(),
            Err(e) => return format!(r#"{{"error":"table_list_failed","detail":"{}"}}"#, e),
        };

    // 2. 筛选关键表（按关键词）
    let key_keywords = [
        "goal", "target", "race", "fan", "career", "turn", "program",
        "condition", "saddle", "story", "event", "choice", "reward",
        "skill", "hint", "chara", "card", "support", "relation",
        "succession", "inherit", "factor", "gauge", "training",
        "single_mode", "text_data", "grade", "rank", "point",
    ];
    let key_lower = |t: &str| t.to_lowercase();
    let is_key_table = |name: &str| {
        let nl = key_lower(name);
        key_keywords.iter().any(|k| nl.contains(k))
    };

    // 3. 对关键表: schema + 前20行
    let mut tables_json: Vec<String> = Vec::new();
    for name in &all_tables {
        let safe_name = name.replace("]", "]]");
        let row_count: i32 = conn
            .query_row(&format!("SELECT COUNT(*) FROM [{}]", safe_name), [], |r| {
                r.get(0)
            })
            .unwrap_or(0);

        let is_key = is_key_table(name);

        // schema
        let schema_json: String = if is_key {
            match conn.prepare(&format!("PRAGMA table_info([{}])", safe_name)) {
                Ok(mut stmt) => {
                    let cols: Vec<String> = stmt
                        .query_map([], |row| {
                            Ok(format!(
                                r#"{{"name":"{}","type":"{}","notnull":{}}}"#,
                                json_escape(&row.get::<_, String>(1).unwrap_or_default()),
                                json_escape(&row.get::<_, String>(2).unwrap_or_default()),
                                row.get::<_, i32>(3).unwrap_or(0),
                            ))
                        })
                        .unwrap()
                        .filter_map(|r| r.ok())
                        .collect();
                    format!("[{}]", cols.join(","))
                }
                Err(_) => "[]".to_string(),
            }
        } else {
            "[]".to_string()
        };

        // sample rows (only for key tables, max 20)
        let sample_json: String = if is_key && row_count > 0 {
            match conn.prepare(&format!(
                "SELECT * FROM [{}] LIMIT 20",
                safe_name
            )) {
                Ok(mut stmt) => {
                    let col_count = stmt.column_count();
                    // Get column names
                    let col_names: Vec<String> = (0..col_count)
                        .map(|i| stmt.column_name(i).unwrap_or("").to_string())
                        .collect();
                    let rows: Vec<String> = stmt
                        .query_map([], |row| {
                            let mut pairs: Vec<String> = Vec::new();
                            for ci in 0..col_count {
                                let cn = col_names.get(ci).cloned().unwrap_or_default();
                                let int_val: Option<i64> = row.get(ci).ok();
                                let val = if let Some(v) = int_val {
                                    v.to_string()
                                } else {
                                    let str_val: Option<String> = row.get(ci).ok();
                                    match str_val {
                                        Some(s) => format!(r#""{}""#, json_escape(&s)),
                                        None => {
                                            let float_val: Option<f64> = row.get(ci).ok();
                                            match float_val {
                                                Some(f) => format!("{}", f),
                                                None => "null".to_string(),
                                            }
                                        }
                                    }
                                };
                                pairs.push(format!(r#""{}":{}"#, json_escape(&cn), val));
                            }
                            Ok(format!(r#"{{{}}}"#, pairs.join(",")))
                        })
                        .unwrap()
                        .filter_map(|r| r.ok())
                        .collect();
                    format!("[{}]", rows.join(","))
                }
                Err(_) => "[]".to_string(),
            }
        } else {
            "[]".to_string()
        };

        tables_json.push(format!(
            r#"{{"name":"{}","rows":{},"key":{},"schema":{},"sample":{}}}"#,
            json_escape(name),
            row_count,
            is_key,
            schema_json,
            sample_json,
        ));
    }

    // 4. 额外查询: single_mode_turn 的 race_entry_type 分布
    let turn_dist: Vec<String> = match conn.prepare(
        "SELECT race_entry_type, COUNT(*) FROM single_mode_turn GROUP BY race_entry_type ORDER BY race_entry_type"
    ) {
        Ok(mut stmt) => stmt
            .query_map([], |row| {
                Ok(format!(
                    r#"{{"type":{},"count":{}}}"#,
                    row.get::<_, i32>(0).unwrap_or(0),
                    row.get::<_, i32>(1).unwrap_or(0),
                ))
            })
            .unwrap()
            .filter_map(|r| r.ok())
            .collect(),
        Err(_) => Vec::new(),
    };

    drop(conn);

    format!(
        r#"{{"ok":true,"total_tables":{},"key_table_count":{},"tables":[{}],"turn_race_entry_dist":[{}]}}"#,
        all_tables.len(),
        tables_json.iter().filter(|t| t.contains("\"key\":true")).count(),
        tables_json.join(","),
        turn_dist.join(","),
    )
}

/// /mdb/dl_batch?prefix=X — 按首字母批量导出 MDB 表数据为 JSON
/// prefix=a → 所有 a 开头的表; prefix=all → 全部表
/// 每张表最多 500 行（防止 OOM），大表只取前 500 行
fn mdb_dl_batch(prefix: &str) -> String {
    let mdb_path = match find_mdb_path() {
        Some(p) => p,
        None => return r#"{"error":"mdb_not_found"}"#.to_string(),
    };
    let conn = match Connection::open_with_flags(&mdb_path, OpenFlags::SQLITE_OPEN_READ_ONLY) {
        Ok(c) => c,
        Err(e) => return format!(r#"{{"error":"open_failed","detail":"{}"}}"#, e),
    };

    // 1. 获取所有表名
    let all_tables: Vec<String> = match conn.prepare(
        "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name"
    ) {
        Ok(mut stmt) => stmt
            .query_map([], |row| Ok(row.get::<_, String>(0).unwrap_or_default()))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect(),
        Err(e) => return format!(r#"{{"error":"table_list_failed","detail":"{}"}}"#, e),
    };

    // 2. 按前缀筛选
    let prefix_lower = prefix.to_lowercase();
    let is_all = prefix_lower == "all" || prefix.is_empty();
    let filtered: Vec<&String> = if is_all {
        all_tables.iter().collect()
    } else {
        all_tables
            .iter()
            .filter(|t| t.to_lowercase().starts_with(&prefix_lower))
            .collect()
    };

    if filtered.is_empty() {
        return format!(
            r#"{{"error":"no_tables","prefix":"{}"}}"#,
            json_escape(prefix)
        );
    }

    // 3. 逐表查询（最多 500 行/表）
    let max_rows_per_table = 500;
    let mut tables_json: Vec<String> = Vec::new();
    let mut total_rows: usize = 0;

    for table_name in &filtered {
        let safe_name = table_name.replace("]", "]]");
        let count: i64 = conn
            .query_row(&format!("SELECT COUNT(*) FROM [{}]", safe_name), [], |r| r.get(0))
            .unwrap_or(0);

        // 获取列名
        let col_names: Vec<String> = match conn.prepare(&format!("SELECT * FROM [{}] LIMIT 1", safe_name)) {
            Ok(mut stmt) => stmt.column_names().iter().map(|s| s.to_string()).collect(),
            Err(_) => Vec::new(),
        };

        // 查询数据
        let limit = (count as usize).min(max_rows_per_table);
        let rows_json: Vec<String> = if limit > 0 {
            match conn.prepare(&format!("SELECT * FROM [{}] LIMIT {}", safe_name, limit)) {
                Ok(mut stmt) => {
                    let col_count = col_names.len();
                    stmt.query_map([], |row| {
                        let mut pairs: Vec<String> = Vec::new();
                        for ci in 0..col_count {
                            let cn = col_names.get(ci).unwrap_or(&String::new()).clone();
                            let int_val: Option<i64> = row.get::<_, Option<i64>>(ci).unwrap_or(None);
                            let val = if let Some(v) = int_val {
                                v.to_string()
                            } else {
                                let str_val: Option<String> = row.get::<_, Option<String>>(ci).unwrap_or(None);
                                match str_val {
                                    Some(s) => format!(r#""{}""#, json_escape(&s)),
                                    None => {
                                        let float_val: Option<f64> = row.get::<_, Option<f64>>(ci).unwrap_or(None);
                                        match float_val {
                                            Some(f) => format!("{}", f),
                                            None => "null".to_string(),
                                        }
                                    }
                                }
                            };
                            pairs.push(format!(r#""{}":{}"#, json_escape(&cn), val));
                        }
                        Ok(format!(r#"{{{}}}"#, pairs.join(",")))
                    })
                    .unwrap()
                    .filter_map(|r| r.ok())
                    .collect()
                }
                Err(e) => {
                    vec![format!(r#"{{"error":"query_failed","detail":"{}"}}"#, json_escape(&e.to_string()))]
                }
            }
        } else {
            Vec::new()
        };

        total_rows += rows_json.len();
        tables_json.push(format!(
            r#""{}":{{"rows":{},"queried":{},"columns":[{}],"data":[{}]}}"#,
            json_escape(table_name),
            count,
            rows_json.len(),
            col_names.iter().map(|c| format!(r#""{}""#, json_escape(c))).collect::<Vec<_>>().join(","),
            rows_json.join(",")
        ));
    }

    format!(
        r#"{{"ok":true,"prefix":"{}","table_count":{},"total_rows_queried":{},"max_rows_per_table":{},"tables":{{{}}}}}"#,
        json_escape(prefix),
        filtered.len(),
        total_rows,
        max_rows_per_table,
        tables_json.join(",")
    )
}

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
        let count: i32 = conn
            .query_row(&format!("SELECT COUNT(*) FROM [{}]", safe_name), [], |r| {
                r.get(0)
            })
            .unwrap_or(0);
        tables_json.push(format!(
            r#"{{"name":"{}","rows":{}}}"#,
            json_escape(name),
            count
        ));
    }

    let mut event_json: Vec<String> = Vec::new();
    for name in &event_names {
        let safe_name = name.replace("]", "]]");
        let count: i32 = conn
            .query_row(&format!("SELECT COUNT(*) FROM [{}]", safe_name), [], |r| {
                r.get(0)
            })
            .unwrap_or(0);
        event_json.push(format!(
            r#"{{"name":"{}","rows":{}}}"#,
            json_escape(name),
            count
        ));
    }

    drop(conn);

    format!(
        r#"{{"ok":true,"single_mode_tables":[{}],"event_tables":[{}]}}"#,
        tables_json.join(","),
        event_json.join(","),
    )
}
// ============================================================
// v3.22.89: MDB增强端点
// /mdb/schema?name=X — 表结构（列名+类型+行数）
// /mdb/search?keyword=X — 搜索所有表名和列名
// /mdb/raw?sql=X — 执行只读SQL（带LIMIT安全限制）
// ============================================================

/// /mdb/schema?name=X — 获取表结构
/// 返回列名、类型、是否可空、行数
fn mdb_schema(table_name: &str) -> String {
    if table_name.is_empty() {
        return r#"{"ok":false,"error":"missing ?name= parameter"}"#.to_string();
    }
    // 安全检查：表名只允许字母数字下划线
    if table_name
        .chars()
        .any(|c| !c.is_alphanumeric() && c != '_' && c != '-')
    {
        return format!(r#"{{"ok":false,"error":"invalid_table_name"}}"#);
    }
    let mdb_path = match find_mdb_path() {
        Some(p) => p,
        None => return r#"{"ok":false,"error":"mdb_not_found"}"#.to_string(),
    };
    let conn = match Connection::open_with_flags(&mdb_path, OpenFlags::SQLITE_OPEN_READ_ONLY) {
        Ok(c) => c,
        Err(e) => return format!(r#"{{"ok":false,"error":"open_failed","detail":"{}"}}"#, e),
    };

    let safe_name = table_name.replace("]", "]]");

    // PRAGMA table_info: cid, name, type, notnull, dflt_value, pk
    let cols: Vec<String> = match conn.prepare(&format!("PRAGMA table_info([{}])", safe_name)) {
        Ok(mut stmt) => stmt
            .query_map([], |row| {
                let cid: i32 = row.get(0).unwrap_or(0);
                let name: String = row.get(1).unwrap_or_default();
                let col_type: String = row.get(2).unwrap_or_default();
                let notnull: i32 = row.get(3).unwrap_or(0);
                let pk: i32 = row.get(5).unwrap_or(0);
                Ok(format!(
                    r#"{{"cid":{},"name":"{}","type":"{}","notnull":{},"pk":{}}}"#,
                    cid,
                    json_escape(&name),
                    json_escape(&col_type),
                    notnull,
                    pk
                ))
            })
            .ok()
            .map(|iter| iter.filter_map(|r| r.ok()).collect())
            .unwrap_or_default(),
        Err(e) => return format!(r#"{{"ok":false,"error":"schema_failed","detail":"{}"}}"#, e),
    };

    if cols.is_empty() {
        return format!(
            r#"{{"ok":false,"error":"table_not_found","table":"{}"}}"#,
            json_escape(table_name)
        );
    }

    // 行数
    let row_count: i64 = conn
        .query_row(&format!("SELECT COUNT(*) FROM [{}]", safe_name), [], |r| {
            r.get(0)
        })
        .unwrap_or(0);

    // 索引信息
    let indexes: Vec<String> = match conn.prepare(&format!("PRAGMA index_list([{}])", safe_name)) {
        Ok(mut stmt) => stmt
            .query_map([], |row| {
                let idx_name: String = row.get(1).unwrap_or_default();
                let unique: i32 = row.get(2).unwrap_or(0);
                Ok(format!(
                    r#"{{"name":"{}","unique":{}}}"#,
                    json_escape(&idx_name),
                    unique
                ))
            })
            .ok()
            .map(|iter| iter.filter_map(|r| r.ok()).collect())
            .unwrap_or_default(),
        Err(_) => Vec::new(),
    };

    format!(
        r#"{{"ok":true,"table":"{}","row_count":{},"columns":[{}],"indexes":[{}]}}"#,
        json_escape(table_name),
        row_count,
        cols.join(","),
        indexes.join(",")
    )
}

/// /mdb/search?keyword=X — 搜索所有表名和列名
/// 返回匹配的表名和列名
fn mdb_search(keyword: &str) -> String {
    if keyword.is_empty() {
        return r#"{"ok":false,"error":"missing ?keyword= parameter"}"#.to_string();
    }
    let mdb_path = match find_mdb_path() {
        Some(p) => p,
        None => return r#"{"ok":false,"error":"mdb_not_found"}"#.to_string(),
    };
    let conn = match Connection::open_with_flags(&mdb_path, OpenFlags::SQLITE_OPEN_READ_ONLY) {
        Ok(c) => c,
        Err(e) => return format!(r#"{{"ok":false,"error":"open_failed","detail":"{}"}}"#, e),
    };

    // 获取所有表名
    let table_names: Vec<String> =
        match conn.prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name") {
            Ok(mut stmt) => stmt
                .query_map([], |row| Ok(row.get::<_, String>(0).unwrap_or_default()))
                .unwrap()
                .filter_map(|r| r.ok())
                .collect(),
            Err(e) => {
                return format!(
                    r#"{{"ok":false,"error":"table_list_failed","detail":"{}"}}"#,
                    e
                )
            }
        };

    let kw_lower = keyword.to_lowercase();
    let mut matched_tables: Vec<String> = Vec::new();
    let mut matched_columns: Vec<String> = Vec::new();

    for tname in &table_names {
        // 表名匹配
        if tname.to_lowercase().contains(&kw_lower) {
            let safe_name = tname.replace("]", "]]");
            let count: i64 = conn
                .query_row(&format!("SELECT COUNT(*) FROM [{}]", safe_name), [], |r| {
                    r.get(0)
                })
                .unwrap_or(0);
            matched_tables.push(format!(
                r#"{{"table":"{}","rows":{}}}"#,
                json_escape(tname),
                count
            ));
        }

        // 列名匹配
        let safe_name = tname.replace("]", "]]");
        if let Ok(mut stmt) = conn.prepare(&format!("PRAGMA table_info([{}])", safe_name)) {
            if let Ok(rows) = stmt.query_map([], |row| {
                let col_name: String = row.get(1).unwrap_or_default();
                let col_type: String = row.get(2).unwrap_or_default();
                Ok((col_name, col_type))
            }) {
                for r in rows.filter_map(|r| r.ok()) {
                    if r.0.to_lowercase().contains(&kw_lower) {
                        matched_columns.push(format!(
                            r#"{{"table":"{}","column":"{}","type":"{}"}}"#,
                            json_escape(tname),
                            json_escape(&r.0),
                            json_escape(&r.1)
                        ));
                    }
                }
            }
        }
    }

    format!(
        r#"{{"ok":true,"keyword":"{}","matched_tables":{},"matched_columns":{},"tables":[{}],"columns":[{}]}}"#,
        json_escape(keyword),
        matched_tables.len(),
        matched_columns.len(),
        matched_tables.join(","),
        matched_columns.join(",")
    )
}

/// /mdb/raw?sql=X — 执行只读SQL查询
/// 安全限制：只允许SELECT，强制LIMIT，最多1000行
fn mdb_raw_query(sql: &str) -> String {
    if sql.is_empty() {
        return r#"{"ok":false,"error":"missing ?sql= parameter"}"#.to_string();
    }

    // 安全检查：只允许SELECT语句
    let sql_trimmed = sql.trim();
    if !sql_trimmed.to_uppercase().starts_with("SELECT") {
        return r#"{"ok":false,"error":"only_select_allowed"}"#.to_string();
    }
    // 禁止危险关键词
    let upper = sql_trimmed.to_uppercase();
    for danger in &[
        "DROP", "DELETE", "INSERT", "UPDATE", "ALTER", "ATTACH", "DETACH", "PRAGMA",
    ] {
        if upper.contains(danger) {
            return format!(
                r#"{{"ok":false,"error":"forbidden_keyword","keyword":"{}"}}"#,
                danger
            );
        }
    }

    let mdb_path = match find_mdb_path() {
        Some(p) => p,
        None => return r#"{"ok":false,"error":"mdb_not_found"}"#.to_string(),
    };
    let conn = match Connection::open_with_flags(&mdb_path, OpenFlags::SQLITE_OPEN_READ_ONLY) {
        Ok(c) => c,
        Err(e) => return format!(r#"{{"ok":false,"error":"open_failed","detail":"{}"}}"#, e),
    };

    // 如果没有LIMIT，自动加上
    let final_sql = if upper.contains("LIMIT") {
        sql_trimmed.to_string()
    } else {
        format!("{} LIMIT 1000", sql_trimmed)
    };

    // 用 if let Ok 模式避免 conn 生命周期问题（与现有代码风格一致）
    let mut col_names: Vec<String> = Vec::new();
    let mut rows: Vec<String> = Vec::new();
    let mut prepare_ok = false;

    if let Ok(mut stmt) = conn.prepare(&final_sql) {
        prepare_ok = true;
        let col_count = stmt.column_count();
        col_names = (0..col_count)
            .map(|i| stmt.column_name(i).unwrap_or("?").to_string())
            .collect();

        // query_map 返回的 Rows 借用 stmt，用 if let Ok 消费后释放借用
        if let Ok(mapped) = stmt.query_map([], |row| {
            let mut pairs: Vec<String> = Vec::new();
            for ci in 0..col_count {
                let cn = col_names.get(ci).unwrap_or(&String::new()).clone();
                let int_val = row.get::<_, Option<i64>>(ci).unwrap_or(None);
                let val = if let Some(v) = int_val {
                    v.to_string()
                } else {
                    let str_val = row.get::<_, Option<String>>(ci).unwrap_or(None);
                    match str_val {
                        Some(s) => format!(r#""{}""#, json_escape(&s)),
                        None => {
                            // 尝试读f64
                            let float_val = row.get::<_, Option<f64>>(ci).unwrap_or(None);
                            match float_val {
                                Some(f) => format!("{}", f),
                                None => "null".to_string(),
                            }
                        }
                    }
                };
                pairs.push(format!(r#""{}":{}"#, json_escape(&cn), val));
            }
            Ok(format!(r#"{{{}}}"#, pairs.join(",")))
        }) {
            rows = mapped.filter_map(|r| r.ok()).collect();
        }
    }
    // stmt 在这里已 drop，conn 的借用释放

    if !prepare_ok {
        return format!(r#"{{"ok":false,"error":"query_failed"}}"#);
    }

    let col_json: Vec<String> = col_names
        .iter()
        .map(|c| format!(r#""{}""#, json_escape(c)))
        .collect();
    format!(
        r#"{{"ok":true,"sql":"{}","columns":[{}],"row_count":{},"rows":[{}]}}"#,
        json_escape(&final_sql),
        col_json.join(","),
        rows.len(),
        rows.join(",")
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
    // ★ v3.22.51: Use SELECT * + PRAGMA table_info for schema resilience
    //   Game updates may add/remove columns; hardcoded column lists break.
    let story_col_names: Vec<String> =
        match conn.prepare("PRAGMA table_info(single_mode_story_data)") {
            Ok(mut stmt) => stmt
                .query_map([], |row| Ok(row.get::<_, String>(1).unwrap_or_default()))
                .ok()
                .map(|iter| iter.filter_map(|r| r.ok()).collect())
                .unwrap_or_default(),
            Err(_) => Vec::new(),
        };

    let stories: Vec<String> = if story_col_names.is_empty() {
        Vec::new()
    } else {
        match conn.prepare("SELECT * FROM single_mode_story_data ORDER BY id") {
            Ok(mut stmt) => stmt
                .query_map([], |row| {
                    let mut parts: Vec<String> = Vec::new();
                    for (i, col) in story_col_names.iter().enumerate() {
                        let val_str = match row.get::<_, i32>(i) {
                            Ok(v) => v.to_string(),
                            Err(_) => match row.get::<_, String>(i) {
                                Ok(s) => format!("\"{}\"", json_escape(&s)),
                                Err(_) => "null".to_string(),
                            },
                        };
                        parts.push(format!("\"{}\":{}", col, val_str));
                    }
                    Ok(format!("{{{}}}", parts.join(",")))
                })
                .unwrap()
                .filter_map(|r| r.ok())
                .collect(),
            Err(e) => return format!(r#"{{"error":"story_query_failed","detail":"{}"}}"#, e),
        }
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
    let titles: Vec<String> = match conn.prepare(&format!(
        "SELECT [index], text FROM text_data WHERE category={} ORDER BY [index]",
        TEXT_DATA_CATEGORY_STORY_TITLE
    )) {
        Ok(mut stmt) => stmt
            .query_map([], |row| {
                let text: String = row
                    .get::<_, Option<String>>(1)
                    .unwrap_or(None)
                    .unwrap_or_default();
                Ok(format!(
                    r#"{{"id":{},"title":"{}"}}"#,
                    row.get::<_, i32>(0).unwrap_or(0),
                    json_escape(&text),
                ))
            })
            .unwrap()
            .filter_map(|r| r.ok())
            .collect(),
        Err(_) => Vec::new(),
    };

    drop(conn);

    format!(
        r#"{{"ok":true,"version":"3.22.91","story_count":{},"choice_count":{},"gain_count":{},"title_count":{},"stories":[{}],"choices":[{}],"gains":[{}],"titles":[{}]}}"#,
        stories.len(),
        choices.len(),
        gains.len(),
        titles.len(),
        stories.join(","),
        choices.join(","),
        gains.join(","),
        titles.join(","),
    )
}

fn read_carddb() -> String {
    let mdb_path = match find_mdb_path() {
        Some(p) => p,
        None => {
            return r#"{"error":"mdb_not_found","hint":"MasterDB file not found on device"}"#
                .to_string()
        }
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
        r#"{{"ok":true,"version":"3.22.91","mdb":"{}","card_count":{},"effect_count":{},"cards":[{}],"effects":[{}]}}"#,
        mdb_path,
        cards.len(),
        effects.len(),
        cards.join(","),
        effects.join(",")
    )
}

/// /skilldata - Read skill data from MasterDB via rusqlite
fn read_skilldata() -> String {
    let mdb_path = match find_mdb_path() {
        Some(p) => p,
        None => {
            return r#"{"error":"mdb_not_found","hint":"MasterDB file not found on device"}"#
                .to_string()
        }
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
    let names: Vec<String> = match conn.prepare(&format!(
        "SELECT id, text FROM text_data WHERE category={} ORDER BY id",
        TEXT_DATA_CATEGORY_SKILL_NAME
    )) {
        Ok(mut stmt) => stmt
            .query_map([], |row| {
                let text: String = row
                    .get::<_, Option<String>>(1)
                    .unwrap_or(None)
                    .unwrap_or_default();
                Ok(format!(
                    r#"{{"id":{},"name":"{}"}}"#,
                    row.get::<_, i32>(0).unwrap_or(0),
                    json_escape(&text),
                ))
            })
            .unwrap()
            .filter_map(|r| r.ok())
            .collect(),
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
        r#"{{"ok":true,"version":"3.22.91","mdb":"{}","skill_count":{},"name_count":{},"point_count":{},"skills":[{}],"names":[{}],"need_points":[{}]}}"#,
        mdb_path,
        skills.len(),
        names.len(),
        points.len(),
        skills.join(","),
        names.join(","),
        points.join(",")
    )
}

/// /saddles - Read G1 win saddle data from MasterDB for compatibility verification
/// Returns: win saddle groups with relation_group_id (5th anniversary field)
fn read_saddles() -> String {
    let mdb_path = match find_mdb_path() {
        Some(p) => p,
        None => {
            return r#"{"error":"mdb_not_found","hint":"MasterDB file not found on device"}"#
                .to_string()
        }
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
    let race_names: Vec<String> = match conn.prepare(&format!(
        "SELECT [index], text FROM text_data WHERE category={} ORDER BY [index]",
        TEXT_DATA_CATEGORY_RACE_NAME
    )) {
        Ok(mut stmt) => stmt
            .query_map([], |row| {
                let text: String = row
                    .get::<_, Option<String>>(1)
                    .unwrap_or(None)
                    .unwrap_or_default();
                Ok(format!(
                    r#"{{"race_id":{},"name":"{}"}}"#,
                    row.get::<_, i32>(0).unwrap_or(0),
                    json_escape(&text),
                ))
            })
            .unwrap()
            .filter_map(|r| r.ok())
            .collect(),
        Err(e) => return format!(r#"{{"error":"race_name_prepare_failed","detail":"{}"}}"#, e),
    };

    // Collect chara names (category=6 = chara name in text_data)
    let chara_names: Vec<String> = match conn.prepare(&format!(
        "SELECT [index], text FROM text_data WHERE category={} ORDER BY [index]",
        TEXT_DATA_CATEGORY_CHARA_NAME
    )) {
        Ok(mut stmt) => stmt
            .query_map([], |row| {
                let text: String = row
                    .get::<_, Option<String>>(1)
                    .unwrap_or(None)
                    .unwrap_or_default();
                Ok(format!(
                    r#"{{"chara_id":{},"name":"{}"}}"#,
                    row.get::<_, i32>(0).unwrap_or(0),
                    json_escape(&text),
                ))
            })
            .unwrap()
            .filter_map(|r| r.ok())
            .collect(),
        Err(e) => {
            return format!(
                r#"{{"error":"chara_name_prepare_failed","detail":"{}"}}"#,
                e
            )
        }
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
        r#"{{"ok":true,"version":"3.22.91","mdb":"{}","saddle_count":{},"program_chara_count":{},"program_count":{},"race_name_count":{},"chara_name_count":{},"relation_count":{},"member_count":{},"race_instance_count":{},"saddles":[{}],"chara_programs":[{}],"programs":[{}],"race_names":[{}],"chara_names":[{}],"relations":[{}],"relation_members":[{}],"race_instances":[{}]}}"#,
        mdb_path,
        saddles.len(),
        chara_programs.len(),
        programs.len(),
        race_names.len(),
        chara_names.len(),
        relations.len(),
        relation_members.len(),
        race_instances.len(),
        saddles.join(","),
        chara_programs.join(","),
        programs.join(","),
        race_names.join(","),
        chara_names.join(","),
        relations.join(","),
        relation_members.join(","),
        race_instances.join(","),
    )
}

/// /hall - Read 殿堂 (Hall of Fame) data via TrainedCharaData
/// Path: WDM -> get_TrainedCharaData -> WorkTrainedCharaData -> get_List -> List<TrainedCharaData>
/// Each TrainedCharaData has get_RankScore (評価点), get_Speed/Stamina/Power/Guts/Wiz, etc.
/// _rankScore is the game's own calculated評価点 (gold standard for verification)
unsafe fn read_hall_data() -> String {
    if API.is_null() {
        return r#"{"error":"api_null"}"#.to_string();
    }
    let image = match get_image() {
        img if !img.is_null() => img,
        _ => return r#"{"error":"image_null"}"#.to_string(),
    };

    // 1. Get WDM singleton
    let wdm_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkDataManager").as_ptr(),
    );
    if wdm_class.is_null() {
        return r#"{"error":"no_wdm"}"#.to_string();
    }
    let wdm_inst = get_singleton(wdm_class);
    if wdm_inst.is_null() {
        return r#"{"error":"no_wdm_inst"}"#.to_string();
    }

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
    let items_arr = std::ptr::read_unaligned::<*mut c_void>(
        list_base.add(IL2CPP_LIST_ARRAY_OFF) as *const *mut c_void
    );
    let list_size =
        std::ptr::read_unaligned::<usize>(list_base.add(IL2CPP_LIST_COUNT_OFF) as *const usize)
            as i32;

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
    let arr_len =
        std::ptr::read_unaligned::<usize>(arr_base.add(IL2CPP_LIST_COUNT_OFF) as *const usize);

    let mut entries = Vec::new();
    let count = std::cmp::min(list_size as usize, std::cmp::min(arr_len, 200));

    for i in 0..count {
        let elem_ptr = std::ptr::read_unaligned::<*mut c_void>(
            arr_base.add(IL2CPP_LIST_ITEMS_OFF + i * IL2CPP_LIST_ITEM_SIZE) as *const *mut c_void,
        );
        if elem_ptr.is_null() {
            continue;
        }

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
        if speed <= 0 && stamina <= 0 && rank_score <= 0 {
            continue;
        }

        entries.push(format!(
            r#"{{"idx":{},"card_id":{},"speed":{},"stamina":{},"power":{},"guts":{},"wiz":{},"rank_score":{},"rank":{},"scenario_id":{},"fans":{},"rarity":{}}}"#,
            i, card_id, speed, stamina, power, guts, wiz, rank_score, rank, scenario_id, fans, rarity
        ));
    }

    if entries.is_empty() {
        return r#"{"error":"no_valid_entries"}"#.to_string();
    }

    ura_log(2, &format!("/hall: {} valid entries", entries.len()));
    format!(
        r#"{{"count":{},"entries":[{}]}}"#,
        entries.len(),
        entries.join(",")
    )
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

/// v3.22.51: /debug/dumpclass","/debug/storydata?name=ClassName — Dump all fields of any IL2CPP class
/// Uses il2cpp_class_get_fields (metadata only, no runtime_invoke)
unsafe fn debug_dumpclass(class_name: &str) -> String {
    if class_name.is_empty() {
        return r#"{"error":"missing ?name= parameter"}"#.to_string();
    }
    if API.is_null() {
        return r#"{"error":"api_null"}"#.to_string();
    }
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
            std::ffi::CStr::from_ptr(name_ptr)
                .to_string_lossy()
                .to_string()
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    // Enumerate all fields (including parent classes)
    let fields_json = enumerate_class_fields(class);

    // Also enumerate methods (for debugging)
    let methods_json = enumerate_class_methods(class);

    format!(
        r#"{{"requested":"{}","found":"{}","fields":{},"methods":{}}}"#,
        class_name, real_name, fields_json, methods_json
    )
}
// ============================================================
// v3.22.89: IL2CPP运行时值dump — 新增端点
// /il2cpp/dump?name=X  — dump单例实例，带运行时字段值
// /il2cpp/call?class=X&method=Y — 调用单例上的getter
// /il2cpp/tree?name=X&depth=N — 递归dump引用类型字段
// /il2cpp/field?class=X&field=Y — 读单个字段值
// ============================================================

// IL2CPP类型枚举（常用子集）
const IL2CPP_TYPE_BOOLEAN: u8 = 2;
const IL2CPP_TYPE_I1: u8 = 3;
const IL2CPP_TYPE_U1: u8 = 4;
const IL2CPP_TYPE_I2: u8 = 5;
const IL2CPP_TYPE_U2: u8 = 6;
const IL2CPP_TYPE_I4: u8 = 7;
const IL2CPP_TYPE_U4: u8 = 8;
const IL2CPP_TYPE_I8: u8 = 9;
const IL2CPP_TYPE_U8: u8 = 10;
const IL2CPP_TYPE_R4: u8 = 11;
const IL2CPP_TYPE_R8: u8 = 12;
const IL2CPP_TYPE_STRING: u8 = 14;
const IL2CPP_TYPE_PTR: u8 = 15;
const IL2CPP_TYPE_VALUETYPE: u8 = 17;
const IL2CPP_TYPE_CLASS: u8 = 18;
const IL2CPP_TYPE_GENERICINST: u8 = 21;

/// 读取IL2CPP字段类型枚举值
/// 优先用il2cpp_type_get_type函数，fallback直接读结构体
unsafe fn il2cpp_type_get_type_enum(type_ptr: *const c_void) -> u8 {
    if type_ptr.is_null() {
        return 0;
    }
    let get_type_fn = resolve_il2cpp_symbol("il2cpp_type_get_type");
    if !get_type_fn.is_null() {
        let get_type: unsafe extern "C" fn(*const c_void) -> i32 = std::mem::transmute(get_type_fn);
        return get_type(type_ptr) as u8;
    }
    // Fallback: Il2CppType布局 data(8)+attrs(2)+type(1) = offset 10
    std::ptr::read_unaligned::<u8>((type_ptr as *const u8).add(10))
}

/// 读取对象字段值并返回JSON表示
/// 根据IL2CPP类型枚举自动选择读取方式
unsafe fn read_field_value_json(
    obj: *const c_void,
    offset: i32,
    type_ptr: *const c_void,
) -> String {
    if obj.is_null() {
        return "null".to_string();
    }
    let base = obj as *const u8;
    let off = offset as usize;
    let type_enum = il2cpp_type_get_type_enum(type_ptr);

    match type_enum {
        IL2CPP_TYPE_BOOLEAN => {
            let v = std::ptr::read_unaligned::<u8>(base.add(off) as *const u8);
            if v != 0 {
                "true".to_string()
            } else {
                "false".to_string()
            }
        }
        IL2CPP_TYPE_I1 => {
            let v = std::ptr::read_unaligned::<i8>(base.add(off) as *const i8);
            v.to_string()
        }
        IL2CPP_TYPE_U1 => {
            let v = std::ptr::read_unaligned::<u8>(base.add(off) as *const u8);
            v.to_string()
        }
        IL2CPP_TYPE_I2 => {
            let v = std::ptr::read_unaligned::<i16>(base.add(off) as *const i16);
            v.to_string()
        }
        IL2CPP_TYPE_U2 => {
            let v = std::ptr::read_unaligned::<u16>(base.add(off) as *const u16);
            v.to_string()
        }
        IL2CPP_TYPE_I4 | IL2CPP_TYPE_VALUETYPE => {
            // 值类型(含enum)通常4字节
            let v = std::ptr::read_unaligned::<i32>(base.add(off) as *const i32);
            v.to_string()
        }
        IL2CPP_TYPE_U4 => {
            let v = std::ptr::read_unaligned::<u32>(base.add(off) as *const u32);
            v.to_string()
        }
        IL2CPP_TYPE_I8 => {
            let v = std::ptr::read_unaligned::<i64>(base.add(off) as *const i64);
            v.to_string()
        }
        IL2CPP_TYPE_U8 => {
            let v = std::ptr::read_unaligned::<u64>(base.add(off) as *const u64);
            v.to_string()
        }
        IL2CPP_TYPE_R4 => {
            let v = std::ptr::read_unaligned::<f32>(base.add(off) as *const f32);
            format!("{}", v)
        }
        IL2CPP_TYPE_R8 => {
            let v = std::ptr::read_unaligned::<f64>(base.add(off) as *const f64);
            format!("{}", v)
        }
        IL2CPP_TYPE_STRING => {
            let ptr_val =
                std::ptr::read_unaligned::<*const c_void>(base.add(off) as *const *const c_void);
            if ptr_val.is_null() {
                "null".to_string()
            } else {
                let s = read_il2cpp_string(ptr_val);
                format!(r#""{}""#, json_escape(&s))
            }
        }
        IL2CPP_TYPE_CLASS | IL2CPP_TYPE_GENERICINST => {
            // 引用类型：读指针，获取类名
            let ptr_val =
                std::ptr::read_unaligned::<*const c_void>(base.add(off) as *const *const c_void);
            if ptr_val.is_null() {
                "null".to_string()
            } else {
                let class_name = get_object_class_name(ptr_val);
                format!(
                    r#"{{"type":"ref","class":"{}","ptr":"0x{:x}"}}"#,
                    json_escape(&class_name),
                    ptr_val as usize
                )
            }
        }
        IL2CPP_TYPE_PTR => {
            let ptr_val =
                std::ptr::read_unaligned::<*const c_void>(base.add(off) as *const *const c_void);
            if ptr_val.is_null() {
                "null".to_string()
            } else {
                format!(r#""0x{:x}""#, ptr_val as usize)
            }
        }
        _ => {
            // 未知类型，读8字节原始数据
            let v = std::ptr::read_unaligned::<i64>(base.add(off) as *const i64);
            format!(r#"{{"type":"unknown({})","raw":{}}}"#, type_enum, v)
        }
    }
}

/// 遍历类的所有字段（含父类），返回 (field_name, offset, type_ptr) 列表
unsafe fn collect_all_fields(class: *mut c_void) -> Vec<(String, i32, *const c_void)> {
    let mut result = Vec::new();
    if class.is_null() || API.is_null() {
        return result;
    }

    let get_fields_fn: Option<FnClassGetFields> = {
        let p = resolve_il2cpp_symbol("il2cpp_class_get_fields");
        if p.is_null() {
            None
        } else {
            Some(std::mem::transmute::<*mut c_void, FnClassGetFields>(p))
        }
    };
    let get_parent_fn: Option<FnClassGetParent> = {
        let p = resolve_il2cpp_symbol("il2cpp_class_get_parent");
        if p.is_null() {
            None
        } else {
            Some(std::mem::transmute::<*mut c_void, FnClassGetParent>(p))
        }
    };

    if get_fields_fn.is_none() {
        return result;
    }

    let mut current_class = class;
    let mut depth = 0;
    loop {
        if current_class.is_null() || depth > 10 {
            break;
        }

        let mut iter: *mut c_void = ptr::null_mut();
        loop {
            let field_info = get_fields_fn.unwrap()(current_class, &mut iter);
            if field_info.is_null() {
                break;
            }

            let fname = if !(*field_info).name.is_null() {
                std::ffi::CStr::from_ptr((*field_info).name)
                    .to_string_lossy()
                    .to_string()
            } else {
                "?".to_string()
            };
            let offset = (*field_info).offset;
            let type_ptr = (*field_info)._ty as *const c_void;
            result.push((fname, offset, type_ptr));
        }

        if let Some(ref get_parent) = get_parent_fn {
            let parent = get_parent(current_class);
            if parent.is_null() || parent == current_class {
                break;
            }
            current_class = parent;
        } else {
            break;
        }
        depth += 1;
    }
    result
}

/// /il2cpp/dump?name=X — Dump单例实例，带运行时字段值
/// 查找类→获取单例→遍历所有字段→读取运行时值
unsafe fn il2cpp_dump_singleton(class_name: &str) -> String {
    if class_name.is_empty() {
        return r#"{"error":"missing ?name= parameter"}"#.to_string();
    }
    if API.is_null() {
        return r#"{"error":"api_null"}"#.to_string();
    }
    let image = match get_image() {
        img if !img.is_null() => img,
        _ => return r#"{"error":"image_null"}"#.to_string(),
    };

    let class = find_class_by_short_name(image, class_name);
    if class.is_null() {
        return format!(r#"{{"error":"class_not_found","name":"{}"}}"#, class_name);
    }

    let real_name = get_class_name_from_pointer(class);
    let instance = get_singleton(class);

    if instance.is_null() {
        // 单例不存在，只返回字段元数据
        let fields = collect_all_fields(class);
        let fields_json: Vec<String> = fields
            .iter()
            .map(|(n, o, t)| {
                let type_enum = il2cpp_type_get_type_enum(*t);
                format!(
                    r#"{{"name":"{}","offset":{},"type":{}}}"#,
                    json_escape(n),
                    o,
                    type_enum
                )
            })
            .collect();
        return format!(
            r#"{{"requested":"{}","found":"{}","instance":"null","fields":[{}]}}"#,
            class_name,
            real_name,
            fields_json.join(",")
        );
    }

    // 读取每个字段的运行时值
    let fields = collect_all_fields(class);
    let fields_json: Vec<String> = fields
        .iter()
        .map(|(n, o, t)| {
            let type_enum = il2cpp_type_get_type_enum(*t);
            let val = read_field_value_json(instance, *o, *t);
            format!(
                r#"{{"name":"{}","offset":{},"type":{},"value":{}}}"#,
                json_escape(n),
                o,
                type_enum,
                val
            )
        })
        .collect();

    format!(
        r#"{{"requested":"{}","found":"{}","instance":"0x{:x}","field_count":{},"fields":[{}]}}"#,
        class_name,
        real_name,
        instance as usize,
        fields_json.len(),
        fields_json.join(",")
    )
}

/// /il2cpp/call?class=X&method=Y — 调用单例上的getter方法
/// 返回值自动判断类型(int/float/string/object)
unsafe fn il2cpp_call_method(class_name: &str, method_name: &str) -> String {
    if class_name.is_empty() || method_name.is_empty() {
        return r#"{"error":"missing ?class= or ?method= parameter"}"#.to_string();
    }
    if API.is_null() {
        return r#"{"error":"api_null"}"#.to_string();
    }
    let image = match get_image() {
        img if !img.is_null() => img,
        _ => return r#"{"error":"image_null"}"#.to_string(),
    };

    let class = find_class_by_short_name(image, class_name);
    if class.is_null() {
        return format!(r#"{{"error":"class_not_found","name":"{}"}}"#, class_name);
    }

    let instance = get_singleton(class);
    if instance.is_null() {
        return format!(r#"{{"error":"singleton_null","class":"{}"}}"#, class_name);
    }

    let result = call_getter_on_instance(class, instance, method_name);
    if result.is_null() {
        return format!(
            r#"{{"ok":false,"class":"{}","method":"{}","error":"call_returned_null"}}"#,
            class_name, method_name
        );
    }

    // 尝试判断返回值类型
    let obj_class = get_class_from_object(result);
    let obj_class_name = get_class_name_from_pointer(obj_class);

    // 尝试作为int读取（boxed value type在offset 16）
    let int_val = std::ptr::read_unaligned::<i32>((result as *const u8).add(16) as *const i32);

    // 尝试作为string读取
    let str_val = if obj_class_name == "String" {
        read_il2cpp_string(result)
    } else {
        String::new()
    };

    if obj_class_name == "String" {
        format!(
            r#"{{"ok":true,"class":"{}","method":"{}","return_type":"string","value":"{}"}}"#,
            class_name,
            method_name,
            json_escape(&str_val)
        )
    } else if int_val != 0
        || obj_class_name.contains("Int")
        || obj_class_name.contains("Boolean")
        || obj_class_name.contains("Single")
    {
        // 可能是值类型
        let float_val =
            std::ptr::read_unaligned::<f32>((result as *const u8).add(16) as *const f32);
        format!(
            r#"{{"ok":true,"class":"{}","method":"{}","return_type":"{}","int_value":{},"float_value":{}}}"#,
            class_name,
            method_name,
            obj_class_name,
            int_val,
            format!("{}", float_val)
        )
    } else {
        // 引用类型，dump其字段
        let fields = collect_all_fields(obj_class);
        let fields_json: Vec<String> = fields
            .iter()
            .map(|(n, o, t)| {
                let type_enum = il2cpp_type_get_type_enum(*t);
                let val = read_field_value_json(result, *o, *t);
                format!(
                    r#"{{"name":"{}","offset":{},"type":{},"value":{}}}"#,
                    json_escape(n),
                    o,
                    type_enum,
                    val
                )
            })
            .collect();
        format!(
            r#"{{"ok":true,"class":"{}","method":"{}","return_type":"{}","ptr":"0x{:x}","fields":[{}]}}"#,
            class_name,
            method_name,
            obj_class_name,
            result as usize,
            fields_json.join(",")
        )
    }
}

/// /il2cpp/tree?name=X&depth=N — 递归dump引用类型字段
/// depth控制递归深度，防止无限循环
unsafe fn il2cpp_tree_dump(class_name: &str, max_depth: usize) -> String {
    if class_name.is_empty() {
        return r#"{"error":"missing ?name= parameter"}"#.to_string();
    }
    if API.is_null() {
        return r#"{"error":"api_null"}"#.to_string();
    }
    let image = match get_image() {
        img if !img.is_null() => img,
        _ => return r#"{"error":"image_null"}"#.to_string(),
    };

    let class = find_class_by_short_name(image, class_name);
    if class.is_null() {
        return format!(r#"{{"error":"class_not_found","name":"{}"}}"#, class_name);
    }

    let instance = get_singleton(class);
    if instance.is_null() {
        return format!(r#"{{"error":"singleton_null","class":"{}"}}"#, class_name);
    }

    let real_name = get_class_name_from_pointer(class);
    let depth = if max_depth == 0 { 2 } else { max_depth.min(5) }; // 最大5层
    let visited = std::collections::HashSet::new();
    let tree = il2cpp_dump_recursive(instance, class, depth, &visited);

    format!(
        r#"{{"requested":"{}","found":"{}","instance":"0x{:x}","max_depth":{},"tree":{}}}"#,
        class_name, real_name, instance as usize, depth, tree
    )
}

/// 递归dump对象字段（内部函数）
unsafe fn il2cpp_dump_recursive(
    obj: *const c_void,
    class: *mut c_void,
    depth: usize,
    visited: &std::collections::HashSet<usize>,
) -> String {
    if obj.is_null() || depth == 0 {
        return "null".to_string();
    }

    let obj_addr = obj as usize;
    if visited.contains(&obj_addr) {
        return format!(r#"{{"type":"cycle","ptr":"0x{:x}"}}"#, obj_addr);
    }

    // 防止访问过多不同对象
    if visited.len() > 200 {
        return format!(r#"{{"type":"limit","ptr":"0x{:x}"}}"#, obj_addr);
    }

    let mut new_visited = visited.clone();
    new_visited.insert(obj_addr);

    let class_name = get_class_name_from_pointer(class);
    let fields = collect_all_fields(class);

    // String类型特殊处理
    if class_name == "String" {
        let s = read_il2cpp_string(obj);
        return format!(
            r#"{{"type":"string","class":"String","value":"{}"}}"#,
            json_escape(&s)
        );
    }

    let fields_json: Vec<String> = fields
        .iter()
        .map(|(n, o, t)| {
            let type_enum = il2cpp_type_get_type_enum(*t);
            let val = if type_enum == IL2CPP_TYPE_CLASS || type_enum == IL2CPP_TYPE_GENERICINST {
                // 引用类型：递归dump
                let ptr_val = std::ptr::read_unaligned::<*const c_void>(
                    (obj as *const u8).add(*o as usize) as *const *const c_void,
                );
                if ptr_val.is_null() {
                    "null".to_string()
                } else {
                    let child_class = get_class_from_object(ptr_val);
                    let child_class_name = get_class_name_from_pointer(child_class);
                    // 简单类型直接读值
                    if child_class_name == "String" {
                        let s = read_il2cpp_string(ptr_val);
                        format!(r#""{}""#, json_escape(&s))
                    } else if child_class_name.contains("Int32")
                        || child_class_name.contains("Boolean")
                    {
                        let v = std::ptr::read_unaligned::<i32>(
                            (ptr_val as *const u8).add(16) as *const i32
                        );
                        v.to_string()
                    } else {
                        // 递归
                        il2cpp_dump_recursive(ptr_val, child_class, depth - 1, &new_visited)
                    }
                }
            } else {
                read_field_value_json(obj, *o, *t)
            };
            format!(
                r#"{{"name":"{}","offset":{},"type":{},"value":{}}}"#,
                json_escape(n),
                o,
                type_enum,
                val
            )
        })
        .collect();

    format!(
        r#"{{"type":"object","class":"{}","ptr":"0x{:x}","field_count":{},"fields":[{}]}}"#,
        json_escape(&class_name),
        obj_addr,
        fields_json.len(),
        fields_json.join(",")
    )
}

/// /il2cpp/field?class=X&field=Y — 读取单例的指定字段值
unsafe fn il2cpp_read_single_field(class_name: &str, field_name: &str) -> String {
    if class_name.is_empty() || field_name.is_empty() {
        return r#"{"error":"missing ?class= or ?field= parameter"}"#.to_string();
    }
    if API.is_null() {
        return r#"{"error":"api_null"}"#.to_string();
    }
    let image = match get_image() {
        img if !img.is_null() => img,
        _ => return r#"{"error":"image_null"}"#.to_string(),
    };

    let class = find_class_by_short_name(image, class_name);
    if class.is_null() {
        return format!(r#"{{"error":"class_not_found","name":"{}"}}"#, class_name);
    }

    let instance = get_singleton(class);
    if instance.is_null() {
        return format!(r#"{{"error":"singleton_null","class":"{}"}}"#, class_name);
    }

    // 遍历字段找到匹配的
    let fields = collect_all_fields(class);
    let normalize = |name: &str| -> String {
        let n = if name.starts_with('<') {
            if let Some(end) = name.find('>') {
                &name[1..end]
            } else {
                name
            }
        } else {
            name
        };
        n.trim_start_matches('_').to_lowercase()
    };
    let target = normalize(field_name);

    for (fname, offset, type_ptr) in &fields {
        if normalize(fname) == target {
            let type_enum = il2cpp_type_get_type_enum(*type_ptr);
            let val = read_field_value_json(instance, *offset, *type_ptr);
            return format!(
                r#"{{"ok":true,"class":"{}","field":"{}","offset":{},"type":{},"value":{}}}"#,
                class_name,
                json_escape(fname),
                offset,
                type_enum,
                val
            );
        }
    }

    // 没找到字段，尝试用read_field_ptr
    let ptr_val = read_field_ptr(instance, class, field_name);
    if !ptr_val.is_null() {
        let obj_name = get_object_class_name(ptr_val);
        return format!(
            r#"{{"ok":true,"class":"{}","field":"{}","type":"ptr","value":"0x{:x}","obj_class":"{}"}}"#,
            class_name,
            field_name,
            ptr_val as usize,
            json_escape(&obj_name)
        );
    }

    format!(
        r#"{{"error":"field_not_found","class":"{}","field":"{}"}}"#,
        class_name, field_name
    )
}

/// v3.22.51: /debug/ramenfields — Walk all ramen arrays, dump element class + fields
/// For each array: read first element, get class from object header, dump all fields + hex
unsafe fn debug_ramenfields() -> String {
    if API.is_null() {
        return r#"{"error":"api_null"}"#.to_string();
    }
    let image = match get_image() {
        img if !img.is_null() => img,
        _ => return r#"{"error":"image_null"}"#.to_string(),
    };
    let wdm_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkDataManager").as_ptr(),
    );
    if wdm_class.is_null() {
        return r#"{"error":"no_wdm"}"#.to_string();
    }
    let wdm_inst = get_singleton(wdm_class);
    if wdm_inst.is_null() {
        return r#"{"error":"no_wdm_inst"}"#.to_string();
    }
    let sm_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkSingleModeData").as_ptr(),
    );
    let sm_obj = call_getter_ref(wdm_class, wdm_inst, "get_SingleMode");
    if sm_obj.is_null() {
        return r#"{"error":"no_sm"}"#.to_string();
    }
    let chara_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkSingleModeCharaData").as_ptr(),
    );
    let chara_obj = call_getter_ref(sm_class, sm_obj, "get_Character");
    if chara_obj.is_null() {
        return r#"{"error":"no_chara"}"#.to_string();
    }
    let ramen_sc_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkSingleModeScenarioRamen").as_ptr(),
    );
    if ramen_sc_class.is_null() {
        return r#"{"error":"no_ramen_sc_class"}"#.to_string();
    }
    let ramen_sc_obj = try_get_scenario_obj(chara_class, chara_obj, 14);
    if ramen_sc_obj.is_null() {
        return r#"{"error":"no_ramen_sc_obj"}"#.to_string();
    }
    let ramen_ds_obj = call_getter_ref(ramen_sc_class, ramen_sc_obj, "get_DataSet");
    if ramen_ds_obj.is_null() {
        return r#"{"error":"no_ramen_ds"}"#.to_string();
    }

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
        let length =
            std::ptr::read_unaligned::<usize>(base.add(IL2CPP_LIST_COUNT_OFF) as *const usize);
        if length == 0 || length > 200 {
            arrays_json.push(format!(
                r#"{{"getter":"{}","status":"empty_or_too_long","len":{}}}"#,
                getter, length
            ));
            continue;
        }

        // Read first element
        let first_elem = std::ptr::read_unaligned::<*mut c_void>(
            base.add(IL2CPP_LIST_ITEMS_OFF) as *const *mut c_void
        );
        if first_elem.is_null() {
            arrays_json.push(format!(
                r#"{{"getter":"{}","status":"null_first_elem"}}"#,
                getter
            ));
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
            getter,
            length,
            elem_class_name,
            fields_json,
            hex_parts.join(",")
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
            uraf_class_name,
            uraf_fields,
            hex_parts.join(",")
        )
    } else {
        r#""uraf_effect":{"status":"null"}"#.to_string()
    };

    format!(
        r#"{{"dataset_class":"{}","arrays":[{}],{}}}"#,
        ds_class_name,
        arrays_json.join(","),
        uraf_json
    )
}

/// 检查一段地址是否完整位于 /proc/self/maps 的可读区间。
fn is_readable_range(addr: usize, len: usize) -> bool {
    if addr < 0x10000 || len == 0 {
        return false;
    }
    let end = match addr.checked_add(len) {
        Some(v) => v,
        None => return false,
    };
    let maps = match std::fs::read_to_string("/proc/self/maps") {
        Ok(v) => v,
        Err(_) => return false,
    };
    for line in maps.lines() {
        let mut parts = line.split_whitespace();
        let range = match parts.next() {
            Some(v) => v,
            None => continue,
        };
        let perms = parts.next().unwrap_or("");
        if !perms.starts_with('r') {
            continue;
        }
        let (start_text, end_text) = match range.split_once('-') {
            Some(v) => v,
            None => continue,
        };
        let start = match usize::from_str_radix(start_text, 16) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let map_end = match usize::from_str_radix(end_text, 16) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if addr >= start && end <= map_end {
            return true;
        }
    }
    false
}

/// 输出诊断对象的原始十六进制内存。地址不可读时返回空字符串。
unsafe fn debug_hex(addr: *const u8, len: usize) -> String {
    if addr.is_null() || !is_readable_range(addr as usize, len) {
        return String::new();
    }
    let mut result = String::with_capacity(len * 2);
    for index in 0..len {
        let value = std::ptr::read_unaligned(addr.add(index));
        result.push_str(&format!("{:02x}", value));
    }
    result
}

/// 诊断 IL2CPP 集合（可能是 Array 或 List<T>）
unsafe fn debug_il2cpp_collection(collection: *mut c_void, max_items: usize) -> String {
    if collection.is_null() {
        return r#"{"error":"null_collection"}"#.to_string();
    }

    let collection_class = get_class_from_object(collection);
    let collection_class_name = get_class_name_from_pointer(collection_class);

    let is_list =
        collection_class_name.contains("List`1") || collection_class_name.starts_with("List<");

    let (count, array) = if is_list {
        let count = std::ptr::read_unaligned::<i32>(
            (collection as *const u8).add(IL2CPP_LIST_COUNT_OFF) as *const i32,
        );
        let array = read_ptr_at(collection, IL2CPP_LIST_ARRAY_OFF as i32);
        (count as i64, array)
    } else {
        let count = std::ptr::read_unaligned::<usize>(
            (collection as *const u8).add(IL2CPP_LIST_COUNT_OFF) as *const usize,
        );
        (count as i64, collection)
    };

    if count > 10000 || count < 0 {
        return format!(
            r#"{{"collection_class":"{}","storage":"{}","length":{},"error":"invalid_count"}}"#,
            json_escape(&collection_class_name),
            if is_list { "list" } else { "array" },
            count
        );
    }

    if array.is_null() {
        return format!(
            r#"{{"collection_class":"{}","storage":"{}","length":{},"error":"null_array"}}"#,
            json_escape(&collection_class_name),
            if is_list { "list" } else { "array" },
            count
        );
    }

    // 获取元素类型
    let get_element_class_fn = resolve_il2cpp_symbol("il2cpp_class_get_element_class");
    let is_valuetype_fn = resolve_il2cpp_symbol("il2cpp_class_is_valuetype");
    let value_size_fn = resolve_il2cpp_symbol("il2cpp_class_value_size");

    let array_class = get_class_from_object(array);
    let element_class = if !get_element_class_fn.is_null() && !array_class.is_null() {
        let f: unsafe extern "C" fn(*mut c_void) -> *mut c_void =
            std::mem::transmute(get_element_class_fn);
        f(array_class)
    } else {
        ptr::null_mut()
    };

    let element_class_name = if !element_class.is_null() {
        get_class_name_from_pointer(element_class)
    } else {
        String::new()
    };

    let element_is_value_type = if !is_valuetype_fn.is_null() && !element_class.is_null() {
        let f: unsafe extern "C" fn(*mut c_void) -> bool = std::mem::transmute(is_valuetype_fn);
        f(element_class)
    } else {
        false
    };

    let element_size = if !value_size_fn.is_null() && !element_class.is_null() {
        let f: unsafe extern "C" fn(*mut c_void) -> i32 = std::mem::transmute(value_size_fn);
        f(element_class) as usize
    } else {
        8 // 默认指针大小
    };

    // 遍历元素
    let limit = (count as usize).min(max_items);
    let mut items: Vec<String> = Vec::new();

    for index in 0..limit {
        if element_is_value_type {
            // 值类型：直接读内存
            let element_address = (array as *const u8).add(0x20 + index * element_size);
            let raw = debug_hex(element_address, element_size.min(0x40));
            items.push(format!(
                r#"{{"index":{},"storage":"inline_value","address":"0x{:x}","size":{},"raw_hex":"{}"}}"#,
                index, element_address as usize, element_size, raw
            ));
        } else {
            // 引用类型：读指针
            let slot = (array as *const u8).add(0x20 + index * 8);
            let object = std::ptr::read_unaligned::<*mut c_void>(slot as *const *mut c_void);

            if object.is_null() {
                items.push(format!(
                    r#"{{"index":{},"storage":"reference","ptr":"null","status":"null"}}"#,
                    index
                ));
                continue;
            }

            if !is_readable_range(object as usize, 0x10) {
                items.push(format!(
                    r#"{{"index":{},"storage":"reference","ptr":"0x{:x}","status":"unreadable_target"}}"#,
                    index, object as usize
                ));
                continue;
            }

            let runtime_class = get_class_from_object(object);
            let runtime_name = get_class_name_from_pointer(runtime_class);
            let raw = debug_hex(object as *const u8, 0x60);

            items.push(format!(
                r#"{{"index":{},"storage":"reference","ptr":"0x{:x}","runtime_class":"{}","raw_hex":"{}"}}"#,
                index, object as usize, json_escape(&runtime_name), raw
            ));
        }
    }

    format!(
        r#"{{"collection_class":"{}","storage":"{}","length":{},"element_class":"{}","element_is_value_type":{},"element_size":{},"items":[{}]}}"#,
        json_escape(&collection_class_name),
        if is_list { "list" } else { "array" },
        count,
        json_escape(&element_class_name),
        element_is_value_type,
        element_size,
        items.join(",")
    )
}

/// 诊断训练伙伴 — 只读，不修改 /summary 或评分
unsafe fn debug_training_partners_inner() -> String {
    if API.is_null() {
        return r#"{"error":"api_null"}"#.to_string();
    }
    let image = match get_image() {
        img if !img.is_null() => img,
        _ => return r#"{"error":"image_null"}"#.to_string(),
    };

    let wdm_class = find_class_by_short_name(image, "WorkDataManager");
    if wdm_class.is_null() {
        return r#"{"error":"wdm_class_null"}"#.to_string();
    }
    let sm_class = find_class_by_short_name(image, "WorkSingleModeData");
    let home_class = find_class_by_short_name(image, "WorkSingleModeHomeInfoData");
    let chara_class = find_class_by_short_name(image, "WorkSingleModeCharaData");

    let wdm = get_singleton(wdm_class);
    if wdm.is_null() {
        return r#"{"error":"wdm_null"}"#.to_string();
    }

    let sm = call_getter_ref(wdm_class, wdm, "get_SingleMode");
    if sm.is_null() {
        return r#"{"error":"sm_null"}"#.to_string();
    }

    let home = call_getter_on_instance(sm_class, sm, "get_HomeInfoData");
    let chara = call_getter_ref(sm_class, sm, "get_Character");

    // CommandInfoArray
    let commands = read_field_value(home_class, home, "CommandInfoArray");
    let commands_diag = if !commands.is_null() {
        debug_il2cpp_collection(commands, 16)
    } else {
        r#"{"error":"null"}"#.to_string()
    };

    // 遍历每个训练项
    let command_count = if !commands.is_null() {
        std::ptr::read_unaligned::<usize>((commands as *const u8).add(0x18) as *const usize)
    } else {
        0
    };

    let mut cmd_parts: Vec<String> = Vec::new();
    let cmd_limit = command_count.min(16);
    for index in 0..cmd_limit {
        let command_slot = (commands as *const u8).add(0x20 + index * 8);
        let command = std::ptr::read_unaligned::<*mut c_void>(command_slot as *const *mut c_void);
        if command.is_null() {
            cmd_parts.push(format!(r#"{{"index":{},"error":"null_command"}}"#, index));
            continue;
        }

        let command_id = read_obscured_int_at(command, 36);
        let command_class = get_class_from_object(command);
        let command_class_name = get_class_name_from_pointer(command_class);
        let command_raw = debug_hex(command as *const u8, 0x70);

        let training_partners = read_ptr_at(command, 80);
        let tips_event_partners = read_ptr_at(command, 88);

        let tp_diag = if !training_partners.is_null() {
            debug_il2cpp_collection(training_partners, 16)
        } else {
            r#"{"error":"null"}"#.to_string()
        };

        let tips_diag = if !tips_event_partners.is_null() {
            debug_il2cpp_collection(tips_event_partners, 16)
        } else {
            r#"{"error":"null"}"#.to_string()
        };

        cmd_parts.push(format!(
            r#"{{"index":{},"command_id":{},"command_ptr":"0x{:x}","command_class":"{}","command_raw_hex":"{}","training_partners":{},"tips_event_partners":{}}}"#,
            index, command_id, command as usize,
            json_escape(&command_class_name), command_raw,
            tp_diag, tips_diag
        ));
    }

    // 支援卡
    let support_cards = call_getter_on_instance(chara_class, chara, "get_EquipSupportCardArray");
    let sc_diag = if !support_cards.is_null() {
        debug_il2cpp_collection(support_cards, 16)
    } else {
        r#"{"error":"null"}"#.to_string()
    };

    // 羁绊列表
    let evaluation_list = read_ptr_at(chara, 1016);
    let eval_diag = if !evaluation_list.is_null() {
        debug_il2cpp_collection(evaluation_list, 100)
    } else {
        r#"{"error":"null"}"#.to_string()
    };

    format!(
        r#"{{"ok":true,"diagnostic_version":1,"read_only":true,"notes":["No partner identity or bond gain is inferred in this stage","shining is not inferred from TipsEventPartnerArray"],"commands_collection":{},"commands":[{}],"support_cards":{},"evaluation_list":{}}}"#,
        commands_diag,
        cmd_parts.join(","),
        sc_diag,
        eval_diag
    )
}

/// 崩溃保护包装
fn debug_training_partners() -> String {
    let _lock = READ_MUTEX.lock().unwrap_or_else(|error| error.into_inner());
    let jump_result = unsafe { sys_sigsetjmp(SIGSEGV_JMP_BUF.as_mut_ptr(), 1) };
    if jump_result != 0 {
        SIGSEGV_RECOVERY.store(false, Ordering::Relaxed);
        return r#"{"error":"sigsegv_recovered","hint":"training partner diagnostic hit an invalid runtime pointer; game was protected"}"#.to_string();
    }
    SIGSEGV_RECOVERY.store(true, Ordering::Relaxed);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
        debug_training_partners_inner()
    }))
    .unwrap_or_else(|_| r#"{"error":"panic_caught"}"#.to_string());
    SIGSEGV_RECOVERY.store(false, Ordering::Relaxed);
    result
}

/// /debug/cmdinfo — Dump command element class info WITHOUT runtime_invoke on command elements
/// Reads class from object header (offset 0), enumerates fields + methods + hex dump
/// Safe: only uses il2cpp_class_get_fields / il2cpp_class_get_methods (no runtime_invoke on cmd elements)
unsafe fn debug_cmdinfo() -> String {
    if API.is_null() {
        return r#"{"error":"api_null"}"#.to_string();
    }
    let image = match get_image() {
        img if !img.is_null() => img,
        _ => return r#"{"error":"image_null"}"#.to_string(),
    };

    let wdm_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkDataManager").as_ptr(),
    );
    if wdm_class.is_null() {
        return r#"{"error":"no_wdm"}"#.to_string();
    }
    let wdm_inst = get_singleton(wdm_class);
    if wdm_inst.is_null() {
        return r#"{"error":"no_wdm_inst"}"#.to_string();
    }
    log_predict_step("P:wdm");

    let sm_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkSingleModeData").as_ptr(),
    );
    let sm_obj = call_getter_ref(wdm_class, wdm_inst, "get_SingleMode");
    if sm_obj.is_null() {
        return r#"{"error":"no_sm"}"#.to_string();
    }

    let home_info_obj = call_getter_on_instance(sm_class, sm_obj, "get_HomeInfoData");
    if home_info_obj.is_null() {
        return r#"{"error":"no_home_info"}"#.to_string();
    }
    log_predict_step("got home_info");
    let hi_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkSingleModeHomeInfoData").as_ptr(),
    );
    if hi_class.is_null() {
        return r#"{"error":"no_home_info_class"}"#.to_string();
    }

    let cmd_arr = read_field_value(hi_class, home_info_obj, "CommandInfoArray");
    if cmd_arr.is_null() {
        return r#"{"error":"no_cmd_arr"}"#.to_string();
    }

    let cmd_base = cmd_arr as *const u8;
    let cmd_len =
        std::ptr::read_unaligned::<usize>(cmd_base.add(IL2CPP_LIST_COUNT_OFF) as *const usize);
    if cmd_len == 0 {
        return r#"{"error":"empty_cmd_arr"}"#.to_string();
    }

    // Read first element
    let ep = std::ptr::read_unaligned::<*mut c_void>(
        cmd_base.add(IL2CPP_LIST_ITEMS_OFF) as *const *mut c_void
    );
    if ep.is_null() {
        return r#"{"error":"null_elem"}"#.to_string();
    }

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
        let ep2 = std::ptr::read_unaligned::<*mut c_void>(
            cmd_base.add(IL2CPP_LIST_ITEMS_OFF + IL2CPP_LIST_ITEM_SIZE) as *const *mut c_void,
        );
        if !ep2.is_null() {
            let ep2_class = std::ptr::read_unaligned::<*mut c_void>(ep2 as *const *mut c_void);
            get_class_name_from_pointer(ep2_class)
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    format!(
        r#"{{"cmd_len":{},"elem0_class":"{}","elem1_class":"{}","fields":{},"methods":{},"hex":{{{}}}}}"#,
        cmd_len,
        class_name,
        ep2_class_name,
        fields_json,
        methods_json,
        hex_parts.join(",")
    )
}

/// /training/predict — Detailed training prediction with NPC partner breakdown
/// Returns per-command: gains, partner details (support card vs NPC), buffs, failure risk
/// Key data sources:
///   - WorkSingleModeData -> get_HomeInfoData -> CommandInfoArray (training layout + partners)
///   - WorkSingleModeCharaData -> CharaEffectBuffArray (active buffs)
///   - WorkSingleModeScenarioRamenDataSet (ramen-specific data, scenario_id==14)
unsafe fn read_ramen_info() -> String {
    if API.is_null() {
        return r#"{"error":"api_null"}"#.to_string();
    }
    let image = match get_image() {
        img if !img.is_null() => img,
        _ => return r#"{"error":"image_null"}"#.to_string(),
    };
    let wdm_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkDataManager").as_ptr(),
    );
    if wdm_class.is_null() {
        return r#"{"error":"no_wdm"}"#.to_string();
    }
    let wdm_inst = get_singleton(wdm_class);
    if wdm_inst.is_null() {
        return r#"{"error":"no_wdm_inst"}"#.to_string();
    }
    let sm_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkSingleModeData").as_ptr(),
    );
    let sm_obj = call_getter_ref(wdm_class, wdm_inst, "get_SingleMode");
    if sm_obj.is_null() {
        return r#"{"error":"no_sm"}"#.to_string();
    }
    let chara_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkSingleModeCharaData").as_ptr(),
    );
    let chara_obj = call_getter_ref(sm_class, sm_obj, "get_Character");
    if chara_obj.is_null() {
        return r#"{"error":"no_chara"}"#.to_string();
    }

    let ramen_sc_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkSingleModeScenarioRamen").as_ptr(),
    );
    if ramen_sc_class.is_null() {
        return r#"{"error":"no_ramen_sc_class"}"#.to_string();
    }
    let ramen_sc_obj = try_get_scenario_obj(chara_class, chara_obj, 14);
    if ramen_sc_obj.is_null() {
        return r#"{"error":"no_ramen_sc_obj"}"#.to_string();
    }
    let ramen_ds_obj = call_getter_ref(ramen_sc_class, ramen_sc_obj, "get_DataSet");
    if ramen_ds_obj.is_null() {
        return r#"{"error":"no_ramen_ds"}"#.to_string();
    }

    // Read class pointer from object header (offset 0 on 64-bit = Il2CppObject.klass)
    let ds_base = ramen_ds_obj as *const u8;
    let ds_class_ptr = std::ptr::read_unaligned::<*mut c_void>(ds_base as *const *mut c_void);

    // Hex dump first 256 bytes
    let mut hex = String::new();
    for i in 0..256usize {
        let b = std::ptr::read_unaligned::<u8>(ds_base.add(i));
        hex.push_str(&format!("{:02x}", b));
        if (i + 1) % 16 == 0 {
            hex.push('\n');
        } else if (i + 1) % 8 == 0 {
            hex.push(' ');
        }
    }

    // Try to read class name via il2cpp class API
    let mut class_name = String::new();
    if !ds_class_ptr.is_null() {
        let get_name_fn = resolve_il2cpp_symbol("il2cpp_class_get_name");
        if !get_name_fn.is_null() {
            let fn_ptr: unsafe extern "C" fn(*mut c_void) -> *const u8 =
                std::mem::transmute(get_name_fn);
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

/// /inherit/compat — Inheritance compatibility calculation
/// Shows exact compatibility values (not just ○△×), split by parent gender,
/// and detects target race overlap
/// Data sources:
///   - SuccessionCharaInfo (parent chara IDs)
///   - SuccessionRelationMember + SuccessionRelation (compatibility data)
///   - mdb succession_relation tables
///   - SingleModeTargetRace (current target races)
unsafe fn read_inherit_compat() -> String {
    if API.is_null() {
        return r#"{"error":"api_null"}"#.to_string();
    }
    let image = match get_image() {
        img if !img.is_null() => img,
        _ => return r#"{"error":"image_null"}"#.to_string(),
    };

    let wdm_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkDataManager").as_ptr(),
    );
    if wdm_class.is_null() {
        return r#"{"error":"no_wdm"}"#.to_string();
    }
    let wdm_inst = get_singleton(wdm_class);
    if wdm_inst.is_null() {
        return r#"{"error":"no_wdm_inst"}"#.to_string();
    }
    log_predict_step("P:wdm");

    let sm_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkSingleModeData").as_ptr(),
    );
    let sm_obj = call_getter_ref(wdm_class, wdm_inst, "get_SingleMode");
    if sm_obj.is_null() {
        return r#"{"error":"no_sm"}"#.to_string();
    }

    let chara_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkSingleModeCharaData").as_ptr(),
    );
    let chara_obj = call_getter_ref(sm_class, sm_obj, "get_Character");
    if chara_obj.is_null() {
        return r#"{"error":"no_chara"}"#.to_string();
    }

    // 1. Read succession parent info
    // WorkSingleModeCharaData.SuccessionTrainedCharaInfoFirst (offset 0x48)
    // WorkSingleModeCharaData.SuccessionTrainedCharaInfoSecond (offset 0x50)
    let sci_class = find_class_by_short_name(image, "SuccessionCharaInfo");
    let first_sci = call_getter_ref(
        chara_class,
        chara_obj,
        "get_SuccessionTrainedCharaInfoFirst",
    );
    let second_sci = call_getter_ref(
        chara_class,
        chara_obj,
        "get_SuccessionTrainedCharaInfoSecond",
    );

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
        factor_count =
            std::ptr::read_unaligned::<usize>(fb.add(IL2CPP_LIST_COUNT_OFF) as *const usize) as i32;
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
            if let Ok(mut stmt) = conn.prepare(
                "SELECT id, relation_type, chara_id FROM succession_relation_member ORDER BY id",
            ) {
                let rows: Vec<String> = stmt
                    .query_map([], |row| {
                        Ok(format!(
                            r#"{{"id":{},"type":{},"chara_id":{}}}"#,
                            row.get::<_, i32>(0).unwrap_or(0),
                            row.get::<_, i32>(1).unwrap_or(0),
                            row.get::<_, i32>(2).unwrap_or(0)
                        ))
                    })
                    .unwrap()
                    .filter_map(|r| r.ok())
                    .collect();
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
                let tp = std::ptr::read_unaligned::<*mut c_void>(
                    trb.add(IL2CPP_LIST_ITEMS_OFF + ti * IL2CPP_LIST_ITEM_SIZE)
                        as *const *mut c_void,
                );
                if tp.is_null() {
                    continue;
                }
                // TargetRace: targetId at offset 0x10, evaluation at 0x14
                let bytes = tp as *const u8;
                let tid = std::ptr::read_unaligned::<i32>(
                    bytes.add(IL2CPP_TARGET_RACE_ID_OFF) as *const i32
                );
                let teval = std::ptr::read_unaligned::<i32>(
                    bytes.add(IL2CPP_TARGET_RACE_EVAL_OFF) as *const i32
                );
                target_races_json
                    .push(format!(r#"{{"target_id":{},"evaluation":{}}}"#, tid, teval));
            }
        }
    }

    // 5. Read route_race from mdb for race name resolution
    let mut race_names_json: Vec<String> = Vec::new();
    if let Some(mdb_path) = find_mdb_path() {
        if let Ok(conn) = Connection::open_with_flags(&mdb_path, OpenFlags::SQLITE_OPEN_READ_ONLY) {
            if let Ok(mut stmt) = conn.prepare(
                "SELECT id, race_id, race_grade FROM single_mode_route_race ORDER BY id LIMIT 200",
            ) {
                let rows: Vec<String> = stmt
                    .query_map([], |row| {
                        Ok(format!(
                            r#"{{"id":{},"race_id":{},"grade":{}}}"#,
                            row.get::<_, i32>(0).unwrap_or(0),
                            row.get::<_, i32>(1).unwrap_or(0),
                            row.get::<_, i32>(2).unwrap_or(0)
                        ))
                    })
                    .unwrap()
                    .filter_map(|r| r.ok())
                    .collect();
                race_names_json = rows;
            }
            drop(conn);
        }
    }

    format!(
        r#"{{"version":"3.22.91","parents":{{"first_chara_id":{},"second_chara_id":{}}},"factor_count":{},"relations":[{}],"relation_members":[{}],"relation_ranks":[{}],"target_races":[{}],"route_races":[{}]}}"#,
        first_chara_id,
        second_chara_id,
        factor_count,
        relations_json.join(","),
        relation_members_json.join(","),
        relation_ranks_json.join(","),
        target_races_json.join(","),
        race_names_json.join(",")
    )
}

/// /saddle-analysis — WinSaddleAnalyzer
/// Reads current trained chara's win saddles from game memory,
/// cross-references with MDB to map each G1 win to its relation_group_id,
/// then outputs which relation groups (compatibility bonuses) the chara has earned.
/// Also reads parent candidates' win saddles for cross-comparison.
unsafe fn read_win_saddle_analysis() -> String {
    if API.is_null() {
        return r#"{"error":"api_null"}"#.to_string();
    }
    let api = &*API;
    let image = match get_image() {
        img if !img.is_null() => img,
        _ => return r#"{"error":"image_null"}"#.to_string(),
    };

    // 1. Get WorkSingleModeData
    let wdm_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkDataManager").as_ptr(),
    );
    if wdm_class.is_null() {
        return r#"{"error":"wdm_class_null"}"#.to_string();
    }
    let get_instance = match api.il2cpp_get_singleton_like_instance_fn {
        Some(f) => f,
        None => return r#"{"error":"no_singleton_fn"}"#.to_string(),
    };
    let wdm = get_instance(wdm_class as *mut c_void);
    if wdm.is_null() {
        return r#"{"error":"wdm_null"}"#.to_string();
    }

    // get_WorkSingleModeData
    let wsm_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkSingleModeData").as_ptr(),
    );
    if wsm_class.is_null() {
        return r#"{"error":"wsm_class_null"}"#.to_string();
    }
    let wsm = call_getter_ref(wdm_class, wdm, "get_WorkSingleModeData");
    if wsm.is_null() {
        return r#"{"error":"wsm_null"}"#.to_string();
    }

    // 2. Read total_race_count and win_count
    let total_races = call_getter_int(wsm_class, wsm, "get_TotalRaceCount");
    let win_count = call_getter_int(wsm_class, wsm, "get_WinCount");

    // 3. Read WinSaddleArray — List<SingleModeWinsSaddle>
    let saddle_arr = call_getter_on_instance(wsm_class, wsm, "get_WinSaddleArray");
    let mut saddle_count = 0i32;
    let mut saddle_entries: Vec<String> = Vec::new();
    if !saddle_arr.is_null() {
        let ab = saddle_arr as *const u8;
        // IL2CPP List<T>: _items (T[] at +0x10), _size (int at +0x18)
        let items_ptr = std::ptr::read_unaligned::<usize>(ab.add(0x10) as *const usize);
        saddle_count = std::ptr::read_unaligned::<i32>(ab.add(0x18) as *const i32);

        // Find SingleModeWinsSaddle class for method calls
        let saddle_class = find_class(
            image,
            to_cstr("Gallop").as_ptr(),
            to_cstr("SingleModeWinsSaddle").as_ptr(),
        );

        for i in 0..saddle_count {
            let elem_ptr = std::ptr::read_unaligned::<usize>(
                (items_ptr + (i as usize) * std::mem::size_of::<usize>()) as *const usize,
            );
            if elem_ptr == 0 {
                continue;
            }

            // Call get_Name on the saddle object
            let name = if !saddle_class.is_null() {
                let n = call_getter_string(saddle_class, elem_ptr as *mut c_void, "get_Name");
                if n.is_null() {
                    String::new()
                } else {
                    read_il2cpp_string(n)
                }
            } else {
                String::new()
            };

            // Call get_Type
            let stype = if !saddle_class.is_null() {
                call_getter_int(saddle_class, elem_ptr as *mut c_void, "get_Type")
            } else {
                -1
            };

            // Call IsRelationBonusWinSaddle (returns bool)
            let is_relation_bonus = if !saddle_class.is_null() {
                let get_method_fn = resolve_il2cpp_symbol("il2cpp_class_get_method_from_name");
                let invoke_fn = resolve_il2cpp_symbol("il2cpp_runtime_invoke");
                if !get_method_fn.is_null() && !invoke_fn.is_null() {
                    type FnGetMethod = unsafe extern "C" fn(*mut c_void, *const c_char, i32) -> *mut c_void;
                    type FnInvoke = unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void, *mut c_void) -> *mut c_void;
                    let f: FnGetMethod = std::mem::transmute(get_method_fn);
                    let inv: FnInvoke = std::mem::transmute(invoke_fn);
                    let m = f(saddle_class, to_cstr("IsRelationBonusWinSaddle").as_ptr(), 0);
                    if !m.is_null() {
                        let ret = inv(m, elem_ptr as *mut c_void, std::ptr::null_mut(), std::ptr::null_mut());
                        ret as i32 != 0
                    } else {
                        false
                    }
                } else {
                    false
                }
            } else {
                false
            };

            // Call GetRelationPoint
            let relation_point = if !saddle_class.is_null() {
                let get_method_fn = resolve_il2cpp_symbol("il2cpp_class_get_method_from_name");
                let invoke_fn = resolve_il2cpp_symbol("il2cpp_runtime_invoke");
                if !get_method_fn.is_null() && !invoke_fn.is_null() {
                    type FnGetMethod = unsafe extern "C" fn(*mut c_void, *const c_char, i32) -> *mut c_void;
                    type FnInvoke = unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void, *mut c_void) -> *mut c_void;
                    let f: FnGetMethod = std::mem::transmute(get_method_fn);
                    let inv: FnInvoke = std::mem::transmute(invoke_fn);
                    let m = f(saddle_class, to_cstr("GetRelationPoint").as_ptr(), 0);
                    if !m.is_null() {
                        let ret = inv(m, elem_ptr as *mut c_void, std::ptr::null_mut(), std::ptr::null_mut());
                        if !ret.is_null() {
                            std::ptr::read_unaligned::<i32>(ret as *const i32)
                        } else {
                            0
                        }
                    } else {
                        0
                    }
                } else {
                    0
                }
            } else {
                0
            };

            saddle_entries.push(format!(
                r#"{{"index":{},"name":"{}","type":{},"is_relation_bonus":{},"relation_point":{}}}"#,
                i,
                json_escape(&name),
                stype,
                is_relation_bonus,
                relation_point,
            ));
        }
    }

    // 4. Read parent candidates' WinSaddleArray via SuccessionCharaData
    // Get WorkSingleModeCharaData → SuccessionTrainedCharaInfo
    let chara_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkSingleModeCharaData").as_ptr(),
    );
    let chara_obj = if !chara_class.is_null() {
        call_getter_ref(wdm_class, wdm, "get_WorkSingleModeCharaData")
    } else {
        std::ptr::null_mut()
    };

    let mut parent_saddles_json: Vec<String> = Vec::new();
    if !chara_obj.is_null() && !chara_class.is_null() {
        let sci_class = find_class(
            image,
            to_cstr("Gallop").as_ptr(),
            to_cstr("SuccessionCharaInfo").as_ptr(),
        );

        for (label, getter_name) in [
            ("parent1", "get_SuccessionTrainedCharaInfoFirst"),
            ("parent2", "get_SuccessionTrainedCharaInfoSecond"),
        ] {
            let sci = call_getter_ref(chara_class, chara_obj, getter_name);
            if sci.is_null() {
                continue;
            }

            let chara_id = if !sci_class.is_null() {
                call_getter_int(sci_class, sci, "get_TrainedCharaId")
            } else {
                0
            };

            // Try to get WinSaddleArray from SuccessionCharaInfo
            let p_saddles = call_getter_on_instance(sci_class, sci, "get_WinSaddleArray");
            let mut p_count = 0i32;
            let mut p_entries: Vec<String> = Vec::new();

            if !p_saddles.is_null() {
                let pb = p_saddles as *const u8;
                let p_items = std::ptr::read_unaligned::<usize>(pb.add(0x10) as *const usize);
                p_count = std::ptr::read_unaligned::<i32>(pb.add(0x18) as *const i32);

                let saddle_class = find_class(
                    image,
                    to_cstr("Gallop").as_ptr(),
                    to_cstr("SingleModeWinsSaddle").as_ptr(),
                );

                for i in 0..p_count.min(30) {
                    let elem_ptr = std::ptr::read_unaligned::<usize>(
                        (p_items + (i as usize) * std::mem::size_of::<usize>()) as *const usize,
                    );
                    if elem_ptr == 0 {
                        continue;
                    }
                    let name = if !saddle_class.is_null() {
                        let n = call_getter_string(saddle_class, elem_ptr as *mut c_void, "get_Name");
                        if n.is_null() { String::new() } else { read_il2cpp_string(n) }
                    } else {
                        String::new()
                    };
                    let stype = if !saddle_class.is_null() {
                        call_getter_int(saddle_class, elem_ptr as *mut c_void, "get_Type")
                    } else {
                        -1
                    };
                    p_entries.push(format!(
                        r#"{{"name":"{}","type":{}}}"#,
                        json_escape(&name),
                        stype,
                    ));
                }
            }

            parent_saddles_json.push(format!(
                r#"{{"label":"{}","chara_id":{},"saddle_count":{},"saddles":[{}]}}"#,
                label,
                chara_id,
                p_count,
                p_entries.join(","),
            ));
        }
    }

    // 5. Cross-reference with MDB for relation_group_id mapping
    let mut mdb_saddle_map_json: Vec<String> = Vec::new();
    let mut relation_groups_json: Vec<String> = Vec::new();

    if let Some(mdb_path) = find_mdb_path() {
        if let Ok(conn) = Connection::open_with_flags(&mdb_path, OpenFlags::SQLITE_OPEN_READ_ONLY) {
            // Map: win_saddle entries from MDB with their relation_group_id
            if let Ok(mut stmt) = conn.prepare(
                "SELECT id, relation_group_id, condition, win_saddle_type, race_instance_id_1, race_instance_id_2 FROM single_mode_wins_saddle WHERE win_saddle_type=3 AND relation_group_id > 0 ORDER BY relation_group_id"
            ) {
                let rows: Vec<String> = stmt.query_map([], |row| {
                    Ok(format!(
                        r#"{{"id":{},"rel_group":{},"cond":{},"type":{},"race1":{},"race2":{}}}"#,
                        row.get::<_, i32>(0).unwrap_or(0),
                        row.get::<_, i32>(1).unwrap_or(0),
                        row.get::<_, i32>(2).unwrap_or(0),
                        row.get::<_, i32>(3).unwrap_or(0),
                        row.get::<_, i32>(4).unwrap_or(0),
                        row.get::<_, i32>(5).unwrap_or(0),
                    ))
                }).unwrap().filter_map(|r| r.ok()).collect();
                mdb_saddle_map_json = rows;
            }

            // succession_relation: check which relation_types give points
            // The G1 win groups are type 1-34 (1pt each)
            if let Ok(mut stmt) = conn.prepare(
                "SELECT relation_type, relation_point FROM succession_relation WHERE relation_type BETWEEN 1 AND 200 ORDER BY relation_type"
            ) {
                let rows: Vec<String> = stmt.query_map([], |row| {
                    Ok(format!(
                        r#"{{"type":{},"point":{}}}"#,
                        row.get::<_, i32>(0).unwrap_or(0),
                        row.get::<_, i32>(1).unwrap_or(0),
                    ))
                }).unwrap().filter_map(|r| r.ok()).collect();
                relation_groups_json = rows;
            }

            // Get race names for G1 race_instance_ids
            // race_instance_id 100301 → race_id → text_data category=32
        }
    }

    // 6. Build output
    format!(
        r#"{{"ok":true,"total_races":{},"win_count":{},"saddle_count":{},"win_saddles":[{}],"parent_saddles":[{}],"mdb_saddle_map":[{}],"relation_groups":[{}]}}"#,
        total_races,
        win_count,
        saddle_count,
        saddle_entries.join(","),
        parent_saddles_json.join(","),
        mdb_saddle_map_json.join(","),
        relation_groups_json.join(","),
    )
}

/// Returns current turn info + history from training log
/// Data sources:
///   - WorkSingleModeData: Month, Half, Turn
///   - WorkSingleModeCharaData: all stats, motivation
///   - SingleModeTurn (mdb): turn config (year, period, training set)
///   - Training log snapshots
unsafe fn read_turn_log() -> String {
    if API.is_null() {
        return r#"{"error":"api_null"}"#.to_string();
    }
    let image = match get_image() {
        img if !img.is_null() => img,
        _ => return r#"{"error":"image_null"}"#.to_string(),
    };

    let wdm_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkDataManager").as_ptr(),
    );
    if wdm_class.is_null() {
        return r#"{"error":"no_wdm"}"#.to_string();
    }
    let wdm_inst = get_singleton(wdm_class);
    if wdm_inst.is_null() {
        return r#"{"error":"no_wdm_inst"}"#.to_string();
    }
    log_predict_step("P:wdm");

    let sm_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkSingleModeData").as_ptr(),
    );
    let sm_obj = call_getter_ref(wdm_class, wdm_inst, "get_SingleMode");
    if sm_obj.is_null() {
        return r#"{"error":"no_sm"}"#.to_string();
    }

    let chara_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkSingleModeCharaData").as_ptr(),
    );
    let chara_obj = call_getter_ref(sm_class, sm_obj, "get_Character");
    if chara_obj.is_null() {
        return r#"{"error":"no_chara"}"#.to_string();
    }

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
                let tp = std::ptr::read_unaligned::<*mut c_void>(
                    tb.add(IL2CPP_LIST_ITEMS_OFF + i * IL2CPP_LIST_ITEM_SIZE) as *const *mut c_void,
                );
                if tp.is_null() {
                    continue;
                }
                let bytes = tp as *const u8;
                // TrainingLevelInfo: commandId at 0x10, level at 0x14 (IL2CPP_COMMAND_ID_OFF/IL2CPP_COMMAND_LEVEL_OFF)
                let cmd_id =
                    std::ptr::read_unaligned::<i32>(bytes.add(IL2CPP_COMMAND_ID_OFF) as *const i32);
                let level = std::ptr::read_unaligned::<i32>(
                    bytes.add(IL2CPP_COMMAND_LEVEL_OFF) as *const i32
                );
                tls.push(format!(r#"{{"command_id":{},"level":{}}}"#, cmd_id, level));
            }
            tl_json = format!("[{}]", tls.join(","));
        }
    }

    format!(
        r#"{{"version":"{}","current":{{"month":{},"half":{},"scenario_id":{},"stats":{{"speed":{},"stamina":{},"power":{},"guts":{},"wiz":{}}},"vital":{},"max_vital":{},"motivation":{},"skill_point":{},"fan":{}}},"training_levels":{},"turn_config":[{}],"history":{}}}"#,
        PLUGIN_VERSION,
        mon,
        half,
        sid,
        spd,
        sta,
        pow_,
        gut,
        wiz,
        vit,
        mvit,
        mot,
        spt,
        fan,
        tl_json,
        turn_config_json,
        log_json
    )
}

/// /event/recommend — Event recommendation based on current game state + event data
/// Reads mdb event data and matches against current support cards + chara
/// Returns: matching events with choice evaluations
unsafe fn read_event_recommend() -> String {
    // Event data is all from mdb (like /events), plus current game state
    if API.is_null() {
        return r#"{"error":"api_null"}"#.to_string();
    }
    let image = match get_image() {
        img if !img.is_null() => img,
        _ => return r#"{"error":"image_null"}"#.to_string(),
    };

    let wdm_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkDataManager").as_ptr(),
    );
    if wdm_class.is_null() {
        return r#"{"error":"no_wdm"}"#.to_string();
    }
    let wdm_inst = get_singleton(wdm_class);
    if wdm_inst.is_null() {
        return r#"{"error":"no_wdm_inst"}"#.to_string();
    }
    log_predict_step("P:wdm");

    let sm_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkSingleModeData").as_ptr(),
    );
    let sm_obj = call_getter_ref(wdm_class, wdm_inst, "get_SingleMode");
    if sm_obj.is_null() {
        return r#"{"error":"no_sm"}"#.to_string();
    }

    let chara_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkSingleModeCharaData").as_ptr(),
    );
    let chara_obj = call_getter_ref(sm_class, sm_obj, "get_Character");
    if chara_obj.is_null() {
        return r#"{"error":"no_chara"}"#.to_string();
    }

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
            let al =
                std::ptr::read_unaligned::<usize>(ab.add(IL2CPP_LIST_COUNT_OFF) as *const usize);
            if al > 0 && al < 100 {
                for i in 0..al {
                    let ep = std::ptr::read_unaligned::<*mut c_void>(
                        ab.add(IL2CPP_LIST_ITEMS_OFF + i * IL2CPP_LIST_ITEM_SIZE)
                            as *const *mut c_void,
                    );
                    if ep.is_null() {
                        continue;
                    }
                    let bytes = ep as *const u8;
                    let sc_id = std::ptr::read_unaligned::<i32>(
                        bytes.add(IL2CPP_SUPPORT_CARD_ID_OFF) as *const i32,
                    );
                    support_card_ids.push(sc_id);
                }
            }
        }
    } else {
        let ab = sc_arr as *const u8;
        let al = std::ptr::read_unaligned::<usize>(ab.add(IL2CPP_LIST_COUNT_OFF) as *const usize);
        if al > 0 && al < 100 {
            for i in 0..al {
                let ep = std::ptr::read_unaligned::<*mut c_void>(
                    ab.add(IL2CPP_LIST_ITEMS_OFF + i * IL2CPP_LIST_ITEM_SIZE) as *const *mut c_void,
                );
                if ep.is_null() {
                    continue;
                }
                let bytes = ep as *const u8;
                let sc_id = std::ptr::read_unaligned::<i32>(
                    bytes.add(IL2CPP_SUPPORT_CARD_ID_OFF) as *const i32
                );
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
                    let ep = std::ptr::read_unaligned::<*mut c_void>(
                        eb.add(IL2CPP_LIST_ITEMS_OFF + ei * IL2CPP_LIST_ITEM_SIZE)
                            as *const *mut c_void,
                    );
                    if ep.is_null() {
                        continue;
                    }
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
            let sc_ids_str: Vec<String> =
                support_card_ids.iter().map(|id| id.to_string()).collect();
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
            all_events_count = conn
                .query_row("SELECT COUNT(*) FROM single_mode_story_data", [], |r| {
                    r.get(0)
                })
                .unwrap_or(0);

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
                r#"{{"version":"3.22.91","current_state":{{"card_id":{},"scenario_id":{},"month":{},"half":{},"stats":{{"speed":{},"stamina":{},"power":{},"guts":{},"wiz":{}}},"vital":{},"max_vital":{},"skill_point":{}}},"support_card_ids":[{}],"eval_chara_ids":[{}],"total_events":{},"matching_events":{},"events":[{}],"choice_rewards":[{}]}}"#,
                card_id,
                sid,
                mon,
                half,
                spd,
                sta,
                pow_,
                gut,
                wiz,
                vit,
                mvit,
                spt,
                support_card_ids
                    .iter()
                    .map(|id| id.to_string())
                    .collect::<Vec<_>>()
                    .join(","),
                eval_chara_ids
                    .iter()
                    .map(|id| id.to_string())
                    .collect::<Vec<_>>()
                    .join(","),
                all_events_count,
                matching_events_count,
                matching_events.join(","),
                choice_rewards.join(",")
            )
        } else {
            format!(
                r#"{{"version":"3.22.91","error":"mdb_open_failed","current_state":{{"card_id":{},"scenario_id":{}}}}}"#,
                card_id, sid
            )
        }
    } else {
        format!(
            r#"{{"version":"3.22.91","error":"mdb_not_found","current_state":{{"card_id":{},"scenario_id":{}}}}}"#,
            card_id, sid
        )
    }
}

/// v3.22.35: /debug/storydata — Pure memory read: dump DataSet + SingleModeData fields + hex
/// NO runtime_invoke on any new path. Only reads memory + uses existing safe getters.
/// 1. Get DataSet pointer via existing safe getters
/// 2. Dump all class fields + offsets (metadata only)
/// 3. Hex dump the object memory
/// 4. For ObscuredInt fields at known offsets, decrypt directly

/// v3.22.35: /debug/storydata — Read _storyInfoListDic + EventChoiceRewardDict from SingleModeData
/// Pure memory read: read pointers at known offsets, dump class info + hex

/// v3.22.35: /debug/storydata — Read event dictionaries from SingleModeData
/// Reads _storyInfoListDic, EventChoiceRewardDict, StoryEventBonusDict
/// Traverses Dictionary`2 _entries array to dump key/value objects

/// v3.22.35: /debug/storydata — Pure memory read event dictionaries
/// ZERO runtime_invoke calls. Only reads raw pointers + hex.
unsafe fn debug_storydata() -> String {
    // ★ v3.22.35: Acquire READ_MUTEX to share SIGSEGV_JMP_BUF safely with read_summary
    let _lock = READ_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    // ★ v3.22.35: SIGSEGV recovery — same pattern as read_summary
    let jmp_result = unsafe { sys_sigsetjmp(SIGSEGV_JMP_BUF.as_mut_ptr(), 1) };
    if jmp_result != 0 {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        SIGSEGV_COOLDOWN_UNTIL.store(now + 60, std::sync::atomic::Ordering::Relaxed);
        return r#"{"error":"sigsegv_recovered_in_storydata"}"#.to_string();
    }
    SIGSEGV_RECOVERY.store(true, std::sync::atomic::Ordering::Relaxed);
    let result = debug_storydata_inner();
    SIGSEGV_RECOVERY.store(false, std::sync::atomic::Ordering::Relaxed);
    result
}

unsafe fn debug_storydata_inner() -> String {
    if API.is_null() {
        return r#"{"error":"api_null"}"#.to_string();
    }
    let image = match get_image() {
        img if !img.is_null() => img,
        _ => return r#"{"error":"image_null"}"#.to_string(),
    };

    let wdm_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkDataManager").as_ptr(),
    );
    if wdm_class.is_null() {
        return r#"{"error":"no_wdm"}"#.to_string();
    }
    let wdm_inst = get_singleton(wdm_class);
    if wdm_inst.is_null() {
        return r#"{"error":"no_wdm_inst"}"#.to_string();
    }
    let _sm_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkSingleModeData").as_ptr(),
    );
    let sm_obj = call_getter_ref(wdm_class, wdm_inst, "get_SingleMode");
    if sm_obj.is_null() {
        return r#"{"error":"no_sm"}"#.to_string();
    }
    let sm_base = sm_obj as *const u8;

    // Helper: read Dictionary`2 — pure memory, zero invoke
    // Dict layout: _buckets@0x10, _entries@0x18, _count@0x20
    let read_dict = |dic_ptr: *mut c_void, dic_name: &str| -> String {
        if dic_ptr.is_null() {
            return format!(r#"{{"name":"{}","ptr":"null"}}"#, dic_name);
        }
        let db = dic_ptr as *const u8;
        let count = std::ptr::read_unaligned::<i32>(db.add(0x20) as *const i32);
        let entries_ptr =
            std::ptr::read_unaligned::<*mut c_void>(db.add(0x18) as *const *mut c_void);

        if count <= 0 || entries_ptr.is_null() {
            let mut hex: Vec<String> = Vec::new();
            for off in (0..0x60).step_by(4) {
                let val = std::ptr::read_unaligned::<i32>(db.add(off) as *const i32);
                hex.push(format!(r#""0x{:02x}:{}"#, off, val));
            }
            return format!(
                r#"{{"name":"{}","ptr":"{:p}","count":{},"hex":{{{}}}}}"#,
                dic_name,
                dic_ptr,
                count,
                hex.join(",")
            );
        }

        // Il2CppArray: header 0x20 bytes, then Entry[] items
        // Entry struct: hashCode(i32) + next(i32) + key(ptr) + value(ptr) = 0x18 bytes
        let count = count as usize;
        let arr_base = entries_ptr as *const u8;
        let entry_size: usize = 0x18;
        let mut entries: Vec<String> = Vec::new();
        let max_entries = if count > 30 { 30 } else { count };

        for i in 0..max_entries {
            let e_off = 0x20 + i * entry_size;
            let key_ptr = std::ptr::read_unaligned::<*mut c_void>(
                arr_base.add(e_off + 0x08) as *const *mut c_void
            );
            let val_ptr = std::ptr::read_unaligned::<*mut c_void>(
                arr_base.add(e_off + 0x10) as *const *mut c_void
            );

            let key_info = if !key_ptr.is_null() {
                let kb = key_ptr as *const u8;
                let mut kh: Vec<String> = Vec::new();
                for off in (0..0x20).step_by(4) {
                    let v = std::ptr::read_unaligned::<i32>(kb.add(off) as *const i32);
                    kh.push(format!(r#""0x{:02x}:{}"#, off, v));
                }
                // Get key class name from object header
                let key_klass_ptr =
                    std::ptr::read_unaligned::<*mut c_void>(kb as *const *mut c_void);
                let key_class_name = if !key_klass_ptr.is_null() {
                    get_class_name_from_pointer(key_klass_ptr)
                } else {
                    "null".to_string()
                };
                format!(
                    r#"{{"ptr":"{:p}","class":"{}","hex":{{{}}}}}"#,
                    key_ptr,
                    key_class_name,
                    kh.join(",")
                )
            } else {
                r#"{"ptr":"null"}"#.to_string()
            };

            let val_info = if !val_ptr.is_null() {
                let vb = val_ptr as *const u8;
                let mut vh: Vec<String> = Vec::new();
                for off in (0..0x80).step_by(4) {
                    let v = std::ptr::read_unaligned::<i32>(vb.add(off) as *const i32);
                    vh.push(format!(r#""0x{:02x}:{}"#, off, v));
                }
                let val_klass_ptr =
                    std::ptr::read_unaligned::<*mut c_void>(vb as *const *mut c_void);
                let val_class_name = if !val_klass_ptr.is_null() {
                    get_class_name_from_pointer(val_klass_ptr)
                } else {
                    "null".to_string()
                };
                format!(
                    r#"{{"ptr":"{:p}","class":"{}","hex":{{{}}}}}"#,
                    val_ptr,
                    val_class_name,
                    vh.join(",")
                )
            } else {
                r#"{"ptr":"null"}"#.to_string()
            };

            entries.push(format!(
                r#"{{"idx":{},"key":{},"val":{}}}"#,
                i, key_info, val_info
            ));
        }

        format!(
            r#"{{"name":"{}","ptr":"{:p}","count":{},"entries":[{}]}}"#,
            dic_name,
            dic_ptr,
            count,
            entries.join(",")
        )
    };

    // Read three dictionaries at known offsets — pure pointer deread, zero invoke
    let story_dic_ptr =
        std::ptr::read_unaligned::<*mut c_void>(sm_base.add(0x38) as *const *mut c_void);
    let story_info = read_dict(story_dic_ptr, "storyInfoListDic");

    let reward_dic_ptr =
        std::ptr::read_unaligned::<*mut c_void>(sm_base.add(0x1b8) as *const *mut c_void);
    let reward_info = read_dict(reward_dic_ptr, "eventChoiceRewardDict");

    let bonus_dic_ptr =
        std::ptr::read_unaligned::<*mut c_void>(sm_base.add(0x180) as *const *mut c_void);
    let bonus_info = read_dict(bonus_dic_ptr, "storyEventBonusDict");

    format!(
        r#"{{"story_info_list_dic":{},"event_choice_reward_dict":{},"story_event_bonus_dict":{}}}"#,
        story_info, reward_info, bonus_info
    )
}

/// ★ v3.22.35: /debug/all — Aggregate summary + scenario + storydata + cmdinfo + rameninfo in one call
/// IMPORTANT: Must acquire READ_MUTEX + sigsetjmp ONCE here, then call _inner functions directly
/// to avoid deadlock (read_summary and debug_storydata both try to lock READ_MUTEX internally)
unsafe fn debug_all() -> String {
    let mut parts: Vec<String> = Vec::new();

    // ★ Acquire READ_MUTEX once for the entire aggregation
    let _lock = READ_MUTEX.lock().unwrap_or_else(|e| e.into_inner());

    // ★ Set up sigsetjmp recovery once for the entire call
    let jmp_result = unsafe { sys_sigsetjmp(SIGSEGV_JMP_BUF.as_mut_ptr(), 1) };
    if jmp_result != 0 {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        SIGSEGV_COOLDOWN_UNTIL.store(now + 60, std::sync::atomic::Ordering::Relaxed);
        SIGSEGV_RECOVERY.store(false, std::sync::atomic::Ordering::Relaxed);
        return r#"{"error":"sigsegv_recovered_in_debug_all"}"#.to_string();
    }
    SIGSEGV_RECOVERY.store(true, std::sync::atomic::Ordering::Relaxed);

    // 1. summary — call _inner directly (skip its own mutex + sigsetjmp)
    let summary = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
        read_summary_inner()
    }))
    .unwrap_or_else(|_| r#"{"error":"summary_panic"}"#.to_string());
    parts.push(format!(r#""summary":{}"#, summary));

    // 2. scenario
    let scenario = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
        read_scenario_detail()
    }))
    .unwrap_or_else(|_| r#"{"error":"scenario_panic"}"#.to_string());
    parts.push(format!(r#""scenario":{}"#, scenario));

    // 3. storydata — call _inner directly (skip its own mutex + sigsetjmp)
    let storydata = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
        debug_storydata_inner()
    }))
    .unwrap_or_else(|_| r#"{"error":"storydata_panic"}"#.to_string());
    parts.push(format!(r#""storydata":{}"#, storydata));

    // 4. cmdinfo
    let cmdinfo =
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe { debug_cmdinfo() }))
            .unwrap_or_else(|_| r#"{"error":"cmdinfo_panic"}"#.to_string());
    parts.push(format!(r#""cmdinfo":{}"#, cmdinfo));

    // 5. rameninfo
    let rameninfo = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
        read_ramen_info()
    }))
    .unwrap_or_else(|_| r#"{"error":"rameninfo_panic"}"#.to_string());
    parts.push(format!(r#""rameninfo":{}"#, rameninfo));

    // ★ Clear recovery flag
    SIGSEGV_RECOVERY.store(false, std::sync::atomic::Ordering::Relaxed);

    format!("{{{}}}", parts.join(","))
}

/// ★ v3.22.39: /debug/gauge — MINIMAL SAFE VERSION
/// Only reads element class names + count. NO dict hex, NO GetGainCount.
/// Will incrementally add features after confirming this doesn't crash.
unsafe fn debug_gauge() -> String {
    if API.is_null() {
        return r#"{"error":"api_null"}"#.to_string();
    }
    let image = match get_image() {
        img if !img.is_null() => img,
        _ => return r#"{"error":"image_null"}"#.to_string(),
    };
    let wdm_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkDataManager").as_ptr(),
    );
    if wdm_class.is_null() {
        return r#"{"error":"no_wdm"}"#.to_string();
    }
    let wdm_inst = get_singleton(wdm_class);
    if wdm_inst.is_null() {
        return r#"{"error":"no_wdm_inst"}"#.to_string();
    }
    let sm_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkSingleModeData").as_ptr(),
    );
    let sm_obj = call_getter_ref(wdm_class, wdm_inst, "get_SingleMode");
    if sm_obj.is_null() {
        return r#"{"error":"no_sm"}"#.to_string();
    }
    let chara_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkSingleModeCharaData").as_ptr(),
    );
    let chara_obj = call_getter_ref(sm_class, sm_obj, "get_Character");
    if chara_obj.is_null() {
        return r#"{"error":"no_chara"}"#.to_string();
    }

    let scenario_id = call_getter_int(chara_class, chara_obj, "get_ScenarioId");
    if scenario_id != 14 {
        return format!(r#"{{"error":"not_ramen","sid":{}}}"#, scenario_id);
    }

    let ramen_sc_obj = try_get_scenario_obj(chara_class, chara_obj, 14);
    if ramen_sc_obj.is_null() {
        return r#"{"error":"no_ramen_sc_obj"}"#.to_string();
    }
    let sc_class = get_class_from_object(ramen_sc_obj);
    let ds_obj = call_getter_ref(sc_class, ramen_sc_obj, "get_DataSet");
    if ds_obj.is_null() {
        return r#"{"error":"no_ds"}"#.to_string();
    }
    let ds_class = get_class_from_object(ds_obj);

    // Read CommandFeelingInfoArray
    let cf_off = cached_find_field_offset(ds_class, "CommandFeelingInfoArray");
    if cf_off < 0 {
        return r#"{"error":"no_CommandFeelingInfoArray_field"}"#.to_string();
    }
    let list_obj = read_ptr_at(ds_obj, cf_off);
    if list_obj.is_null() {
        return r#"{"error":"list_null"}"#.to_string();
    }
    let lb = list_obj as *const u8;
    let llen = std::ptr::read_unaligned::<usize>(lb.add(IL2CPP_LIST_COUNT_OFF) as *const usize);
    if llen == 0 || llen > 100 {
        return format!(r#"{{"error":"bad_len","len":{}}}"#, llen);
    }

    let mut elems: Vec<String> = Vec::new();
    for i in 0..llen {
        let ep = std::ptr::read_unaligned::<*mut c_void>(
            lb.add(IL2CPP_LIST_ITEMS_OFF + i * IL2CPP_LIST_ITEM_SIZE) as *const *mut c_void,
        );
        if ep.is_null() {
            elems.push(format!(r#"{{"idx":{},"error":"null"}}"#, i));
            continue;
        }
        // ★ MINIMAL: only read class name + check _gaugeGainCountDict pointer
        let ep_class = get_class_from_object(ep);
        let ep_class_name = get_class_name_from_pointer(ep_class);

        // Read _gaugeGainCountDict pointer at offset 16 (only if TrainingFeelingEntity)
        let dict_info = if ep_class_name == "TrainingFeelingEntity" {
            let dict_ptr = read_ptr_at(ep, 16); // _gaugeGainCountDict at offset 16
            if dict_ptr.is_null() {
                "dict:null".to_string()
            } else {
                // Read dict _count field safely (Dictionary has count at a known offset)
                // DON'T read hex — just report pointer value
                format!("dict:ptr({:p})", dict_ptr)
            }
        } else {
            format!("not_tfe({})", ep_class_name)
        };

        elems.push(format!(
            r#"{{"idx":{},"class":"{}","dict_info":"{}"}}"#,
            i, ep_class_name, dict_info
        ));
    }

    format!(
        r#"{{"version":"3.22.91","count":{},"elements":[{}]}}"#,
        llen,
        elems.join(",")
    )
}

/// v3.22.39: /debug/gauge2 - Scan ALL DataSet array fields for element class names
/// Purpose: find which array contains TrainingFeelingEntity
unsafe fn debug_gauge2() -> String {
    if API.is_null() {
        return r#"{"error":"api_null"}"#.to_string();
    }
    let image = match get_image() {
        img if !img.is_null() => img,
        _ => return r#"{"error":"image_null"}"#.to_string(),
    };
    let wdm_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkDataManager").as_ptr(),
    );
    if wdm_class.is_null() {
        return r#"{"error":"no_wdm"}"#.to_string();
    }
    let wdm_inst = get_singleton(wdm_class);
    if wdm_inst.is_null() {
        return r#"{"error":"no_wdm_inst"}"#.to_string();
    }
    let sm_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkSingleModeData").as_ptr(),
    );
    let sm_obj = call_getter_ref(wdm_class, wdm_inst, "get_SingleMode");
    if sm_obj.is_null() {
        return r#"{"error":"no_sm"}"#.to_string();
    }
    let chara_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkSingleModeCharaData").as_ptr(),
    );
    let chara_obj = call_getter_ref(sm_class, sm_obj, "get_Character");
    if chara_obj.is_null() {
        return r#"{"error":"no_chara"}"#.to_string();
    }
    let scenario_id = call_getter_int(chara_class, chara_obj, "get_ScenarioId");
    if scenario_id != 14 {
        return format!(r#"{{"error":"not_ramen","sid":{}}}"#, scenario_id);
    }
    let ramen_sc_obj = try_get_scenario_obj(chara_class, chara_obj, 14);
    if ramen_sc_obj.is_null() {
        return r#"{"error":"no_ramen_sc_obj"}"#.to_string();
    }
    let sc_class = get_class_from_object(ramen_sc_obj);
    let ds_obj = call_getter_ref(sc_class, ramen_sc_obj, "get_DataSet");
    if ds_obj.is_null() {
        return r#"{"error":"no_ds"}"#.to_string();
    }
    let ds_class = get_class_from_object(ds_obj);

    let array_fields: &[(&str, i32)] = &[
        ("CommandInfoArray", 16),
        ("FeelingReduceTurnInfoArray", 32),
        ("FeelingTurnInfoArray", 40),
        ("FeelingInfoArray", 48),
        ("CommandFeelingInfoArray", 96),
        ("TrainingExecInfoArray", 104),
        ("CheckPointInfoArray", 168),
    ];

    let mut results: Vec<String> = Vec::new();
    for &(fname, foff) in array_fields {
        let list_obj = read_ptr_at(ds_obj, foff);
        if list_obj.is_null() {
            results.push(format!(r#"{{"field":"{}","status":"null"}}"#, fname));
            continue;
        }
        let lb = list_obj as *const u8;
        let llen = std::ptr::read_unaligned::<usize>(lb.add(IL2CPP_LIST_COUNT_OFF) as *const usize);
        if llen > 1000 {
            results.push(format!(
                r#"{{"field":"{}","status":"bad_len","len":{}}}"#,
                fname, llen
            ));
            continue;
        }
        let max_read = if llen < 3 { llen } else { 3 };
        let mut classes: Vec<String> = Vec::new();
        for i in 0..max_read {
            let ep = std::ptr::read_unaligned::<*mut c_void>(
                lb.add(IL2CPP_LIST_ITEMS_OFF + i * IL2CPP_LIST_ITEM_SIZE) as *const *mut c_void,
            );
            if ep.is_null() {
                classes.push("null".to_string());
            } else {
                let ep_class = get_class_from_object(ep);
                classes.push(get_class_name_from_pointer(ep_class));
            }
        }
        let sample_str = {
            let mut quoted: Vec<String> = Vec::new();
            for c in &classes {
                quoted.push(format!("\"{}\"", c));
            }
            quoted.join(",")
        };
        results.push(format!(
            r#"{{"field":"{}","count":{},"sample_classes":[{}]}}"#,
            fname, llen, sample_str
        ));
    }

    format!(
        r#"{{"version":"3.22.91","arrays":[{}]}}"#,
        results.join(",")
    )
}

/// v3.22.40: /debug/paramsincdec - Read DataSet CommandInfoArray[0].ParamsIncDecInfoArray
/// Purpose: find element class name inside ParamsIncDecInfoArray to locate gauge data
unsafe fn debug_paramsincdec() -> String {
    if API.is_null() {
        return r#"{"error":"api_null"}"#.to_string();
    }
    let image = match get_image() {
        img if !img.is_null() => img,
        _ => return r#"{"error":"image_null"}"#.to_string(),
    };
    let wdm_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkDataManager").as_ptr(),
    );
    if wdm_class.is_null() {
        return r#"{"error":"no_wdm"}"#.to_string();
    }
    let wdm_inst = get_singleton(wdm_class);
    if wdm_inst.is_null() {
        return r#"{"error":"no_wdm_inst"}"#.to_string();
    }
    let sm_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkSingleModeData").as_ptr(),
    );
    let sm_obj = call_getter_ref(wdm_class, wdm_inst, "get_SingleMode");
    if sm_obj.is_null() {
        return r#"{"error":"no_sm"}"#.to_string();
    }
    let chara_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkSingleModeCharaData").as_ptr(),
    );
    let chara_obj = call_getter_ref(sm_class, sm_obj, "get_Character");
    if chara_obj.is_null() {
        return r#"{"error":"no_chara"}"#.to_string();
    }
    let scenario_id = call_getter_int(chara_class, chara_obj, "get_ScenarioId");
    if scenario_id != 14 {
        return format!(r#"{{"error":"not_ramen","sid":{}}}"#, scenario_id);
    }
    let ramen_sc_obj = try_get_scenario_obj(chara_class, chara_obj, 14);
    if ramen_sc_obj.is_null() {
        return r#"{"error":"no_ramen_sc_obj"}"#.to_string();
    }
    let sc_class = get_class_from_object(ramen_sc_obj);
    let ds_obj = call_getter_ref(sc_class, ramen_sc_obj, "get_DataSet");
    if ds_obj.is_null() {
        return r#"{"error":"no_ds"}"#.to_string();
    }

    // Read CommandInfoArray at offset 16 in DataSet
    let cmd_list = read_ptr_at(ds_obj, 16);
    if cmd_list.is_null() {
        return r#"{"error":"cmd_list_null"}"#.to_string();
    }
    let cmd_lb = cmd_list as *const u8;
    let cmd_len =
        std::ptr::read_unaligned::<usize>(cmd_lb.add(IL2CPP_LIST_COUNT_OFF) as *const usize);
    if cmd_len == 0 {
        return r#"{"error":"cmd_empty"}"#.to_string();
    }
    if cmd_len > 100 {
        return format!(r#"{{"error":"cmd_bad_len","len":{}}}"#, cmd_len);
    }

    // Read CommandType (offset 16) and CommandId (offset 36) from each CommandInfo
    // Read ParamsIncDecInfoArray (offset 56) and its elements: target_type@16, value@20
    let mut cmd_details: Vec<String> = Vec::new();
    for ci in 0..cmd_len {
        let ce = std::ptr::read_unaligned::<*mut c_void>(
            cmd_lb.add(IL2CPP_LIST_ITEMS_OFF + ci * IL2CPP_LIST_ITEM_SIZE) as *const *mut c_void,
        );
        if ce.is_null() {
            cmd_details.push(format!(r#"{{"idx":{},"error":"null"}}"#, ci));
            continue;
        }
        // Read CommandType (ObscuredInt at offset 16 — use read_obscured_int_at to avoid wild ptr)
        let cmd_type = read_obscured_int_at(ce, 16);
        // Read CommandId (ObscuredInt at offset 36 — use read_obscured_int_at to avoid wild ptr)
        let cmd_id = read_obscured_int_at(ce, RAMEN_CMD_COMMAND_ID_OFF as i32);

        let ce_params = read_ptr_at(ce, RAMEN_CMD_PARAMS_ARRAY_OFF as i32);
        if ce_params.is_null() {
            cmd_details.push(format!(
                r#"{{"idx":{},"cmd_type":{},"cmd_id":{},"params":[]}}"#,
                ci, cmd_type, cmd_id
            ));
            continue;
        }
        let ce_plb = ce_params as *const u8;
        let ce_plen =
            std::ptr::read_unaligned::<usize>(ce_plb.add(IL2CPP_LIST_COUNT_OFF) as *const usize);
        if ce_plen > 1000 {
            cmd_details.push(format!(
                r#"{{"idx":{},"cmd_type":{},"cmd_id":{},"params_error":"bad_len"}}"#,
                ci, cmd_type, cmd_id
            ));
            continue;
        }
        let mut params: Vec<String> = Vec::new();
        for pi in 0..ce_plen {
            let pe = std::ptr::read_unaligned::<*mut c_void>(
                ce_plb.add(IL2CPP_LIST_ITEMS_OFF + pi * IL2CPP_LIST_ITEM_SIZE)
                    as *const *mut c_void,
            );
            if pe.is_null() {
                params.push(r#"{"target_type":null,"value":null}"#.to_string());
                continue;
            }
            // SingleModeParamsIncDecInfo: target_type at offset 16 (int), value at offset 20 (int)
            let tt = std::ptr::read_unaligned::<i32>(
                (pe as *const u8).add(PARAMS_INCDEC_TARGET_TYPE_OFF) as *const i32,
            );
            let vv = std::ptr::read_unaligned::<i32>(
                (pe as *const u8).add(PARAMS_INCDEC_VALUE_OFF) as *const i32,
            );
            params.push(format!(r#"{{"target_type":{},"value":{}}}"#, tt, vv));
        }
        let params_str = params.join(",");
        cmd_details.push(format!(
            r#"{{"idx":{},"cmd_type":{},"cmd_id":{},"params_count":{},"params":[{}]}}"#,
            ci, cmd_type, cmd_id, ce_plen, params_str
        ));
    }

    // IsGaugeGained is bool — use call_getter_bool (v3.22.51)
    let ds_class = get_class_from_object(ds_obj);
    let is_gauge_gained = if !ds_class.is_null() {
        if call_getter_bool(ds_class, ds_obj, "get_IsGaugeGained") {
            1
        } else {
            0
        }
    } else {
        -1
    };

    format!(
        r#"{{"version":"3.22.91","cmd_len":{},"cmds":[{}],"IsGaugeGained":{}}}"#,
        cmd_len,
        cmd_details.join(","),
        is_gauge_gained
    )
}

/// 一键查找训练种子：WorkDataManager → WorkSingleModeData → _fixedTurnCharaSeed
/// 自动完成 /singletons + read_mem(offset 96) + read_mem(offset 408) 的手动3步流程
unsafe fn debug_training_seed() -> String {
    if API.is_null() { return r#"{"error":"api_null"}"#.to_string(); }
    let image = get_image();
    if image.is_null() { return r#"{"error":"image_null"}"#.to_string(); }
    let wdm_cls = find_class_by_short_name(image, "WorkDataManager");
    if wdm_cls.is_null() { return r#"{"error":"wdm_class_not_found"}"#.to_string(); }
    let wdm_inst = get_singleton(wdm_cls);
    if wdm_inst.is_null() {
        return r#"{"error":"wdm_null","hint":"game_not_loaded"}"#.to_string();
    }
    let sm_ptr = std::ptr::read_unaligned::<usize>((wdm_inst as *const u8).add(96) as *const usize);
    if sm_ptr == 0 {
        return r#"{"error":"single_mode_null","hint":"not_in_training_scene"}"#.to_string();
    }
    let value = read_obscured_int_at(sm_ptr as *const c_void, 408);
    format!(
        r#"{{"ok":true,"field":"WorkSingleModeData._fixedTurnCharaSeed","offset":408,"type":"CodeStage.AntiCheat.ObscuredTypes.ObscuredInt","fixed_turn_chara_seed":{},"is_complete_prng_state":false}}"#,
        value
    )
}
// ★ v3.22.98: ExecTraining hook — intercept before training to read seed + predict
static mut EXEC_TRAINING_HOOK_INSTALLED: bool = false;
static mut ORIG_EXEC_TRAINING_PROLOGUE: [u8; 16] = [0; 16];
static mut EXEC_TRAINING_ADDR: usize = 0;
// ExecTraining hook is retained only to preserve existing hook behavior; no RNG observations are captured.
static mut LAST_FAILURE_RATE: i32 = -1;
static mut FAILURE_RATE_HOOK_INSTALLED: bool = false;
static mut ORIG_FAILURE_RATE_PROLOGUE: [u8; 16] = [0; 16];
static mut FAILURE_RATE_ADDR: usize = 0;

extern "C" fn exec_training_hook(param1: *mut c_void, param2: *mut c_void) {
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
        let trampoline = interceptor_get_trampoline(exec_training_hook as usize);
        if trampoline == 0 {
            ura_log(1, "exec_training_hook: trampoline not found");
            return;
        }
        type FnType = unsafe extern "C" fn(*mut c_void, *mut c_void);
        let original: FnType = std::mem::transmute(trampoline);
        original(param1, param2);
    }));
}
unsafe fn read_training_context_inner() -> (i32, i32) {
    if API.is_null() { return (-1, -1); }
    let image = get_image();
    if image.is_null() { return (-1, -1); }
    let wdm_cls = find_class_by_short_name(image, "WorkDataManager");
    if wdm_cls.is_null() { return (-1, -1); }
    let wdm_inst = get_singleton(wdm_cls);
    if wdm_inst.is_null() { return (-1, -1); }
    let sm_ptr = std::ptr::read_unaligned::<usize>((wdm_inst as *const u8).add(96) as *const usize);
    if sm_ptr == 0 { return (-1, -1); }
    let turn = read_int_at(sm_ptr as *const c_void, 68);
    let sm_class = find_class_by_short_name(image, "WorkSingleModeData");
    if sm_class.is_null() { return (turn, -1); }
    let chara = call_getter_on_instance(sm_class, sm_ptr as *const c_void, "get_Character");
    let scenario_id = if chara.is_null() { -1 } else { read_obscured_int_at(chara, 568) };
    (turn, scenario_id)
}

// Read motivation (干劲) from character data, same path as read_seed_inner
// Returns 1-5 (Worst/Bad/Normal/Good/Best), -1 on error
unsafe fn read_motivation_inner() -> i32 {
    if API.is_null() {
        return -1;
    }
    let image = get_image();
    if image.is_null() {
        return -1;
    }

    let wdm_cls = find_class_by_short_name(image, "WorkDataManager");
    if wdm_cls.is_null() {
        return -1;
    }
    let wdm_inst = get_singleton(wdm_cls);
    if wdm_inst.is_null() {
        return -1;
    }

    let sm_ptr = std::ptr::read_unaligned::<usize>((wdm_inst as *const u8).add(96) as *const usize);
    if sm_ptr == 0 {
        return -1;
    }

    let sm_class = find_class_by_short_name(image, "WorkSingleModeData");
    if sm_class.is_null() {
        return -1;
    }

    let chara_obj = call_getter_on_instance(sm_class, sm_ptr as *const c_void, "get_Character");
    if chara_obj.is_null() {
        return -1;
    }

    let chara_class = find_class_by_short_name(image, "WorkSingleModeCharaData");
    if chara_class.is_null() {
        return -1;
    }

    call_getter_int(chara_class, chara_obj, "get_Motivation")
}

// Hook SingleModeTrainingFailureRateService.GetTrainingFailureRateIgnoreCharaEffect
// Captures the last failure rate (0-10000 = 0%-100%) for use in training log
extern "C" fn failure_rate_hook(param1: *mut c_void, param2: *mut c_void) -> i32 {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
        let trampoline = interceptor_get_trampoline(failure_rate_hook as usize);
        if trampoline == 0 {
            ura_log(1, "failure_rate_hook: trampoline not found");
            return 0;
        }
        type FnType = unsafe extern "C" fn(*mut c_void, *mut c_void) -> i32;
        let original: FnType = std::mem::transmute(trampoline);
        let result = original(param1, param2);
        LAST_FAILURE_RATE = result;
        result
    }))
    .unwrap_or_else(|e| {
        unsafe {
            ura_log(1, &format!("failure_rate_hook: panic: {:?}", e));
        }
        0
    })
}

unsafe fn install_failure_rate_hook() {
    if FAILURE_RATE_HOOK_INSTALLED {
        return;
    }
    if API.is_null() {
        return;
    }

    let image = match get_image() {
        img if !img.is_null() => img,
        _ => return,
    };

    let class = find_class_by_short_name(image, "SingleModeTrainingFailureRateService");
    if class.is_null() {
        ura_log(3, "FailureRate hook: class not found");
        return;
    }

    let method_addr = find_method_addr(class, "GetTrainingFailureRateIgnoreCharaEffect", 2);
    if method_addr == 0 {
        ura_log(3, "FailureRate hook: method not found");
        return;
    }

    FAILURE_RATE_ADDR = method_addr;
    install_hook_safe(
        "FailureRate",
        method_addr,
        failure_rate_hook as usize,
        &mut ORIG_FAILURE_RATE_PROLOGUE,
    );
    FAILURE_RATE_HOOK_INSTALLED = true;
}

unsafe fn install_exec_training_hook() {
    if EXEC_TRAINING_HOOK_INSTALLED {
        return;
    }
    if API.is_null() {
        return;
    }

    let image = match get_image() {
        img if !img.is_null() => img,
        _ => return,
    };

    let class = find_class_by_short_name(image, "SingleModeTrainingCommandService");
    if class.is_null() {
        ura_log(3, "ExecTraining hook: class not found");
        return;
    }

    let method_addr = find_method_addr(class, "ExecTraining", 2);
    if method_addr == 0 {
        ura_log(3, "ExecTraining hook: method not found");
        return;
    }

    EXEC_TRAINING_ADDR = method_addr;
    install_hook_safe(
        "ExecTraining",
        method_addr,
        exec_training_hook as usize,
        &mut ORIG_EXEC_TRAINING_PROLOGUE,
    );
    EXEC_TRAINING_HOOK_INSTALLED = true;
}

/// v3.22.51: 启动时自动检查更新（后台线程）
/// 延迟30秒后执行，避免影响游戏启动
fn start_auto_update_thread() {
    std::thread::spawn(|| {
        std::thread::sleep(std::time::Duration::from_secs(30));
        let result = update_so();
        if let Ok(mut status) = AUTO_UPDATE_STATUS.lock() {
            *status = Some(result);
        }
    });
}

/// v3.22.51: /update — Self-update SO from GitHub Release
/// 1. Find current SO path via /proc/self/maps
/// 2. Query GitHub API for latest release tag
/// 3. Download new SO if version differs
/// 4. Replace file (unlink old + rename new)
/// 5. Returns result; user must restart game to load new SO
fn update_so() -> String {
    // Step 1: Find current SO path
    let so_path = match find_own_so_path() {
        Some(p) => p,
        None => return r#"{"error":"cannot_find_own_path"}"#.to_string(),
    };

    // Step 2: Query GitHub API
    let api_url = "https://api.github.com/repos/xf8410/hlpatch/releases/latest";
    let body = match ureq::get(api_url)
        .set("User-Agent", "URA-Plugin-Updater")
        .call()
    {
        Ok(resp) => match resp.into_string() {
            Ok(s) => s,
            Err(e) => return format!(r#"{{"error":"read_api_body_failed","detail":"{}"}}"#, e),
        },
        Err(e) => return format!(r#"{{"error":"github_api_failed","detail":"{}"}}"#, e),
    };

    // Step 3: Parse tag_name and asset API URL (avoids 302 redirect from browser_download_url)
    let tag_name = extract_json_string(&body, "tag_name").unwrap_or_default();
    let asset_api_url = match extract_so_asset_api_url(&body) {
        Some(u) => u,
        None => return format!(r#"{{"error":"no_so_asset_url","tag":"{}"}}"#, tag_name),
    };

    // Compare versions: use CARGO_PKG_VERSION (auto-synced with Cargo.toml)
    let current_ver = PLUGIN_VERSION;
    if tag_name == format!("v{}", current_ver) {
        return format!(
            r#"{{"status":"already_latest","current":"{}","latest":"{}"}}"#,
            current_ver, tag_name
        );
    }

    // Step 4: Download new SO to temp file (with timeout + retry)
    let tmp_path = format!("{}.tmp", so_path);
    let mut data = Vec::new();
    let mut download_ok = false;
    for attempt in 0..3 {
        if attempt > 0 {
            std::thread::sleep(std::time::Duration::from_secs(2));
        }
        // Use GitHub API asset endpoint with Accept: application/octet-stream
        // This returns binary directly — no 302 redirect to CDN
        let resp_result = ureq::get(&asset_api_url)
            .set("User-Agent", "URA-Plugin-Updater")
            .set("Accept", "application/octet-stream")
            .timeout(std::time::Duration::from_secs(120))
            .call();
        match resp_result {
            Ok(resp) => {
                data.clear();
                match resp.into_reader().read_to_end(&mut data) {
                    Ok(_) => {
                        download_ok = true;
                        break;
                    }
                    Err(e) => {
                        if attempt == 2 {
                            return format!(
                                r#"{{"error":"read_download_failed","detail":"{}","attempts":3}}"#,
                                e
                            );
                        }
                    }
                }
            }
            Err(e) => {
                if attempt == 2 {
                    return format!(
                        r#"{{"error":"download_failed","detail":"{}","attempts":3}}"#,
                        e
                    );
                }
            }
        }
    }
    if !download_ok {
        return r#"{"error":"download_failed_all_attempts"}"#.to_string();
    }

    // Step 5: Try to write directly next to the old SO first
    // If that fails (read-only directory), write to a writable fallback location
    // ★ v3.22.89: Extract fallback write into helper to avoid code duplication
    // When direct write + remove/rename fails, try fallback paths instead of returning error
    let direct_ok = std::fs::write(&tmp_path, &data).is_ok();
    let replaced = if direct_ok {
        if std::fs::remove_file(&so_path).is_ok() && std::fs::rename(&tmp_path, &so_path).is_ok() {
            true
        } else {
            // Tmp written but can't replace original — clean up tmp, try fallback
            let _ = std::fs::remove_file(&tmp_path);
            false
        }
    } else {
        false
    };
    if replaced {
        return format!(
            r#"{{"status":"updated","old":"{}","new":"{}","so_path":"{}","hint":"restart_game_to_apply"}}"#,
            current_ver, tag_name, so_path
        );
    }
    // Fallback: try /data/local/tmp, then /sdcard/Download
    let fb_dir = "/data/local/tmp";
    let fb_path = format!("{}/libhachimi_ura.so", fb_dir);
    let fb_tmp = format!("{}.tmp", fb_path);
    match std::fs::write(&fb_tmp, &data) {
        Ok(_) => {
            let _ = std::fs::remove_file(&fb_path);
            match std::fs::rename(&fb_tmp, &fb_path) {
                Ok(_) => {}
                Err(e) => {
                    return format!(r#"{{"error":"rename_fallback_failed","detail":"{}"}}"#, e)
                }
            }
        }
        Err(_) => {
            // Last resort: /sdcard/Download
            let sd_path = "/sdcard/Download/libhachimi_ura.so";
            match std::fs::write(sd_path, &data) {
                Ok(_) => {
                    return format!(
                        r#"{{"status":"downloaded_to_sdcard","old":"{}","new":"{}","path":"{}","hint":"install_manually_then_restart"}}"#,
                        current_ver, tag_name, sd_path
                    );
                }
                Err(e) => return format!(r#"{{"error":"write_all_failed","detail":"{}"}}"#, e),
            }
        }
    }
    format!(
        r#"{{"status":"updated_to_fallback","old":"{}","new":"{}","so_path":"{}","hint":"restart_game_to_apply"}}"#,
        current_ver, tag_name, fb_path
    )
}

/// Find our own .so file path by scanning /proc/self/maps
fn find_own_so_path() -> Option<String> {
    let maps = std::fs::read_to_string("/proc/self/maps").ok()?;
    for line in maps.lines() {
        let path = line.split_whitespace().last()?;
        if path.contains("libhachimi_") && path.ends_with(".so") {
            return Some(path.to_string());
        }
    }
    None
}

/// Extract a JSON string value by key from a JSON body (simple, no parser)
fn extract_json_string(body: &str, key: &str) -> Option<String> {
    let pattern = format!(r##""{}":"##, key);
    let start = body.find(&pattern)?;
    let val_start = start + pattern.len();
    let val_end = body[val_start..].find('"')?;
    Some(body[val_start..val_start + val_end].to_string())
}

/// Extract the .so asset API URL from GitHub release JSON (not browser_download_url)
/// Using the API URL with Accept: application/octet-stream avoids 302 redirects
fn extract_so_asset_api_url(body: &str) -> Option<String> {
    // Look for "url" entries inside assets that end with .so in "name"
    // GitHub release JSON: "assets": [{ "url": "https://api.github.com/.../assets/123", "name": "libhachimi_ura.so", ... }]
    let mut search_from = 0;
    loop {
        // Find "name":"*.so" first
        let name_pattern = r##""name":""##;
        let name_idx = match body[search_from..].find(name_pattern) {
            Some(i) => i,
            None => break,
        };
        let name_start = search_from + name_idx + name_pattern.len();
        let name_end = match body[name_start..].find('"') {
            Some(i) => i,
            None => break,
        };
        let name = &body[name_start..name_start + name_end];
        if name.ends_with(".so") {
            // Found .so asset — now find its "url" field (appears before "name")
            // Search backwards from name_idx for "url":"
            let block_start = if search_from > 200 {
                search_from - 200
            } else {
                0
            };
            let block = &body[block_start..search_from + name_idx];
            let url_pattern = r##""url":""##;
            if let Some(ui) = block.rfind(url_pattern) {
                let u_start = block_start + ui + url_pattern.len();
                let u_end = match body[u_start..].find('"') {
                    Some(i) => i,
                    None => {
                        search_from = name_start + name_end + 1;
                        continue;
                    }
                };
                return Some(body[u_start..u_start + u_end].to_string());
            }
        }
        search_from = name_start + name_end + 1;
        if search_from >= body.len() {
            break;
        }
    }
    None
}

// ============================================================
// v3.22.89: 新增3个IL2CPP端点
// A: /il2cpp/classes?keyword=X — 搜索类名（关键词过滤）
// B: /il2cpp/static?name=X — 读取静态类常量值（不依赖单例实例）
// D: /il2cpp/search_float?value=X — 在代码段搜索浮点常量
// ============================================================

/// A: /il2cpp/classes?keyword=X — 搜索IL2CPP类名
/// 遍历umamusume.dll所有类，按关键词过滤，返回匹配的类名+命名空间
unsafe fn il2cpp_search_classes(keyword: &str) -> String {
    let image = get_image();
    if image.is_null() {
        return r#"{"error":"image_null"}"#.to_string();
    }

    let get_count_fn = resolve_il2cpp_symbol("il2cpp_image_get_class_count");
    let get_class_fn = resolve_il2cpp_symbol("il2cpp_image_get_class");
    if get_count_fn.is_null() || get_class_fn.is_null() {
        return r#"{"error":"class_enum_api_not_found"}"#.to_string();
    }

    let get_count: FnImageGetClassCount = std::mem::transmute(get_count_fn);
    let get_class: FnImageGetClass = std::mem::transmute(get_class_fn);
    let get_name_fn = resolve_il2cpp_symbol("il2cpp_class_get_name");
    let get_namespace_fn = resolve_il2cpp_symbol("il2cpp_class_get_namespace");

    let total = get_count(image);
    let search_lower = keyword.to_lowercase();
    let mut results = Vec::new();

    for i in 0..total {
        let cls = get_class(image, i);
        if cls.is_null() {
            continue;
        }

        // 读取类名
        let name = if !get_name_fn.is_null() {
            let name_fn: FnClassGetName = std::mem::transmute(get_name_fn);
            let cstr = name_fn(cls);
            if cstr.is_null() {
                continue;
            }
            std::ffi::CStr::from_ptr(cstr)
                .to_string_lossy()
                .into_owned()
        } else {
            continue;
        };

        // 读取命名空间
        let namespace = if !get_namespace_fn.is_null() {
            let ns_fn: FnClassGetName = std::mem::transmute(get_namespace_fn);
            let cstr = ns_fn(cls);
            if cstr.is_null() {
                String::new()
            } else {
                std::ffi::CStr::from_ptr(cstr)
                    .to_string_lossy()
                    .into_owned()
            }
        } else {
            String::new()
        };

        // 关键词过滤（空关键词返回所有）
        if !keyword.is_empty() {
            let full = format!("{}.{}", namespace, name).to_lowercase();
            if !full.contains(&search_lower) {
                continue;
            }
        }

        // 获取该类的字段数量，帮助判断类大小
        let fields = collect_all_fields(cls);
        let field_count = fields.len();
        // 检查是否有单例
        let has_singleton = !get_singleton(cls).is_null();

        results.push(format!(
            r#"{{"ns":"{}","name":"{}","fields":{},"singleton":{}}}"#,
            namespace, name, field_count, has_singleton
        ));
    }

    format!(
        r#"{{"ok":true,"total_classes":{},"matched":{},"keyword":"{}","classes":[{}]}}"#,
        total,
        results.len(),
        json_escape(keyword),
        results.join(",")
    )
}

/// B: /il2cpp/static?name=X — 读取静态类常量值
/// 不需要单例实例，直接用il2cpp_get_static_field_value读取静态字段
/// 适用于SingleModeDefine等只有const/literal字段的静态类
unsafe fn il2cpp_read_static_fields(class_name: &str) -> String {
    if class_name.is_empty() {
        return r#"{"error":"missing ?name= parameter"}"#.to_string();
    }
    if API.is_null() {
        return r#"{"error":"api_null"}"#.to_string();
    }
    let image = match get_image() {
        img if !img.is_null() => img,
        _ => return r#"{"error":"image_null"}"#.to_string(),
    };

    let class = find_class_by_short_name(image, class_name);
    if class.is_null() {
        return format!(r#"{{"error":"class_not_found","name":"{}"}}"#, class_name);
    }

    let real_name = get_class_name_from_pointer(class);

    // 遍历所有字段（含父类）
    let fields = collect_all_fields(class);
    let mut results = Vec::new();

    // ★ v3.22.89: 解析il2cpp_field_get_flags，用于判断literal（const）字段
    // literal字段没有运行时静态存储，il2cpp_get_static_field_value会闪退
    let field_get_flags: Option<unsafe extern "C" fn(*const c_void) -> u32> = {
        let p = resolve_il2cpp_symbol("il2cpp_field_get_flags");
        if p.is_null() {
            None
        } else {
            Some(std::mem::transmute(p))
        }
    };

    // ★ v3.22.89: il2cpp_field_get_default_value API不存在，enum值改为C#规范推算

    // ★ v3.22.89: 检查类是否是enum — enum类的字段没有运行时静态存储
    // 对enum类调get_static_field_value会闪退，需要特殊处理
    let is_enum_class: bool = {
        let is_enum_fn = resolve_il2cpp_symbol("il2cpp_class_is_enum");
        if !is_enum_fn.is_null() {
            let f: unsafe extern "C" fn(*mut c_void) -> bool = std::mem::transmute(is_enum_fn);
            f(class)
        } else {
            false
        }
    };

    if is_enum_class {
        // ★ v3.22.89: enum值双重策略
        // 策略1: C#规范推算（字段声明顺序从0递增）
        // 策略2: 用il2cpp_runtime_invoke调Enum.GetValues做交叉验证
        let internal_names = ["value__", "enumSeperatorCharArray", "enumSeperator"];

        // ★ 策略2: 尝试用runtime_invoke调Enum.GetValues获取真实值
        // 需要找到System.Enum类和GetValues方法
        let mut runtime_values: Vec<(&str, i32)> = Vec::new(); // (name, value)
        let get_method_fn: Option<FnClassGetMethodFromName> = {
            let p = resolve_il2cpp_symbol("il2cpp_class_get_method_from_name");
            if p.is_null() {
                None
            } else {
                Some(std::mem::transmute(p))
            }
        };
        let invoke_fn: Option<FnRuntimeInvoke> = {
            let p = resolve_il2cpp_symbol("il2cpp_runtime_invoke");
            if p.is_null() {
                None
            } else {
                Some(std::mem::transmute(p))
            }
        };
        // 找System.Enum类
        let enum_class = find_class_by_short_name(image, "Enum");
        if !enum_class.is_null() && get_method_fn.is_some() && invoke_fn.is_some() {
            // GetValues(Type enumType) -> Array
            // 参数个数=1，需要传Type对象
            let get_values_method =
                get_method_fn.unwrap()(enum_class, to_cstr("GetValues").as_ptr(), 1);
            if !get_values_method.is_null() {
                // 需要构造Type参数：il2cpp_class_get_type返回Il2CppType*
                // 然后il2cpp_type_get_object把它转成System.Type的运行时对象
                // 但il2cpp_type_get_object可能不存在，先试另一种方式：
                // 用typeof(GainParameterType) = class的type handle
                // 直接传class指针作为type参数不行，需要Type对象
                // 尝试找il2cpp_type_get_object
                let type_get_object: Option<unsafe extern "C" fn(*const c_void) -> *mut c_void> = {
                    let p = resolve_il2cpp_symbol("il2cpp_type_get_object");
                    if p.is_null() {
                        None
                    } else {
                        Some(std::mem::transmute(p))
                    }
                };
                // 也尝试从class获取type: il2cpp_class_get_type
                let class_get_type: Option<unsafe extern "C" fn(*mut c_void) -> *const c_void> = {
                    let p = resolve_il2cpp_symbol("il2cpp_class_get_type");
                    if p.is_null() {
                        None
                    } else {
                        Some(std::mem::transmute(p))
                    }
                };
                if type_get_object.is_some() && class_get_type.is_some() {
                    let il2cpp_type = class_get_type.unwrap()(class);
                    if !il2cpp_type.is_null() {
                        let type_obj = type_get_object.unwrap()(il2cpp_type);
                        if !type_obj.is_null() {
                            // 调用Enum.GetValues(type_obj)
                            let mut args: [*mut c_void; 1] = [type_obj as *mut c_void];
                            let mut exc: *mut c_void = ptr::null_mut();
                            let result = invoke_fn.unwrap()(
                                get_values_method,
                                ptr::null_mut(), // 静态方法，instance=null
                                args.as_mut_ptr(),
                                &mut exc,
                            );
                            if exc.is_null() && !result.is_null() {
                                // result是Il2CppArray*，读取元素
                                // Il2CppArray: header 32bytes, then max_length(u32), then data
                                // 实际布局: Il2CppObject(16) + BoundsInfo*(8) + max_length(8) + data
                                let arr_ptr = result as *const u8;
                                let arr_len =
                                    std::ptr::read_unaligned::<i64>(arr_ptr.add(24) as *const i64);
                                let data_start = arr_ptr.add(32);
                                // 收集命名常量的字段名
                                let named_fields: Vec<&str> = fields
                                    .iter()
                                    .filter(|(n, _, _)| !internal_names.contains(&n.as_str()))
                                    .map(|(n, _, _)| n.as_str())
                                    .collect();
                                for (i, &fname) in named_fields.iter().enumerate() {
                                    if (i as i64) < arr_len {
                                        let v = std::ptr::read_unaligned::<i32>(
                                            data_start.add(i * 4) as *const i32,
                                        );
                                        runtime_values.push((fname, v));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        let has_runtime_values = !runtime_values.is_empty();

        // 生成结果：如果runtime_values有数据则用真实值，否则用C#规范推算
        let mut enum_idx: i32 = 0;
        for (fname, offset, type_ptr) in &fields {
            let type_enum = il2cpp_type_get_type_enum(*type_ptr);
            if internal_names.contains(&fname.as_str()) {
                results.push(format!(
                    r#"{{"name":"{}","offset":{},"type":{},"value":null,"enum":true,"internal":true}}"#,
                    json_escape(fname), offset, type_enum
                ));
                continue;
            }
            if has_runtime_values {
                // 用runtime_invoke获取的真实值
                let val = runtime_values
                    .iter()
                    .find(|(n, _)| *n == fname.as_str())
                    .map(|(_, v)| *v)
                    .unwrap_or(enum_idx);
                results.push(format!(
                    r#"{{"name":"{}","offset":{},"type":{},"value":{},"enum":true}}"#,
                    json_escape(fname),
                    offset,
                    type_enum,
                    val
                ));
            } else {
                // 回退：C#规范推算，标注inferred
                results.push(format!(
                    r#"{{"name":"{}","offset":{},"type":{},"value":{},"enum":true,"inferred":true}}"#,
                    json_escape(fname), offset, type_enum, enum_idx
                ));
            }
            enum_idx += 1;
        }
        return format!(
            r#"{{"ok":true,"requested":"{}","found":"{}","field_count":{},"is_enum":true,"values_source":"{}","fields":[{}]}}"#,
            class_name,
            real_name,
            results.len(),
            if has_runtime_values {
                "runtime_invoke"
            } else {
                "csharp_spec_inference"
            },
            results.join(",")
        );
    }

    for (fname, offset, type_ptr) in &fields {
        let type_enum = il2cpp_type_get_type_enum(*type_ptr);

        // ★ v3.22.89: 检查字段是否是static，非static字段没有静态存储，读值会闪退
        // FIELD_ATTRIBUTE_STATIC = 0x0010
        let is_static_field = {
            let field_info_check = match (*API).il2cpp_get_field_from_name_fn {
                Some(f) => f(class, to_cstr(fname).as_ptr()),
                None => ptr::null_mut(),
            };
            if !field_info_check.is_null() {
                if let Some(get_flags) = field_get_flags {
                    (get_flags(field_info_check) & 0x0010) != 0
                } else {
                    // 无法获取flags，默认认为是static（保守策略）
                    true
                }
            } else {
                // 无法获取field_info，默认认为是static
                true
            }
        };

        if !is_static_field {
            // 非static字段：只返回定义，不读值，避免闪退
            results.push(format!(
                r#"{{"name":"{}","offset":{},"type":{},"value":null,"instance_field":true}}"#,
                json_escape(fname),
                offset,
                type_enum
            ));
            continue;
        }

        // 先尝试通过field name获取FieldInfo
        let field_info = match (*API).il2cpp_get_field_from_name_fn {
            Some(f) => f(class, to_cstr(fname).as_ptr()),
            None => ptr::null_mut(),
        };

        if field_info.is_null() {
            results.push(format!(
                r#"{{"name":"{}","offset":{},"type":{},"value":null,"error":"no_field_info"}}"#,
                json_escape(fname),
                offset,
                type_enum
            ));
            continue;
        }

        // ★ v3.22.89: 检查是否是literal（const）字段
        // FIELD_ATTRIBUTE_LITERAL = 0x0040
        // 注意：literal字段的I4/U4/I8/U8/BOOLEAN/R4类型仍可用get_static_field_value正常读取
        // 只有R8类型有4字节bug需要workaround
        let is_literal = match field_get_flags {
            Some(f) => (f(field_info) & 0x0040) != 0,
            None => false,
        };

        // 所有字段（literal和非literal）统一用get_static_field_value读取
        // R8类型用8字节buffer workaround处理4字节bug
        match (*API).il2cpp_get_static_field_value_fn {
            Some(get_static) => {
                match type_enum {
                    IL2CPP_TYPE_R4 => {
                        let mut val: f32 = 0.0;
                        get_static(field_info, &mut val as *mut f32 as *mut c_void);
                        results.push(format!(
                            r#"{{"name":"{}","offset":{},"type":{},"value":{}}}"#,
                            json_escape(fname),
                            offset,
                            type_enum,
                            val
                        ));
                    }
                    IL2CPP_TYPE_R8 => {
                        // ★ v3.22.89: 修复f64读取bug
                        // il2cpp_get_static_field_value对f64只写4字节（低4字节=正确的f32值，高4字节=0）
                        // 读8字节buffer，检测高4字节是否为0来判定bug
                        let mut buf: [u8; 8] = [0; 8];
                        get_static(field_info, buf.as_mut_ptr() as *mut c_void);
                        let low4 = u32::from_ne_bytes([buf[0], buf[1], buf[2], buf[3]]);
                        let high4 = u32::from_ne_bytes([buf[4], buf[5], buf[6], buf[7]]);
                        if high4 == 0 && low4 != 0 {
                            // 高4字节为0且低4字节非0：只写了4字节的bug，从低4字节取f32
                            let f32_val = f32::from_ne_bytes(low4.to_ne_bytes());
                            results.push(format!(
                                r#"{{"name":"{}","offset":{},"type":{},"value":{},"note":"f64_as_f32"}}"#,
                                json_escape(fname), offset, type_enum, f32_val
                            ));
                        } else {
                            // 正常8字节f64
                            let f64_val = f64::from_ne_bytes(buf);
                            results.push(format!(
                                r#"{{"name":"{}","offset":{},"type":{},"value":{}}}"#,
                                json_escape(fname),
                                offset,
                                type_enum,
                                f64_val
                            ));
                        }
                    }
                    IL2CPP_TYPE_I4 | IL2CPP_TYPE_VALUETYPE => {
                        let mut val: i32 = 0;
                        get_static(field_info, &mut val as *mut i32 as *mut c_void);
                        results.push(format!(
                            r#"{{"name":"{}","offset":{},"type":{},"value":{}}}"#,
                            json_escape(fname),
                            offset,
                            type_enum,
                            val
                        ));
                    }
                    IL2CPP_TYPE_U4 => {
                        let mut val: u32 = 0;
                        get_static(field_info, &mut val as *mut u32 as *mut c_void);
                        results.push(format!(
                            r#"{{"name":"{}","offset":{},"type":{},"value":{}}}"#,
                            json_escape(fname),
                            offset,
                            type_enum,
                            val
                        ));
                    }
                    IL2CPP_TYPE_I8 => {
                        let mut val: i64 = 0;
                        get_static(field_info, &mut val as *mut i64 as *mut c_void);
                        results.push(format!(
                            r#"{{"name":"{}","offset":{},"type":{},"value":{}}}"#,
                            json_escape(fname),
                            offset,
                            type_enum,
                            val
                        ));
                    }
                    IL2CPP_TYPE_U8 => {
                        let mut val: u64 = 0;
                        get_static(field_info, &mut val as *mut u64 as *mut c_void);
                        results.push(format!(
                            r#"{{"name":"{}","offset":{},"type":{},"value":{}}}"#,
                            json_escape(fname),
                            offset,
                            type_enum,
                            val
                        ));
                    }
                    IL2CPP_TYPE_BOOLEAN => {
                        let mut val: u8 = 0;
                        get_static(field_info, &mut val as *mut u8 as *mut c_void);
                        results.push(format!(
                            r#"{{"name":"{}","offset":{},"type":{},"value":{}}}"#,
                            json_escape(fname),
                            offset,
                            type_enum,
                            if val != 0 { "true" } else { "false" }
                        ));
                    }
                    IL2CPP_TYPE_STRING => {
                        let mut val: *const c_void = ptr::null();
                        get_static(field_info, &mut val as *mut *const c_void as *mut c_void);
                        if val.is_null() {
                            results.push(format!(
                                r#"{{"name":"{}","offset":{},"type":{},"value":null}}"#,
                                json_escape(fname),
                                offset,
                                type_enum
                            ));
                        } else {
                            let s = read_il2cpp_string(val);
                            results.push(format!(
                                r#"{{"name":"{}","offset":{},"type":{},"value":"{}"}}"#,
                                json_escape(fname),
                                offset,
                                type_enum,
                                json_escape(&s)
                            ));
                        }
                    }
                    _ => {
                        // 其他类型：尝试读8字节原始数据
                        let mut buf: [u8; 8] = [0; 8];
                        get_static(field_info, buf.as_mut_ptr() as *mut c_void);
                        let raw_i64 = i64::from_ne_bytes(buf);
                        results.push(format!(
                            r#"{{"name":"{}","offset":{},"type":{},"raw":{}}}"#,
                            json_escape(fname),
                            offset,
                            type_enum,
                            raw_i64
                        ));
                    }
                }
            }
            None => {
                results.push(format!(
                    r#"{{"name":"{}","offset":{},"type":{},"value":null,"error":"no_static_field_fn"}}"#,
                    json_escape(fname), offset, type_enum
                ));
            }
        }
    }

    format!(
        r#"{{"ok":true,"requested":"{}","found":"{}","field_count":{},"fields":[{}]}}"#,
        class_name,
        real_name,
        results.len(),
        results.join(",")
    )
}

/// D: /il2cpp/search_float?value=X — 在代码段搜索浮点常量
/// 扫描umamusume.dll的代码段(.text)，搜索指定浮点值的IEEE 754编码
/// 用于定位硬编码的训练系数等常量
/// E: /il2cpp/methods?name=X — 列出类的所有方法名和参数数量
/// 用于找到计算方法（如やる気系数计算等纯逻辑类的方法）
unsafe fn il2cpp_list_methods(class_name: &str) -> String {
    if class_name.is_empty() {
        return r#"{"error":"missing ?name= parameter"}"#.to_string();
    }
    if API.is_null() {
        return r#"{"error":"api_null"}"#.to_string();
    }
    let image = match get_image() {
        img if !img.is_null() => img,
        _ => return r#"{"error":"image_null"}"#.to_string(),
    };

    let class = find_class_by_short_name(image, class_name);
    if class.is_null() {
        return format!(r#"{{"error":"class_not_found","name":"{}"}}"#, class_name);
    }

    let real_name = get_class_name_from_pointer(class);

    // il2cpp_class_get_methods 遍历所有方法
    // 原型: const MethodInfo* il2cpp_class_get_methods(Il2CppClass* klass, void** iter)
    let get_methods_fn: Option<
        unsafe extern "C" fn(*mut c_void, *mut *mut c_void) -> *const c_void,
    > = {
        let p = resolve_il2cpp_symbol("il2cpp_class_get_methods");
        if p.is_null() {
            None
        } else {
            Some(std::mem::transmute(p))
        }
    };

    // il2cpp_method_get_name 获取方法名
    // 原型: const char* il2cpp_method_get_name(const MethodInfo* method)
    let method_get_name_fn: Option<unsafe extern "C" fn(*const c_void) -> *const c_char> = {
        let p = resolve_il2cpp_symbol("il2cpp_method_get_name");
        if p.is_null() {
            None
        } else {
            Some(std::mem::transmute(p))
        }
    };

    // il2cpp_method_get_param_count 获取参数数量
    // 原型: uint32_t il2cpp_method_get_param_count(const MethodInfo* method)
    let method_get_param_count_fn: Option<unsafe extern "C" fn(*const c_void) -> u32> = {
        let p = resolve_il2cpp_symbol("il2cpp_method_get_param_count");
        if p.is_null() {
            None
        } else {
            Some(std::mem::transmute(p))
        }
    };

    // il2cpp_method_get_return_type 获取返回类型
    let method_get_return_type_fn: Option<unsafe extern "C" fn(*const c_void) -> *const c_void> = {
        let p = resolve_il2cpp_symbol("il2cpp_method_get_return_type");
        if p.is_null() {
            None
        } else {
            Some(std::mem::transmute(p))
        }
    };

    let type_get_name_fn: Option<unsafe extern "C" fn(*const c_void) -> *const c_char> = {
        let p = resolve_il2cpp_symbol("il2cpp_type_get_name");
        if p.is_null() { None } else { Some(std::mem::transmute(p)) }
    };

    // il2cpp_method_get_flags 获取方法标志
    // 原型: uint32_t il2cpp_method_get_flags(const MethodInfo* method, uint32_t* iflags)
    let method_get_flags_fn: Option<unsafe extern "C" fn(*const c_void, *mut u32) -> u32> = {
        let p = resolve_il2cpp_symbol("il2cpp_method_get_flags");
        if p.is_null() {
            None
        } else {
            Some(std::mem::transmute(p))
        }
    };

    // il2cpp_method_get_class 获取方法所属类
    let method_get_class_fn: Option<unsafe extern "C" fn(*const c_void) -> *mut c_void> = {
        let p = resolve_il2cpp_symbol("il2cpp_method_get_class");
        if p.is_null() {
            None
        } else {
            Some(std::mem::transmute(p))
        }
    };

    if get_methods_fn.is_none() || method_get_name_fn.is_none() {
        return format!(
            r#"{{"ok":true,"requested":"{}","found":"{}","method_count":0,"methods":[],"error":"missing_il2cpp_api"}}"#,
            class_name, real_name
        );
    }

    let mut methods = Vec::new();
    let mut iter: *mut c_void = ptr::null_mut();
    loop {
        let method_info = get_methods_fn.unwrap()(class, &mut iter);
        if method_info.is_null() {
            break;
        }

        let method_name = {
            let name_ptr = method_get_name_fn.unwrap()(method_info);
            if name_ptr.is_null() {
                "(null)".to_string()
            } else {
                let cstr = CStr::from_ptr(name_ptr);
                cstr.to_string_lossy().into_owned()
            }
        };

        // 跳过.ctor和.cctor（构造函数）
        if method_name.starts_with('.') {
            continue;
        }

        let param_count = method_get_param_count_fn
            .map(|f| f(method_info))
            .unwrap_or(0);

        // 判断是否是静态方法: METHOD_ATTRIBUTE_STATIC = 0x0010
        let is_static = method_get_flags_fn
            .map(|f| {
                let mut iflags: u32 = 0;
                let flags = f(method_info, &mut iflags);
                (flags & 0x0010) != 0
            })
            .unwrap_or(false);

        // Preserve the coarse enum for compatibility and add the exact IL2CPP type name.
        let return_type_ptr = method_get_return_type_fn
            .map(|f| f(method_info))
            .unwrap_or(ptr::null());
        let return_type_str = if return_type_ptr.is_null() {
            "void".to_string()
        } else {
            type_enum_to_name(il2cpp_type_get_type_enum(return_type_ptr))
        };
        let return_type_name = if return_type_ptr.is_null() {
            "void".to_string()
        } else {
            type_get_name_fn
                .and_then(|f| {
                    let p = f(return_type_ptr);
                    if p.is_null() { None } else { Some(CStr::from_ptr(p).to_string_lossy().into_owned()) }
                })
                .unwrap_or_else(|| return_type_str.clone())
        };

        // 检查是否是本类定义的方法（不是继承的）
        let is_own_method = method_get_class_fn
            .map(|f| {
                let declaring_class = f(method_info);
                declaring_class == class
            })
            .unwrap_or(true);

        methods.push(format!(
            r#"{{"name":"{}","params":{},"return_type":"{}","return_type_name":"{}","static":{},"own":{}}}"#,
            json_escape(&method_name),
            param_count,
            return_type_str,
            json_escape(&return_type_name),
            is_static,
            is_own_method
        ));

    }
    format!(
        r#"{{"ok":true,"requested":"{}","found":"{}","method_count":{},"methods":[{}]}}"#,
        class_name,
        real_name,
        methods.len(),
        methods.join(",")
    )
}

/// 辅助函数：IL2CPP类型枚举转可读名称
fn type_enum_to_name(te: u8) -> String {
    match te {
        1 => "void".to_string(),
        2 => "boolean".to_string(),
        3 => "char".to_string(),
        4 => "i1".to_string(),
        5 => "u1".to_string(),
        6 => "i2".to_string(),
        7 => "u2".to_string(),
        8 => "i4".to_string(),
        9 => "u4".to_string(),
        10 => "i8".to_string(),
        11 => "u8".to_string(),
        12 => "r4".to_string(),
        13 => "r8".to_string(),
        14 => "string".to_string(),
        17 => "ptr".to_string(),
        18 => "byref".to_string(),
        21 => "valuetype".to_string(),
        22 => "class".to_string(),
        24 => "array".to_string(),
        25 => "genericinst".to_string(),
        28 => "cmplx".to_string(),
        29 => "fnptr".to_string(),
        30 => "object".to_string(),
        _ => format!("type_{}", te),
    }
}

/// v3.22.89: /il2cpp/search_methods_page — 搜索方法名HTML页面（A-Z分组下载）
fn search_methods_page() -> String {
    let letters = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let mut btns = String::new();
    for ch in letters.chars() {
        btns.push_str(&format!(
            r#"<button class="g" onclick="goLetter('{}')">{}</button> "#,
            ch, ch
        ));
    }
    format!(
        r#"<!DOCTYPE html><html><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Search Methods</title><style>body{{font-family:system-ui;max-width:600px;margin:12px auto;padding:0 8px;background:#1a1a2e;color:#e0e0e0}}h1{{color:#4fc3f7;font-size:1.2em;margin:8px 0}}.g{{display:inline-block;margin:4px 2px;padding:8px 12px;background:#16213e;border:1px solid #333;border-radius:4px;color:#fff;cursor:pointer;font-size:14px;min-width:36px;text-align:center}}.g:disabled{{background:#555;color:#333;cursor:default}}.g.ok{{background:#2e7d32;border-color:#4caf50}}.g.run{{background:#e65100;border-color:#ff9800}}input{{width:100%;padding:8px;background:#16213e;border:1px solid #333;border-radius:4px;color:#fff;box-sizing:border-box;font-size:16px}}.p{{margin:8px 0;font-size:0.95em}}.ok{{color:#4caf50}}.err{{color:#ff5252}}#lst{{margin:8px 0;font-size:0.8em;color:#aaa;max-height:300px;overflow-y:auto}}</style></head><body><h1>IL2CPP Method Search</h1><input id="kw" placeholder="keyword (e.g. Motivation)" value="Motivation"><div style="margin:8px 0">{}</div><div class="p">Click a letter to search classes starting with that letter, or click ALL for all classes. Results download as JSON.</div><div class="p" id="pg">Ready</div><div id="lst"></div><script>function goLetter(ch){{var kw=document.getElementById("kw").value;if(!kw){{document.getElementById("pg").innerHTML='<span class="err">Enter a keyword first</span>';return;}}var btn=event.target;btn.disabled=true;btn.className="g run";var url="/il2cpp/search_methods_dl?keyword="+encodeURIComponent(kw)+"&letter="+ch;document.getElementById("pg").innerHTML='<span class="ok">Searching '+ch+'...</span>';fetch(url).then(r=>{{if(!r.ok)throw new Error("HTTP "+r.status);return r.blob();}}).then(blob=>{{var url2=URL.createObjectURL(blob);var a=document.createElement("a");a.href=url2;a.download="search_methods_"+ch+"_"+kw+".json";a.click();URL.revokeObjectURL(url2);btn.className="g ok";btn.disabled=false;document.getElementById("pg").innerHTML='<span class="ok">'+ch+': downloaded!</span>';}}).catch(e=>{{btn.className="g ok";btn.disabled=false;document.getElementById("pg").innerHTML='<span class="err">Error: '+e+'</span>';}});}}</script></body></html>"#,
        btns
    )
}

/// v3.22.89: /il2cpp/search_methods?keyword=X — 跨类搜索方法名
/// 遍历所有IL2CPP类的方法表，按方法名关键词过滤，返回匹配的类名+方法名
/// 用于定位やる気系数等散落在各类中的计算方法
unsafe fn il2cpp_search_methods(keyword: &str, letter: &str) -> String {
    if keyword.is_empty() {
        return r#"{"error":"missing ?keyword= parameter"}"#.to_string();
    }
    let image = get_image();
    if image.is_null() {
        return r#"{"error":"image_null"}"#.to_string();
    }

    // 解析IL2CPP API函数指针
    let get_count_fn = resolve_il2cpp_symbol("il2cpp_image_get_class_count");
    let get_class_fn = resolve_il2cpp_symbol("il2cpp_image_get_class");
    if get_count_fn.is_null() || get_class_fn.is_null() {
        return r#"{"error":"class_enum_api_not_found"}"#.to_string();
    }
    let get_count: FnImageGetClassCount = std::mem::transmute(get_count_fn);
    let get_class: FnImageGetClass = std::mem::transmute(get_class_fn);

    // 方法遍历API
    let get_methods_fn: Option<
        unsafe extern "C" fn(*mut c_void, *mut *mut c_void) -> *const c_void,
    > = {
        let p = resolve_il2cpp_symbol("il2cpp_class_get_methods");
        if p.is_null() {
            None
        } else {
            Some(std::mem::transmute(p))
        }
    };
    let method_get_name_fn: Option<unsafe extern "C" fn(*const c_void) -> *const c_char> = {
        let p = resolve_il2cpp_symbol("il2cpp_method_get_name");
        if p.is_null() {
            None
        } else {
            Some(std::mem::transmute(p))
        }
    };
    let method_get_param_count_fn: Option<unsafe extern "C" fn(*const c_void) -> u32> = {
        let p = resolve_il2cpp_symbol("il2cpp_method_get_param_count");
        if p.is_null() {
            None
        } else {
            Some(std::mem::transmute(p))
        }
    };
    let method_get_return_type_fn: Option<unsafe extern "C" fn(*const c_void) -> *const c_void> = {
        let p = resolve_il2cpp_symbol("il2cpp_method_get_return_type");
        if p.is_null() {
            None
        } else {
            Some(std::mem::transmute(p))
        }
    };
    let method_get_flags_fn: Option<unsafe extern "C" fn(*const c_void, *mut u32) -> u32> = {
        let p = resolve_il2cpp_symbol("il2cpp_method_get_flags");
        if p.is_null() {
            None
        } else {
            Some(std::mem::transmute(p))
        }
    };

    if get_methods_fn.is_none() || method_get_name_fn.is_none() {
        return r#"{"error":"method_enum_api_not_found"}"#.to_string();
    }

    // 类名/命名空间读取
    let get_name_fn = resolve_il2cpp_symbol("il2cpp_class_get_name");
    let get_namespace_fn = resolve_il2cpp_symbol("il2cpp_class_get_namespace");

    let total = get_count(image);
    let search_lower = keyword.to_lowercase();
    let mut matched_methods: Vec<String> = Vec::new();
    let mut classes_with_hits: i32 = 0;
    let max_results = 200; // 限制最多200个方法结果

    for i in 0..total {
        let cls = get_class(image, i);
        if cls.is_null() {
            continue;
        }

        // 读取类名
        let class_name = if !get_name_fn.is_null() {
            let name_fn: FnClassGetName = std::mem::transmute(get_name_fn);
            let cstr = name_fn(cls);
            if cstr.is_null() {
                continue;
            }
            std::ffi::CStr::from_ptr(cstr)
                .to_string_lossy()
                .into_owned()
        } else {
            continue;
        };

        // 读取命名空间
        let namespace = if !get_namespace_fn.is_null() {
            let ns_fn: FnClassGetName = std::mem::transmute(get_namespace_fn);
            let cstr = ns_fn(cls);
            if cstr.is_null() {
                String::new()
            } else {
                std::ffi::CStr::from_ptr(cstr)
                    .to_string_lossy()
                    .into_owned()
            }
        } else {
            String::new()
        };

        // 按类名首字母过滤（letter参数，A-Z分组下载对策）
        if !letter.is_empty() {
            let first = class_name
                .chars()
                .next()
                .unwrap_or('_')
                .to_ascii_uppercase();
            let target = letter.chars().next().unwrap_or('_').to_ascii_uppercase();
            if first != target {
                continue;
            }
        }

        // 遍历该类的所有方法
        let mut iter: *mut c_void = ptr::null_mut();
        let mut class_hit = false;
        loop {
            let method_info = get_methods_fn.unwrap()(cls, &mut iter);
            if method_info.is_null() {
                break;
            }

            let method_name = {
                let name_ptr = method_get_name_fn.unwrap()(method_info);
                if name_ptr.is_null() {
                    "(null)".to_string()
                } else {
                    let cstr = CStr::from_ptr(name_ptr);
                    cstr.to_string_lossy().into_owned()
                }
            };

            // 跳过构造函数
            if method_name.starts_with('.') {
                continue;
            }

            // 关键词过滤（不区分大小写）
            if !method_name.to_lowercase().contains(&search_lower) {
                continue;
            }

            // 补充信息：参数数、返回类型、是否静态
            let param_count = method_get_param_count_fn
                .map(|f| f(method_info))
                .unwrap_or(0);
            let return_type_str = method_get_return_type_fn
                .map(|f| {
                    let rt = f(method_info);
                    if rt.is_null() {
                        "void".to_string()
                    } else {
                        let te = il2cpp_type_get_type_enum(rt);
                        type_enum_to_name(te)
                    }
                })
                .unwrap_or_else(|| "?".to_string());
            let is_static = method_get_flags_fn
                .map(|f| {
                    let mut iflags: u32 = 0;
                    let flags = f(method_info, &mut iflags);
                    (flags & 0x0010) != 0
                })
                .unwrap_or(false);

            matched_methods.push(format!(
                r#"{{"class":"{}","ns":"{}","method":"{}","params":{},"return_type":"{}","static":{}}}"#,
                class_name, namespace, json_escape(&method_name),
                param_count, return_type_str, is_static
            ));
            class_hit = true;

            if matched_methods.len() >= max_results {
                break;
            }
        }
        if class_hit {
            classes_with_hits += 1;
        }
        if matched_methods.len() >= max_results {
            break;
        }
    }

    format!(
        r#"{{"ok":true,"total_classes":{},"keyword":"{}","classes_with_hits":{},"method_hits":{},"methods":[{}]}}"#,
        total,
        json_escape(keyword),
        classes_with_hits,
        matched_methods.len(),
        matched_methods.join(",")
    )
}

unsafe fn il2cpp_search_float(value_str: &str) -> String {
    let value: f64 = match value_str.parse::<f64>() {
        Ok(v) => v,
        Err(_) => {
            return format!(
                r#"{{"error":"invalid_float","input":"{}"}}"#,
                json_escape(value_str)
            )
        }
    };

    // 获取umamusume.dll基址和大小
    // 通过il2cpp_get_assembly_image获取的image指针，再通过dlsym找到模块基址
    // 更可靠的方式：扫描/proc/self/maps找到umamusume.dll
    let mut base_addr: usize = 0;
    let mut text_size: usize = 0;

    // 读取/proc/self/maps找到umamusume.dll的内存映射
    if let Ok(maps) = std::fs::read_to_string("/proc/self/maps") {
        for line in maps.lines() {
            if line.contains("umamusume") && line.contains("r-xp") {
                // 格式: start-end r-xp ... path
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.is_empty() {
                    continue;
                }
                let addr_parts: Vec<&str> = parts[0].split('-').collect();
                if addr_parts.len() != 2 {
                    continue;
                }
                if let Ok(start) = usize::from_str_radix(addr_parts[0], 16) {
                    if let Ok(end) = usize::from_str_radix(addr_parts[1], 16) {
                        // 取第一个可执行段作为代码段
                        if base_addr == 0 {
                            base_addr = start;
                            text_size = end - start;
                        }
                    }
                }
            }
        }
    }

    if base_addr == 0 || text_size == 0 {
        return r#"{"error":"cannot_find_umamusume_text_section"}"#.to_string();
    }

    // 限制搜索范围，避免越界（最大32MB代码段）
    if text_size > 32 * 1024 * 1024 {
        text_size = 32 * 1024 * 1024;
    }

    let base_ptr = base_addr as *const u8;
    let f32_bytes = value as f32; // 同时搜索f32和f64编码
    let f32_bits = f32_bytes.to_bits().to_le_bytes();
    let f64_bits = value.to_bits().to_le_bytes();

    let mut f32_matches = Vec::new();
    let mut f64_matches = Vec::new();

    // 搜索f32（4字节对齐）
    for off in (0..text_size.saturating_sub(4)).step_by(4) {
        let ptr = base_ptr.add(off);
        // 安全检查
        if (ptr as usize) < base_addr || (ptr as usize) >= base_addr + text_size {
            continue;
        }
        let b0 = std::ptr::read_unaligned::<u8>(ptr);
        let b1 = std::ptr::read_unaligned::<u8>(ptr.add(1));
        let b2 = std::ptr::read_unaligned::<u8>(ptr.add(2));
        let b3 = std::ptr::read_unaligned::<u8>(ptr.add(3));
        if b0 == f32_bits[0] && b1 == f32_bits[1] && b2 == f32_bits[2] && b3 == f32_bits[3] {
            f32_matches.push(off);
        }
    }

    // 搜索f64（8字节对齐）
    for off in (0..text_size.saturating_sub(8)).step_by(4) {
        let ptr = base_ptr.add(off);
        if (ptr as usize) < base_addr || (ptr as usize) >= base_addr + text_size {
            continue;
        }
        let b0 = std::ptr::read_unaligned::<u8>(ptr);
        let b1 = std::ptr::read_unaligned::<u8>(ptr.add(1));
        let b2 = std::ptr::read_unaligned::<u8>(ptr.add(2));
        let b3 = std::ptr::read_unaligned::<u8>(ptr.add(3));
        let b4 = std::ptr::read_unaligned::<u8>(ptr.add(4));
        let b5 = std::ptr::read_unaligned::<u8>(ptr.add(5));
        let b6 = std::ptr::read_unaligned::<u8>(ptr.add(6));
        let b7 = std::ptr::read_unaligned::<u8>(ptr.add(7));
        if b0 == f64_bits[0]
            && b1 == f64_bits[1]
            && b2 == f64_bits[2]
            && b3 == f64_bits[3]
            && b4 == f64_bits[4]
            && b5 == f64_bits[5]
            && b6 == f64_bits[6]
            && b7 == f64_bits[7]
        {
            f64_matches.push(off);
        }
    }

    // 限制输出数量
    let max_results = 50;
    f32_matches.truncate(max_results);
    f64_matches.truncate(max_results);

    let f32_json: Vec<String> = f32_matches
        .iter()
        .map(|&off| {
            format!(
                r#"{{"offset":"0x{:x}","abs_addr":"0x{:x}"}}"#,
                off,
                base_addr + off
            )
        })
        .collect();
    let f64_json: Vec<String> = f64_matches
        .iter()
        .map(|&off| {
            format!(
                r#"{{"offset":"0x{:x}","abs_addr":"0x{:x}"}}"#,
                off,
                base_addr + off
            )
        })
        .collect();

    format!(
        r#"{{"ok":true,"search_value":{},"f32_encoded":"0x{:08x}","f64_encoded":"0x{:016x}","base_addr":"0x{:x}","text_size":{},"f32_hits":{},"f64_hits":{},"f32_matches":[{}],"f64_matches":[{}]}}"#,
        value,
        (value as f32).to_bits(),
        value.to_bits(),
        base_addr,
        text_size,
        f32_json.len(),
        f64_json.len(),
        f32_json.join(","),
        f64_json.join(",")
    )
}

/// ★ v3.22.89: /il2cpp/search_int?values=800,900,1000,1100,1200
/// 在umamusume所有内存段（代码+只读数据+数据）中搜索整数千分比
/// 心情系数极可能是整数千分比（1200=1.2倍），硬编码在数据表或指令立即数中
/// 搜索所有umamusume段（含r-- data段，不只r-x代码段）
unsafe fn il2cpp_search_int(values_str: &str) -> String {
    // 解析逗号分隔的整数列表
    let values: Vec<u32> = values_str
        .split(',')
        .filter_map(|s| s.trim().parse::<u32>().ok())
        .collect();
    if values.is_empty() {
        return format!(
            r#"{{"error":"no_valid_integers","input":"{}"}}"#,
            json_escape(values_str)
        );
    }
    if values.len() > 20 {
        return r#"{"error":"too_many_values","max":20}"#.to_string();
    }

    let mut total_segments = 0usize;
    let mut total_scanned = 0usize;
    let mut all_matches: Vec<String> = Vec::new();

    // 读取/proc/self/maps，搜索所有umamusume段
    if let Ok(maps) = std::fs::read_to_string("/proc/self/maps") {
        for line in maps.lines() {
            if !line.contains("umamusume") {
                continue;
            }
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 5 {
                continue;
            }
            let addr_parts: Vec<&str> = parts[0].split('-').collect();
            if addr_parts.len() != 2 {
                continue;
            }
            let start = match usize::from_str_radix(addr_parts[0], 16) {
                Ok(s) => s,
                Err(_) => continue,
            };
            let end = match usize::from_str_radix(addr_parts[1], 16) {
                Ok(e) => e,
                Err(_) => continue,
            };
            let size = end - start;
            let perms = parts[1];
            let mapping = parts.last().unwrap_or(&"");

            if size == 0 {
                continue;
            }
            total_segments += 1;
            total_scanned += size;

            // 限制单段最大扫描（避免超大段超时）
            let scan_size = if size > 256 * 1024 * 1024 {
                256 * 1024 * 1024
            } else {
                size
            };
            let base_ptr = start as *const u8;
            let mut matches_in_segment: Vec<String> = Vec::new();

            for &val in &values {
                if val == 0 {
                    continue;
                } // 跳过0，避免过多匹配
                let le_bytes = val.to_le_bytes();
                let mut found_in_seg = 0usize;

                for off in (0..scan_size.saturating_sub(4)).step_by(4) {
                    let ptr = unsafe { base_ptr.add(off) };
                    let addr = start + off;
                    if addr >= start + scan_size {
                        break;
                    }
                    // 逐字节比对
                    let b0 = unsafe { std::ptr::read_unaligned::<u8>(ptr) };
                    if b0 != le_bytes[0] {
                        continue;
                    }
                    let b1 = unsafe { std::ptr::read_unaligned::<u8>(ptr.add(1)) };
                    if b1 != le_bytes[1] {
                        continue;
                    }
                    let b2 = unsafe { std::ptr::read_unaligned::<u8>(ptr.add(2)) };
                    if b2 != le_bytes[2] {
                        continue;
                    }
                    let b3 = unsafe { std::ptr::read_unaligned::<u8>(ptr.add(3)) };
                    if b3 != le_bytes[3] {
                        continue;
                    }

                    found_in_seg += 1;
                    if found_in_seg > 100 {
                        break;
                    } // 每段每值最多100个匹配
                    matches_in_segment.push(format!(
                        r#"{{"value":{},"addr":"0x{:x}","offset":"0x{:x}"}}"#,
                        val, addr, off
                    ));
                }
            }

            if !matches_in_segment.is_empty() {
                all_matches.push(format!(
                    r#"{{"seg_start":"0x{:x}","seg_end":"0x{:x}","size":{},"perms":"{}","mapping":"{}","hits":{},"matches":[{}]}}"#,
                    start, end, size, perms, mapping,
                    matches_in_segment.len(),
                    matches_in_segment.join(",")
                ));
            }
        }
    }

    if total_segments == 0 {
        return r#"{"error":"cannot_find_umamusume_mapping"}"#.to_string();
    }

    let values_json: Vec<String> = values.iter().map(|v| v.to_string()).collect();
    format!(
        r#"{{"ok":true,"search_values":[{}],"total_segments":{},"total_scanned":{},"segments_with_hits":{},"segments":[{}]}}"#,
        values_json.join(","),
        total_segments,
        total_scanned,
        all_matches.len(),
        all_matches.join(",")
    )
}

/// ★ v3.22.91: /il2cpp/read_mem?addr=0x...&size=4096 (safe /proc/self/mem)
/// 读取任意映射内存地址的原始字节，返回hex dump+ASCII
/// 安全措施：验证地址在/proc/self/maps映射区域内，限制最大65536字节
fn il2cpp_read_mem(addr_str: &str, size_str: &str) -> String {
    let raw_addr = match usize::from_str_radix(
        addr_str.trim_start_matches("0x").trim_start_matches("0X"),
        16,
    ) {
        Ok(a) => a,
        Err(_) => {
            return format!(
                r#"{{"error":"invalid_addr","input":"{}"}}"#,
                json_escape(addr_str)
            )
        }
    };
    let size = size_str.parse::<usize>().unwrap_or(4096);
    if size == 0 || size > 65536 {
        return format!(r#"{{"error":"invalid_size","size":{},"max":65536}}"#, size);
    }
    // ★ v3.22.91: 自动检测偏移量模式。如果addr < 0x1000000，当作SO内偏移
    let mut addr = raw_addr;
    if raw_addr < 0x1000000 {
        if let Ok(maps) = std::fs::read_to_string("/proc/self/maps") {
            for line in maps.lines() {
                if line.contains("umamusume") && line.contains("r-xp") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if !parts.is_empty() {
                        let ap: Vec<&str> = parts[0].split('-').collect();
                        if ap.len() == 2 {
                            if let Ok(start) = usize::from_str_radix(ap[0], 16) {
                                addr = start + raw_addr;
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
    // 验证地址范围在映射区域内
    if let Ok(maps) = std::fs::read_to_string("/proc/self/maps") {
        let mut in_mapped = false;
        let mut seg_name = "";
        for line in maps.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 5 {
                continue;
            }
            let ap: Vec<&str> = parts[0].split('-').collect();
            if ap.len() != 2 {
                continue;
            }
            let seg_start = match usize::from_str_radix(ap[0], 16) {
                Ok(s) => s,
                Err(_) => continue,
            };
            let seg_end = match usize::from_str_radix(ap[1], 16) {
                Ok(e) => e,
                Err(_) => continue,
            };
            if addr >= seg_start && (addr + size) <= seg_end {
                in_mapped = true;
                seg_name = parts.last().unwrap_or(&"");
                break;
            }
        }
        if !in_mapped {
            return format!(
                r#"{{"error":"addr_not_mapped","addr":"0x{:x}","size":{}}}"#,
                addr, size
            );
        }
        let _ = seg_name;
    } else {
        return r#"{"error":"cannot_read_maps"}"#.to_string();
    }

    // v3.22.91: 使用pipe+write系统调用安全读取内存
    // 内核copy_from_user在遇到不可读页时返回EFAULT，不会SIGSEGV
    let mut pipe_fds = [-1i32; 2];
    if unsafe { libc::pipe(pipe_fds.as_mut_ptr()) } != 0 {
        return r#"{"error":"pipe_failed"}"#.to_string();
    }
    let mut buf = vec![0u8; size];
    let mut total_read = 0usize;
    let page_size = 4096;
    for chunk_start in (0..size).step_by(page_size) {
        let chunk_len = if chunk_start + page_size <= size {
            page_size
        } else {
            size - chunk_start
        };
        let src = (addr + chunk_start) as *const libc::c_void;
        let ret = unsafe { libc::write(pipe_fds[1], src, chunk_len) };
        if ret > 0 {
            let n = ret as usize;
            let mut got = 0usize;
            while got < n {
                let r = unsafe {
                    libc::read(
                        pipe_fds[0],
                        buf[total_read..].as_mut_ptr() as *mut libc::c_void,
                        n - got,
                    )
                };
                if r <= 0 {
                    break;
                }
                got += r as usize;
            }
            total_read += n;
        }
        // ret <= 0: page unreadable, skip (leave as 0x00)
    }
    unsafe {
        libc::close(pipe_fds[0]);
        libc::close(pipe_fds[1]);
    }
    if total_read == 0 {
        return format!(
            r#"{{"error":"read_failed","addr":"0x{:x}","size":{},"bytes_read":0}}"#,
            addr, size
        );
    }
    let mut hex_lines: Vec<String> = Vec::new();
    for off in (0..total_read).step_by(16) {
        let remaining = total_read - off;
        let line_size = if remaining >= 16 { 16 } else { remaining };
        let mut hex = String::new();
        let mut ascii = String::new();
        for i in 0..line_size {
            let b = buf[off + i];
            hex.push_str(&format!("{:02x} ", b));
            ascii.push(if b >= 0x20 && b < 0x7f {
                b as char
            } else {
                '.'
            });
        }
        for _ in line_size..16 {
            hex.push_str("   ");
        }
        hex_lines.push(format!("0x{:08x}:  {} {}", off, hex, ascii));
    }
    format!(
        "addr: 0x{:x}\nsize: {}\nbytes_read: {}\n\n{}",
        addr,
        size,
        total_read,
        hex_lines.join("\n")
    )
}

/// D: /il2cpp/disassemble?class=XXX&method=YYY&bytes=2048
/// 反汇编IL2CPP方法的ARM64指令体，返回hex dump + 浮点常量扫描
/// 安全措施：验证methodPointer在umamusume.dll代码段内、4字节对齐检查、限制最大4096字节
unsafe fn il2cpp_disassemble(class_name: &str, method_name: &str, bytes_limit: usize) -> String {
    // 参数验证
    if class_name.is_empty() || method_name.is_empty() {
        return r#"{"error":"missing class or method parameter"}"#.to_string();
    }
    // 限制最大读取字节数防止闪退（最大4096字节，默认2048）
    let max_bytes: usize = 4096;
    let bytes_limit = if bytes_limit == 0 || bytes_limit > max_bytes {
        2048
    } else {
        bytes_limit
    };

    if API.is_null() {
        return r#"{"error":"api_null"}"#.to_string();
    }
    let image = match get_image() {
        img if !img.is_null() => img,
        _ => return r#"{"error":"image_null"}"#.to_string(),
    };

    // 找到目标类
    let class = find_class_by_short_name(image, class_name);
    if class.is_null() {
        return format!(r#"{{"error":"class_not_found","name":"{}"}}"#, class_name);
    }
    let real_class = get_class_name_from_pointer(class);

    // 解析IL2CPP API函数
    let get_methods_fn: Option<
        unsafe extern "C" fn(*mut c_void, *mut *mut c_void) -> *const c_void,
    > = {
        let p = resolve_il2cpp_symbol("il2cpp_class_get_methods");
        if p.is_null() {
            None
        } else {
            Some(std::mem::transmute(p))
        }
    };
    let method_get_name_fn: Option<unsafe extern "C" fn(*const c_void) -> *const c_char> = {
        let p = resolve_il2cpp_symbol("il2cpp_method_get_name");
        if p.is_null() {
            None
        } else {
            Some(std::mem::transmute(p))
        }
    };

    if get_methods_fn.is_none() || method_get_name_fn.is_none() {
        return r#"{"error":"missing_il2cpp_api"}"#.to_string();
    }

    // 遍历类方法，找到目标方法
    let mut target_method_info: *const c_void = ptr::null();
    let mut iter: *mut c_void = ptr::null_mut();
    loop {
        let mi = get_methods_fn.unwrap()(class, &mut iter);
        if mi.is_null() {
            break;
        }
        let name_ptr = method_get_name_fn.unwrap()(mi);
        if !name_ptr.is_null() {
            let name = CStr::from_ptr(name_ptr).to_string_lossy();
            if name == method_name {
                target_method_info = mi;
                break;
            }
        }
    }

    if target_method_info.is_null() {
        return format!(
            r#"{{"error":"method_not_found","class":"{}","method":"{}"}}"#,
            json_escape(class_name),
            json_escape(method_name)
        );
    }

    // 读取methodPointer（MethodInfo结构体，methodPointer在offset 0）
    // IL2CPP MethodInfo布局：offset 0 = methodPointer (8字节指针，64位)
    let method_ptr =
        std::ptr::read_unaligned::<*const c_void>(target_method_info as *const *const c_void);

    if method_ptr.is_null() {
        return format!(
            r#"{{"error":"method_pointer_null","class":"{}","method":"{}"}}"#,
            json_escape(class_name),
            json_escape(method_name)
        );
    }

    let method_addr = method_ptr as usize;

    // 4字节对齐检查（ARM64指令必须是4字节对齐的）
    if method_addr % 4 != 0 {
        return format!(
            r#"{{"error":"pointer_not_aligned","method_addr":"0x{:x}"}}"#,
            method_addr
        );
    }

    // 安全校验：读取/proc/self/maps确认指针在umamusume.dll代码段内
    let mut code_start: usize = 0;
    let mut code_end: usize = 0;
    if let Ok(maps) = std::fs::read_to_string("/proc/self/maps") {
        for line in maps.lines() {
            if line.contains("umamusume") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.is_empty() {
                    continue;
                }
                let addr_parts: Vec<&str> = parts[0].split('-').collect();
                if addr_parts.len() != 2 {
                    continue;
                }
                if let Ok(start) = usize::from_str_radix(addr_parts[0], 16) {
                    if let Ok(end) = usize::from_str_radix(addr_parts[1], 16) {
                        // 合并所有umamusume段（代码段+只读数据段+literal pool）
                        if code_start == 0 || start < code_start {
                            code_start = start;
                        }
                        if end > code_end {
                            code_end = end;
                        }
                    }
                }
            }
        }
    }

    if code_start == 0 || method_addr < code_start || method_addr >= code_end {
        return format!(
            r#"{{"error":"pointer_outside_code_section","method_addr":"0x{:x}","code_start":"0x{:x}","code_end":"0x{:x}"}}"#,
            method_addr, code_start, code_end
        );
    }

    // 计算安全读取字节数（不超出代码段边界，4字节对齐）
    let available_bytes = code_end - method_addr;
    let safe_bytes = available_bytes.min(bytes_limit) & !3;
    if safe_bytes < 16 {
        return format!(
            r#"{{"error":"insufficient_code_bytes","available":{},"method_addr":"0x{:x}"}}"#,
            available_bytes, method_addr
        );
    }

    // 读取方法字节（逐字节read_unaligned，安全检查每个地址）
    let mut bytes = Vec::with_capacity(safe_bytes);
    let mut read_ok = true;
    for i in 0..safe_bytes {
        let byte_ptr = method_ptr as *const u8;
        let addr = byte_ptr as usize + i;
        // 每个字节都检查在代码段内
        if addr < code_start || addr >= code_end {
            read_ok = false;
            break;
        }
        let b = std::ptr::read_unaligned::<u8>(byte_ptr.add(i));
        bytes.push(b);
    }

    if !read_ok || bytes.len() < 16 {
        return r#"{"error":"read_failed"}"#.to_string();
    }

    let bytes_read = bytes.len();

    // 生成hex dump（每16字节一行，适合ARM64分析）
    let mut hex_lines = Vec::new();
    for (i, chunk) in bytes.chunks(16).enumerate() {
        let offset = i * 16;
        let hex: Vec<String> = chunk.iter().map(|b| format!("{:02x}", b)).collect();
        hex_lines.push(format!("{:04x}: {}", offset, hex.join(" ")));
    }

    // 扫描已知浮点常量（やる気系数候选值 + 常见训练系数）
    let known_floats: Vec<(&str, f32)> = vec![
        ("0.6", 0.6),
        ("0.75", 0.75),
        ("0.8", 0.8),
        ("0.9", 0.9),
        ("1.0", 1.0),
        ("1.1", 1.1),
        ("1.2", 1.2),
        ("1.3", 1.3),
        ("1.4", 1.4),
        ("1.5", 1.5),
        ("0.5", 0.5),
        ("2.0", 2.0),
        ("0.7", 0.7),
        ("1.05", 1.05),
        ("1.15", 1.15),
        ("1.25", 1.25),
    ];

    let mut found_constants = Vec::new();

    for (name, val) in &known_floats {
        let f32_bits = val.to_bits().to_le_bytes();
        let f64_val = *val as f64;
        let f64_bits = f64_val.to_bits().to_le_bytes();

        // 搜索f32字节模式（4字节步进，避免误报ARM64指令编码）
        for off in (0..bytes_read.saturating_sub(4)).step_by(4) {
            if bytes[off] == f32_bits[0]
                && bytes[off + 1] == f32_bits[1]
                && bytes[off + 2] == f32_bits[2]
                && bytes[off + 3] == f32_bits[3]
            {
                found_constants.push(format!(
                    r#"{{"name":"{}","type":"f32","offset":{},"hex":"{:02x}{:02x}{:02x}{:02x}"}}"#,
                    name, off, f32_bits[0], f32_bits[1], f32_bits[2], f32_bits[3]
                ));
            }
        }

        // 搜索f64字节模式（8字节，4字节步进）
        if bytes_read >= 8 {
            for off in (0..bytes_read.saturating_sub(8)).step_by(4) {
                if bytes[off..off + 8] == f64_bits[..] {
                    found_constants.push(format!(
                        r#"{{"name":"{}","type":"f64","offset":{},"hex":"{}"}}"#,
                        name,
                        off,
                        f64_bits
                            .iter()
                            .map(|b| format!("{:02x}", b))
                            .collect::<String>()
                    ));
                }
            }
        }
    }

    // 搜索RET指令（0xD65F03C0）标记函数边界
    let ret_bytes: [u8; 4] = [0xC0, 0x03, 0x5F, 0xD6]; // RET的little-endian编码
    let mut ret_offsets = Vec::new();
    for off in (0..bytes_read.saturating_sub(4)).step_by(4) {
        if bytes[off] == ret_bytes[0]
            && bytes[off + 1] == ret_bytes[1]
            && bytes[off + 2] == ret_bytes[2]
            && bytes[off + 3] == ret_bytes[3]
        {
            ret_offsets.push(off);
        }
    }

    let ret_json: Vec<String> = ret_offsets.iter().map(|&off| format!("{}", off)).collect();

    format!(
        r#"{{"ok":true,"class":"{}","real_class":"{}","method":"{}","method_addr":"0x{:x}","bytes_read":{},"code_section":"0x{:x}-0x{:x}","hex_dump":{},"ret_offsets":[{}],"found_constants":[{}]}}"#,
        json_escape(class_name),
        json_escape(&real_class),
        json_escape(method_name),
        method_addr,
        bytes_read,
        code_start,
        code_end,
        json_escape(&hex_lines.join("\n")),
        ret_json.join(","),
        found_constants.join(",")
    )
}

// v3.22.89: 按地址反汇编ARM64指令体（用于分析ExecTraining等方法的子函数调用目标）
// 安全措施：地址必须在umamusume.dll代码段内+4字节对齐+逐字节地址验证+大小限制+RET标记+浮点常量扫描
unsafe fn il2cpp_disassemble_addr(addr_str: &str, bytes_limit: usize) -> String {
    // ★ 先尝试作为绝对地址，失败则尝试作为偏移（base_addr + offset）
    let trimmed = addr_str.trim_start_matches("0x").trim_start_matches("0X");
    let addr_val = match usize::from_str_radix(trimmed, 16) {
        Ok(v) => v,
        Err(_) => {
            return format!(
                r#"{{"error":"invalid_addr_format","received":"{}","hint":"use hex like 0x7336296890"}}"#,
                addr_str
            )
        }
    };

    if addr_val == 0 {
        return r#"{"error":"addr_zero"}"#.to_string();
    }

    // ★ v3.22.91: 自动检测偏移量模式。如果addr < 0x1000000（16MB），当作SO内偏移，需先读取base_addr
    // 注意：search_float返回的abs_addr是绝对地址，不需要此转换；但用户手动传offset时走此路径
    let mut working_addr = addr_val;
    if addr_val < 0x1000000 {
        // 从/proc/self/maps获取umamusume的base_addr
        if let Ok(maps) = std::fs::read_to_string("/proc/self/maps") {
            for line in maps.lines() {
                if line.contains("umamusume") && line.contains("r-xp") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if !parts.is_empty() {
                        let addr_parts: Vec<&str> = parts[0].split('-').collect();
                        if addr_parts.len() == 2 {
                            if let Ok(start) = usize::from_str_radix(addr_parts[0], 16) {
                                working_addr = start + addr_val;
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    // 限制最大读取字节数防止闪退（最大4096字节，默认2048）
    let max_bytes: usize = 4096;
    let bytes_limit = if bytes_limit == 0 || bytes_limit > max_bytes {
        2048
    } else {
        bytes_limit
    };

    // 4字节对齐检查（ARM64指令必须是4字节对齐的）
    if working_addr % 4 != 0 {
        return format!(
            r#"{{"error":"addr_not_aligned","addr":"0x{:x}"}}"#,
            working_addr
        );
    }

    // 安全校验：读取/proc/self/maps确认地址在umamusume.dll代码段内
    let mut code_start: usize = 0;
    let mut code_end: usize = 0;
    if let Ok(maps) = std::fs::read_to_string("/proc/self/maps") {
        for line in maps.lines() {
            if line.contains("umamusume") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.is_empty() {
                    continue;
                }
                let addr_parts: Vec<&str> = parts[0].split('-').collect();
                if addr_parts.len() != 2 {
                    continue;
                }
                if let Ok(start) = usize::from_str_radix(addr_parts[0], 16) {
                    if let Ok(end) = usize::from_str_radix(addr_parts[1], 16) {
                        // 合并所有umamusume段
                        if code_start == 0 || start < code_start {
                            code_start = start;
                        }
                        if end > code_end {
                            code_end = end;
                        }
                    }
                }
            }
        }
    }

    if code_start == 0 || working_addr < code_start || working_addr >= code_end {
        return format!(
            r#"{{"error":"addr_outside_code_section","addr":"0x{:x}","code_start":"0x{:x}","code_end":"0x{:x}"}}"#,
            working_addr, code_start, code_end
        );
    }

    // 计算安全读取字节数（不超出代码段边界，4字节对齐）
    let available_bytes = code_end - working_addr;
    let safe_bytes = available_bytes.min(bytes_limit) & !3;
    if safe_bytes < 16 {
        return format!(
            r#"{{"error":"insufficient_code_bytes","available":{},"addr":"0x{:x}"}}"#,
            available_bytes, working_addr
        );
    }

    // 读取指令字节（逐字节read_unaligned，安全检查每个地址）
    let src_ptr = working_addr as *const u8;
    let mut bytes = Vec::with_capacity(safe_bytes);
    let mut read_ok = true;
    for i in 0..safe_bytes {
        let byte_addr = working_addr + i;
        // 每个字节都检查在代码段内
        if byte_addr < code_start || byte_addr >= code_end {
            read_ok = false;
            break;
        }
        let b = std::ptr::read_unaligned::<u8>(src_ptr.add(i));
        bytes.push(b);
    }

    if !read_ok || bytes.len() < 16 {
        return r#"{"error":"read_failed"}"#.to_string();
    }

    let bytes_read = bytes.len();

    // 生成hex dump（每16字节一行）
    let mut hex_lines = Vec::new();
    for (i, chunk) in bytes.chunks(16).enumerate() {
        let offset = i * 16;
        let hex: Vec<String> = chunk.iter().map(|b| format!("{:02x}", b)).collect();
        hex_lines.push(format!("{:04x}: {}", offset, hex.join(" ")));
    }

    // 扫描已知浮点常量（やる気系数候选值 + 常见训练系数）
    let known_floats: Vec<(&str, f32)> = vec![
        ("0.6", 0.6),
        ("0.75", 0.75),
        ("0.8", 0.8),
        ("0.9", 0.9),
        ("1.0", 1.0),
        ("1.1", 1.1),
        ("1.2", 1.2),
        ("1.3", 1.3),
        ("1.4", 1.4),
        ("1.5", 1.5),
        ("0.5", 0.5),
        ("2.0", 2.0),
        ("0.7", 0.7),
        ("1.05", 1.05),
        ("1.15", 1.15),
        ("1.25", 1.25),
        // 新增やる気系数相关整数（80/90/100/110/120的整数表示）
        ("80.0", 80.0),
        ("90.0", 90.0),
        ("100.0", 100.0),
        ("110.0", 110.0),
        ("120.0", 120.0),
    ];

    let mut found_constants = Vec::new();

    for (name, val) in &known_floats {
        let f32_bits = val.to_bits().to_le_bytes();
        let f64_val = *val as f64;
        let f64_bits = f64_val.to_bits().to_le_bytes();

        // 搜索f32字节模式（4字节步进）
        for off in (0..bytes_read.saturating_sub(4)).step_by(4) {
            if bytes[off] == f32_bits[0]
                && bytes[off + 1] == f32_bits[1]
                && bytes[off + 2] == f32_bits[2]
                && bytes[off + 3] == f32_bits[3]
            {
                found_constants.push(format!(
                    r#"{{"name":"{}","type":"f32","offset":{},"hex":"{:02x}{:02x}{:02x}{:02x}"}}"#,
                    name, off, f32_bits[0], f32_bits[1], f32_bits[2], f32_bits[3]
                ));
            }
        }

        // 搜索f64字节模式（8字节，4字节步进）
        if bytes_read >= 8 {
            for off in (0..bytes_read.saturating_sub(8)).step_by(4) {
                if bytes[off..off + 8] == f64_bits[..] {
                    found_constants.push(format!(
                        r#"{{"name":"{}","type":"f64","offset":{},"hex":"{}"}}"#,
                        name,
                        off,
                        f64_bits
                            .iter()
                            .map(|b| format!("{:02x}", b))
                            .collect::<String>()
                    ));
                }
            }
        }
    }

    // 搜索RET指令（0xD65F03C0）标记函数边界
    let ret_bytes: [u8; 4] = [0xC0, 0x03, 0x5F, 0xD6];
    let mut ret_offsets = Vec::new();
    for off in (0..bytes_read.saturating_sub(4)).step_by(4) {
        if bytes[off] == ret_bytes[0]
            && bytes[off + 1] == ret_bytes[1]
            && bytes[off + 2] == ret_bytes[2]
            && bytes[off + 3] == ret_bytes[3]
        {
            ret_offsets.push(off);
        }
    }

    // 搜索BL指令（ARM64相对跳转），提取调用目标地址
    let mut bl_targets = Vec::new();
    for off in (0..bytes_read.saturating_sub(4)).step_by(4) {
        let insn = u32::from_le_bytes([bytes[off], bytes[off + 1], bytes[off + 2], bytes[off + 3]]);
        // BL指令格式：100101xx xxxxxxxx xxxxxxxx xxxxxxxx (bit31-26=100101)
        if (insn >> 26) == 0b100101 {
            // 解码26位有符号偏移（单位：4字节）
            let imm26 = insn & 0x03FFFFFF;
            // 符号扩展26位到i64
            let offset = if imm26 & 0x02000000 != 0 {
                ((imm26 | 0xFC000000) as u32) as i32 as i64
            } else {
                imm26 as i64
            };
            let target = (working_addr as i64 + (off as i64) + (offset * 4)) as usize;
            bl_targets.push(format!(r#"{{"offset":{},"target":"0x{:x}"}}"#, off, target));
        }
    }

    // 搜索SCVTF指令（整数转浮点）和FMUL/FDIV等浮点运算
    let mut float_ops = Vec::new();
    for off in (0..bytes_read.saturating_sub(4)).step_by(4) {
        let insn = u32::from_le_bytes([bytes[off], bytes[off + 1], bytes[off + 2], bytes[off + 3]]);
        // SCVTF: 0001_1110_xxx0_0010_1100_0010_0000_0000 (多格式)
        let is_scvtf = (insn & 0x7F3FFC00) == 0x1E220000
            || (insn & 0x7F3FFC00) == 0x1E620000
            || (insn & 0x5F3FFC00) == 0x4E220000
            || (insn & 0x5F3FFC00) == 0x0E220000;
        // FMUL: 0001_1110_xx10_0000_xxxx_xxxx_xx00_0000
        let is_fmul = (insn & 0x7F20FC00) == 0x1E200800 || (insn & 0x7F20FC00) == 0x1E008400;
        // FDIV: 0001_1110_xx10_0000_xxxx_xxxx_xx00_1100
        let is_fdiv = (insn & 0x7F20FC00) == 0x1E200800 || (insn & 0x7F20FC00) == 0x1E008C00;
        // SDIV/UDIV: 0001_1010_101x_xxxx_0000_xx_xxxx_00001 / 0001_1010_100x_xxxx_0000_xx_xxxx_00001
        let is_sdiv = (insn & 0x7FE0FC00) == 0x1AC00C00;
        let is_udiv = (insn & 0x7FE0FC00) == 0x1AC00800;

        let op_type = if is_scvtf {
            Some("SCVTF")
        } else if is_fmul {
            Some("FMUL")
        } else if is_fdiv {
            Some("FDIV")
        } else if is_sdiv {
            Some("SDIV")
        } else if is_udiv {
            Some("UDIV")
        } else {
            None
        };

        if let Some(t) = op_type {
            float_ops.push(format!(
                r#"{{"offset":{},"type":"{}","hex":"{:02x}{:02x}{:02x}{:02x}"}}"#,
                off,
                t,
                bytes[off],
                bytes[off + 1],
                bytes[off + 2],
                bytes[off + 3]
            ));
        }
    }

    let ret_json: Vec<String> = ret_offsets.iter().map(|&off| format!("{}", off)).collect();

    format!(
        r#"{{"ok":true,"addr":"0x{:x}","bytes_read":{},"code_section":"0x{:x}-0x{:x}","hex_dump":{},"ret_offsets":[{}],"found_constants":[{}],"bl_targets":[{}],"float_ops":[{}]}}"#,
        working_addr,
        bytes_read,
        code_start,
        code_end,
        json_escape(&hex_lines.join("\n")),
        ret_json.join(","),
        found_constants.join(","),
        bl_targets.join(","),
        float_ops.join(",")
    )
}

// v3.22.89: 暴力dump全部类的方法目录（类名+方法名+地址+签名+静态标记）
// 支持letter参数按A-Z分组，避免手机端一次性下载数据过大
unsafe fn il2cpp_dump_all_methods(letter: &str) -> String {
    let image = get_image();
    if image.is_null() {
        return r#"{"error":"image_null"}"#.to_string();
    }

    // 解析IL2CPP API函数指针
    let get_count_fn = resolve_il2cpp_symbol("il2cpp_image_get_class_count");
    let get_class_fn = resolve_il2cpp_symbol("il2cpp_image_get_class");
    if get_count_fn.is_null() || get_class_fn.is_null() {
        return r#"{"error":"class_enum_api_not_found"}"#.to_string();
    }
    let get_count: FnImageGetClassCount = std::mem::transmute(get_count_fn);
    let get_class: FnImageGetClass = std::mem::transmute(get_class_fn);

    // 方法遍历API
    let get_methods_fn: Option<
        unsafe extern "C" fn(*mut c_void, *mut *mut c_void) -> *const c_void,
    > = {
        let p = resolve_il2cpp_symbol("il2cpp_class_get_methods");
        if p.is_null() {
            None
        } else {
            Some(std::mem::transmute(p))
        }
    };
    let method_get_name_fn: Option<unsafe extern "C" fn(*const c_void) -> *const c_char> = {
        let p = resolve_il2cpp_symbol("il2cpp_method_get_name");
        if p.is_null() {
            None
        } else {
            Some(std::mem::transmute(p))
        }
    };
    let method_get_param_count_fn: Option<unsafe extern "C" fn(*const c_void) -> u32> = {
        let p = resolve_il2cpp_symbol("il2cpp_method_get_param_count");
        if p.is_null() {
            None
        } else {
            Some(std::mem::transmute(p))
        }
    };
    let method_get_return_type_fn: Option<unsafe extern "C" fn(*const c_void) -> *const c_void> = {
        let p = resolve_il2cpp_symbol("il2cpp_method_get_return_type");
        if p.is_null() {
            None
        } else {
            Some(std::mem::transmute(p))
        }
    };
    let method_get_flags_fn: Option<unsafe extern "C" fn(*const c_void, *mut u32) -> u32> = {
        let p = resolve_il2cpp_symbol("il2cpp_method_get_flags");
        if p.is_null() {
            None
        } else {
            Some(std::mem::transmute(p))
        }
    };

    // 类名/命名空间读取
    let get_name_fn = resolve_il2cpp_symbol("il2cpp_class_get_name");
    let get_namespace_fn = resolve_il2cpp_symbol("il2cpp_class_get_namespace");

    if get_methods_fn.is_none() || method_get_name_fn.is_none() {
        return r#"{"error":"method_enum_api_not_found"}"#.to_string();
    }

    let total = get_count(image);
    let mut all_classes: Vec<String> = Vec::new();
    let mut total_methods: usize = 0;

    // letter参数：只dump指定首字母的类（A-Z分组），ALL或空=全部
    let filter_letter = letter.trim().to_uppercase();
    let do_filter = !filter_letter.is_empty() && filter_letter != "ALL";

    for i in 0..total {
        let cls = get_class(image, i);
        if cls.is_null() {
            continue;
        }

        // 读取类名
        let class_name = if !get_name_fn.is_null() {
            let name_fn: FnClassGetName = std::mem::transmute(get_name_fn);
            let cstr = name_fn(cls);
            if cstr.is_null() {
                continue;
            }
            std::ffi::CStr::from_ptr(cstr)
                .to_string_lossy()
                .into_owned()
        } else {
            continue;
        };

        // 读取命名空间
        let namespace = if !get_namespace_fn.is_null() {
            let ns_fn: FnClassGetName = std::mem::transmute(get_namespace_fn);
            let cstr = ns_fn(cls);
            if cstr.is_null() {
                String::new()
            } else {
                std::ffi::CStr::from_ptr(cstr)
                    .to_string_lossy()
                    .into_owned()
            }
        } else {
            String::new()
        };

        // 按首字母过滤
        if do_filter {
            let first = class_name
                .chars()
                .next()
                .unwrap_or('_')
                .to_ascii_uppercase();
            let target = filter_letter
                .chars()
                .next()
                .unwrap_or('_')
                .to_ascii_uppercase();
            if first != target {
                continue;
            }
        }

        // 检查是否枚举（枚举类没有有意义的方法）
        let is_enum_fn: Option<unsafe extern "C" fn(*const c_void) -> bool> = {
            let p = resolve_il2cpp_symbol("il2cpp_class_is_enum");
            if p.is_null() {
                None
            } else {
                Some(std::mem::transmute(p))
            }
        };
        let is_enum = is_enum_fn.map(|f| f(cls)).unwrap_or(false);

        // 遍历该类的所有方法
        let mut methods_arr: Vec<String> = Vec::new();
        let mut iter: *mut c_void = ptr::null_mut();
        loop {
            let method_info = get_methods_fn.unwrap()(cls, &mut iter);
            if method_info.is_null() {
                break;
            }

            let method_name = {
                let name_ptr = method_get_name_fn.unwrap()(method_info);
                if name_ptr.is_null() {
                    "(null)".to_string()
                } else {
                    let cstr = CStr::from_ptr(name_ptr);
                    cstr.to_string_lossy().into_owned()
                }
            };

            // 跳过构造函数(.ctor/.cctor)
            if method_name.starts_with('.') {
                continue;
            }

            // 读取methodPointer（MethodInfo offset 0）
            let method_ptr =
                std::ptr::read_unaligned::<*const c_void>(method_info as *const *const c_void);
            let method_addr = if method_ptr.is_null() {
                0usize
            } else {
                method_ptr as usize
            };

            // 参数数、返回类型、是否静态
            let param_count = method_get_param_count_fn
                .map(|f| f(method_info))
                .unwrap_or(0);
            let return_type_str = method_get_return_type_fn
                .map(|f| {
                    let rt = f(method_info);
                    if rt.is_null() {
                        "void".to_string()
                    } else {
                        let te = il2cpp_type_get_type_enum(rt);
                        type_enum_to_name(te)
                    }
                })
                .unwrap_or_else(|| "?".to_string());
            let is_static = method_get_flags_fn
                .map(|f| {
                    let mut iflags: u32 = 0;
                    let flags = f(method_info, &mut iflags);
                    (flags & 0x0010) != 0
                })
                .unwrap_or(false);

            methods_arr.push(format!(
                r#"{{"name":"{}","addr":"0x{:x}","params":{},"return_type":"{}","static":{}}}"#,
                json_escape(&method_name),
                method_addr,
                param_count,
                return_type_str,
                is_static
            ));
        }

        total_methods += methods_arr.len();

        // 即使没有方法也记录类信息（标记is_enum）
        if !methods_arr.is_empty() || !is_enum {
            all_classes.push(format!(
                r#"{{"class":"{}","ns":"{}","is_enum":{},"method_count":{},"methods":[{}]}}"#,
                json_escape(&class_name),
                json_escape(&namespace),
                is_enum,
                methods_arr.len(),
                methods_arr.join(",")
            ));
        }
    }

    format!(
        r#"{{"ok":true,"total_classes":{},"filtered_classes":{},"total_methods":{},"letter":"{}","classes":[{}]}}"#,
        total,
        all_classes.len(),
        total_methods,
        json_escape(&filter_letter),
        all_classes.join(",")
    )
}
