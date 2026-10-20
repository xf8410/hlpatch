//! ===== Result-screen factor index badge (v3.28.13) =====
//!
//! Prepends "1. ", "2. ", ... to factor names on the result screens so they
//! can be matched to factor-roll order at a glance.
//!
//! Screens covered:
//!   1) 因子(再獲得)列表 — Gallop.PartsSingleModeResultFactorList
//!        CreateFactorList + PlayInFactorList (double hook; text is assigned
//!        asynchronously after both return, idempotency via "N. " prefix check)
//!   2) 因子選択（二次确认/元因子）— Gallop.PartsSingleModeResultFactorListCarouselScrollItem
//!        UpdateFactorList + UpdateItem (double hook; the carousel re-pools
//!        page items on tab switches, so text must be re-applied whenever the
//!        game rewrites it)
//!
//! Both lists hold Gallop.PartsFactorListContent items with the same layout
//! (_factorName TextCommon @ +0x38), so one walk routine serves both.
//!
//! The list walk is read-only except TextCommon.set_text on items whose text
//! does not start with a digit prefix yet. Everything is local UI only.
//!
//! Toggle: /api/factor_index/on|off (default on). Status: /api/factor_index.

use std::ffi::{c_char, c_void};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};

static ENABLED: AtomicBool = AtomicBool::new(true);
static HOOK_ADDR: AtomicUsize = AtomicUsize::new(0);
static PLAYIN_HOOK_ADDR: AtomicUsize = AtomicUsize::new(0);
static SELECT_UPDATE_ADDR: AtomicUsize = AtomicUsize::new(0);
static SELECT_UPDATE_ITEM_ADDR: AtomicUsize = AtomicUsize::new(0);
static SET_TEXT_ADDR: AtomicUsize = AtomicUsize::new(0);
static GET_TEXT_ADDR: AtomicUsize = AtomicUsize::new(0);
static STRING_NEW_ADDR: AtomicUsize = AtomicUsize::new(0);
static APPLIED_LISTS: AtomicU64 = AtomicU64::new(0);
static APPLIED_VIA_CREATE: AtomicU64 = AtomicU64::new(0);
static APPLIED_VIA_PLAYIN: AtomicU64 = AtomicU64::new(0);
static APPLIED_VIA_SELECT_UPDATE: AtomicU64 = AtomicU64::new(0);
static APPLIED_VIA_SELECT_ITEM: AtomicU64 = AtomicU64::new(0);
static BADGED_ITEMS: AtomicU64 = AtomicU64::new(0);
static LAST_ERROR: AtomicU64 = AtomicU64::new(0);

const OFFSET_PARTS_LIST_RESULT: usize = 0x60; // PartsSingleModeResultFactorList._partsList
const OFFSET_PARTS_LIST_CAROUSEL: usize = 0x90; // ...FactorListCarouselScrollItem._partsFactorListContentList
const OFFSET_ITEM_TEXT: usize = 0x38; // PartsFactorListContent._factorName (TextCommon)
const MAX_ITEMS: usize = 64;

fn utf16_to_string(s: usize) -> String {
    unsafe {
        if s == 0 {
            return String::new();
        }
        let len = *((s + 0x10) as *const i32);
        if len <= 0 || len > 4096 {
            return String::new();
        }
        let chars = (s + 0x14) as *const u16;
        let slice = std::slice::from_raw_parts(chars, len as usize);
        String::from_utf16_lossy(slice)
    }
}

fn already_prefixed(s: &str) -> bool {
    let b = s.as_bytes();
    let mut i = 0usize;
    while i < b.len() && b[i].is_ascii_digit() {
        i += 1;
    }
    i > 0 && i + 1 < b.len() && b[i] == b'.' && b[i + 1] == b' '
}

unsafe fn get_text(text: usize) -> String {
    let addr = GET_TEXT_ADDR.load(Ordering::Acquire);
    if addr == 0 || text == 0 {
        return String::new();
    }
    type GetTextFn = unsafe extern "C" fn(usize) -> usize;
    let f: GetTextFn = std::mem::transmute(addr);
    let s = f(text);
    utf16_to_string(s)
}

