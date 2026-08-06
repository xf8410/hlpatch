# Remaining integration anchors

source_commit: `a340a147acf13672b2fbc64925bfa321d08091fd`

## lines 106-122

```rust
106: });
107: static mut TRAINING_HOOK_INSTALLED: bool = false;
108: static mut ORIG_ON_SUCCESS_PROLOGUE: [u8; 16] = [0; 16];
109: static mut ON_SUCCESS_ADDR: usize = 0;
110: // ★ v3.23.3: API sniffing — use Hachimi Interceptor API (hook+trampoline) + WWWRequest.Post for URL (replaces _Send+SetHeader)
111: static SNIFF_ENABLED: AtomicBool = AtomicBool::new(true);
112: static SNIFF_MUTEX: Mutex<()> = Mutex::new(());
113: // Raw payloads and protocol observations use separate rings.
114: static mut SNIFF_REQUESTS: Vec<(u64, String, String, Vec<u8>)> = Vec::new();
115: static mut SNIFF_RESPONSES: Vec<(u64, Vec<u8>)> = Vec::new();
116: const SNIFF_RAW_MAX: usize = 50;
117: const SNIFF_METADATA_MAX: usize = 1000;
118: static SNIFF_REQ_ID: AtomicU64 = AtomicU64::new(1);
119: static SNIFF_METADATA_ID: AtomicU64 = AtomicU64::new(1);
120: #[derive(Clone)]
121: struct SniffMetadata {
122:     id: u64,
```

## lines 107-123

```rust
107: static mut TRAINING_HOOK_INSTALLED: bool = false;
108: static mut ORIG_ON_SUCCESS_PROLOGUE: [u8; 16] = [0; 16];
109: static mut ON_SUCCESS_ADDR: usize = 0;
110: // ★ v3.23.3: API sniffing — use Hachimi Interceptor API (hook+trampoline) + WWWRequest.Post for URL (replaces _Send+SetHeader)
111: static SNIFF_ENABLED: AtomicBool = AtomicBool::new(true);
112: static SNIFF_MUTEX: Mutex<()> = Mutex::new(());
113: // Raw payloads and protocol observations use separate rings.
114: static mut SNIFF_REQUESTS: Vec<(u64, String, String, Vec<u8>)> = Vec::new();
115: static mut SNIFF_RESPONSES: Vec<(u64, Vec<u8>)> = Vec::new();
116: const SNIFF_RAW_MAX: usize = 50;
117: const SNIFF_METADATA_MAX: usize = 1000;
118: static SNIFF_REQ_ID: AtomicU64 = AtomicU64::new(1);
119: static SNIFF_METADATA_ID: AtomicU64 = AtomicU64::new(1);
120: #[derive(Clone)]
121: struct SniffMetadata {
122:     id: u64,
123:     request_id: u64,
```

## lines 549-565

```rust
549:     fn sys_siglongjmp(env: *const u8, val: i32) -> !;
550: }
551: 
552: const CRASH_LOG_PATH: &str = "/data/data/jp.pokemon.pokeuma/files/uma_predict.log";
553: 
554: // ★ v3.22.35: SIGSEGV recovery for push thread
555: // sigsetjmp buffer: 200 bytes is enough for jmp_buf on aarch64 (typically 24 x 8 = 192 bytes)
556: static mut SIGSEGV_JMP_BUF: [u8; 200] = [0u8; 200];
557: static SIGSEGV_RECOVERY: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
558: // Cooldown: after SIGSEGV recovery, skip reads for N seconds
559: static SIGSEGV_COOLDOWN_UNTIL: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
560: 
561: extern "C" fn crash_signal_handler(sig: i32) {
562:     CRASH_SIG.store(sig, std::sync::atomic::Ordering::Relaxed);
563:     CRASH_STEP.store(
564:         PREDICT_STEP.load(std::sync::atomic::Ordering::Relaxed),
565:         std::sync::atomic::Ordering::Relaxed,
```

## lines 616-632

```rust
616:     let fd = unsafe { sys_open(path.as_ptr() as *const i8, 1 | 64 | 1024, 0o644) };
617:     if fd >= 0 {
618:         unsafe {
619:             sys_write(fd, msg.as_ptr(), len);
620:             sys_close(fd);
621:         }
622:     }
623:     // ★ v3.22.35: If sigsetjmp was set (push thread), longjmp back instead of killing process
624:     if SIGSEGV_RECOVERY.load(std::sync::atomic::Ordering::Relaxed) {
625:         // Set cooldown: skip reads for 60 seconds
626:         let now = std::time::SystemTime::now()
627:             .duration_since(std::time::UNIX_EPOCH)
628:             .unwrap_or_default()
629:             .as_secs();
630:         SIGSEGV_COOLDOWN_UNTIL.store(now + 60, std::sync::atomic::Ordering::Relaxed);
631:         SIGSEGV_RECOVERY.store(false, std::sync::atomic::Ordering::Relaxed);
632:         unsafe {
```

## lines 623-639

```rust
623:     // ★ v3.22.35: If sigsetjmp was set (push thread), longjmp back instead of killing process
624:     if SIGSEGV_RECOVERY.load(std::sync::atomic::Ordering::Relaxed) {
625:         // Set cooldown: skip reads for 60 seconds
626:         let now = std::time::SystemTime::now()
627:             .duration_since(std::time::UNIX_EPOCH)
628:             .unwrap_or_default()
629:             .as_secs();
630:         SIGSEGV_COOLDOWN_UNTIL.store(now + 60, std::sync::atomic::Ordering::Relaxed);
631:         SIGSEGV_RECOVERY.store(false, std::sync::atomic::Ordering::Relaxed);
632:         unsafe {
633:             sys_siglongjmp(SIGSEGV_JMP_BUF.as_ptr(), 1);
634:         }
635:     }
636:     // Not in recovery context — re-raise signal to kill process (unrecoverable)
637:     unsafe {
638:         sys_signal(sig, 0);
639:         sys_raise(sig);
```

## lines 4316-4332

```rust
4316:     (
4317:         total_eval,
4318:         skills.len() as i32,
4319:         format!("[{}]", breakdown.join(",")),
4320:     )
4321: }
4322: 
4323: // ★ v3.22.51: Summary cache — reduce IL2CPP metadata reads
4324: static CACHED_SUMMARY: std::sync::Mutex<Option<(String, u64)>> = std::sync::Mutex::new(None);
4325: const SUMMARY_CACHE_TTL_SECS: u64 = 3;
4326: 
4327: // v3.24.71: compact, observational-only Ramen transition probe.
4328: // This records co-occurring runtime changes; it deliberately does not claim
4329: // that tasting caused a point/member change.
4330: #[derive(Clone)]
4331: struct RamenObservedFrame {
4332:     captured_at: u64,
```

## lines 4540-4556

```rust
4540:     let cooldown = SIGSEGV_COOLDOWN_UNTIL.load(std::sync::atomic::Ordering::Relaxed);
4541:     if now < cooldown {
4542:         return format!(
4543:             r#"{{"error":"sigsegv_cooldown","retry_after":{}}}"#,
4544:             cooldown - now
4545:         );
4546:     }
4547:     // ★ v3.22.51: Check cache first — avoid IL2CPP calls if data hasn't changed
4548:     if let Ok(guard) = CACHED_SUMMARY.lock() {
4549:         if let Some((ref cached, ts)) = *guard {
4550:             if now.saturating_sub(ts) < SUMMARY_CACHE_TTL_SECS {
4551:                 return cached.clone();
4552:             }
4553:         }
4554:     }
4555:     // ★ v3.15.2: Mutex lock prevents concurrent il2cpp reads from HTTP + push threads
4556:     let _lock = READ_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
```

## lines 4560-4576

```rust
4560:     if jmp_result != 0 {
4561:         // We jumped back from SIGSEGV handler — read_summary_inner crashed
4562:         unsafe {
4563:             ura_log(1, "★ SIGSEGV recovered in read_summary — skipping for 60s");
4564:         };
4565:         let err =
4566:             r#"{"error":"sigsegv_recovered","hint":"read_summary hit native crash, cooling down"}"#
4567:                 .to_string();
4568:         if let Ok(mut guard) = CACHED_SUMMARY.lock() {
4569:             *guard = Some((err.clone(), now));
4570:         }
4571:         return err;
4572:     }
4573:     // Set recovery flag so signal handler knows to longjmp instead of killing process
4574:     SIGSEGV_RECOVERY.store(true, std::sync::atomic::Ordering::Relaxed);
4575:     let summary = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
4576:         read_summary_inner()
```

## lines 4566-4582

```rust
4566:             r#"{"error":"sigsegv_recovered","hint":"read_summary hit native crash, cooling down"}"#
4567:                 .to_string();
4568:         if let Ok(mut guard) = CACHED_SUMMARY.lock() {
4569:             *guard = Some((err.clone(), now));
4570:         }
4571:         return err;
4572:     }
4573:     // Set recovery flag so signal handler knows to longjmp instead of killing process
4574:     SIGSEGV_RECOVERY.store(true, std::sync::atomic::Ordering::Relaxed);
4575:     let summary = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
4576:         read_summary_inner()
4577:     }))
4578:     .unwrap_or_else(|_| {
4579:         r#"{"error":"panic_caught","hint":"read_summary panicked, game protected"}"#.to_string()
4580:     });
4581:     // Clear recovery flag — normal return, no crash
4582:     SIGSEGV_RECOVERY.store(false, std::sync::atomic::Ordering::Relaxed);
```

## lines 4574-4590

