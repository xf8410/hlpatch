#![allow(dead_code)]
//! URA 小黑板 v2.0.0
//! Hachimi Edge 插件 — 赛马娘育成数据实时显示与训练推荐
//!
//! 核心功能:
//! - 通过 IL2CPP API 实时读取育成数据
//! - HTTP Server (:18765) 推送数据给 uma-juece 浮窗App
//! - Hachimi GUI 面板显示五维属性与训练推荐
//!
//! 关键设计决策:
//! - 仅使用 hachimi_init (V2 API)，不导出 hachimi_init_v3
//! - 通过 hachimi_register_on_game_initialized 等待 IL2CPP 就绪后再解析元数据
//! - Arc<Mutex<GameData>> 全局共享数据，GUI面板和HTTP Server都从这里读
//! - 后台线程定时刷新数据，HTTP Server即时响应
//! - 所有 unsafe 代码均有 null 检查

use std::ffi::{c_char, c_void, CString};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::ptr;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

// ============================================================
// 颜色常量 (与URA小黑板一致)
// ============================================================
const C_SPEED: (u8, u8, u8) = (68, 136, 255);       // 蓝
const C_STAMINA: (u8, u8, u8) = (255, 68, 68);      // 红
const C_POWER: (u8, u8, u8) = (255, 136, 0);        // 橙
const C_GUTS: (u8, u8, u8) = (255, 102, 170);       // 粉
const C_WISDOM: (u8, u8, u8) = (255, 221, 0);       // 黄
const C_RECOMMEND: (u8, u8, u8) = (0, 255, 136);    // 推荐绿
const C_MOT5: (u8, u8, u8) = (0, 255, 100);         // 绝好调
const C_MOT4: (u8, u8, u8) = (255, 221, 0);         // 好调
const C_MOT3: (u8, u8, u8) = (255, 170, 0);         // 普通
const C_MOT2: (u8, u8, u8) = (255, 68, 68);         // 不调
const C_MOT1: (u8, u8, u8) = (180, 0, 0);           // 绝不调
const C_ENERGY_HI: (u8, u8, u8) = (0, 255, 100);
const C_ENERGY_MID: (u8, u8, u8) = (255, 221, 0);
const C_ENERGY_LO: (u8, u8, u8) = (255, 68, 68);
const C_WHITE: (u8, u8, u8) = (255, 255, 255);
const C_GRAY: (u8, u8, u8) = (180, 180, 180);
const AA: u8 = 255; // alpha

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
// Vtable (Hachimi Plugin API v2)
// ============================================================
#[repr(C)]
pub struct Vtable {
    pub hachimi_instance: unsafe extern "C" fn() -> *const c_void,
    pub hachimi_get_interceptor: unsafe extern "C" fn(this: *const c_void) -> *const c_void,
    pub interceptor_hook: unsafe extern "C" fn(this: *const c_void, orig: *mut c_void, hook: *mut c_void) -> *mut c_void,
    pub interceptor_hook_vtable: unsafe extern "C" fn(this: *const c_void, vt: *mut *mut c_void, idx: usize, hook: *mut c_void) -> *mut c_void,
    pub interceptor_get_trampoline_addr: unsafe extern "C" fn(this: *const c_void, hook: *mut c_void) -> *mut c_void,
    pub interceptor_unhook: unsafe extern "C" fn(this: *const c_void, hook: *mut c_void) -> *mut c_void,
    pub il2cpp_resolve_symbol: unsafe extern "C" fn(name: *const c_char) -> *mut c_void,
    pub il2cpp_get_assembly_image: unsafe extern "C" fn(assembly_name: *const c_char) -> *const c_void,
    pub il2cpp_get_class: unsafe extern "C" fn(image: *const c_void, ns: *const c_char, name: *const c_char) -> *mut c_void,
    pub il2cpp_get_method: unsafe extern "C" fn(klass: *mut c_void, name: *const c_char, args: i32) -> *const c_void,
    pub il2cpp_get_method_overload: unsafe extern "C" fn(klass: *mut c_void, name: *const c_char, params: *const c_void, n: usize) -> *const c_void,
    pub il2cpp_get_method_addr: unsafe extern "C" fn(klass: *mut c_void, name: *const c_char, args: i32) -> *mut c_void,
    pub il2cpp_get_method_overload_addr: unsafe extern "C" fn(klass: *mut c_void, name: *const c_char, params: *const c_void, n: usize) -> *mut c_void,
    pub il2cpp_get_method_cached: unsafe extern "C" fn(klass: *mut c_void, name: *const c_char, args: i32) -> *const c_void,
    pub il2cpp_get_method_addr_cached: unsafe extern "C" fn(klass: *mut c_void, name: *const c_char, args: i32) -> *mut c_void,
    pub il2cpp_find_nested_class: unsafe extern "C" fn(klass: *mut c_void, name: *const c_char) -> *mut c_void,
    pub il2cpp_resolve_icall: unsafe extern "C" fn(name: *const c_char) -> *mut c_void,
    pub il2cpp_class_get_methods: unsafe extern "C" fn(klass: *mut c_void, iter: *mut *mut c_void) -> *const c_void,
    pub il2cpp_get_field_from_name: unsafe extern "C" fn(klass: *mut c_void, name: *const c_char) -> *mut c_void,
    pub il2cpp_get_field_value: unsafe extern "C" fn(obj: *mut c_void, field: *mut c_void, out: *mut c_void),
    pub il2cpp_set_field_value: unsafe extern "C" fn(obj: *mut c_void, field: *mut c_void, val: *const c_void),
    pub il2cpp_get_static_field_value: unsafe extern "C" fn(field: *mut c_void, out: *mut c_void),
    pub il2cpp_set_static_field_value: unsafe extern "C" fn(field: *mut c_void, val: *const c_void),
    pub il2cpp_object_new: unsafe extern "C" fn(klass: *const c_void) -> *mut c_void,
    pub il2cpp_unbox: unsafe extern "C" fn(obj: *mut c_void) -> *mut c_void,
    pub il2cpp_get_main_thread: unsafe extern "C" fn() -> *mut c_void,
    pub il2cpp_get_attached_threads: unsafe extern "C" fn(out_size: *mut usize) -> *mut *mut c_void,
    pub il2cpp_schedule_on_thread: unsafe extern "C" fn(thread: *mut c_void, cb: unsafe extern "C" fn()),
    pub il2cpp_create_array: unsafe extern "C" fn(element_type: *mut c_void, len: usize) -> *mut c_void,
    pub il2cpp_get_singleton_like_instance: unsafe extern "C" fn(klass: *mut c_void) -> *mut c_void,
    pub log: unsafe extern "C" fn(level: i32, target: *const c_char, message: *const c_char),
    pub gui_register_menu_item: unsafe extern "C" fn(label: *const c_char, cb: Option<extern "C" fn(*mut c_void)>, ud: *mut c_void) -> bool,
    pub gui_register_menu_section: unsafe extern "C" fn(cb: Option<extern "C" fn(*mut c_void, *mut c_void)>, ud: *mut c_void) -> bool,
    pub gui_show_notification: unsafe extern "C" fn(msg: *const c_char) -> bool,
    pub gui_ui_heading: unsafe extern "C" fn(ui: *mut c_void, text: *const c_char) -> bool,
    pub gui_ui_label: unsafe extern "C" fn(ui: *mut c_void, text: *const c_char) -> bool,
    pub gui_ui_small: unsafe extern "C" fn(ui: *mut c_void, text: *const c_char) -> bool,
    pub gui_ui_separator: unsafe extern "C" fn(ui: *mut c_void) -> bool,
    pub gui_ui_button: unsafe extern "C" fn(ui: *mut c_void, text: *const c_char) -> bool,
    pub gui_ui_small_button: unsafe extern "C" fn(ui: *mut c_void, text: *const c_char) -> bool,
    pub gui_ui_checkbox: unsafe extern "C" fn(ui: *mut c_void, text: *const c_char, val: *mut bool) -> bool,
    pub gui_ui_text_edit_singleline: unsafe extern "C" fn(ui: *mut c_void, buf: *mut c_char, len: usize) -> bool,
    pub gui_ui_horizontal: unsafe extern "C" fn(ui: *mut c_void, cb: Option<extern "C" fn(*mut c_void, *mut c_void)>, ud: *mut c_void) -> bool,
    pub gui_ui_grid: unsafe extern "C" fn(ui: *mut c_void, id: *const c_char, cols: usize, sx: f32, sy: f32, cb: Option<extern "C" fn(*mut c_void, *mut c_void)>, ud: *mut c_void) -> bool,
    pub gui_ui_end_row: unsafe extern "C" fn(ui: *mut c_void) -> bool,
    pub gui_ui_colored_label: unsafe extern "C" fn(ui: *mut c_void, r: u8, g: u8, b: u8, a: u8, text: *const c_char) -> bool,
    pub gui_register_menu_item_icon: unsafe extern "C" fn(label: *const c_char, icon_uri: *const c_char, icon_ptr: *const u8, icon_len: usize) -> bool,
    pub gui_register_menu_section_with_icon: unsafe extern "C" fn(title: *const c_char, icon_uri: *const c_char, icon_ptr: *const u8, icon_len: usize, cb: Option<extern "C" fn(*mut c_void, *mut c_void)>, ud: *mut c_void) -> bool,
    pub gui_new_window_id: unsafe extern "C" fn() -> i32,
    pub gui_show_window: unsafe extern "C" fn(id: i32, title: *const c_char, contents: Option<extern "C" fn(*mut c_void, *mut c_void)>, bottom: Option<extern "C" fn(*mut c_void, *mut c_void)>, ud: *mut c_void) -> bool,
    pub gui_close_window: unsafe extern "C" fn(id: i32),
    pub android_dex_load: unsafe extern "C" fn(dex_ptr: *const u8, dex_len: usize, cls: *const c_char) -> u64,
    pub android_dex_unload: unsafe extern "C" fn(handle: u64) -> bool,
    pub android_dex_call_static_noargs: unsafe extern "C" fn(handle: u64, method: *const c_char, sig: *const c_char) -> bool,
    pub android_dex_call_static_string: unsafe extern "C" fn(handle: u64, method: *const c_char, sig: *const c_char, arg: *const c_char) -> bool,
    pub il2cpp_runtime_object_init: unsafe extern "C" fn(object: *mut c_void),
    pub il2cpp_string_new: unsafe extern "C" fn(text: *const c_char) -> *mut c_void,
    pub il2cpp_string_chars: unsafe extern "C" fn(s: *mut c_void) -> *mut u16,
    pub il2cpp_string_length: unsafe extern "C" fn(s: *mut c_void) -> i32,
    pub gui_ui_combo_menu: unsafe extern "C" fn(ui: *mut c_void, id: *const c_char, sel: *mut i32, items: *const *const c_char, n: usize, search: *mut c_char, search_len: usize) -> bool,
    pub hachimi_register_on_game_initialized: unsafe extern "C" fn(cb: Option<unsafe extern "C" fn(*mut c_void)>, ud: *mut c_void) -> bool,
    pub hachimi_register_present_callback: unsafe extern "C" fn(cb: Option<unsafe extern "C" fn(*mut c_void, *mut c_void)>, ud: *mut c_void) -> bool,
    pub gui_get_menu_width: unsafe extern "C" fn() -> f32,
    pub gui_set_menu_width: unsafe extern "C" fn(w: f32),
    pub hachimi_get_base_dir: unsafe extern "C" fn() -> *const c_char,
    pub hachimi_get_data_path: unsafe extern "C" fn() -> *const c_char,
}