/// Walk a List<PartsFactorListContent> and prefix un-numbered items.
unsafe fn apply_badges_parts(parts: usize) {
    if parts == 0 {
        return;
    }
    let size_ptr = (parts + 0x18) as *const i32;
    let count = (*size_ptr).clamp(0, MAX_ITEMS as i32) as usize;
    if count == 0 {
        return;
    }
    let items = *(parts as *const usize);
    if items == 0 {
        return;
    }
    let set_text_addr = SET_TEXT_ADDR.load(Ordering::Acquire);
    let string_new_addr = STRING_NEW_ADDR.load(Ordering::Acquire);
    if set_text_addr == 0 || string_new_addr == 0 {
        return;
    }
    type SetTextFn = unsafe extern "C" fn(usize, usize);
    let set_text: SetTextFn = std::mem::transmute(set_text_addr);
    type StringNewFn = unsafe extern "C" fn(*const u16, i32) -> usize;
    let string_new: StringNewFn = std::mem::transmute(string_new_addr);
    for i in 0..count {
        let item = *((items + 0x20 + i * 8) as *const usize);
        if item == 0 {
            continue;
        }
        let text_ptr = *((item + OFFSET_ITEM_TEXT) as *const usize);
        if text_ptr == 0 {
            continue;
        }
        let badge_no = i + 1;
        let current = get_text(text_ptr);
        if current.is_empty() || already_prefixed(&current) {
            continue;
        }
        let new_text = format!("{}. {}", badge_no, current);
        let mut wide: Vec<u16> = new_text.encode_utf16().collect();
        let new_str = string_new(wide.as_mut_ptr(), wide.len() as i32);
        if new_str == 0 {
            LAST_ERROR.store(3, Ordering::Release);
            continue;
        }
        set_text(text_ptr, new_str);
        BADGED_ITEMS.fetch_add(1, Ordering::AcqRel);
    }
}

unsafe fn apply_badges(this: *mut c_void, parts_list_offset: usize) {
    if this.is_null() {
        return;
    }
    let parts = *((this as usize + parts_list_offset) as *const usize);
    apply_badges_parts(parts);
}

extern "C" fn create_factor_list_hook(
    this: *mut c_void,
    a1: usize,
    a2: usize,
    a3: usize,
) -> u128 {
    let result = unsafe {
        let orig_ptr = super::interceptor_get_trampoline(create_factor_list_hook as usize);
        if orig_ptr != 0 {
            type FnTy = unsafe extern "C" fn(*mut c_void, usize, usize, usize) -> u128;
            let original: FnTy = std::mem::transmute(orig_ptr);
            original(this, a1, a2, a3)
        } else {
            LAST_ERROR.store(4, Ordering::Release);
            0
        }
    };
    if is_enabled() && !this.is_null() {
        let _ = catch_unwind(AssertUnwindSafe(|| unsafe {
            apply_badges(this, OFFSET_PARTS_LIST_RESULT);
        }));
        APPLIED_LISTS.fetch_add(1, Ordering::AcqRel);
        APPLIED_VIA_CREATE.fetch_add(1, Ordering::AcqRel);
    }
    result
}

extern "C" fn play_in_factor_list_hook(
    this: *mut c_void,
    a1: usize,
    a2: usize,
    a3: usize,
) -> u128 {
    let result = unsafe {
        let orig_ptr = super::interceptor_get_trampoline(play_in_factor_list_hook as usize);
        if orig_ptr != 0 {
            type FnTy = unsafe extern "C" fn(*mut c_void, usize, usize, usize) -> u128;
            let original: FnTy = std::mem::transmute(orig_ptr);
            original(this, a1, a2, a3)
        } else {
            LAST_ERROR.store(5, Ordering::Release);
            0
        }
    };
    if is_enabled() && !this.is_null() {
        let _ = catch_unwind(AssertUnwindSafe(|| unsafe {
            apply_badges(this, OFFSET_PARTS_LIST_RESULT);
        }));
        APPLIED_LISTS.fetch_add(1, Ordering::AcqRel);
        APPLIED_VIA_PLAYIN.fetch_add(1, Ordering::AcqRel);
    }
    result
}

extern "C" fn select_update_factor_list_hook(
    this: *mut c_void,
    a1: usize,
    a2: usize,
    a3: usize,
) -> u128 {
    let result = unsafe {
        let orig_ptr = super::interceptor_get_trampoline(select_update_factor_list_hook as usize);
        if orig_ptr != 0 {
            type FnTy = unsafe extern "C" fn(*mut c_void, usize, usize, usize) -> u128;
            let original: FnTy = std::mem::transmute(orig_ptr);
            original(this, a1, a2, a3)
        } else {
            LAST_ERROR.store(6, Ordering::Release);
            0
        }
    };
    if is_enabled() && !this.is_null() {
        let _ = catch_unwind(AssertUnwindSafe(|| unsafe {
            apply_badges(this, OFFSET_PARTS_LIST_CAROUSEL);
        }));
        APPLIED_LISTS.fetch_add(1, Ordering::AcqRel);
        APPLIED_VIA_SELECT_UPDATE.fetch_add(1, Ordering::AcqRel);
    }
    result
}

