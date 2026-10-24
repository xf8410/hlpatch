//! ===== Factor index badge (育成结束因子序号显示) =====
//!
//! Numbers every factor item on the training-result factor lists so the
//! player no longer has to count rows by hand. Purely cosmetic and local:
//! no packets, no server state, no game-data mutation.
//!
//! Mechanism: post-hook the UI builders, then walk the list and prefix each
//! item's `_factorName` (TextCommon) text with "N. ":
//!   1. `Gallop.PartsSingleModeResultFactorList.CreateFactorList`
//!   2. `Gallop.PartsSingleModeResultFactorList.PlayInFactorList`
//!   3. `...FactorListCarouselScrollItem.UpdateFactorList` (因子選択 建表)
//!   4. `...FactorListCarouselScrollItem.UpdateItem`      (翻页/滚动重显)
//! Both call sites are idempotent via already_prefixed().
//!
//! v3.28.12 ABI fix: the v3.28.11 detours returned `usize` (x0 only).
//! PlayInFactorList returns UniTask — a 16-byte struct returned in x0:x1 under
//! AAPCS64 — so x1 kept garbage and the async continuation consumed a corrupt
//! UniTask (SIGSEGV). u128 occupies exactly x0:x1.
//!
//! v3.28.14 crash hardening (three independent native-crash vectors removed):
//!   a) LATE-INSTALL GUARD. Installing an interceptor while the target function
//!      is on another thread's stack corrupts the process. The only late
//!      installer is the /api/event/choices HTTP lazy retry, which fires
//!      minutes into a session. We now record the first install attempt and
//!      refuse to install once the boot window (120s) has passed: feature is
//!      lost for that session, stability is not.
//!   b) CLASS GATE. `UpdateItem` is declared on `CarouselScrollItemBase<T>` —
//!      `il2cpp_class_get_method_from_name` walks base classes, so the address
//!      we hook can be shared by unrelated carousels (home banners, gacha...).
//!      Walking +0x90 of a foreign object reads a garbage "List" and derefs it.
//!      Every detour now verifies `*(Il2CppClass**)this == expected class`.
//!   c) POINTER SANITY. List pointer / items array / element pointers are
//!      range- and alignment-checked, size is bounded before any deref, and a
//!      NULL get_text result is never decoded. Plus a thread-local re-entry
//!      guard: our own set_text can trigger a layout rebuild that calls the
//!      hooked UI method again.
//!
//! Field offsets verified live on the device (uma_get_fields, stable build,
//! 2026-10-24):
//!   PartsSingleModeResultFactorList._partsList                  = +96  (0x60)
//!   ...FactorListCarouselScrollItem._partsFactorListContentList = +144 (0x90)
//!   PartsFactorListContent._factorName                          = +56  (0x38)
//! IL2CPP List<T>: _items +0x10, _size +0x18; array data +0x20, stride 8.
//! IL2CPP object header: klass* at +0x00.
//!
//! Red-line compliance: we hook game result-screen UI builders which Hachimi
//! does NOT hook. We only *call* TextCommon.set_text through a method pointer;
//! hooking set_text itself (as an external review suggested) would hijack the
//! single text path used by every label in the game and is deliberately NOT
//! done here.
//!
//! Toggle: GET /api/factor_index/on|off ; status: GET /api/factor_index

use std::cell::Cell;
use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::OnceLock;
use std::time::Instant;

static ENABLED: AtomicBool = AtomicBool::new(true);
static HOOK_ADDR: AtomicUsize = AtomicUsize::new(0);
static PLAYIN_HOOK_ADDR: AtomicUsize = AtomicUsize::new(0);
static SELECT_UPDATE_ADDR: AtomicUsize = AtomicUsize::new(0);
static SELECT_UPDATE_ITEM_ADDR: AtomicUsize = AtomicUsize::new(0);
static SET_TEXT_ADDR: AtomicUsize = AtomicUsize::new(0);
static GET_TEXT_ADDR: AtomicUsize = AtomicUsize::new(0);
static STRING_NEW_UTF16: AtomicUsize = AtomicUsize::new(0);
// Class identities used for the per-call gate (IL2CPP object header, +0x00).
static LIST_CLASS: AtomicUsize = AtomicUsize::new(0);
static SELECT_CLASS: AtomicUsize = AtomicUsize::new(0);
// First install() attempt — bounds the window in which patching is safe.
static FIRST_INSTALL_ATTEMPT: OnceLock<Instant> = OnceLock::new();
static APPLIED_LISTS: AtomicU64 = AtomicU64::new(0);
static APPLIED_VIA_CREATE: AtomicU64 = AtomicU64::new(0);
static APPLIED_VIA_PLAYIN: AtomicU64 = AtomicU64::new(0);
static APPLIED_VIA_SELECT_UPDATE: AtomicU64 = AtomicU64::new(0);
static APPLIED_VIA_SELECT_ITEM: AtomicU64 = AtomicU64::new(0);
static BADGED_ITEMS: AtomicU64 = AtomicU64::new(0);
static SKIPPED_FOREIGN: AtomicU64 = AtomicU64::new(0);
static SKIPPED_BADPTR: AtomicU64 = AtomicU64::new(0);
static REENTRY_SKIPPED: AtomicU64 = AtomicU64::new(0);
// 0=ok 1=api 2=set_text/string_new unresolved 3=parts_list null/bogus
// 4=items null/bogus 5=hook install failed 6=class not found 7=method not found
// 8=late install refused, 9=foreign object / klass mismatch
static LAST_ERROR: AtomicU64 = AtomicU64::new(0);

