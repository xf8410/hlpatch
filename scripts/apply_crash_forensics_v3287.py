#!/usr/bin/env python3
"""v3.28.7 crash forensics fixes (diagnostic build, no behavior change).

The v3.28.6 sample exposed one bug and two gaps:

    CRASH at step 118 sig=11 addr=0x1f022058000109 tid=486774850816
    pc=native+1511833d0 lr=native+1511834b0 thr=init FATAL

  * "native" is a MERGE of libnative.so + libnativewindow.so (both matched
    `starts_with("libnative")`), so the printed offset (+0x1511833d0) is
    nonsense - the two libs sit GBs apart. The labeler must never merge
    distant same-prefix libs.
  * thr=init says the fault hit the plugin-init thread, not our push/http
    threads - but the ring can be churned by http threads, so the tid must
    also be written into the predict log (T:label=tid) for offline match.
  * Latent hazard: the handler longjmps whenever the global recovery flag is
    set, even if the faulting thread is NOT the armed one - jumping to
    another thread's stack. Gate the longjmp on armed-tid == fault-tid.

This patch (forensics only, zero data-path change):
  1) module table: strict labels native/natwin/nathelp; merge same-name rows
     only when near (<=64MB); match by smallest span.
  2) thread registry: same-label slots update in place; register() also logs
     `T:<label>=<tid>` into the predict log.
  3) hl_arm_recovery(): every arming site records its tid; handler and the
     verdict line require armed-tid == faulting tid for RECOVERED/longjmp.
  4) HL_MOD_MAX 128 -> 256.

Idempotent: `fn hl_arm_recovery()` present -> already_applied.
"""
import re
from pathlib import Path

SOURCE = Path("hachimi_ura_plugin/src/lib.rs")
text = SOURCE.read_text(encoding="utf-8")

if "fn hl_arm_recovery()" in text:
    print("crash_forensics_v3287=already_applied")
    raise SystemExit(0)


def replace_once(old: str, new: str, label: str) -> None:
    global text
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"{label} anchor count={count} (expect 1)")
    text = text.replace(old, new, 1)


# ---- 1) HL_MOD_MAX 256 ----
replace_once(
    "const HL_MOD_MAX: usize = 128;",
    "const HL_MOD_MAX: usize = 256; // v3.28.7: room for per-module rows",
    "mod_max",
)

# ---- 2) replace hl_modules_init body ----
NEW_MODULES = r'''fn hl_modules_init() {
    let text = match std::fs::read_to_string("/proc/self/maps") {
        Ok(t) => t,
        Err(_) => return,
    };
    let mut count = 0usize;
    unsafe {
        let bases = std::ptr::addr_of_mut!(HL_MOD_BASE) as *mut usize;
        let ends = std::ptr::addr_of_mut!(HL_MOD_END) as *mut usize;
        let names = std::ptr::addr_of_mut!(HL_MOD_NAME) as *mut u8;
        let lens = std::ptr::addr_of_mut!(HL_MOD_NAME_LEN) as *mut u8;
        for line in text.lines() {
            let mut it = line.split_whitespace();
            let range = match it.next() { Some(r) => r, None => continue };
            let perms = match it.next() { Some(p) => p, None => continue };
            let _offset = match it.next() { Some(o) => o, None => continue };
            let _dev = it.next();
            let _ino = it.next();
            let path = it.next().unwrap_or("");
            if !perms.contains('x') || path.is_empty() || !path.contains('/') {
                continue;
            }
            let mut rparts = range.splitn(2, '-');
            let base_s = match rparts.next() { Some(v) => v, None => continue };
            let end_s = match rparts.next() { Some(v) => v, None => continue };
            let b = usize::from_str_radix(base_s, 16).unwrap_or(0);
            let e = usize::from_str_radix(end_s, 16).unwrap_or(0);
            if e <= b {
                continue;
            }
            let fname = path.rsplit('/').next().unwrap_or(path);
            // v3.28.7: strict label mapping. libnative* variants live in
            // different address regions (GBs apart); merging them produced
            // nonsense offsets, so each gets its own row now.
            let short: &str = if fname.starts_with("libhachimi_ura") {
                "hlpatch"
            } else if fname.starts_with("libil2cpp") {
                "il2cpp"
            } else if fname == "libnative.so" {
                "native"
            } else if fname.starts_with("libnativewindow") {
                "natwin"
            } else if fname.starts_with("libnativehelper") {
                "nathelp"
            } else if fname.starts_with("libc.so") {
                "libc"
            } else if fname.starts_with("libmain") {
                "main"
            } else {
                fname
            };
            let short_bytes = short.as_bytes();
            let sn = short_bytes.len().min(15);
            let mut merged = false;
            let mut i = 0usize;
            while i < count {
                if *lens.add(i) as usize == sn {
                    let mut same = true;
                    let mut j = 0usize;
                    while j < sn {
                        if *names.add(i * 16 + j) != short_bytes[j] {
                            same = false;
                            break;
                        }
                        j += 1;
                    }
                    if same {
                        // v3.28.7: only extend a row when the new segment is
                        // near (or overlaps) it; far same-name mappings stay
                        // as separate rows.
                        let sb = *bases.add(i);
                        let se = *ends.add(i);
                        if b <= se.saturating_add(0x4000000)
                            && e.saturating_add(0x4000000) >= sb
                        {
                            if b < sb {
                                *bases.add(i) = b;
                            }
                            if e > se {
                                *ends.add(i) = e;
                            }
                            merged = true;
                            break;
                        }
                    }
                }
                i += 1;
            }
            if merged {
                continue;
            }
            if count >= HL_MOD_MAX {
                continue;
            }
            *bases.add(count) = b;
            *ends.add(count) = e;
            let mut j = 0usize;
            while j < sn {
                *names.add(count * 16 + j) = short_bytes[j];
                j += 1;
            }
            *lens.add(count) = sn as u8;
            count += 1;
        }
    }
    HL_MOD_COUNT.store(count, std::sync::atomic::Ordering::Relaxed);
}'''

