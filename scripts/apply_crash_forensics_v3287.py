#!/usr/bin/env python3
"""v3.28.7 crash forensics v3 (diagnostic build; zero data-path change).

The v3.28.6 sample exposed three flaws in our own instrumentation:

    CRASH at step 118 sig=11 addr=0x1f022058000109 tid=486774850816
    pc=native+1511833d0 lr=native+1511834b0 thr=init FATAL

  1) "native" merged libnative.so + libnativewindow.so (+nativehelper) into
     one row, so the printed offsets (+0x1511833d0, ~5.6 GB) are nonsense.
  2) thr=init is not trustworthy: the 16-slot ring can evict entries, and
     pthread_t values are recycled after a thread exits, so an old label can
     hijack a new thread's identity.
  3) safety: the handler longjmp'd regardless of WHICH thread armed recovery
     (single shared jmp_buf) - cross-thread longjmp is stack corruption.

This patch:
  A) module table: strict labels native / natwin / nathelp; merge same-label
     segments only when near (<=64MB); lookup = smallest containing span.
  B) thread registry: per-slot write-order seq; same-label update in place;
     lookup returns the label of the NEWEST matching registration. Every
     register() also appends `T:<label>=<tid>` to the predict log, so the
     crash tid can be resolved offline.
  C) hl_arm_recovery(): records the arming tid; the handler only longjmps
     (and only prints RECOVERED) when armed-tid == faulting tid.
  D) forensics fields: ` fa=0x..` (kernel fault address echo from the
     context - validates our context offsets) and ` c=<si_code>`
     (1=MAPERR wild pointer, 2=ACCERR permission/execute fault).
  E) S:enter / S:exit markers now carry the writing thread's tid, so the
     crash tid can be checked against the summary-caller tid directly.

Idempotent: `fn hl_arm_recovery()` present -> already_applied.
"""
import re
from pathlib import Path

SOURCE = Path("hachimi_ura_plugin/src/lib.rs")
text = SOURCE.read_text(encoding="utf-8")

if "fn hl_arm_recovery()" in text or "HL_THR_SEQ" in text:
    print("crash_forensics_v3287=already_applied")
    raise SystemExit(0)


def replace_once(old: str, new: str, label: str) -> None:
    global text
    c = text.count(old)
    if c != 1:
        raise RuntimeError(f"{label} anchor count={c} (expect 1)")
    text = text.replace(old, new, 1)


# ---- 1) module table ceiling ----
replace_once(
    "const HL_MOD_MAX: usize = 128;",
    "const HL_MOD_MAX: usize = 256; // v3.28.7: room for per-module rows",
    "mod_max",
)

# ---- 2) thread registry: add write-order seq statics ----
replace_once(
    "const HL_THR_MAX: usize = 16;\n"
    "static mut HL_THR_TID: [usize; HL_THR_MAX] = [0; HL_THR_MAX];\n"
    "static mut HL_THR_NAMES: [[u8; 8]; HL_THR_MAX] = [[0u8; 8]; HL_THR_MAX];\n"
    "static HL_THR_NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);",
    "const HL_THR_MAX: usize = 16;\n"
    "static mut HL_THR_TID: [usize; HL_THR_MAX] = [0; HL_THR_MAX];\n"
    "static mut HL_THR_NAMES: [[u8; 8]; HL_THR_MAX] = [[0u8; 8]; HL_THR_MAX];\n"
    "static HL_THR_NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);\n"
    "// v3.28.7: write-order per slot - recycled pthread_t resolves to the newest label.\n"
    "static mut HL_THR_SEQ: [u64; HL_THR_MAX] = [0; HL_THR_MAX];\n"
    "static HL_THR_SEQ_NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);",
    "thread_seq_statics",
)

# ---- 3) modules fn: strict labels + near merge + best span ----
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
            // v3.28.7: strict labels. libnative* variants live gigabytes apart;
            // merging them produced nonsense offsets, so each gets its own row.
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
                        // v3.28.7: extend a same-label row only when near;
                        // far same-name mappings stay as separate rows.
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
old_modules = text[start:end]
if 'fname.starts_with("libnative")' not in old_modules:
    raise RuntimeError("v3.28.6 modules fn shape unexpected")
