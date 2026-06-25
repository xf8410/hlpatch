#![allow(dead_code)]
//! URA 小黑板 v3.0.0
//! Hachimi Edge 插件 — 赛马娘育成数据实时显示与训练推荐
//!
//! 核心功能:
//! - 通过 IL2CPP API 实时读取育成数据
//! - HTTP Server (:18765) 推送数据给 uma-juece 浮窗App
//! - Hachimi GUI 面板显示五维属性与训练推荐
//!
//! 关键设计决策:
//! - 使用 hachimi_init_v3 (V3 API)，通过 get_api 按名称查找函数指针
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
// V3 API: 通过 get_api 按名称查找的函数指针
// 所有函数指针在 hachimi_init_v3 中一次性解析并缓存
// ============================================================

struct Api {
    // Core
    hachimi_instance: unsafe fn() -> *const c_void,
    hachimi_get_interceptor: unsafe fn(*const c_void) -> *const c_void,
    // Interceptor
    interceptor_hook: unsafe fn(*const c_void, *mut c_void, *mut c_void) -> *mut c_void,
    interceptor_get_trampoline_addr: unsafe fn(*const c_void, *mut c_void) -> *mut c_void,
    interceptor_unhook: unsafe fn(*const c_void, *mut c_void) -> *mut c_void,
    // IL2CPP
    il2cpp_get_assembly_image: unsafe fn(*const c_char) -> *const c_void,
    il2cpp_get_class: unsafe fn(*const c_void, *const c_char, *const c_char) -> *mut c_void,
    il2cpp_get_method_addr: unsafe fn(*mut c_void, *const c_char, i32) -> *mut c_void,
    il2cpp_get_method: unsafe fn(*mut c_void, *const c_char, i32) -> *const c_void,
    il2cpp_get_field_from_name: unsafe fn(*mut c_void, *const c_char) -> *mut c_void,
    il2cpp_get_field_value: unsafe fn(*mut c_void, *mut c_void, *mut c_void),
    il2cpp_get_static_field_value: unsafe fn(*mut c_void, *mut c_void),
    il2cpp_set_field_value: unsafe fn(*mut c_void, *mut c_void, *const c_void),
    il2cpp_get_singleton_like_instance: unsafe fn(*mut c_void) -> *mut c_void,
    il2cpp_resolve_symbol: unsafe fn(*const c_char) -> *mut c_void,
    il2cpp_object_new: unsafe fn(*const c_void) -> *mut c_void,
    il2cpp_unbox: unsafe fn(*mut c_void) -> *mut c_void,
    il2cpp_runtime_object_init: unsafe fn(*mut c_void),
    il2cpp_string_new: unsafe fn(*const c_char) -> *mut c_void,
    il2cpp_string_chars: unsafe fn(*mut c_void) -> *mut u16,
    il2cpp_string_length: unsafe fn(*mut c_void) -> i32,
    il2cpp_create_array: unsafe fn(*mut c_void, usize) -> *mut c_void,
    il2cpp_schedule_on_thread: unsafe fn(*mut c_void, *const c_void),
    il2cpp_get_main_thread: unsafe fn() -> *mut c_void,
    il2cpp_get_attached_threads: unsafe fn() -> *mut c_void,
    il2cpp_find_nested_class: unsafe fn(*mut c_void, *const c_char) -> *mut c_void,
    // Log
    log: unsafe fn(i32, *const c_char, *const c_char),
    // GUI
    gui_register_menu_item: unsafe fn(*const c_char, Option<extern "C" fn(*mut c_void)>, *mut c_void) -> bool,
    gui_register_menu_section: unsafe fn(*const c_char, Option<extern "C" fn(*mut c_void, *mut c_void)>, *mut c_void) -> bool,
    gui_show_notification: unsafe fn(*const c_char) -> bool,
    gui_ui_heading: unsafe fn(*mut c_void, *const c_char) -> bool,
    gui_ui_label: unsafe fn(*mut c_void, *const c_char) -> bool,
    gui_ui_small: unsafe fn(*mut c_void, *const c_char) -> bool,
    gui_ui_separator: unsafe fn(*mut c_void) -> bool,
    gui_ui_button: unsafe fn(*mut c_void, *const c_char) -> bool,
    gui_ui_colored_label: unsafe fn(*mut c_void, u8, u8, u8, u8, *const c_char) -> bool,
    gui_ui_checkbox: unsafe fn(*mut c_void, *const c_char, *mut bool) -> bool,
    gui_ui_text_edit_singleline: unsafe fn(*mut c_void, *mut c_char, usize) -> bool,
    gui_ui_horizontal: unsafe fn(*mut c_void, Option<extern "C" fn(*mut c_void, *mut c_void)>, *mut c_void) -> bool,
    gui_ui_grid: unsafe fn(*mut c_void, *const c_char, usize, f32, f32, Option<extern "C" fn(*mut c_void, *mut c_void)>, *mut c_void) -> bool,
    gui_ui_end_row: unsafe fn(*mut c_void) -> bool,
    gui_ui_combo_menu: unsafe fn(*mut c_void, *const c_char, *mut i32, *const *const c_char, usize, *mut c_char, usize) -> bool,
    gui_new_window_id: unsafe fn() -> i32,
    gui_show_window: unsafe fn(i32, *const c_char, Option<extern "C" fn(*mut c_void, *mut c_void)>, Option<extern "C" fn(*mut c_void, *mut c_void)>, *mut c_void) -> bool,
    gui_close_window: unsafe fn(i32),
    gui_get_menu_width: unsafe fn() -> f32,
    gui_set_menu_width: unsafe fn(f32),
    // Hachimi
    hachimi_get_base_dir: unsafe fn() -> *const c_char,
    hachimi_get_data_path: unsafe fn() -> *const c_char,
    hachimi_register_on_game_initialized: unsafe fn(Option<extern "C" fn(*mut c_void)>, *mut c_void) -> bool,
    hachimi_register_present_callback: unsafe fn(Option<extern "C" fn(*mut c_void, *mut c_void)>, *mut c_void) -> bool,
}