```rust
4574:     SIGSEGV_RECOVERY.store(true, std::sync::atomic::Ordering::Relaxed);
4575:     let summary = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
4576:         read_summary_inner()
4577:     }))
4578:     .unwrap_or_else(|_| {
4579:         r#"{"error":"panic_caught","hint":"read_summary panicked, game protected"}"#.to_string()
4580:     });
4581:     // Clear recovery flag — normal return, no crash
4582:     SIGSEGV_RECOVERY.store(false, std::sync::atomic::Ordering::Relaxed);
4583:     // v3.24.71: compare only fresh runtime reads (never cache hits).
4584:     observe_ramen_transition(&summary, now);
4585:     // ★ v3.22.51: Update cache
4586:     if let Ok(mut guard) = CACHED_SUMMARY.lock() {
4587:         *guard = Some((summary.clone(), now));
4588:     }
4589:     summary
4590: }
```

## lines 4578-4594

```rust
4578:     .unwrap_or_else(|_| {
4579:         r#"{"error":"panic_caught","hint":"read_summary panicked, game protected"}"#.to_string()
4580:     });
4581:     // Clear recovery flag — normal return, no crash
4582:     SIGSEGV_RECOVERY.store(false, std::sync::atomic::Ordering::Relaxed);
4583:     // v3.24.71: compare only fresh runtime reads (never cache hits).
4584:     observe_ramen_transition(&summary, now);
4585:     // ★ v3.22.51: Update cache
4586:     if let Ok(mut guard) = CACHED_SUMMARY.lock() {
4587:         *guard = Some((summary.clone(), now));
4588:     }
4589:     summary
4590: }
4591: 
4592: unsafe fn read_summary_inner() -> String {
4593:     // v3.22.51: IN_READ_PATH disabled - /debug/ramenfields proves IL2CPP APIs are safe from HTTP thread
4594:     // Keep the wrapper for potential future use, but don't block any APIs
```

## lines 6121-6137

```rust
6121:                 // CharaId is a computed property. The app can resolve it
6122:                 // through support_card_id and card_db.json.
6123:                 let sc_chara_id = -1;
6124: 
6125:                 // Runtime capture confirmed that support-card positions
6126:                 // 1..=6 are also the corresponding partner IDs.
6127:                 let kizuna = partner_evaluation.get(&position).copied().unwrap_or(-1);
6128:                 scs.push(format!(
6129:                     r#"{{"position":{},"support_card_id":{},"limit_break_count":{},"training_partner_state":{},"chara_id":{},"kizuna":{},"exp":{},"rental_type":{}}}"#,
6130:                     position, support_card_id, limit_break_count, training_partner_state, sc_chara_id, kizuna, sc_exp, rental_type
6131:                 ));
6132:             }
6133:             sc_json = format!("[{}]", scs.join(","));
6134:             ura_log(
6135:                 3,
6136:                 &format!(
6137:                     "sc: {} cards found, partner_evaluation: {} entries",
```

## lines 6122-6138

```rust
6122:                 // through support_card_id and card_db.json.
6123:                 let sc_chara_id = -1;
6124: 
6125:                 // Runtime capture confirmed that support-card positions
6126:                 // 1..=6 are also the corresponding partner IDs.
6127:                 let kizuna = partner_evaluation.get(&position).copied().unwrap_or(-1);
6128:                 scs.push(format!(
6129:                     r#"{{"position":{},"support_card_id":{},"limit_break_count":{},"training_partner_state":{},"chara_id":{},"kizuna":{},"exp":{},"rental_type":{}}}"#,
6130:                     position, support_card_id, limit_break_count, training_partner_state, sc_chara_id, kizuna, sc_exp, rental_type
6131:                 ));
6132:             }
6133:             sc_json = format!("[{}]", scs.join(","));
6134:             ura_log(
6135:                 3,
6136:                 &format!(
6137:                     "sc: {} cards found, partner_evaluation: {} entries",
6138:                     scs.len(),
```

## lines 6543-6559

```rust
6543:             )
6544:         } else {
6545:             String::new()
6546:         }
6547:     };
6548: 
6549:     log_predict_step("S:json");
6550:     format!(
6551:         r#"{{"version":"{}","year":{},"turn":{},"raw_total_turn_num":{},"ui_turn_semantics":"countdown","raw_field_mapping":"unverified","month":{},"half":{},"scenario":"{}","chara_id":{},"stats":{{"speed":{},"stamina":{},"power":{},"guts":{},"wiz":{},"vital":{},"max_vital":{},"motivation":"{}","skill_point":{},"fan":{}}},"max_stats":{{"speed":{},"stamina":{},"power":{},"guts":{},"wiz":{}}},"proper":{{"dist_short":{},"dist_mile":{},"dist_mid":{},"dist_long":{},"ground_turf":{},"ground_dirt":{}}},"running_style":{},"scenario_progress":{},"training_event_type":{},"talent_level":{},"chara_grade":{},"difficulty":{},"fixed_turn_chara_seed":{},"trainings":{},"support_cards":{},"evaluation":{},"training_levels":{},"buffs":{},"chara_effect_ids":[{}],"skills":{{"eval":{},"count":{},"list":{}}},"ai":{}{}{}{} }}"#,
6552:         PLUGIN_VERSION,
6553:         year,
6554:         cumulative_turn,
6555:         raw_total_turn_num,
6556:         mon,
6557:         half,
6558:         scn_s,
6559:         chara_id,
```

## lines 6551-6567

```rust
6551:         r#"{{"version":"{}","year":{},"turn":{},"raw_total_turn_num":{},"ui_turn_semantics":"countdown","raw_field_mapping":"unverified","month":{},"half":{},"scenario":"{}","chara_id":{},"stats":{{"speed":{},"stamina":{},"power":{},"guts":{},"wiz":{},"vital":{},"max_vital":{},"motivation":"{}","skill_point":{},"fan":{}}},"max_stats":{{"speed":{},"stamina":{},"power":{},"guts":{},"wiz":{}}},"proper":{{"dist_short":{},"dist_mile":{},"dist_mid":{},"dist_long":{},"ground_turf":{},"ground_dirt":{}}},"running_style":{},"scenario_progress":{},"training_event_type":{},"talent_level":{},"chara_grade":{},"difficulty":{},"fixed_turn_chara_seed":{},"trainings":{},"support_cards":{},"evaluation":{},"training_levels":{},"buffs":{},"chara_effect_ids":[{}],"skills":{{"eval":{},"count":{},"list":{}}},"ai":{}{}{}{} }}"#,
6552:         PLUGIN_VERSION,
6553:         year,
6554:         cumulative_turn,
6555:         raw_total_turn_num,
6556:         mon,
6557:         half,
6558:         scn_s,
6559:         chara_id,
6560:         spd,
6561:         sta,
6562:         pow_,
6563:         gut,
6564:         wiz,
6565:         vit,
6566:         mvit,
6567:         mot_s,
```

## lines 7695-7711

```rust
7695:                 .count();
7696:             let response_count = SNIFF_METADATA
7697:                 .iter()
7698:                 .filter(|m| m.direction == "response")
7699:                 .count();
7700:             format!(
7701:                 r#"{{"enabled":{},"raw_request_count":{},"raw_response_count":{},"metadata_count":{},"request_count":{},"response_count":{},"last_id":{},"raw_limit":{},"metadata_limit":{}}}"#,
7702:                 SNIFF_ENABLED.load(Ordering::Relaxed),
7703:                 SNIFF_REQUESTS.len(),
7704:                 SNIFF_RESPONSES.len(),
7705:                 SNIFF_METADATA.len(),
7706:                 request_count,
7707:                 response_count,
7708:                 last_id,
7709:                 SNIFF_RAW_MAX,
7710:                 SNIFF_METADATA_MAX
7711:             )
```

## lines 7696-7712

```rust
7696:             let response_count = SNIFF_METADATA
7697:                 .iter()
7698:                 .filter(|m| m.direction == "response")
7699:                 .count();
7700:             format!(
7701:                 r#"{{"enabled":{},"raw_request_count":{},"raw_response_count":{},"metadata_count":{},"request_count":{},"response_count":{},"last_id":{},"raw_limit":{},"metadata_limit":{}}}"#,
7702:                 SNIFF_ENABLED.load(Ordering::Relaxed),
7703:                 SNIFF_REQUESTS.len(),
7704:                 SNIFF_RESPONSES.len(),
7705:                 SNIFF_METADATA.len(),
7706:                 request_count,
7707:                 response_count,
7708:                 last_id,
7709:                 SNIFF_RAW_MAX,
7710:                 SNIFF_METADATA_MAX
7711:             )
7712:         }
```

## lines 7768-7784

```rust
7768:         let post_hooked = unsafe { POST_ADDR != 0 };
7769:         format!(
7770:             r#"{{"sniff_enabled":{},"compress_hooked":{},"decompress_hooked":{},"post_hooked":{}}}"#,
7771:             new_val, req_hooked, resp_hooked, post_hooked
7772:         )
7773:     } else if path == "/api/sniff/clear" {
7774:         let _lock = SNIFF_MUTEX.lock();
7775:         unsafe {
7776:             SNIFF_REQUESTS.clear();
7777:             SNIFF_RESPONSES.clear();
7778:             if let Ok(mut entries) = UNITY_OBSERVATIONS.lock() {
7779:                 entries.clear();
7780:             }
7781:             SNIFF_METADATA.clear();
7782:             SNIFF_RESPONSE_QUEUE.clear();
7783:             PENDING_REQ_BODY = None;
7784:         }
```

## lines 7769-7785

