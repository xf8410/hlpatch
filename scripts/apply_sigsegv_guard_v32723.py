# v3.27.23 SIGSEGV guard, extracted verbatim from the proven v3.28.1 release
# workflow (inline python -> script file) so releases stop re-embedding it.
# Idempotent: re-running does not change the file (marker + hl_ptr_mapped check).
from pathlib import Path

source = Path('hachimi_ura_plugin/src/lib.rs')
text = source.read_text(encoding='utf-8')
MARK = 'v3.27.23 sigsegv-guard'

if 'fn hl_ptr_mapped(' in text:
    print('sigsegv_guard=already_applied')
    raise SystemExit(0)

# v3 order: rename first, then insert.
# (v2 lesson: inserting first made the rename anchor also hit the wrapper
# definition -> count=2. Only signature lines are anchored; never function
# bodies, because generator function bodies drift.)

# 1) rename originals to _inner (signature line is unique here, count==1 asserted)
renames = [
    ('unsafe fn read_obscured_int_at(obj: *const c_void, field_offset: i32) -> i32 {',
     'unsafe fn read_obscured_int_at_inner(obj: *const c_void, field_offset: i32) -> i32 {'),
    ('unsafe fn read_ptr_at(obj: *const c_void, field_offset: i32) -> *mut c_void {',
     'unsafe fn read_ptr_at_inner(obj: *const c_void, field_offset: i32) -> *mut c_void {'),
    ('unsafe fn read_int_at(obj: *const c_void, field_offset: i32) -> i32 {',
     'unsafe fn read_int_at_inner(obj: *const c_void, field_offset: i32) -> i32 {'),
]
for old, new in renames:
    c = text.count(old)
    if c != 1:
        raise RuntimeError(f'rename anchor count={c} (expect 1): {old[:60]}')
    text = text.replace(old, new, 1)

# 2) helpers + same-name wrappers inserted before read_obscured_int_at_inner
anchor = 'unsafe fn read_obscured_int_at_inner(obj: *const c_void, field_offset: i32) -> i32 {'
if text.count(anchor) != 1:
    raise RuntimeError(f'insert anchor count={text.count(anchor)} (expect 1)')

block = '''// v3.27.23 sigsegv-guard: mapped-region validation for raw memory reads.
// Root cause: read_summary dereferenced game objects already freed on
// screen change (scenario/data-set swap), raising SIGSEGV -> 60s
// recovery cooldown with empty trainings/ramen output. Any address
// outside a currently-readable mapping is now read as null / -1.
// Maps snapshot cached 2s; unreadable maps fail OPEN (never brick reads).
// Original helpers renamed *_inner; same-name wrappers add the guard
// without assuming anything about the original bodies.
static HL_MAP_CACHE: std::sync::Mutex<Option<(u64, Vec<(usize, usize)>)>> =
    std::sync::Mutex::new(None);

fn hl_parse_maps_readable() -> Vec<(usize, usize)> {
    let text = match std::fs::read_to_string("/proc/self/maps") {
        Ok(t) => t,
        Err(_) => return Vec::new(),
    };
    let mut out: Vec<(usize, usize)> = Vec::new();
    for line in text.lines() {
        let range = match line.split_whitespace().next() {
            Some(r) => r,
            None => continue,
        };
        let perms = line.split_whitespace().nth(1).unwrap_or("");
        if !perms.starts_with('r') {
            continue;
        }
        let mut it = range.splitn(2, '-');
        let s = match it.next() { Some(v) => v, None => continue };
        let e = match it.next() { Some(v) => v, None => continue };
        let s = usize::from_str_radix(s, 16).unwrap_or(0);
        let e = usize::from_str_radix(e, 16).unwrap_or(0);
        if e > s {
            out.push((s, e));
        }
    }
    out
}

fn hl_ptr_mapped(addr: usize) -> bool {
    if addr == 0 {
        return false;
    }
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let mut cache = match HL_MAP_CACHE.lock() {
        Ok(g) => g,
        Err(poisoned) => poisoned.into_inner(),
    };
    let stale = match *cache {
        Some((ts, _)) => now.saturating_sub(ts) >= 2,
        None => true,
    };
    if stale {
        *cache = Some((now, hl_parse_maps_readable()));
    }
    let regions = match cache.as_ref() {
        Some((_, r)) => r,
        None => return true,
    };
    if regions.is_empty() {
        return true; // maps unreadable — fail open
    }
    let mut lo = 0usize;
    let mut hi = regions.len();
    while lo < hi {
        let mid = (lo + hi) / 2;
        let (s, e) = regions[mid];
        if addr < s {
            hi = mid;
        } else if addr >= e {
            lo = mid + 1;
        } else {
            return true;
        }
    }
    false
}

// v3.27.23 sigsegv-guard wrapper: unmapped holder or returned pointer -> null
unsafe fn read_ptr_at(obj: *const c_void, field_offset: i32) -> *mut c_void {
    if !hl_ptr_mapped(obj as usize) {
        return std::ptr::null_mut();
    }
    let value = read_ptr_at_inner(obj, field_offset);
    if !value.is_null() && !hl_ptr_mapped(value as usize) {
        return std::ptr::null_mut(); // freed/dangling pointer read as null
    }
    value
}

// v3.27.23 sigsegv-guard wrapper: unmapped holder -> -1
unsafe fn read_obscured_int_at(obj: *const c_void, field_offset: i32) -> i32 {
    if !hl_ptr_mapped(obj as usize) {
        return -1;
    }
    read_obscured_int_at_inner(obj, field_offset)
}

// v3.27.23 sigsegv-guard wrapper: unmapped holder -> -1
unsafe fn read_int_at(obj: *const c_void, field_offset: i32) -> i32 {
    if !hl_ptr_mapped(obj as usize) {
        return -1;
    }
    read_int_at_inner(obj, field_offset)
}

'''
text = text.replace(anchor, block + anchor, 1)

source.write_text(text, encoding='utf-8')
print('sigsegv_guard=applied marker_count=%d' % text.count(MARK))
