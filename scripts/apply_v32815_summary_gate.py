#!/usr/bin/env python3
"""v3.28.15: stop read_summary from walking a dead 育成 object (the real crash).

Device evidence (uma_predict.log, 2026-10-25, idling on the home screen):

    [208] S:buffs done
    [209] S:json
    CRASH at step 209 sig=11 RECOVERED
    [1] S:wdm ... [189] S:before_get_character
    CRASH at step 189 sig=11 RECOVERED
    ... (cycles forever)

`read_summary_inner_impl()` does:

    let sm_obj = call_getter_ref(wdm_class, wdm_inst, "get_SingleMode");
    if sm_obj.is_null() { return {"error":"no_sm"} }
    let chara_obj = call_getter_ref(sm_class, sm_obj, "get_Character");  // <-- SIGSEGV

On the home screen `get_SingleMode` still returns a *stale* WorkSingleModeData
(the finished run's object, or a half-disposed one): the null check passes and
`get_Character` dereferences freed memory. Each hit is caught by the
sigsetjmp/siglongjmp guard, which longjmps out from under whatever locks/allocator
state the fault left behind — repeated recoveries eventually kill the process
(that is the "闪退 on the home screen, doing nothing").

Fix (two cheap checks before touching Character, plus a longer cooldown):

1. class gate — IL2CPP stores Il2CppClass* at object+0x00; if it is not
   WorkSingleModeData the object was freed/reused -> bail out.
2. active-run gate — `_totalTurnNum` (ObscuredInt @+68, the same field the summary
   reports as raw_total_turn_num) is <= 0 when no run is in progress -> bail out.
3. SIGSEGV cooldown 60s -> 600s (all three store sites) so one bad read cannot
   loop every minute.

Both gates return before ANY field read of Character, so a non-training screen
never produces a fault at all.

Idempotent: refuses to patch twice.
"""

from pathlib import Path

LIB = Path("hachimi_ura_plugin/src/lib.rs")
MARKER = "v3.28.15: training-session gate"

GATE_ANCHOR = '    log_predict_step("S:before_chara_class");\n'

GATE = '''    // ===== v3.28.15: training-session gate =====
    // On the home screen / after a run, get_SingleMode can still return a stale
    // WorkSingleModeData whose Character is already freed. Calling get_Character
    // there is the SIGSEGV behind "CRASH at step 189/209 sig=11 RECOVERED".
    // Validate the object before touching anything else; bail out cleanly.
    if read_ptr_at(sm_obj as *const c_void, 0) as usize != sm_class as usize {
        ura_log(2, "read_summary: sm klass mismatch — not in training, skip");
        return r#"{"error":"not_in_training","reason":"sm_klass_mismatch"}"#.to_string();
    }
    let gate_total_turn = read_obscured_int_at(sm_obj as *const c_void, 68);
    if gate_total_turn <= 0 {
        ura_log(2, "read_summary: totalTurnNum<=0 — no active run, skip");
        return r#"{"error":"not_in_training","reason":"no_active_run"}"#.to_string();
    }
'''

COOLDOWN_OLD = "SIGSEGV_COOLDOWN_UNTIL.store(now + 60,"
COOLDOWN_NEW = "SIGSEGV_COOLDOWN_UNTIL.store(now + 600,"


def replace_once(src: str, old: str, new: str, label: str) -> str:
    count = src.count(old)
    assert count == 1, f"{label} anchor count={count}"
    return src.replace(old, new, 1)


def main() -> None:
    src = LIB.read_text(encoding="utf-8")
    if MARKER in src:
        print("v32815_summary_gate=already_applied")
        return
    src = replace_once(src, GATE_ANCHOR, GATE + GATE_ANCHOR, "before_chara_class")
    cooldown_sites = src.count(COOLDOWN_OLD)
    assert cooldown_sites >= 1, f"sigsegv_cooldown anchor count={cooldown_sites}"
    src = src.replace(COOLDOWN_OLD, COOLDOWN_NEW)
    LIB.write_text(src, encoding="utf-8")
    print(f"v32815_summary_gate=applied cooldown_sites={cooldown_sites}")


if __name__ == "__main__":
    main()