const OFF_PARTS_LIST: i32 = 0x60;
const OFF_PARTS_LIST_CAROUSEL: i32 = 0x90;
const OFF_FACTOR_NAME: i32 = 0x38;
const LIST_ITEMS_OFF: i32 = 0x10;
const LIST_SIZE_OFF: i32 = 0x18;
const ARRAY_DATA_OFF: i32 = 0x20;
const MAX_ITEMS: i32 = 64;
const OBJ_KLASS_OFF: i32 = 0x00;
// Installing an interceptor after this many seconds from the first attempt is
// refused: by then the game's UI loops are live and patching them in place is
// a data race that shows up as a silent native SIGSEGV.
const LATE_INSTALL_WINDOW_SECS: u64 = 120;

thread_local! {
    // >0 while we are inside apply_badges on this thread.
    static BADGE_DEPTH: Cell<u8> = const { Cell::new(0) };
}

pub fn is_enabled() -> bool {
    ENABLED.load(Ordering::Acquire)
}

pub fn set_enabled(value: bool) {
    ENABLED.store(value, Ordering::Release);
}

/// True once at least one result-screen hook is live. Used by the boot
/// installer in lib.rs to know whether it may stop retrying.
pub fn installed() -> bool {
    HOOK_ADDR.load(Ordering::Acquire) != 0
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

/// Cheap plausibility test for an IL2CPP managed pointer before we deref it:
/// non-null, 8-byte aligned, inside the user/kernel-mapped user range.
fn plausible(p: *mut c_void) -> bool {
    let a = p as usize;
    a >= 0x10000 && a < 0x0000_7fff_ffff_ffff && (a & 0x7) == 0
}

/// Does `this` really belong to `expected_klass`? IL2CPP stores the
/// Il2CppClass* at object+0x00, so this is one aligned pointer read.
unsafe fn klass_matches(this: *mut c_void, expected_klass: usize) -> bool {
    if expected_klass == 0 {
        return true; // gate not armed (shouldn't happen after install)
    }
    let actual = super::read_ptr_at(this, OBJ_KLASS_OFF) as usize;
    actual == expected_klass
}

unsafe fn apply_badges(
    list_this: *mut c_void,
    parts_off: i32,
    expected_klass: usize,
    counter: &AtomicU64,
) {
    if list_this.is_null() {
        return;
    }
    if !klass_matches(list_this, expected_klass) {
        SKIPPED_FOREIGN.fetch_add(1, Ordering::Relaxed);
        return;
    }

    let set_text = SET_TEXT_ADDR.load(Ordering::Acquire);
    let string_new = STRING_NEW_UTF16.load(Ordering::Acquire);
    if set_text == 0 || string_new == 0 {
        LAST_ERROR.store(2, Ordering::Release);
        return;
    }
    let get_text = GET_TEXT_ADDR.load(Ordering::Acquire);

    let parts_list = super::read_ptr_at(list_this, parts_off);
    if !plausible(parts_list) {
        LAST_ERROR.store(3, Ordering::Release);
        SKIPPED_BADPTR.fetch_add(1, Ordering::Relaxed);
        return;
    }
    let size = super::read_int_at(parts_list, LIST_SIZE_OFF);
    if size <= 0 || size > MAX_ITEMS {
        // Bogus size means this is not a List we understand — never iterate.
        LAST_ERROR.store(4, Ordering::Release);
        SKIPPED_BADPTR.fetch_add(1, Ordering::Relaxed);
        return;
    }
    let items = super::read_ptr_at(parts_list, LIST_ITEMS_OFF);
    if !plausible(items) {
        LAST_ERROR.store(4, Ordering::Release);
        SKIPPED_BADPTR.fetch_add(1, Ordering::Relaxed);
        return;
    }

    let setter: FnSetText = std::mem::transmute(set_text);
    let string_new_fn: FnStringNewUtf16 = std::mem::transmute(string_new);

    for i in 0..size {
        let slot = ARRAY_DATA_OFF + i * 8;
        if (slot as usize) > 0x10000 {
            break;
        }
        let item = super::read_ptr_at(items, slot);
        if !plausible(item) {
            continue;
        }
        let text_common = super::read_ptr_at(item, OFF_FACTOR_NAME);
        if !plausible(text_common) {
            continue;
        }

        let mut label = format!("{}. ", i + 1);
        if get_text != 0 {
            let getter: FnGetText = std::mem::transmute(get_text);
            let current = getter(text_common);
            if current.is_null() {
                // No readable text yet — leave the row alone instead of
                // writing a bare "N. " over a label that is still being set.
                continue;
            }
            let current_text = super::read_il2cpp_string(current);
            if already_prefixed(&current_text) {
                continue;
            }
            if current_text.is_empty() {
                continue;
            }
            label.push_str(&current_text);
        }

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
    counter.fetch_add(1, Ordering::AcqRel);
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
    parts_off: i32,
    expected_klass: usize,
    counter: &AtomicU64,
) -> u128 {
    let trampoline = super::interceptor_get_trampoline(hook_fn);
    if trampoline == 0 {
        return 0;
    }
    let original: FnHookTarget = std::mem::transmute(trampoline);
    let result = original(this, a1, a2, a3);
    if !is_enabled() || this.is_null() || !klass_matches(this, expected_klass) {
        if !this.is_null() && !klass_matches(this, expected_klass) {
            SKIPPED_FOREIGN.fetch_add(1, Ordering::Relaxed);
        }
        return result;
    }
    // Re-entry guard: writing text can drive a layout rebuild that calls the
    // hooked UI method again on the same thread.
    let already_inside = BADGE_DEPTH.with(|d| {
        if d.get() != 0 {
            true
        } else {
            d.set(1);
            false
        }
    });
    if already_inside {
        REENTRY_SKIPPED.fetch_add(1, Ordering::Relaxed);
        return result;
    }
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        apply_badges(this, parts_off, expected_klass, counter);
    }));
    BADGE_DEPTH.with(|d| d.set(0));
    result
}