// ============================================================
// 全局状态
// ============================================================
static mut VT: *const Vtable = ptr::null();

/// 缓存的 IL2CPP 类/字段元数据
#[repr(C)]
#[derive(Copy, Clone)]
struct Meta {
    ok: bool,
    // Classes
    smd_cls: *mut c_void,
    smci_cls: *mut c_void,
    smcmdi_cls: *mut c_void,
    smpidci_cls: *mut c_void,
    sm_model_cls: *mut c_void,
    // SingleModeData fields
    f_chara_info: *mut c_void,
    f_command_array: *mut c_void,
    // SingleModeCharaInfo fields
    f_speed: *mut c_void,
    f_stamina: *mut c_void,
    f_power: *mut c_void,
    f_guts: *mut c_void,
    f_wisdom: *mut c_void,
    f_skill_point: *mut c_void,
    f_motivation: *mut c_void,
    f_energy: *mut c_void,
    f_max_energy: *mut c_void,
    f_turn: *mut c_void,
    f_fan_count: *mut c_void,
    // SingleModeCommandInfo fields
    f_command_type: *mut c_void,
    f_command_id: *mut c_void,
    f_training_partner_array: *mut c_void,
    f_failure_rate: *mut c_void,
    f_params_inc_dec_info_array: *mut c_void,
    // SingleModeParamsIncDecInfo fields
    f_value_array: *mut c_void,
    // SingleModel fallback field
    f_data: *mut c_void,
}

