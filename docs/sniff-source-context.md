# Targeted sniff source context

hits=55 blocks=9

## Lines 87-157
```rust
000087: static PREDICT_STEP: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
000088: static CRASH_SIG: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(0);
000089: static CRASH_STEP: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
000090: static mut LAST_STEP_BUF: [u8; 128] = [0; 128];
000091: static LAST_STEP_LEN: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
000092: static AUTO_UPDATE_STATUS: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);
000093: // ★ Training result/action state is shared by the game hook and HTTP/summary threads.
000094: // Keep correlated fields under one mutex to avoid data races and torn records.
000095: struct ActionState {
000096:     training_result: i32,
000097:     training_sub_id: i32,
000098:     command_id: i32,
000099:     sequence: u64,
000100: }
000101: static ACTION_STATE: Mutex<ActionState> = Mutex::new(ActionState {
000102:     training_result: -1, training_sub_id: -1, command_id: -1, sequence: 0,
000103: });
000104: static mut TRAINING_HOOK_INSTALLED: bool = false;
000105: static mut ORIG_ON_SUCCESS_PROLOGUE: [u8; 16] = [0; 16];
000106: static mut ON_SUCCESS_ADDR: usize = 0;
000107: // ★ v3.23.3: API sniffing — use Hachimi Interceptor API (hook+trampoline) + WWWRequest.Post for URL (replaces _Send+SetHeader)
000108: static SNIFF_ENABLED: AtomicBool = AtomicBool::new(false);
000109: static SNIFF_MUTEX: Mutex<()> = Mutex::new(());
000110: // SniffEntry: (id, url, headers_json, body)
000111: static mut SNIFF_REQUESTS: Vec<(u64, String, String, Vec<u8>)> = Vec::new();
000112: static mut SNIFF_RESPONSES: Vec<(u64, Vec<u8>)> = Vec::new();
000113: static SNIFF_MAX: usize = 20;
000114: static SNIFF_REQ_ID: AtomicU64 = AtomicU64::new(0);
000115: static mut PENDING_URL: String = String::new();
000116: static mut PENDING_HEADERS: Vec<(String, String)> = Vec::new();
000117: static mut PENDING_REQ_ID: u64 = 0;
000118: // CompressRequest/DecompressResponse/Post hook addresses (via Interceptor API)
000119: static mut COMPRESS_REQUEST_ADDR: usize = 0;
000120: static mut DECOMPRESS_RESPONSE_ADDR: usize = 0;
000121: static mut POST_ADDR: usize = 0;
000122: // Pending request body parking (CompressRequest → Post matching)
000123: static mut PENDING_REQ_BODY: Option<Vec<u8>> = None;
000124: static mut PENDING_COMPRESSED: usize = 0;
000125: // ★ Mutex to prevent concurrent read_summary_inner calls from HTTP + push threads
000126: static READ_MUTEX: Mutex<()> = Mutex::new(());
000127: 
000128: // ★ v3.24.2: Story event choice hook — capture career event choices (options, effects, branches)
000129: static mut EVENT_CHOICE_HOOK_INSTALLED: bool = false;
000130: static mut EVENT_CHOICE_ADDR: usize = 0; // StoryChoiceController.Choice
000131: static mut EVENT_ADD_BTN_ADDR: usize = 0; // StoryChoiceController.AddChoiceButton
000132: static mut ORIG_EVENT_CHOICE_PROLOGUE: [u8; 16] = [0; 16];
000133: static mut ORIG_EVENT_ADD_BTN_PROLOGUE: [u8; 16] = [0; 16];
000134: // ★ v3.24.2: StoryManager.SetStory hook — capture story_id and chara_id for event type identification
000135: static mut STORY_SET_HOOK_INSTALLED: bool = false;
000136: static mut STORY_SET_ADDR: usize = 0;
000137: static mut ORIG_STORY_SET_PROLOGUE: [u8; 16] = [0; 16];
000138: // Event state: accumulated choices for current event
000139: static EVENT_STATE_MUTEX: Mutex<()> = Mutex::new(());
000140: 
000141: // ★ v3.24.40: mirror every ura_log line into a queryable ring buffer
000142: // (Hachimi logcat was the only outlet before; /debug/hooklog exposes it).
000143: static HOOK_LOG: Mutex<Vec<String>> = Mutex::new(Vec::new());
000144: const HOOK_LOG_MAX: usize = 256;
000145: 
000146: // ★ v3.24.42: high-frequency read_summary/push spam is excluded from the
000147: // ring (still goes to logcat) so event/sniff diagnostics survive.
000148: const HOOK_LOG_NOISE: &[&str] = &[
000149:     "★ read_summary", "ramen scalar", "ramen arrays", "evaluation_list",
000150:     "sc: ", "skill_eval=", "v3.22.51 ramen", "★ Scenario 14", "Push:",
000151:     "call_getter: 'get_Skill", "call_getter: 'get_PossessSkill",
000152:     "find_field_offset: 'RemainTurn'",
000153: ];
000154: fn hook_log(msg: &str) {
000155:     if HOOK_LOG_NOISE.iter().any(|n| msg.contains(n)) { return; }
000156:     if let Ok(mut g) = HOOK_LOG.lock() {
000157:         if g.len() >= HOOK_LOG_MAX { g.remove(0); }
```