```rust
7769:         format!(
7770:             r#"{{"sniff_enabled":{},"compress_hooked":{},"decompress_hooked":{},"post_hooked":{}}}"#,
7771:             new_val, req_hooked, resp_hooked, post_hooked
7772:         )
7773:     } else if path == "/api/sniff/clear" {
7774:         let _lock = SNIFF_MUTEX.lock();
7775:         unsafe {
7776:             SNIFF_REQUESTS.clear();
7777:             SNIFF_RESPONSES.clear();
7778:             if let Ok(mut entries) = UNITY_OBSERVATIONS.lock() {
7779:                 entries.clear();
7780:             }
7781:             SNIFF_METADATA.clear();
7782:             SNIFF_RESPONSE_QUEUE.clear();
7783:             PENDING_REQ_BODY = None;
7784:         }
7785:         r#"{"ok":true}"#.to_string()
```

## lines 8325-8341

```rust
8325:                     r#"{"ok":false,"error":"interceptor_hook_failed"}"#.to_string()
8326:                 }
8327:             }
8328:         }
8329:         })()
8330:     } else if path == "/api/sniff" {
8331:         let _lock = SNIFF_MUTEX.lock();
8332:         unsafe {
8333:             let reqs: Vec<String> = SNIFF_REQUESTS
8334:                 .iter()
8335:                 .map(|(rid, url, headers, data)| {
8336:                     let preview = String::from_utf8_lossy(&data[..data.len().min(2048)]);
8337:                     let preview = preview
8338:                         .replace('\\', "\\\\")
8339:                         .replace('"', "\\\"")
8340:                         .replace('\n', "\\n")
8341:                         .replace('\r', "");
```

## lines 8346-8362

```rust
8346:                         url_escaped,
8347:                         headers,
8348:                         data.len(),
8349:                         hex_encode(&data[..data.len().min(256)]),
8350:                         preview
8351:                     )
8352:                 })
8353:                 .collect();
8354:             let resps: Vec<String> = SNIFF_RESPONSES
8355:                 .iter()
8356:                 .map(|(rid, data)| {
8357:                     let preview = String::from_utf8_lossy(&data[..data.len().min(2048)]);
8358:                     let preview = preview
8359:                         .replace('\\', "\\\\")
8360:                         .replace('"', "\\\"")
8361:                         .replace('\n', "\\n")
8362:                         .replace('\r', "");
```

## lines 8387-8403

```rust
8387:         let _lock = EVENT_STATE_MUTEX.lock();
8388:         unsafe {
8389:             let choices_json: Vec<String> = EVENT_CHOICES.iter().map(|c| {
8390:                 format!(r#"{{"label":"{}","gain_id":{},"next_block_idx":{},"loop_exit_gain_id":{}}}"#,
8391:                     c.label.replace('\\', "\\\\").replace('"', "\\\""),
8392:                     c.gain_id, c.next_block_idx, c.loop_exit_gain_id)
8393:             }).collect();
8394:             let result = format!(
8395:                 r#"{{"generation":{},"story_id":{},"chara_id":{},"selected_idx":{},"choices":[{}]}}"#,
8396:                 EVENT_GENERATION,
8397:                 EVENT_STORY_ID,
8398:                 EVENT_CHARA_ID,
8399:                 EVENT_SELECTED_IDX,
8400:                 choices_json.join(",")
8401:             );
8402:             drop(_lock);
8403:             result
```

## lines 8533-8549

```rust
8533:         .unwrap_or_else(|_| r#"{"error":"ramenfields_panic"}"#.to_string())
8534:     } else if path == "/debug/gauge" {
8535:         // ★ v3.22.39: sigsetjmp + READ_MUTEX protection — prevent game crash on SIGSEGV
8536:         let _lock = READ_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
8537:         let jmp_result = unsafe { sys_sigsetjmp(SIGSEGV_JMP_BUF.as_mut_ptr(), 1) };
8538:         if jmp_result != 0 {
8539:             r#"{"error":"sigsegv_recovered","hint":"/debug/gauge hit native crash, game protected"}"#.to_string()
8540:         } else {
8541:             SIGSEGV_RECOVERY.store(true, std::sync::atomic::Ordering::Relaxed);
8542:             let result =
8543:                 std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe { debug_gauge() }))
8544:                     .unwrap_or_else(|_| r#"{"error":"gauge_panic"}"#.to_string());
8545:             SIGSEGV_RECOVERY.store(false, std::sync::atomic::Ordering::Relaxed);
8546:             result
8547:         }
8548:     } else if path == "/debug/gauge2" {
8549:         // v3.22.39: Scan all DataSet array fields for element class names
```

## lines 8537-8553

```rust
8537:         let jmp_result = unsafe { sys_sigsetjmp(SIGSEGV_JMP_BUF.as_mut_ptr(), 1) };
8538:         if jmp_result != 0 {
8539:             r#"{"error":"sigsegv_recovered","hint":"/debug/gauge hit native crash, game protected"}"#.to_string()
8540:         } else {
8541:             SIGSEGV_RECOVERY.store(true, std::sync::atomic::Ordering::Relaxed);
8542:             let result =
8543:                 std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe { debug_gauge() }))
8544:                     .unwrap_or_else(|_| r#"{"error":"gauge_panic"}"#.to_string());
8545:             SIGSEGV_RECOVERY.store(false, std::sync::atomic::Ordering::Relaxed);
8546:             result
8547:         }
8548:     } else if path == "/debug/gauge2" {
8549:         // v3.22.39: Scan all DataSet array fields for element class names
8550:         let _lock = READ_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
8551:         let jmp_result = unsafe { sys_sigsetjmp(SIGSEGV_JMP_BUF.as_mut_ptr(), 1) };
8552:         if jmp_result != 0 {
8553:             r#"{"error":"sigsegv_recovered","hint":"/debug/gauge2 hit native crash, game protected"}"#.to_string()
```

## lines 8547-8563

```rust
8547:         }
8548:     } else if path == "/debug/gauge2" {
8549:         // v3.22.39: Scan all DataSet array fields for element class names
8550:         let _lock = READ_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
8551:         let jmp_result = unsafe { sys_sigsetjmp(SIGSEGV_JMP_BUF.as_mut_ptr(), 1) };
8552:         if jmp_result != 0 {
8553:             r#"{"error":"sigsegv_recovered","hint":"/debug/gauge2 hit native crash, game protected"}"#.to_string()
8554:         } else {
8555:             SIGSEGV_RECOVERY.store(true, std::sync::atomic::Ordering::Relaxed);
8556:             let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
8557:                 debug_gauge2()
8558:             }))
8559:             .unwrap_or_else(|_| r#"{"error":"gauge2_panic"}"#.to_string());
8560:             SIGSEGV_RECOVERY.store(false, std::sync::atomic::Ordering::Relaxed);
8561:             result
8562:         }
8563:     } else if path == "/debug/ramengains" {
```

## lines 8552-8568

```rust
8552:         if jmp_result != 0 {
8553:             r#"{"error":"sigsegv_recovered","hint":"/debug/gauge2 hit native crash, game protected"}"#.to_string()
8554:         } else {
8555:             SIGSEGV_RECOVERY.store(true, std::sync::atomic::Ordering::Relaxed);
8556:             let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
8557:                 debug_gauge2()
8558:             }))
8559:             .unwrap_or_else(|_| r#"{"error":"gauge2_panic"}"#.to_string());
8560:             SIGSEGV_RECOVERY.store(false, std::sync::atomic::Ordering::Relaxed);
8561:             result
8562:         }
8563:     } else if path == "/debug/ramengains" {
8564:         // ★ v3.24.9: Diagnose Ramen gains reading — trace every step
8565:         let _lock = READ_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
8566:         let jmp_result = unsafe { sys_sigsetjmp(SIGSEGV_JMP_BUF.as_mut_ptr(), 1) };
8567:         if jmp_result != 0 {
8568:             r#"{"error":"sigsegv_recovered"}"#.to_string()
```

## lines 8562-8578

```rust
8562:         }
8563:     } else if path == "/debug/ramengains" {
8564:         // ★ v3.24.9: Diagnose Ramen gains reading — trace every step
8565:         let _lock = READ_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
8566:         let jmp_result = unsafe { sys_sigsetjmp(SIGSEGV_JMP_BUF.as_mut_ptr(), 1) };
8567:         if jmp_result != 0 {
8568:             r#"{"error":"sigsegv_recovered"}"#.to_string()
8569:         } else {
8570:             SIGSEGV_RECOVERY.store(true, std::sync::atomic::Ordering::Relaxed);
8571:             let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
8572:                 debug_ramengains()
8573:             }))
8574:             .unwrap_or_else(|_| r#"{"error":"ramengains_panic"}"#.to_string());
8575:             SIGSEGV_RECOVERY.store(false, std::sync::atomic::Ordering::Relaxed);
8576:             result
8577:         }
8578:     } else if path == "/debug/paramsincdec" {
```

## lines 8567-8583

```rust
8567:         if jmp_result != 0 {
8568:             r#"{"error":"sigsegv_recovered"}"#.to_string()
8569:         } else {
8570:             SIGSEGV_RECOVERY.store(true, std::sync::atomic::Ordering::Relaxed);
8571:             let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
8572:                 debug_ramengains()
8573:             }))
8574:             .unwrap_or_else(|_| r#"{"error":"ramengains_panic"}"#.to_string());
8575:             SIGSEGV_RECOVERY.store(false, std::sync::atomic::Ordering::Relaxed);
8576:             result
8577:         }
8578:     } else if path == "/debug/paramsincdec" {
8579:         // v3.22.40: Read DataSet CommandInfo ParamsIncDecInfoArray element class names
8580:         let _lock = READ_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
8581:         let jmp_result = unsafe { sys_sigsetjmp(SIGSEGV_JMP_BUF.as_mut_ptr(), 1) };
8582:         if jmp_result != 0 {
8583:             r#"{"error":"sigsegv_recovered","hint":"/debug/paramsincdec hit native crash, game protected"}"#.to_string()
```