impl Meta {
    const fn zero() -> Self {
        Self {
            ok: false,
            smd_cls: ptr::null_mut(), smci_cls: ptr::null_mut(),
            smcmdi_cls: ptr::null_mut(), smpidci_cls: ptr::null_mut(),
            sm_model_cls: ptr::null_mut(),
            f_chara_info: ptr::null_mut(), f_command_array: ptr::null_mut(),
            f_speed: ptr::null_mut(), f_stamina: ptr::null_mut(),
            f_power: ptr::null_mut(), f_guts: ptr::null_mut(),
            f_wisdom: ptr::null_mut(), f_skill_point: ptr::null_mut(),
            f_motivation: ptr::null_mut(), f_energy: ptr::null_mut(),
            f_max_energy: ptr::null_mut(), f_turn: ptr::null_mut(),
            f_fan_count: ptr::null_mut(),
            f_command_type: ptr::null_mut(), f_command_id: ptr::null_mut(),
            f_training_partner_array: ptr::null_mut(),
            f_failure_rate: ptr::null_mut(),
            f_params_inc_dec_info_array: ptr::null_mut(),
            f_value_array: ptr::null_mut(),
            f_data: ptr::null_mut(),
        }
    }
}

static mut META: Meta = Meta::zero();
static mut NOTIFICATION_SENT: bool = false;

// ============================================================
// 全局共享游戏数据 (Arc<Mutex<GameData>>)
// GUI 面板渲染和 HTTP Server 都从这里读
// ============================================================

/// 单条训练的摘要数据（供 HTTP JSON 输出和推荐计算使用）
#[derive(Clone)]
struct TrainingItemData {
    valid: bool,
    command_type: i32,
    gains: [i32; 5],       // [速, 耐, 力, 根, 智]
    pt_gain: i32,
    failure_rate: i32,
    partner_count: i32,
    score: f32,
}

impl TrainingItemData {
    fn invalid() -> Self {
        Self {
            valid: false, command_type: 0,
            gains: [0; 5], pt_gain: 0,
            failure_rate: 0, partner_count: 0, score: -1.0,
        }
    }
}

/// 全局共享的育成数据快照
#[derive(Clone)]
struct GameData {
    available: bool,
    turn: i32,
    speed: i32, stamina: i32, power: i32, guts: i32, wisdom: i32,
    energy: i32, max_energy: i32,
    motivation: i32,
    skill_point: i32,
    fan_count: i32,
    trainings: [TrainingItemData; 5],
    best_training_idx: usize,
}

impl GameData {
    fn empty() -> Self {
        Self {
            available: false,
            turn: 0,
            speed: 0, stamina: 0, power: 0, guts: 0, wisdom: 0,
            energy: 0, max_energy: 0,
            motivation: 0,
            skill_point: 0,
            fan_count: 0,
            trainings: [
                TrainingItemData::invalid(), TrainingItemData::invalid(),
                TrainingItemData::invalid(), TrainingItemData::invalid(),
                TrainingItemData::invalid(),
            ],
            best_training_idx: 0,
        }
    }

    /// 超过 1200 的属性按双倍算
    fn revise(x: i32) -> i32 {
        if x > 1200 { x * 2 - 1200 } else { x }
    }

    fn revised_total(&self) -> i32 {
        Self::revise(self.speed) + Self::revise(self.stamina) +
        Self::revise(self.power) + Self::revise(self.guts) +
        Self::revise(self.wisdom)
    }

    /// 生成 HTTP /data 响应的 JSON（零依赖手拼）
    fn to_data_json(&self) -> String {
        if !self.available {
            return r#"{"error":"no_data","status":"waiting"}"#.to_string();
        }

        let total = self.speed + self.stamina + self.power + self.guts + self.wisdom;

        // 推荐训练
        let best = &self.trainings[self.best_training_idx];
        let (recommend, recommend_color) = if best.valid && best.score > 0.0 {
            (cmd_type_to_english(best.command_type), cmd_type_to_hex_color(best.command_type))
        } else {
            ("none", "#FFFFFF")
        };

        // 训练列表 JSON
        let mut training_parts: [String; 5] = [String::new(), String::new(), String::new(), String::new(), String::new()];
        for i in 0..5 {
            let t = &self.trainings[i];
            if t.valid && t.score > 0.0 {
                let primary_gain = t.gains[cmd_type_to_gains_idx(t.command_type)];
                let score_rounded = (t.score + 0.5) as i32; // 四舍五入
                training_parts[i] = format!(
                    r#"{{"type":"{}","gain":{},"failure_rate":{},"score":{}}}"#,
                    cmd_type_to_english(t.command_type),
                    primary_gain,
                    t.failure_rate,
                    score_rounded,
                );
            }
        }
        let training_json: String = training_parts.iter()
            .filter(|s| !s.is_empty())
            .cloned()
            .collect::<Vec<_>>()
            .join(",");

        format!(
            r#"{{"turn":{},"speed":{},"stamina":{},"power":{},"guts":{},"wisdom":{},"total":{},"energy":{},"max_energy":{},"motivation":{},"skill_point":{},"fan_count":{},"recommend":"{}","recommend_color":"{}","training":[{}]}}"#,
            self.turn, self.speed, self.stamina, self.power, self.guts, self.wisdom,
            total, self.energy, self.max_energy, self.motivation, self.skill_point,
            self.fan_count, recommend, recommend_color, training_json
        )
    }
}

/// 全局 GAME_DATA 指针（堆上分配的 Mutex<GameData>，进程生命周期内不释放）
static mut GAME_DATA: *mut Mutex<GameData> = ptr::null_mut();

/// 记录实际绑定的 HTTP 端口
static mut HTTP_PORT: u16 = 0;

// ============================================================
// 训练类型映射辅助函数
// ============================================================

/// command_type → 英文名（HTTP JSON 用）
fn cmd_type_to_english(ct: i32) -> &'static str {
    match ct {
        101 => "speed",
        105 => "stamina",
        102 => "power",
        103 => "guts",
        106 => "wisdom",
        _   => "unknown",
    }
}

/// command_type → gains 数组索引
fn cmd_type_to_gains_idx(ct: i32) -> usize {
    match ct {
        101 => 0, // speed
        105 => 1, // stamina
        102 => 2, // power
        103 => 3, // guts
        106 => 4, // wisdom
        _   => 0,
    }
}

/// command_type → 十六进制颜色（HTTP JSON 用）
fn cmd_type_to_hex_color(ct: i32) -> &'static str {
    match ct {
        101 => "#4488FF", // 速度蓝
        105 => "#FF4444", // 耐力红
        102 => "#FF8800", // 力量橙
        103 => "#FF66AA", // 根性粉
        106 => "#FFDD00", // 智力黄
        _   => "#FFFFFF",
    }
}

// ============================================================
// IL2CPP 辅助函数
// ============================================================

