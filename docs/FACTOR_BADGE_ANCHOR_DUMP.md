# Factor badge anchor dump

- run: 39032660932 (2026-10-17T12:28:39Z)
- source_commit: 828da2e8adf9833223320df5a810de3cae42c5ef

## chain state

- cargo_version=3.28.0
- lib_rs_sha256=9bbedc34768add432f5da562db0496e8f3dec439f89c358b39c579ab44b3f35a
- lib_rs_bytes=1151969
- pokeuma_paths=26
- media_paths=5

## anchor counts (expect column is what the patch script will assert)

| anchor | count |
|---|---|
| `const PLUGIN_VERSION: &str = env!("CARGO_PKG_VERSION");` | 1 |
| `mod training_anim_skip;` | 0 |
| `mod signup_plaintext;` | 0 |
| `install_event_choice_hook();` | 3 |
| `install_api_sniff_hooks();` | 3 |
| `install_training_hook();` | 2 |
| `install_screen_mirror_hook();` | 0 |
| `} else if path == "/api/sniff/clear" {` | 1 |
| `} else if path == "/api/training/anim_skip" {` | 0 |
| `fn install_hook_safe(` | 1 |
| `unsafe fn interceptor_hook(` | 1 |
| `unsafe fn read_il2cpp_string(` | 1 |
| `fn hl_ptr_mapped(` | 1 |
| `unsafe fn read_ptr_at(` | 1 |
| `unsafe fn read_int_at(` | 1 |
| `unsafe fn resolve_il2cpp_symbol(` | 1 |
| `fn set_hook_status(` | 1 |

## BOOT_SAFE_EXACT block (first definition)

```rust
        const BOOT_SAFE_EXACT: &[&str] = &[
    "/il2cpp/exact_method",
    "/runtime/init_status",
    "/hooks/registry",
    "/hooks/diagnostics",
    "/capture/status",
    "/storage/status",
    "/storage/sessions",
    "/storage/session",
    "/storage/files",
    "/storage/download",
    "/storage/turn_event_jsons",
    "/storage/read_range",
    "/storage/flush",
    "/storage/recover",
    "/storage/audit",
    "/api/sniff/exchange",
    "/api/sniff/exchanges",
            "/",
            "/health",
            "/status",
            "/config",
            "/config.html",
            "/update",
            "/update/status",
            "/debug/hookdiag",
            "/debug/hooklog",
            "/debug/crashlog",
            "/debug/upload",
            "/api/sniff",
            "/api/sniff/metadata",
            "/api/sniff/status",
            "/api/sniff/diag",
            "/api/sniff/toggle",
            "/api/sniff/clear",
            "/api/md5log",
            "/api/md5log/clear",
            "/api/frame",
            "/api/frame_toggle",
            "/api/touch",
            "/api/md5log/install",
            "/api/event/choices",
            "/api/event/observations",
            "/api/event/observations/clear",
            "/api/event/clear",
            "/action/latest",
            "/seed/history",
            "/seed/stats",
            "/log",
            "/carddb",
            "/skilldata",
            "/debug/table",
            "/debug/push_table",
            "/debug/download_table",
            "/debug/mdb_all_tables",
            "/debug/mdb_schema_dump",
        ];
```

## install call sites context (install_event_choice_hook)

```text
7114-                install_exec_training_hook();
7115-                install_failure_rate_hook();
7116:                install_event_choice_hook();
7117-                // ★ v3.24.40: sniff hooks were missing here — fallback mode
7118-                // left /api/sniff permanently unhooked.
--
8850-        unsafe {
8851-            if !EVENT_CHOICE_HOOK_INSTALLED || !STORY_SET_HOOK_INSTALLED {
8852:                install_event_choice_hook();
8853-            }
8854-        }
--
12535-        install_failure_rate_hook();
12536-        install_api_sniff_hooks();
12537:        install_event_choice_hook();
12538-        // v3.22.51: Pre-cache all IL2CPP metadata on game thread
12539-        precache_metadata();
```