## Lines 6417-6471
```rust
006417:     // Don't rely solely on GAME_INITIALIZED callback — it may never fire
006418:     // if the game was already initialized before the plugin loaded.
006419:     // Instead, try reading data; if it succeeds, the game is ready.
006420:     for wait_round in 0..60 {
006421:         if GAME_INITIALIZED.load(Ordering::Relaxed) {
006422:             break;
006423:         }
006424:         boot_trace("push_probe_begin");
006425:         // Try a probe read — if it doesn't error, game is ready
006426:         let probe = read_summary();
006427:         if !probe.contains("\"error\"") {
006428:             GAME_INITIALIZED.store(true, Ordering::Relaxed);
006429:             unsafe {
006430:                 ura_log(3, "Push: game detected via probe (no callback)");
006431:                 // v3.22.98: Install hooks in fallback (on_game_initialized may never fire)
006432:                 install_training_hook();
006433:                 install_exec_training_hook();
006434:                 install_failure_rate_hook();
006435:                 install_event_choice_hook();
006436:                 // ★ v3.24.40: sniff hooks were missing here — fallback mode
006437:                 // left /api/sniff permanently unhooked.
006438:                 install_api_sniff_hooks();
006439:             }
006440:             break;
006441:         }
006442:         if wait_round % 10 == 0 {
006443:             unsafe {
006444:                 ura_log(
006445:                     3,
006446:                     &format!("Push: waiting for game... round={}", wait_round),
006447:     
006448:                 );
006449:             }
006450:         }
006451:         std::thread::sleep(std::time::Duration::from_secs(1));
006452:     }
006453:     let init_summary = read_summary();
006454:     if !init_summary.contains("\"error\"") {
006455:         unsafe {
006456:             LAST_PUSH_HASH = simple_hash(&init_summary);
006457:         }
006458:         push_to_app(&init_summary);
006459:         unsafe {
006460:             ura_log(3, "Push: initial data pushed");
006461:         }
006462:     }
006463: 
006464:     loop {
006465:         std::thread::sleep(interval);
006466:         // Don't gate on GAME_INITIALIZED — just try reading;
006467:         // if the game isn't ready, read_summary returns error and we skip.
006468:         let summary = read_summary();
006469:         if summary.contains("\"error\"") {
006470:             consecutive_errors += 1;
006471:             // ★ v3.22.89: Extra cooldown for SIGSEGV recovery — game state transition
```

## Lines 6961-7058
```rust
006961:     let req = std::str::from_utf8(&buf[..n]).unwrap_or("");
006962:     let path = parse_path(req);
006963:     let full_uri = req
006964:         .lines()
006965:         .next()
006966:         .unwrap_or("")
006967:         .split(' ')
006968:         .nth(1)
006969:         .unwrap_or("/");
006970: 
006971:     // ★ v3.24.55: boot gate. Crash autopsy via hachimi.log: the floating app
006972:     // polls /summary during game boot; IL2CPP reads on the HTTP thread against
006973:     // transitional objects SIGSEGV the process (sigjmp recovery only exists on
006974:     // the push thread). Until the game is initialized, refuse every endpoint
006975:     // that touches game memory; static/self-state endpoints stay available.
006976:     if !GAME_INITIALIZED.load(Ordering::Relaxed) {
006977:         const BOOT_SAFE_EXACT: &[&str] = &[
006978:             "/", "/health", "/status", "/config", "/config.html",
006979:             "/update", "/update/status",
006980:             "/debug/hookdiag", "/debug/hooklog", "/debug/crashlog", "/debug/upload",
006981:             "/api/sniff", "/api/sniff/diag", "/api/sniff/toggle", "/api/sniff/clear",
006982:             "/api/event/choices", "/api/event/observations", "/api/event/observations/clear", "/api/event/clear",
006983:             "/action/latest", "/seed/history", "/seed/stats",
006984:             "/log", "/carddb", "/skilldata",
006985:             "/debug/table", "/debug/push_table", "/debug/download_table",
006986:             "/debug/mdb_all_tables", "/debug/mdb_schema_dump",
006987:         ];
006988:         const BOOT_SAFE_PREFIX: &[&str] = &[
006989:             "/mdb", "/debug/resource_", "/debug/private_file", "/debug/mem_scan_sqlite", "/debug/mem_scan_zdict", "/debug/mem_scan_hex", "/debug/file_scan_hex", "/debug/maps_list", "/debug/file_dl", "/debug/file_range_hex",
006990:         ];
006991:         let safe = BOOT_SAFE_EXACT.iter().any(|p| path == *p)
006992:             || BOOT_SAFE_PREFIX.iter().any(|p| path.starts_with(p));
006993:         if !safe {
006994:             let b = r#"{"status":"booting","game_initialized":false}"#;
006995:             let resp = format!(
006996:                 "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
006997:                 b.len(), b
006998:             );
006999:             let _ = stream.write_all(resp.as_bytes());
007000:             return;
007001:         }
007002:     }
007003: 
007004:     // ★ 白名单下载开关：名单内端点追加 ?dl=1 即以附件形式返回（解决手机复制长度上限）
007005:     //    ?dl=1&name=xxx 可自定义文件名（仅保留字母数字和下划线/连字符）
007006:     //    大文件仍走各专用流式 _dl 端点，避免此路径内存翻倍
007007:     const DL_ALLOWED: &[&str] = &[
007008:         "/summary", "/scenario", "/data", "/ramen", "/debug/ramen_transition",
007009:         "/api/sniff", "/api/sniff/diag", "/api/event/choices", "/api/event/observations",
007010:         "/debug/event_reward_targets", "/debug/resource_meta_schema", "/debug/resource_meta_probe", "/debug/resource_crypto_symbols",
007011:         "/debug/all", "/debug/params", "/debug/cmdinfo", "/debug/breeders",
007012:         "/debug/training_partners", "/debug/rameninfo", "/debug/laststep",
007013:         "/debug/storydata", "/debug/ramenfields", "/debug/gauge", "/debug/gauge2",
007014:         "/debug/ramengains", "/debug/paramsincdec", "/debug/training_seed",
007015:         "/debug/unique_skills", "/debug/hint_gain", "/debug/sc_effect",
007016:         "/debug/unique_detail", "/classes",
007017:     ];
007018:     let dl_flag = parse_query(&full_uri, "dl");
007019:     let dl_name = parse_query(&full_uri, "name");
007020:     let dl_enabled = !dl_flag.is_empty() && dl_flag != "0"
007021:         && DL_ALLOWED.iter().any(|p| path == *p);
007022: 
007023:     let body = if path == "/" || path == "/health" {
007024:         format!(r#"{{"status":"ok","version":"{}","endpoints":["/summary","/data","/scenario","/debug/rameninfo","/debug/laststep","/event/recommend","/inherit/compat","/saddle-analysis","/log/turn","/debug/params","/debug/breeders","/debug/cmdinfo","/debug/training_partners","/debug/crashlog","/debug/upload","/debug/dumpclass","/debug/storydata","/debug/ramenfields","/debug/gauge","/debug/gauge2","/debug/ramengains","/debug/paramsincdec","/debug/training_seed","/debug/training_log","/debug/training_log_dl","/update","/update/status","/debug/all","/debug/unique_skills","/debug/mdb_all_tables","/debug/mdb_schema_dump","/debug/hint_gain","/debug/sc_effect","/debug/unique_detail","/debug/table","/debug/push_table","/debug/download_table","/mdb","/carddb","/skilldata","/hall","/saddles","/saddles-dl","/log","/status","/health","/mdb/schema","/mdb/search","/mdb/raw","/mdb/dl_batch","/il2cpp/dump","/il2cpp/call","/il2cpp/tree","/il2cpp/field","/il2cpp/classes","/il2cpp/static","/il2cpp/methods","/il2cpp/disassemble","/il2cpp/disassemble_dl","/il2cpp/disassemble_addr","/il2cpp/disassemble_addr_dl","/il2cpp/dump_all_methods","/il2cpp/dump_all_methods_dl","/il2cpp/search_float","/il2cpp/search_float_dl","/il2cpp/search_int","/il2cpp/search_int_dl","/il2cpp/search_methods","/il2cpp/search_methods_dl","/il2cpp/read_mem","/il2cpp/read_mem_dl","/training/result","/api/sniff","/api/sniff/toggle","/api/sniff/clear","/api/sniff/diag","/api/event/choices","/api/event/clear","/debug/hooklog","/debug/hookdiag","/debug/resource_meta_key","/debug/resource_db_keys","/debug/resource_reads","/debug/mem_scan_sqlite","/debug/meta_dump","/action/latest","/seed/history","/seed/stats","/debug/ramen_planner_state","/debug/ramen_participants","/debug/ramen_transition","/debug/ramen_dataset_path","/debug/ramen_formula_targets","/debug/event_reward_targets", "/debug/resource_storage","/debug/resource_meta_schema","/debug/resource_meta_probe", "/debug/resource_crypto_symbols","/debug/resource_meta_dl","/debug/resource_file_dl","/debug/private_file_inventory","/debug/private_file_dl"]}}"#, PLUGIN_VERSION)
007025:     } else if path == "/scan" {
007026:         unsafe { scan_il2cpp_classes() }
007027:     } else if path == "/data" {
007028:         let result = unsafe { read_training_data() };
007029:         unsafe {
007030:             log_snapshot("data", &result);
007031:         }
007032:         result
007033:     } else if path == "/status" {
007034:         format!(
007035:             r#"{{"game_initialized":{},"http_running":{}}}"#,
007036:             GAME_INITIALIZED.load(Ordering::Relaxed),
007037:             HTTP_RUNNING.load(Ordering::Relaxed)
007038:         )
007039:     } else if path == "/singletons" {
007040:         unsafe { find_all_singletons() }
007041:     } else if path.starts_with("/find_method") {
007042:         let method_name = if path == "/find_method" || path == "/find_method/" {
007043:             "get_SingleMode"
007044:         } else {
007045:             path.strip_prefix("/find_method/")
007046:                 .unwrap_or("get_SingleMode")
007047:         };
007048:         unsafe { find_method_in_all_classes(method_name) }
007049:     } else if path.starts_with("/fields") {
007050:         let class_name = if path == "/fields" || path == "/fields/" {
007051:             "WorkDataManager"
007052:         } else {
007053:             path.strip_prefix("/fields/").unwrap_or("WorkDataManager")
007054:         };
007055:         unsafe {
007056:             let image = get_image();
007057:             if image.is_null() {
007058:                 r#"{"error":"image_null"}"#.to_string()
```

