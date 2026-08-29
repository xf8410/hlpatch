//! ★ v3.28.0: Career skip — hidden super-high-speed skip setting (IsEnableSuperHighSpeedSkip)
//!
//! Integration points verified live on device (2026-08-29, SO v3.27.23):
//!   Gallop.SaveDataManager (live singleton; `_instance` static @8)
//!     └─ get_SaveLoader() -> Gallop.ApplicationSettingSaveLoader (NON-singleton)
//!          ├─ set_IsEnableSuperHighSpeedSkip(bool) / get_IsEnableSuperHighSpeedSkip() : bool
//!          │    — hidden master switch; writes through SQLiteSaveLoadHelper (local, persisted)
//!          ├─ set_StoryHighSpeedType(i4) / set_TrainingHighSpeedType(i4)
//!          │    — HighSpeedType { None=0, SpeedX1=1, HighSpeedX4=2 }
//!   Gates (read-only, no changes needed):
//!     StoryTimelineController.IsSettingSuperHighSpeedSkip (STATIC) reads the setting
//!     StoryTimelineController.IsSuperHighSpeedSkipMode = setting AND instance state
//!     StoryManager.<IsSkipChoiceOnSuperHighSpeedMode>k__BackingField @101 (byte)
//!       — keep FALSE so event/support choices still pop up (decision safety)
//!
//! Endpoints (register 1 arm in lib.rs handle_http):
//!   GET /skip/status   — read flag + speed types + choice guard + pointers
//!   GET /skip/enable   — flag=true + Story/TrainingHighSpeedType=2 (X4), persisted
//!   GET /skip/disable  — flag=false
//!
//! All calls are local il2cpp_runtime_invoke; zero network traffic.
//! This module is self-contained: it only uses crate-root helpers
//! (find_class_by_short_name / get_singleton / call_getter_* / read_int_at /
//!  resolve_il2cpp_symbol / get_class_from_object / ura_log / get_image).

/// Route entry — return Some(body) if this module owns the path.
pub unsafe fn handle(path: &str) -> Option<String> {
    if path == "/skip/status" {
        Some(skip_status())
    } else if path == "/skip/enable" {
        Some(skip_set(true))
    } else if path == "/skip/disable" {
        Some(skip_set(false))
    } else {
        None
    }
}

const HS_X4: i32 = 2; // HighSpeedType.HighSpeedX4

/// Invoke a void method with one bool (i32-marshalled) arg on an instance.
/// Returns true when il2cpp_runtime_invoke completed without exception.
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
        crate::ura_log(1, "skip: il2cpp invoke symbols missing");
        return false;
    }
    let get_method: crate::FnClassGetMethodFromName = std::mem::transmute(gm_ptr);
    let invoke: crate::FnRuntimeInvoke = std::mem::transmute(inv_ptr);

    let method_info = get_method(class, crate::to_cstr(method_name).as_ptr(), 1);
    if method_info.is_null() {
        crate::ura_log(1, &format!("skip: method '{}' not found", method_name));
        return false;
    }
    // il2cpp_runtime_invoke expects argv entries pointing at unboxed value data.
    // bool param: reads the first byte — i32 0/1 works on little-endian ARM64.
    let mut arg: i32 = if value { 1 } else { 0 };
    let mut args: [*mut c_void; 1] = [&mut arg as *mut i32 as *mut c_void];
    let mut exc: *mut c_void = std::ptr::null_mut();
    let _ = invoke(
        method_info,
        instance as *mut c_void,
        args.as_mut_ptr(),
        &mut exc,
    );
    if !exc.is_null() {
        crate::ura_log(1, &format!("skip: '{}' threw exception", method_name));
        return false;
    }
    true
}

/// Invoke a void method with one i32 arg on an instance.
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
        crate::ura_log(1, &format!("skip: method '{}' not found", method_name));
        return false;
    }
    let mut arg: i32 = value;
    let mut args: [*mut c_void; 1] = [&mut arg as *mut i32 as *mut c_void];
    let mut exc: *mut c_void = std::ptr::null_mut();
    let _ = invoke(
        method_info,
        instance as *mut c_void,
        args.as_mut_ptr(),
        &mut exc,
    );
    exc.is_null()
}

struct SkipContext {
    loader: *const c_void,
    loader_class: *mut c_void,
    sd_class: *mut c_void,
    sd_instance: *const c_void,
}

/// Resolve SaveDataManager -> get_SaveLoader() -> loader instance,
/// and derive the loader's EXACT class from the object header
/// (two ApplicationSettingSaveLoader classes exist: Gallop ns + global ns —
///  header-derived class removes the ambiguity entirely).
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
        sd_class,
        sd_instance,
    })
}

/// Read StoryManager choice-guard flag (offset 101 byte). Null when no story active.
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
        return "null".to_string(); // not in a story/career scene right now
    }
    let v = crate::read_int_at(sm_inst, 101);
    if v == 0 {
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
            let story_hs = crate::call_getter_int(
                ctx.loader_class,
                ctx.loader,
                "get_StoryHighSpeedType",
            );
            let train_hs = crate::call_getter_int(
                ctx.loader_class,
                ctx.loader,
                "get_TrainingHighSpeedType",
            );
            format!(
                r#"{{"ok":true,"enabled":{},"story_high_speed_type":{},"training_high_speed_type":{},"choice_guard_101":{},"save_data_manager":"{:p}","save_loader":"{:p}"}},"#,
                enabled, story_hs, train_hs,
                read_choice_guard(), ctx.sd_instance, ctx.loader
            )
            .trim_end_matches(",}")
            .to_string()
                + "}"
        }
    }
}

unsafe fn skip_set(enable: bool) -> String {
    match resolve_skip_context() {
        Err(e) => format!(r#"{{"ok":false,"error":"{}"}}"#, e),
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
                // Bump both scene speed ladders to the X4 tier while we are at it.
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
            // Read back for verification (authoritative: the game's own getter).
            let enabled = crate::call_getter_bool(
                ctx.loader_class,
                ctx.loader,
                "get_IsEnableSuperHighSpeedSkip",
            );
            let story_hs = crate::call_getter_int(
                ctx.loader_class,
                ctx.loader,
                "get_StoryHighSpeedType",
            );
            let train_hs = crate::call_getter_int(
                ctx.loader_class,
                ctx.loader,
                "get_TrainingHighSpeedType",
            );
            if set_flag_ok && enabled != enable {
                // getter disagrees with the setter — surface it loudly
                crate::ura_log(
                    1,
                    &format!(
                        "skip: setter ok={} but readback {} != {}",
                        set_flag_ok, enabled, enable
                    ),
                );
                set_flag_ok = false;
            }
            crate::ura_log(
                3,
                &format!(
                    "skip: enable={} set_ok={} readback={} story_hs={} train_hs={}",
                    enable, set_flag_ok, enabled, story_hs, train_hs
                ),
            );
            format!(
                r#"{{"ok":{},"requested":{},"readback_enabled":{},"story_high_speed_type":{},"training_high_speed_type":{},"story_hs_set":{},"train_hs_set":{},"choice_guard_101":{},"persisted":"SQLiteSaveLoadHelper(local)"}}"#,
                set_flag_ok,
                enable,
                enabled,
                story_hs,
                train_hs,
                story_hs_set.map(|b| b.to_string()).unwrap_or_else(|| "skipped".to_string()),
                train_hs_set.map(|b| b.to_string()).unwrap_or_else(|| "skipped".to_string()),
                read_choice_guard()
            )
        }
    }
}
