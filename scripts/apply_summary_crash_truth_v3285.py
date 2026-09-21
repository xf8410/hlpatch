#!/usr/bin/env python3
"""v3.28.5 summary crash truth + recovery-window fix (stability-only).

Evidence (uma_predict.log, both v3.28.4 sessions):
  * session tail: `[N] S:json` then `CRASH at step N sig=11 RECOVERED`,
    next session restarts at `[1] S:wdm` -> the process died twice at the
    pass boundary right after the /summary JSON stage.

Two facts found while auditing the generated source (read-only CI dumps):
  1) The crash handler writes " RECOVERED" UNCONDITIONALLY, before checking
     SIGSEGV_RECOVERY. The log could never distinguish a saved process from
     a dead one; both sessions restarted from [1], so the longjmp trampoline
     did NOT save them (or the fault was in an unarmed window).
  2) read_summary clears SIGSEGV_RECOVERY right after catch_unwind returns,
     BEFORE observe_ramen_transition / cache update / return - a fault in
     that tail is not recoverable at all.

This patch (no data behavior change):
  A) handler truth: sigaction(SA_SIGINFO|SA_ONSTACK) + sigaltstack; message
     carries fault address (raw siginfo_t +16), faulting thread id, and the
     real verdict (RECOVERED only when the longjmp trampoline engages,
     else FATAL). Allocation-free, raw syscalls only. The siginfo pointer
     is typed *mut c_void so the patch does not depend on libc::siginfo_t.
  B) checkpoints: S:json_built after the JSON format! completes, plus
     S:obs_done / S:cache_done in read_summary's tail.
  C) recovery window: SIGSEGV_RECOVERY is cleared only at the very end of
     read_summary, covering observe/cache/return.
  D) deep stacks: push thread + per-request HTTP threads spawn with 4 MiB
     stacks (summary reads run on both).

Idempotent: 'S:json_built' present -> already_applied. Anchors fail closed.
"""
from pathlib import Path

SOURCE = Path("hachimi_ura_plugin/src/lib.rs")
text = SOURCE.read_text(encoding="utf-8")

if "S:json_built" in text or "hl_put_hex" in text:
    print("summary_crash_truth=already_applied")
    raise SystemExit(0)


def replace_once(old: str, new: str, label: str) -> None:
    global text
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"{label} anchor count={count} (expect 1)")
    text = text.replace(old, new, 1)


# ---- A1) allocation-free helpers + alt stack + sigaction install ----
handler_sig_old = 'extern "C" fn crash_signal_handler(sig: i32) {'
handler_sig_new = r'''// v3.28.5 summary crash truth: allocation-free helpers for the signal
// handler, a dedicated alt stack (so stack-exhaustion faults are catchable)
// and sigaction-based installation with SA_SIGINFO|SA_ONSTACK.
fn hl_put_uint(msg: &mut [u8], mut len: usize, mut n: usize) -> usize {
    if n == 0 {
        msg[len] = b'0';
        return len + 1;
    }
    let mut digits = [0u8; 20];
    let mut dlen = 0usize;
    while n > 0 && dlen < 20 {
        digits[dlen] = b'0' + (n % 10) as u8;
        n /= 10;
        dlen += 1;
    }
    while dlen > 0 {
        dlen -= 1;
        msg[len] = digits[dlen];
        len += 1;
    }
    len
}

fn hl_put_hex(msg: &mut [u8], mut len: usize, v: usize) -> usize {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut started = false;
    let mut shift = (std::mem::size_of::<usize>() * 8) - 4;
    loop {
        let nib = (v >> shift) & 0xf;
        if nib != 0 || started || shift == 0 {
            msg[len] = HEX[nib as usize];
            len += 1;
            started = true;
        }
        if shift == 0 {
            break;
        }
        shift -= 4;
    }
    len
}

#[repr(align(16))]
struct HlSigStack([u8; 65536]);
static mut HL_SIG_STACK: HlSigStack = HlSigStack([0u8; 65536]);

fn hl_init_sigaltstack() {
    unsafe {
        let stack = libc::stack_t {
            ss_sp: std::ptr::addr_of_mut!(HL_SIG_STACK) as *mut libc::c_void,
            ss_flags: 0,
            ss_size: 65536,
        };
        libc::sigaltstack(&stack, std::ptr::null_mut());
    }
}

unsafe fn hl_install_signal(sig: i32) {
    let mut action: libc::sigaction = std::mem::zeroed();
    action.sa_sigaction = crash_signal_handler as usize;
    action.sa_flags = libc::SA_SIGINFO | libc::SA_ONSTACK;
    let _ = libc::sigemptyset(&mut action.sa_mask);
    libc::sigaction(sig, &action, std::ptr::null_mut());
}

extern "C" fn crash_signal_handler(sig: i32, info: *mut libc::c_void, _ctx: *mut libc::c_void) {'''
replace_once(handler_sig_old, handler_sig_new, "handler_signature")

# ---- A2) bigger message buffer ----
replace_once(
    "    let mut msg = [0u8; 64];",
    "    let mut msg = [0u8; 200]; // v3.28.5: addr + tid + verdict",
    "handler_msg_buffer",
)

