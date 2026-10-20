//! ===== Factor index badge (育成结束因子序号显示) =====
//!
//! Numbers every factor item on the training-result factor list so the
//! player no longer has to count rows by hand. Purely cosmetic and local:
//! no packets, no server state, no game-data mutation.
//!
//! Mechanism: post-hook TWO entry points, whichever fires last wins:
//!   1. `Gallop.PartsSingleModeResultFactorList.CreateFactorList`
//!   2. `Gallop.PartsSingleModeResultFactorList.PlayInFactorList`
//! After the original populates `_partsList` (List<PartsFactorListContent>),
//! walk it and prefix each item's `_factorName` (TextCommon) text with "N. ".
//!
//! Why two hooks (playin-fix, 2026-10-20): CreateFactorList contains
//! compiler-generated lambdas (b__14_0/b__14_1) and PlayInFactorList returns
//! a byref (UniTask-ish) — item text is written asynchronously AFTER
//! CreateFactorList returns (via SetupFactorNameCommon). Badging at
//! CreateFactorList-return time raced with that deferred text setup:
//! get_text read empty/stale text, the game's later set_text overwrote our
//! prefix (or our bare "N. " clobbered an item), producing broken UI both
//! with and without Hachimi translation. PlayInFactorList fires when the
//! list is actually displayed — by then every item's name is final.
//! Both call sites are idempotent via already_prefixed().
//!
//! v3.28.12 ABI fix (2026-10-20, device crash report): the v3.28.11 detours
//! returned `usize` (x0 only). PlayInFactorList returns UniTask — a 16-byte
//! struct returned in x0:x1 under AAPCS64 — so x1 kept whatever garbage it
//! held and the result screen's async machinery consumed a corrupt UniTask:
//! the game SIGSEGVed the instant 育成 was entered with a pending result
//! screen. u128 occupies exactly x0:x1 and is ignored by void/pointer
//! callers, so one signature now covers all hooked targets.
//!
//! Field offsets verified live on the device, SO v3.28.2, 2026-10-17
//! (via /fields endpoints); re-verified on 3.28.10 runtime, 2026-10-20:
//!   PartsSingleModeResultFactorList._partsList  = +96  (0x60)  List<PartsFactorListContent>
//!   PartsFactorListContent._factorName          = +56  (0x38)  TextCommon
//! IL2CPP List<T>: _items +0x10, _size +0x18; element array data +0x20, stride 8.
//!
//! Red-line compliance: we hook CreateFactorList/PlayInFactorList (game
//! result-screen UI builders) which Hachimi does NOT hook (Hachimi's hooks
//! are text/asset/network functions). We only *call* TextCommon.set_text
//! through a method pointer — calling through Hachimi's hooked function is
//! safe by design (unknown strings pass the translation layer through
//! unchanged).
//!
//! Toggle: GET /api/factor_index/on|off ; status: GET /api/factor_index

use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};

static ENABLED: AtomicBool = AtomicBool::new(true);
static HOOK_ADDR: AtomicUsize = AtomicUsize::new(0);
static PLAYIN_HOOK_ADDR: AtomicUsize = AtomicUsize::new(0);
static SET_TEXT_ADDR: AtomicUsize = AtomicUsize::new(0);
static GET_TEXT_ADDR: AtomicUsize = AtomicUsize::new(0);
static STRING_NEW_UTF16: AtomicUsize = AtomicUsize::new(0);
static APPLIED_LISTS: AtomicU64 = AtomicU64::new(0);
static APPLIED_VIA_CREATE: AtomicU64 = AtomicU64::new(0);
static APPLIED_VIA_PLAYIN: AtomicU64 = AtomicU64::new(0);
static BADGED_ITEMS: AtomicU64 = AtomicU64::new(0);
// 0=ok 1=api 2=set_text/string_new unresolved 3=parts_list null 4=items null
// 5=hook install failed 6=class not found 7=method not found
static LAST_ERROR: AtomicU64 = AtomicU64::new(0);

