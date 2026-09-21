#!/usr/bin/env python3
"""v3.28.6 crash forensics ++ (diagnostic build, no behavior change).

v3.28.5's honest handler captured the first real crash sample:

    CRASH at step 95 sig=11 addr=0x72bf999f80 tid=486810748160 FATAL

after `[93] S:json_built [94] S:obs_done [95] S:cache_done` - i.e. the
fault is NOT in JSON assembly (that completed) and NOT inside the armed
recovery window (FATAL verdict): it sits at the very end of read_summary
or in a concurrent thread, touching a stale heap pointer (addr in the
0x72.. game-heap region).

This build adds attribution so the next crash names the culprit:
  1) crash handler also logs pc=/lr= (from the ucontext) with MODULE
     attribution (module table parsed from /proc/self/maps at init;
     names: hlpatch / il2cpp / native / libc / ... + offset).
  2) thread labels: push / http / init threads self-register; handler
     prints thr=<label> when the faulting tid matches, else thr=?.
  3) extra checkpoints: S:enter (read_summary entry, covers cache-hit
     path), S:exit (last statement of read_summary), H:ret / H:saved
     (/summary route + save_endpoint_log), P:read_done / P:push_start /
     P:push_done (push loop). Next crash's last marker localizes the
     window to a single statement.

All signal-path code is allocation-free, raw writes only. Idempotent:
`hl_put_mod(` present -> already_applied. Anchors fail closed.
"""
from pathlib import Path

SOURCE = Path("hachimi_ura_plugin/src/lib.rs")
text = SOURCE.read_text(encoding="utf-8")

if "hl_put_mod(" in text:
    print("crash_forensics_v3286=already_applied")
    raise SystemExit(0)


def replace_once(old: str, new: str, label: str) -> None:
    global text
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"{label} anchor count={count} (expect 1)")
    text = text.replace(old, new, 1)


NEWBLOCK = r'''// ===== v3.28.6 crash forensics: module table + thread labels =====
// Parsed from /proc/self/maps at init (normal context, allocation OK).
// The signal handler only reads these static tables - no locks, no allocs.
const HL_MOD_MAX: usize = 128;
static mut HL_MOD_BASE: [usize; HL_MOD_MAX] = [0; HL_MOD_MAX];
static mut HL_MOD_END: [usize; HL_MOD_MAX] = [0; HL_MOD_MAX];
static mut HL_MOD_NAME: [[u8; 16]; HL_MOD_MAX] = [[0u8; 16]; HL_MOD_MAX];
static mut HL_MOD_NAME_LEN: [u8; HL_MOD_MAX] = [0; HL_MOD_MAX];
static HL_MOD_COUNT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

const HL_THR_MAX: usize = 16;
static mut HL_THR_ID: [usize; HL_THR_MAX] = [0; HL_THR_MAX];
static mut HL_THR_NAME: [[u8; 8]; HL_THR_MAX] = [[0u8; 8]; HL_THR_MAX];
static HL_THR_COUNT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

fn hl_modules_init() {
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
            let short: &str = if fname.starts_with("libhachimi_ura") {
                "hlpatch"
            } else if fname.starts_with("libil2cpp") {
                "il2cpp"
            } else if fname.starts_with("libnative") {
                "native"
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
                        if b < *bases.add(i) { *bases.add(i) = b; }
                        if e > *ends.add(i) { *ends.add(i) = e; }
                        merged = true;
                        break;
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
}

fn hl_thread_register(label: &str) {
    let idx = HL_THR_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    if idx >= HL_THR_MAX {
        return;
    }
    let tid = unsafe { libc::pthread_self() as usize };
    unsafe {
        let ids = std::ptr::addr_of_mut!(HL_THR_ID) as *mut usize;
        let names = std::ptr::addr_of_mut!(HL_THR_NAME) as *mut u8;
        *ids.add(idx) = tid;
        let bytes = label.as_bytes();
        let n = bytes.len().min(7);
        let mut j = 0usize;
        while j < n {
            *names.add(idx * 8 + j) = bytes[j];
            j += 1;
        }
        *names.add(idx * 8 + n) = 0;
    }
}

fn hl_put_mod(msg: &mut [u8], mut len: usize, addr: usize) -> usize {
    let count = HL_MOD_COUNT.load(std::sync::atomic::Ordering::Relaxed);
    let mut i = 0usize;
    let mut hit = false;
    let mut base = 0usize;
    let mut name_ptr: *const u8 = std::ptr::null();
    let mut name_len = 0usize;
    unsafe {
        let bases = std::ptr::addr_of!(HL_MOD_BASE) as *const usize;
        let ends = std::ptr::addr_of!(HL_MOD_END) as *const usize;
        let names = std::ptr::addr_of!(HL_MOD_NAME) as *const u8;
        let lens = std::ptr::addr_of!(HL_MOD_NAME_LEN) as *const u8;
        while i < count && i < HL_MOD_MAX {
            let b = *bases.add(i);
            let e = *ends.add(i);
            if addr >= b && addr < e {
                hit = true;
                base = b;
                name_ptr = names.add(i * 16);
                name_len = *lens.add(i) as usize;
                break;
            }
            i += 1;
        }
        if !hit {
            msg[len] = b'?';
            return len + 1;
        }
        let mut j = 0usize;
        while j < name_len {
            msg[len] = *name_ptr.add(j);
            len += 1;
            j += 1;
        }
        msg[len] = b'+';
        len += 1;
        let off = addr.wrapping_sub(base);
        len = hl_put_hex(msg, len, off);
    }
    len
}

fn hl_put_thr(msg: &mut [u8], mut len: usize, tid: usize) -> usize {
    let count = HL_THR_COUNT.load(std::sync::atomic::Ordering::Relaxed);
    let mut i = 0usize;
    unsafe {
        let ids = std::ptr::addr_of!(HL_THR_ID) as *const usize;
        let names = std::ptr::addr_of!(HL_THR_NAME) as *const u8;
        while i < count && i < HL_THR_MAX {
            if *ids.add(i) == tid {
                let mut j = 0usize;
                while j < 8 {
                    let c = *names.add(i * 8 + j);
                    if c == 0 {
                        break;
                    }
                    msg[len] = c;
                    len += 1;
                    j += 1;
                }
                return len;
            }
            i += 1;
        }
    }
    msg[len] = b'?';
    len + 1
}

'''

