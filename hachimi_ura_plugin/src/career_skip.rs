//! v3.27.25: Career auto-skip — force-apply super-high-speed on live timelines.
//!
//! v3.27.24 lesson (device-verified): flipping the persisted hidden setting
//! (IsEnableSuperHighSpeedSkip) is NOT sufficient — the game only reads it to
//! decide whether the PLAYER may engage skip; the actual fast-forward is a
//! per-scene runtime state on Gallop.StoryTimelineController:
//!   - instance bool _isApplySuperHighSpeedSkip  (dump offset 868)
//!   - static  HighSpeedType _highSpeedType      (None=0, SpeedX1=1, HighSpeedX4=2)
//!   - static  bool IsAutoPlay                   (auto-advance waits)
//! The game sets those only when the player taps the skip button. This module
//! now replicates that automatically: a watcher finds the live controller via
//! UnityEngine.Object.FindObjectOfType and force-applies the trio whenever a
//! timeline scene (career event / support training cut-in) is on screen.
//!
//! Endpoints (register 1 arm in lib.rs handle_http):
//!   GET /skip/status   — setting + speed types + choice guard + watcher state
//!   GET /skip/enable   — persist hidden setting + start watcher + apply now
//!   GET /skip/apply    — force-apply on the current scene right now
//!   GET /skip/disable  — stop watcher + unset persisted setting
//!
//! All calls are local il2cpp; zero network traffic. Choices are never
//! written by us (StoryManager +101 and IsSkipMultiChoice untouched).

use std::ffi::{c_void, CStr};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Once;

static SKIP_ENABLED: AtomicBool = AtomicBool::new(false);
static WATCHER_ONCE: Once = Once::new();
static UO_CLASS: AtomicUsize = AtomicUsize::new(0);

const HS_X4: i32 = 2; // HighSpeedType.HighSpeedX4

