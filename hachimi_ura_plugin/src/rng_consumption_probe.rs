//! RNG 消费点插桩 / RNG consumption-point probe  —  v3.29.1
//!
//! ============================================================
//! 方向变更（2026-11-06）：老账改道，不再猜种子
//! ============================================================
//! 旧路线（已证伪，2026-07）：`WorkSingleModeData._fixedTurnCharaSeed`
//!   @ offset 0x198 是 `CodeStage.AntiCheat.ObscuredTypes.ObscuredInt`，
//!   即一个加密 int。它 **不是 PRNG 状态**（不是 u32x4 / xorshift state），
//!   单值无法重放随机流。`/seed/history`、`/debug/training_seed` 一类端点
//!   因此被标记 deprecated（rng_invalid_reason =
//!   "offset_0x198_is_ObscuredInt_not_u32x4"）。
//!
//! 新路线：不猜种子、不重放，只做 **消费点观测**，回答
//!   「什么时候、在哪个消费点、用了什么随机数」。
//! 直接 hook 三个目标：
//!   A. StandaloneSimulator.RaceRandom.GetRandom
//!      —— 4 个重载：i4(1参/2参)、r4(1参/2参)
//!   B. Gallop.ObscuredIdleSingleModeEndInfo
//!        .get_RaceRandomProgramArray / .set_RaceRandomProgramArray
//!      —— 预计算随机程序数组的读取 / 写入时序
//!   C. Gallop.SingleModeRaceRandomProgram
//!      —— 程序项数据类：本轮做类解析 + 方法清单登记（真实消费入口
//!         需真机 dump 后补 hook，见 status 端点 pending_items）
//!
//! ============================================================
//! 性能与安全纪律（针对「轮询过多导致游戏卡死」的老问题）
//! ============================================================
//! 1. **默认不安装 hook**。GetRandom 是比赛/训练模拟里的高频调用，
//!    默认零开销；只有 GET /api/rng/consumption/toggle?enabled=1
//!    才懒安装 detour。装好后不可卸载（interceptor 无 unhook），
//!    但可用 ENABLED 原子开关立刻停止记录。
//! 2. hook 体内 **不做 IL2CPP invoke、不读游戏对象内存、不分配、
//!    不做字符串格式化**，只写一块固定容量的 POD 环形缓冲。
//! 3. 原函数通过 interceptor trampoline 调用，返回值原样透传；
//!    i4 用 usize detour、r4 用 f32 detour —— AAPCS64 下整数返回走 x0、
//!    单精度返回走 v0/s0，混用会破坏游戏随机值（v3.28.11 的教训）。
//! 4. 记录代码用 catch_unwind 包住，且放在 trampoline 调用之后：
//!    即使记录 panic，也不影响原函数返回值、不跨 FFI 边界。
//! 5. 环形缓冲 CAP=4096，覆盖写；读端带 seq/slot 一致性校验，
//!    避免撕裂读。内存恒定，不会随游戏时长增长。
//!
//! 端点：
//!   GET /api/rng/consumption             ?after=<seq>&limit=<n>  流水
//!   GET /api/rng/consumption/status                              安装/解析状态
//!   GET /api/rng/consumption/toggle      ?enabled=1|0             开关(懒安装)
//!   GET /api/rng/consumption/clear                               清空
//!   GET /api/rng/consumption/context     ?turn=<n>&action=<m>     标注上下文
//!
//! 依赖：仅 super::{API, to_cstr, resolve_il2cpp_symbol, get_image,
//!   interceptor_hook, interceptor_get_trampoline, set_hook_status,
//!   ura_log, PREDICT_STEP, parse_query} + libc::gettid。

use std::ffi::{c_char, c_void, CStr};
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU64, Ordering};
use std::sync::OnceLock;
use std::time::Instant;

// ------------------------------------------------------------
// 常量与状态
// ------------------------------------------------------------

/// 环形缓冲容量，必须是 2 的幂（用 & (CAP-1) 取模）。
pub const CAP: usize = 4096;

/// 消费点编号。端点输出时映射成可读名。
pub const SITE_GR_I4_1: u8 = 1;
pub const SITE_GR_I4_2: u8 = 2;
pub const SITE_GR_R4_1: u8 = 3;
pub const SITE_GR_R4_2: u8 = 4;
pub const SITE_RPA_GET: u8 = 5;
pub const SITE_RPA_SET: u8 = 6;
pub const SITE_CTX_MARK: u8 = 7;