const OFF_PARTS_LIST: i32 = 0x60;
const OFF_FACTOR_NAME: i32 = 0x38;
const LIST_ITEMS_OFF: i32 = 0x10;
const LIST_SIZE_OFF: i32 = 0x18;
const ARRAY_DATA_OFF: i32 = 0x20;
const MAX_ITEMS: i32 = 64;

pub fn is_enabled() -> bool {
    ENABLED.load(Ordering::Acquire)
}

pub fn set_enabled(value: bool) {
    ENABLED.store(value, Ordering::Release);
}

type FnGetText = unsafe extern "C" fn(*mut c_void) -> *mut c_void;
type FnSetText = unsafe extern "C" fn(*mut c_void, *mut c_void);
type FnStringNewUtf16 = unsafe extern "C" fn(*const u16, i32) -> *mut c_void;

fn already_prefixed(text: &str) -> bool {
    // Detect our own "N. " / "NN. " prefix so re-entry never double-prefixes.
    let bytes = text.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() && i < 3 && bytes[i].is_ascii_digit() {
        i += 1;
    }
    i > 0 && i + 1 < bytes.len() && bytes[i] == b'.' && bytes[i + 1] == b' '
}

unsafe fn apply_badges(list_this: *mut c_void, via_playin: bool) {
    let set_text = SET_TEXT_ADDR.load(Ordering::Acquire);
    let string_new = STRING_NEW_UTF16.load(Ordering::Acquire);
    if set_text == 0 || string_new == 0 {
        LAST_ERROR.store(2, Ordering::Release);
        return;
    }
    let get_text = GET_TEXT_ADDR.load(Ordering::Acquire);

    let parts_list = super::read_ptr_at(list_this, OFF_PARTS_LIST);
    if parts_list.is_null() {
        LAST_ERROR.store(3, Ordering::Release);
        return;
    }
    let items = super::read_ptr_at(parts_list, LIST_ITEMS_OFF);
    if items.is_null() {
        LAST_ERROR.store(4, Ordering::Release);
        return;
    }
    let size = super::read_int_at(parts_list, LIST_SIZE_OFF);
    if size <= 0 {
        return;
    }
    let size = size.min(MAX_ITEMS);

    let setter: FnSetText = std::mem::transmute(set_text);
    let string_new_fn: FnStringNewUtf16 = std::mem::transmute(string_new);

    for i in 0..size {
        let item = super::read_ptr_at(items, ARRAY_DATA_OFF + i * 8);
        if item.is_null() {
            continue;
        }
        let text_common = super::read_ptr_at(item, OFF_FACTOR_NAME);
        if text_common.is_null() {
            continue;
        }

        let mut label = format!("{}. ", i + 1);
        if get_text != 0 {
            let getter: FnGetText = std::mem::transmute(get_text);
            let current = getter(text_common);
            let current_text = super::read_il2cpp_string(current);
            if already_prefixed(&current_text) {
                continue;
            }
            label.push_str(&current_text);
        }
        // Without a getter we still write the bare "N. " prefix; the index is
        // the information the player needs, the name stays visible in the
        // original color block anyway.

        let utf16: Vec<u16> = label.encode_utf16().collect();
        if utf16.is_empty() || utf16.len() > 256 {
            continue;
        }
        let new_str = string_new_fn(utf16.as_ptr(), utf16.len() as i32);
        if new_str.is_null() {
            continue;
        }
        setter(text_common, new_str);
        BADGED_ITEMS.fetch_add(1, Ordering::AcqRel);
    }
    APPLIED_LISTS.fetch_add(1, Ordering::AcqRel);
    if via_playin {
        APPLIED_VIA_PLAYIN.fetch_add(1, Ordering::AcqRel);
    } else {
        APPLIED_VIA_CREATE.fetch_add(1, Ordering::AcqRel);
    }
    LAST_ERROR.store(0, Ordering::Release);
}