/// 读取 i32 值类型字段
unsafe fn read_i32(obj: *mut c_void, field: *mut c_void) -> i32 {
    if obj.is_null() || field.is_null() { return 0; }
    let mut v: i32 = 0;
    ((*VT).il2cpp_get_field_value)(obj, field, &mut v as *mut _ as *mut c_void);
    v
}

/// 读取引用类型字段（返回对象指针）
unsafe fn read_obj(obj: *mut c_void, field: *mut c_void) -> *mut c_void {
    if obj.is_null() || field.is_null() { return ptr::null_mut(); }
    let mut v: *mut c_void = ptr::null_mut();
    ((*VT).il2cpp_get_field_value)(obj, field, &mut v as *mut _ as *mut c_void);
    v
}

/// IL2CPP Array 布局 (aarch64):
///   +0x00  klass    (8)
///   +0x08  monitor  (8)
///   +0x10  bounds   (8)
///   +0x18  max_length (4, il2cpp_array_size_t = u32)
///   +0x1C  padding  (4)
///   +0x20  data[0]
const ARR_DATA_OFF: usize = 0x20;

unsafe fn arr_len(arr: *mut c_void) -> i32 {
    if arr.is_null() { return 0; }
    let len = ptr::read((arr as *const u8).add(0x18) as *const i32);
    if len < 0 { 0 } else { len }
}

/// 引用类型数组：取第 idx 个元素（指针，8 bytes each）
unsafe fn arr_get_ptr(arr: *mut c_void, idx: i32) -> *mut c_void {
    if arr.is_null() || idx < 0 { return ptr::null_mut(); }
    if idx >= arr_len(arr) { return ptr::null_mut(); }
    ptr::read((arr as *const u8).add(ARR_DATA_OFF + (idx as usize) * 8) as *const *mut c_void)
}

/// int[] 数组：取第 idx 个元素（4 bytes each）
unsafe fn arr_get_i32(arr: *mut c_void, idx: i32) -> i32 {
    if arr.is_null() || idx < 0 { return 0; }
    if idx >= arr_len(arr) { return 0; }
    ptr::read((arr as *const u8).add(ARR_DATA_OFF + (idx as usize) * 4) as *const i32)
}

// ============================================================
// 游戏数据结构（IL2CPP 读取用的中间结构）
// ============================================================

struct CharaInfo {
    speed: i32, stamina: i32, power: i32, guts: i32, wisdom: i32,
    skill_point: i32, motivation: i32, energy: i32, max_energy: i32,
    turn: i32, fan_count: i32,
}

impl CharaInfo {
    fn revise(x: i32) -> i32 {
        if x > 1200 { x * 2 - 1200 } else { x }
    }
    fn revised_total(&self) -> i32 {
        Self::revise(self.speed) + Self::revise(self.stamina) +
        Self::revise(self.power) + Self::revise(self.guts) +
        Self::revise(self.wisdom)
    }
    fn stats_array(&self) -> [i32; 5] {
        [self.speed, self.stamina, self.power, self.guts, self.wisdom]
    }
}

const MAX_TRAININGS: usize = 5;

#[derive(Copy, Clone)]
struct TrainingCmd {
    valid: bool,
    command_type: i32,
    command_id: i32,
    gains: [i32; 5],  // [速, 耐, 力, 根, 智] 来自 value_array[0..5]
    pt_gain: i32,     // 来自 value_array[5]
    failure_rate: i32,
    partner_count: i32,
}

impl TrainingCmd {
    const fn invalid() -> Self {
        Self { valid: false, command_type: 0, command_id: 0,
               gains: [0; 5], pt_gain: 0, failure_rate: 0, partner_count: 0 }
    }
}

// ============================================================
// 游戏元数据解析（on_game_initialized 回调中执行）
// ============================================================

unsafe fn resolve_class(image: *const c_void, ns: &str, name: &str) -> *mut c_void {
    let vt = &*VT;
    let ns_c = CString::new(ns).unwrap();
    let name_c = CString::new(name).unwrap();
    let mut klass = (vt.il2cpp_get_class)(image, ns_c.as_ptr(), name_c.as_ptr());
    if klass.is_null() {
        let empty = CString::new("").unwrap();
        klass = (vt.il2cpp_get_class)(image, empty.as_ptr(), name_c.as_ptr());
    }
    klass
}

unsafe fn resolve_field(klass: *mut c_void, name: &str) -> *mut c_void {
    if klass.is_null() { return ptr::null_mut(); }
    let name_c = CString::new(name).unwrap();
    ((*VT).il2cpp_get_field_from_name)(klass, name_c.as_ptr())
}

unsafe fn resolve_field_multi(klass: *mut c_void, names: &[&str]) -> *mut c_void {
    for &name in names {
        let f = resolve_field(klass, name);
        if !f.is_null() { return f; }
    }
    ptr::null_mut()
}

