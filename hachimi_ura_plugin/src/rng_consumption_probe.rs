//! RNG 消费点插桩 v3.29.1 —— 不猜种子，只记「何时/哪个消费点/用了什么随机数」。
//!
//! 旧路线 `_fixedTurnCharaSeed @0x198` 是 ObscuredInt（加密 int），非 PRNG 状态，单值不可重放。
//! 新路线 hook：RaceRandom.GetRandom(i4/r4 × 1/2 参)、get/set_RaceRandomProgramArray，
//! 另登记 SingleModeRaceRandomProgram 方法清单（真实消费入口待真机定位）。
//!
//! 纪律：默认不装 hook（GetRandom 是高频调用）；hook 内不 invoke IL2CPP、不分配、
//! 不格式化，只写 CAP=4096 的 POD 环形缓冲；原函数走 trampoline，返回值原样透传；
//! i4 用整数返回、r4 用 f32 返回（AAPCS64 x0 / s0 不能混）；记录用 catch_unwind 隔离。
//!
//! 端点：/api/rng/consumption{,/status,/toggle,/clear,/context}

use std::ffi::{c_char, c_void, CStr};
use std::fmt::Write as _;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU64, Ordering::Relaxed};
use std::sync::OnceLock;
use std::time::Instant;

pub const CAP: usize = 4096; // 2 的幂

const SITE_I4_1: u8 = 1;
const SITE_I4_2: u8 = 2;
const SITE_R4_1: u8 = 3;
const SITE_R4_2: u8 = 4;
const SITE_RPA_GET: u8 = 5;
const SITE_RPA_SET: u8 = 6;
const SITE_CTX: u8 = 7;

const KIND_I4: u8 = 1;
const KIND_R4: u8 = 2;
const KIND_PTR: u8 = 3;
const KIND_VOID: u8 = 4;

#[repr(C)]
#[derive(Copy, Clone)]
struct Event {
    seq: u64,
    t_us: u64,   // 单调微秒：什么时候
    step: u32,   // PREDICT_STEP 快照
    turn: i32,   // /context 标注，默认 -1
    action: i32,
    tid: u32,
    site: u8,    // 哪个消费点
    kind: u8,
    a0: u64,     // 参数原始位
    a1: u64,
    ret: u64,    // 用了什么随机数
}
impl Event {
    const ZERO: Self = Self {
        seq: 0, t_us: 0, step: 0, turn: -1, action: -1, tid: 0,
        site: 0, kind: 0, a0: 0, a1: 0, ret: 0,
    };
}

static mut EVENTS: [Event; CAP] = [Event::ZERO; CAP];
static WRITE_SEQ: AtomicU64 = AtomicU64::new(0);
static ENABLED: AtomicBool = AtomicBool::new(false);
static HOOKS_INSTALLED: AtomicBool = AtomicBool::new(false);
static CTX_TURN: AtomicI32 = AtomicI32::new(-1);
static CTX_ACTION: AtomicI32 = AtomicI32::new(-1);
static BASE: OnceLock<Instant> = OnceLock::new();

static ADDR_I4_1: AtomicU64 = AtomicU64::new(0);
static ADDR_I4_2: AtomicU64 = AtomicU64::new(0);
static ADDR_R4_1: AtomicU64 = AtomicU64::new(0);
static ADDR_R4_2: AtomicU64 = AtomicU64::new(0);
static ADDR_RPA_GET: AtomicU64 = AtomicU64::new(0);
static ADDR_RPA_SET: AtomicU64 = AtomicU64::new(0);
static SMRRP_FOUND: AtomicBool = AtomicBool::new(false);
static RESOLVE_MODE: AtomicI32 = AtomicI32::new(0); // 0 未解析 1 反射精确 2 降级
static SMRRP_JSON: OnceLock<String> = OnceLock::new();

#[inline(always)]
fn now_us() -> u64 {
    BASE.get_or_init(Instant::now).elapsed().as_micros() as u64
}

#[inline(always)]
unsafe fn record(site: u8, kind: u8, a0: u64, a1: u64, ret: u64) {
    if !ENABLED.load(Relaxed) {
        return;
    }
    let s = WRITE_SEQ.fetch_add(1, Relaxed) + 1;
    let e = Event {
        seq: s,
        t_us: now_us(),
        step: super::PREDICT_STEP.load(Relaxed),
        turn: CTX_TURN.load(Relaxed),
        action: CTX_ACTION.load(Relaxed),
        tid: libc::gettid() as u32,
        site, kind, a0, a1, ret,
    };
    let slot = core::ptr::addr_of_mut!(EVENTS).cast::<Event>().add(s as usize & (CAP - 1));
    core::ptr::write_volatile(slot, e);
}