/// 通过 get_api 查找并转换函数指针
/// # Safety: get_api 必须指向有效的 Hachimi V3 get_api 函数
unsafe fn resolve_api(get_api: extern "C" fn(name: *const c_char) -> *mut c_void) -> Option<Api> {
    macro_rules! api_fn {
        ($name:expr, $ty:ty) => {{
            let cname = CString::new($name).unwrap();
            let ptr = get_api(cname.as_ptr());
            if ptr.is_null() {
                return None;
            }
            ::std::mem::transmute::<*mut c_void, $ty>(ptr)
        }};
    }

    Some(Api {
        hachimi_instance: api_fn!("hachimi_instance", unsafe fn() -> *const c_void),
        hachimi_get_interceptor: api_fn!("hachimi_get_interceptor", unsafe fn(*const c_void) -> *const c_void),
        interceptor_hook: api_fn!("interceptor_hook", unsafe fn(*const c_void, *mut c_void, *mut c_void) -> *mut c_void),
        interceptor_get_trampoline_addr: api_fn!("interceptor_get_trampoline_addr", unsafe fn(*const c_void, *mut c_void) -> *mut c_void),
        interceptor_unhook: api_fn!("interceptor_unhook", unsafe fn(*const c_void, *mut c_void) -> *mut c_void),
        il2cpp_get_assembly_image: api_fn!("il2cpp_get_assembly_image", unsafe fn(*const c_char) -> *const c_void),
        il2cpp_get_class: api_fn!("il2cpp_get_class", unsafe fn(*const c_void, *const c_char, *const c_char) -> *mut c_void),
        il2cpp_get_method_addr: api_fn!("il2cpp_get_method_addr", unsafe fn(*mut c_void, *const c_char, i32) -> *mut c_void),
        il2cpp_get_method: api_fn!("il2cpp_get_method", unsafe fn(*mut c_void, *const c_char, i32) -> *const c_void),
        il2cpp_get_field_from_name: api_fn!("il2cpp_get_field_from_name", unsafe fn(*mut c_void, *const c_char) -> *mut c_void),
        il2cpp_get_field_value: api_fn!("il2cpp_get_field_value", unsafe fn(*mut c_void, *mut c_void, *mut c_void)),
        il2cpp_get_static_field_value: api_fn!("il2cpp_get_static_field_value", unsafe fn(*mut c_void, *mut c_void)),
        il2cpp_set_field_value: api_fn!("il2cpp_set_field_value", unsafe fn(*mut c_void, *mut c_void, *const c_void)),
        il2cpp_get_singleton_like_instance: api_fn!("il2cpp_get_singleton_like_instance", unsafe fn(*mut c_void) -> *mut c_void),
        il2cpp_resolve_symbol: api_fn!("il2cpp_resolve_symbol", unsafe fn(*const c_char) -> *mut c_void),
        il2cpp_object_new: api_fn!("il2cpp_object_new", unsafe fn(*const c_void) -> *mut c_void),
        il2cpp_unbox: api_fn!("il2cpp_unbox", unsafe fn(*mut c_void) -> *mut c_void),
        il2cpp_runtime_object_init: api_fn!("il2cpp_runtime_object_init", unsafe fn(*mut c_void)),
        il2cpp_string_new: api_fn!("il2cpp_string_new", unsafe fn(*const c_char) -> *mut c_void),
        il2cpp_string_chars: api_fn!("il2cpp_string_chars", unsafe fn(*mut c_void) -> *mut u16),
        il2cpp_string_length: api_fn!("il2cpp_string_length", unsafe fn(*mut c_void) -> i32),
        il2cpp_create_array: api_fn!("il2cpp_create_array", unsafe fn(*mut c_void, usize) -> *mut c_void),
        il2cpp_schedule_on_thread: api_fn!("il2cpp_schedule_on_thread", unsafe fn(*mut c_void, *const c_void)),
        il2cpp_get_main_thread: api_fn!("il2cpp_get_main_thread", unsafe fn() -> *mut c_void),
        il2cpp_get_attached_threads: api_fn!("il2cpp_get_attached_threads", unsafe fn() -> *mut c_void),
        il2cpp_find_nested_class: api_fn!("il2cpp_find_nested_class", unsafe fn(*mut c_void, *const c_char) -> *mut c_void),
        log: api_fn!("log", unsafe fn(i32, *const c_char, *const c_char)),
        gui_register_menu_item: api_fn!("gui_register_menu_item", unsafe fn(*const c_char, Option<extern "C" fn(*mut c_void)>, *mut c_void) -> bool),
        gui_register_menu_section: api_fn!("gui_register_menu_section", unsafe fn(*const c_char, Option<extern "C" fn(*mut c_void, *mut c_void)>, *mut c_void) -> bool),
        gui_show_notification: api_fn!("gui_show_notification", unsafe fn(*const c_char) -> bool),
        gui_ui_heading: api_fn!("gui_ui_heading", unsafe fn(*mut c_void, *const c_char) -> bool),
        gui_ui_label: api_fn!("gui_ui_label", unsafe fn(*mut c_void, *const c_char) -> bool),
        gui_ui_small: api_fn!("gui_ui_small", unsafe fn(*mut c_void, *const c_char) -> bool),
        gui_ui_separator: api_fn!("gui_ui_separator", unsafe fn(*mut c_void) -> bool),
        gui_ui_button: api_fn!("gui_ui_button", unsafe fn(*mut c_void, *const c_char) -> bool),
        gui_ui_colored_label: api_fn!("gui_ui_colored_label", unsafe fn(*mut c_void, u8, u8, u8, u8, *const c_char) -> bool),
        gui_ui_checkbox: api_fn!("gui_ui_checkbox", unsafe fn(*mut c_void, *const c_char, *mut bool) -> bool),
        gui_ui_text_edit_singleline: api_fn!("gui_ui_text_edit_singleline", unsafe fn(*mut c_void, *mut c_char, usize) -> bool),
        gui_ui_horizontal: api_fn!("gui_ui_horizontal", unsafe fn(*mut c_void, Option<extern "C" fn(*mut c_void, *mut c_void)>, *mut c_void) -> bool),
        gui_ui_grid: api_fn!("gui_ui_grid", unsafe fn(*mut c_void, *const c_char, usize, f32, f32, Option<extern "C" fn(*mut c_void, *mut c_void)>, *mut c_void) -> bool),
        gui_ui_end_row: api_fn!("gui_ui_end_row", unsafe fn(*mut c_void) -> bool),
        gui_ui_combo_menu: api_fn!("gui_ui_combo_menu", unsafe fn(*mut c_void, *const c_char, *mut i32, *const *const c_char, usize, *mut c_char, usize) -> bool),
        gui_new_window_id: api_fn!("gui_new_window_id", unsafe fn() -> i32),
        gui_show_window: api_fn!("gui_show_window", unsafe fn(i32, *const c_char, Option<extern "C" fn(*mut c_void, *mut c_void)>, Option<extern "C" fn(*mut c_void, *mut c_void)>, *mut c_void) -> bool),
        gui_close_window: api_fn!("gui_close_window", unsafe fn(i32)),
        gui_get_menu_width: api_fn!("gui_get_menu_width", unsafe fn() -> f32),
        gui_set_menu_width: api_fn!("gui_set_menu_width", unsafe fn(f32)),
        hachimi_get_base_dir: api_fn!("hachimi_get_base_dir", unsafe fn() -> *const c_char),
        hachimi_get_data_path: api_fn!("hachimi_get_data_path", unsafe fn() -> *const c_char),
        hachimi_register_on_game_initialized: api_fn!("hachimi_register_on_game_initialized", unsafe fn(Option<extern "C" fn(*mut c_void)>, *mut c_void) -> bool),
        hachimi_register_present_callback: api_fn!("hachimi_register_present_callback", unsafe fn(Option<extern "C" fn(*mut c_void, *mut c_void)>, *mut c_void) -> bool),
    })
}