/// 返回值种类。
pub const KIND_I4: u8 = 1;
pub const KIND_R4: u8 = 2;
pub const KIND_PTR: u8 = 3;
pub const KIND_VOID: u8 = 4;

/// i4 GetRandom(min,max) 的 1 参形式通常只传上界；i32 位模式存 a0。
/// 所有字段都是定长整数，避免 hook 内出现浮点格式化/分配。
#[repr(C)]
#[derive(Copy, Clone)]
struct Event {
    seq: u64,
    t_us: u64,    // 进程启动以来的单调微秒 —— 「什么时候」
    step: u32,    // super::PREDICT_STEP 快照（回合/推进上下文，尽力而为）
    turn: i32,    // 由 /context 端点人工标注，默认 -1 = 未知
    action: i32,  // 同上，默认 -1 = 未知
    tid: u32,
    site: u8,     // 「在哪个消费点」
    kind: u8,     // 返回值种类
    a0: u64,      // 参数 1 的原始位（i32 零扩展 / f32.to_bits / 指针）
    a1: u64,      // 参数 2 的原始位
    ret: u64,     // 「用了什么随机数」：返回值原始位
}
impl Event {
    const ZERO: Event = Event {
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

// 每个目标一个地址槽（status 端点回报，便于真机核对）
static ADDR_GR_I4_1: AtomicU64 = AtomicU64::new(0);
static ADDR_GR_I4_2: AtomicU64 = AtomicU64::new(0);
static ADDR_GR_R4_1: AtomicU64 = AtomicU64::new(0);
static ADDR_GR_R4_2: AtomicU64 = AtomicU64::new(0);
static ADDR_RPA_GET: AtomicU64 = AtomicU64::new(0);
static ADDR_RPA_SET: AtomicU64 = AtomicU64::new(0);
static CLASS_SMRRP_FOUND: AtomicBool = AtomicBool::new(false);
static RESOLVE_MODE: AtomicI32 = AtomicI32::new(0); // 0=未解析 1=反射精确 2=降级

/// 进程内单调时钟（微秒）。OnceLock 无分配、无锁竞争。
#[inline(always)]
fn now_us() -> u64 {
    BASE.get_or_init(Instant::now).elapsed().as_micros() as u64
}

#[inline(always)]
fn tid() -> u32 {
    unsafe { libc::gettid() as u32 }
}

/// 记录一条消费事件。只在 ENABLED 时生效；纯 POD 写，无分配。
/// 用 addr_of_mut! + write_volatile，避免 static_mut_refs lint 与引用别名。
#[inline(always)]
unsafe fn record(site: u8, kind: u8, a0: u64, a1: u64, ret: u64) {
    if !ENABLED.load(Ordering::Relaxed) {
        return;
    }
    let s = WRITE_SEQ.fetch_add(1, Ordering::Relaxed) + 1;
    let idx = (s as usize) & (CAP - 1);
    let e = Event {
        seq: s,
        t_us: now_us(),
        step: super::PREDICT_STEP.load(Ordering::Relaxed),
        turn: CTX_TURN.load(Ordering::Relaxed),
        action: CTX_ACTION.load(Ordering::Relaxed),
        tid: tid(),
        site,
        kind,
        a0,
        a1,
        ret,
    };
    let slot = core::ptr::addr_of_mut!(EVENTS).cast::<Event>().add(idx);
    core::ptr::write_volatile(slot, e);
}

/// 把「记录」这一侧与 FFI 隔开：panic 不影响原函数返回值、不跨边界。
#[inline(always)]
fn safe_record(site: u8, kind: u8, a0: u64, a1: u64, ret: u64) {
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
        record(site, kind, a0, a1, ret);
    }));
}

// ------------------------------------------------------------
// trampoline 调用助手
// ------------------------------------------------------------
// 每个 detour 必须用 **自己的函数地址** 去查 trampoline：
// interceptor_get_trampoline_addr_fn 以 hook 地址为键，多个目标共用一个
// handler 会串线。因此下面 6 个 extern "C" fn 一一对应 6 个目标。