## lines 8577-8593

```rust
8577:         }
8578:     } else if path == "/debug/paramsincdec" {
8579:         // v3.22.40: Read DataSet CommandInfo ParamsIncDecInfoArray element class names
8580:         let _lock = READ_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
8581:         let jmp_result = unsafe { sys_sigsetjmp(SIGSEGV_JMP_BUF.as_mut_ptr(), 1) };
8582:         if jmp_result != 0 {
8583:             r#"{"error":"sigsegv_recovered","hint":"/debug/paramsincdec hit native crash, game protected"}"#.to_string()
8584:         } else {
8585:             SIGSEGV_RECOVERY.store(true, std::sync::atomic::Ordering::Relaxed);
8586:             let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
8587:                 debug_paramsincdec()
8588:             }))
8589:             .unwrap_or_else(|_| r#"{"error":"paramsincdec_panic"}"#.to_string());
8590:             SIGSEGV_RECOVERY.store(false, std::sync::atomic::Ordering::Relaxed);
8591:             result
8592:         }
8593:     } else if path == "/debug/training_seed" {
```

## lines 8582-8598

```rust
8582:         if jmp_result != 0 {
8583:             r#"{"error":"sigsegv_recovered","hint":"/debug/paramsincdec hit native crash, game protected"}"#.to_string()
8584:         } else {
8585:             SIGSEGV_RECOVERY.store(true, std::sync::atomic::Ordering::Relaxed);
8586:             let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
8587:                 debug_paramsincdec()
8588:             }))
8589:             .unwrap_or_else(|_| r#"{"error":"paramsincdec_panic"}"#.to_string());
8590:             SIGSEGV_RECOVERY.store(false, std::sync::atomic::Ordering::Relaxed);
8591:             result
8592:         }
8593:     } else if path == "/debug/training_seed" {
8594:         // 一键查找训练种子：自动完成 WorkDataManager → WorkSingleModeData → _fixedTurnCharaSeed
8595:         let _lock = READ_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
8596:         let jmp_result = unsafe { sys_sigsetjmp(SIGSEGV_JMP_BUF.as_mut_ptr(), 1) };
8597:         if jmp_result != 0 {
8598:             r#"{"error":"sigsegv_recovered","hint":"/debug/training_seed hit native crash, game protected"}"#.to_string()
```

## lines 8592-8608

```rust
8592:         }
8593:     } else if path == "/debug/training_seed" {
8594:         // 一键查找训练种子：自动完成 WorkDataManager → WorkSingleModeData → _fixedTurnCharaSeed
8595:         let _lock = READ_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
8596:         let jmp_result = unsafe { sys_sigsetjmp(SIGSEGV_JMP_BUF.as_mut_ptr(), 1) };
8597:         if jmp_result != 0 {
8598:             r#"{"error":"sigsegv_recovered","hint":"/debug/training_seed hit native crash, game protected"}"#.to_string()
8599:         } else {
8600:             SIGSEGV_RECOVERY.store(true, std::sync::atomic::Ordering::Relaxed);
8601:             let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
8602:                 debug_training_seed()
8603:             }))
8604:             .unwrap_or_else(|_| r#"{"error":"training_seed_panic"}"#.to_string());
8605:             SIGSEGV_RECOVERY.store(false, std::sync::atomic::Ordering::Relaxed);
8606:             result
8607:         }
8608:     } else if path == "/update" {
```

## lines 8597-8613

```rust
8597:         if jmp_result != 0 {
8598:             r#"{"error":"sigsegv_recovered","hint":"/debug/training_seed hit native crash, game protected"}"#.to_string()
8599:         } else {
8600:             SIGSEGV_RECOVERY.store(true, std::sync::atomic::Ordering::Relaxed);
8601:             let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
8602:                 debug_training_seed()
8603:             }))
8604:             .unwrap_or_else(|_| r#"{"error":"training_seed_panic"}"#.to_string());
8605:             SIGSEGV_RECOVERY.store(false, std::sync::atomic::Ordering::Relaxed);
8606:             result
8607:         }
8608:     } else if path == "/update" {
8609:         // v3.22.51: Self-update SO from GitHub Release
8610:         update_so()
8611:     } else if path == "/update/status" {
8612:         // v3.22.51: Return auto-update status
8613:         match AUTO_UPDATE_STATUS.lock() {
```

## lines 10096-10112

```rust
10096:                     let preview_len = bytes.len().min(EVENT_RESPONSE_PREVIEW_MAX);
10097:                     let preview = String::from_utf8_lossy(&bytes[..preview_len]);
10098:                     let (label, gain_id, next_block_idx, loop_exit_gain_id) = match sel.choice {
10099:                         Some(c) => (c.label, c.gain_id, c.next_block_idx, c.loop_exit_gain_id),
10100:                         None => (String::new(), -1, -1, -1),
10101:                     };
10102:                     let observation_id = EVENT_OBSERVATION_ID.fetch_add(1, Ordering::Relaxed);
10103:                     let record = format!(
10104:                         r#"{{"schema_version":2,"observation_id":{},"source":"runtime_observation","causality":"unknown","result_label":"unknown","captured_at":{},"generation":{},"story_id":{},"chara_id":{},"selected_idx_raw":{},"choice":{{"label":"{}","gain_id":{},"next_block_idx":{},"loop_exit_gain_id":{}}},"response":{{"request_id":{},"url":"{}","size_captured":{},"preview_truncated":{},"hex_prefix":"{}","text_preview":"{}"}}}}"#,
10105:                         observation_id,
10106:                         sel.captured_at,
10107:                         sel.generation,
10108:                         sel.story_id,
10109:                         sel.chara_id,
10110:                         sel.selected_idx_raw,
10111:                         json_escape(&label),
10112:                         gain_id,
```

## lines 10101-10117

```rust
10101:                     };
10102:                     let observation_id = EVENT_OBSERVATION_ID.fetch_add(1, Ordering::Relaxed);
10103:                     let record = format!(
10104:                         r#"{{"schema_version":2,"observation_id":{},"source":"runtime_observation","causality":"unknown","result_label":"unknown","captured_at":{},"generation":{},"story_id":{},"chara_id":{},"selected_idx_raw":{},"choice":{{"label":"{}","gain_id":{},"next_block_idx":{},"loop_exit_gain_id":{}}},"response":{{"request_id":{},"url":"{}","size_captured":{},"preview_truncated":{},"hex_prefix":"{}","text_preview":"{}"}}}}"#,
10105:                         observation_id,
10106:                         sel.captured_at,
10107:                         sel.generation,
10108:                         sel.story_id,
10109:                         sel.chara_id,
10110:                         sel.selected_idx_raw,
10111:                         json_escape(&label),
10112:                         gain_id,
10113:                         next_block_idx,
10114:                         loop_exit_gain_id,
10115:                         PENDING_REQ_ID,
10116:                         json_escape(&PENDING_URL),
10117:                         bytes.len(),
```

## lines 10132-10148

```rust
10132:             if !bytes.is_empty() {
10133:                 let _lock = SNIFF_MUTEX.lock();
10134:                 let (rid, response_url) = if SNIFF_RESPONSE_QUEUE.is_empty() {
10135:                     (0, String::new())
10136:                 } else {
10137:                     SNIFF_RESPONSE_QUEUE.remove(0)
10138:                 };
10139:                 push_sniff_metadata(rid, "response", &response_url, bytes.len(), &bytes, Vec::new());
10140:                 SNIFF_RESPONSES.push((rid, bytes));
10141:                 if SNIFF_RESPONSES.len() > SNIFF_RAW_MAX {
10142:                     SNIFF_RESPONSES.remove(0);
10143:                 }
10144:             }
10145:         }
10146:         decompressed
10147:     }
10148: }
```

## lines 10133-10149

```rust
10133:                 let _lock = SNIFF_MUTEX.lock();
10134:                 let (rid, response_url) = if SNIFF_RESPONSE_QUEUE.is_empty() {
10135:                     (0, String::new())
10136:                 } else {
10137:                     SNIFF_RESPONSE_QUEUE.remove(0)
10138:                 };
10139:                 push_sniff_metadata(rid, "response", &response_url, bytes.len(), &bytes, Vec::new());
10140:                 SNIFF_RESPONSES.push((rid, bytes));
10141:                 if SNIFF_RESPONSES.len() > SNIFF_RAW_MAX {
10142:                     SNIFF_RESPONSES.remove(0);
10143:                 }
10144:             }
10145:         }
10146:         decompressed
10147:     }
10148: }
10149: 
```

## lines 10134-10150

```rust
10134:                 let (rid, response_url) = if SNIFF_RESPONSE_QUEUE.is_empty() {
10135:                     (0, String::new())
10136:                 } else {
10137:                     SNIFF_RESPONSE_QUEUE.remove(0)
10138:                 };
10139:                 push_sniff_metadata(rid, "response", &response_url, bytes.len(), &bytes, Vec::new());
10140:                 SNIFF_RESPONSES.push((rid, bytes));
10141:                 if SNIFF_RESPONSES.len() > SNIFF_RAW_MAX {
10142:                     SNIFF_RESPONSES.remove(0);
10143:                 }
10144:             }
10145:         }
10146:         decompressed
10147:     }
10148: }
10149: 
10150: // ★ v3.23.3: Hook handler for WWWRequest.Post(this, url, postData, headers)
```

