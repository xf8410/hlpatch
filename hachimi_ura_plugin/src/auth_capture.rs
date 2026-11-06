//! 登录器认证/会话头抓取 v3.29.1 —— 补 UnityWebRequest 层观测缺口。
//!
//! 仓库依据：uma-launcher-login-roadmap / uma-single-mode-disasm@coneshell-sniff /
//! uma-ai-context@auth-sniff-checkpoint / uma-ref-hub@auth-and-repack-analysis /
//! uma-sniffkit/routes.json。认证 4 端点走 UnityWebRequest，旧 hook（WWWRequest.Post
//! / HttpHelper.Compress|Decompress）抓不到；lib.rs 已有 SendWebRequest hook 观测请求
//! 入口却不抓头，且 /api/sniff/unity 没登记到 /health / 404 / BOOT_SAFE。
//!
//! 本模块挂两个 Unity 标准 API（ABI 确定、返回原样透传）：
//!   SetRequestHeader(this, name, value) -> void      请求头：SID/ViewerID/APP-VER/...
//!   GetResponseHeader(this, name)       -> String    响应头：data_headers.sid
//! 另复用 UNITY_OBSERVATIONS 把认证 4 端点握手单独分类输出。
//!
//! 纪律（同 rng_consumption_probe）：默认不装 hook、懒安装；hook 内零分配、只写
//! POD 环形缓冲；字符串读带长度上限；catch_unwind 隔离；状态端点明确列未覆盖项。
//! 端点：/api/auth/{status,targets,headers,handshake,toggle,clear}

use std::ffi::c_void;
use std::fmt::Write as _;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering::Relaxed};
use std::sync::OnceLock;
use std::time::Instant;

const CAP: usize = 1024; // 2 的幂
const KIND_REQ: u8 = 1;
const KIND_RESP: u8 = 2;
const NAME_MAX: usize = 48;
const VALUE_MAX: usize = 192;

#[repr(C)]
#[derive(Copy, Clone)]
struct HeaderEvent {
    seq: u64,
    t_us: u64,
    kind: u8,
    name_len: u8,
    value_len: u16,
    name: [u8; NAME_MAX],
    value: [u8; VALUE_MAX],
}
impl HeaderEvent {
    const ZERO: Self = Self {
        seq: 0, t_us: 0, kind: 0, name_len: 0, value_len: 0,
        name: [0; NAME_MAX], value: [0; VALUE_MAX],
    };
}

static mut EVENTS: [HeaderEvent; CAP] = [HeaderEvent::ZERO; CAP];
static WRITE_SEQ: AtomicU64 = AtomicU64::new(0);
static ENABLED: AtomicBool = AtomicBool::new(false);
static HOOKS_INSTALLED: AtomicBool = AtomicBool::new(false);
static ADDR_SET_HEADER: AtomicU64 = AtomicU64::new(0);
static ADDR_GET_RESP: AtomicU64 = AtomicU64::new(0);
static CLASS_FOUND: AtomicBool = AtomicBool::new(false);
static BASE: OnceLock<Instant> = OnceLock::new();

#[inline(always)]
fn now_us() -> u64 {
    BASE.get_or_init(Instant::now).elapsed().as_micros() as u64
}

/// 只读 IL2CPP String 的前 n 个字符，压成 ASCII 写进定长缓冲。零分配、零格式化。
/// 头名/头值都是 ASCII；非 ASCII 用 '?' 占位。长度异常返回 0。
#[inline(always)]
unsafe fn copy_str(s: *mut c_void, out: &mut [u8]) -> usize {
    if s.is_null() {
        return 0;
    }
    let len = core::ptr::read_unaligned((s as *const u8).add(16) as *const i32);
    if len <= 0 || len as usize > out.len() {
        return 0;
    }
    let chars = (s as *const u8).add(20) as *const u16;
    for i in 0..len as usize {
        let c = core::ptr::read_unaligned(chars.add(i));
        out[i] = if c < 128 { c as u8 } else { b'?' };
    }
    len as usize
}

#[inline(always)]
unsafe fn record(kind: u8, name: *mut c_void, value: *mut c_void) {
    if !ENABLED.load(Relaxed) {
        return;
    }
    let mut e = HeaderEvent::ZERO;
    e.name_len = copy_str(name, &mut e.name) as u8;
    e.value_len = copy_str(value, &mut e.value) as u16;
    e.kind = kind;
    e.seq = WRITE_SEQ.fetch_add(1, Relaxed) + 1;
    e.t_us = now_us();
    let slot = core::ptr::addr_of_mut!(EVENTS)
        .cast::<HeaderEvent>()
        .add(e.seq as usize & (CAP - 1));
    core::ptr::write_volatile(slot, e);
}