type FnI4_1 = unsafe extern "C" fn(*mut c_void, usize) -> usize;
type FnI4_2 = unsafe extern "C" fn(*mut c_void, usize, usize) -> usize;
type FnR4_1 = unsafe extern "C" fn(*mut c_void, f32) -> f32;
type FnR4_2 = unsafe extern "C" fn(*mut c_void, f32, f32) -> f32;
type FnPtrGet = unsafe extern "C" fn(*mut c_void) -> usize;
type FnPtrSet = unsafe extern "C" fn(*mut c_void, usize);

#[inline(always)]
unsafe fn trampoline(hook_addr: usize) -> usize {
    super::interceptor_get_trampoline(hook_addr)
}

// ---- A. RaceRandom.GetRandom 四个重载 ----

extern "C" fn gr_i4_1(this: *mut c_void, a1: usize) -> usize {
    let t = unsafe { trampoline(gr_i4_1 as *const () as usize) };
    if t == 0 {
        return 0;
    }
    let original: FnI4_1 = unsafe { std::mem::transmute(t) };
    let r = unsafe { original(this, a1) };
    safe_record(SITE_GR_I4_1, KIND_I4, a1 as u64, 0, r as u64);
    r
}

extern "C" fn gr_i4_2(this: *mut c_void, a1: usize, a2: usize) -> usize {
    let t = unsafe { trampoline(gr_i4_2 as *const () as usize) };
    if t == 0 {
        return 0;
    }
    let original: FnI4_2 = unsafe { std::mem::transmute(t) };
    let r = unsafe { original(this, a1, a2) };
    safe_record(SITE_GR_I4_2, KIND_I4, a1 as u64, a2 as u64, r as u64);
    r
}

extern "C" fn gr_r4_1(this: *mut c_void, a1: f32) -> f32 {
    let t = unsafe { trampoline(gr_r4_1 as *const () as usize) };
    if t == 0 {
        return 0.0;
    }
    let original: FnR4_1 = unsafe { std::mem::transmute(t) };
    let r = unsafe { original(this, a1) };
    safe_record(SITE_GR_R4_1, KIND_R4, a1.to_bits() as u64, 0, r.to_bits() as u64);
    r
}

extern "C" fn gr_r4_2(this: *mut c_void, a1: f32, a2: f32) -> f32 {
    let t = unsafe { trampoline(gr_r4_2 as *const () as usize) };
    if t == 0 {
        return 0.0;
    }
    let original: FnR4_2 = unsafe { std::mem::transmute(t) };
    let r = unsafe { original(this, a1, a2) };
    safe_record(
        SITE_GR_R4_2, KIND_R4,
        a1.to_bits() as u64, a2.to_bits() as u64, r.to_bits() as u64,
    );
    r
}

// ---- B. RaceRandomProgramArray getter / setter ----
// getter 返回 fnptr（数组对象指针）：只记指针值，**不解引用**，
// 避免在 hook 里读游戏内存触发 SIGSEGV。数组长度/元素内容留给真机项。

extern "C" fn rpa_get(this: *mut c_void) -> usize {
    let t = unsafe { trampoline(rpa_get as *const () as usize) };
    if t == 0 {
        return 0;
    }
    let original: FnPtrGet = unsafe { std::mem::transmute(t) };
    let r = unsafe { original(this) };
    safe_record(SITE_RPA_GET, KIND_PTR, 0, 0, r as u64);
    r
}

extern "C" fn rpa_set(this: *mut c_void, value: usize) {
    let t = unsafe { trampoline(rpa_set as *const () as usize) };
    if t != 0 {
        let original: FnPtrSet = unsafe { std::mem::transmute(t) };
        unsafe { original(this, value) };
    }
    safe_record(SITE_RPA_SET, KIND_VOID, value as u64, 0, 0);
}

// ------------------------------------------------------------
// IL2CPP 反射：精确区分 GetRandom 的 i4 / r4 重载
// ------------------------------------------------------------
// il2cpp_get_method_addr(class, name, param_count) 只能按参数个数取址，
// 而 i4 与 r4 各有 1 参 / 2 参重载 —— 同名同参个数，必须靠返回类型区分。
// 用标准 il2cpp C API 动态解析（resolve_il2cpp_symbol），不依赖硬编码偏移。

type FnGetMethods = unsafe extern "C" fn(*mut c_void, *mut *mut c_void) -> *const c_void;
type FnMethodGetName = unsafe extern "C" fn(*const c_void) -> *const c_char;
type FnParamCount = unsafe extern "C" fn(*const c_void) -> u32;
type FnReturnType = unsafe extern "C" fn(*const c_void) -> *const c_void;
type FnTypeGetType = unsafe extern "C" fn(*const c_void) -> i32;
type FnMethodGetPointer = unsafe extern "C" fn(*const c_void) -> *const c_void;