extern "C" fn select_update_item_hook(
    this: *mut c_void,
    a1: usize,
    a2: usize,
    a3: usize,
) -> u128 {
    let result = unsafe {
        let orig_ptr = super::interceptor_get_trampoline(select_update_item_hook as usize);
        if orig_ptr != 0 {
            type FnTy = unsafe extern "C" fn(*mut c_void, usize, usize, usize) -> u128;
            let original: FnTy = std::mem::transmute(orig_ptr);
            original(this, a1, a2, a3)
        } else {
            LAST_ERROR.store(7, Ordering::Release);
            0
        }
    };
    if is_enabled() && !this.is_null() {
        let _ = catch_unwind(AssertUnwindSafe(|| unsafe {
            apply_badges(this, OFFSET_PARTS_LIST_CAROUSEL);
        }));
        APPLIED_LISTS.fetch_add(1, Ordering::AcqRel);
        APPLIED_VIA_SELECT_ITEM.fetch_add(1, Ordering::AcqRel);
    }
    result
}

unsafe fn resolve_method(class: usize, names: &[&str], counts: &[i32]) -> usize {
    if class == 0 {
        return 0;
    }
    for name in names {
        let cname = super::to_cstr(name);
        for &count in counts {
            let addr = super::get_method_addr(class, cname.as_ptr(), count);
            if addr != 0 {
                return addr;
            }
        }
    }
    0
}

pub unsafe fn install() -> bool {
    if HOOK_ADDR.load(Ordering::Acquire) != 0 {
        return true;
    }
    if super::API.is_null() {
        super::set_hook_status("factor_index", "failed: api_null");
        LAST_ERROR.store(1, Ordering::Release);
        return false;
    }
    if api().interceptor == 0 {
        super::set_hook_status("factor_index", "failed: api_null");
        LAST_ERROR.store(1, Ordering::Release);
        return false;
    }
    let image = super::get_image(b"Gallop\0".as_ptr() as *const c_char);
    if image.is_none() {
        super::set_hook_status("factor_index", "failed: image_not_found");
        LAST_ERROR.store(5, Ordering::Release);
        return false;
    }
    let image = image.unwrap();
    if image == 0 {
        super::set_hook_status("factor_index", "failed: image_null");
        LAST_ERROR.store(5, Ordering::Release);
        return false;
    }

    let text_common = super::get_class(
        image,
        super::to_cstr("Gallop").as_ptr(),
        super::to_cstr("TextCommon").as_ptr(),
    );
    if text_common != 0 {
        let set_text = resolve_method(text_common, &["set_text", "SetText", "Set"], &[1]);
        let get_text = resolve_method(text_common, &["get_text", "GetText"], &[0]);
        SET_TEXT_ADDR.store(set_text, Ordering::Release);
        GET_TEXT_ADDR.store(get_text, Ordering::Release);
    }

    let corlib = super::get_image(b"corlib\0".as_ptr() as *const c_char);
    if let Some(core) = corlib {
        if core != 0 {
            let string_class = super::get_class(
                core,
                super::to_cstr("System").as_ptr(),
                super::to_cstr("String").as_ptr(),
            );
            if string_class != 0 {
                let ctor = resolve_method(string_class, &["CreateString"], &[1, 2]);
                if ctor == 0 {
                    let ctor2 = resolve_method(string_class, &[".ctor"], &[1, 2]);
                    STRING_NEW_ADDR.store(ctor2, Ordering::Release);
                } else {
                    STRING_NEW_ADDR.store(ctor, Ordering::Release);
                }
            }
        }
    }
    if STRING_NEW_ADDR.load(Ordering::Acquire) == 0 {
        let fast = super::get_method_addr_by_name(
            b"System.String::CreateString\0".as_ptr() as *const c_char
        );
        STRING_NEW_ADDR.store(fast, Ordering::Release);
    }
    if SET_TEXT_ADDR.load(Ordering::Acquire) == 0 {
        super::set_hook_status("factor_index", "failed: set_text_unresolved");
        LAST_ERROR.store(2, Ordering::Release);
        return false;
    }

    let class_name = "PartsSingleModeResultFactorList";
    let class = super::get_class(
        image,
        super::to_cstr("Gallop").as_ptr(),
        super::to_cstr(class_name).as_ptr(),
    );
    if class == 0 {
        super::set_hook_status("factor_index", "failed: class_not_found");
        LAST_ERROR.store(6, Ordering::Release);
        return false;
    }

    let create_list = resolve_method(class, &["CreateFactorList"], &[0, 1, 2, 3]);
    if create_list == 0 || !super::interceptor_hook(create_list, create_factor_list_hook as usize)
    {
        super::set_hook_status("factor_index", "failed: create_hook");
        LAST_ERROR.store(7, Ordering::Release);
        return false;
    }
    HOOK_ADDR.store(create_list, Ordering::Release);

    let play_in = resolve_method(class, &["PlayInFactorList"], &[0]);
    let play_in_hooked =
        play_in != 0 && super::interceptor_hook(play_in, play_in_factor_list_hook as usize);
    if play_in_hooked {
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
            STRING_NEW_ADDR.load(Ordering::Acquire)
        ),
    );

    // v3.28.13: 因子選択（二次确认）屏 — best effort, must not block main hooks.
    let sel_item = super::get_class(
        image,
        super::to_cstr("Gallop").as_ptr(),
        super::to_cstr("PartsSingleModeResultFactorListCarouselScrollItem").as_ptr(),
    );
    if sel_item == 0 {
        super::set_hook_status("factor_index.select", "failed: class_not_found");
    } else {
        let upd = resolve_method(sel_item, &["UpdateFactorList"], &[0, 1, 2]);
        if upd != 0 && super::interceptor_hook(upd, select_update_factor_list_hook as usize) {
            SELECT_UPDATE_ADDR.store(upd, Ordering::Release);
            super::set_hook_status(
                "factor_index.select_update",
                &format!("hooked@0x{:x}", upd),
            );
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
    }
    true
}