// AAPCS64: extra declared parameters are harmless when the real signature
// has fewer. The return type MUST be u128 (x0:x1): PlayInFactorList returns
// UniTask, a 16-byte struct delivered in x0:x1 — a usize detour left x1
// holding garbage and the async continuation crashed on the corrupt value
// (v3.28.11 device crash). u128 covers struct/pointer returns and is
// ignored by void callers.
type FnHookTarget = unsafe extern "C" fn(*mut c_void, usize, usize, usize) -> u128;

unsafe fn run_original_then_badge(
    hook_fn: usize,
    this: *mut c_void,
    a1: usize,
    a2: usize,
    a3: usize,
    via_playin: bool,
) -> u128 {
    let trampoline = super::interceptor_get_trampoline(hook_fn);
    if trampoline == 0 {
        return 0;
    }
    let original: FnHookTarget = std::mem::transmute(trampoline);
    let result = original(this, a1, a2, a3);
    if is_enabled() && !this.is_null() {
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            apply_badges(this, via_playin);
        }));
    }
    result
}

extern "C" fn create_factor_list_hook(
    this: *mut c_void,
    a1: usize,
    a2: usize,
    a3: usize,
) -> u128 {
    unsafe {
        run_original_then_badge(create_factor_list_hook as usize, this, a1, a2, a3, false)
    }
}

extern "C" fn play_in_factor_list_hook(
    this: *mut c_void,
    a1: usize,
    a2: usize,
    a3: usize,
) -> u128 {
    unsafe {
        run_original_then_badge(play_in_factor_list_hook as usize, this, a1, a2, a3, true)
    }
}

unsafe fn resolve_method(class: *mut c_void, names: &[&str], parameter_counts: &[i32]) -> usize {
    let api = &*super::API;
    match api.il2cpp_get_method_addr_fn {
        Some(get_method_addr) => {
            for name in names {
                for count in parameter_counts {
                    let addr =
                        get_method_addr(class as usize, super::to_cstr(name).as_ptr(), *count);
                    if addr != 0 {
                        return addr;
                    }
                }
            }
            0
        }
        None => 0,
    }
}