#[inline(always)]
fn safe_record(kind: u8, name: *mut c_void, value: *mut c_void) {
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
        record(kind, name, value);
    }));
}

// ---- trampoline：每个 detour 用自身地址查，避免串线 -----------------------

type FnSetHeader = unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void);
type FnGetResp = unsafe extern "C" fn(*mut c_void, *mut c_void) -> *mut c_void;

#[inline(always)]
unsafe fn orig<T: Copy>(hook: usize) -> Option<T> {
    let t = super::interceptor_get_trampoline(hook);
    if t == 0 { None } else { Some(std::mem::transmute_copy(&t)) }
}

macro_rules! ha {
    ($f:ident) => { $f as *const () as usize };
}

extern "C" fn set_header_hook(this: *mut c_void, name: *mut c_void, value: *mut c_void) {
    let Some(f) = (unsafe { orig::<FnSetHeader>(ha!(set_header_hook)) }) else { return };
    unsafe { f(this, name, value) };
    safe_record(KIND_REQ, name, value);
}

extern "C" fn get_resp_hook(this: *mut c_void, name: *mut c_void) -> *mut c_void {
    let Some(f) = (unsafe { orig::<FnGetResp>(ha!(get_resp_hook)) }) else {
        return std::ptr::null_mut();
    };
    let r = unsafe { f(this, name) };
    safe_record(KIND_RESP, name, r); // 返回值即头值 String
    r
}

// ---- 解析 UnityWebRequest 两个标准方法 -------------------------------------

unsafe fn resolve() {
    if super::API.is_null() {
        return;
    }
    let Some(get_asm) = (*super::API).il2cpp_get_assembly_image_fn else { return };
    let Some(get_class) = (*super::API).il2cpp_get_class_fn else { return };
    let Some(get_method) = (*super::API).il2cpp_get_method_addr_fn else { return };

    let asm = super::to_cstr("UnityEngine.UnityWebRequestModule.dll");
    let img = get_asm(asm.as_ptr());
    if img.is_null() {
        return;
    }
    let ns = super::to_cstr("UnityEngine.Networking");
    let cn = super::to_cstr("UnityWebRequest");
    let cls = get_class(img, ns.as_ptr(), cn.as_ptr());
    if cls.is_null() {
        return;
    }
    CLASS_FOUND.store(true, Relaxed);

    let n1 = super::to_cstr("SetRequestHeader");
    let n2 = super::to_cstr("GetResponseHeader");
    let a1 = get_method(cls as usize, n1.as_ptr(), 2);
    let a2 = get_method(cls as usize, n2.as_ptr(), 1);
    ADDR_SET_HEADER.store(a1 as u64, Relaxed);
    ADDR_GET_RESP.store(a2 as u64, Relaxed);
}

unsafe fn install_hooks() -> i32 {
    if HOOKS_INSTALLED.load(std::sync::atomic::Ordering::Acquire) {
        return 2;
    }
    if super::API.is_null() {
        return 0;
    }
    let pairs: [(u64, usize, &str); 2] = [
        (ADDR_SET_HEADER.load(Relaxed), ha!(set_header_hook), "SetRequestHeader"),
        (ADDR_GET_RESP.load(Relaxed), ha!(get_resp_hook), "GetResponseHeader"),
    ];
    let mut ok = 0;
    for (addr, h, label) in pairs {
        let key = format!("auth.{}", label);
        let st = if addr == 0 {
            "skipped: no_addr".to_string()
        } else if super::interceptor_hook(addr as usize, h) {
            ok += 1;
            format!("hooked@0x{:x}", addr)
        } else {
            "failed: interceptor_hook".to_string()
        };
        super::set_hook_status(&key, &st);
    }
    if ok > 0 {
        HOOKS_INSTALLED.store(true, std::sync::atomic::Ordering::Release);
        let msg = format!("auth_capture: installed {}/2 detours", ok);
        super::ura_log(3, &msg);
    }
    ok
}

// ---- JSON：格式串不含字面花括号，外层括号 push，零双花括号 ------------------

fn text(buf: &[u8], n: usize) -> String {
    super::json_escape(&String::from_utf8_lossy(&buf[..n.min(buf.len())]))
}

fn event_json(e: &HeaderEvent) -> String {
    let mut s = String::from("{");
    let _ = write!(s,
        r#""seq":{},"t_us":{},"kind":"{}""#,
        e.seq, e.t_us, if e.kind == KIND_REQ { "request" } else { "response" });
    let _ = write!(s, r#","name":"{}","value":"{}""#,
        text(&e.name, e.name_len as usize), text(&e.value, e.value_len as usize));
    s.push('}');
    s
}

