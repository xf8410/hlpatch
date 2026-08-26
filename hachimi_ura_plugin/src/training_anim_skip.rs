//! ===== Training animation skip (D-T1) =====
//!
//! Auto-skips in-training cut-in animations (育成内演出) by hooking the
//! game's own cut-in helper entry point and invoking the built-in skip
//! routine on the same instance right after the original Init completes.
//!
//! Primary target : Gallop.SingleModeTrainingCutInHelper.Init -> .SkipRuntime
//! Secondary      : Gallop.CampaignTrainingCutInHelper.Init  -> .SkipRuntime (best effort)
//!
//! Safety: disabled by default. Flip with GET /api/training/anim_skip?enabled=1
//! Status: GET /api/training/anim_skip

use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};

static ENABLED: AtomicBool = AtomicBool::new(false);
static INIT_ADDR: AtomicUsize = AtomicUsize::new(0);
static SKIP_ADDR: AtomicUsize = AtomicUsize::new(0);
static CAMPAIGN_INIT_ADDR: AtomicUsize = AtomicUsize::new(0);
static CAMPAIGN_SKIP_ADDR: AtomicUsize = AtomicUsize::new(0);
static SKIP_CALLS: AtomicU64 = AtomicU64::new(0);
static LAST_ERROR_TS: AtomicU64 = AtomicU64::new(0);

pub fn is_enabled() -> bool {
    ENABLED.load(Ordering::Acquire)
}

pub fn set_enabled(value: bool) {
    ENABLED.store(value, Ordering::Release);
}

extern "C" fn init_hook(this: *mut c_void) -> *const c_void {
    unsafe {
        let trampoline = super::interceptor_get_trampoline(init_hook as usize);
        if trampoline == 0 {
            return std::ptr::null();
        }
        type FnType = unsafe extern "C" fn(*mut c_void) -> *const c_void;
        let original: FnType = std::mem::transmute(trampoline);
        let result = original(this);
        // Auto-skip right after the cut-in helper initializes. SkipRuntime is
        // the same routine the in-game skip button triggers; calling it here
        // finishes the cut-in instantly without touching result state.
        if is_enabled() && this != std::ptr::null_mut() {
            let skip = SKIP_ADDR.load(Ordering::Acquire);
            if skip != 0 {
                type SkipFn = unsafe extern "C" fn(*mut c_void);
                let skip_fn: SkipFn = std::mem::transmute(skip);
                skip_fn(this);
                SKIP_CALLS.fetch_add(1, Ordering::AcqRel);
            }
        }
        result
    }
}

extern "C" fn campaign_init_hook(this: *mut c_void) -> *const c_void {
    unsafe {
        let trampoline = super::interceptor_get_trampoline(campaign_init_hook as usize);
        if trampoline == 0 {
            return std::ptr::null();
        }
        type FnType = unsafe extern "C" fn(*mut c_void) -> *const c_void;
        let original: FnType = std::mem::transmute(trampoline);
        let result = original(this);
        if is_enabled() && this != std::ptr::null_mut() {
            let skip = CAMPAIGN_SKIP_ADDR.load(Ordering::Acquire);
            if skip != 0 {
                type SkipFn = unsafe extern "C" fn(*mut c_void);
                let skip_fn: SkipFn = std::mem::transmute(skip);
                skip_fn(this);
                SKIP_CALLS.fetch_add(1, Ordering::AcqRel);
            }
        }
        result
    }
}

unsafe fn resolve_method(class: *mut c_void, name: &str, parameter_counts: &[i32]) -> usize {
    let api = &*super::API;
    match api.il2cpp_get_method_addr_fn {
        Some(get_method_addr) => {
            for count in parameter_counts {
                let addr = get_method_addr(class as usize, super::to_cstr(name).as_ptr(), *count);
                if addr != 0 {
                    return addr;
                }
            }
            0
        }
        None => 0,
    }
}

fn note_error(reason: &str) {
    let _ = reason;
    LAST_ERROR_TS.store(super::sniff_timestamp_ms(), Ordering::Release);
}