#[inline(always)]
fn safe_record(site: u8, kind: u8, a0: u64, a1: u64, ret: u64) {
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
        record(site, kind, a0, a1, ret);
    }));
}

// ---- 原函数调用：每个 detour 用自身地址查 trampoline，避免串线 -------------

type I4One = unsafe extern "C" fn(*mut c_void, usize) -> usize;
type I4Two = unsafe extern "C" fn(*mut c_void, usize, usize) -> usize;
type R4One = unsafe extern "C" fn(*mut c_void, f32) -> f32;
type R4Two = unsafe extern "C" fn(*mut c_void, f32, f32) -> f32;
type PtrGet = unsafe extern "C" fn(*mut c_void) -> usize;
type PtrSet = unsafe extern "C" fn(*mut c_void, usize);

#[inline(always)]
unsafe fn orig<T: Copy>(hook: usize) -> Option<T> {
    let t = super::interceptor_get_trampoline(hook);
    if t == 0 { None } else { Some(std::mem::transmute_copy(&t)) }
}

// 函数地址统一走 *const () 转 usize，避免 fn-to-numeric-cast 警告
macro_rules! ha {
    ($f:ident) => { $f as *const () as usize };
}

extern "C" fn gr_i4_1(this: *mut c_void, a: usize) -> usize {
    let Some(f) = (unsafe { orig::<I4One>(ha!(gr_i4_1)) }) else { return 0 };
    let r = unsafe { f(this, a) };
    safe_record(SITE_I4_1, KIND_I4, a as u64, 0, r as u64);
    r
}
extern "C" fn gr_i4_2(this: *mut c_void, a: usize, b: usize) -> usize {
    let Some(f) = (unsafe { orig::<I4Two>(ha!(gr_i4_2)) }) else { return 0 };
    let r = unsafe { f(this, a, b) };
    safe_record(SITE_I4_2, KIND_I4, a as u64, b as u64, r as u64);
    r
}
extern "C" fn gr_r4_1(this: *mut c_void, a: f32) -> f32 {
    let Some(f) = (unsafe { orig::<R4One>(ha!(gr_r4_1)) }) else { return 0.0 };
    let r = unsafe { f(this, a) };
    safe_record(SITE_R4_1, KIND_R4, a.to_bits() as u64, 0, r.to_bits() as u64);
    r
}
extern "C" fn gr_r4_2(this: *mut c_void, a: f32, b: f32) -> f32 {
    let Some(f) = (unsafe { orig::<R4Two>(ha!(gr_r4_2)) }) else { return 0.0 };
    let r = unsafe { f(this, a, b) };
    safe_record(SITE_R4_2, KIND_R4, a.to_bits() as u64, b.to_bits() as u64, r.to_bits() as u64);
    r
}
// getter 只记指针，不解引用（hook 内读游戏内存会 SIGSEGV）
extern "C" fn rpa_get(this: *mut c_void) -> usize {
    let Some(f) = (unsafe { orig::<PtrGet>(ha!(rpa_get)) }) else { return 0 };
    let r = unsafe { f(this) };
    safe_record(SITE_RPA_GET, KIND_PTR, 0, 0, r as u64);
    r
}
extern "C" fn rpa_set(this: *mut c_void, v: usize) {
    if let Some(f) = unsafe { orig::<PtrSet>(ha!(rpa_set)) } {
        unsafe { f(this, v) };
    }
    safe_record(SITE_RPA_SET, KIND_VOID, v as u64, 0, 0);
}

// ---- IL2CPP 反射：按 (返回类型, 参数个数) 区分 GetRandom 的 i4/r4 重载 ----

type GetMethods = unsafe extern "C" fn(*mut c_void, *mut *mut c_void) -> *const c_void;
type GetName = unsafe extern "C" fn(*const c_void) -> *const c_char;
type ParamCount = unsafe extern "C" fn(*const c_void) -> u32;
type ReturnType = unsafe extern "C" fn(*const c_void) -> *const c_void;
type TypeEnum = unsafe extern "C" fn(*const c_void) -> i32;
type MethodPointer = unsafe extern "C" fn(*const c_void) -> *const c_void;

