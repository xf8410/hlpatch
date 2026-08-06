# `read_summary`

source_commit: `ffc3748df2d3c8c57b34aa3fdd64f75d09ed0866`
source_line: `4534`

```rust
fn read_summary() -> String {
    // ★ v3.22.35: SIGSEGV cooldown — if we recently recovered from a crash, skip reads
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let cooldown = SIGSEGV_COOLDOWN_UNTIL.load(std::sync::atomic::Ordering::Relaxed);
    if now < cooldown {
        return format!(
            r#"{{"error":"sigsegv_cooldown","retry_after":{}}}"#,
            cooldown - now
        );
    }
    // ★ v3.22.51: Check cache first — avoid IL2CPP calls if data hasn't changed
    if let Ok(guard) = CACHED_SUMMARY.lock() {
        if let Some((ref cached, ts)) = *guard {
            if now.saturating_sub(ts) < SUMMARY_CACHE_TTL_SECS {
                return cached.clone();
            }
        }
    }
    // ★ v3.15.2: Mutex lock prevents concurrent il2cpp reads from HTTP + push threads
    let _lock = READ_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    // ★ v3.22.35: sigsetjmp recovery — catch SIGSEGV from il2cpp_runtime_invoke
    // If SIGSEGV fires during read_summary_inner, signal handler will longjmp back here
    let jmp_result = unsafe { sys_sigsetjmp(SIGSEGV_JMP_BUF.as_mut_ptr(), 1) };
    if jmp_result != 0 {
        // We jumped back from SIGSEGV handler — read_summary_inner crashed
        unsafe {
            ura_log(1, "★ SIGSEGV recovered in read_summary — skipping for 60s");
        };
        let err =
            r#"{"error":"sigsegv_recovered","hint":"read_summary hit native crash, cooling down"}"#
                .to_string();
        if let Ok(mut guard) = CACHED_SUMMARY.lock() {
            *guard = Some((err.clone(), now));
        }
        return err;
    }
    // Set recovery flag so signal handler knows to longjmp instead of killing process
    SIGSEGV_RECOVERY.store(true, std::sync::atomic::Ordering::Relaxed);
    let summary = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
        read_summary_inner()
    }))
    .unwrap_or_else(|_| {
        r#"{"error":"panic_caught","hint":"read_summary panicked, game protected"}"#.to_string()
    });
    // Clear recovery flag — normal return, no crash
    SIGSEGV_RECOVERY.store(false, std::sync::atomic::Ordering::Relaxed);
    // v3.24.71: compare only fresh runtime reads (never cache hits).
    observe_ramen_transition(&summary, now);
    // ★ v3.22.51: Update cache
    if let Ok(mut guard) = CACHED_SUMMARY.lock() {
        *guard = Some((summary.clone(), now));
    }
    summary
}
```