pub unsafe fn install() {
    if HOOK_ADDR.load(Ordering::Acquire) != 0 {
        return;
    }
    if super::API.is_null() {
        super::set_hook_status("factor_index", "failed: api_null");
        LAST_ERROR.store(1, Ordering::Release);
        return;
    }
    let api = &*super::API;
    if api.interceptor == 0 {
        super::set_hook_status("factor_index", "failed: interceptor_unavailable");
        LAST_ERROR.store(1, Ordering::Release);
        return;
    }
    let get_image = match api.il2cpp_get_assembly_image_fn {
        Some(v) => v,
        None => {
            super::set_hook_status("factor_index", "failed: assembly_api_unavailable");
            LAST_ERROR.store(1, Ordering::Release);
            return;
        }
    };
    let get_class = match api.il2cpp_get_class_fn {
        Some(v) => v,
        None => {
            super::set_hook_status("factor_index", "failed: class_api_unavailable");
            LAST_ERROR.store(1, Ordering::Release);
            return;
        }
    };
    let image = get_image(super::to_cstr("umamusume.dll").as_ptr());
    if image.is_null() {
        super::set_hook_status("factor_index", "failed: image_not_found");
        LAST_ERROR.store(1, Ordering::Release);
        return;
    }

    // il2cpp_string_new_utf16: create Il2CppString from UTF-16 data.
    let string_new = super::resolve_il2cpp_symbol("il2cpp_string_new_utf16");
    if !string_new.is_null() {
        STRING_NEW_UTF16.store(string_new as usize, Ordering::Release);
    }

    // TextCommon text accessors. Hachimi hooks set_text for translations;
    // we only *call* it, never hook it. Try known name casings.
    let text_common = get_class(
        image,
        super::to_cstr("Gallop").as_ptr(),
        super::to_cstr("TextCommon").as_ptr(),
    );
    if !text_common.is_null() {
        let set_addr = resolve_method(text_common, &["set_text", "SetText"], &[1]);
        if set_addr != 0 {
            SET_TEXT_ADDR.store(set_addr, Ordering::Release);
        }
        let get_addr = resolve_method(text_common, &["get_text", "GetText"], &[0]);
        if get_addr != 0 {
            GET_TEXT_ADDR.store(get_addr, Ordering::Release);
        }
    }

    let class = get_class(
        image,
        super::to_cstr("Gallop").as_ptr(),
        super::to_cstr("PartsSingleModeResultFactorList").as_ptr(),
    );
    if class.is_null() {
        super::set_hook_status("factor_index", "failed: class_not_found");
        LAST_ERROR.store(6, Ordering::Release);
        return;
    }
    let create_list = resolve_method(class, &["CreateFactorList"], &[0, 1, 2, 3]);
    if create_list == 0 {
        super::set_hook_status("factor_index", "failed: method_not_found");
        LAST_ERROR.store(7, Ordering::Release);
        return;
    }
    if !super::interceptor_hook(create_list, create_factor_list_hook as usize) {
        super::set_hook_status("factor_index", "failed: interceptor_hook");
        LAST_ERROR.store(5, Ordering::Release);
        return;
    }
    HOOK_ADDR.store(create_list, Ordering::Release);

    // Second hook: PlayInFactorList fires when the list is displayed, after
    // the deferred per-item text setup has run. Failure here is non-fatal —
    // the CreateFactorList hook alone still works on timing-safe flows.
    let play_in = resolve_method(class, &["PlayInFactorList"], &[0]);
    if play_in != 0 && super::interceptor_hook(play_in, play_in_factor_list_hook as usize) {
        PLAYIN_HOOK_ADDR.store(play_in, Ordering::Release);
    }

    super::set_hook_status(
        "factor_index",
        &format!(
            "hooked@0x{:x} playin@0x{:x} set_text@0x{:x} get_text@0x{:x} string_new@0x{:x}",
            create_list,
            PLAYIN_HOOK_ADDR.load(Ordering::Acquire),
            SET_TEXT_ADDR.load(Ordering::Acquire),
            GET_TEXT_ADDR.load(Ordering::Acquire),
            STRING_NEW_UTF16.load(Ordering::Acquire)
        ),
    );
}

pub fn endpoint() -> String {
    format!(
        r#"{{"ok":true,"feature":"factor_index_badge","enabled":{},"installed":{},"hook_addr":"0x{:x}","playin_hook_addr":"0x{:x}","set_text_addr":"0x{:x}","get_text_addr":"0x{:x}","string_new_addr":"0x{:x}","applied_lists":{},"applied_via_create":{},"applied_via_playin":{},"badged_items":{},"last_error":{},"usage":"GET /api/factor_index/on|off to toggle; numbers are added to the result-screen factor list while enabled"}}"#,
        is_enabled(),
        HOOK_ADDR.load(Ordering::Acquire) != 0,
        HOOK_ADDR.load(Ordering::Acquire),
        PLAYIN_HOOK_ADDR.load(Ordering::Acquire),
        SET_TEXT_ADDR.load(Ordering::Acquire),
        GET_TEXT_ADDR.load(Ordering::Acquire),
        STRING_NEW_UTF16.load(Ordering::Acquire),
        APPLIED_LISTS.load(Ordering::Acquire),
        APPLIED_VIA_CREATE.load(Ordering::Acquire),
        APPLIED_VIA_PLAYIN.load(Ordering::Acquire),
        BADGED_ITEMS.load(Ordering::Acquire),
        LAST_ERROR.load(Ordering::Acquire),
    )
}

pub fn enable_endpoint() -> String {
    set_enabled(true);
    endpoint()
}

pub fn disable_endpoint() -> String {
    set_enabled(false);
    endpoint()
}
