#!/usr/bin/env python3
"""v3.28.2 crash-log-path fix (baseline pinned to v3.28.0 by the workflow).

Why this exists
---------------
Every crash sink in this plugin wrote to

    /data/data/jp.pokemon.pokeuma/files/uma_predict.log

but `jp.pokemon.pokeuma` is not the package running on device — the real one,
read from /proc/self/cmdline, is `jp.co.cygames.umamusume`. Opening that path
always failed, so

  * crash_signal_handler records ("CRASH at step N sig=11") were dropped,
  * the panic hook record was dropped,
  * every log_predict_step line was dropped,
  * read_crash_log / the GitHub auto-upload had nothing to read or send.

That is why the 育成第五回合 crash left zero evidence on device.

Fix
---
Resolve a directory the app can actually write to without any permission —
the same one ura_boot.log already lands in successfully:

    /sdcard/Android/media/<pkg>/hachimi/uma_predict.log

Legacy paths stay as fallbacks so behaviour is never worse than before, and the
signal-handler path is pre-warmed into a static buffer so the handler itself
allocates nothing (async-signal-safe).

Anchor policy: every anchor must match at least once (fail closed otherwise);
all matches are replaced, and the count is reported. The OpenOptions block in
particular appears twice on purpose (panic hook + the v3.22.51 std::fs
fallback inside log_predict_step) and both must be redirected.

Idempotent: once the marker is present, re-runs print already_applied.
"""
from pathlib import Path

SOURCE = Path("hachimi_ura_plugin/src/lib.rs")
CARGO = Path("hachimi_ura_plugin/Cargo.toml")
LOCK = Path("hachimi_ura_plugin/Cargo.lock")

MARK = "// ===== v3.28.2 crash-log-path fix ====="
BASE_VERSION = "3.28.0"
NEW_VERSION = "3.28.2"

HELPERS = r'''// ===== v3.28.2 crash-log-path fix =====
// Resolves a user-reachable crash-log directory (Android/media/<pkg>/hachimi,
// the same place ura_boot.log already writes successfully) instead of the
// dead /data/data/jp.pokemon.pokeuma path, which belongs to a package that is
// not running on device and silently swallowed every crash record.
// The signal handler reads a pre-warmed static buffer, so it never allocates.
static mut CRASH_LOG_FILE_BUF: [u8; 320] = [0u8; 320];
static CRASH_LOG_FILE_READY: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

fn hl_pkg_name() -> String {
    let raw = std::fs::read("/proc/self/cmdline").unwrap_or_default();
    String::from_utf8_lossy(&raw)
        .trim_matches(char::from(0))
        .trim()
        .to_string()
}

/// Log directory, resolved once. Falls back to the historical private path
/// when the package name cannot be trusted, so this can never be worse than
/// the behaviour it replaces.
fn hl_crash_log_dir() -> &'static str {
    static DIR: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    DIR.get_or_init(|| {
        let pkg = hl_pkg_name();
        let plausible = pkg.len() >= 4
            && pkg.len() < 128
            && pkg.contains('.')
            && !pkg.contains('/')
            && !pkg.contains(' ');
        if plausible {
            let dir = format!("/sdcard/Android/media/{}/hachimi", pkg);
            if std::fs::create_dir_all(&dir).is_ok() {
                return dir;
            }
        }
        "/data/data/jp.pokemon.pokeuma/files".to_string()
    })
    .as_str()
}

fn hl_crash_log_file() -> String {
    format!("{}/uma_predict.log", hl_crash_log_dir())
}

/// Pre-warm the NUL-terminated path used from the signal handler. Called from
/// init_crash_handler, i.e. before any signal can arrive.
fn hl_crash_log_init() {
    let path = hl_crash_log_file();
    let bytes = path.as_bytes();
    unsafe {
        let buf = std::ptr::addr_of_mut!(CRASH_LOG_FILE_BUF) as *mut u8;
        let n = bytes.len().min(319);
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), buf, n);
        *buf.add(n) = 0;
    }
    CRASH_LOG_FILE_READY.store(true, std::sync::atomic::Ordering::Release);
    boot_trace(&format!("crash_log={}", path));
}

/// NUL-terminated path slice for raw syscalls. Allocates nothing.
fn hl_crash_log_file_cstr() -> &'static [u8] {
    const FALLBACK: &[u8] = b"/data/local/tmp/uma_predict.log\0";
    if !CRASH_LOG_FILE_READY.load(std::sync::atomic::Ordering::Acquire) {
        return FALLBACK;
    }
    unsafe {
        let base = std::ptr::addr_of!(CRASH_LOG_FILE_BUF) as *const u8;
        let mut end = 0usize;
        while end < 319 && *base.add(end) != 0 {
            end += 1;
        }
        if end == 0 {
            return FALLBACK;
        }
        std::slice::from_raw_parts(base, end + 1)
    }
}

'''