// Il2CppTypeEnum（libil2cpp 标准定义）
const IL2CPP_TYPE_I4: i32 = 0x08;
const IL2CPP_TYPE_R4: i32 = 0x0c;

unsafe fn sym<T: Copy>(name: &str) -> Option<T> {
    let p = super::resolve_il2cpp_symbol(name);
    if p.is_null() {
        None
    } else {
        Some(std::mem::transmute_copy::<*mut c_void, T>(&p))
    }
}

unsafe fn get_class(image: *const c_void, ns: &str, name: &str) -> *mut c_void {
    if super::API.is_null() {
        return std::ptr::null_mut();
    }
    let f = match (*super::API).il2cpp_get_class_fn {
        Some(f) => f,
        None => return std::ptr::null_mut(),
    };
    let c_ns = super::to_cstr(ns);
    let c_nm = super::to_cstr(name);
    f(image, c_ns.as_ptr(), c_nm.as_ptr())
}

/// 遍历类方法，按 (返回类型, 参数个数) 把 4 个 GetRandom 重载落到各自地址槽。
/// 返回成功解析出的重载数（0..=4）。
unsafe fn resolve_getrandom_overloads(race_random: *mut c_void) -> i32 {
    let get_methods: FnGetMethods = match sym("il2cpp_class_get_methods") {
        Some(f) => f,
        None => return 0,
    };
    let get_name: FnMethodGetName = match sym("il2cpp_method_get_name") {
        Some(f) => f,
        None => return 0,
    };
    let param_count: FnParamCount = match sym("il2cpp_method_get_param_count") {
        Some(f) => f,
        None => return 0,
    };
    let return_type: FnReturnType = match sym("il2cpp_method_get_return_type") {
        Some(f) => f,
        None => return 0,
    };
    let type_get_type: FnTypeGetType = match sym("il2cpp_type_get_type") {
        Some(f) => f,
        None => return 0,
    };
    let method_pointer: FnMethodGetPointer = match sym("il2cpp_method_get_pointer") {
        Some(f) => f,
        None => return 0,
    };

    let mut iter: *mut c_void = std::ptr::null_mut();
    let mut found = 0i32;
    for _ in 0..256 {
        let mi = get_methods(race_random, &mut iter);
        if mi.is_null() {
            break;
        }
        let np = get_name(mi);
        if np.is_null() || CStr::from_ptr(np).to_bytes() != b"GetRandom" {
            continue;
        }
        let nargs = param_count(mi);
        if nargs != 1 && nargs != 2 {
            continue;
        }
        let rt = return_type(mi);
        if rt.is_null() {
            continue;
        }
        let tenum = type_get_type(rt);
        let ptr = method_pointer(mi);
        let addr = if ptr.is_null() {
            // 少数版本 methodPointer 在 MethodInfo 首字段；保守读取
            std::ptr::read_unaligned::<usize>(mi as *const usize) as usize
        } else {
            ptr as usize
        };
        if addr == 0 {
            continue;
        }
        let slot: &AtomicU64 = match (tenum, nargs) {
            (IL2CPP_TYPE_I4, 1) => &ADDR_GR_I4_1,
            (IL2CPP_TYPE_I4, 2) => &ADDR_GR_I4_2,
            (IL2CPP_TYPE_R4, 1) => &ADDR_GR_R4_1,
            (IL2CPP_TYPE_R4, 2) => &ADDR_GR_R4_2,
            _ => continue,
        };
        slot.store(addr as u64, Ordering::Relaxed);
        found += 1;
    }
    found
}

/// 降级路径：反射 API 缺失时，只能按参数个数取址。
/// i4/r4 无法区分 —— status 端点会明确标 degraded=true，绝不假装精确。
unsafe fn resolve_getrandom_degraded(race_random: *mut c_void) -> i32 {
    if super::API.is_null() {
        return 0;
    }
    let f = match (*super::API).il2cpp_get_method_addr_fn {
        Some(f) => f,
        None => return 0,
    };
    let nm = super::to_cstr("GetRandom");
    let a1 = f(race_random as usize, nm.as_ptr(), 1);
    let a2 = f(race_random as usize, nm.as_ptr(), 2);
    let mut n = 0;
    if a1 != 0 {
        ADDR_GR_I4_1.store(a1 as u64, Ordering::Relaxed);
        n += 1;
    }
    if a2 != 0 {
        ADDR_GR_I4_2.store(a2 as u64, Ordering::Relaxed);
        n += 1;
    }
    n
}