## Lines 7153-7225
```rust
007153:     } else if path == "/debug/ramen_participants" {
007154:         debug_ramen_participants()
007155:     } else if path == "/training/result" {
007156:         // v3.22.94: Read latest training result from hook
007157:         let (result, sub_id) = ACTION_STATE.lock()
007158:             .map(|state| (state.training_result, state.training_sub_id))
007159:             .unwrap_or((-1, -1));
007160:         let hooked = unsafe { TRAINING_HOOK_INSTALLED };
007161:         format!(
007162:             r#"{{"result_type":{},"sub_id":{},"hooked":{},"result_name":"{}"}}"#,
007163:             result,
007164:             sub_id,
007165:             hooked,
007166:             match result {
007167:                 0 => "GreatSuccess",
007168:                 1 => "Success",
007169:                 2 => "Failure",
007170:                 _ => "Unknown",
007171:             }
007172:         )
007173:     } else if path == "/api/sniff/toggle" {
007174:         // ★ v3.24.40: lazy retry for fallback-mode installs.
007175:         unsafe {
007176:             install_api_sniff_hooks();
007177:         }
007178:         let new_val = !SNIFF_ENABLED.load(Ordering::Relaxed);
007179:         SNIFF_ENABLED.store(new_val, Ordering::Relaxed);
007180:         let req_hooked = unsafe { COMPRESS_REQUEST_ADDR != 0 };
007181:         let resp_hooked = unsafe { DECOMPRESS_RESPONSE_ADDR != 0 };
007182:         let post_hooked = unsafe { POST_ADDR != 0 };
007183:         format!(
007184:             r#"{{"sniff_enabled":{},"compress_hooked":{},"decompress_hooked":{},"post_hooked":{}}}"#,
007185:             new_val, req_hooked, resp_hooked, post_hooked
007186:         )
007187:     } else if path == "/api/sniff/clear" {
007188:         let _lock = SNIFF_MUTEX.lock();
007189:         unsafe {
007190:             SNIFF_REQUESTS.clear();
007191:             SNIFF_RESPONSES.clear();
007192:         }
007193:         r#"{"ok":true}"#.to_string()
007194:     } else if path.starts_with("/debug/hooklog") {
007195:         // ★ v3.24.40/42: last HOOK_LOG_MAX lines, optional ?filter=substr
007196:         let filter = parse_query(&full_uri, "filter");
007197:         let entries: Vec<String> = match HOOK_LOG.lock() {
007198:             Ok(g) => g.iter()
007199:                 .filter(|l| filter.is_empty() || l.contains(&filter))
007200:                 .map(|l| json_escape(l)).collect(),
007201:             Err(_) => Vec::new(),
007202:         };
007203:         format!(r#"{{"count":{},"entries":[{}]}}"#, entries.len(), entries.join(","))
007204:     } else if path == "/debug/resource_reads" {
007205:         // ★ v3.24.58: meta/dat file-read trace. Lazy-starts the /proc watcher
007206:         // on first request (never at init — thread spawn in init context).
007207:         start_res_fd_watcher();
007208:         let entries: Vec<String> = match RES_READ_LOG.lock() {
007209:             Ok(g) => g.iter().map(|l| format!("\"{}\"", json_escape(l))).collect(),
007210:             Err(_) => Vec::new(),
007211:         };
007212:         format!(r#"{{"count":{},"entries":[{}]}}"#, entries.len(), entries.join(","))
007213:     } else if path.starts_with("/debug/mem_scan_sqlite") {
007214:         // ★ v3.24.58: hunt plaintext "SQLite format 3" pages in process memory
007215:         // — any custom decryption MUST materialize this in RAM.
007216:         let max_hits: usize = parse_query(&full_uri, "max").parse().unwrap_or(8);
007217:         let mut hits: Vec<String> = Vec::new();
007218:         let needle = b"SQLite format 3 ";
007219:         if let Ok(maps) = std::fs::read_to_string("/proc/self/maps") {
007220:             let mem = std::fs::File::open("/proc/self/mem");
007221:             use std::os::unix::fs::FileExt;
007222:             if let Ok(mem) = mem {
007223:                 'outer: for line in maps.lines() {
007224:                     let cols: Vec<&str> = line.split_whitespace().collect();
007225:                     if cols.len() < 6 { continue; }
```