extern "C" fn create_factor_list_hook(
    this: *mut c_void,
    a1: usize,
    a2: usize,
    a3: usize,
) -> u128 {
    unsafe {
        run_original_then_badge(
            create_factor_list_hook as usize,
            this,
            a1,
            a2,
            a3,
            OFF_PARTS_LIST,
            LIST_CLASS.load(Ordering::Acquire),
            &APPLIED_VIA_CREATE,
        )
    }
}

extern "C" fn play_in_factor_list_hook(
    this: *mut c_void,
    a1: usize,
    a2: usize,
    a3: usize,
) -> u128 {
    unsafe {
        run_original_then_badge(
            play_in_factor_list_hook as usize,
            this,
            a1,
            a2,
            a3,
            OFF_PARTS_LIST,
            LIST_CLASS.load(Ordering::Acquire),
            &APPLIED_VIA_PLAYIN,
        )
    }
}

extern "C" fn select_update_factor_list_hook(
    this: *mut c_void,
    a1: usize,
    a2: usize,
    a3: usize,
) -> u128 {
    unsafe {
        run_original_then_badge(
            select_update_factor_list_hook as usize,
            this,
            a1,
            a2,
            a3,
            OFF_PARTS_LIST_CAROUSEL,
            SELECT_CLASS.load(Ordering::Acquire),
            &APPLIED_VIA_SELECT_UPDATE,
        )
    }
}

