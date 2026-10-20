#!/usr/bin/env python3
"""v3.28.13: fix the push_loop install race (root cause of the v3.28.11/v3.28.12
silent SIGSEGVs).

push_loop() breaks out of its wait loop as soon as GAME_INITIALIZED is true —
but GAME_INITIALIZED can be set by paths that never install the hooks. The
event-choices lazy retry then installs interceptors mid-game, potentially while
the target functions are executing, which crashes the process natively with no
crashlog.

Fix: when the loop observes GAME_INITIALIZED already set, run the same
idempotent install block before breaking. All install functions are guarded
(HOOK_ADDR != 0 early-return etc.), so re-install is safe.

Idempotent: checks for its own marker comment before patching.
"""

from pathlib import Path

LIB = Path("hachimi_ura_plugin/src/lib.rs")
MARKER = "v3.28.13: install hooks before breaking"
ANCHOR = """        if GAME_INITIALIZED.load(Ordering::Relaxed) {
            break;
        }
        boot_trace("push_probe_begin");"""
REPLACEMENT = """        if GAME_INITIALIZED.load(Ordering::Relaxed) {
            // v3.28.13: install hooks before breaking — another path may have set
            // GAME_INITIALIZED without installing (all installs are idempotent).
            unsafe {
                install_training_hook();
                install_exec_training_hook();
                install_failure_rate_hook();
                install_event_choice_hook();
                factor_index_badge::install();
                install_api_sniff_hooks();
            }
            break;
        }
        boot_trace("push_probe_begin");"""


def main() -> None:
    src = LIB.read_text()
    if MARKER in src:
        print("pushloop_install_fix=already_applied")
        return
    count = src.count(ANCHOR)
    assert count == 1, f"anchor count {count} != 1 — lib.rs drifted"
    LIB.write_text(src.replace(ANCHOR, REPLACEMENT))
    print("pushloop_install_fix=applied")


if __name__ == "__main__":
    main()