unsafe fn resolve_rpa(end_info: *mut c_void) -> i32 {
    if super::API.is_null() {
        return 0;
    }
    let f = match (*super::API).il2cpp_get_method_addr_fn {
        Some(f) => f,
        None => return 0,
    };
    let g = super::to_cstr("get_RaceRandomProgramArray");
    let s = super::to_cstr("set_RaceRandomProgramArray");
    let ga = f(end_info as usize, g.as_ptr(), 0);
    let sa = f(end_info as usize, s.as_ptr(), 1);
    let mut n = 0;
    if ga != 0 {
        ADDR_RPA_GET.store(ga as u64, Ordering::Relaxed);
        n += 1;
    }
    if sa != 0 {
        ADDR_RPA_SET.store(sa as u64, Ordering::Relaxed);
        n += 1;
    }
    n
}

/// 目标 C：SingleModeRaceRandomProgram 是 MsgPack 反序列化的数据容器类，
/// 没有独立的「取随机数」入口。本轮只确认类是否存在、并枚举方法清单
/// 作为下一步真机定位消费点的依据；不盲目 hook 全部 getter（ABI 未知，
/// 乱挂会破坏 v0/x0 返回值）。清单在 status 端点输出。
unsafe fn probe_smrrp_methods(image: *const c_void) -> String {
    let cls = get_class(image, "Gallop", "SingleModeRaceRandomProgram");
    if cls.is_null() {
        CLASS_SMRRP_FOUND.store(false, Ordering::Relaxed);
        return r#"{"found":false,"reason":"class_not_in_image"}"#.to_string();
    }
    CLASS_SMRRP_FOUND.store(true, Ordering::Relaxed);
    let get_methods: FnGetMethods = match sym("il2cpp_class_get_methods") {
        Some(f) => f,
        None => return r#"{"found":true,"methods":[],"reason":"no_get_methods_api"}"#.to_string(),
    };
    let get_name: FnMethodGetName = match sym("il2cpp_method_get_name") {
        Some(f) => f,
        None => return r#"{"found":true,"methods":[],"reason":"no_get_name_api"}"#.to_string(),
    };
    let param_count: FnParamCount = match sym("il2cpp_method_get_param_count") {
        Some(f) => f,
        None => return r#"{"found":true,"methods":[],"reason":"no_param_count_api"}"#.to_string(),
    };
    let return_type: Option<FnReturnType> = sym("il2cpp_method_get_return_type");
    let type_get_type: Option<FnTypeGetType> = sym("il2cpp_type_get_type");
    let mut iter: *mut c_void = std::ptr::null_mut();
    let mut items: Vec<String> = Vec::new();
    for _ in 0..128 {
        let mi = get_methods(cls, &mut iter);
        if mi.is_null() {
            break;
        }
        let np = get_name(mi);
        if np.is_null() {
            continue;
        }
        let name = CStr::from_ptr(np).to_string_lossy().into_owned();
        // 噪音控制：只列 get_/set_/含 Random/Program 的方法
        if !(name.starts_with("get_") || name.starts_with("set_")
            || name.contains("Random") || name.contains("Program"))
        {
            continue;
        }
        let nargs = param_count(mi);
        let tenum = match (return_type, type_get_type) {
            (Some(rt), Some(tg)) => {
                let p = rt(mi);
                if p.is_null() { -1 } else { tg(p) }
            }
            _ => -1,
        };
        items.push(format!(
            r#"{{"name":"{}","params":{},"ret_type":{}}}"#, name, nargs, tenum
        ));
    }
    format!(
        r#"{{"found":true,"count":{},"methods":[{}],"note":"data_container_no_direct_rng_entry; see pending_items"}}"#,
        items.len(), items.join(",")
    )
}

static SMRRP_JSON: OnceLock<String> = OnceLock::new();

// ------------------------------------------------------------
// 安装（两段式：install 只做元数据；enable 才装 hook）
// ------------------------------------------------------------