unsafe fn resolve_meta() {
    let vt = &*VT;

    let gallop_c = CString::new("Gallop").unwrap();
    let image = (vt.il2cpp_get_assembly_image)(gallop_c.as_ptr());
    if image.is_null() {
        let msg = CString::new("URA: Gallop image not found").unwrap();
        (vt.log)(2, b"URA\0".as_ptr() as *const c_char, msg.as_ptr());
        return;
    }

    META.smd_cls    = resolve_class(image, "Gallop", "SingleModeData");
    META.smci_cls   = resolve_class(image, "Gallop", "SingleModeCharaInfo");
    META.smcmdi_cls = resolve_class(image, "Gallop", "SingleModeCommandInfo");
    META.smpidci_cls = resolve_class(image, "Gallop", "SingleModeParamsIncDecInfo");
    META.sm_model_cls = resolve_class(image, "Gallop", "SingleModel");

    if META.smci_cls.is_null() {
        let msg = CString::new("URA: CharaInfo class not found").unwrap();
        (vt.log)(2, b"URA\0".as_ptr() as *const c_char, msg.as_ptr());
        return;
    }

    // SingleModeData 字段
    if !META.smd_cls.is_null() {
        META.f_chara_info = resolve_field_multi(META.smd_cls, &[
            "CharaInfo", "chara_info", "_charaInfo",
        ]);
        META.f_command_array = resolve_field_multi(META.smd_cls, &[
            "CommandArray", "commandArray", "_commandArray",
            "CommandInfoArray", "commandInfoArray",
        ]);
    }

    // SingleModel 回退字段
    if !META.sm_model_cls.is_null() {
        META.f_data = resolve_field_multi(META.sm_model_cls, &[
            "Data", "data", "_data", "SingleModeData",
        ]);
    }

    // SingleModeCharaInfo 字段
    META.f_speed      = resolve_field_multi(META.smci_cls, &["Speed", "speed", "_speed"]);
    META.f_stamina    = resolve_field_multi(META.smci_cls, &["Stamina", "stamina", "_stamina"]);
    META.f_power      = resolve_field_multi(META.smci_cls, &["Power", "power", "_power"]);
    META.f_guts       = resolve_field_multi(META.smci_cls, &["Guts", "guts", "_guts"]);
    META.f_wisdom     = resolve_field_multi(META.smci_cls, &["Wisdom", "wisdom", "_wisdom"]);
    META.f_skill_point = resolve_field_multi(META.smci_cls, &["SkillPoint", "skillPoint", "_skillPoint", "skill_point"]);
    META.f_motivation = resolve_field_multi(META.smci_cls, &["Motivation", "motivation", "_motivation"]);
    META.f_energy     = resolve_field_multi(META.smci_cls, &["Energy", "energy", "_energy"]);
    META.f_max_energy = resolve_field_multi(META.smci_cls, &["MaxEnergy", "maxEnergy", "_maxEnergy", "max_energy"]);
    META.f_turn       = resolve_field_multi(META.smci_cls, &["Turn", "turn", "_turn"]);
    META.f_fan_count  = resolve_field_multi(META.smci_cls, &["FanCount", "fanCount", "_fanCount", "fan_count"]);

    // SingleModeCommandInfo 字段
    if !META.smcmdi_cls.is_null() {
        META.f_command_type = resolve_field_multi(META.smcmdi_cls, &[
            "CommandType", "commandType", "_commandType",
        ]);
        META.f_command_id = resolve_field_multi(META.smcmdi_cls, &[
            "CommandId", "commandId", "_commandId",
        ]);
        META.f_training_partner_array = resolve_field_multi(META.smcmdi_cls, &[
            "TrainingPartnerArray", "trainingPartnerArray", "_trainingPartnerArray",
        ]);
        META.f_failure_rate = resolve_field_multi(META.smcmdi_cls, &[
            "FailureRate", "failureRate", "_failureRate",
        ]);
        META.f_params_inc_dec_info_array = resolve_field_multi(META.smcmdi_cls, &[
            "ParamsIncDecInfoArray", "paramsIncDecInfoArray", "_paramsIncDecInfoArray",
        ]);
    }

    // SingleModeParamsIncDecInfo 字段
    if !META.smpidci_cls.is_null() {
        META.f_value_array = resolve_field_multi(META.smpidci_cls, &[
            "ValueArray", "valueArray", "_valueArray",
        ]);
    }

    META.ok = true;
    let msg = CString::new("URA: metadata resolved").unwrap();
    (vt.log)(0, b"URA\0".as_ptr() as *const c_char, msg.as_ptr());
}

// ============================================================
// 实时数据读取
// ============================================================

/// 获取 SingleModeData 实例（尝试多种路径）
unsafe fn get_smd_instance() -> *mut c_void {
    let vt = &*VT;

    // 路径1: SingleModeData 作为 singleton
    if !META.smd_cls.is_null() {
        let inst = (vt.il2cpp_get_singleton_like_instance)(META.smd_cls);
        if !inst.is_null() { return inst; }
    }

    // 路径2: SingleModel singleton → Data 字段
    if !META.sm_model_cls.is_null() && !META.f_data.is_null() {
        let model = (vt.il2cpp_get_singleton_like_instance)(META.sm_model_cls);
        if !model.is_null() {
            let data = read_obj(model, META.f_data);
            if !data.is_null() { return data; }
        }
    }

    ptr::null_mut()
}

/// 读取角色信息
unsafe fn read_chara_info(smd: *mut c_void) -> Option<CharaInfo> {
    if smd.is_null() || META.f_chara_info.is_null() { return None; }
    let ci = read_obj(smd, META.f_chara_info);
    if ci.is_null() { return None; }

    Some(CharaInfo {
        speed:      read_i32(ci, META.f_speed),
        stamina:    read_i32(ci, META.f_stamina),
        power:      read_i32(ci, META.f_power),
        guts:       read_i32(ci, META.f_guts),
        wisdom:     read_i32(ci, META.f_wisdom),
        skill_point: read_i32(ci, META.f_skill_point),
        motivation: read_i32(ci, META.f_motivation),
        energy:     read_i32(ci, META.f_energy),
        max_energy: read_i32(ci, META.f_max_energy),
        turn:       read_i32(ci, META.f_turn),
        fan_count:  read_i32(ci, META.f_fan_count),
    })
}

/// 读取训练指令信息
unsafe fn read_trainings(smd: *mut c_void) -> [TrainingCmd; MAX_TRAININGS] {
    let mut cmds = [TrainingCmd::invalid(); MAX_TRAININGS];

    if smd.is_null() || META.f_command_array.is_null() { return cmds; }
    let arr = read_obj(smd, META.f_command_array);
    if arr.is_null() { return cmds; }

    let len = arr_len(arr);
    if len <= 0 { return cmds; }

    let count = if (len as usize) < MAX_TRAININGS { len as usize } else { MAX_TRAININGS };

    for i in 0..count {
        let cmd_obj = arr_get_ptr(arr, i as i32);
        if cmd_obj.is_null() { continue; }

        let mut cmd = TrainingCmd::invalid();
        cmd.valid = true;
        cmd.command_type = read_i32(cmd_obj, META.f_command_type);
        cmd.command_id   = read_i32(cmd_obj, META.f_command_id);
        cmd.failure_rate  = read_i32(cmd_obj, META.f_failure_rate);

        // training_partner_array → 统计伙伴数量
        let partner_arr = read_obj(cmd_obj, META.f_training_partner_array);
        cmd.partner_count = arr_len(partner_arr);

        // params_inc_dec_info_array → 累加增益
        let inc_arr = read_obj(cmd_obj, META.f_params_inc_dec_info_array);
        let inc_len = arr_len(inc_arr);
        for j in 0..inc_len {
            let inc_obj = arr_get_ptr(inc_arr, j);
            if inc_obj.is_null() { continue; }
            let val_arr = read_obj(inc_obj, META.f_value_array);
            if val_arr.is_null() { continue; }
            // value_array: [速, 耐, 力, 根, 智, Pt, ...]
            for k in 0..5usize {
                cmd.gains[k] += arr_get_i32(val_arr, k as i32);
            }
            cmd.pt_gain += arr_get_i32(val_arr, 5);
        }

        cmds[i] = cmd;
    }

    cmds
}

// ============================================================
// 数据刷新 → 更新全局 GameData
// ============================================================