fn install_pair(
    label: &str,
    namespace_name: &str,
    class_name: &str,
    init_out: &AtomicUsize,
    skip_out: &AtomicUsize,
) -> bool {
    unsafe {
        if super::API.is_null() {
            super::set_hook_status(label, "failed: api_null");
            return false;
        }
        let api = &*super::API;
        if api.interceptor == 0 {
            super::set_hook_status(label, "failed: interceptor_unavailable");
            return false;
        }
        let get_image = match api.il2cpp_get_assembly_image_fn {
            Some(v) => v,
            None => {
                super::set_hook_status(label, "failed: assembly_api_unavailable");
                return false;
            }
        };
        let get_class = match api.il2cpp_get_class_fn {
            Some(v) => v,
            None => {
                super::set_hook_status(label, "failed: class_api_unavailable");
                return false;
            }
        };
        let image = get_image(super::to_cstr("umamusume.dll").as_ptr());
        if image.is_null() {
            super::set_hook_status(label, "failed: image_not_found");
            return false;
        }
        let class = get_class(image, super::to_cstr(namespace_name).as_ptr(), super::to_cstr(class_name).as_ptr());
        if class.is_null() {
            super::set_hook_status(label, "failed: class_not_found");
            return false;
        }
        // Init/SkipRuntime are typically parameterless; probe 0 then 1 args.
        let skip = resolve_method(class, "SkipRuntime", &[0, 1]);
        if skip == 0 {
            super::set_hook_status(label, "failed: skip_runtime_unresolved");
            return false;
        }
        skip_out.store(skip, Ordering::Release);
        if init_out.load(Ordering::Acquire) != 0 {
            super::set_hook_status(label, "already_installed");
            return true;
        }
        let init = resolve_method(class, "Init", &[0, 1]);
        if init == 0 || !super::interceptor_hook(init, init_hook as usize) {
            super::set_hook_status(label, "failed: init_resolve_or_hook");
            note_error("init");
            return false;
        }
        init_out.store(init, Ordering::Release);
        super::set_hook_status(label, &format!("hooked@0x{:x} skip@0x{:x}", init, skip));
        true
    }
}

pub unsafe fn install() {
    let primary_ok = install_pair(
        "training.anim_skip",
        "Gallop",
        "SingleModeTrainingCutInHelper",
        &INIT_ADDR,
        &SKIP_ADDR,
    );
    // Best effort: campaign ticket training uses a separate helper class in
    // some builds. Failure here is non-fatal.
    let _campaign_ok = install_pair(
        "training.anim_skip.campaign",
        "Gallop",
        "CampaignTrainingCutInHelper",
        &CAMPAIGN_INIT_ADDR,
        &CAMPAIGN_SKIP_ADDR,
    );
    if !primary_ok && INIT_ADDR.load(Ordering::Acquire) == 0 {
        note_error("primary_install");
    }
}

pub fn endpoint(uri: &str) -> String {
    let pairs = match super::parse_query_pairs(uri) {
        Ok(value) => value,
        Err(error) => return super::k_json_error(&error),
    };
    let enabled_text = super::query_pair(&pairs, "enabled");
    if !enabled_text.is_empty() {
        match enabled_text.as_str() {
            "1" | "true" | "on" => set_enabled(true),
            "0" | "false" | "off" => set_enabled(false),
            other => {
                return format!(
                    r#"{{"ok":false,"error":"invalid_enabled_value","value":"{}"}}"#,
                    super::json_escape(other)
                )
            }
        }
    }
    let installed = INIT_ADDR.load(Ordering::Acquire) != 0;
    let campaign_installed = CAMPAIGN_INIT_ADDR.load(Ordering::Acquire) != 0;
    format!(
        r#"{{"ok":true,"feature":"training_anim_skip","enabled":{},"installed":{},"campaign_installed":{},"skip_calls":{},"last_error_ts_ms":{},"usage":"GET ?enabled=1|0 to toggle; skips run only while enabled"}}"#,
        is_enabled(),
        installed,
        campaign_installed,
        SKIP_CALLS.load(Ordering::Acquire),
        LAST_ERROR_TS.load(Ordering::Acquire),
    )
}