/// 在 on_game_initialized / 懒重试路径调用。只解析类与方法、不装 detour，
/// 因此默认状态对游戏零影响。幂等。
pub unsafe fn install() {
    if RESOLVE_MODE.load(Ordering::Relaxed) != 0 {
        return;
    }
    if super::API.is_null() {
        return;
    }
    let image = super::get_image();
    if image.is_null() {
        return;
    }
    let rr = get_class(image, "StandaloneSimulator", "RaceRandom");
    let mut n = 0;
    if !rr.is_null() {
        n = resolve_getrandom_overloads(rr);
    }
    if n == 0 {
        // 反射不可用或解析失败 → 降级，明确标注
        if !rr.is_null() {
            n = resolve_getrandom_degraded(rr);
        }
        RESOLVE_MODE.store(2, Ordering::Relaxed);
    } else {
        RESOLVE_MODE.store(1, Ordering::Relaxed);
    }

    let ei = get_class(image, "Gallop", "ObscuredIdleSingleModeEndInfo");
    let nrpa = if ei.is_null() { 0 } else { resolve_rpa(ei) };

    let smrrp = probe_smrrp_methods(image);
    let _ = SMRRP_JSON.set(smrrp);

    let msg = format!(
        "rng_probe: resolved GetRandom={} mode={} rpa={} smrrp={}",
        n, RESOLVE_MODE.load(Ordering::Relaxed), nrpa,
        CLASS_SMRRP_FOUND.load(Ordering::Relaxed),
    );
    ura_log_safe(3, &msg);
    super::set_hook_status("rng_probe.resolve", &msg);
}

fn ura_log_safe(level: i32, msg: &str) {
    unsafe {
        super::ura_log(level, msg);
    }
}

/// 懒安装 6 个 detour。只在用户显式 toggle on 时调用一次。
/// 返回成功挂上的数量。
unsafe fn install_hooks() -> i32 {
    if HOOKS_INSTALLED.load(Ordering::Acquire) {
        return 6;
    }
    if super::API.is_null() {
        return 0;
    }
    let pairs: [(u64, usize, &str); 6] = [
        (ADDR_GR_I4_1.load(Ordering::Relaxed), gr_i4_1 as *const () as usize, "GetRandom.i4.1"),
        (ADDR_GR_I4_2.load(Ordering::Relaxed), gr_i4_2 as *const () as usize, "GetRandom.i4.2"),
        (ADDR_GR_R4_1.load(Ordering::Relaxed), gr_r4_1 as *const () as usize, "GetRandom.r4.1"),
        (ADDR_GR_R4_2.load(Ordering::Relaxed), gr_r4_2 as *const () as usize, "GetRandom.r4.2"),
        (ADDR_RPA_GET.load(Ordering::Relaxed), rpa_get as *const () as usize, "RaceRandomProgramArray.get"),
        (ADDR_RPA_SET.load(Ordering::Relaxed), rpa_set as *const () as usize, "RaceRandomProgramArray.set"),
    ];
    let mut ok = 0;
    for (addr, hook, label) in pairs.iter() {
        if *addr == 0 {
            super::set_hook_status(&format!("rng_probe.{}", label), "skipped: no_addr");
            continue;
        }
        if super::interceptor_hook(*addr as usize, *hook) {
            ok += 1;
            super::set_hook_status(&format!("rng_probe.{}", label),
                                  &format!("hooked@0x{:x}", *addr));
        } else {
            super::set_hook_status(&format!("rng_probe.{}", label), "failed: interceptor_hook");
        }
    }
    if ok > 0 {
        HOOKS_INSTALLED.store(true, Ordering::Release);
        ura_log_safe(3, &format!("rng_probe: installed {}/6 detours", ok));
    }
    ok
}

// ------------------------------------------------------------
// HTTP 端点
// ------------------------------------------------------------

fn site_name(s: u8) -> &'static str {
    match s {
        SITE_GR_I4_1 => "RaceRandom.GetRandom(i4,1)",
        SITE_GR_I4_2 => "RaceRandom.GetRandom(i4,2)",
        SITE_GR_R4_1 => "RaceRandom.GetRandom(r4,1)",
        SITE_GR_R4_2 => "RaceRandom.GetRandom(r4,2)",
        SITE_RPA_GET => "get_RaceRandomProgramArray",
        SITE_RPA_SET => "set_RaceRandomProgramArray",
        SITE_CTX_MARK => "context_mark",
        _ => "unknown",
    }
}