## lines 10192-10208

```rust
10192:             let url_str = game_url.clone().unwrap_or_default();
10193:             {
10194:                 let _lock = SNIFF_MUTEX.lock();
10195:                 push_sniff_metadata(rid, "request", &url_str, body.len(), &body, req_headers.clone());
10196:                 SNIFF_RESPONSE_QUEUE.push((rid, url_str.clone()));
10197:                 if SNIFF_RESPONSE_QUEUE.len() > SNIFF_METADATA_MAX {
10198:                     SNIFF_RESPONSE_QUEUE.remove(0);
10199:                 }
10200:                 SNIFF_REQUESTS.push((rid, url_str, headers_json, body));
10201:                 if SNIFF_REQUESTS.len() > SNIFF_RAW_MAX {
10202:                     SNIFF_REQUESTS.remove(0);
10203:                 }
10204:             }
10205:             PENDING_URL = game_url.clone().unwrap_or_default();
10206:             PENDING_HEADERS = req_headers.clone();
10207:         }
10208: 
```

## lines 10193-10209

```rust
10193:             {
10194:                 let _lock = SNIFF_MUTEX.lock();
10195:                 push_sniff_metadata(rid, "request", &url_str, body.len(), &body, req_headers.clone());
10196:                 SNIFF_RESPONSE_QUEUE.push((rid, url_str.clone()));
10197:                 if SNIFF_RESPONSE_QUEUE.len() > SNIFF_METADATA_MAX {
10198:                     SNIFF_RESPONSE_QUEUE.remove(0);
10199:                 }
10200:                 SNIFF_REQUESTS.push((rid, url_str, headers_json, body));
10201:                 if SNIFF_REQUESTS.len() > SNIFF_RAW_MAX {
10202:                     SNIFF_REQUESTS.remove(0);
10203:                 }
10204:             }
10205:             PENDING_URL = game_url.clone().unwrap_or_default();
10206:             PENDING_HEADERS = req_headers.clone();
10207:         }
10208: 
10209:         let _ = this;
```

## lines 10194-10210

```rust
10194:                 let _lock = SNIFF_MUTEX.lock();
10195:                 push_sniff_metadata(rid, "request", &url_str, body.len(), &body, req_headers.clone());
10196:                 SNIFF_RESPONSE_QUEUE.push((rid, url_str.clone()));
10197:                 if SNIFF_RESPONSE_QUEUE.len() > SNIFF_METADATA_MAX {
10198:                     SNIFF_RESPONSE_QUEUE.remove(0);
10199:                 }
10200:                 SNIFF_REQUESTS.push((rid, url_str, headers_json, body));
10201:                 if SNIFF_REQUESTS.len() > SNIFF_RAW_MAX {
10202:                     SNIFF_REQUESTS.remove(0);
10203:                 }
10204:             }
10205:             PENDING_URL = game_url.clone().unwrap_or_default();
10206:             PENDING_HEADERS = req_headers.clone();
10207:         }
10208: 
10209:         let _ = this;
10210:         original(this, url, post_data, headers)
```

## lines 12876-12892

```rust
12876:         // v3.15.8: removed gauge>=3 burst fallback (gauge not available on TeamMemberInfo)
12877: 
12878:         if level < min_level && level >= 0 {
12879:             min_level = level;
12880:         }
12881: 
12882:         if found_data {
12883:             members_json.push(format!(
12884:                 r#"{{"chara_id":{},"level":{},"dream_gauge":{},"burst_ready":{},"exp":{}}}"#,
12885:                 chara_id, level, gauge, burst_ready, exp
12886:             ));
12887:         } else {
12888:             // Fallback: include raw hex dump + discovered class name for analysis
12889:             members_json.push(format!(
12890:                 r#"{{"idx":{},"chara_id":{},"level":{},"gauge":{},"burst_ready":{},"exp":{},"klass_name":"{}","raw":"{}"}}"#,
12891:                 i, chara_id, level, gauge, burst_ready, exp,
12892:                 discovered_member_class_name, hex
```

## lines 12877-12893

```rust
12877: 
12878:         if level < min_level && level >= 0 {
12879:             min_level = level;
12880:         }
12881: 
12882:         if found_data {
12883:             members_json.push(format!(
12884:                 r#"{{"chara_id":{},"level":{},"dream_gauge":{},"burst_ready":{},"exp":{}}}"#,
12885:                 chara_id, level, gauge, burst_ready, exp
12886:             ));
12887:         } else {
12888:             // Fallback: include raw hex dump + discovered class name for analysis
12889:             members_json.push(format!(
12890:                 r#"{{"idx":{},"chara_id":{},"level":{},"gauge":{},"burst_ready":{},"exp":{},"klass_name":"{}","raw":"{}"}}"#,
12891:                 i, chara_id, level, gauge, burst_ready, exp,
12892:                 discovered_member_class_name, hex
12893:             ));
```

## lines 12882-12898

```rust
12882:         if found_data {
12883:             members_json.push(format!(
12884:                 r#"{{"chara_id":{},"level":{},"dream_gauge":{},"burst_ready":{},"exp":{}}}"#,
12885:                 chara_id, level, gauge, burst_ready, exp
12886:             ));
12887:         } else {
12888:             // Fallback: include raw hex dump + discovered class name for analysis
12889:             members_json.push(format!(
12890:                 r#"{{"idx":{},"chara_id":{},"level":{},"gauge":{},"burst_ready":{},"exp":{},"klass_name":"{}","raw":"{}"}}"#,
12891:                 i, chara_id, level, gauge, burst_ready, exp,
12892:                 discovered_member_class_name, hex
12893:             ));
12894:         }
12895:     }
12896: 
12897:     if min_level == 999 {
12898:         min_level = 0;
```

## lines 12883-12899

```rust
12883:             members_json.push(format!(
12884:                 r#"{{"chara_id":{},"level":{},"dream_gauge":{},"burst_ready":{},"exp":{}}}"#,
12885:                 chara_id, level, gauge, burst_ready, exp
12886:             ));
12887:         } else {
12888:             // Fallback: include raw hex dump + discovered class name for analysis
12889:             members_json.push(format!(
12890:                 r#"{{"idx":{},"chara_id":{},"level":{},"gauge":{},"burst_ready":{},"exp":{},"klass_name":"{}","raw":"{}"}}"#,
12891:                 i, chara_id, level, gauge, burst_ready, exp,
12892:                 discovered_member_class_name, hex
12893:             ));
12894:         }
12895:     }
12896: 
12897:     if min_level == 999 {
12898:         min_level = 0;
12899:     }
```

## lines 14597-14613

```rust
14597:     let conn = match Connection::open_with_flags(&mdb_path, OpenFlags::SQLITE_OPEN_READ_ONLY) {
14598:         Ok(c) => c,
14599:         Err(e) => return format!(r#"{{"error":"open_failed","detail":"{}"}}"#, e),
14600:     };
14601: 
14602:     // 1. Get all support_card_data rows with unique_effect_id > 0
14603:     let mut cards: Vec<String> = Vec::new();
14604:     if let Ok(mut stmt) = conn.prepare(
14605:         "SELECT id, chara_id, rarity, command_id, unique_effect_id, support_card_type \
14606:          FROM support_card_data WHERE unique_effect_id > 0 ORDER BY id",
14607:     ) {
14608:         cards = stmt
14609:             .query_map([], |row| {
14610:                 let id: i64 = row.get(0).unwrap_or(0);
14611:                 let cid: i64 = row.get(1).unwrap_or(0);
14612:                 let rar: i64 = row.get(2).unwrap_or(0);
14613:                 let cmd: i64 = row.get(3).unwrap_or(0);
```

## lines 15041-15057

```rust
15041:             csid,
15042:             entries.join(",")
15043:         ));
15044:     }
15045: 
15046:     // 5. Also check single_mode_unique_chara which links partner_id to unique skills
15047:     let mut unique_chara: Vec<String> = Vec::new();
15048:     if let Ok(mut stmt) = conn.prepare(
15049:         "SELECT id, partner_id, scenario_id, chara_id, period, training_placement, gain_flag_id, is_support_featured_stock, gain_role_id FROM single_mode_unique_chara LIMIT 10"
15050:     ) {
15051:         unique_chara = stmt.query_map([], |row| {
15052:             let id: i64 = row.get(0).unwrap_or(0);
15053:             let pid: i64 = row.get(1).unwrap_or(0);
15054:             let ssid: i64 = row.get(2).unwrap_or(0);
15055:             let cid: i64 = row.get(3).unwrap_or(0);
15056:             let per: i64 = row.get(4).unwrap_or(0);
15057:             let tp: i64 = row.get(5).unwrap_or(0);
```

## lines 15982-15998

```rust
15982: 
15983:     let conn = match Connection::open_with_flags(&mdb_path, OpenFlags::SQLITE_OPEN_READ_ONLY) {
15984:         Ok(c) => c,
15985:         Err(e) => return format!(r#"{{"error":"open_failed","detail":"{}"}}"#, e),
15986:     };
15987: 
15988:     // Collect all card data (consumes iterator, releases borrow)
15989:     let cards: Vec<String> = match conn.prepare(
15990:         "SELECT id, chara_id, rarity, command_id, effect_table_id, unique_effect_id, support_card_type, outing_max FROM support_card_data ORDER BY id"
15991:     ) {
15992:         Ok(mut stmt) => stmt.query_map([], |row| {
15993:             Ok(format!(
15994:                 r#"{{"id":{},"chara_id":{},"rarity":{},"command_id":{},"effect_table_id":{},"unique_effect_id":{},"support_card_type":{},"outing_max":{}}}"#,
15995:                 row.get::<_, i32>(0).unwrap_or(0),
15996:                 row.get::<_, i32>(1).unwrap_or(0),
15997:                 row.get::<_, i32>(2).unwrap_or(0),
15998:                 row.get::<_, i32>(3).unwrap_or(0),
```