## Lines 7499-7600
```rust
007499:     } else if path == "/debug/resource_meta_key" {
007500:         // ★ v3.24.44: captured SQLCipher key for the resource `meta` DB
007501:         let key = META_KEY_HEX.lock().map(|g| g.clone()).unwrap_or_default();
007502:         format!(
007503:             r#"{{"captured":{},"key_len":{},"key_hex":"{}","persisted_file":"files/ura_meta_key.txt"}}"#,
007504:             if key.is_empty() { "false" } else { "true" },
007505:             key.len() / 2,
007506:             key
007507:         )
007508:     } else if path == "/debug/hookdiag" {
007509:         // ★ v3.24.40: per-hook install status
007510:         let items: Vec<String> = match HOOK_STATUS.lock() {
007511:             Ok(g) => g.iter().map(|(n, st)| format!(r#"{{"hook":"{}","status":"{}"}}"#, json_escape(n), json_escape(st))).collect(),
007512:             Err(_) => Vec::new(),
007513:         };
007514:         format!(
007515:             r#"{{"game_initialized":{},"hooks":[{}]}}"#,
007516:             GAME_INITIALIZED.load(Ordering::Relaxed),
007517:             items.join(",")
007518:         )
007519:     } else if path == "/api/sniff/diag" {
007520:         // v3.23.3: Diagnostic endpoint for hook installation (Interceptor API)
007521:         let req_hooked = unsafe { COMPRESS_REQUEST_ADDR != 0 };
007522:         let resp_hooked = unsafe { DECOMPRESS_RESPONSE_ADDR != 0 };
007523:         let post_hooked = unsafe { POST_ADDR != 0 };
007524:         let req_addr = unsafe { COMPRESS_REQUEST_ADDR };
007525:         let resp_addr = unsafe { DECOMPRESS_RESPONSE_ADDR };
007526:         let post_addr = unsafe { POST_ADDR };
007527:         let interceptor_available = unsafe { !API.is_null() && (*API).interceptor != 0 };
007528:         let has_get_method_addr =
007529:             unsafe { !API.is_null() && (*API).il2cpp_get_method_addr_fn.is_some() };
007530:         format!(
007531:             r#"{{"sniff_enabled":{},"compress_hooked":{},"decompress_hooked":{},"post_hooked":{},"compress_addr":"0x{:x}","decompress_addr":"0x{:x}","post_addr":"0x{:x}","interceptor_available":{},"get_method_addr_available":{}}}"#,
007532:             SNIFF_ENABLED.load(Ordering::Relaxed),
007533:             req_hooked,
007534:             resp_hooked,
007535:             post_hooked,
007536:             req_addr,
007537:             resp_addr,
007538:             post_addr,
007539:             interceptor_available,
007540:             has_get_method_addr
007541:         )
007542:     } else if path == "/api/sniff" {
007543:         let _lock = SNIFF_MUTEX.lock();
007544:         unsafe {
007545:             let reqs: Vec<String> = SNIFF_REQUESTS
007546:                 .iter()
007547:                 .map(|(rid, url, headers, data)| {
007548:                     let preview = String::from_utf8_lossy(&data[..data.len().min(2048)]);
007549:                     let preview = preview
007550:                         .replace('\\', "\\\\")
007551:                         .replace('"', "\\\"")
007552:                         .replace('\n', "\\n")
007553:                         .replace('\r', "");
007554:                     let url_escaped = url.replace('\\', "\\\\").replace('"', "\\\"");
007555:                     format!(
007556:                         r#"{{"id":{},"url":"{}","headers":{},"size":{},"hex":"{}","text":"{}"}}"#,
007557:                         rid,
007558:                         url_escaped,
007559:                         headers,
007560:                         data.len(),
007561:                         hex_encode(&data[..data.len().min(256)]),
007562:                         preview
007563:                     )
007564:                 })
007565:                 .collect();
007566:             let resps: Vec<String> = SNIFF_RESPONSES
007567:                 .iter()
007568:                 .map(|(rid, data)| {
007569:                     let preview = String::from_utf8_lossy(&data[..data.len().min(2048)]);
007570:                     let preview = preview
007571:                         .replace('\\', "\\\\")
007572:                         .replace('"', "\\\"")
007573:                         .replace('\n', "\\n")
007574:                         .replace('\r', "");
007575:                     format!(
007576:                         r#"{{"id":{},"size":{},"hex":"{}","text":"{}"}}"#,
007577:                         rid,
007578:                         data.len(),
007579:                         hex_encode(&data[..data.len().min(256)]),
007580:                         preview
007581:                     )
007582:                 })
007583:                 .collect();
007584:             format!(
007585:                 r#"{{"enabled":{},"requests":[{}],"responses":[{}]}}"#,
007586:                 SNIFF_ENABLED.load(Ordering::Relaxed),
007587:                 reqs.join(","),
007588:                 resps.join(",")
007589:             )
007590:         }
007591:     } else if path == "/api/event/choices" {
007592:         // ★ v3.24.40: lazy retry — early-boot install may have missed.
007593:         unsafe {
007594:             if !EVENT_CHOICE_HOOK_INSTALLED || !STORY_SET_HOOK_INSTALLED {
007595:                 install_event_choice_hook();
007596:             }
007597:         }
007598:         // v3.24.2: Return captured event choices
007599:         let _lock = EVENT_STATE_MUTEX.lock();
007600:         unsafe {
```