text = text[:start] + NEW_MODULES + text[end:]

# ---- 4) thread register: seq-tracked, in-place same-label update, T: log ----
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
    let seq = HL_THR_SEQ_NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
    unsafe {
        let ids = std::ptr::addr_of_mut!(HL_THR_TID) as *mut usize;
        let names = std::ptr::addr_of_mut!(HL_THR_NAMES) as *mut u8;
        let seqs = std::ptr::addr_of_mut!(HL_THR_SEQ) as *mut u64;
        // v3.28.7: same-label entries update in place (init/push never evicted).
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
            target = HL_THR_NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed) % HL_THR_MAX;
        }
        *ids.add(target) = tid;
        let mut j = 0usize;
        while j < 8 {
            *names.add(target * 8 + j) = want[j];
            j += 1;
        }
        *seqs.add(target) = seq;
    }
    // v3.28.7: persist the tid mapping into the predict log for offline match.
    log_predict_step(&format!("T:{}={}", label, tid));
}'''

rstart = text.index("fn hl_thread_register(label: &str) {")
rend = text.index("fn hl_put_mod(", rstart)
text = text[:rstart] + NEW_REGISTER + "\n\n" + text[rend:]

# ---- 5) hl_put_mod: best-fit (smallest containing span) ----
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

# ---- 6) hl_put_thr: newest matching registration wins ----
NEW_PUT_THR = r'''fn hl_put_thr(msg: &mut [u8], mut len: usize, tid: usize) -> usize {
    let mut best = usize::MAX;
    let mut best_seq = 0u64;
    unsafe {
        let ids = std::ptr::addr_of!(HL_THR_TID) as *const usize;
        let seqs = std::ptr::addr_of!(HL_THR_SEQ) as *const u64;
        let mut i = 0usize;
        while i < HL_THR_MAX {
            if *ids.add(i) == tid && tid != 0 {
                let s = *seqs.add(i);
                if s > best_seq {
                    best_seq = s;
                    best = i;
                }
            }
            i += 1;
        }
        if best == usize::MAX {
            msg[len] = b'?';
            return len + 1;
        }
        let names = std::ptr::addr_of!(HL_THR_NAMES) as *const u8;
        let mut j = 0usize;
        while j < 8 {
            let c = *names.add(best * 8 + j);
            if c == 0 {
                break;
            }
            msg[len] = c;
            len += 1;
            j += 1;
        }
    }
    len
}'''

tstart = text.index("fn hl_put_thr(")
tend = text.index(
    'extern "C" fn crash_signal_handler(sig: i32, info: *mut libc::c_void, ctx: *mut libc::c_void) {',
    tstart,
)
text = text[:tstart] + NEW_PUT_THR + "\n\n" + text[tend:]

# ---- 7) arming sites -> hl_arm_recovery() (before helper exists) ----
pattern = re.compile(r"SIGSEGV_RECOVERY\.store\(true,[^;]*\);")
text, n_armed = pattern.subn("hl_arm_recovery();", text)
if n_armed < 9:
    raise RuntimeError(f"arming sites replaced={n_armed} (expect >=9)")
if "SIGSEGV_RECOVERY.store(true" in text:
    raise RuntimeError("store(true) still present before helper insert")

# ---- 8) helper with THE single store(true) ----
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
if text.count("SIGSEGV_RECOVERY.store(true") != 1:
    raise RuntimeError("store(true) count != 1 (helper only)")

# ---- 9) handler gate: longjmp only on the armed thread ----
replace_once(
    "    if SIGSEGV_RECOVERY.load(std::sync::atomic::Ordering::Relaxed) {\n"
    "        // Set cooldown: skip reads for 60 seconds",
    "    if SIGSEGV_RECOVERY.load(std::sync::atomic::Ordering::Relaxed)\n"
    "        && HL_RECOVERY_ARMED_TID.load(std::sync::atomic::Ordering::Relaxed) == fault_tid\n"
    "    {\n"
    "        // Set cooldown: skip reads for 60 seconds",
    "handler_longjmp_gate",
)

# ---- 10) verdict gate: RECOVERED only for the armed thread ----
replace_once(
    '    let verdict: &[u8] = if SIGSEGV_RECOVERY.load(std::sync::atomic::Ordering::Relaxed) {\n'
    '        &b" RECOVERED"[..]',
    '    let verdict: &[u8] = if SIGSEGV_RECOVERY.load(std::sync::atomic::Ordering::Relaxed)\n'
    "        && HL_RECOVERY_ARMED_TID.load(std::sync::atomic::Ordering::Relaxed) == fault_tid\n"
    "    {\n"
    '        &b" RECOVERED"[..]',
    "handler_verdict_gate",
)

# ---- 11) fa= / c= forensics fields ----
EXTRACT = (
    "    };\n"
    "    // v3.28.7: kernel fault address echo + SEGV code (1=MAPERR, 2=ACCERR).\n"
    "    let ctx_fault_addr = if ctx.is_null() {\n"
    "        0usize\n"
    "    } else {\n"
    "        unsafe { std::ptr::read_unaligned((ctx as *const u8).add(168) as *const usize) }\n"
    "    };\n"
    "    let segv_code = if info.is_null() {\n"
    "        0u32\n"
    "    } else {\n"
    "        unsafe { std::ptr::read_unaligned((info as *const u8).add(8) as *const u32) }\n"
    "    };\n"
    '    let seg_a = b" addr=0x";'
)
replace_once(
    '    };\n    let seg_a = b" addr=0x";',
    EXTRACT,
    "fa_c_extract",
)
replace_once(
    "    len = hl_put_thr(&mut msg, len, fault_tid);\n",
    "    len = hl_put_thr(&mut msg, len, fault_tid);\n"
    '    let seg_fa = b" fa=0x";\n'
    "    msg[len..len + seg_fa.len()].copy_from_slice(seg_fa);\n"
    "    len += seg_fa.len();\n"
    "    len = hl_put_hex(&mut msg, len, ctx_fault_addr);\n"
    '    let seg_cd = b" c=";\n'
    "    msg[len..len + seg_cd.len()].copy_from_slice(seg_cd);\n"
    "    len += seg_cd.len();\n"
    "    len = hl_put_uint(&mut msg, len, segv_code as usize);\n",
    "fa_c_pieces",
)

# ---- 12) S:enter / S:exit carry the writing thread's tid ----
replace_once(
    '    log_predict_step("S:enter"); // v3.28.6 forensics',
    '    log_predict_step(&format!("S:enter tid={}", unsafe { libc::pthread_self() as usize })); // v3.28.7',
    "senter_tid",
)
replace_once(
    '    log_predict_step("S:exit"); // v3.28.6 forensics: post-window marker',
    '    log_predict_step(&format!("S:exit tid={}", unsafe { libc::pthread_self() as usize })); // v3.28.7',
    "sexit_tid",
)

# ---- post-checks (fail closed) ----
for required in (
    "fn hl_arm_recovery()",
    "HL_RECOVERY_ARMED_TID",
    'fname == "libnative.so"',
    '"natwin"',
    '"nathelp"',
    "HL_THR_SEQ",
    "T:{}={}",
    "S:enter tid=",
    "S:exit tid=",
    'b" fa=0x"',
    'b" c="',
    "best_span",
    "best_seq",
    "HL_MOD_MAX: usize = 256",
):
    if required not in text:
        raise RuntimeError(f"post-check missing: {required}")
if 'starts_with("libnative")' in text:
    raise RuntimeError("old libnative prefix merge still present")
if text.count("HL_RECOVERY_ARMED_TID.load") != 2:
    raise RuntimeError("armed-tid gate count != 2")
if 'log_predict_step("S:enter")' in text or 'log_predict_step("S:exit")' in text:
    raise RuntimeError("plain S:enter/S:exit marker still present")

SOURCE.write_text(text, encoding="utf-8")
print(
    "crash_forensics_v3287=applied "
    f"armed_sites={n_armed} modules=strict+bestfit threads=seq+tidlog gates=2 fields=fa,c markers=tid"
)