/// 从 IL2CPP 读取最新游戏数据并更新 GAME_DATA
/// 安全性：VT/META 在初始化后只读，并发访问实际安全
unsafe fn refresh_game_data() {
    if VT.is_null() || !META.ok { return; }

    let smd = get_smd_instance();
    if smd.is_null() {
        // 育成模式未激活，保持旧数据但标记不可用
        if GAME_DATA.is_null() { return; }
        let mut gd = (*GAME_DATA).lock().unwrap();
        gd.available = false;
        return;
    }

    let ci = match read_chara_info(smd) {
        Some(info) => info,
        None => return,
    };

    let trainings = read_trainings(smd);
    let stats = ci.stats_array();

    // 计算训练推荐得分
    let mut best_idx: usize = 0;
    let mut best_score: f32 = -1.0;
    let mut scores: [f32; MAX_TRAININGS] = [-1.0; MAX_TRAININGS];

    for i in 0..MAX_TRAININGS {
        let s = training_score(&trainings[i], &stats, ci.motivation);
        scores[i] = s;
        if s > best_score {
            best_score = s;
            best_idx = i;
        }
    }

    // 更新全局数据
    if GAME_DATA.is_null() { return; }
    let mut gd = (*GAME_DATA).lock().unwrap();
    gd.available = true;
    gd.turn = ci.turn;
    gd.speed = ci.speed;
    gd.stamina = ci.stamina;
    gd.power = ci.power;
    gd.guts = ci.guts;
    gd.wisdom = ci.wisdom;
    gd.energy = ci.energy;
    gd.max_energy = ci.max_energy;
    gd.motivation = ci.motivation;
    gd.skill_point = ci.skill_point;
    gd.fan_count = ci.fan_count;
    gd.best_training_idx = best_idx;

    for i in 0..MAX_TRAININGS {
        gd.trainings[i] = TrainingItemData {
            valid: trainings[i].valid,
            command_type: trainings[i].command_type,
            gains: trainings[i].gains,
            pt_gain: trainings[i].pt_gain,
            failure_rate: trainings[i].failure_rate,
            partner_count: trainings[i].partner_count,
            score: scores[i],
        };
    }
}

// ============================================================
// 训练推荐计算
// ============================================================

/// 干劲倍率
fn mot_mult(m: i32) -> f32 {
    match m {
        1 => 0.6,  // 绝不调
        2 => 0.8,  // 不调
        3 => 1.0,  // 普通
        4 => 1.1,  // 好调
        5 => 1.2,  // 绝好调
        _ => 1.0,
    }
}

/// 高属性衰减系数
fn stat_decay(current: i32) -> f32 {
    if current > 900 { 0.6 }
    else if current > 800 { 0.8 }
    else { 1.0 }
}

/// 计算单个训练的期望收益
fn training_score(cmd: &TrainingCmd, stats: &[i32; 5], motivation: i32) -> f32 {
    if !cmd.valid { return -1.0; }

    let mm = mot_mult(motivation);
    let mut total = 0.0f32;

    // 属性权重：速1.0 耐1.0 力1.0 根0.7 智0.8
    let weights: [f32; 5] = [1.0, 1.0, 1.0, 0.7, 0.8];

    for i in 0..5 {
        let gain = cmd.gains[i] as f32;
        let adjusted = gain * mm * stat_decay(stats[i]) * weights[i];
        total += adjusted;
    }

    // 加上 Pt 增益（权重较低，0.3）
    total += cmd.pt_gain as f32 * 0.3;

    // 失败率惩罚
    let fail_rate = cmd.failure_rate as f32 / 100.0;
    total *= (1.0 - fail_rate).max(0.0);

    // 伙伴加成（每个伙伴 +5% 效率）
    let partner_bonus = 1.0 + (cmd.partner_count as f32 * 0.05);
    total *= partner_bonus;

    total
}

// ============================================================
// 显示辅助函数
// ============================================================

fn mot_text(m: i32) -> &'static str {
    match m {
        1 => "\u{7edd}\u{4e0d}\u{8c03}",    // 绝不调
        2 => "\u{4e0d}\u{8c03}",             // 不调
        3 => "\u{666e}\u{901a}",             // 普通
        4 => "\u{597d}\u{8c03}",             // 好调
        5 => "\u{7edd}\u{597d}\u{8c03}",    // 绝好调
        _ => "?",
    }
}

fn mot_color(m: i32) -> (u8, u8, u8) {
    match m {
        1 => C_MOT1, 2 => C_MOT2, 3 => C_MOT3,
        4 => C_MOT4, 5 => C_MOT5, _ => C_WHITE,
    }
}

fn energy_color(ratio: f32) -> (u8, u8, u8) {
    if ratio >= 0.5 { C_ENERGY_HI }
    else if ratio >= 0.25 { C_ENERGY_MID }
    else { C_ENERGY_LO }
}

/// 修正合计评级
fn rating(total: i32) -> &'static str {
    if total >= 2400 { "SS+" }
    else if total >= 2100 { "SS" }
    else if total >= 1800 { "S+" }
    else if total >= 1500 { "S" }
    else if total >= 1200 { "A+" }
    else if total >= 1000 { "A" }
    else if total >= 800  { "B+" }
    else if total >= 600  { "B" }
    else if total >= 400  { "C+" }
    else if total >= 200  { "C" }
    else { "D" }
}

/// 回合 → (年份, 月份, 是否前半)
fn turn_to_ym(turn: i32) -> (i32, i32, bool) {
    if turn <= 0 { return (1, 1, true); }
    let year  = (turn - 1) / 24 + 1;
    let month = ((turn - 1) % 24) / 2 + 1;
    let first_half = turn % 2 == 1;
    (year, month, first_half)
}

/// 训练类型名（按 command_type：101=速, 105=耐, 102=力, 103=根, 106=智）
fn train_name(ct: i32) -> &'static str {
    match ct {
        101 => "\u{901f}\u{5ea6}",    // 速度
        105 => "\u{8010}\u{529b}",    // 耐力
        102 => "\u{529b}\u{91cf}",    // 力量
        103 => "\u{6839}\u{6027}",    // 根性
        106 => "\u{667a}\u{529b}",    // 智力
        _   => "?",
    }
}

fn train_color(ct: i32) -> (u8, u8, u8) {
    match ct {
        101 => C_SPEED,
        105 => C_STAMINA,
        102 => C_POWER,
        103 => C_GUTS,
        106 => C_WISDOM,
        _   => C_WHITE,
    }
}

fn to_cstr(s: &str) -> CString {
    CString::new(s).unwrap_or_else(|_| CString::new("<err>").unwrap())
}

// ============================================================
// GUI 渲染
// ============================================================

unsafe fn colored(ui: *mut c_void, c: (u8,u8,u8), text: &str) {
    let t = to_cstr(text);
    ((*VT).gui_ui_colored_label)(ui, c.0, c.1, c.2, AA, t.as_ptr());
}