## Lines 8276-8330
```rust
008276:                 Ok((_, dat)) => {
008277:                     let target = std::path::Path::new(&dat).join(&hash[..2]).join(&hash);
008278:                     if !target.is_file() {
008279:                         format!(r#"{{"error":"resource_not_found","hash":"{}"}}"#, hash)
008280:                     } else {
008281:                         stream_private_file(&mut stream, &target.to_string_lossy(), &hash); return;
008282:                     }
008283:                 },
008284:                 Err(e) => format!(r#"{{"error":"{}"}}"#, json_escape(&e)),
008285:             }
008286:         }
008287:     } else if path == "/mdb" {
008288:         // v3.22.51: Serve raw MasterDB file for client-side processing
008289:         // Uses marker string; binary file sent in response handler below
008290:         match find_mdb_path() {
008291:             Some(mdb_path) => format!("__MDB_BINARY__{}", mdb_path),
008292:             None => r#"{"error":"mdb_not_found"}"#.to_string(),
008293:         }
008294:     } else {
008295:         format!(
008296:             r#"{{"error":"not_found","path":"{}","available":["/scan","/data","/status","/health","/scenario","/debug/upload","/debug/rameninfo","/debug/laststep","/event/recommend","/inherit/compat","/saddle-analysis","/log/turn","/log","/debug/params","/fields","/methods","/singletons","/find_method","/classes","/carddb","/skilldata","/hall","/debug/breeders","/debug/cmdinfo","/debug/training_partners","/debug/ramengains","/debug/paramsincdec","/debug/training_seed","/debug/training_log","/debug/training_log_dl","/update","/update/status","/debug/dumpclass","/debug/storydata","/debug/ramenfields","/debug/all","/mdb","/debug/push_table","/debug/download_table","/classes/search/keyword","/mdb/schema","/mdb/search","/mdb/raw","/mdb/dl_batch","/il2cpp/dump","/il2cpp/call","/il2cpp/tree","/il2cpp/field","/il2cpp/classes","/il2cpp/static","/il2cpp/methods","/il2cpp/search_float","/il2cpp/search_float_dl","/il2cpp/search_int","/il2cpp/search_int_dl","/il2cpp/search_methods","/il2cpp/search_methods_dl","/il2cpp/search_methods_page","/il2cpp/read_mem","/il2cpp/read_mem_dl","/training/result","/api/sniff","/api/sniff/toggle","/api/sniff/clear","/api/sniff/diag","/api/event/choices","/api/event/clear"]}}"#,
008297:             path
008298:         )
008299:     };
008300: 
008301:     save_endpoint_log(&path, &body);
008302: 
008303:     if body.starts_with("__MDB_BINARY__") {
008304:         // v3.22.51: Serve raw mdb file as binary response
008305:         let mdb_path = &body[14..]; // skip "__MDB_BINARY__"
008306:         match std::fs::read(mdb_path) {
008307:             Ok(data) => {
008308:                 let header = format!(
008309:                     "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Disposition: attachment; filename=\"master.mdb\"\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
008310:                     data.len()
008311:                 );
008312:                 let _ = stream.write_all(header.as_bytes());
008313:                 // Write in chunks to avoid memory spike
008314:                 for chunk in data.chunks(65536) {
008315:                     let _ = stream.write_all(chunk);
008316:                 }
008317:             }
008318:             Err(e) => {
008319:                 let err_json = format!(r#"{{"error":"mdb_read_failed","detail":"{}"}}"#, e);
008320:                 let resp = format!(
008321:                     "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
008322:                     err_json.len(), err_json
008323:                 );
008324:                 let _ = stream.write_all(resp.as_bytes());
008325:             }
008326:         }
008327:     } else if path == "/saddles-dl" {
008328:         let resp = format!(
008329:             "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Disposition: attachment; filename=\"saddles.json\"\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
008330:             body.len(), body
```