const IL2CPP_TYPE_I4: i32 = 0x08;
const IL2CPP_TYPE_R4: i32 = 0x0c;

unsafe fn sym<T: Copy>(name: &str) -> Option<T> {
    let p = super::resolve_il2cpp_symbol(name);
    if p.is_null() { None } else { Some(std::mem::transmute_copy(&p)) }
}

unsafe fn get_class(image: *const c_void, ns: &str, name: &str) -> *mut c_void {
    if super::API.is_null() {
        return std::ptr::null_mut();
    }
    let Some(f) = (*super::API).il2cpp_get_class_fn else { return std::ptr::null_mut() };
    let (ns, nm) = (super::to_cstr(ns), super::to_cstr(name));
    f(image, ns.as_ptr(), nm.as_ptr())
}

unsafe fn method_addr(class: *mut c_void, name: &str, nargs: i32) -> usize {
    if super::API.is_null() {
        return 0;
    }
    let Some(f) = (*super::API).il2cpp_get_method_addr_fn else { return 0 };
    let nm = super::to_cstr(name);
    f(class as usize, nm.as_ptr(), nargs)
}

// 一次取齐 6 个反射 API；任一缺失则返回 0，交给降级路径
struct Reflect {
    methods: GetMethods,
    name: GetName,
    params: ParamCount,
    ret: ReturnType,
    type_enum: TypeEnum,
    pointer: MethodPointer,
}

unsafe fn reflect() -> Option<Reflect> {
    Some(Reflect {
        methods: sym("il2cpp_class_get_methods")?,
        name: sym("il2cpp_method_get_name")?,
        params: sym("il2cpp_method_get_param_count")?,
        ret: sym("il2cpp_method_get_return_type")?,
        type_enum: sym("il2cpp_type_get_type")?,
        pointer: sym("il2cpp_method_get_pointer")?,
    })
}

unsafe fn method_pointer(r: &Reflect, mi: *const c_void) -> usize {
    let p = (r.pointer)(mi);
    if p.is_null() {
        std::ptr::read_unaligned(mi as *const usize) // 部分版本 methodPointer 在首字段
    } else {
        p as usize
    }
}

unsafe fn resolve_getrandom(rr: *mut c_void) -> i32 {
    let Some(r) = reflect() else { return 0 };
    let mut iter: *mut c_void = std::ptr::null_mut();
    let mut n = 0;
    for _ in 0..256 {
        let mi = (r.methods)(rr, &mut iter);
        if mi.is_null() {
            break;
        }
        let np = (r.name)(mi);
        if np.is_null() || CStr::from_ptr(np).to_bytes() != b"GetRandom" {
            continue;
        }
        let na = (r.params)(mi);
        if na != 1 && na != 2 {
            continue;
        }
        let rt = (r.ret)(mi);
        if rt.is_null() {
            continue;
        }
        let slot = match ((r.type_enum)(rt), na) {
            (IL2CPP_TYPE_I4, 1) => &ADDR_I4_1,
            (IL2CPP_TYPE_I4, 2) => &ADDR_I4_2,
            (IL2CPP_TYPE_R4, 1) => &ADDR_R4_1,
            (IL2CPP_TYPE_R4, 2) => &ADDR_R4_2,
            _ => continue,
        };
        let addr = method_pointer(&r, mi);
        if addr == 0 {
            continue;
        }
        slot.store(addr as u64, Relaxed);
        n += 1;
    }
    n
}

// 降级：反射缺失时只能按参数个数取址，i4/r4 不可分；status 会标 degraded
unsafe fn resolve_degraded(rr: *mut c_void) -> i32 {
    let a1 = method_addr(rr, "GetRandom", 1);
    let a2 = method_addr(rr, "GetRandom", 2);
    if a1 != 0 { ADDR_I4_1.store(a1 as u64, Relaxed); }
    if a2 != 0 { ADDR_I4_2.store(a2 as u64, Relaxed); }
    (a1 != 0) as i32 + (a2 != 0) as i32
}

unsafe fn resolve_rpa(ei: *mut c_void) -> i32 {
    let g = method_addr(ei, "get_RaceRandomProgramArray", 0);
    let s = method_addr(ei, "set_RaceRandomProgramArray", 1);
    if g != 0 { ADDR_RPA_GET.store(g as u64, Relaxed); }
    if s != 0 { ADDR_RPA_SET.store(s as u64, Relaxed); }
    (g != 0) as i32 + (s != 0) as i32
}

