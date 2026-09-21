#!/usr/bin/env python3
"""v3.28.8 crash forensics v4: raw registers + anon exec regions + module count.

The v3.28.7 sample:

    CRASH at step 120 sig=11 addr=0x0 tid=486854096128 pc=? lr=? thr=init fa=0x0 c=1 FATAL

showed a pure NULL fault (addr=fa=0x0, c=1 MAPERR) on the init thread
(registry-match: T:init=486854096128), but `pc=? lr=?` left the decisive
fork unanswered: null jump (pc=0) vs anonymous executable region (JIT /
trampoline) vs unknown module. Also, anonymous executable mappings were
never indexed, and raw hex values were never printed.

This patch (forensics only, zero data-path change):
  0) module table ceiling 256 -> 512 (anonymous exec regions add rows);
  1) crash message buffer 320 -> 384 bytes;
  2) `sp=0x…` field (sigcontext.sp @+424) - stack position context;
  3) pc/lr now print RAW hex plus attribution: `pc=0x…(<mod>+off)` or
     `pc=0x…(?)` when not in a known executable region;
  4) anonymous executable regions (no path / no '/') are indexed as `anon`
     (JIT / trampoline candidates);
  5) `mc=<n>` field - module table row count (validates the table actually
     parsed on this boot).

Note (self-check precision): the 320-byte check targets the HANDLER message
buffer only (`let mut msg = [0u8; 320]`). The crash-log path fix has an
unrelated `CRASH_LOG_FILE_BUF: [u8; 320]` that must stay.

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

SOURCE.write_text(text, encoding="utf-8")
print(
    "crash_forensics_v3288=applied "
    "raw_regs=pc,lr,sp anon_regions=1 module_count=mc mod_max=512 buffer=384"
)