# (label, old, new) — applied in this order; each must match at least once.
REPLACEMENTS = [
    (
        "helpers",
        'const CRASH_LOG_PATH: &str = "/data/data/jp.pokemon.pokeuma/files/uma_predict.log";',
        HELPERS
        + 'const CRASH_LOG_PATH: &str = "/data/data/jp.pokemon.pokeuma/files/uma_predict.log";',
    ),
    (
        "signal_handler_path",
        '    let path = b"/data/data/jp.pokemon.pokeuma/files/uma_predict.log\\0";',
        '    let path = hl_crash_log_file_cstr();',
    ),
    (
        "openoptions_sinks",
        '        let _ = std::fs::OpenOptions::new()\n'
        '            .create(true)\n'
        '            .append(true)\n'
        '            .open("/data/data/jp.pokemon.pokeuma/files/uma_predict.log")',
        '        let _ = std::fs::OpenOptions::new()\n'
        '            .create(true)\n'
        '            .append(true)\n'
        '            .open(hl_crash_log_file())',
    ),
    (
        "predict_step_writer",
        '    let path1 = b"/data/data/jp.pokemon.pokeuma/files/uma_predict.log\\0";\n'
        '    let path2 = b"/data/local/tmp/uma_predict.log\\0";\n'
        '    let line_bytes = line.as_bytes();',
        '    let path1 = hl_crash_log_file_cstr();\n'
        '    let path2 = b"/data/local/tmp/uma_predict.log\\0";\n'
        '    let line_bytes = line.as_bytes();',
    ),
    (
        "predict_step_truncate",
        '    let path1 = b"/data/data/jp.pokemon.pokeuma/files/uma_predict.log\\0";\n'
        '    let path2 = b"/data/local/tmp/uma_predict.log\\0";\n'
        '    unsafe {',
        '    let path1 = hl_crash_log_file_cstr();\n'
        '    let path2 = b"/data/local/tmp/uma_predict.log\\0";\n'
        '    unsafe {',
    ),
    (
        "crash_log_reader",
        '    match std::fs::read_to_string("/data/data/jp.pokemon.pokeuma/files/uma_predict.log") {\n'
        '        Ok(s) if !s.is_empty() => s,\n'
        '        _ => match std::fs::read_to_string("/data/local/tmp/uma_predict.log") {\n'
        '            Ok(s) if !s.is_empty() => s,\n'
        '            _ => r#"{"error":"no_crash_log"}"#.to_string(),\n'
        '        },\n'
        '    }',
        '    let mut parts: Vec<String> = Vec::new();\n'
        '    for p in [\n'
        '        hl_crash_log_file(),\n'
        '        "/data/data/jp.pokemon.pokeuma/files/uma_predict.log".to_string(),\n'
        '        "/data/local/tmp/uma_predict.log".to_string(),\n'
        '    ] {\n'
        '        if let Ok(s) = std::fs::read_to_string(&p) {\n'
        '            if !s.is_empty() {\n'
        '                parts.push(s);\n'
        '            }\n'
        '        }\n'
        '    }\n'
        '    if parts.is_empty() {\n'
        '        return r#"{"error":"no_crash_log"}"#.to_string();\n'
        '    }\n'
        '    parts.join("\\n--- next sink ---\\n")',
    ),
    (
        "prewarm_call",
        'fn init_crash_handler() {\n    unsafe {',
        'fn init_crash_handler() {\n    hl_crash_log_init();\n    unsafe {',
    ),
    (
        "boot_trace_fallback",
        '        .unwrap_or_else(|| "/data/data/jp.pokemon.pokeuma/files".to_string());',
        '        .unwrap_or_else(|| hl_crash_log_dir().to_string());',
    ),
]

# Anchors that legitimately appear more than once; anything else must be 1:1.
MULTI_OK = {"openoptions_sinks"}


def apply() -> str:
    text = SOURCE.read_text(encoding="utf-8")
    if MARK in text:
        return "already_applied"

    counts = []
    for label, old, new in REPLACEMENTS:
        found = text.count(old)
        if found == 0:
            raise RuntimeError(f"anchor MISSING ({label}): {old.strip()[:70]!r}")
        if found > 1 and label not in MULTI_OK:
            raise RuntimeError(
                f"anchor ambiguous ({label}) count={found} (expect 1): {old.strip()[:70]!r}"
            )
        text = text.replace(old, new)
        counts.append(f"{label}={found}")

    # Fail closed: no crash sink may still target the package that is not running.
    if 'let path = b"/data/data/jp.pokemon.pokeuma' in text:
        raise RuntimeError("regression: signal handler still writes to the dead path")

    SOURCE.write_text(text, encoding="utf-8")

    cargo = CARGO.read_text(encoding="utf-8")
    if f'version = "{BASE_VERSION}"' not in cargo:
        raise RuntimeError(f"Cargo.toml is not at {BASE_VERSION}, refusing to bump")
    cargo = cargo.replace(
        f'version = "{BASE_VERSION}"', f'version = "{NEW_VERSION}"', 1
    )
    CARGO.write_text(cargo, encoding="utf-8")

    lock = LOCK.read_text(encoding="utf-8")
    pair = f'name = "hachimi_ura"\nversion = "{BASE_VERSION}"'
    if pair not in lock:
        raise RuntimeError(f"Cargo.lock hachimi_ura is not at {BASE_VERSION}")
    lock = lock.replace(
        pair, f'name = "hachimi_ura"\nversion = "{NEW_VERSION}"', 1
    )
    LOCK.write_text(lock, encoding="utf-8")

    return f"applied {' '.join(counts)} version={NEW_VERSION}"


if __name__ == "__main__":
    print(f"crash_log_path={apply()}")