fn kind_name(k: u8) -> &'static str {
    match k {
        KIND_I4 => "i4",
        KIND_R4 => "r4",
        KIND_PTR => "ptr",
        KIND_VOID => "void",
        _ => "unknown",
    }
}

/// GET /api/rng/consumption?after=<seq>&limit=<n>
/// 输出按 seq 递增；每条含「什么时候(t_us/step/turn)」「哪个消费点(site)」
/// 「用了什么随机数(ret + 类型)」「参数(args)」。
pub fn endpoint(uri: &str) -> String {
    let after = super::parse_query(uri, "after")
        .parse::<u64>().unwrap_or(0);
    let limit = super::parse_query(uri, "limit")
        .parse::<usize>().unwrap_or(256).min(CAP);
    let total = WRITE_SEQ.load(Ordering::Relaxed);
    let base = core::ptr::addr_of_mut!(EVENTS).cast::<Event>();
    let mut out: Vec<(u64, String)> = Vec::new();
    unsafe {
        for idx in 0..CAP {
            let e = core::ptr::read_volatile(base.add(idx));
            // slot 一致性：只接受仍属于本槽且未被覆盖的记录
            if e.seq == 0 || e.seq <= after { continue; }
            if ((e.seq as usize) & (CAP - 1)) != idx { continue; }
            // r4 的位模式还原成可读浮点；i4 保留有符号
            let (a0s, a1s, rets) = if e.kind == KIND_R4 {
                (
                    format!("{:.6}", f32::from_bits(e.a0 as u32)),
                    format!("{:.6}", f32::from_bits(e.a1 as u32)),
                    format!("{:.6}", f32::from_bits(e.ret as u32)),
                )
            } else if e.kind == KIND_I4 {
                (
                    (e.a0 as u32 as i32).to_string(),
                    (e.a1 as u32 as i32).to_string(),
                    (e.ret as u32 as i32).to_string(),
                )
            } else {
                (format!("0x{:x}", e.a0), format!("0x{:x}", e.a1), format!("0x{:x}", e.ret))
            };
            out.push((e.seq, format!(
                r#"{{"seq":{},"t_us":{},"step":{},"turn":{},"action":{},"tid":{},"site":"{}","kind":"{}","args":[{},{}],"value":{}}}"#,
                e.seq, e.t_us, e.step, e.turn, e.action, e.tid,
                site_name(e.site), kind_name(e.kind), a0s, a1s, rets
            )));
        }
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    if out.len() > limit {
        out.drain(0..out.len() - limit);
    }
    let joined = out.iter().map(|x| x.1.as_str()).collect::<Vec<_>>().join(",");
    format!(
        r#"{{"ok":true,"version":"{}","direction":"consumption_probe_not_seed_replay","enabled":{},"hooks_installed":{},"total_captured":{},"cap":{},"after":{},"count":{},"events":[{}]}}"#,
        super::PLUGIN_VERSION,
        ENABLED.load(Ordering::Relaxed),
        HOOKS_INSTALLED.load(Ordering::Relaxed),
        total, CAP, after, out.len(), joined
    )
}

/// GET /api/rng/consumption/status
pub fn status_endpoint() -> String {
    let mode = RESOLVE_MODE.load(Ordering::Relaxed);
    let smrrp = SMRRP_JSON.get().cloned()
        .unwrap_or_else(|| r#"{"found":false,"reason":"not_resolved_yet"}"#.to_string());
    let pending = r#"[
      {"id":"G1","item":"真机确认 il2cpp_method_get_return_type / il2cpp_type_get_type 在本游戏 libil2cpp 导出；若缺失则维持 degraded，勿假装精确区分 i4/r4"},
      {"id":"G2","item":"确认 RaceRandom.GetRandom 4 个重载的真实参数语义（min,max / max）与 AAPCS64 浮点传参站位"},
      {"id":"G3","item":"get_RaceRandomProgramArray 返回的数组长度与元素布局（hook 内不解引用，需安全读路径 /process_vm_readv）"},
      {"id":"G4","item":"SingleModeRaceRandomProgram 类内真实消费入口方法定位后再挂 detour；本轮只登记方法清单"},
      {"id":"G5","item":"把 turn / command_id 自动绑定到时间线（当前靠 /context 人工标注 + PREDICT_STEP 快照）"},
      {"id":"G6","item":"一局完整育成 + 比赛的端到端流水回放校验，确认不丢数、不改游戏随机行为、无 SIGSEGV"},
      {"id":"G7","item":"CI 用 NDK r26c 重编译 aarch64 .so；本地无 NDK，源码与 3.29.0 基线二进制必须重新对齐"}
    ]"#;
    format!(
        r#"{{"ok":true,"version":"{}","enabled":{},"hooks_installed":{},"resolve_mode":{},"resolve_mode_name":"{}","degraded":{},"cap":{},"total_captured":{},"addresses":{{"GetRandom.i4.1":"0x{:x}","GetRandom.i4.2":"0x{:x}","GetRandom.r4.1":"0x{:x}","GetRandom.r4.2":"0x{:x}","RaceRandomProgramArray.get":"0x{:x}","RaceRandomProgramArray.set":"0x{:x}"}},"SingleModeRaceRandomProgram":{},"fixedTurnCharaSeed":{{"deprecated":true,"reason":"offset_0x198_is_ObscuredInt_not_u32x4","is_prng_state":false,"single_value_replay":false}},"pending_items":{}}}"#,
        super::PLUGIN_VERSION,
        ENABLED.load(Ordering::Relaxed),
        HOOKS_INSTALLED.load(Ordering::Relaxed),
        mode,
        if mode == 1 { "reflection_exact" } else if mode == 2 { "degraded_by_param_count" } else { "not_resolved" },
        mode == 2,
        CAP,
        WRITE_SEQ.load(Ordering::Relaxed),
        ADDR_GR_I4_1.load(Ordering::Relaxed),
        ADDR_GR_I4_2.load(Ordering::Relaxed),
        ADDR_GR_R4_1.load(Ordering::Relaxed),
        ADDR_GR_R4_2.load(Ordering::Relaxed),
        ADDR_RPA_GET.load(Ordering::Relaxed),
        ADDR_RPA_SET.load(Ordering::Relaxed),
        smrrp,
        pending
    )
}

/// GET /api/rng/consumption/toggle?enabled=1|0
pub fn toggle_endpoint(uri: &str) -> String {
    let q = super::parse_query(uri, "enabled");
    let want = match q.as_str() {
        "1" | "true" | "on" => true,
        "0" | "false" | "off" => false,
        _ => !ENABLED.load(Ordering::Relaxed),
    };
    if want {
        unsafe { install(); }
        let n = unsafe { install_hooks() };
        ENABLED.store(true, Ordering::Release);
        format!(
            r#"{{"ok":true,"enabled":true,"detours_installed":{},"hint":"高频 hook 已开启；观测完请 enabled=0 停止记录"}}"#, n
        )
    } else {
        ENABLED.store(false, Ordering::Release);
        r#"{"ok":true,"enabled":false,"hint":"记录已停止；detour 仍在（interceptor 无 unhook），但 hook 内直接跳过，开销极低"}"#.to_string()
    }
}

/// GET /api/rng/consumption/clear
pub fn clear_endpoint() -> String {
    WRITE_SEQ.store(0, Ordering::Release);
    unsafe {
        let base = core::ptr::addr_of_mut!(EVENTS).cast::<Event>();
        for i in 0..CAP {
            core::ptr::write_volatile(base.add(i), Event::ZERO);
        }
    }
    r#"{"ok":true,"cleared":true}"#.to_string()
}

/// GET /api/rng/consumption/context?turn=<n>&action=<m>
pub fn context_endpoint(uri: &str) -> String {
    let t = super::parse_query(uri, "turn").parse::<i32>().unwrap_or(-1);
    let a = super::parse_query(uri, "action").parse::<i32>().unwrap_or(-1);
    CTX_TURN.store(t, Ordering::Relaxed);
    CTX_ACTION.store(a, Ordering::Relaxed);
    // 打一条显式标记，方便在流水里对齐业务边界。
    // 不改动用户设定的 ENABLED 开关：临时借用后立即恢复原值。
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
        let prev = ENABLED.swap(true, Ordering::AcqRel);
        record(SITE_CTX_MARK, KIND_VOID, t as u32 as u64, a as u32 as u64, 0);
        ENABLED.store(prev, Ordering::Release);
    }));
    format!(r#"{{"ok":true,"turn":{},"action":{},"mark_written":true}}"#, t, a)
}