/// Route entry — return Some(body) if this module owns the path.
/// Safe fn: the lib.rs route chain runs in a safe context (E0133 otherwise).
/// Panic guard mirrors lib.rs's own read_summary pattern.
pub fn handle(path: &str) -> Option<String> {
    match path {
        "/skip/status" => Some(
            std::panic::catch_unwind(|| unsafe { skip_status() })
                .unwrap_or_else(|_| r#"{"ok":false,"error":"career_skip_panic_caught"}"#.to_string()),
        ),
        "/skip/enable" => Some(
            std::panic::catch_unwind(|| unsafe { skip_set(true) })
                .unwrap_or_else(|_| r#"{"ok":false,"error":"career_skip_panic_caught"}"#.to_string()),
        ),
        "/skip/apply" => Some(
            std::panic::catch_unwind(|| unsafe { skip_apply_json() })
                .unwrap_or_else(|_| r#"{"ok":false,"error":"career_skip_panic_caught"}"#.to_string()),
        ),
        "/skip/disable" => Some(
            std::panic::catch_unwind(|| unsafe { skip_set(false) })
                .unwrap_or_else(|_| r#"{"ok":false,"error":"career_skip_panic_caught"}"#.to_string()),
        ),
        _ => None,
    }
}

// ───────────────────────── raw il2cpp plumbing ─────────────────────────

unsafe fn sym(name: &str) -> *mut c_void {
    crate::resolve_il2cpp_symbol(name)
}

type FnDomainGet = unsafe extern "C" fn() -> *mut c_void;
type FnDomainGetAssemblies = unsafe extern "C" fn(*mut c_void, *mut usize) -> *mut *mut c_void;
type FnAssemblyGetImage = unsafe extern "C" fn(*const c_void) -> *mut c_void;
type FnImageGetName = unsafe extern "C" fn(*const c_void) -> *const c_char;
type FnClassFromName = unsafe extern "C" fn(*const c_void, *const c_char, *const c_char) -> *mut c_void;
type FnClassGetType = unsafe extern "C" fn(*mut c_void) -> *mut c_void;
type FnTypeGetObject = unsafe extern "C" fn(*mut c_void) -> *mut c_void;
type FnClassGetFieldFromName = unsafe extern "C" fn(*mut c_void, *const c_char) -> *mut c_void;
type FnFieldGetValue = unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void);
type FnFieldSetValue = unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void);
type FnFieldStaticGetValue = unsafe extern "C" fn(*mut c_void, *mut c_void);
type FnFieldStaticSetValue = unsafe extern "C" fn(*mut c_void, *mut c_void);
type FnThreadAttach = unsafe extern "C" fn(*mut c_void) -> *mut c_void;

/// UnityEngine.Object class, resolved once (lives in UnityEngine.CoreModule).
unsafe fn unity_object_class() -> *mut c_void {
    let cached = UO_CLASS.load(Ordering::Relaxed);
    if cached != 0 {
        return cached as *mut c_void;
    }
    let domain_get: FnDomainGet = std::mem::transmute(sym("il2cpp_domain_get"));
    let get_asms: FnDomainGetAssemblies =
        std::mem::transmute(sym("il2cpp_domain_get_assemblies"));
    let asm_get_image: FnAssemblyGetImage =
        std::mem::transmute(sym("il2cpp_assembly_get_image"));
    let img_get_name: FnImageGetName = std::mem::transmute(sym("il2cpp_image_get_name"));
    let class_from_name: FnClassFromName = std::mem::transmute(sym("il2cpp_class_from_name"));
    if domain_get.is_null() || get_asms.is_null() {
        return std::ptr::null_mut();
    }
    let dom = domain_get();
    let mut size: usize = 0;
    let asms = get_asms(dom, &mut size);
    if asms.is_null() {
        return std::ptr::null_mut();
    }
    for i in 0..size {
        let asm = *asms.add(i);
        if asm.is_null() {
            continue;
        }
        let img = asm_get_image(asm);
        if img.is_null() {
            continue;
        }
        let name_ptr = img_get_name(img);
        if name_ptr.is_null() {
            continue;
        }
        let name = CStr::from_ptr(name_ptr).to_string_lossy();
        if name.starts_with("UnityEngine.CoreModule") {
            let k = class_from_name(
                img,
                b"UnityEngine\0".as_ptr() as *const c_char,
                b"Object\0".as_ptr() as *const c_char,
            );
            if !k.is_null() {
                UO_CLASS.store(k as usize, Ordering::Relaxed);
            }
            return k;
        }
    }
    std::ptr::null_mut()
}

/// Live StoryTimelineController instance via FindObjectOfType(Type).
/// Null between scenes — that's expected and harmless.
unsafe fn find_timeline_instance(tl_class: *mut c_void) -> *mut c_void {
    let uo = unity_object_class();
    if uo.is_null() {
        return std::ptr::null_mut();
    }
    let class_get_type: FnClassGetType = std::mem::transmute(sym("il2cpp_class_get_type"));
    let type_get_object: FnTypeGetObject = std::mem::transmute(sym("il2cpp_type_get_object"));
    let t = class_get_type(tl_class);
    if t.is_null() {
        return std::ptr::null_mut();
    }
    let type_obj = type_get_object(t);
    if type_obj.is_null() {
        return std::ptr::null_mut();
    }
    let gm_ptr = crate::resolve_il2cpp_symbol("il2cpp_class_get_method_from_name");
    let inv_ptr = crate::resolve_il2cpp_symbol("il2cpp_runtime_invoke");
    if gm_ptr.is_null() || inv_ptr.is_null() {
        return std::ptr::null_mut();
    }
    let get_method: crate::FnClassGetMethodFromName = std::mem::transmute(gm_ptr);
    let invoke: crate::FnRuntimeInvoke = std::mem::transmute(inv_ptr);
    let m = get_method(uo, crate::to_cstr("FindObjectOfType").as_ptr(), 1);
    if m.is_null() {
        return std::ptr::null_mut();
    }
    let mut args: [*mut c_void; 1] = [type_obj];
    let mut exc: *mut c_void = std::ptr::null_mut();
    let r = invoke(m, std::ptr::null_mut(), args.as_mut_ptr(), &mut exc);
    if !exc.is_null() {
        return std::ptr::null_mut();
    }
    r
}

/// Force-apply the super-high-speed trio on the live timeline instance.
/// Returns (applied_bitmask, instance_ptr). Bit 1 = _isApplySuperHighSpeedSkip,
/// bit 2 = static _highSpeedType=X4, bit 4 = static IsAutoPlay.
/// Nothing is written when the state is already correct (idempotent).
unsafe fn force_apply_inner() -> Result<(u32, *mut c_void), String> {
    let image = crate::get_image();
    if image.is_null() {
        return Err("image_null".to_string());
    }
    let tl = crate::find_class_by_short_name(image, "StoryTimelineController");
    if tl.is_null() {
        return Err("no_StoryTimelineController_class".to_string());
    }
    let inst = find_timeline_instance(tl);
    if inst.is_null() {
        return Err("no_active_timeline_scene".to_string());
    }
    let getf: FnClassGetFieldFromName =
        std::mem::transmute(sym("il2cpp_class_get_field_from_name"));
    let fget: FnFieldGetValue = std::mem::transmute(sym("il2cpp_field_get_value"));
    let fset: FnFieldSetValue = std::mem::transmute(sym("il2cpp_field_set_value"));
    let sget: FnFieldStaticGetValue = std::mem::transmute(sym("il2cpp_field_static_get_value"));
    let sset: FnFieldStaticSetValue = std::mem::transmute(sym("il2cpp_field_static_set_value"));
    if getf.is_null() || fget.is_null() || fset.is_null() || sget.is_null() || sset.is_null() {
        return Err("il2cpp_field_symbols_missing".to_string());
    }

    let mut applied: u32 = 0;

    // 1) instance bool _isApplySuperHighSpeedSkip = true
    let f = getf(tl, crate::to_cstr("_isApplySuperHighSpeedSkip").as_ptr());
    if !f.is_null() {
        let mut buf = [0u8; 8];
        fget(inst, f, buf.as_mut_ptr() as *mut c_void);
        if buf[0] == 0 {
            let one: u8 = 1;
            fset(inst, f, &one as *const u8 as *mut c_void);
            applied |= 1;
        }
    }

    // 2) static HighSpeedType _highSpeedType = HighSpeedX4
    let fh = getf(tl, crate::to_cstr("_highSpeedType").as_ptr());
    if !fh.is_null() {
        let mut v: i32 = 0;
        sget(fh, &mut v as *mut i32 as *mut c_void);
        if v != HS_X4 {
            v = HS_X4;
            sset(fh, &v as *mut i32 as *mut c_void);
            applied |= 2;
        }
    }

    // 3) static bool IsAutoPlay = true (auto-advance waits)
    let fa = getf(tl, crate::to_cstr("IsAutoPlay").as_ptr());
    if !fa.is_null() {
        let mut b = [0u8; 1];
        sget(fa, b.as_mut_ptr() as *mut c_void);
        if b[0] == 0 {
            let one: u8 = 1;
            sset(fa, &one as *const u8 as *mut c_void);
            applied |= 4;
        }
    }

    Ok((applied, inst))
}

unsafe fn force_apply_json() -> String {
    match force_apply_inner() {
        Err(e) => format!(r#"{{"ok":false,"error":"{}"}}"#, e),
        Ok((mask, inst)) => format!(
            r#"{{"ok":true,"applied_mask":{},"applied":{{"super_high_speed":{},"high_speed_x4":{},"auto_play":{}}},"instance":"{:p}"}}"#,
            mask,
            mask & 1 != 0,
            mask & 2 != 0,
            mask & 4 != 0,
            inst
        ),
    }
}

fn ensure_watcher() {
    WATCHER_ONCE.call_once(|| {
        std::thread::spawn(|| unsafe {
            watcher_loop();
        });
    });
}

unsafe fn watcher_loop() {
    // Attach this thread to the il2cpp runtime once.
    let attach: FnThreadAttach = std::mem::transmute(sym("il2cpp_thread_attach"));
    let domain_get: FnDomainGet = std::mem::transmute(sym("il2cpp_domain_get"));
    if !attach.is_null() && !domain_get.is_null() {
        let _ = attach(domain_get());
    }
    loop {
        if SKIP_ENABLED.load(Ordering::Relaxed) {
            let _ = std::panic::catch_unwind(|| unsafe { force_apply_tick() });
        }
        std::thread::sleep(std::time::Duration::from_millis(1500));
    }
}

/// One watcher tick: apply and log only when something actually changed.
unsafe fn force_apply_tick() {
    match force_apply_inner() {
        Ok((mask, _)) => {
            if mask != 0 {
                crate::ura_log(
                    3,
                    &format!("skip: force-applied super-high-speed mask={}", mask),
                );
            }
        }
        Err(_) => {} // no scene active / transient — silent
    }
}

// ───────────────────────── save-loader setting (persistence) ─────────────────────────

struct SkipContext {
    loader: *const c_void,
    loader_class: *mut c_void,
    sd_instance: *const c_void,
}

unsafe fn invoke_set_bool(
    class: *mut c_void,
    instance: *const c_void,
    method_name: &str,
    value: bool,
) -> bool {
    if class.is_null() || instance.is_null() {
        return false;
    }
    let gm_ptr = crate::resolve_il2cpp_symbol("il2cpp_class_get_method_from_name");
    let inv_ptr = crate::resolve_il2cpp_symbol("il2cpp_runtime_invoke");
    if gm_ptr.is_null() || inv_ptr.is_null() {
        return false;
    }
    let get_method: crate::FnClassGetMethodFromName = std::mem::transmute(gm_ptr);
    let invoke: crate::FnRuntimeInvoke = std::mem::transmute(inv_ptr);
    let method_info = get_method(class, crate::to_cstr(method_name).as_ptr(), 1);
    if method_info.is_null() {
        return false;
    }
    let mut arg: i32 = if value { 1 } else { 0 };
    let mut args: [*mut c_void; 1] = [&mut arg as *mut i32 as *mut c_void];
    let mut exc: *mut c_void = std::ptr::null_mut();
    let _ = invoke(method_info, instance as *mut c_void, args.as_mut_ptr(), &mut exc);
    exc.is_null()
}

unsafe fn invoke_set_int(
    class: *mut c_void,
    instance: *const c_void,
    method_name: &str,
    value: i32,
) -> bool {
    if class.is_null() || instance.is_null() {
        return false;
    }
    let gm_ptr = crate::resolve_il2cpp_symbol("il2cpp_class_get_method_from_name");
    let inv_ptr = crate::resolve_il2cpp_symbol("il2cpp_runtime_invoke");
    if gm_ptr.is_null() || inv_ptr.is_null() {
        return false;
    }
    let get_method: crate::FnClassGetMethodFromName = std::mem::transmute(gm_ptr);
    let invoke: crate::FnRuntimeInvoke = std::mem::transmute(inv_ptr);
    let method_info = get_method(class, crate::to_cstr(method_name).as_ptr(), 1);
    if method_info.is_null() {
        return false;
    }
    let mut arg: i32 = value;
    let mut args: [*mut c_void; 1] = [&mut arg as *mut i32 as *mut c_void];
    let mut exc: *mut c_void = std::ptr::null_mut();
    let _ = invoke(method_info, instance as *mut c_void, args.as_mut_ptr(), &mut exc);
    exc.is_null()
}

/// Resolve SaveDataManager -> get_SaveLoader() -> loader instance, with the
/// loader's EXACT class derived from the object header.
unsafe fn resolve_skip_context() -> Result<SkipContext, String> {
    let image = crate::get_image();
    if image.is_null() {
        return Err("image_null".to_string());
    }
    let sd_class = crate::find_class_by_short_name(image, "SaveDataManager");
    if sd_class.is_null() {
        return Err("no_SaveDataManager_class".to_string());
    }
    let sd_instance = crate::get_singleton(sd_class);
    if sd_instance.is_null() {
        return Err("no_SaveDataManager_singleton".to_string());
    }
    let loader = crate::call_getter_ref(sd_class, sd_instance, "get_SaveLoader");
    if loader.is_null() {
        return Err("get_SaveLoader_null".to_string());
    }
    let loader_class = crate::get_class_from_object(loader);
    if loader_class.is_null() {
        return Err("loader_class_null".to_string());
    }
    Ok(SkipContext {
        loader,
        loader_class,
        sd_instance,
    })
}

/// Read StoryManager choice-guard flag (byte @101). "null" when no story active.
unsafe fn read_choice_guard() -> String {
    let image = crate::get_image();
    if image.is_null() {
        return "null".to_string();
    }
    let sm_class = crate::find_class_by_short_name(image, "StoryManager");
    if sm_class.is_null() {
        return "class_null".to_string();
    }
    let sm_inst = crate::get_singleton(sm_class);
    if sm_inst.is_null() {
        return "null".to_string();
    }
    if crate::read_int_at(sm_inst, 101) == 0 {
        "false".to_string()
    } else {
        "true".to_string()
    }
}

unsafe fn skip_status() -> String {
    match resolve_skip_context() {
        Err(e) => format!(r#"{{"ok":false,"error":"{}"}}"#, e),
        Ok(ctx) => {
            let enabled = crate::call_getter_bool(
                ctx.loader_class,
                ctx.loader,
                "get_IsEnableSuperHighSpeedSkip",
            );
            let story_hs =
                crate::call_getter_int(ctx.loader_class, ctx.loader, "get_StoryHighSpeedType");
            let train_hs = crate::call_getter_int(
                ctx.loader_class,
                ctx.loader,
                "get_TrainingHighSpeedType",
            );
            format!(
                r#"{{"ok":true,"enabled":{},"watcher_active":{},"story_high_speed_type":{},"training_high_speed_type":{},"choice_guard_101":{},"save_data_manager":"{:p}","save_loader":"{:p}"}}"#,
                enabled,
                SKIP_ENABLED.load(Ordering::Relaxed),
                story_hs,
                train_hs,
                read_choice_guard(),
                ctx.sd_instance,
                ctx.loader
            )
        }
    }
}

unsafe fn skip_set(enable: bool) -> String {
    // Watcher gate first so enable takes effect this instant.
    SKIP_ENABLED.store(enable, Ordering::Relaxed);
    if enable {
        ensure_watcher();
    }
    match resolve_skip_context() {
        Err(e) => format!(r#"{{"ok":false,"error":"{}","watcher_active":{},"note":"watcher state applied; save-loader unresolved"}}"#,
            e, SKIP_ENABLED.load(Ordering::Relaxed)),
        Ok(ctx) => {
            let mut set_flag_ok = invoke_set_bool(
                ctx.loader_class,
                ctx.loader,
                "set_IsEnableSuperHighSpeedSkip",
                enable,
            );
            let mut story_hs_set: Option<bool> = None;
            let mut train_hs_set: Option<bool> = None;
            if enable {
                story_hs_set = Some(invoke_set_int(
                    ctx.loader_class,
                    ctx.loader,
                    "set_StoryHighSpeedType",
                    HS_X4,
                ));
                train_hs_set = Some(invoke_set_int(
                    ctx.loader_class,
                    ctx.loader,
                    "set_TrainingHighSpeedType",
                    HS_X4,
                ));
            }
            let enabled = crate::call_getter_bool(
                ctx.loader_class,
                ctx.loader,
                "get_IsEnableSuperHighSpeedSkip",
            );
            if set_flag_ok && enabled != enable {
                set_flag_ok = false;
            }
            if enable {
                ensure_watcher();
                // Apply to whatever scene is on screen right now.
                let _ = std::panic::catch_unwind(|| unsafe { force_apply_tick() });
            }
            crate::ura_log(
                3,
                &format!(
                    "skip: enable={} set_ok={} readback={} watcher={}",
                    enable,
                    set_flag_ok,
                    enabled,
                    SKIP_ENABLED.load(Ordering::Relaxed)
                ),
            );
            format!(
                r#"{{"ok":{},"requested":{},"readback_enabled":{},"watcher_active":{},"story_high_speed_type":{},"training_high_speed_type":{},"story_hs_set":{},"train_hs_set":{},"choice_guard_101":{},"persisted":"SQLiteSaveLoadHelper(local)","note":"watcher force-applies super-high-speed on every timeline scene while active"}}"#,
                set_flag_ok,
                enable,
                enabled,
                SKIP_ENABLED.load(Ordering::Relaxed),
                crate::call_getter_int(ctx.loader_class, ctx.loader, "get_StoryHighSpeedType"),
                crate::call_getter_int(ctx.loader_class, ctx.loader, "get_TrainingHighSpeedType"),
                story_hs_set
                    .map(|b| b.to_string())
                    .unwrap_or_else(|| "\"skipped\"".to_string()),
                train_hs_set
                    .map(|b| b.to_string())
                    .unwrap_or_else(|| "\"skipped\"".to_string()),
                read_choice_guard()
            )
        }
    }
}