## Lines 8933-9113
```rust
008933:     if method_addr == 0 {
008934:         return false;
008935:     }
008936:     if interceptor_hook(method_addr, handler_addr) {
008937:         ura_log(
008938:             3,
008939:             &format!("{}: hooked at 0x{:x} (interceptor)", name, method_addr),
008940:         );
008941:         true
008942:     } else {
008943:         ura_log(
008944:             2,
008945:             &format!("{}: interceptor failed, fallback to write_hook_bytes", name),
008946:         );
008947:         std::ptr::copy_nonoverlapping(method_addr as *const u8, orig_prologue.as_mut_ptr(), 16);
008948:         write_hook_bytes(method_addr, handler_addr);
008949:         true
008950:     }
008951: }
008952: 
008953: // ★ v3.23.3: Hook handler for CompressRequest(byte[] data) -> byte[]
008954: // Parks the uncompressed request body, keyed by the compressed byte array returned by the original.
008955: // WWWRequest.Post will match it later.
008956: extern "C" fn compress_request_hook_handler(data: *mut c_void) -> *mut c_void {
008957:     let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
008958:         let body = read_il2cpp_byte_array(data);
008959:         let trampoline = interceptor_get_trampoline(compress_request_hook_handler as usize);
008960:         if trampoline == 0 {
008961:             return std::ptr::null_mut();
008962:         }
008963:         type FnType = unsafe extern "C" fn(*mut c_void) -> *mut c_void;
008964:         let original: FnType = std::mem::transmute(trampoline);
008965:         let compressed = original(data);
008966:         if !body.is_empty() && POST_ADDR != 0 {
008967:             PENDING_REQ_BODY = Some(body);
008968:             PENDING_COMPRESSED = compressed as usize;
008969:         }
008970:         compressed
008971:     }));
008972:     result.unwrap_or_else(|e| {
008973:         unsafe {
008974:             ura_log(1, &format!("compress_hook panic: {:?}", e));
008975:         }
008976:         std::ptr::null_mut()
008977:     })
008978: }
008979: 
008980: // ★ v3.23.3: Hook handler for DecompressResponse(byte[] data) -> byte[]
008981: // Forwards the decompressed response body with the matching request's URL + headers.
008982: extern "C" fn decompress_response_hook_handler(data: *mut c_void) -> *mut c_void {
008983:     unsafe {
008984:         let trampoline = interceptor_get_trampoline(decompress_response_hook_handler as usize);
008985:         if trampoline == 0 {
008986:             return std::ptr::null_mut();
008987:         }
008988:         type FnType = unsafe extern "C" fn(*mut c_void) -> *mut c_void;
008989:         let original: FnType = std::mem::transmute(trampoline);
008990:         let decompressed = original(data);
008991:         let bytes = read_il2cpp_byte_array(decompressed);
008992:         if !bytes.is_empty() {
008993:             if let Ok(mut pending) = EVENT_PENDING_RESULT.lock() {
008994:                 if let Some(sel) = pending.take() {
008995:                     let preview_len = bytes.len().min(EVENT_RESPONSE_PREVIEW_MAX);
008996:                     let preview = String::from_utf8_lossy(&bytes[..preview_len]);
008997:                     let (label, gain_id, next_block_idx, loop_exit_gain_id) = match sel.choice {
008998:                         Some(c) => (c.label, c.gain_id, c.next_block_idx, c.loop_exit_gain_id),
008999:                         None => (String::new(), -1, -1, -1),
009000:                     };
009001:                     let observation_id = EVENT_OBSERVATION_ID.fetch_add(1, Ordering::Relaxed);
009002:                     let record = format!(r#"{{"schema_version":2,"observation_id":{},"source":"runtime_observation","causality":"unknown","result_label":"unknown","captured_at":{},"generation":{},"story_id":{},"chara_id":{},"selected_idx_raw":{},"choice":{{"label":"{}","gain_id":{},"next_block_idx":{},"loop_exit_gain_id":{}}},"response":{{"request_id":{},"url":"{}","size_captured":{},"preview_truncated":{},"hex_prefix":"{}","text_preview":"{}"}}}}"#,
009003:                         observation_id, sel.captured_at, sel.generation, sel.story_id, sel.chara_id,
009004:                         sel.selected_idx_raw, json_escape(&label), gain_id, next_block_idx,
009005:                         loop_exit_gain_id, PENDING_REQ_ID, json_escape(&PENDING_URL), bytes.len(),
009006:                         bytes.len() > preview_len, hex_encode(&bytes[..bytes.len().min(64)]),
009007:                         json_escape(&preview));
009008:                     if let Ok(mut obs) = EVENT_OBSERVATIONS.lock() {
009009:                         if obs.len() >= EVENT_OBSERVATIONS_MAX { obs.remove(0); }
009010:                         obs.push(record);
009011:                     }
009012:                 }
009013:             }
009014:         }
009015:         if SNIFF_ENABLED.load(Ordering::Relaxed) {
009016:             if !bytes.is_empty() {
009017:                 let _lock = SNIFF_MUTEX.lock();
009018:                 let rid = PENDING_REQ_ID;
009019:                 SNIFF_RESPONSES.push((rid, bytes));
009020:                 if SNIFF_RESPONSES.len() > SNIFF_MAX {
009021:                     SNIFF_RESPONSES.remove(0);
009022:                 }
009023:             }
009024:         }
009025:         decompressed
009026:     }
009027: }
009028: 
009029: // ★ v3.23.3: Hook handler for WWWRequest.Post(this, url, postData, headers)
009030: // Captures URL + headers directly, and matches the parked request body from CompressRequest.
009031: // This replaces the old _Send + SetHeader approach.
009032: extern "C" fn post_hook_handler(
009033:     this: *mut c_void,
009034:     url: *const c_void,
009035:     post_data: *mut c_void,
009036:     headers: *mut c_void,
009037: ) -> *mut c_void {
009038:     unsafe {
009039:         let trampoline = interceptor_get_trampoline(post_hook_handler as usize);
009040:         if trampoline == 0 {
009041:             return std::ptr::null_mut();
009042:         }
009043:         type FnType = unsafe extern "C" fn(
009044:             *mut c_void,
009045:             *const c_void,
009046:             *mut c_void,
009047:             *mut c_void,
009048:         ) -> *mut c_void;
009049:         let original: FnType = std::mem::transmute(trampoline);
009050: 
009051:         // Capture URL
009052:         let game_url = if !url.is_null() {
009053:             read_il2cpp_string(url)
009054:         } else {
009055:             String::new()
009056:         };
009057:         let game_url = if game_url.is_empty() {
009058:             None
009059:         } else {
009060:             Some(game_url)
009061:         };
009062: 
009063:         // Capture headers from Dictionary<string,string>
009064:         let req_headers = read_string_dict(headers);
009065: 
009066:         if SNIFF_ENABLED.load(Ordering::Relaxed) {
009067:             let rid = SNIFF_REQ_ID.fetch_add(1, Ordering::Relaxed);
009068:             PENDING_REQ_ID = rid;
009069:             // Try to match parked request body
009070:             if let Some(body) = PENDING_REQ_BODY.take() {
009071:                 let headers_json = format_headers_json(&req_headers);
009072:                 let url_str = game_url.clone().unwrap_or_default();
009073:                 let _lock = SNIFF_MUTEX.lock();
009074:                 SNIFF_REQUESTS.push((rid, url_str, headers_json, body));
009075:                 if SNIFF_REQUESTS.len() > SNIFF_MAX {
009076:                     SNIFF_REQUESTS.remove(0);
009077:                 }
009078:             }
009079:             PENDING_URL = game_url.clone().unwrap_or_default();
009080:             PENDING_HEADERS = req_headers.clone();
009081:         }
009082: 
009083:         let _ = this;
009084:         original(this, url, post_data, headers)
009085:     }
009086: }
009087: 
009088: // ★ v3.23.3: Read IL2CPP Dictionary<string,string> into Vec<(String,String)>
009089: // Layout: [hdr 0x10][fields...]; _entries @+0x18, _count @+0x20
009090: // Entry: [hashCode:i32][next:i32][key:ptr][value:ptr] = 24B per entry
009091: unsafe fn read_string_dict(dict: *mut c_void) -> Vec<(String, String)> {
009092:     let mut out = Vec::new();
009093:     if dict.is_null() {
009094:         return out;
009095:     }
009096:     let count = std::ptr::read_unaligned::<i32>((dict as *const u8).add(0x20) as *const i32);
009097:     if count <= 0 {
009098:         return out;
009099:     }
009100:     let entries = std::ptr::read_unaligned::<usize>((dict as *const u8).add(0x18) as *const usize);
009101:     if entries == 0 {
009102:         return out;
009103:     }
009104:     // Il2CppArray header: 0x20 bytes, then entries
009105:     let capacity =
009106:         std::ptr::read_unaligned::<usize>((entries as *const u8).add(0x18) as *const usize);
009107:     let entries_base = entries + 0x20;
009108:     for i in 0..capacity {
009109:         let entry_addr = entries_base + i * 24;
009110:         let hash_code = std::ptr::read_unaligned::<i32>((entry_addr as *const u8) as *const i32);
009111:         if hash_code < 0 {
009112:             continue;
009113:         } // free entry
```