extern "C" fn select_update_item_hook(
    this: *mut c_void,
    a1: usize,
    a2: usize,
    a3: usize,
) -> u128 {
    unsafe {
        run_original_then_badge(
            select_update_item_hook as usize,
            this,
            a1,
            a2,
            a3,
            OFF_PARTS_LIST_CAROUSEL,
            SELECT_CLASS.load(Ordering::Acquire),
            &APPLIED_VIA_SELECT_ITEM,
        )
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
    // v3.28.14 (a): refuse late installs. The first attempt happens during
    // boot (push_loop / on_game_initialized); anything after the window is an
    // HTTP-triggered retry that would patch a live function in place.
    let first = *FIRST_INSTALL_ATTEMPT.get_or_init(Instant::now);
    if first.elapsed().as_secs() > LATE_INSTALL_WINDOW_SECS {
        super::set_hook_status("factor_index", "skipped: late_install_unsafe");
        LAST_ERROR.store(8, Ordering::Release);
        super::boot_trace("factor_index: late install refused");
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
    // we only *call* it, never hook it.
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
    LIST_CLASS.store(class as usize, Ordering::Release);
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
    super::boot_trace("factor_index: create hook ok");

    // Second hook: PlayInFactorList fires when the list is displayed, after
    // the deferred per-item text setup has run. Failure here is non-fatal.
    let play_in = resolve_method(class, &["PlayInFactorList"], &[0]);
    if play_in != 0 && super::interceptor_hook(play_in, play_in_factor_list_hook as usize) {
        PLAYIN_HOOK_ADDR.store(play_in, Ordering::Release);
    }

    // 因子選択（二次确认/元因子）屏 — best effort, must not block above.
    let sel_item = get_class(
        image,
        super::to_cstr("Gallop").as_ptr(),
        super::to_cstr("PartsSingleModeResultFactorListCarouselScrollItem").as_ptr(),
    );
    if !sel_item.is_null() {
        SELECT_CLASS.store(sel_item as usize, Ordering::Release);
        // v3.28.14 (b): UpdateItem exists on the generic base class, so the
        // resolved address may be shared with unrelated carousels. The class
        // gate in the detour is what makes this hook safe.
        let upd = resolve_method(sel_item, &["UpdateFactorList"], &[0, 1, 2]);
        if upd != 0 && super::interceptor_hook(upd, select_update_factor_list_hook as usize) {
            SELECT_UPDATE_ADDR.store(upd, Ordering::Release);
            super::set_hook_status("factor_index.select_update", &format!("hooked@0x{:x}", upd));
        } else {
            super::set_hook_status("factor_index.select_update", "failed: resolve_or_hook");
        }
        let upd_item = resolve_method(sel_item, &["UpdateItem"], &[0, 1, 2, 3]);
        if upd_item != 0 && super::interceptor_hook(upd_item, select_update_item_hook as usize) {
            SELECT_UPDATE_ITEM_ADDR.store(upd_item, Ordering::Release);
            super::set_hook_status(
                "factor_index.select_update_item",
                &format!("hooked@0x{:x}", upd_item),
            );
        } else {
            super::set_hook_status("factor_index.select_update_item", "failed: resolve_or_hook");
        }
    } else {
        super::set_hook_status("factor_index.select", "failed: class_not_found");
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
    super::boot_trace("factor_index: install done");
}

pub fn endpoint() -> String {
    format!(
        r#"{{"ok":true,"feature":"factor_index_badge","enabled":{},"installed":{},"hook_addr":"0x{:x}","playin_hook_addr":"0x{:x}","select_update_addr":"0x{:x}","select_update_item_addr":"0x{:x}","set_text_addr":"0x{:x}","get_text_addr":"0x{:x}","string_new_addr":"0x{:x}","list_class":"0x{:x}","select_class":"0x{:x}","applied_lists":{},"applied_via_create":{},"applied_via_playin":{},"applied_via_select_update":{},"applied_via_select_item":{},"badged_items":{},"skipped_foreign":{},"skipped_badptr":{},"reentry_skipped":{},"last_error":{},"usage":"GET /api/factor_index/on|off to toggle; numbers are added to the result-screen factor lists while enabled"}}"#,
        is_enabled(),
        HOOK_ADDR.load(Ordering::Acquire) != 0,
        HOOK_ADDR.load(Ordering::Acquire),
        PLAYIN_HOOK_ADDR.load(Ordering::Acquire),
        SELECT_UPDATE_ADDR.load(Ordering::Acquire),
        SELECT_UPDATE_ITEM_ADDR.load(Ordering::Acquire),
        SET_TEXT_ADDR.load(Ordering::Acquire),
        GET_TEXT_ADDR.load(Ordering::Acquire),
        STRING_NEW_UTF16.load(Ordering::Acquire),
        LIST_CLASS.load(Ordering::Acquire),
        SELECT_CLASS.load(Ordering::Acquire),
        APPLIED_LISTS.load(Ordering::Acquire),
        APPLIED_VIA_CREATE.load(Ordering::Acquire),
        APPLIED_VIA_PLAYIN.load(Ordering::Acquire),
        APPLIED_VIA_SELECT_UPDATE.load(Ordering::Acquire),
        APPLIED_VIA_SELECT_ITEM.load(Ordering::Acquire),
        BADGED_ITEMS.load(Ordering::Acquire),
        SKIPPED_FOREIGN.load(Ordering::Acquire),
        SKIPPED_BADPTR.load(Ordering::Acquire),
        REENTRY_SKIPPED.load(Ordering::Acquire),
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
