#!/usr/bin/env python3
"""v3.28.14: install hooks on EVERY push_loop exit, not just the happy ones.

`push_loop()` waits up to 60 rounds for a successful `read_summary()` probe:

    for wait_round in 0..60 {
        if GAME_INITIALIZED { break; }            // exit A (v3.28.13 installs here)
        if !probe.contains("error") { ...install...; break; }   // exit B (installs)
        sleep(1s);
    }
    let init_summary = read_summary();            // <-- exit C: NOTHING installed

Exit C is the common one in practice: at the title screen / before a training run
`read_summary()` keeps returning an error, GAME_INITIALIZED is never set, and after
60 seconds the loop simply falls through with zero hooks installed. The only
installer left at that point is the `/api/event/choices` HTTP lazy retry — i.e. an
interceptor patch applied minutes into the session, while the target functions are
live on the game threads. That is the silent native SIGSEGV behind
v3.28.11 / v3.28.12 / v3.28.13.

Fix: after the wait loop, install unconditionally (all installers are idempotent)
and retry briefly until il2cpp is available, so the patch always happens inside the
boot window. Paired with the late-install guard in factor_index_badge::install(),
the HTTP lazy path can no longer patch anything mid-session.

Idempotent: refuses to patch twice (marker comment).
"""

from pathlib import Path

LIB = Path("hachimi_ura_plugin/src/lib.rs")
MARKER = "v3.28.14: unconditional boot install"

HELPER_ANCHOR = "fn push_loop() {"

HELPER = '''// ===== v3.28.14: unconditional boot install =====
// push_loop's wait loop has three exits: probe success (installs), GAME_INITIALIZED
// already set (v3.28.13 installs), and the 60-round timeout (installed NOTHING).
// The timeout exit used to fall through, leaving the /api/event/choices HTTP lazy
// retry as the only installer — an interceptor patch applied minutes into the
// session while the target functions were live on other threads (silent SIGSEGV).
// All installers are idempotent, so retrying here is safe.
unsafe fn install_all_boot_hooks() {
    install_training_hook();
    install_exec_training_hook();
    install_failure_rate_hook();
    install_event_choice_hook();
    factor_index_badge::install();
    install_api_sniff_hooks();
}

unsafe fn install_all_boot_hooks_retry() {
    if API.is_null() || (&*API).interceptor == 0 {
        return;
    }
    install_all_boot_hooks();
    if factor_index_badge::installed() {
        boot_trace("boot install: ok");
        return;
    }
    boot_trace("boot install: retry");
    for _ in 0..40u32 {
        std::thread::sleep(std::time::Duration::from_millis(500));
        if factor_index_badge::installed() {
            boot_trace("boot install: ok after retry");
            return;
        }
        install_all_boot_hooks();
    }
    boot_trace("boot install: gave up (il2cpp not ready)");
}

'''

CALL_ANCHOR = '''    let init_summary = read_summary();
    if !init_summary.contains("\\"error\\"") {
'''

CALL = '''    // v3.28.14: unconditional boot install — the 60-round timeout exit above
    // otherwise leaves nothing installed until an HTTP retry patches live code.
    unsafe {
        install_all_boot_hooks_retry();
    }
    let init_summary = read_summary();
    if !init_summary.contains("\\"error\\"") {
'''


def replace_once(src: str, old: str, new: str, label: str) -> str:
    count = src.count(old)
    assert count == 1, f"{label} anchor count={count}"
    return src.replace(old, new, 1)


def main() -> None:
    src = LIB.read_text(encoding="utf-8")
    if MARKER in src:
        print("v32814_boot_install=already_applied")
        return
    src = replace_once(src, HELPER_ANCHOR, HELPER + HELPER_ANCHOR, "push_loop_def")
    src = replace_once(src, CALL_ANCHOR, CALL, "init_summary")
    LIB.write_text(src, encoding="utf-8")
    print("v32814_boot_install=applied")


if __name__ == "__main__":
    main()
