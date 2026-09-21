#!/usr/bin/env python3
"""v3.28.8 crash forensics v4: raw registers + anon exec regions + rescan.

The v3.28.7 sample:

    CRASH at step 120 sig=11 addr=0x0 tid=486854096128 pc=? lr=? thr=init fa=0x0 c=1 FATAL

showed a pure NULL fault (addr=fa=0x0, c=1 MAPERR) on the init thread
(registry-match: T:init=486854096128), but `pc=? lr=?` left the decisive
fork unanswered: null jump (pc=0) vs anonymous executable region (JIT /
trampoline) vs missing module row. Anonymous exec mappings were never
indexed, raw hex values were never printed, and the module table was built
once at init - lazily-loaded libraries never got rows.

This patch (forensics only, zero data-path change):
  0) module table ceiling 256 -> 512 (anonymous exec regions add rows);
  1) crash message buffer 320 -> 384 bytes;
  2) `sp=0x…` field (sigcontext.sp @+424) - stack position calibration;
  3) pc/lr now print RAW hex plus attribution: `pc=0x…(<mod>+off)` or
     `pc=0x…(?)` when not in any known executable region;
  4) anonymous executable regions (no path / no '/') are indexed as `anon`;
  5) `mc=<n>` field - module table row count at crash time;
  6) periodic rescan: `hl_modules_rescan_maybe()` (15 s throttle, CAS gate)
     re-parses /proc/self/maps so lazily-loaded libs / new anon regions get
     attributed; logs `M:rescan=<rows>`;
  7) hl_put_mod clamps cached name length to 15 (torn-read hardening).

Self-check precision: the stale-buffer guard targets the HANDLER message
buffer only (`let mut msg = [0u8; 320]`). The crash-log path fix has an
unrelated `CRASH_LOG_FILE_BUF: [u8; 320]` that must stay - asserted present.

Idempotent: `fn hl_put_paren_mod(` present -> already_applied.
"""
from pathlib import Path

SOURCE = Path("hachimi_ura_plugin/src/lib.rs")
text = SOURCE.read_text(encoding="utf-8")

if "fn hl_put_paren_mod(" in text:
    print("crash_forensics_v3288=already_applied")
    raise SystemExit(0)


def replace_once(old: str, new: str, label: str) -> None:
    global text
    c = text.count(old)
    if c != 1:
        raise RuntimeError(f"{label} anchor count={c} (expect 1)")
    text = text.replace(old, new, 1)


# ---- 0) module table ceiling 256 -> 512 ----
replace_once(
    "const HL_MOD_MAX: usize = 256; // v3.28.7: room for per-module rows",
    "const HL_MOD_MAX: usize = 512; // v3.28.8: anonymous exec regions included",
    "mod_max_512",
)

# ---- 1) handler buffer 320 -> 384 ----
replace_once(
    "    let mut msg = [0u8; 320]; // v3.28.6: addr + tid + pc + lr + thr + verdict",
    "    let mut msg = [0u8; 384]; // v3.28.8: addr + tid + pc/lr/sp(raw) + thr + fa + c + mc",
    "buf384",
)

# ---- 2) sp extraction in the context tuple ----
old_tuple = (
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
    "    };"
)
new_tuple = (
    "    let (fault_pc, fault_lr, fault_sp) = unsafe {\n"
    "        if ctx.is_null() {\n"
    "            (0usize, 0usize, 0usize)\n"
    "        } else {\n"
    "            let base = ctx as *const u8;\n"
    "            (\n"
    "                std::ptr::read_unaligned(base.add(432) as *const usize),\n"
    "                std::ptr::read_unaligned(base.add(416) as *const usize),\n"
    "                std::ptr::read_unaligned(base.add(424) as *const usize),\n"
    "            )\n"
    "        }\n"
    "    };"
)
replace_once(old_tuple, new_tuple, "sp_extract")