start = text.index("fn hl_modules_init() {")
end_marker = "HL_MOD_COUNT.store(count, std::sync::atomic::Ordering::Relaxed);\n}"
end = text.index(end_marker, start) + len(end_marker)
old_mod_modules = text[start:end]
if "starts_with(\"libnative\")" not in old_mod_modules:
    raise RuntimeError("old modules fn shape unexpected")
text = text[:start] + NEW_MODULES + text[end:]

# ---- 3) replace hl_thread_register body ----
NEW_REGISTER = r'''fn hl_thread_register(label: &str) {
    let tid = unsafe { libc::pthread_self() as usize };
    let mut want = [0u8; 8];
    let label_bytes = label.as_bytes();
    let n = label_bytes.len().min(7);
    let mut k = 0usize;
    while k < n {
        want[k] = label_bytes[k];
        k += 1;
    }
    unsafe {
        let ids = std::ptr::addr_of_mut!(HL_THR_TID) as *mut usize;
        let names = std::ptr::addr_of_mut!(HL_THR_NAMES) as *mut u8;
        // v3.28.7: same-label entries update in place so init/push never get
        // evicted by http-thread churn.
        let mut target = usize::MAX;
        let mut i = 0usize;
        while i < HL_THR_MAX {
            let mut same = true;
            let mut j = 0usize;
            while j < 8 {
                if *names.add(i * 8 + j) != want[j] {
                    same = false;
                    break;
                }
                j += 1;
            }
            if same && *ids.add(i) != 0 {
                target = i;
                break;
            }
            i += 1;
        }
        if target == usize::MAX {
            i = 0;
            while i < HL_THR_MAX {
                if *ids.add(i) == 0 {
                    target = i;
                    break;
                }
                i += 1;
            }
        }
        if target == usize::MAX {
            target =
                HL_THR_NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed) % HL_THR_MAX;
        }
        *ids.add(target) = tid;
        let mut j = 0usize;
        while j < 8 {
            *names.add(target * 8 + j) = want[j];
            j += 1;
        }
    }
    // v3.28.7: also write the tid mapping into the predict log so a crash
    // line's tid can be resolved offline even if the ring is stale.
    log_predict_step(&format!("T:{}={}", label, tid));
}'''

rstart = text.index("fn hl_thread_register(label: &str) {")
rend = text.index("fn hl_put_mod(", rstart)
text = text[:rstart] + NEW_REGISTER + "\n\n" + text[rend:]