# 1) definition block + renamed ctx parameter
replace_once(
    'extern "C" fn crash_signal_handler(sig: i32, info: *mut libc::c_void, _ctx: *mut libc::c_void) {',
    NEWBLOCK + 'extern "C" fn crash_signal_handler(sig: i32, info: *mut libc::c_void, ctx: *mut libc::c_void) {',
    "handler_signature_and_block",
)

# 2) bigger buffer (320: addr + tid + pc + lr + thr + verdict)
replace_once(
    "    let mut msg = [0u8; 200]; // v3.28.5: addr + tid + verdict",
    "    let mut msg = [0u8; 320]; // v3.28.6: addr + tid + pc + lr + thr + verdict",
    "handler_buffer",
)

# 3) pc/lr extraction from ucontext (arm64: uc_mcontext @168, regs[30] lr @416, pc @432)
replace_once(
    "    let fault_tid = unsafe { libc::pthread_self() as usize };",
    "    let fault_tid = unsafe { libc::pthread_self() as usize };\n"
    "    let (fault_pc, fault_lr) = unsafe {\n"
    "        if ctx.is_null() {\n"
    "            (0usize, 0usize)\n"
    "        } else {\n"
    "            let base = ctx as *const u8;\n"
    "            (\n"
    "                std::ptr::read_unaligned(base.add(432) as *const usize),\n"
    "                std::ptr::read_unaligned(base.add(416) as *const usize),\n"
    "            )\n"
    "        }\n"
    "    };",
    "handler_pc_lr_extract",
)

# 4) append pc=/lr=/thr= pieces before the verdict
replace_once(
    '    let seg_b = b" tid=";\n'
    "    msg[len..len + seg_b.len()].copy_from_slice(seg_b);\n"
    "    len += seg_b.len();\n"
    "    len = hl_put_uint(&mut msg, len, fault_tid);\n",
    '    let seg_b = b" tid=";\n'
    "    msg[len..len + seg_b.len()].copy_from_slice(seg_b);\n"
    "    len += seg_b.len();\n"
    "    len = hl_put_uint(&mut msg, len, fault_tid);\n"
    '    let seg_c = b" pc=";\n'
    "    msg[len..len + seg_c.len()].copy_from_slice(seg_c);\n"
    "    len += seg_c.len();\n"
    "    len = hl_put_mod(&mut msg, len, fault_pc);\n"
    '    let seg_d = b" lr=";\n'
    "    msg[len..len + seg_d.len()].copy_from_slice(seg_d);\n"
    "    len += seg_d.len();\n"
    "    len = hl_put_mod(&mut msg, len, fault_lr);\n"
    '    let seg_e = b" thr=";\n'
    "    msg[len..len + seg_e.len()].copy_from_slice(seg_e);\n"
    "    len += seg_e.len();\n"
    "    len = hl_put_thr(&mut msg, len, fault_tid);\n",
    "handler_pc_lr_thr_pieces",
)