fn collect(after: u64, limit: usize) -> (String, usize) {
    let base = core::ptr::addr_of_mut!(EVENTS).cast::<HeaderEvent>();
    let mut out: Vec<(u64, String)> = Vec::new();
    unsafe {
        for i in 0..CAP {
            let e = core::ptr::read_volatile(base.add(i));
            if e.seq == 0 || e.seq <= after || (e.seq as usize & (CAP - 1)) != i {
                continue;
            }
            out.push((e.seq, event_json(&e)));
        }
    }
    out.sort_by_key(|x| x.0);
    if out.len() > limit {
        out.drain(..out.len() - limit);
    }
    let n = out.len();
    (out.iter().map(|x| x.1.as_str()).collect::<Vec<_>>().join(","), n)
}

/// 认证握手 4 端点（仓库实测顺序）
const HANDSHAKE: [&str; 4] = [
    "/umamusume/tool/get_verify_token",
    "/umamusume/tool/pre_signup",
    "/umamusume/tool/signup",
    "/umamusume/tool/start_session",
];

fn handshake_index(path: &str) -> Option<usize> {
    HANDSHAKE.iter().position(|p| path.ends_with(p) || path.contains(p))
}

/// GET /api/auth/handshake —— 从已有 SendWebRequest 观测里分类认证 4 端点
pub fn handshake_endpoint() -> String {
    let mut seen: [Option<(u64, usize, String)>; 4] = Default::default();
    if let Ok(g) = super::UNITY_OBSERVATIONS.lock() {
        for o in g.iter() {
            if let Some(i) = handshake_index(&o.path) {
                seen[i] = Some((o.id, o.body_size, o.body_hex.clone()));
            }
        }
    }
    let mut items: Vec<String> = Vec::with_capacity(4);
    for (i, p) in HANDSHAKE.iter().enumerate() {
        let mut s = String::from("{");
        let _ = write!(s, r#""step":{},"path":"{}","observed":{}"#, i + 1, p, seen[i].is_some());
        if let Some((id, size, hex)) = &seen[i] {
            let _ = write!(s, r#","observation_id":{},"body_size":{},"body_hex":"{}""#, id, size, hex);
        }
        s.push('}');
        items.push(s);
    }
    let missing: Vec<String> = HANDSHAKE.iter().enumerate()
        .filter(|(i, _)| seen[*i].is_none())
        .map(|(_, p)| format!(r#""{}""#, p))
        .collect();
    let mut s = String::from("{");
    let _ = write!(s,
        r#""ok":true,"send_hook_installed":{},"count":{},"missing_count":{}"#,
        unsafe { super::UNITY_SEND_ADDR != 0 }, 4 - missing.len(), missing.len());
    s.push_str(r#","steps":["#);
    s.push_str(&items.join(","));
    s.push_str(r#"],"missing":["#);
    s.push_str(&missing.join(","));
    s.push_str(r#"],"hint":"冷启动才会走认证流程；未观测到即该局未重新登录""#);
    s.push('}');
    s
}

/// GET /api/auth/targets —— 全仓比对后的覆盖表，直接回答「什么没挂上」。
/// 每行一条完整 JSON 对象；join 生成，响应体不含裸换行。
const TARGET_ROWS: &[&str] = &[
    r#"{"id":"A","item":"身份标识 ViewerID/device/device_id/device_name/locale/carrier","state":"covered","by":"SetRequestHeader + sniff body_hex"}"#,
    r#"{"id":"B","item":"SID 链：响应 data_headers.sid 与下一请求 SID","state":"covered","by":"GetResponseHeader + MakeMd5 MD5_LOG + sniff 响应体"}"#,
    r#"{"id":"C","item":"APP-VER / RES-VER(含 manifest token)","state":"covered","by":"SetRequestHeader"}"#,
    r#"{"id":"D","item":"设备指纹 graphics_device_name/ip_address/platform_os_version/keychain/button_info","state":"partial","by":"sniff body_hex；keychain/button_info 内嵌 JSON 未解析"}"#,
    r#"{"id":"E","item":"传输栈补全头 X-Unity-Version / User-Agent / Accept-Encoding","state":"missing","by":"Unity native 层自动追加，IL2CPP hook 抓不到，需登录器手动补"}"#,
    r#"{"id":"F","item":"认证 4 端点握手包 get_verify_token/pre_signup/signup/start_session","state":"covered","by":"SendWebRequest 观测 + /api/auth/handshake 分类（本轮新登记）"}"#,
    r#"{"id":"G","item":"Coneshell V2/V3/V4 密文冷启动抓取","state":"missing","by":"需在 UnityWebRequest 更早层或 native 层落盘，本模块只抓解密后头/体"}"#,
    r#"{"id":"H","item":"serverPublicKey(32B)/versionKey(20B)/vfsCert(472B)","state":"missing","by":"需 InitializeConeshell 反汇编取 rodata 常量，属真机取证"}"#,
    r#"{"id":"I","item":"Certification._viewerId/_sessionId/_udid 会话注入目标","state":"missing","by":"需真机定位 MonoSingleton getter 后安全读，本轮不盲挂"}"#,
    r#"{"id":"J","item":"PlayIntegrity / CygamesIdLogin / LoginToken 登录令牌","state":"missing","by":"属账号凭据，按仓库纪律不抓、不落盘"}"#,
];

pub fn targets_endpoint() -> String {
    let mut s = String::from("{");
    let _ = write!(s,
        r#""ok":true,"version":"{}","direction":"consumption_probe_not_seed_replay","note":"state=covered|partial|missing；以全仓比对为准，不虚报""#,
        super::PLUGIN_VERSION);
    s.push_str(r#","targets":["#);
    s.push_str(&TARGET_ROWS.join(","));
    s.push_str("]}");
    s
}

/// GET /api/auth/status
pub fn status_endpoint() -> String {
    let a1 = ADDR_SET_HEADER.load(Relaxed);
    let a2 = ADDR_GET_RESP.load(Relaxed);
    let mut s = String::from("{");
    let _ = write!(s,
        r#""ok":true,"version":"{}","enabled":{},"hooks_installed":{},"class_found":{},"cap":{},"total_captured":{}"#,
        super::PLUGIN_VERSION, ENABLED.load(Relaxed),
        HOOKS_INSTALLED.load(Relaxed), CLASS_FOUND.load(Relaxed),
        CAP, WRITE_SEQ.load(Relaxed));
    s.push_str(r#","addresses":{"#);
    let _ = write!(s,
        r#""SetRequestHeader":"0x{:x}","GetResponseHeader":"0x{:x}""#, a1, a2);
    s.push('}');
    let _ = write!(s,
        r#","sendwebrequest_hooked":{},"sniff_enabled":{},"targets":"#,
        unsafe { super::UNITY_SEND_ADDR != 0 },
        super::SNIFF_ENABLED.load(Relaxed));
    s.push_str(r#""/api/auth/targets""#);
    s.push_str(r#","pending":["真机确认 SetRequestHeader/GetResponseHeader 的 IL2CPP 参数个数与调用约定","确认 UnityWebRequestModule.dll 程序集名（部分版本为 UnityEngine.CoreModule.dll）","响应头 sid 是否已由 GetResponseHeader 覆盖，缺失时改读响应体 data_headers.sid"]"#);
    s.push('}');
    s
}

/// GET /api/auth/headers?after=<seq>&limit=<n>
pub fn headers_endpoint(uri: &str) -> String {
    let after = super::parse_query(uri, "after").parse::<u64>().unwrap_or(0);
    let limit = super::parse_query(uri, "limit").parse::<usize>().unwrap_or(256).min(CAP);
    let (joined, n) = collect(after, limit);
    let mut s = String::from("{");
    let _ = write!(s,
        r#""ok":true,"enabled":{},"total_captured":{},"after":{},"count":{}"#,
        ENABLED.load(Relaxed), WRITE_SEQ.load(Relaxed), after, n);
    s.push_str(r#","headers":["#);
    s.push_str(&joined);
    s.push_str("]}");
    s
}

/// GET /api/auth/toggle?enabled=1|0
pub fn toggle_endpoint(uri: &str) -> String {
    let want = match super::parse_query(uri, "enabled").as_str() {
        "1" | "true" | "on" => true,
        "0" | "false" | "off" => false,
        _ => !ENABLED.load(Relaxed),
    };
    if !want {
        ENABLED.store(false, std::sync::atomic::Ordering::Release);
        return r#"{"ok":true,"enabled":false,"hint":"记录停止；detour 仍在但 hook 内直接跳过"}"#.into();
    }
    unsafe { resolve(); }
    let n = unsafe { install_hooks() };
    ENABLED.store(true, std::sync::atomic::Ordering::Release);
    let mut s = String::from("{");
    let _ = write!(s,
        r#""ok":true,"enabled":true,"detours_installed":{},"hint":"高频头 hook 已开启；观测完请 enabled=0""#, n);
    s.push('}');
    s
}

/// GET /api/auth/clear
pub fn clear_endpoint() -> String {
    WRITE_SEQ.store(0, std::sync::atomic::Ordering::Release);
    unsafe {
        let base = core::ptr::addr_of_mut!(EVENTS).cast::<HeaderEvent>();
        for i in 0..CAP {
            core::ptr::write_volatile(base.add(i), HeaderEvent::ZERO);
        }
    }
    r#"{"ok":true,"cleared":true}"#.into()
}