unsafe fn label(ui: *mut c_void, text: &str) {
    let t = to_cstr(text);
    ((*VT).gui_ui_label)(ui, t.as_ptr());
}

unsafe fn small(ui: *mut c_void, text: &str) {
    let t = to_cstr(text);
    ((*VT).gui_ui_small)(ui, t.as_ptr());
}

unsafe fn heading(ui: *mut c_void, text: &str) {
    let t = to_cstr(text);
    ((*VT).gui_ui_heading)(ui, t.as_ptr());
}

unsafe fn sep(ui: *mut c_void) {
    ((*VT).gui_ui_separator)(ui);
}

/// 渲染五维属性行
unsafe fn render_stat_line(ui: *mut c_void, name: &str, color: (u8,u8,u8), val: i32) {
    let rv = GameData::revise(val);
    if val > 1200 {
        colored(ui, color, &format!("{}  {} (\u{4fee}\u{6b63}{})", name, val, rv));
    } else {
        colored(ui, color, &format!("{}  {}", name, val));
    }
}

/// 主面板渲染 — 从 GAME_DATA 读取数据
unsafe fn render_panel(ui: *mut c_void) {
    heading(ui, "URA \u{5c0f}\u{9ed1}\u{677f}");  // URA 小黑板
    sep(ui);

    if !META.ok {
        label(ui, "\u{7b49}\u{5f85}\u{6e38}\u{620f}\u{521d}\u{59cb}\u{5316}...");
        small(ui, "\u{8fdb}\u{5165}\u{80b2}\u{6210}\u{6a21}\u{5f0f}\u{540e}\u{53ef}\u{67e5}\u{770b}\u{6570}\u{636e}");
        return;
    }

    // 刷新数据（面板打开时触发即时读取）
    refresh_game_data();

    // 从全局 GAME_DATA 读取
    if GAME_DATA.is_null() {
        label(ui, "\u{5185}\u{90e8}\u{9519}\u{8bef}");
        return;
    }

    let gd = (*GAME_DATA).lock().unwrap();
    let gd = &*gd;

    if !gd.available {
        label(ui, "\u{6682}\u{65e0}\u{80b2}\u{6210}\u{6570}\u{636e}");
        small(ui, "\u{8bf7}\u{5148}\u{8fdb}\u{5165}\u{80b2}\u{6210}\u{6a21}\u{5f0f}");
        NOTIFICATION_SENT = false;
        return;
    }

    // 首次检测到育成数据时推送通知
    if !NOTIFICATION_SENT {
        let msg = to_cstr("URA \u{5c0f}\u{9ed1}\u{677f}\u{5df2}\u{6fc0}\u{6d3b}");
        ((*VT).gui_show_notification)(msg.as_ptr());
        NOTIFICATION_SENT = true;
    }

    // ---- 回合信息 ----
    let (year, month, first_half) = turn_to_ym(gd.turn);
    let half_str = if first_half {
        "\u{524d}\u{534a}"
    } else {
        "\u{540e}\u{534a}"
    };
    colored(ui, C_WHITE, &format!(
        "{}\u{5e74}\u{76ee} {}\u{6708}{}  Turn {}",
        year, month, half_str, gd.turn
    ));

    // ---- 体力 ----
    let max_e = if gd.max_energy > 0 { gd.max_energy } else { 1 };
    let e_ratio = gd.energy as f32 / max_e as f32;
    let e_pct = (e_ratio * 100.0) as i32;
    let ec = energy_color(e_ratio);
    colored(ui, ec, &format!(
        "\u{4f53}\u{529b}: {}/{} ({}%)",
        gd.energy, max_e, e_pct
    ));

    // ---- 干劲 ----
    let mc = mot_color(gd.motivation);
    colored(ui, mc, &format!(
        "\u{5e72}\u{52b2}: {}  \u{00d7}{:.1}",
        mot_text(gd.motivation), mot_mult(gd.motivation)
    ));

    sep(ui);

    // ---- 五维属性 ----
    render_stat_line(ui, "\u{901f}\u{5ea6}", C_SPEED, gd.speed);
    render_stat_line(ui, "\u{8010}\u{529b}", C_STAMINA, gd.stamina);
    render_stat_line(ui, "\u{529b}\u{91cf}", C_POWER, gd.power);
    render_stat_line(ui, "\u{6839}\u{6027}", C_GUTS, gd.guts);
    render_stat_line(ui, "\u{667a}\u{529b}", C_WISDOM, gd.wisdom);

    sep(ui);

    // ---- 修正合计 ----
    let rv_total = gd.revised_total();
    colored(ui, C_WHITE, &format!(
        "\u{4fee}\u{6b63}\u{5408}\u{8ba1}: {}  [{}]",
        rv_total, rating(rv_total)
    ));

    // ---- 技能Pt ----
    colored(ui, C_WISDOM, &format!(
        "\u{6280}\u{80fd}Pt: {}", gd.skill_point
    ));

    // ---- 粉丝数 ----
    colored(ui, C_GRAY, &format!(
        "\u{7c89}\u{4e1d}: {}", gd.fan_count
    ));

    sep(ui);

    // ---- 训练推荐 ----
    let best = &gd.trainings[gd.best_training_idx];
    let best_score = best.score;

    heading(ui, "\u{8bad}\u{7ec3}\u{63a8}\u{8350}");

    if best_score <= 0.0 {
        small(ui, "\u{65e0}\u{53ef}\u{7528}\u{8bad}\u{7ec3}\u{6570}\u{636e}");
    } else {
        let tc = train_color(best.command_type);
        colored(ui, C_RECOMMEND, &format!(
            "\u{2605} {} \u{8bad}\u{7ec3}",
            train_name(best.command_type)
        ));
        colored(ui, tc, &format!(
            "  \u{671f}\u{671b}\u{6536}\u{76ca}: +{:.1}", best_score
        ));
        if best.failure_rate > 0 {
            colored(ui, C_MOT2, &format!(
                "  \u{5931}\u{8d25}\u{7387}: {}%", best.failure_rate
            ));
        }
        if best.partner_count > 0 {
            small(ui, &format!(
                "  \u{4f19}\u{4f34}: {} \u{4eba}", best.partner_count
            ));
        }

        // 显示所有训练的分数（降序）
        sep(ui);
        small(ui, "\u{5168}\u{8bad}\u{7ec3}\u{5f97}\u{5206}:");
        let mut order: [usize; MAX_TRAININGS] = [0, 1, 2, 3, 4];
        for _ in 0..MAX_TRAININGS {
            for j in 0..MAX_TRAININGS - 1 {
                if gd.trainings[order[j]].score < gd.trainings[order[j + 1]].score {
                    let tmp = order[j];
                    order[j] = order[j + 1];
                    order[j + 1] = tmp;
                }
            }
        }
        for &idx in &order {
            let t = &gd.trainings[idx];
            if !t.valid || t.score <= 0.0 { continue; }
            let tc2 = train_color(t.command_type);
            let is_best = idx == gd.best_training_idx;
            let marker = if is_best { " \u{2605}" } else { "" };
            colored(ui, tc2, &format!(
                "{}{}: {:.1}  (\u{5931}\u{8d25}{}%)",
                train_name(t.command_type), marker,
                t.score, t.failure_rate
            ));
        }
    }

    // ---- HTTP Server 状态 ----
    sep(ui);
    let port = HTTP_PORT;
    if port > 0 {
        small(ui, &format!("HTTP :{}/data", port));
    }
}