# ---- 3) pc/lr raw + paren attribution; sp field ----
old_pieces = (
    '    let seg_c = b" pc=";\n'
    "    msg[len..len + seg_c.len()].copy_from_slice(seg_c);\n"
    "    len += seg_c.len();\n"
    "    len = hl_put_mod(&mut msg, len, fault_pc);\n"
    '    let seg_d = b" lr=";\n'
    "    msg[len..len + seg_d.len()].copy_from_slice(seg_d);\n"
    "    len += seg_d.len();\n"
    "    len = hl_put_mod(&mut msg, len, fault_lr);"
)
new_pieces = (
    '    let seg_c = b" pc=0x";\n'
    "    msg[len..len + seg_c.len()].copy_from_slice(seg_c);\n"
    "    len += seg_c.len();\n"
    "    len = hl_put_hex(&mut msg, len, fault_pc);\n"
    "    len = hl_put_paren_mod(&mut msg, len, fault_pc);\n"
    '    let seg_d = b" lr=0x";\n'
    "    msg[len..len + seg_d.len()].copy_from_slice(seg_d);\n"
    "    len += seg_d.len();\n"
    "    len = hl_put_hex(&mut msg, len, fault_lr);\n"
    "    len = hl_put_paren_mod(&mut msg, len, fault_lr);\n"
    '    let seg_sp = b" sp=0x";\n'
    "    msg[len..len + seg_sp.len()].copy_from_slice(seg_sp);\n"
    "    len += seg_sp.len();\n"
    "    len = hl_put_hex(&mut msg, len, fault_sp);"
)
replace_once(old_pieces, new_pieces, "pc_lr_sp_pieces")

# ---- 4) mc field after c= ----
old_cd = (
    '    let seg_cd = b" c=";\n'
    "    msg[len..len + seg_cd.len()].copy_from_slice(seg_cd);\n"
    "    len += seg_cd.len();\n"
    "    len = hl_put_uint(&mut msg, len, segv_code as usize);"
)
new_cd = (
    old_cd + "\n"
    '    let seg_mc = b" mc=";\n'
    "    msg[len..len + seg_mc.len()].copy_from_slice(seg_mc);\n"
    "    len += seg_mc.len();\n"
    "    len = hl_put_uint(\n"
    "        &mut msg,\n"
    "        len,\n"
    "        HL_MOD_COUNT.load(std::sync::atomic::Ordering::Relaxed),\n"
    "    );"
)
replace_once(old_cd, new_cd, "mc_field")

# ---- 5) helper: (mod+off) or (?) ----
helper = (
    "fn hl_put_paren_mod(msg: &mut [u8], mut len: usize, addr: usize) -> usize {\n"
    "    msg[len] = b'(';\n"
    "    len += 1;\n"
    "    len = hl_put_mod(msg, len, addr);\n"
    "    msg[len] = b')';\n"
    "    len + 1\n"
    "}\n"
    "\n"
)
replace_once(
    "fn hl_put_mod(msg: &mut [u8], mut len: usize, addr: usize) -> usize {",
    helper + "fn hl_put_mod(msg: &mut [u8], mut len: usize, addr: usize) -> usize {",
    "paren_helper",
)

# ---- 6a) module parse: stop skipping anonymous exec regions ----
replace_once(
    "            if !perms.contains('x') || path.is_empty() || !path.contains('/') {\n"
    "                continue;\n"
    "            }",
    "            if !perms.contains('x') {\n"
    "                continue;\n"
    "            }",
    "anon_accept",
)

# ---- 6b) label anonymous exec regions as `anon` ----
replace_once(
    '            let short: &str = if fname.starts_with("libhachimi_ura") {',
    '            let short: &str = if path.is_empty() || !path.contains(\'/\') {\n'
    '                "anon"\n'
    '            } else if fname.starts_with("libhachimi_ura") {',
    "anon_label",
)

# ---- 6c) clamp cached name length (torn-read hardening) ----
replace_once(
    "        let name_len = *lens.add(best) as usize;",
    "        let name_len = (*lens.add(best) as usize).min(15); // v3.28.8: clamped",
    "name_len_clamp",
)