## Lines 9599-9653
```rust
009599:     }
009600:     let mut any = false;
009601:     if a1 != 0 {
009602:         let ok = interceptor_hook(a1, sqlcipher_key_hook as usize);
009603:         set_hook_status("meta.key_v1", if ok { "hooked" } else { "failed: interceptor_hook" });
009604:         any |= ok;
009605:     } else {
009606:         set_hook_status("meta.key_v1", "failed: resolve");
009607:     }
009608:     if a2 != 0 {
009609:         let ok = interceptor_hook(a2, sqlcipher_key_v2_hook as usize);
009610:         set_hook_status("meta.key_v2", if ok { "hooked" } else { "failed: interceptor_hook" });
009611:         any |= ok;
009612:     } else {
009613:         set_hook_status("meta.key_v2", "failed: resolve");
009614:     }
009615:     ura_log(3, &format!("sqlcipher key hook install: v1=0x{:x} v2=0x{:x} any={}", a1, a2, any));
009616: }
009617: 
009618: /// ★ v3.24.40: fuzzy variant — first method whose name CONTAINS `substr`.
009619: /// Self-heals when Cygames renames methods (e.g. CompressRequest_v2).
009620: unsafe fn find_method_fuzzy(class: *mut c_void, substr: &str) -> usize {
009621:     if class.is_null() { return 0; }
009622:     let get_methods_fn: Option<
009623:         unsafe extern "C" fn(*mut c_void, *mut *mut c_void) -> *const c_void,
009624:     > = {
009625:         let p = resolve_il2cpp_symbol("il2cpp_class_get_methods");
009626:         if p.is_null() { None } else { Some(std::mem::transmute(p)) }
009627:     };
009628:     let method_get_name_fn: Option<unsafe extern "C" fn(*const c_void) -> *const c_char> = {
009629:         let p = resolve_il2cpp_symbol("il2cpp_method_get_name");
009630:         if p.is_null() { None } else { Some(std::mem::transmute(p)) }
009631:     };
009632:     let method_get_ptr_fn: Option<unsafe extern "C" fn(*const c_void) -> *const c_void> = {
009633:         let p = resolve_il2cpp_symbol("il2cpp_method_get_pointer");
009634:         if p.is_null() { None } else { Some(std::mem::transmute(p)) }
009635:     };
009636:     if get_methods_fn.is_none() || method_get_name_fn.is_none() { return 0; }
009637:     let mut iter: *mut c_void = std::ptr::null_mut();
009638:     loop {
009639:         let mi = get_methods_fn.unwrap()(class, &mut iter);
009640:         if mi.is_null() { break; }
009641:         let name_ptr = method_get_name_fn.unwrap()(mi);
009642:         if name_ptr.is_null() { continue; }
009643:         let name = CStr::from_ptr(name_ptr).to_string_lossy();
009644:         if name.contains(substr) {
009645:             if let Some(get_ptr) = method_get_ptr_fn {
009646:                 let ptr = get_ptr(mi);
009647:                 if !ptr.is_null() {
009648:                     ura_log(3, &format!("find_method_fuzzy: {}~{} -> 0x{:x}", substr, name, ptr as usize));
009649:                     return ptr as usize;
009650:                 }
009651:             }
009652:         }
009653:     }
```