// ============================================================
// 全局状态
// ============================================================

/// 全局 API 指针（堆上分配，进程生命周期不释放）
static mut API: *const Api = ptr::null();

/// 日志辅助函数
unsafe fn ura_log(level: i32, msg: &str) {
    if API.is_null() { return; }
    let cmsg = CString::new(msg).unwrap_or_else(|_| CString::new("<log error>").unwrap());
    ((*API).log)(level, b"URA\0".as_ptr() as *const c_char, cmsg.as_ptr());
}

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
    ((*API).il2cpp_get_field_value)(obj, field, &mut v as *mut _ as *mut c_void);
    v
}

/// 读取引用类型字段（返回对象指针）
unsafe fn read_obj(obj: *mut c_void, field: *mut c_void) -> *mut c_void {
    if obj.is_null() || field.is_null() { return ptr::null_mut(); }
    let mut v: *mut c_void = ptr::null_mut();
    ((*API).il2cpp_get_field_value)(obj, field, &mut v as *mut _ as *mut c_void);
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
    let ns_c = CString::new(ns).unwrap();
    let name_c = CString::new(name).unwrap();
    let mut klass = ((*API).il2cpp_get_class)(image, ns_c.as_ptr(), name_c.as_ptr());
    if klass.is_null() {
        let empty = CString::new("").unwrap();
        klass = ((*API).il2cpp_get_class)(image, empty.as_ptr(), name_c.as_ptr());
    }
    klass
}