pub fn endpoint() -> String {
    format!(
        concat!(
            "{{\"ok\":true,\"feature\":\"factor_index_badge\",\"enabled\":{},",
            "\"installed\":{},\"hook_addr\":\"0x{:x}\",\"playin_hook_addr\":\"0x{:x}\",",
            "\"select_update_addr\":\"0x{:x}\",\"select_update_item_addr\":\"0x{:x}\",",
            "\"set_text_addr\":\"0x{:x}\",\"get_text_addr\":\"0x{:x}\",",
            "\"string_new_addr\":\"0x{:x}\",\"applied_lists\":{},",
            "\"applied_via_create\":{},\"applied_via_playin\":{},",
            "\"applied_via_select_update\":{},\"applied_via_select_item\":{},",
            "\"badged_items\":{},\"last_error\":{},",
            "\"usage\":\"GET /api/factor_index/on|off to toggle; numbers are added to the result-screen factor lists while enabled\"}}"
        ),
        ENABLED.load(Ordering::Acquire),
        HOOK_ADDR.load(Ordering::Acquire) != 0,
        HOOK_ADDR.load(Ordering::Acquire),
        PLAYIN_HOOK_ADDR.load(Ordering::Acquire),
        SELECT_UPDATE_ADDR.load(Ordering::Acquire),
        SELECT_UPDATE_ITEM_ADDR.load(Ordering::Acquire),
        SET_TEXT_ADDR.load(Ordering::Acquire),
        GET_TEXT_ADDR.load(Ordering::Acquire),
        STRING_NEW_ADDR.load(Ordering::Acquire),
        APPLIED_LISTS.load(Ordering::Acquire),
        APPLIED_VIA_CREATE.load(Ordering::Acquire),
        APPLIED_VIA_PLAYIN.load(Ordering::Acquire),
        APPLIED_VIA_SELECT_UPDATE.load(Ordering::Acquire),
        APPLIED_VIA_SELECT_ITEM.load(Ordering::Acquire),
        BADGED_ITEMS.load(Ordering::Acquire),
        LAST_ERROR.load(Ordering::Acquire)
    )
}

pub fn enable() -> String {
    ENABLED.store(true, Ordering::Release);
    endpoint()
}

pub fn disable() -> String {
    ENABLED.store(false, Ordering::Release);
    endpoint()
}

pub fn set_enabled(on: bool) {
    ENABLED.store(on, Ordering::Release);
}

pub fn is_enabled() -> bool {
    ENABLED.load(Ordering::Acquire)
}

unsafe fn api() -> &'static super::Il2CppApi {
    &*super::API
}