## lines 15986-16002

```rust
15986:     };
15987: 
15988:     // Collect all card data (consumes iterator, releases borrow)
15989:     let cards: Vec<String> = match conn.prepare(
15990:         "SELECT id, chara_id, rarity, command_id, effect_table_id, unique_effect_id, support_card_type, outing_max FROM support_card_data ORDER BY id"
15991:     ) {
15992:         Ok(mut stmt) => stmt.query_map([], |row| {
15993:             Ok(format!(
15994:                 r#"{{"id":{},"chara_id":{},"rarity":{},"command_id":{},"effect_table_id":{},"unique_effect_id":{},"support_card_type":{},"outing_max":{}}}"#,
15995:                 row.get::<_, i32>(0).unwrap_or(0),
15996:                 row.get::<_, i32>(1).unwrap_or(0),
15997:                 row.get::<_, i32>(2).unwrap_or(0),
15998:                 row.get::<_, i32>(3).unwrap_or(0),
15999:                 row.get::<_, i32>(4).unwrap_or(0),
16000:                 row.get::<_, i32>(5).unwrap_or(0),
16001:                 row.get::<_, i32>(6).unwrap_or(0),
16002:                 row.get::<_, i32>(7).unwrap_or(0),
```

## lines 16170-16186

```rust
16170:                 row.get::<_, i32>(13).unwrap_or(0),
16171:             ))
16172:         }).unwrap().filter_map(|r| r.ok()).collect(),
16173:         Err(e) => return format!(r#"{{"error":"saddle_prepare_failed","detail":"{}"}}"#, e),
16174:     };
16175: 
16176:     // Collect chara_program (which chara runs which program_group)
16177:     let chara_programs: Vec<String> = match conn.prepare(
16178:         "SELECT chara_id, program_group, program_group_2 FROM single_mode_chara_program ORDER BY program_group, chara_id"
16179:     ) {
16180:         Ok(mut stmt) => stmt.query_map([], |row| {
16181:             Ok(format!(
16182:                 r#"{{"chara_id":{},"program_group":{},"program_group_2":{}}}"#,
16183:                 row.get::<_, i32>(0).unwrap_or(0),
16184:                 row.get::<_, i32>(1).unwrap_or(0),
16185:                 row.get::<_, i32>(2).unwrap_or(0),
16186:             ))
```

## lines 16174-16190

```rust
16174:     };
16175: 
16176:     // Collect chara_program (which chara runs which program_group)
16177:     let chara_programs: Vec<String> = match conn.prepare(
16178:         "SELECT chara_id, program_group, program_group_2 FROM single_mode_chara_program ORDER BY program_group, chara_id"
16179:     ) {
16180:         Ok(mut stmt) => stmt.query_map([], |row| {
16181:             Ok(format!(
16182:                 r#"{{"chara_id":{},"program_group":{},"program_group_2":{}}}"#,
16183:                 row.get::<_, i32>(0).unwrap_or(0),
16184:                 row.get::<_, i32>(1).unwrap_or(0),
16185:                 row.get::<_, i32>(2).unwrap_or(0),
16186:             ))
16187:         }).unwrap().filter_map(|r| r.ok()).collect(),
16188:         Err(e) => return format!(r#"{{"error":"program_prepare_failed","detail":"{}"}}"#, e),
16189:     };
16190: 
```

## lines 16235-16251

```rust
16235:     )) {
16236:         Ok(mut stmt) => stmt
16237:             .query_map([], |row| {
16238:                 let text: String = row
16239:                     .get::<_, Option<String>>(1)
16240:                     .unwrap_or(None)
16241:                     .unwrap_or_default();
16242:                 Ok(format!(
16243:                     r#"{{"chara_id":{},"name":"{}"}}"#,
16244:                     row.get::<_, i32>(0).unwrap_or(0),
16245:                     json_escape(&text),
16246:                 ))
16247:             })
16248:             .unwrap()
16249:             .filter_map(|r| r.ok())
16250:             .collect(),
16251:         Err(e) => {
```

## lines 16271-16287

```rust
16271:     };
16272: 
16273:     // Collect succession_relation_member
16274:     let relation_members: Vec<String> = match conn.prepare(
16275:         "SELECT id, relation_type, chara_id FROM succession_relation_member ORDER BY relation_type, chara_id"
16276:     ) {
16277:         Ok(mut stmt) => stmt.query_map([], |row| {
16278:             Ok(format!(
16279:                 r#"{{"id":{},"relation_type":{},"chara_id":{}}}"#,
16280:                 row.get::<_, i32>(0).unwrap_or(0),
16281:                 row.get::<_, i32>(1).unwrap_or(0),
16282:                 row.get::<_, i32>(2).unwrap_or(0),
16283:             ))
16284:         }).unwrap().filter_map(|r| r.ok()).collect(),
16285:         Err(e) => return format!(r#"{{"error":"member_prepare_failed","detail":"{}"}}"#, e),
16286:     };
16287: 
```

## lines 19210-19226

```rust
19210:         command_json.join(",")
19211:     )
19212: }
19213: 
19214: fn debug_ramen_participants() -> String {
19215:     let _lock = READ_MUTEX.lock().unwrap_or_else(|error| error.into_inner());
19216:     let jump_result = unsafe { sys_sigsetjmp(SIGSEGV_JMP_BUF.as_mut_ptr(), 1) };
19217:     if jump_result != 0 {
19218:         SIGSEGV_RECOVERY.store(false, Ordering::Relaxed);
19219:         return r#"{"error":"sigsegv_recovered"}"#.to_string();
19220:     }
19221:     SIGSEGV_RECOVERY.store(true, Ordering::Relaxed);
19222:     let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
19223:         debug_ramen_participants_inner()
19224:     }))
19225:     .unwrap_or_else(|_| r#"{"error":"panic_caught"}"#.to_string());
19226:     SIGSEGV_RECOVERY.store(false, Ordering::Relaxed);
```

## lines 19213-19229

```rust
19213: 
19214: fn debug_ramen_participants() -> String {
19215:     let _lock = READ_MUTEX.lock().unwrap_or_else(|error| error.into_inner());
19216:     let jump_result = unsafe { sys_sigsetjmp(SIGSEGV_JMP_BUF.as_mut_ptr(), 1) };
19217:     if jump_result != 0 {
19218:         SIGSEGV_RECOVERY.store(false, Ordering::Relaxed);
19219:         return r#"{"error":"sigsegv_recovered"}"#.to_string();
19220:     }
19221:     SIGSEGV_RECOVERY.store(true, Ordering::Relaxed);
19222:     let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
19223:         debug_ramen_participants_inner()
19224:     }))
19225:     .unwrap_or_else(|_| r#"{"error":"panic_caught"}"#.to_string());
19226:     SIGSEGV_RECOVERY.store(false, Ordering::Relaxed);
19227:     result
19228: }
19229: 
```

## lines 19218-19234

```rust
19218:         SIGSEGV_RECOVERY.store(false, Ordering::Relaxed);
19219:         return r#"{"error":"sigsegv_recovered"}"#.to_string();
19220:     }
19221:     SIGSEGV_RECOVERY.store(true, Ordering::Relaxed);
19222:     let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
19223:         debug_ramen_participants_inner()
19224:     }))
19225:     .unwrap_or_else(|_| r#"{"error":"panic_caught"}"#.to_string());
19226:     SIGSEGV_RECOVERY.store(false, Ordering::Relaxed);
19227:     result
19228: }
19229: 
19230: /// 诊断训练伙伴 — 只读，不修改 /summary 或评分
19231: unsafe fn debug_training_partners_inner() -> String {
19232:     if API.is_null() {
19233:         return r#"{"error":"api_null"}"#.to_string();
19234:     }
```

## lines 19336-19352

```rust
19336:     )
19337: }
19338: 
19339: /// 崩溃保护包装
19340: fn debug_training_partners() -> String {
19341:     let _lock = READ_MUTEX.lock().unwrap_or_else(|error| error.into_inner());
19342:     let jump_result = unsafe { sys_sigsetjmp(SIGSEGV_JMP_BUF.as_mut_ptr(), 1) };
19343:     if jump_result != 0 {
19344:         SIGSEGV_RECOVERY.store(false, Ordering::Relaxed);
19345:         return r#"{"error":"sigsegv_recovered","hint":"training partner diagnostic hit an invalid runtime pointer; game was protected"}"#.to_string();
19346:     }
19347:     SIGSEGV_RECOVERY.store(true, Ordering::Relaxed);
19348:     let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
19349:         debug_training_partners_inner()
19350:     }))
19351:     .unwrap_or_else(|_| r#"{"error":"panic_caught"}"#.to_string());
19352:     SIGSEGV_RECOVERY.store(false, Ordering::Relaxed);
```

## lines 19339-19355