# ---- 7) periodic rescan block (after modules_init, before thread registry) ----
RESCAN_BLOCK = (
    "\n"
    "static HL_MOD_LAST_SCAN_MS: std::sync::atomic::AtomicU64 =\n"
    "    std::sync::atomic::AtomicU64::new(0);\n"
    "\n"
    "fn hl_now_ms() -> u64 {\n"
    "    use std::time::{SystemTime, UNIX_EPOCH};\n"
    "    SystemTime::now()\n"
    "        .duration_since(UNIX_EPOCH)\n"
    "        .map(|d| d.as_millis() as u64)\n"
    "        .unwrap_or(0)\n"
    "}\n"
    "\n"
    "/// v3.28.8: re-parse /proc/self/maps so lazily-loaded libraries and new\n"
    "/// anonymous executable regions are attributed in future crash records.\n"
    "/// Throttled to once per 15 s; a CAS gate keeps a single scanner at a time.\n"
    "fn hl_modules_rescan_maybe() {\n"
    "    let now = hl_now_ms();\n"
    "    let last = HL_MOD_LAST_SCAN_MS.load(std::sync::atomic::Ordering::Relaxed);\n"
    "    if last != 0 && now.saturating_sub(last) < 15000 {\n"
    "        return;\n"
    "    }\n"
    "    if HL_MOD_LAST_SCAN_MS\n"
    "        .compare_exchange(\n"
    "            last,\n"
    "            now,\n"
    "            std::sync::atomic::Ordering::Relaxed,\n"
    "            std::sync::atomic::Ordering::Relaxed,\n"
    "        )\n"
    "        .is_err()\n"
    "    {\n"
    "        return;\n"
    "    }\n"
    "    hl_modules_init();\n"
    "    log_predict_step(&format!(\n"
    "        \"M:rescan={}\",\n"
    "        HL_MOD_COUNT.load(std::sync::atomic::Ordering::Relaxed)\n"
    "    ));\n"
    "}\n"
)
replace_once(
    "    HL_MOD_COUNT.store(count, std::sync::atomic::Ordering::Relaxed);\n}",
    "    HL_MOD_COUNT.store(count, std::sync::atomic::Ordering::Relaxed);\n}"
    + RESCAN_BLOCK,
    "rescan_block",
)

# ---- 8) call rescan at read_summary entry ----
replace_once(
    "fn read_summary() -> String {\n"
    '    log_predict_step(&format!("S:enter tid={}", unsafe { libc::pthread_self() as usize })); // v3.28.7',
    "fn read_summary() -> String {\n"
    "    hl_modules_rescan_maybe(); // v3.28.8 forensics: pick up lazy mappings\n"
    '    log_predict_step(&format!("S:enter tid={}", unsafe { libc::pthread_self() as usize })); // v3.28.7',
    "rescan_call",
)

# ---- post-checks (fail closed) ----
for required in (
    "fn hl_put_paren_mod(",
    'b" pc=0x"',
    'b" lr=0x"',
    'b" sp=0x"',
    'b" mc="',
    '"anon"',
    "let mut msg = [0u8; 384];",
    "fault_sp",
    "read_unaligned(base.add(424)",
    "path.is_empty() || !path.contains",
    "const HL_MOD_MAX: usize = 512;",
    "fn hl_modules_rescan_maybe()",
    "HL_MOD_LAST_SCAN_MS",
    "M:rescan=",
    ".min(15)",
    "CRASH_LOG_FILE_BUF",
):
    if required not in text:
        raise RuntimeError(f"post-check missing: {required}")
if 'let seg_c = b" pc=";' in text:
    raise RuntimeError("old pc piece still present")
if 'let seg_d = b" lr=";' in text:
    raise RuntimeError("old lr piece still present")
if "let mut msg = [0u8; 320]" in text:
    raise RuntimeError("old handler msg buffer still present")
if "!perms.contains('x') || path.is_empty()" in text:
    raise RuntimeError("old module skip condition still present")
if "const HL_MOD_MAX: usize = 256;" in text:
    raise RuntimeError("old module ceiling still present")
if text.count("hl_modules_rescan_maybe") < 2:
    raise RuntimeError("rescan call/definition missing")

SOURCE.write_text(text, encoding="utf-8")
print(
    "crash_forensics_v3288=applied "
    "raw_regs=pc,lr,sp anon_regions=1 module_count=mc rescan=15s "
    "mod_max=512 buffer=384 clamp=15"
)