# 5) module table init + register init thread
replace_once(
    "fn init_crash_handler() {\n"
    "    hl_crash_log_init();\n"
    "    hl_init_sigaltstack();\n"
    "    unsafe {",
    "fn init_crash_handler() {\n"
    "    hl_crash_log_init();\n"
    "    hl_init_sigaltstack();\n"
    "    hl_modules_init(); // v3.28.6 forensics: module table for pc attribution\n"
    '    hl_thread_register("init"); // v3.28.6 forensics\n'
    "    unsafe {",
    "init_crash_handler_modules",
)

# 6) read_summary entry marker (covers cache-hit path too)
replace_once(
    "fn read_summary() -> String {",
    'fn read_summary() -> String {\n    log_predict_step("S:enter"); // v3.28.6 forensics',
    "senter_marker",
)

# 7) read_summary exit marker (first statement after the window closes)
replace_once(
    "    // Keep the window open across the whole tail; close it at the very end.\n"
    "    SIGSEGV_RECOVERY.store(false, std::sync::atomic::Ordering::Relaxed);\n"
    "    summary\n"
    "}",
    "    // Keep the window open across the whole tail; close it at the very end.\n"
    "    SIGSEGV_RECOVERY.store(false, std::sync::atomic::Ordering::Relaxed);\n"
    '    log_predict_step("S:exit"); // v3.28.6 forensics: post-window marker\n'
    "    summary\n"
    "}",
    "sexit_marker",
)

# 8) /summary HTTP route: H:ret
replace_once(
    '    } else if path == "/summary" {\n'
    "        read_summary()\n"
    '    } else if path == "/debug/turn_probe" {',
    '    } else if path == "/summary" {\n'
    "        let __summary_body = read_summary();\n"
    '        log_predict_step("H:ret"); // v3.28.6 forensics\n'
    "        __summary_body\n"
    '    } else if path == "/debug/turn_probe" {',
    "hret_marker",
)

# 9) save_endpoint_log: H:saved
replace_once(
    "    save_endpoint_log(&path, &body);\n",
    "    save_endpoint_log(&path, &body);\n"
    '    if path == "/summary" {\n'
    '        log_predict_step("H:saved"); // v3.28.6 forensics\n'
    "    }\n",
    "hsaved_marker",
)

# 10) push loop: P:read_done
replace_once(
    "        let summary = read_summary();\n"
    '        if summary.contains("\\"error\\"") {',
    "        let summary = read_summary();\n"
    '        log_predict_step("P:read_done"); // v3.28.6 forensics\n'
    '        if summary.contains("\\"error\\"") {',
    "pread_done_marker",
)

# 11) push loop: P:push_start / P:push_done
replace_once(
    "            unsafe {\n"
    '                ura_log(3, "Push: data changed, pushing to app");\n'
    "            }\n"
    "            push_to_app(&summary);",
    "            unsafe {\n"
    '                ura_log(3, "Push: data changed, pushing to app");\n'
    "            }\n"
    '            log_predict_step("P:push_start"); // v3.28.6 forensics\n'
    "            push_to_app(&summary);\n"
    '            log_predict_step("P:push_done"); // v3.28.6 forensics',
    "ppush_markers",
)

# 12) push thread label
replace_once(
    "            .spawn(|| {\n"
    "                push_loop();\n"
    "            });",
    "            .spawn(|| {\n"
    '                hl_thread_register("push");\n'
    "                push_loop();\n"
    "            });",
    "push_thread_label",
)

# 13) http thread label
replace_once(
    "                        .spawn(move || handle_http(stream));",
    "                        .spawn(move || {\n"
    '                            hl_thread_register("http");\n'
    "                            handle_http(stream);\n"
    "                        });",
    "http_thread_label",
)

# ---- post-checks (fail closed) ----
for required in (
    "fn hl_put_mod(",
    "fn hl_put_thr(",
    "fn hl_modules_init()",
    'hl_thread_register("init")',
    'hl_thread_register("push")',
    'hl_thread_register("http")',
    'b" pc="',
    'b" lr="',
    'b" thr="',
    "S:enter",
    "S:exit",
    "H:ret",
    "H:saved",
    "P:read_done",
    "P:push_start",
    "P:push_done",
    "[0u8; 320]",
):
    if required not in text:
        raise RuntimeError(f"post-check missing: {required}")
if "_ctx: *mut libc::c_void" in text:
    raise RuntimeError("old _ctx signature still present")

SOURCE.write_text(text, encoding="utf-8")
print(
    "crash_forensics_v3286=applied "
    "pc_lr=1 modules=1 threads=3 markers=S:enter,S:exit,H:ret,H:saved,P:read_done,P:push_start,P:push_done "
    "buffer=320"
)