```rust
19339: /// 崩溃保护包装
19340: fn debug_training_partners() -> String {
19341:     let _lock = READ_MUTEX.lock().unwrap_or_else(|error| error.into_inner());
19342:     let jump_result = unsafe { sys_sigsetjmp(SIGSEGV_JMP_BUF.as_mut_ptr(), 1) };
19343:     if jump_result != 0 {
19344:         SIGSEGV_RECOVERY.store(false, Ordering::Relaxed);
19345:         return r#"{"error":"sigsegv_recovered","hint":"training partner diagnostic hit an invalid runtime pointer; game was protected"}"#.to_string();
19346:     }
19347:     SIGSEGV_RECOVERY.store(true, Ordering::Relaxed);
19348:     let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
19349:         debug_training_partners_inner()
19350:     }))
19351:     .unwrap_or_else(|_| r#"{"error":"panic_caught"}"#.to_string());
19352:     SIGSEGV_RECOVERY.store(false, Ordering::Relaxed);
19353:     result
19354: }
19355: 
```

## lines 19344-19360

```rust
19344:         SIGSEGV_RECOVERY.store(false, Ordering::Relaxed);
19345:         return r#"{"error":"sigsegv_recovered","hint":"training partner diagnostic hit an invalid runtime pointer; game was protected"}"#.to_string();
19346:     }
19347:     SIGSEGV_RECOVERY.store(true, Ordering::Relaxed);
19348:     let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
19349:         debug_training_partners_inner()
19350:     }))
19351:     .unwrap_or_else(|_| r#"{"error":"panic_caught"}"#.to_string());
19352:     SIGSEGV_RECOVERY.store(false, Ordering::Relaxed);
19353:     result
19354: }
19355: 
19356: /// /debug/cmdinfo — Dump command element class info WITHOUT runtime_invoke on command elements
19357: /// Reads class from object header (offset 0), enumerates fields + methods + hex dump
19358: /// Safe: only uses il2cpp_class_get_fields / il2cpp_class_get_methods (no runtime_invoke on cmd elements)
19359: unsafe fn debug_cmdinfo() -> String {
19360:     if API.is_null() {
```

## lines 19667-19683

```rust
19667: 
19668:             // succession_relation_member: id + type + chara_id
19669:             if let Ok(mut stmt) = conn.prepare(
19670:                 "SELECT id, relation_type, chara_id FROM succession_relation_member ORDER BY id",
19671:             ) {
19672:                 let rows: Vec<String> = stmt
19673:                     .query_map([], |row| {
19674:                         Ok(format!(
19675:                             r#"{{"id":{},"type":{},"chara_id":{}}}"#,
19676:                             row.get::<_, i32>(0).unwrap_or(0),
19677:                             row.get::<_, i32>(1).unwrap_or(0),
19678:                             row.get::<_, i32>(2).unwrap_or(0)
19679:                         ))
19680:                     })
19681:                     .unwrap()
19682:                     .filter_map(|r| r.ok())
19683:                     .collect();
```

## lines 19750-19766

```rust
19750:                 race_names_json = rows;
19751:             }
19752:             drop(conn);
19753:         }
19754:     }
19755: 
19756:     format!(
19757:         r#"{{"version":"3.22.91","parents":{{"first_chara_id":{},"second_chara_id":{}}},"factor_count":{},"relations":[{}],"relation_members":[{}],"relation_ranks":[{}],"target_races":[{}],"route_races":[{}]}}"#,
19758:         first_chara_id,
19759:         second_chara_id,
19760:         factor_count,
19761:         relations_json.join(","),
19762:         relation_members_json.join(","),
19763:         relation_ranks_json.join(","),
19764:         target_races_json.join(","),
19765:         race_names_json.join(",")
19766:     )
```

## lines 19751-19767

```rust
19751:             }
19752:             drop(conn);
19753:         }
19754:     }
19755: 
19756:     format!(
19757:         r#"{{"version":"3.22.91","parents":{{"first_chara_id":{},"second_chara_id":{}}},"factor_count":{},"relations":[{}],"relation_members":[{}],"relation_ranks":[{}],"target_races":[{}],"route_races":[{}]}}"#,
19758:         first_chara_id,
19759:         second_chara_id,
19760:         factor_count,
19761:         relations_json.join(","),
19762:         relation_members_json.join(","),
19763:         relation_ranks_json.join(","),
19764:         target_races_json.join(","),
19765:         race_names_json.join(",")
19766:     )
19767: }
```

## lines 20027-20043

```rust
20027:                         r#"{{"name":"{}","type":{}}}"#,
20028:                         json_escape(&name),
20029:                         stype,
20030:                     ));
20031:                 }
20032:             }
20033: 
20034:             parent_saddles_json.push(format!(
20035:                 r#"{{"label":"{}","chara_id":{},"saddle_count":{},"saddles":[{}]}}"#,
20036:                 label,
20037:                 chara_id,
20038:                 p_count,
20039:                 p_entries.join(","),
20040:             ));
20041:         }
20042:     }
20043: 
```

## lines 20029-20045

```rust
20029:                         stype,
20030:                     ));
20031:                 }
20032:             }
20033: 
20034:             parent_saddles_json.push(format!(
20035:                 r#"{{"label":"{}","chara_id":{},"saddle_count":{},"saddles":[{}]}}"#,
20036:                 label,
20037:                 chara_id,
20038:                 p_count,
20039:                 p_entries.join(","),
20040:             ));
20041:         }
20042:     }
20043: 
20044:     // 5. Cross-reference with MDB for relation_group_id mapping
20045:     let mut mdb_saddle_map_json: Vec<String> = Vec::new();
```

## lines 20505-20521

```rust
20505:     if jmp_result != 0 {
20506:         let now = std::time::SystemTime::now()
20507:             .duration_since(std::time::UNIX_EPOCH)
20508:             .unwrap_or_default()
20509:             .as_secs();
20510:         SIGSEGV_COOLDOWN_UNTIL.store(now + 60, std::sync::atomic::Ordering::Relaxed);
20511:         return r#"{"error":"sigsegv_recovered_in_storydata"}"#.to_string();
20512:     }
20513:     SIGSEGV_RECOVERY.store(true, std::sync::atomic::Ordering::Relaxed);
20514:     let result = debug_storydata_inner();
20515:     SIGSEGV_RECOVERY.store(false, std::sync::atomic::Ordering::Relaxed);
20516:     result
20517: }
20518: 
20519: unsafe fn debug_storydata_inner() -> String {
20520:     if API.is_null() {
20521:         return r#"{"error":"api_null"}"#.to_string();
```

## lines 20507-20523

```rust
20507:             .duration_since(std::time::UNIX_EPOCH)
20508:             .unwrap_or_default()
20509:             .as_secs();
20510:         SIGSEGV_COOLDOWN_UNTIL.store(now + 60, std::sync::atomic::Ordering::Relaxed);
20511:         return r#"{"error":"sigsegv_recovered_in_storydata"}"#.to_string();
20512:     }
20513:     SIGSEGV_RECOVERY.store(true, std::sync::atomic::Ordering::Relaxed);
20514:     let result = debug_storydata_inner();
20515:     SIGSEGV_RECOVERY.store(false, std::sync::atomic::Ordering::Relaxed);
20516:     result
20517: }
20518: 
20519: unsafe fn debug_storydata_inner() -> String {
20520:     if API.is_null() {
20521:         return r#"{"error":"api_null"}"#.to_string();
20522:     }
20523:     let image = match get_image() {
```

## lines 20686-20702

```rust
20686:     // ★ Set up sigsetjmp recovery once for the entire call
20687:     let jmp_result = unsafe { sys_sigsetjmp(SIGSEGV_JMP_BUF.as_mut_ptr(), 1) };
20688:     if jmp_result != 0 {
20689:         let now = std::time::SystemTime::now()
20690:             .duration_since(std::time::UNIX_EPOCH)
20691:             .unwrap_or_default()
20692:             .as_secs();
20693:         SIGSEGV_COOLDOWN_UNTIL.store(now + 60, std::sync::atomic::Ordering::Relaxed);
20694:         SIGSEGV_RECOVERY.store(false, std::sync::atomic::Ordering::Relaxed);
20695:         return r#"{"error":"sigsegv_recovered_in_debug_all"}"#.to_string();
20696:     }
20697:     SIGSEGV_RECOVERY.store(true, std::sync::atomic::Ordering::Relaxed);
20698: 
20699:     // 1. summary — call _inner directly (skip its own mutex + sigsetjmp)
20700:     let summary = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
20701:         read_summary_inner()
20702:     }))
```

## lines 20689-20705

```rust
20689:         let now = std::time::SystemTime::now()
20690:             .duration_since(std::time::UNIX_EPOCH)
20691:             .unwrap_or_default()
20692:             .as_secs();
20693:         SIGSEGV_COOLDOWN_UNTIL.store(now + 60, std::sync::atomic::Ordering::Relaxed);
20694:         SIGSEGV_RECOVERY.store(false, std::sync::atomic::Ordering::Relaxed);
20695:         return r#"{"error":"sigsegv_recovered_in_debug_all"}"#.to_string();
20696:     }
20697:     SIGSEGV_RECOVERY.store(true, std::sync::atomic::Ordering::Relaxed);
20698: 
20699:     // 1. summary — call _inner directly (skip its own mutex + sigsetjmp)
20700:     let summary = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
20701:         read_summary_inner()
20702:     }))
20703:     .unwrap_or_else(|_| r#"{"error":"summary_panic"}"#.to_string());
20704:     parts.push(format!(r#""summary":{}"#, summary));
20705: 
```

## lines 20726-20742