## Lines 9726-9868
```rust
009726:         set_hook_status("sniff", "failed: image_not_found");
009727:         return;
009728:     }
009729: 
009730:     // HttpHelper class (exact, then fuzzy fallback — v3.24.40)
009731:     let mut http_helper = get_class(
009732:         umamusume,
009733:         to_cstr("Gallop").as_ptr(),
009734:         to_cstr("HttpHelper").as_ptr(),
009735:     );
009736:     if http_helper.is_null() {
009737:         http_helper = find_class_fuzzy(umamusume, "HttpHelper");
009738:     }
009739:     if http_helper.is_null() {
009740:         ura_log(3, "API sniff: HttpHelper class not found");
009741:         set_hook_status("sniff", "failed: httphelper_class_not_found");
009742:         return;
009743:     }
009744:     ura_log(3, "API sniff: HttpHelper class found");
009745: 
009746:     // Hook CompressRequest
009747:     if COMPRESS_REQUEST_ADDR == 0 {
009748:         let mut addr = get_method_addr(http_helper as usize, to_cstr("CompressRequest").as_ptr(), 1);
009749:         if addr == 0 {
009750:             addr = find_method_fuzzy(http_helper, "CompressRequest");
009751:         }
009752:         if addr != 0 {
009753:             if interceptor_hook(addr, compress_request_hook_handler as usize) {
009754:                 COMPRESS_REQUEST_ADDR = addr;
009755:                 ura_log(
009756:                     3,
009757:                     &format!("API sniff: CompressRequest hooked at 0x{:x}", addr),
009758:                 );
009759:                 set_hook_status("sniff.compress", &format!("hooked@0x{:x}", addr));
009760:             } else {
009761:                 ura_log(
009762:                     3,
009763:                     &format!("API sniff: CompressRequest hook FAILED at 0x{:x}", addr),
009764:                 );
009765:                 set_hook_status("sniff.compress", "failed: interceptor_hook");
009766:             }
009767:         } else {
009768:             ura_log(3, "API sniff: CompressRequest NOT FOUND");
009769:             set_hook_status("sniff.compress", "failed: method_not_found");
009770:         }
009771:     }
009772: 
009773:     // Hook DecompressResponse
009774:     if DECOMPRESS_RESPONSE_ADDR == 0 {
009775:         let addr = get_method_addr(
009776:             http_helper as usize,
009777:             to_cstr("DecompressResponse").as_ptr(),
009778:             1,
009779:         );
009780:         if addr != 0 {
009781:             if interceptor_hook(addr, decompress_response_hook_handler as usize) {
009782:                 DECOMPRESS_RESPONSE_ADDR = addr;
009783:                 ura_log(
009784:                     3,
009785:                     &format!("API sniff: DecompressResponse hooked at 0x{:x}", addr),
009786:                 );
009787:                 set_hook_status("sniff.decompress", &format!("hooked@0x{:x}", addr));
009788:             } else {
009789:                 ura_log(
009790:                     3,
009791:                     &format!("API sniff: DecompressResponse hook FAILED at 0x{:x}", addr),
009792:                 );
009793:                 set_hook_status("sniff.decompress", "failed: interceptor_hook");
009794:             }
009795:         } else {
009796:             ura_log(3, "API sniff: DecompressResponse NOT FOUND");
009797:             set_hook_status("sniff.decompress", "failed: method_not_found");
009798:         }
009799:     }
009800: 
009801:     // Hook WWWRequest.Post (from Cute.Http.Assembly.dll)
009802:     if POST_ADDR == 0 {
009803:         let cute_http = get_asm(to_cstr("Cute.Http.Assembly.dll").as_ptr());
009804:         if !cute_http.is_null() {
009805:             let mut www_request = get_class(
009806:                 cute_http,
009807:                 to_cstr("Cute.Http").as_ptr(),
009808:                 to_cstr("WWWRequest").as_ptr(),
009809:             );
009810:             if www_request.is_null() {
009811:                 www_request = find_class_fuzzy(cute_http, "WWWRequest");
009812:             }
009813:             if !www_request.is_null() {
009814:                 let mut addr = get_method_addr(www_request as usize, to_cstr("Post").as_ptr(), 3);
009815:                 if addr == 0 {
009816:                     addr = find_method_fuzzy(www_request, "Post");
009817:                 }
009818:                 if addr != 0 {
009819:                     if interceptor_hook(addr, post_hook_handler as usize) {
009820:                         POST_ADDR = addr;
009821:                         ura_log(
009822:                             3,
009823:                             &format!("API sniff: WWWRequest.Post hooked at 0x{:x}", addr),
009824:                         );
009825:                         set_hook_status("sniff.post", &format!("hooked@0x{:x}", addr));
009826:                     } else {
009827:                         ura_log(
009828:                             3,
009829:                             &format!("API sniff: WWWRequest.Post hook FAILED at 0x{:x}", addr),
009830:                         );
009831:                         set_hook_status("sniff.post", "failed: interceptor_hook");
009832:                     }
009833:                 } else {
009834:                     ura_log(3, "API sniff: WWWRequest.Post NOT FOUND");
009835:                     set_hook_status("sniff.post", "failed: method_not_found");
009836:                 }
009837:             } else {
009838:                 ura_log(3, "API sniff: Cute.Http.WWWRequest class not found");
009839:                 set_hook_status("sniff.post", "failed: class_not_found");
009840:             }
009841:         } else {
009842:             ura_log(3, "API sniff: Cute.Http.Assembly.dll image not found");
009843:         }
009844:     }
009845: }
009846: 
009847: // ★ v3.24.2: Story event choice hook — capture career event choices
009848: // StoryChoiceController.Choice(int choiceIndex, ???)
009849: // ARM64: X0=this, W1=choiceIndex, X2=???
009850: extern "C" fn event_choice_hook_handler(
009851:     this: *mut c_void,
009852:     choice_index: i32,
009853:     _param2: *mut c_void,
009854: ) {
009855:     let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
009856:         let choices_count = {
009857:             let _lock = EVENT_STATE_MUTEX.lock();
009858:             EVENT_SELECTED_IDX = choice_index;
009859:             let choice = if choice_index >= 0 {
009860:                 EVENT_CHOICES.get(choice_index as usize).cloned()
009861:             } else { None };
009862:             if let Ok(mut pending) = EVENT_PENDING_RESULT.lock() {
009863:                 *pending = Some(PendingEventSelection {
009864:                     captured_at: sniff_timestamp(),
009865:                     generation: EVENT_GENERATION,
009866:                     story_id: EVENT_STORY_ID,
009867:                     chara_id: EVENT_CHARA_ID,
009868:                     selected_idx_raw: choice_index,
```