# ---- 4) replace hl_put_mod (best-fit: smallest containing span) ----
NEW_PUT_MOD = r'''fn hl_put_mod(msg: &mut [u8], mut len: usize, addr: usize) -> usize {
    let count = HL_MOD_COUNT.load(std::sync::atomic::Ordering::Relaxed);
    let mut best = usize::MAX;
    let mut best_span = usize::MAX;
    unsafe {
        let bases = std::ptr::addr_of!(HL_MOD_BASE) as *const usize;
        let ends = std::ptr::addr_of!(HL_MOD_END) as *const usize;
        let mut i = 0usize;
        while i < count && i < HL_MOD_MAX {
            let b = *bases.add(i);
            let e = *ends.add(i);
            if addr >= b && addr < e {
                let span = e - b;
                if span < best_span {
                    best_span = span;
                    best = i;
                }
            }
            i += 1;
        }
        if best == usize::MAX {
            msg[len] = b'?';
            return len + 1;
        }
        let names = std::ptr::addr_of!(HL_MOD_NAME) as *const u8;
        let lens = std::ptr::addr_of!(HL_MOD_NAME_LEN) as *const u8;
        let b = *bases.add(best);
        let name_ptr = names.add(best * 16);
        let name_len = *lens.add(best) as usize;
        let mut j = 0usize;
        while j < name_len {
            msg[len] = *name_ptr.add(j);
            len += 1;
            j += 1;
        }
        msg[len] = b'+';
        len += 1;
        len = hl_put_hex(msg, len, addr.wrapping_sub(b));
    }
    len
}'''

pstart = text.index("fn hl_put_mod(")
pend = text.index("fn hl_put_thr(", pstart)
text = text[:pstart] + NEW_PUT_MOD + "\n\n" + text[pend:]

# ---- 5) armed-tid static + helper ----
ARM_BLOCK = r'''static SIGSEGV_COOLDOWN_UNTIL: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
// v3.28.7: a recovery longjmp is only legal on the thread that armed it.
static HL_RECOVERY_ARMED_TID: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);

fn hl_arm_recovery() {
    HL_RECOVERY_ARMED_TID.store(
        unsafe { libc::pthread_self() as usize },
        std::sync::atomic::Ordering::Relaxed,
    );
    SIGSEGV_RECOVERY.store(true, std::sync::atomic::Ordering::Relaxed);
}'''
replace_once(
    "static SIGSEGV_COOLDOWN_UNTIL: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);",
    ARM_BLOCK,
    "armed_tid_block",
)

# ---- 6) all arming sites -> hl_arm_recovery() ----
pattern = re.compile(r"SIGSEGV_RECOVERY\.store\(true,[^;]*\);")
text, n_replaced = pattern.subn("hl_arm_recovery();", text)
if n_replaced < 9:
    raise RuntimeError(f"arming sites replaced={n_replaced} (expect >=9)")
if "SIGSEGV_RECOVERY.store(true" in text:
    raise RuntimeError("store(true) still present after rewrite")

# ---- 7) handler gate + verdict require armed-tid == faulting tid ----
replace_once(
    "    if SIGSEGV_RECOVERY.load(std::sync::atomic::Ordering::Relaxed) {\n"
    "        // Set cooldown: skip reads for 60 seconds",
    "    if SIGSEGV_RECOVERY.load(std::sync::atomic::Ordering::Relaxed)\n"
    "        && HL_RECOVERY_ARMED_TID.load(std::sync::atomic::Ordering::Relaxed) == fault_tid\n"
    "    {\n"
    "        // Set cooldown: skip reads for 60 seconds",
    "handler_longjmp_gate",
)
replace_once(
    '    let verdict: &[u8] = if SIGSEGV_RECOVERY.load(std::sync::atomic::Ordering::Relaxed) {\n'
    '        &b" RECOVERED"[..]',
    '    let verdict: &[u8] = if SIGSEGV_RECOVERY.load(std::sync::atomic::Ordering::Relaxed)\n'
    "        && HL_RECOVERY_ARMED_TID.load(std::sync::atomic::Ordering::Relaxed) == fault_tid\n"
    "    {\n"
    '        &b" RECOVERED"[..]',
    "handler_verdict_gate",
)

# ---- post-checks (fail closed) ----
for required in (
    "fn hl_arm_recovery()",
    "HL_RECOVERY_ARMED_TID",
    "natwin",
    "nathelp",
    'fname == "libnative.so"',
    'T:{}={}',
    "best_span",
    "HL_MOD_MAX: usize = 256",
):
    if required not in text:
        raise RuntimeError(f"post-check missing: {required}")
if 'starts_with("libnative")' in text:
    raise RuntimeError("old libnative prefix merge still present")
if text.count("SIGSEGV_RECOVERY.store(true") != 1:
    raise RuntimeError("store(true) count != 1 (helper only)")

SOURCE.write_text(text, encoding="utf-8")
print(
    "crash_forensics_v3287=applied "
    f"armed_sites={n_replaced} modules=strict threads=tid_logged gate=armed_tid"
)