// SingleModeRaceRandomProgram 是数据容器，本轮只登记方法清单，不盲挂 getter
unsafe fn probe_smrrp(image: *const c_void) -> String {
    let cls = get_class(image, "Gallop", "SingleModeRaceRandomProgram");
    if cls.is_null() {
        SMRRP_FOUND.store(false, Relaxed);
        return r#"{"found":false,"reason":"class_not_in_image"}"#.into();
    }
    SMRRP_FOUND.store(true, Relaxed);
    let Some(r) = reflect() else {
        return r#"{"found":true,"methods":[],"reason":"no_reflection_api"}"#.into();
    };
    let mut iter: *mut c_void = std::ptr::null_mut();
    let mut items: Vec<String> = Vec::new();
    for _ in 0..128 {
        let mi = (r.methods)(cls, &mut iter);
        if mi.is_null() {
            break;
        }
        let np = (r.name)(mi);
        if np.is_null() {
            continue;
        }
        let name = CStr::from_ptr(np).to_string_lossy().into_owned();
        if !(name.starts_with("get_") || name.starts_with("set_")
            || name.contains("Random") || name.contains("Program")) {
            continue;
        }
        let rt = (r.ret)(mi);
        let te = if rt.is_null() { -1 } else { (r.type_enum)(rt) };
        let mut m = String::from("{");
        let _ = write!(m, r#""name":"{}","params":{},"ret_type":{}"#, name, (r.params)(mi), te);
        m.push('}');
        items.push(m);
    }
    let mut s = String::from("{");
    let _ = write!(s, r#""found":true,"count":{},"methods":["#, items.len());
    s.push_str(&items.join(","));
    s.push_str(r#"],"note":"data_container_no_direct_rng_entry; see pending_items"}"#);
    s
}

// ---- 安装：install 只解析元数据；toggle on 才懒装 detour -------------------

pub unsafe fn install() {
    if RESOLVE_MODE.load(Relaxed) != 0 || super::API.is_null() {
        return;
    }
    let image = super::get_image();
    if image.is_null() {
        return;
    }
    let rr = get_class(image, "StandaloneSimulator", "RaceRandom");
    let mut n = if rr.is_null() { 0 } else { resolve_getrandom(rr) };
    let mode = if n > 0 {
        1
    } else {
        n = if rr.is_null() { 0 } else { resolve_degraded(rr) };
        2
    };
    RESOLVE_MODE.store(mode, Relaxed);

    let ei = get_class(image, "Gallop", "ObscuredIdleSingleModeEndInfo");
    let nrpa = if ei.is_null() { 0 } else { resolve_rpa(ei) };
    let _ = SMRRP_JSON.set(probe_smrrp(image));

    let msg = format!(
        "rng_probe: GetRandom={} mode={} rpa={} smrrp={}",
        n, mode, nrpa, SMRRP_FOUND.load(Relaxed),
    );
    unsafe { super::ura_log(3, &msg) };
    super::set_hook_status("rng_probe.resolve", &msg);
}

unsafe fn install_hooks() -> i32 {
    if HOOKS_INSTALLED.load(std::sync::atomic::Ordering::Acquire) {
        return 6;
    }
    let pairs: [(u64, usize, &str); 6] = [
        (ADDR_I4_1.load(Relaxed), ha!(gr_i4_1), "GetRandom.i4.1"),
        (ADDR_I4_2.load(Relaxed), ha!(gr_i4_2), "GetRandom.i4.2"),
        (ADDR_R4_1.load(Relaxed), ha!(gr_r4_1), "GetRandom.r4.1"),
        (ADDR_R4_2.load(Relaxed), ha!(gr_r4_2), "GetRandom.r4.2"),
        (ADDR_RPA_GET.load(Relaxed), ha!(rpa_get), "RaceRandomProgramArray.get"),
        (ADDR_RPA_SET.load(Relaxed), ha!(rpa_set), "RaceRandomProgramArray.set"),
    ];
    let mut ok = 0;
    for (addr, hook, label) in pairs {
        let key = format!("rng_probe.{}", label);
        let status = if addr == 0 {
            "skipped: no_addr".to_string()
        } else if unsafe { super::interceptor_hook(addr as usize, hook) } {
            ok += 1;
            format!("hooked@0x{:x}", addr)
        } else {
            "failed: interceptor_hook".to_string()
        };
        super::set_hook_status(&key, &status);
    }
    if ok > 0 {
        HOOKS_INSTALLED.store(true, std::sync::atomic::Ordering::Release);
        let msg = format!("rng_probe: installed {}/6 detours", ok);
        unsafe { super::ura_log(3, &msg) };
    }
    ok
}

// ---- JSON 输出：格式串不含字面花括号，无需双花括号转义；外层括号直接 push -----

fn site_name(s: u8) -> &'static str {
    match s {
        SITE_I4_1 => "RaceRandom.GetRandom(i4,1)",
        SITE_I4_2 => "RaceRandom.GetRandom(i4,2)",
        SITE_R4_1 => "RaceRandom.GetRandom(r4,1)",
        SITE_R4_2 => "RaceRandom.GetRandom(r4,2)",
        SITE_RPA_GET => "get_RaceRandomProgramArray",
        SITE_RPA_SET => "set_RaceRandomProgramArray",
        SITE_CTX => "context_mark",
        _ => "unknown",
    }
}

fn kind_name(k: u8) -> &'static str {
    match k {
        KIND_I4 => "i4", KIND_R4 => "r4", KIND_PTR => "ptr", KIND_VOID => "void",
        _ => "unknown",
    }
}

fn json_i4(v: u64) -> String {
    (v as u32 as i32).to_string()
}

fn json_r4(v: u64) -> String {
    let x = f32::from_bits(v as u32);
    if x.is_finite() { format!("{:.6}", x) } else { "null".into() }
}

fn json_hex(v: u64) -> String {
    format!(r#""0x{:x}""#, v)
}

fn event_json(e: &Event) -> String {
    let (a0, a1, ret) = match e.kind {
        KIND_R4 => (json_r4(e.a0), json_r4(e.a1), json_r4(e.ret)),
        KIND_I4 => (json_i4(e.a0), json_i4(e.a1), json_i4(e.ret)),
        _ => (json_hex(e.a0), json_hex(e.a1), json_hex(e.ret)),
    };
    let mut s = String::from("{");
    let _ = write!(s,
        r#""seq":{},"t_us":{},"step":{},"turn":{},"action":{},"tid":{}"#,
        e.seq, e.t_us, e.step, e.turn, e.action, e.tid);
    let _ = write!(s, r#","site":"{}","kind":"{}""#, site_name(e.site), kind_name(e.kind));
    let _ = write!(s, r#","args":[{},{}],"value":{}"#, a0, a1, ret);
    s.push('}');
    s
}

const NOT_RESOLVED: &str = r#"{"found":false,"reason":"not_resolved_yet"}"#;
const SEED_INFO: &str = r#","fixedTurnCharaSeed":{"deprecated":true,"reason":"offset_0x198_is_ObscuredInt_not_u32x4","is_prng_state":false,"single_value_replay":false,"superseded_by":"/api/rng/consumption"}"#;
const PENDING: &str = r#"["G1 真机确认 il2cpp_method_get_return_type / il2cpp_type_get_type 是否导出","G2 确认 GetRandom 四重载的真实参数语义与浮点传参站位","G3 从安全读路径取 RaceRandomProgramArray 长度与元素布局","G4 定位 SingleModeRaceRandomProgram 真实消费入口后再挂 detour","G5 turn / command_id 自动绑定时间线（当前靠 /context 人工标注）","G6 完整一局流水回放校验：不丢数、不改游戏随机行为、无 SIGSEGV"]"#;

/// GET /api/rng/consumption?after=<seq>&limit=<n>
pub fn endpoint(uri: &str) -> String {
    let after = super::parse_query(uri, "after").parse::<u64>().unwrap_or(0);
    let limit = super::parse_query(uri, "limit").parse::<usize>().unwrap_or(256).min(CAP);
    let total = WRITE_SEQ.load(Relaxed);
    let base = core::ptr::addr_of_mut!(EVENTS).cast::<Event>();
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
    let joined = out.iter().map(|x| x.1.as_str()).collect::<Vec<_>>().join(",");

    let mut s = String::from("{");
    let _ = write!(s,
        r#""ok":true,"version":"{}","direction":"consumption_probe_not_seed_replay","enabled":{},"hooks_installed":{},"total_captured":{},"cap":{},"after":{},"count":{}"#,
        super::PLUGIN_VERSION, ENABLED.load(Relaxed), HOOKS_INSTALLED.load(Relaxed),
        total, CAP, after, out.len());
    s.push_str(r#","events":["#);
    s.push_str(&joined);
    s.push_str("]}");
    s
}

/// GET /api/rng/consumption/status
pub fn status_endpoint() -> String {
    let mode = RESOLVE_MODE.load(Relaxed);
    let mode_name = match mode {
        1 => "reflection_exact", 2 => "degraded_by_param_count", _ => "not_resolved",
    };
    let smrrp = SMRRP_JSON.get().map(String::as_str).unwrap_or(NOT_RESOLVED);
    let mut s = String::from("{");
    let _ = write!(s,
        r#""ok":true,"version":"{}","enabled":{},"hooks_installed":{},"resolve_mode":{},"resolve_mode_name":"{}","degraded":{},"cap":{},"total_captured":{}"#,
        super::PLUGIN_VERSION, ENABLED.load(Relaxed), HOOKS_INSTALLED.load(Relaxed),
        mode, mode_name, mode == 2, CAP, WRITE_SEQ.load(Relaxed));
    s.push_str(r#","addresses":{"#);
    let _ = write!(s,
        r#""GetRandom.i4.1":"0x{:x}","GetRandom.i4.2":"0x{:x}","GetRandom.r4.1":"0x{:x}","GetRandom.r4.2":"0x{:x}","RaceRandomProgramArray.get":"0x{:x}","RaceRandomProgramArray.set":"0x{:x}""#,
        ADDR_I4_1.load(Relaxed), ADDR_I4_2.load(Relaxed), ADDR_R4_1.load(Relaxed),
        ADDR_R4_2.load(Relaxed), ADDR_RPA_GET.load(Relaxed), ADDR_RPA_SET.load(Relaxed));
    s.push('}');
    s.push_str(r#","SingleModeRaceRandomProgram":"#);
    s.push_str(smrrp);
    s.push_str(SEED_INFO);
    s.push_str(r#","pending_items":"#);
    s.push_str(PENDING);
    s.push('}');
    s
}

/// GET /api/rng/consumption/toggle?enabled=1|0
pub fn toggle_endpoint(uri: &str) -> String {
    let want = match super::parse_query(uri, "enabled").as_str() {
        "1" | "true" | "on" => true,
        "0" | "false" | "off" => false,
        _ => !ENABLED.load(Relaxed),
    };
    if !want {
        ENABLED.store(false, std::sync::atomic::Ordering::Release);
        return r#"{"ok":true,"enabled":false,"hint":"记录已停止；detour 仍在但 hook 内直接跳过"}"#.into();
    }
    unsafe { install(); }
    let n = unsafe { install_hooks() };
    ENABLED.store(true, std::sync::atomic::Ordering::Release);
    let mut s = String::from("{");
    let _ = write!(s,
        r#""ok":true,"enabled":true,"detours_installed":{},"hint":"高频 hook 已开启；观测完请 enabled=0""#,
        n);
    s.push('}');
    s
}

/// GET /api/rng/consumption/clear
pub fn clear_endpoint() -> String {
    WRITE_SEQ.store(0, std::sync::atomic::Ordering::Release);
    unsafe {
        let base = core::ptr::addr_of_mut!(EVENTS).cast::<Event>();
        for i in 0..CAP {
            core::ptr::write_volatile(base.add(i), Event::ZERO);
        }
    }
    r#"{"ok":true,"cleared":true}"#.into()
}

/// GET /api/rng/consumption/context?turn=<n>&action=<m>
pub fn context_endpoint(uri: &str) -> String {
    let t = super::parse_query(uri, "turn").parse::<i32>().unwrap_or(-1);
    let a = super::parse_query(uri, "action").parse::<i32>().unwrap_or(-1);
    CTX_TURN.store(t, Relaxed);
    CTX_ACTION.store(a, Relaxed);
    // 临时借用开关打一条标记，随后恢复用户设定
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
        let prev = ENABLED.swap(true, std::sync::atomic::Ordering::AcqRel);
        record(SITE_CTX, KIND_VOID, t as u32 as u64, a as u32 as u64, 0);
        ENABLED.store(prev, std::sync::atomic::Ordering::Release);
    }));
    let mut s = String::from("{");
    let _ = write!(s, r#""ok":true,"turn":{},"action":{},"mark_written":true"#, t, a);
    s.push('}');
    s
}