# ---- A3) verdict block: addr + tid + real RECOVERED/FATAL ----
verdict_old = r'''    let r = b" RECOVERED";
    msg[len..len + r.len()].copy_from_slice(r);
    len += r.len();
    msg[len] = b'\n';
    len += 1;
'''
verdict_new = r'''    let fault_addr = if info.is_null() {
        0usize
    } else {
        // siginfo_t on arm64/bionic: si_signo(0) si_errno(4) si_code(8)
        // __pad0(12..16), union starts at 16 and _sigfault.si_addr is the
        // first member of that union.
        unsafe { std::ptr::read_unaligned((info as *const u8).add(16) as *const usize) }
    };
    let fault_tid = unsafe { libc::pthread_self() as usize };
    let seg_a = b" addr=0x";
    msg[len..len + seg_a.len()].copy_from_slice(seg_a);
    len += seg_a.len();
    len = hl_put_hex(&mut msg, len, fault_addr);
    let seg_b = b" tid=";
    msg[len..len + seg_b.len()].copy_from_slice(seg_b);
    len += seg_b.len();
    len = hl_put_uint(&mut msg, len, fault_tid);
    let verdict: &[u8] = if SIGSEGV_RECOVERY.load(std::sync::atomic::Ordering::Relaxed) {
        &b" RECOVERED"[..]
    } else {
        &b" FATAL"[..]
    };
    msg[len..len + verdict.len()].copy_from_slice(verdict);
    len += verdict.len();
    msg[len] = b'\n';
    len += 1;
'''
replace_once(verdict_old, verdict_new, "handler_verdict")

# ---- A4) install via sigaction + alt stack ----
install_old = '''    unsafe {
        let handler = crash_signal_handler as usize;
        sys_signal(11, handler); // SIGSEGV
        sys_signal(6, handler); // SIGABRT
        sys_signal(7, handler); // SIGBUS
        sys_signal(8, handler); // SIGFPE
    }'''
install_new = '''    hl_init_sigaltstack();
    unsafe {
        // v3.28.5 summary crash truth: SA_SIGINFO for the fault address,
        // SA_ONSTACK so stack-exhaustion faults stay catchable.
        hl_install_signal(11); // SIGSEGV
        hl_install_signal(6); // SIGABRT
        hl_install_signal(7); // SIGBUS
        hl_install_signal(8); // SIGFPE
    }'''
replace_once(install_old, install_new, "handler_install")

# ---- B1) S:json_built checkpoint ----
replace_once(
    '    log_predict_step("S:json");\n    format!(',
    '    log_predict_step("S:json");\n    let summary_body = format!(',
    "json_built_open",
)
replace_once(
    '''        last_action_json
    )
}

// ============================================================
// HTTP Server''',
    '''        last_action_json
    );
    log_predict_step("S:json_built"); // v3.28.5 summary crash truth: JSON stage done
    summary_body
}

// ============================================================
// HTTP Server''',
    "json_built_close",
)

# ---- B2+C) tail checkpoints + extended recovery window ----
tail_old = '''    // Clear recovery flag — normal return, no crash
    SIGSEGV_RECOVERY.store(false, std::sync::atomic::Ordering::Relaxed);
    // v3.24.71: compare only fresh runtime reads (never cache hits).
    observe_ramen_transition(&summary, now);
    // ★ v3.22.51: Update cache
    if let Ok(mut guard) = CACHED_SUMMARY.lock() {
        *guard = Some((summary.clone(), now));
    }
    summary
}'''
tail_new = '''    // v3.24.71: compare only fresh runtime reads (never cache hits).
    observe_ramen_transition(&summary, now);
    log_predict_step("S:obs_done"); // v3.28.5 summary crash truth: tail checkpoint A
    // ★ v3.22.51: Update cache
    if let Ok(mut guard) = CACHED_SUMMARY.lock() {
        *guard = Some((summary.clone(), now));
    }
    log_predict_step("S:cache_done"); // v3.28.5 summary crash truth: tail checkpoint B
    // v3.28.5 summary crash truth: the old code cleared the recovery flag
    // BEFORE observe/cache/return, so a fault in that tail killed the
    // process even though the handler message still said RECOVERED.
    // Keep the window open across the whole tail; close it at the very end.
    SIGSEGV_RECOVERY.store(false, std::sync::atomic::Ordering::Relaxed);
    summary
}'''
replace_once(tail_old, tail_new, "summary_tail_window")

# ---- D) deep stacks for the two summary-reading threads ----
stack_push_old = '''        std::thread::spawn(|| {
            push_loop();
        });'''
stack_push_new = '''        // v3.28.5 summary crash truth: deep stack for the summary-reading
        // push thread (the giant JSON builder runs here).
        let _ = std::thread::Builder::new()
            .name("hlpush".to_string())
            .stack_size(4 * 1024 * 1024)
            .spawn(|| {
                push_loop();
            });'''
replace_once(stack_push_old, stack_push_new, "push_thread_stack")

stack_http_old = "                    std::thread::spawn(move || handle_http(stream));"
stack_http_new = '''                    // v3.28.5 summary crash truth: deep stack for per-request
                    // threads — the floating app polls /summary on these.
                    let _ = std::thread::Builder::new()
                        .name("hlhttp".to_string())
                        .stack_size(4 * 1024 * 1024)
                        .spawn(move || handle_http(stream));'''
replace_once(stack_http_old, stack_http_new, "http_thread_stack")

# ---- post-checks (fail closed) ----
for required in (
    "S:json_built",
    "S:obs_done",
    "S:cache_done",
    "hl_install_signal(11);",
    "hl_init_sigaltstack();",
    " FATAL",
    "let summary_body = format!(",
):
    if required not in text:
        raise RuntimeError(f"post-check missing: {required}")
if 'let r = b" RECOVERED";' in text:
    raise RuntimeError("old unconditional verdict still present")

SOURCE.write_text(text, encoding="utf-8")
print(
    "summary_crash_truth=applied "
    "json_built=1 tail_markers=2 handler=siginfo+altstack stack_bumps=2 verdict=real"
)