unsafe fn resolve_field(klass: *mut c_void, name: &str) -> *mut c_void {
    if klass.is_null() { return ptr::null_mut(); }
    let name_c = CString::new(name).unwrap();
    ((*API).il2cpp_get_field_from_name)(klass, name_c.as_ptr())
}

unsafe fn resolve_field_multi(klass: *mut c_void, names: &[&str]) -> *mut c_void {
    for &name in names {
        let f = resolve_field(klass, name);
        if !f.is_null() { return f; }
    }
    ptr::null_mut()
}

unsafe fn resolve_meta() {
    // ===== 阶段4a: 解析IL2CPP元数据 =====
    ura_log(3, "URA: resolve_meta started");

    let gallop_c = CString::new("Gallop").unwrap();
    let image = ((*API).il2cpp_get_assembly_image)(gallop_c.as_ptr());
    if image.is_null() {
        ura_log(2, "URA: Gallop image is NULL - IL2CPP not ready?");
        return;
    }
    ura_log(3, &format!("URA: Gallop image = {:p}", image));

    META.smd_cls    = resolve_class(image, "Gallop", "SingleModeData");
    META.smci_cls   = resolve_class(image, "Gallop", "SingleModeCharaInfo");
    META.smcmdi_cls = resolve_class(image, "Gallop", "SingleModeCommandInfo");
    META.smpidci_cls = resolve_class(image, "Gallop", "SingleModeParamsIncDecInfo");
    META.sm_model_cls = resolve_class(image, "Gallop", "SingleModel");

    // 日志：各class解析结果
    ura_log(3, &format!(
        "URA: classes - SMD={:?} SMCI={:?} SMCDI={:?} SMPIDCI={:?} SMModel={:?}",
        META.smd_cls, META.smci_cls, META.smcmdi_cls, META.smpidci_cls, META.sm_model_cls
    ));

    if META.smci_cls.is_null() {
        ura_log(2, "URA: CRITICAL - CharaInfo class is NULL, cannot read game data");
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
    
    // ===== 阶段4b: 字段解析结果 =====
    ura_log(3, &format!(
        "URA: metadata resolved - fields: chara_info={:?} cmd_array={:?} speed={:?} stamina={:?} power={:?} guts={:?} wisdom={:?} motivation={:?} energy={:?} max_energy={:?} turn={:?} skill_pt={:?} fan={:?}",
        META.f_chara_info, META.f_command_array,
        META.f_speed, META.f_stamina, META.f_power, META.f_guts, META.f_wisdom,
        META.f_motivation, META.f_energy, META.f_max_energy,
        META.f_turn, META.f_skill_point, META.f_fan_count
    ));
}

// ============================================================
// 实时数据读取
// ============================================================

/// 获取 SingleModeData 实例（尝试多种路径）
unsafe fn get_smd_instance() -> *mut c_void {
    // 路径1: SingleModeData 作为 singleton
    if !META.smd_cls.is_null() {
        let inst = ((*API).il2cpp_get_singleton_like_instance)(META.smd_cls);
        if !inst.is_null() { return inst; }
    }

    // 路径2: SingleModel singleton -> Data 字段
    if !META.sm_model_cls.is_null() && !META.f_data.is_null() {
        let model = ((*API).il2cpp_get_singleton_like_instance)(META.sm_model_cls);
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

        // training_partner_array -> 统计伙伴数量
        let partner_arr = read_obj(cmd_obj, META.f_training_partner_array);
        cmd.partner_count = arr_len(partner_arr);

        // params_inc_dec_info_array -> 累加增益
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
// 数据刷新 -> 更新全局 GameData
// ============================================================

/// 从 IL2CPP 读取最新游戏数据并更新 GAME_DATA
/// 安全性：API/META 在初始化后只读，并发访问实际安全
unsafe fn refresh_game_data() {
    if API.is_null() || !META.ok { return; }

    let smd = get_smd_instance();
    if smd.is_null() {
        // 育成模式未激活，保持旧数据但标记不可用
        if GAME_DATA.is_null() { return; }
        let mut gd = (*GAME_DATA).lock().unwrap();
        gd.available = false;
        return;
    }
    
    // 首次拿到smd实例时记录
    {
        static FIRST_SMD: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
        if !FIRST_SMD.swap(true, std::sync::atomic::Ordering::Relaxed) {
            ura_log(3, &format!("URA: first SMD instance = {:p}", smd));
        }
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
        1 => "绝不调",
        2 => "不调",
        3 => "普通",
        4 => "好调",
        5 => "绝好调",
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

/// 回合 -> (年份, 月份, 是否前半)
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
        101 => "速度",
        105 => "耐力",
        102 => "力量",
        103 => "根性",
        106 => "智力",
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
    ((*API).gui_ui_colored_label)(ui, c.0, c.1, c.2, AA, t.as_ptr());
}

unsafe fn label(ui: *mut c_void, text: &str) {
    let t = to_cstr(text);
    ((*API).gui_ui_label)(ui, t.as_ptr());
}

unsafe fn small(ui: *mut c_void, text: &str) {
    let t = to_cstr(text);
    ((*API).gui_ui_small)(ui, t.as_ptr());
}

unsafe fn heading(ui: *mut c_void, text: &str) {
    let t = to_cstr(text);
    ((*API).gui_ui_heading)(ui, t.as_ptr());
}

unsafe fn sep(ui: *mut c_void) {
    ((*API).gui_ui_separator)(ui);
}

/// 渲染五维属性行
unsafe fn render_stat_line(ui: *mut c_void, name: &str, color: (u8,u8,u8), val: i32) {
    let rv = GameData::revise(val);
    if val > 1200 {
        colored(ui, color, &format!("{}  {} (修正{})", name, val, rv));
    } else {
        colored(ui, color, &format!("{}  {}", name, val));
    }
}

/// 主面板渲染 — 从 GAME_DATA 读取数据
unsafe fn render_panel(ui: *mut c_void) {
    heading(ui, "URA 小黑板");
    sep(ui);

    if !META.ok {
        label(ui, "等待游戏初始化...");
        small(ui, "进入育成模式后可查看数据");
        return;
    }

    // 刷新数据（面板打开时触发即时读取）
    refresh_game_data();

    // 从全局 GAME_DATA 读取
    if GAME_DATA.is_null() {
        label(ui, "内部错误");
        return;
    }

    let gd = (*GAME_DATA).lock().unwrap();
    let gd = &*gd;

    if !gd.available {
        label(ui, "暂无育成数据");
        small(ui, "请先进入育成模式");
        NOTIFICATION_SENT = false;
        return;
    }

    // 首次检测到育成数据时推送通知
    if !NOTIFICATION_SENT {
        let msg = to_cstr("URA 小黑板已激活");
        ((*API).gui_show_notification)(msg.as_ptr());
        NOTIFICATION_SENT = true;
    }

    // ---- 回合信息 ----
    let (year, month, first_half) = turn_to_ym(gd.turn);
    let half_str = if first_half { "前半" } else { "后半" };
    colored(ui, C_WHITE, &format!("{}年目 {}月{}  Turn {}", year, month, half_str, gd.turn));

    // ---- 体力 ----
    let max_e = if gd.max_energy > 0 { gd.max_energy } else { 1 };
    let e_ratio = gd.energy as f32 / max_e as f32;
    let e_pct = (e_ratio * 100.0) as i32;
    let ec = energy_color(e_ratio);
    colored(ui, ec, &format!("体力: {}/{} ({}%)", gd.energy, max_e, e_pct));

    // ---- 干劲 ----
    let mc = mot_color(gd.motivation);
    colored(ui, mc, &format!("干劲: {}  ×{:.1}", mot_text(gd.motivation), mot_mult(gd.motivation)));

    sep(ui);

    // ---- 五维属性 ----
    render_stat_line(ui, "速度", C_SPEED, gd.speed);
    render_stat_line(ui, "耐力", C_STAMINA, gd.stamina);
    render_stat_line(ui, "力量", C_POWER, gd.power);
    render_stat_line(ui, "根性", C_GUTS, gd.guts);
    render_stat_line(ui, "智力", C_WISDOM, gd.wisdom);

    sep(ui);

    // ---- 修正合计 ----
    let rv_total = gd.revised_total();
    colored(ui, C_WHITE, &format!("修正合计: {}  [{}]", rv_total, rating(rv_total)));

    // ---- 技能Pt ----
    colored(ui, C_WISDOM, &format!("技能Pt: {}", gd.skill_point));

    // ---- 粉丝数 ----
    colored(ui, C_GRAY, &format!("粉丝: {}", gd.fan_count));

    sep(ui);

    // ---- 训练推荐 ----
    let best = &gd.trainings[gd.best_training_idx];
    let best_score = best.score;

    heading(ui, "训练推荐");

    if best_score <= 0.0 {
        small(ui, "无可用训练数据");
    } else {
        let tc = train_color(best.command_type);
        colored(ui, C_RECOMMEND, &format!("★ {} 训练", train_name(best.command_type)));
        colored(ui, tc, &format!("  期望收益: +{:.1}", best_score));
        if best.failure_rate > 0 {
            colored(ui, C_MOT2, &format!("  失败率: {}%", best.failure_rate));
        }
        if best.partner_count > 0 {
            small(ui, &format!("  伙伴: {} 人", best.partner_count));
        }

        // 显示所有训练的分数（降序）
        sep(ui);
        small(ui, "全训练得分:");
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
            let marker = if is_best { " ★" } else { "" };
            colored(ui, tc2, &format!("{}{}: {:.1}  (失败{}%)", train_name(t.command_type), marker, t.score, t.failure_rate));
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
            unsafe { ura_log(2, "URA: HTTP server failed to bind port"); }
            return;
        }
    };

    // 记录实际端口
    unsafe { HTTP_PORT = bound_port; }

    unsafe { ura_log(3, &format!("URA: HTTP server started on :{}", bound_port)); }

    thread::spawn(move || {
        listener.set_nonblocking(false).ok();
        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
                    let _ = stream.set_write_timeout(Some(Duration::from_secs(5)));

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
                            r#"{"version":"3.0.0","mode":"hachimi_ura","status":"running"}"#.to_string()
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
        let mut waited = 0u32;
        loop {
            thread::sleep(Duration::from_secs(1));
            waited += 1;
            unsafe {
                if META.ok { break; }
            }
            if waited > 120 {
                unsafe { ura_log(2, "URA: refresh thread timeout - META never ready"); }
                return;
            }
        }
        
        unsafe { ura_log(3, &format!("URA: refresh thread started after {}s", waited)); }

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
    if API.is_null() { return; }
    
    ura_log(3, "URA: on_game_initialized triggered");
    
    resolve_meta();

    if META.ok {
        refresh_game_data();
        unsafe {
            if !GAME_DATA.is_null() {
                let gd = (*GAME_DATA).lock().unwrap();
                ura_log(3, &format!(
                    "URA: first data read - turn={}, spd={}, sta={}, pow={}, gut={}, wis={}",
                    gd.turn, gd.speed, gd.stamina, gd.power, gd.guts, gd.wisdom
                ));
            }
        }
    } else {
        ura_log(2, "URA: metadata resolve FAILED");
    }
}

/// 菜单面板渲染回调
extern "C" fn on_menu_section(_userdata: *mut c_void, ui: *mut c_void) {
    unsafe {
        if API.is_null() || ui.is_null() { return; }
        render_panel(ui);
    }
}

// ============================================================
// 插件入口 — hachimi_init_v3 (V3 API)
// ============================================================

/// V3 API 入口：通过 get_api 按名称查找函数指针
/// Hachimi Edge v0.26.3+ 优先调用此函数
#[no_mangle]
pub unsafe extern "C" fn hachimi_init_v3(
    get_api: extern "C" fn(name: *const c_char) -> *mut c_void,
    version: i32,
) -> i32 {
    // 解析所有 API 函数指针
    let api = match resolve_api(get_api) {
        Some(a) => a,
        None => {
            // 无法解析必需 API，致命错误
            return InitResult::Error as i32;
        }
    };

    // 堆分配 API 结构体，进程生命周期不释放
    API = Box::into_raw(Box::new(api));

    ura_log(3, &format!("URA: hachimi_init_v3 called, version={}", version));

    // 初始化全局 GameData（堆上分配，进程生命周期不释放）
    let gd = Box::new(Mutex::new(GameData::empty()));
    GAME_DATA = Box::into_raw(gd);

    // 启动 HTTP Server
    start_http_server();

    // 启动后台数据刷新线程
    start_refresh_thread();

    // 注册回调
    let _ = ((*API).hachimi_register_on_game_initialized)(
        Some(on_game_initialized),
        ptr::null_mut(),
    );

    // 注册菜单面板（每次打开菜单时调用 on_menu_section 渲染）
    let label_c = to_cstr("URA 小黑板");
    let _ = ((*API).gui_register_menu_section)(
        label_c.as_ptr(),
        Some(on_menu_section),
        ptr::null_mut(),
    );

    ura_log(3, "URA 小黑板 v3.0.0 fully loaded - callbacks registered");
    
    // 发送通知
    let notif = to_cstr("URA小黑板 v3.0.0 已加载");
    ((*API).gui_show_notification)(notif.as_ptr());

    InitResult::Ok as i32
}