// ============================================================
// HTTP Server
// ============================================================

/// 从 HTTP 请求中提取路径
fn extract_path(request: &str) -> &str {
    if let Some(line) = request.lines().next() {
        let mut parts = line.split_whitespace();
        let _method = parts.next();
        if let Some(path) = parts.next() {
            return path;
        }
    }
    "/"
}

/// 在后台线程启动 HTTP Server
fn start_http_server() {
    let ports: [u16; 2] = [18765, 18767];
    let mut bound_port: u16 = 0;
    let mut bound_listener: Option<TcpListener> = None;

    for &port in &ports {
        match TcpListener::bind(format!("127.0.0.1:{}", port)) {
            Ok(l) => {
                bound_listener = Some(l);
                bound_port = port;
                break;
            }
            Err(_) => continue,
        }
    }

    let listener = match bound_listener {
        Some(l) => l,
        None => {
            // 无法绑定任何端口，记录日志
            unsafe {
                if !VT.is_null() {
                    let msg = CString::new("URA: HTTP server failed to bind port").unwrap();
                    ((*VT).log)(2, b"URA\0".as_ptr() as *const c_char, msg.as_ptr());
                }
            }
            return;
        }
    };

    // 记录实际端口
    unsafe { HTTP_PORT = bound_port; }

    // 日志
    unsafe {
        if !VT.is_null() {
            let msg = CString::new(format!("URA: HTTP server on :{}", bound_port)).unwrap();
            ((*VT).log)(0, b"URA\0".as_ptr() as *const c_char, msg.as_ptr());
        }
    }

    thread::spawn(move || {
        listener.set_nonblocking(false).ok();
        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    // 设置超时避免阻塞
                    let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
                    let _ = stream.set_write_timeout(Some(Duration::from_secs(5)));

                    // 读取请求
                    let mut buf = [0u8; 2048];
                    let _ = stream.read(&mut buf);

                    let request = String::from_utf8_lossy(&buf);
                    let path = extract_path(&request);

                    let body = match path {
                        "/data" => {
                            unsafe {
                                if GAME_DATA.is_null() {
                                    r#"{"error":"no_data"}"#.to_string()
                                } else {
                                    let gd = (*GAME_DATA).lock().unwrap();
                                    gd.to_data_json()
                                }
                            }
                        }
                        "/status" => {
                            r#"{"version":"2.0.0","mode":"hachimi_ura","status":"running"}"#.to_string()
                        }
                        _ => {
                            r#"{"error":"not_found"}"#.to_string()
                        }
                    };

                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    );

                    let _ = stream.write_all(response.as_bytes());
                    let _ = stream.flush();
                }
                Err(_) => continue,
            }
        }
    });
}

// ============================================================
// 后台数据刷新线程
// ============================================================

/// 每 2 秒刷新一次游戏数据到 GAME_DATA
fn start_refresh_thread() {
    thread::spawn(move || {
        // 等待游戏初始化
        loop {
            thread::sleep(Duration::from_secs(1));
            unsafe {
                if META.ok { break; }
            }
        }

        // 初始延迟再等 2 秒确保 IL2CPP 元数据稳定
        thread::sleep(Duration::from_secs(2));

        loop {
            unsafe { refresh_game_data(); }
            thread::sleep(Duration::from_secs(2));
        }
    });
}

// ============================================================
// 回调函数
// ============================================================

/// 游戏初始化完成回调
unsafe extern "C" fn on_game_initialized(_userdata: *mut c_void) {
    if VT.is_null() { return; }
    resolve_meta();

    // 元数据解析完毕后立即尝试刷新一次数据
    if META.ok {
        refresh_game_data();
    }
}

/// 菜单面板渲染回调
extern "C" fn on_menu_section(_userdata: *mut c_void, ui: *mut c_void) {
    unsafe {
        if VT.is_null() || ui.is_null() { return; }
        render_panel(ui);
    }
}

// ============================================================
// 插件入口 — 仅 hachimi_init (V2)
// ============================================================

/// CRITICAL: 只导出 hachimi_init (V2 API)。
/// 不导出 hachimi_init_v3 — 如果存在 v3，Hachimi 会先调它并跳过 V2，
/// 之前的 bug 就是 v3 空实现导致 V2 不被调用。
#[no_mangle]
pub extern "C" fn hachimi_init(vtable: *const Vtable, version: i32) -> InitResult {
    if vtable.is_null() || version < 2 {
        return InitResult::Error;
    }

    unsafe {
        VT = vtable;
    }

    // 初始化全局 GameData（堆上分配，进程生命周期不释放）
    let gd = Box::new(Mutex::new(GameData::empty()));
    unsafe {
        GAME_DATA = Box::into_raw(gd);
    }

    // 启动 HTTP Server（在 hachimi_init 里启动）
    start_http_server();

    // 启动后台数据刷新线程
    start_refresh_thread();

    // 注册回调
    unsafe {
        let vt = &*vtable;

        // 等游戏初始化完再解析 IL2CPP 元数据
        let _ = (vt.hachimi_register_on_game_initialized)(
            Some(on_game_initialized),
            ptr::null_mut(),
        );

        // 注册菜单面板（每次打开菜单时调用 on_menu_section 渲染）
        let _ = (vt.gui_register_menu_section)(
            Some(on_menu_section),
            ptr::null_mut(),
        );

        // 日志：插件已加载
        let gallop_c = CString::new("Gallop").unwrap();
        let msg = CString::new("URA \u{5c0f}\u{9ed1}\u{677f} v2.0.0 loaded").unwrap();
        (vt.log)(0, gallop_c.as_ptr(), msg.as_ptr());
    }

    InitResult::Ok
}

// 注意：hachimi_init_v3 被故意移除。
// 如果存在 v3 导出，Hachimi 会优先调用它；之前 v3 空壳只返回 Ok
// 导致 V2 (hachimi_init) 不被调用，菜单无法注册。