```rust
20726:     // 5. rameninfo
20727:     let rameninfo = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
20728:         read_ramen_info()
20729:     }))
20730:     .unwrap_or_else(|_| r#"{"error":"rameninfo_panic"}"#.to_string());
20731:     parts.push(format!(r#""rameninfo":{}"#, rameninfo));
20732: 
20733:     // ★ Clear recovery flag
20734:     SIGSEGV_RECOVERY.store(false, std::sync::atomic::Ordering::Relaxed);
20735: 
20736:     format!("{{{}}}", parts.join(","))
20737: }
20738: 
20739: /// ★ v3.22.39: /debug/gauge — MINIMAL SAFE VERSION
20740: /// Only reads element class names + count. NO dict hex, NO GetGainCount.
20741: /// Will incrementally add features after confirming this doesn't crash.
20742: unsafe fn debug_gauge() -> String {
```

## lines 22732-22748

```rust
22732:              process_id INTEGER NOT NULL,
22733:              plugin_version TEXT NOT NULL,
22734:              started_at_ms INTEGER NOT NULL,
22735:              last_flush_ms INTEGER NOT NULL,
22736:              state TEXT NOT NULL,
22737:              recovered_after_restart INTEGER NOT NULL DEFAULT 0,
22738:              root_path TEXT NOT NULL
22739:          );
22740:          CREATE TABLE IF NOT EXISTS observation_files(
22741:              file_id INTEGER PRIMARY KEY AUTOINCREMENT,
22742:              session_id TEXT NOT NULL,
22743:              relative_path TEXT NOT NULL,
22744:              content_type TEXT NOT NULL,
22745:              byte_length INTEGER NOT NULL,
22746:              sha256 TEXT,
22747:              created_at_ms INTEGER NOT NULL,
22748:              UNIQUE(session_id, relative_path),
```

## lines 22743-22759

```rust
22743:              relative_path TEXT NOT NULL,
22744:              content_type TEXT NOT NULL,
22745:              byte_length INTEGER NOT NULL,
22746:              sha256 TEXT,
22747:              created_at_ms INTEGER NOT NULL,
22748:              UNIQUE(session_id, relative_path),
22749:              FOREIGN KEY(session_id) REFERENCES observation_sessions(session_id)
22750:          );
22751:          CREATE INDEX IF NOT EXISTS idx_observation_files_session_id_file_id
22752:              ON observation_files(session_id, file_id);"
22753:     ).map_err(|error| format!("initialize_schema:{}", error))?;
22754:     Ok(connection)
22755: }
22756: 
22757: fn ensure_observation_session() -> Result<String, String> {
22758:     if let Ok(value) = STORAGE_SESSION_ID.lock() {
22759:         if let Some(session_id) = value.as_ref() {
```

## lines 22744-22760

```rust
22744:              content_type TEXT NOT NULL,
22745:              byte_length INTEGER NOT NULL,
22746:              sha256 TEXT,
22747:              created_at_ms INTEGER NOT NULL,
22748:              UNIQUE(session_id, relative_path),
22749:              FOREIGN KEY(session_id) REFERENCES observation_sessions(session_id)
22750:          );
22751:          CREATE INDEX IF NOT EXISTS idx_observation_files_session_id_file_id
22752:              ON observation_files(session_id, file_id);"
22753:     ).map_err(|error| format!("initialize_schema:{}", error))?;
22754:     Ok(connection)
22755: }
22756: 
22757: fn ensure_observation_session() -> Result<String, String> {
22758:     if let Ok(value) = STORAGE_SESSION_ID.lock() {
22759:         if let Some(session_id) = value.as_ref() {
22760:             return Ok(session_id.clone());
```

## lines 22779-22795

```rust
22779:         rusqlite::params![session_id, process_id as i64, PLUGIN_VERSION, now as i64, now as i64, root_text],
22780:     ).map_err(|error| format!("insert_session:{}", error))?;
22781:     let session_directory = observation_storage_root().join("sessions").join(&session_id);
22782:     std::fs::create_dir_all(&session_directory).map_err(|error| format!("create_session_dir:{}", error))?;
22783:     let session_json = format!(
22784:         r#"{{"session_id":"{}","process_id":{},"plugin_version":"{}","started_at_ms":{},"state":"open","recovered_after_restart":false,"root_path":"{}"}}"#,
22785:         json_escape(&session_id), process_id, json_escape(PLUGIN_VERSION), now, json_escape(&root_text)
22786:     );
22787:     std::fs::write(session_directory.join("session.json"), session_json.as_bytes())
22788:         .map_err(|error| format!("write_session_json:{}", error))?;
22789:     connection.execute(
22790:         "INSERT OR REPLACE INTO observation_files(
22791:              session_id, relative_path, content_type, byte_length, sha256, created_at_ms
22792:          ) VALUES(?1, 'session.json', 'application/json', ?2, NULL, ?3)",
22793:         rusqlite::params![session_id, session_json.as_bytes().len() as i64, now as i64],
22794:     ).map_err(|error| format!("index_session_json:{}", error))?;
22795:     STORAGE_LAST_FLUSH_MS.store(now, Ordering::Relaxed);
```

## lines 22782-22798

```rust
22782:     std::fs::create_dir_all(&session_directory).map_err(|error| format!("create_session_dir:{}", error))?;
22783:     let session_json = format!(
22784:         r#"{{"session_id":"{}","process_id":{},"plugin_version":"{}","started_at_ms":{},"state":"open","recovered_after_restart":false,"root_path":"{}"}}"#,
22785:         json_escape(&session_id), process_id, json_escape(PLUGIN_VERSION), now, json_escape(&root_text)
22786:     );
22787:     std::fs::write(session_directory.join("session.json"), session_json.as_bytes())
22788:         .map_err(|error| format!("write_session_json:{}", error))?;
22789:     connection.execute(
22790:         "INSERT OR REPLACE INTO observation_files(
22791:              session_id, relative_path, content_type, byte_length, sha256, created_at_ms
22792:          ) VALUES(?1, 'session.json', 'application/json', ?2, NULL, ?3)",
22793:         rusqlite::params![session_id, session_json.as_bytes().len() as i64, now as i64],
22794:     ).map_err(|error| format!("index_session_json:{}", error))?;
22795:     STORAGE_LAST_FLUSH_MS.store(now, Ordering::Relaxed);
22796:     let mut state = STORAGE_SESSION_ID.lock().map_err(|_| "storage_session_lock_poisoned".to_string())?;
22797:     *state = Some(session_id.clone());
22798:     Ok(session_id)
```

## lines 22784-22800

```rust
22784:         r#"{{"session_id":"{}","process_id":{},"plugin_version":"{}","started_at_ms":{},"state":"open","recovered_after_restart":false,"root_path":"{}"}}"#,
22785:         json_escape(&session_id), process_id, json_escape(PLUGIN_VERSION), now, json_escape(&root_text)
22786:     );
22787:     std::fs::write(session_directory.join("session.json"), session_json.as_bytes())
22788:         .map_err(|error| format!("write_session_json:{}", error))?;
22789:     connection.execute(
22790:         "INSERT OR REPLACE INTO observation_files(
22791:              session_id, relative_path, content_type, byte_length, sha256, created_at_ms
22792:          ) VALUES(?1, 'session.json', 'application/json', ?2, NULL, ?3)",
22793:         rusqlite::params![session_id, session_json.as_bytes().len() as i64, now as i64],
22794:     ).map_err(|error| format!("index_session_json:{}", error))?;
22795:     STORAGE_LAST_FLUSH_MS.store(now, Ordering::Relaxed);
22796:     let mut state = STORAGE_SESSION_ID.lock().map_err(|_| "storage_session_lock_poisoned".to_string())?;
22797:     *state = Some(session_id.clone());
22798:     Ok(session_id)
22799: }
22800: 
```

## lines 23061-23077

```rust
23061:         );
23062:         format!(
23063:             r#"{{"slot":"{}","selected":true,"trained_chara_id":{},"trained_chara_record":null}}"#,
23064:             slot, trained_chara_id
23065:         )
23066:     };
23067: 
23068:     format!(
23069:         r#"{{"ok":true,"source":"current_work_single_mode_character","scope":"selected_parent_ids_only","target":{{"card_id":{},"chara_id":{}}},"parents":[{},{}],"trained_chara_record_resolution":null,"ancestor_tree":null,"pair_compatibility":null,"race_bonus":null,"runtime_consumer_result":null,"id_semantics":"trained_chara_id","getter_decode":"existing_runtime_invoke_int_path","runtime_validation":"pending_device_execution"}}"#,
23070:         target_card_id,
23071:         target_chara_id,
23072:         render_slot("first", first),
23073:         render_slot("second", second),
23074:     )
23075: }
23076: 
23077: // ===== Unified runtime correction E-stage =====
```

## lines 23063-23079

```rust
23063:             r#"{{"slot":"{}","selected":true,"trained_chara_id":{},"trained_chara_record":null}}"#,
23064:             slot, trained_chara_id
23065:         )
23066:     };
23067: 
23068:     format!(
23069:         r#"{{"ok":true,"source":"current_work_single_mode_character","scope":"selected_parent_ids_only","target":{{"card_id":{},"chara_id":{}}},"parents":[{},{}],"trained_chara_record_resolution":null,"ancestor_tree":null,"pair_compatibility":null,"race_bonus":null,"runtime_consumer_result":null,"id_semantics":"trained_chara_id","getter_decode":"existing_runtime_invoke_int_path","runtime_validation":"pending_device_execution"}}"#,
23070:         target_card_id,
23071:         target_chara_id,
23072:         render_slot("first", first),
23073:         render_slot("second", second),
23074:     )
23075: }
23076: 
23077: // ===== Unified runtime correction E-stage =====
23078: /// 辅助函数：IL2CPP类型枚举转可读名称
23079: fn type_enum_to_name(te: u8) -> String {
```
