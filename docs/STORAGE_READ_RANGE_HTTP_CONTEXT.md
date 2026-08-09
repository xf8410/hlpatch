# Storage read_range exact HTTP context

source_commit: `9df5239ef391ded8397ddefde15bc1cd70794382`

## lines 672-749

```rust
672: }
673: 
674: fn init_crash_handler() {
675:     unsafe {
676:         let handler = crash_signal_handler as usize;
677:         sys_signal(11, handler); // SIGSEGV
678:         sys_signal(6, handler); // SIGABRT
679:         sys_signal(7, handler); // SIGBUS
680:         sys_signal(8, handler); // SIGFPE
681:     }
682:     std::panic::set_hook(Box::new(|info| {
683:         let msg = format!("PANIC: {}\n", info);
684:         let _ = std::fs::OpenOptions::new()
685:             .create(true)
686:             .append(true)
687:             .open("/data/data/jp.pokemon.pokeuma/files/uma_predict.log")
688:             .and_then(|mut f| std::io::Write::write_all(&mut f, msg.as_bytes()));
689:     }));
690: }
691: 
692: fn log_predict_step(msg: &str) {
693:     let step = PREDICT_STEP.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
694:     let line = format!("[{}] {}\n", step, msg);
695: 
696:     // Store last step in static buffer for /debug/laststep
697:     let bytes = msg.as_bytes();
698:     let len = bytes.len().min(120);
699:     unsafe {
700:         std::ptr::copy_nonoverlapping(bytes.as_ptr(), LAST_STEP_BUF.as_mut_ptr(), len);
701:         LAST_STEP_BUF[len] = 0;
702:     }
703:     LAST_STEP_LEN.store(len as u32, std::sync::atomic::Ordering::Relaxed);
704: 
705:     // Write to file using raw libc syscalls (more reliable than std::fs on Android)
706:     let path1 = b"/data/data/jp.pokemon.pokeuma/files/uma_predict.log\0";
707:     let path2 = b"/data/local/tmp/uma_predict.log\0";
708:     let line_bytes = line.as_bytes();
709:     unsafe {
710:         let fd = sys_open(path1.as_ptr() as *const i8, 1 | 64 | 1024, 0o644);
711:         if fd >= 0 {
712:             sys_write(fd, line_bytes.as_ptr(), line_bytes.len());
713:             sys_close(fd);
714:         }
715:         let fd2 = sys_open(path2.as_ptr() as *const i8, 1 | 64 | 1024, 0o644);
716:         if fd2 >= 0 {
717:             sys_write(fd2, line_bytes.as_ptr(), line_bytes.len());
718:             sys_close(fd2);
719:         }
720:         // v3.22.51: std::fs fallback
721:         let _ = std::fs::OpenOptions::new()
722:             .create(true)
723:             .append(true)
724:             .open("/data/data/jp.pokemon.pokeuma/files/uma_predict.log")
725:             .and_then(|mut f| std::io::Write::write_all(&mut f, line.as_bytes()));
726:     }
727: }
728: 
729: fn clear_predict_log() {
730:     PREDICT_STEP.store(0, std::sync::atomic::Ordering::Relaxed);
731:     LAST_STEP_LEN.store(0, std::sync::atomic::Ordering::Relaxed);
732:     let path1 = b"/data/data/jp.pokemon.pokeuma/files/uma_predict.log\0";
733:     let path2 = b"/data/local/tmp/uma_predict.log\0";
734:     unsafe {
735:         let fd = sys_open(path1.as_ptr() as *const i8, 1 | 64 | 512, 0o644);
736:         if fd >= 0 {
737:             sys_close(fd);
738:         }
739:         let fd2 = sys_open(path2.as_ptr() as *const i8, 1 | 64 | 512, 0o644);
740:         if fd2 >= 0 {
741:             sys_close(fd2);
742:         }
743:     }
744: }
745: 
746: fn read_crash_log() -> String {
747:     let sig = CRASH_SIG.load(std::sync::atomic::Ordering::Relaxed);
748:     let step = CRASH_STEP.load(std::sync::atomic::Ordering::Relaxed);
749:     if sig != 0 {
```

## lines 811-865

```rust
811:         let pkg_raw = std::fs::read("/proc/self/cmdline").unwrap_or_default();
812:         let pkg = String::from_utf8_lossy(&pkg_raw)
813:             .trim_matches(char::from(0))
814:             .trim()
815:             .to_string();
816:         if !pkg.is_empty() {
817:             let alt = format!("/sdcard/Android/media/{}/hachimi/ura_boot.log", pkg);
818:             let _ = std::fs::write(&alt, "ura boot trace\n");
819:         }
820:         return;
821:     }
822:     let line = format!("{}\n", step);
823:     let _ = std::fs::OpenOptions::new()
824:         .create(true)
825:         .append(true)
826:         .open(&log_path)
827:         .and_then(|mut f| std::io::Write::write_all(&mut f, line.as_bytes()));
828:     // ★ v3.24.56: unbuffered black box in the ACCESSIBLE media dir
829:     // (/sdcard/Android/data is root-only on modern Android).
830:     let pkg_raw = std::fs::read("/proc/self/cmdline").unwrap_or_default();
831:     let pkg = String::from_utf8_lossy(&pkg_raw)
832:         .trim_matches(char::from(0))
833:         .trim()
834:         .to_string();
835:     if !pkg.is_empty() {
836:         let alt = format!("/sdcard/Android/media/{}/hachimi/ura_boot.log", pkg);
837:         let _ = std::fs::OpenOptions::new()
838:             .create(true)
839:             .append(true)
840:             .open(&alt)
841:             .and_then(|mut f| std::io::Write::write_all(&mut f, line.as_bytes()));
842:     }
843: }
844: 
845: // ★ v3.24.53: upload an arbitrary local file to the repo (contents API).
846: // Used for crash log and boot trace so the user can read them on GitHub web
847: // even when the game dies before HTTP is reachable.
848: fn upload_file_to_repo(local_path: &str, repo_name: &str) {
849:     let content = match std::fs::read(local_path) {
850:         Ok(c) if !c.is_empty() => c,
851:         _ => return,
852:     };
853:     let gh_token = read_github_token();
854:     if gh_token.is_empty() {
855:         return;
856:     }
857:     let b64 = base64_encode(&content);
858:     let json = format!(
859:         r#"{{"message":"auto-upload {}","content":"{}"}}"#,
860:         repo_name, b64
861:     );
862:     let tmp = "/data/data/jp.pokemon.pokeuma/files/uma_upload.json";
863:     let _ = std::fs::write(tmp, &json);
864:     let cmd = format!(
865:         "curl -s --max-time 15 -X PUT -H 'Authorization: token {}' -H 'Content-Type: application/json' -d @{} https://api.github.com/repos/xf8410/hlpatch/contents/{} >/dev/null 2>&1",
```

## lines 6651-6701

```rust
6651:     }
6652:     h
6653: }
6654: 
6655: fn push_to_app(json: &str) {
6656:     use std::io::{Read, Write};
6657:     let cfg = unsafe { get_config() };
6658:     if !cfg.push_enabled {
6659:         return;
6660:     }
6661:     let addr_str = cfg.push_addr();
6662:     let addr: std::net::SocketAddr = match addr_str.parse() {
6663:         Ok(a) => a,
6664:         Err(_) => return,
6665:     };
6666:     let mut stream =
6667:         match std::net::TcpStream::connect_timeout(&addr, std::time::Duration::from_secs(2)) {
6668:             Ok(s) => s,
6669:             Err(_) => return, // App not running, that's fine
6670:         };
6671:     let body = json.as_bytes();
6672:     let req = format!(
6673:         "POST /data HTTP/1.1\r\nHost: {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
6674:         addr_str, body.len()
6675:     );
6676:     let _ = stream.write_all(req.as_bytes());
6677:     let _ = stream.write_all(body);
6678:     let _ = stream.flush();
6679:     let mut buf = [0u8; 256];
6680:     let _ = stream.read(&mut buf);
6681: }
6682: 
6683: fn push_loop() {
6684:     let interval =
6685:         std::time::Duration::from_secs(unsafe { get_config() }.push_interval_secs.max(2));
6686:     let mut consecutive_errors: u32 = 0;
6687: 
6688:     // ★ Initial push: try pushing current data on startup
6689:     // Don't rely solely on GAME_INITIALIZED callback — it may never fire
6690:     // if the game was already initialized before the plugin loaded.
6691:     // Instead, try reading data; if it succeeds, the game is ready.
6692:     for wait_round in 0..60 {
6693:         if GAME_INITIALIZED.load(Ordering::Relaxed) {
6694:             break;
6695:         }
6696:         boot_trace("push_probe_begin");
6697:         // Try a probe read — if it doesn't error, game is ready
6698:         let probe = read_summary();
6699:         if !probe.contains("\"error\"") {
6700:             GAME_INITIALIZED.store(true, Ordering::Relaxed);
6701:             unsafe {
```

## lines 7415-7464

```rust
7415:             if carry>0 { buf.copy_within(total-carry..total,0); }
7416:             pos+=n as u64;
7417:             if n<want { failures+=1; break; }
7418:         }
7419:     }
7420:     format!(r#"{{"ok":true,"safe":true,"metadata_only":{},"maps_total":{},"maps_selected":{},"bytes_attempted":{},"bytes_scanned":{},"read_failures":{},"elapsed_ms":{},"truncated":{},"hits":{},"locations":[{}]}}"#,
7421:         metadata_only,maps.len(),selected,attempted,scanned,failures,started.elapsed().as_millis(),truncated,hits.len(),hits.join(","))
7422: }
7423: 
7424: fn safe_maps_summary() -> String {
7425:     let maps=match safe_maps(){Ok(v)=>v,Err(e)=>return format!(r#"{{"ok":false,"error":"maps_read_failed","detail":"{}"}}"#,safe_json(&e.to_string()))};
7426:     let readable=maps.iter().filter(|m|m.perms.starts_with('r')).count();
7427:     let sample=maps.iter().filter(|m|m.perms.starts_with('r')).take(64).map(|m|format!(r#"{{"start":"0x{:x}","end":"0x{:x}","size":{},"perms":"{}","path":"{}"}}"#,m.start,m.end,m.end-m.start,safe_json(&m.perms),safe_json(&m.path))).collect::<Vec<_>>().join(",");
7428:     format!(r#"{{"ok":true,"maps_total":{},"readable":{},"sample_limited":true,"maps":[{}]}}"#,maps.len(),readable,sample)
7429: }
7430: 
7431: fn handle_http(mut stream: std::net::TcpStream) {
7432:     use std::io::{Read, Write};
7433:     let mut buf = [0u8; 8192];
7434:     let n = match stream.read(&mut buf) {
7435:         Ok(n) if n > 0 => n,
7436:         _ => return,
7437:     };
7438:     let req = std::str::from_utf8(&buf[..n]).unwrap_or("");
7439:     let path = parse_path(req);
7440:     let full_uri = req
7441:         .lines()
7442:         .next()
7443:         .unwrap_or("")
7444:         .split(' ')
7445:         .nth(1)
7446:         .unwrap_or("/");
7447: 
7448:     // ★ v3.24.55: boot gate. Crash autopsy via hachimi.log: the floating app
7449:     // polls /summary during game boot; IL2CPP reads on the HTTP thread against
7450:     // transitional objects SIGSEGV the process (sigjmp recovery only exists on
7451:     // the push thread). Until the game is initialized, refuse every endpoint
7452:     // that touches game memory; static/self-state endpoints stay available.
7453:     if !GAME_INITIALIZED.load(Ordering::Relaxed) {
7454:         const BOOT_SAFE_EXACT: &[&str] = &[
7455:     "/runtime/init_status",
7456:     "/hooks/registry",
7457:     "/hooks/diagnostics",
7458:     "/capture/status",
7459:     "/storage/status",
7460:     "/storage/sessions",
7461:     "/storage/session",
7462:     "/storage/files",
7463:     "/storage/download",
7464:     "/storage/flush",
```

## lines 7508-7551

```rust
7508:             "/debug/private_file",
7509:             "/debug/mem_scan_sqlite",
7510:             "/debug/mem_scan_zdict",
7511:             "/debug/mem_scan_hex",
7512:             "/debug/file_scan_hex",
7513:             "/debug/maps_list",
7514:             "/debug/file_dl",
7515:             "/debug/file_range_hex",
7516:             "/il2cpp/read_string",
7517:             "/il2cpp/read_mem",
7518:         ];
7519:         let safe = BOOT_SAFE_EXACT.iter().any(|p| path == *p)
7520:             || BOOT_SAFE_PREFIX.iter().any(|p| path.starts_with(p));
7521:         if !safe {
7522:             let b = r#"{"status":"booting","game_initialized":false}"#;
7523:             let resp = format!(
7524:                 "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
7525:                 b.len(), b
7526:             );
7527:             let _ = stream.write_all(resp.as_bytes());
7528:             return;
7529:         }
7530:     }
7531: 
7532:     // ★ 白名单下载开关：名单内端点追加 ?dl=1 即以附件形式返回（解决手机复制长度上限）
7533:     //    ?dl=1&name=xxx 可自定义文件名（仅保留字母数字和下划线/连字符）
7534:     //    大文件仍走各专用流式 _dl 端点，避免此路径内存翻倍
7535:     const DL_ALLOWED: &[&str] = &[
7536:         "/summary",
7537:         "/scenario",
7538:         "/data",
7539:         "/ramen",
7540:         "/debug/ramen_transition",
7541:         "/api/sniff",
7542:         "/api/sniff/metadata",
7543:         "/api/sniff/diag",
7544:         "/api/event/choices",
7545:         "/api/event/observations",
7546:         "/debug/event_reward_targets",
7547:         "/debug/resource_meta_schema",
7548:         "/debug/resource_meta_probe",
7549:         "/debug/resource_crypto_symbols",
7550:         "/debug/all",
7551:         "/debug/params",
```

## lines 7554-7599

```rust
7554:         "/debug/training_partners",
7555:         "/debug/rameninfo",
7556:         "/debug/laststep",
7557:         "/debug/storydata",
7558:         "/debug/ramenfields",
7559:         "/debug/gauge",
7560:         "/debug/gauge2",
7561:         "/debug/ramengains",
7562:         "/debug/paramsincdec",
7563:         "/debug/training_seed",
7564:         "/debug/unique_skills",
7565:         "/debug/hint_gain",
7566:         "/debug/sc_effect",
7567:         "/debug/unique_detail",
7568:         "/classes",
7569:     ];
7570:     let dl_flag = parse_query(&full_uri, "dl");
7571:     let dl_name = parse_query(&full_uri, "name");
7572:     let dl_enabled = !dl_flag.is_empty() && dl_flag != "0" && DL_ALLOWED.iter().any(|p| path == *p);
7573: 
7574:     let _parsed_request_uri = parse_request_uri(req).unwrap_or_else(|_| full_uri.to_string());
7575:     let body = if path == "/debug/global_metadata_probe" {
7576:         safe_mem_scan(req, true)
7577:     } else if path == "/debug/mem_scan_hex" {
7578:         safe_mem_scan(req, false)
7579:     } else if path == "/debug/mem_maps" {
7580:         safe_maps_summary()
7581:     } else if path == "/" || path == "/health" {
7582:         format!(
7583:             r#"{{"status":"ok","version":"{}","endpoints":[\"/inherit/tree\",\"/inherit/parent_records\",\"/inherit/race_history\",\"/inherit/race_compat\",\"/inherit/full_compat\",\"/inherit/compat_trace\",\"/inherit/factor_tree\",\"/inherit/bonus_params\",\"/inherit/event_trace\",\"/inherit/deck_runtime\",\"/inherit/deck_validate\",\"/inherit/friend_rental_context\",\"/inherit/auto_select_trace\",\"/autoplay/runtime\",\"/autoplay/plan\",\"/autoplay/action_trace\",\"/autoplay/factor_select_trace\",\"/offline_auto/runtime\",\"/offline_auto/start_request\",\"/offline_auto/race_reserve\",\"/offline_auto/result\",\"/generate_succession/status\",\"/generate_succession/limits\",\"/generate_succession/request\",\"/generate_succession/result\",\"/generate_succession/candidates\",\"/generate_succession/race_reserve\",\"/generate_succession/race_validation\",\"/generate_succession/factor_priority\",\"/generate_succession/factor_order\",\"/generate_succession/probability_trace\",\"/generate_succession/cost_trace\",\"/factor/finish_trace\",\"/factor/candidates\",\"/factor/roll_trace\",\"/factor/probability_model\",\"/factor/history\",\"/factor/stats\",\"/factor/breeding_advice\",\"/il2cpp/call_targets\",\"/il2cpp/callers\",\"/il2cpp/type_detail\",\"/il2cpp/object_dump\",\"/api/sniff/exchanges\",\"/api/sniff/exchange\",\"/api/hook/install\",\"/api/hook/remove\",\"/api/hook/list\",\"/api/hook/events\",\"/storage/files\",\"/storage/download\","/storage/status","/storage/sessions","/storage/session","/storage/flush","/storage/recover","/il2cpp/method_index_status","/il2cpp/method_by_addr","/il2cpp/method_detail","/il2cpp/nested_types","/il2cpp/enum_values","/inherit/pair_compat","/inherit/selected_parent_runtime","/summary","/data","/scenario","/debug/rameninfo","/debug/laststep","/event/recommend","/inherit/compat","/saddle-analysis","/log/turn","/debug/params","/debug/breeders","/debug/cmdinfo","/debug/training_partners","/debug/crashlog","/debug/upload","/debug/dumpclass","/debug/storydata","/debug/ramenfields","/debug/gauge","/debug/gauge2","/debug/ramengains","/debug/paramsincdec","/debug/training_seed","/debug/training_log","/debug/training_log_dl","/update","/update/status","/debug/all","/debug/unique_skills","/debug/mdb_all_tables","/debug/mdb_schema_dump","/debug/hint_gain","/debug/sc_effect","/debug/unique_detail","/debug/table","/debug/push_table","/debug/download_table","/mdb","/carddb","/skilldata","/hall","/saddles","/saddles-dl","/log","/status","/health","/mdb/schema","/mdb/search","/mdb/raw","/mdb/dl_batch","/il2cpp/dump","/il2cpp/call","/il2cpp/tree","/il2cpp/field","/il2cpp/classes","/il2cpp/static","/il2cpp/methods","/il2cpp/disassemble","/il2cpp/disassemble_dl","/il2cpp/disassemble_addr","/il2cpp/disassemble_addr_dl","/il2cpp/dump_all_methods","/il2cpp/dump_all_methods_dl","/il2cpp/search_float","/il2cpp/search_float_dl","/il2cpp/search_int","/il2cpp/search_int_dl","/il2cpp/search_methods","/il2cpp/search_methods_dl","/il2cpp/read_mem","/il2cpp/read_mem_dl","/training/result","/api/sniff","/api/sniff/metadata","/api/sniff/status","/api/sniff/toggle","/api/sniff/clear","/api/sniff/diag","/api/event/choices","/api/event/clear","/debug/hooklog","/debug/hookdiag","/debug/resource_meta_key","/debug/resource_db_keys","/debug/resource_reads","/debug/mem_scan_sqlite","/debug/meta_dump","/action/latest","/seed/history","/seed/stats","/debug/ramen_planner_state","/debug/ramen_participants","/debug/ramen_transition","/debug/ramen_dataset_path","/debug/ramen_formula_targets","/debug/event_reward_targets", "/debug/resource_storage","/debug/resource_meta_schema","/debug/resource_meta_probe", "/debug/resource_crypto_symbols","/debug/resource_meta_dl","/debug/resource_file_dl","/debug/private_file_inventory","/debug/private_file_dl"]}}"#,
7584:             PLUGIN_VERSION
7585:         )
7586:     } else if path == "/scan" {
7587:         unsafe { scan_il2cpp_classes() }
7588:     } else if path == "/data" {
7589:         let result = unsafe { read_training_data() };
7590:         unsafe {
7591:             log_snapshot("data", &result);
7592:         }
7593:         result
7594:     } else if path == "/status" {
7595:         format!(
7596:             r#"{{"game_initialized":{},"http_running":{}}}"#,
7597:             GAME_INITIALIZED.load(Ordering::Relaxed),
7598:             HTTP_RUNNING.load(Ordering::Relaxed)
7599:         )
```

## lines 7745-7785

```rust
7745:                 .filter(|m| m.direction == "response")
7746:                 .count();
7747:             format!(
7748:                 r#"{{"enabled":{},"raw_request_count":{},"raw_response_count":{},"metadata_count":{},"request_count":{},"response_count":{},"last_id":{},"raw_limit":{},"metadata_limit":{}}}"#,
7749:                 SNIFF_ENABLED.load(Ordering::Relaxed),
7750:                 SNIFF_REQUESTS.len(),
7751:                 SNIFF_RESPONSES.len(),
7752:                 SNIFF_METADATA.len(),
7753:                 request_count,
7754:                 response_count,
7755:                 last_id,
7756:                 SNIFF_RAW_MAX,
7757:                 SNIFF_METADATA_MAX
7758:             )
7759:         }
7760:     } else if path == "/api/sniff/metadata" {
7761:         let after_id = parse_query(&full_uri, "after_id")
7762:             .parse::<u64>()
7763:             .unwrap_or(0);
7764:         let _lock = SNIFF_MUTEX.lock();
7765:         unsafe {
7766:             let entries: Vec<String> = SNIFF_METADATA.iter()
7767:                 .filter(|m| m.id > after_id)
7768:                 .map(|m| {
7769:                     let headers_json: String = m.headers.iter()
7770:                         .map(|(k, v)| format!(r#"{{"key":"{}","value":"{}"}}"#, json_escape(k), json_escape(v)))
7771:                         .collect::<Vec<String>>()
7772:                         .join(",");
7773:                     format!(r#"{{"id":{},"request_id":{},"timestamp_ms":{},"direction":"{}","path":"{}","size":{},"body_hex":"{}","headers":[{}]}}"#,
7774:                         m.id, m.request_id, m.timestamp_ms, m.direction, json_escape(&m.path), m.size, m.body_hex, headers_json)
7775:                 })
7776:                 .collect();
7777:             let last_id = SNIFF_METADATA.last().map(|m| m.id).unwrap_or(after_id);
7778:             format!(
7779:                 r#"{{"enabled":{},"after_id":{},"last_id":{},"count":{},"entries":[{}]}}"#,
7780:                 SNIFF_ENABLED.load(Ordering::Relaxed),
7781:                 after_id,
7782:                 last_id,
7783:                 entries.len(),
7784:                 entries.join(",")
7785:             )
```

## lines 7790-7895

```rust
7790:             install_api_sniff_hooks();
7791:         }
7792:         // ★ If hooks installed successfully, game is ready — set GAME_INITIALIZED
7793:         let any_hooked = unsafe {
7794:             COMPRESS_REQUEST_ADDR != 0
7795:                 || DECOMPRESS_RESPONSE_ADDR != 0
7796:                 || POST_ADDR != 0
7797:                 || MAKEMD5_ADDR != 0
7798:                 || COMPUTEHASH_ADDR != 0
7799:         };
7800:         if any_hooked && !GAME_INITIALIZED.load(Ordering::Relaxed) {
7801:             GAME_INITIALIZED.store(true, Ordering::Relaxed);
7802:             unsafe {
7803:                 ura_log(3, "sniff/toggle: GAME_INITIALIZED set (hooks installed via toggle)");
7804:             }
7805:         }
7806:         let requested = parse_query(&full_uri, "enabled");
7807:         let new_val = match requested.as_str() {
7808:             "1" | "true" => true,
7809:             "0" | "false" => false,
7810:             _ => !SNIFF_ENABLED.load(Ordering::Relaxed),
7811:         };
7812:         SNIFF_ENABLED.store(new_val, Ordering::Relaxed);
7813:         let req_hooked = unsafe { COMPRESS_REQUEST_ADDR != 0 };
7814:         let resp_hooked = unsafe { DECOMPRESS_RESPONSE_ADDR != 0 };
7815:         let post_hooked = unsafe { POST_ADDR != 0 };
7816:         format!(
7817:             r#"{{"sniff_enabled":{},"compress_hooked":{},"decompress_hooked":{},"post_hooked":{}}}"#,
7818:             new_val, req_hooked, resp_hooked, post_hooked
7819:         )
7820:     } else if path == "/api/sniff/clear" {
7821:         let _lock = SNIFF_MUTEX.lock();
7822:         unsafe {
7823:             SNIFF_REQUESTS.clear();
7824:             SNIFF_RESPONSES.clear();
7825:             if let Ok(mut entries) = UNITY_OBSERVATIONS.lock() {
7826:                 entries.clear();
7827:             }
7828:             if let Ok(mut completed) = UNITY_COMPLETED_RESPONSE_HEADERS.lock() {
7829:                 completed.clear();
7830:             }
7831:             SNIFF_METADATA.clear();
7832:             SNIFF_RESPONSE_QUEUE.clear();
7833:             PENDING_REQ_BODY = None;
7834:         }
7835:         r#"{"ok":true}"#.to_string()
7836:     } else if path.starts_with("/debug/hooklog") {
7837:         // ★ v3.24.40/42: last HOOK_LOG_MAX lines, optional ?filter=substr
7838:         let filter = parse_query(&full_uri, "filter");
7839:         let entries: Vec<String> = match HOOK_LOG.lock() {
7840:             Ok(g) => g
7841:                 .iter()
7842:                 .filter(|l| filter.is_empty() || l.contains(&filter))
7843:                 .map(|l| json_escape(l))
7844:                 .collect(),
7845:             Err(_) => Vec::new(),
7846:         };
7847:         format!(
7848:             r#"{{"count":{},"entries":[{}]}}"#,
7849:             entries.len(),
7850:             entries.join(",")
7851:         )
7852:     } else if path == "/debug/resource_reads" {
7853:         // ★ v3.24.58: meta/dat file-read trace. Lazy-starts the /proc watcher
7854:         // on first request (never at init — thread spawn in init context).
7855:         start_res_fd_watcher();
7856:         let entries: Vec<String> = match RES_READ_LOG.lock() {
7857:             Ok(g) => g
7858:                 .iter()
7859:                 .map(|l| format!("\"{}\"", json_escape(l)))
7860:                 .collect(),
7861:             Err(_) => Vec::new(),
7862:         };
7863:         format!(
7864:             r#"{{"count":{},"entries":[{}]}}"#,
7865:             entries.len(),
7866:             entries.join(",")
7867:         )
7868:     } else if path.starts_with("/debug/mem_scan_sqlite") {
7869:         // ★ v3.24.58: hunt plaintext "SQLite format 3" pages in process memory
7870:         // — any custom decryption MUST materialize this in RAM.
7871:         let max_hits: usize = parse_query(&full_uri, "max").parse().unwrap_or(8);
7872:         let mut hits: Vec<String> = Vec::new();
7873:         let needle = b"SQLite format 3 ";
7874:         if let Ok(maps) = std::fs::read_to_string("/proc/self/maps") {
7875:             let mem = std::fs::File::open("/proc/self/mem");
7876:             use std::os::unix::fs::FileExt;
7877:             if let Ok(mem) = mem {
7878:                 'outer: for line in maps.lines() {
7879:                     let cols: Vec<&str> = line.split_whitespace().collect();
7880:                     if cols.len() < 6 {
7881:                         continue;
7882:                     }
7883:                     if !cols[1].contains("rw") {
7884:                         continue;
7885:                     }
7886:                     let range: Vec<&str> = cols[0].split('-').collect();
7887:                     if range.len() != 2 {
7888:                         continue;
7889:                     }
7890:                     let (Ok(sa), Ok(ea)) = (
7891:                         usize::from_str_radix(range[0], 16),
7892:                         usize::from_str_radix(range[1], 16),
7893:                     ) else {
7894:                         continue;
7895:                     };
```

## lines 7920-7960

```rust
7920:                 }
7921:             }
7922:         }
7923:         format!(
7924:             r#"{{"needle":"SQLite format 3","hits":{},"locations":[{}]}}"#,
7925:             hits.len(),
7926:             hits.iter()
7927:                 .map(|h| format!("\"{}\"", h))
7928:                 .collect::<Vec<_>>()
7929:                 .join(",")
7930:         )
7931:     } else if path == "/debug/mem_scan_zdict" {
7932:         // ★ v3.24.63: hunt zstd dictionary magic (37 A4 30 EC) in ALL readable
7933:         // memory regions (incl. r-- rodata of .so files). Each hit dumps 256KB
7934:         // of context to the media dir for offline inspection.
7935:         let needle = [0x37u8, 0xa4, 0x30, 0xec];
7936:         let max_hits: usize = parse_query(&full_uri, "max").parse().unwrap_or(4);
7937:         let mut hits: Vec<String> = Vec::new();
7938:         if let Ok(maps) = std::fs::read_to_string("/proc/self/maps") {
7939:             if let Ok(mem) = std::fs::File::open("/proc/self/mem") {
7940:                 use std::os::unix::fs::FileExt;
7941:                 'outer: for line in maps.lines() {
7942:                     let cols: Vec<&str> = line.split_whitespace().collect();
7943:                     if cols.len() < 2 {
7944:                         continue;
7945:                     }
7946:                     if !cols[1].starts_with('r') {
7947:                         continue;
7948:                     }
7949:                     let range: Vec<&str> = cols[0].split('-').collect();
7950:                     if range.len() != 2 {
7951:                         continue;
7952:                     }
7953:                     let (Ok(sa), Ok(ea)) = (
7954:                         usize::from_str_radix(range[0], 16),
7955:                         usize::from_str_radix(range[1], 16),
7956:                     ) else {
7957:                         continue;
7958:                     };
7959:                     let len = ea - sa;
7960:                     if len < 4096 || len > 1024 * 1024 * 1024 {
```

## lines 7992-8048

```rust
7992:                         }
7993:                         off += chunk;
7994:                     }
7995:                 }
7996:             }
7997:         }
7998:         format!(
7999:             r#"{{"needle":"37a430ec","hits":{},"locations":[{}],"note":"raw-content dicts have no magic; if 0 hits use /debug/mem_scan_hex"}}"#,
8000:             hits.len(),
8001:             hits.iter()
8002:                 .map(|h| format!("\"{}\"", json_escape(h)))
8003:                 .collect::<Vec<_>>()
8004:                 .join(",")
8005:         )
8006:     } else if path.starts_with("/debug/mem_scan_hex") {
8007:         // ★ v3.24.63: arbitrary <=32B hex pattern scan across readable maps
8008:         let hexq = parse_query(&full_uri, "hex");
8009:         let mut needle: Vec<u8> = Vec::new();
8010:         let hb = hexq.as_bytes();
8011:         let mut i = 0;
8012:         while i + 1 < hb.len() && needle.len() < 32 {
8013:             if let Ok(b) = u8::from_str_radix(&hexq[i..i + 2], 16) {
8014:                 needle.push(b);
8015:             }
8016:             i += 2;
8017:         }
8018:         let max_hits: usize = parse_query(&full_uri, "max").parse().unwrap_or(8);
8019:         let mut hits: Vec<String> = Vec::new();
8020:         if needle.is_empty() {
8021:             let body = r#"{"error":"empty_needle","usage":"/debug/mem_scan_hex?hex=37a430ec"}"#
8022:                 .to_string();
8023:             let resp = format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", body.len(), body);
8024:             let _ = stream.write_all(resp.as_bytes());
8025:             return;
8026:         }
8027:         if let Ok(maps) = std::fs::read_to_string("/proc/self/maps") {
8028:             if let Ok(mem) = std::fs::File::open("/proc/self/mem") {
8029:                 use std::os::unix::fs::FileExt;
8030:                 'outer: for line in maps.lines() {
8031:                     let cols: Vec<&str> = line.split_whitespace().collect();
8032:                     if cols.len() < 2 || !cols[1].starts_with('r') {
8033:                         continue;
8034:                     }
8035:                     let range: Vec<&str> = cols[0].split('-').collect();
8036:                     if range.len() != 2 {
8037:                         continue;
8038:                     }
8039:                     let (Ok(sa), Ok(ea)) = (
8040:                         usize::from_str_radix(range[0], 16),
8041:                         usize::from_str_radix(range[1], 16),
8042:                     ) else {
8043:                         continue;
8044:                     };
8045:                     let len = ea - sa;
8046:                     if len < 4096 || len > 1024 * 1024 * 1024 {
8047:                         continue;
8048:                     }
```

## lines 8073-8124

```rust
8073:                     }
8074:                 }
8075:             }
8076:         }
8077:         format!(
8078:             r#"{{"hits":{},"locations":[{}]}}"#,
8079:             hits.len(),
8080:             hits.iter()
8081:                 .map(|h| format!("\"{}\"", json_escape(h)))
8082:                 .collect::<Vec<_>>()
8083:                 .join(",")
8084:         )
8085:     } else if path.starts_with("/debug/file_scan_hex") {
8086:         // ★ v3.24.64: scan device files for a hex pattern.
8087:         // path= empty -> scan every file-backed .so/.dat region listed in maps (dedup).
8088:         // Reports file offset hits with 24 bytes of trailing context.
8089:         let hexq = parse_query(&full_uri, "hex");
8090:         let mut needle: Vec<u8> = Vec::new();
8091:         let hb = hexq.as_bytes();
8092:         let mut i = 0;
8093:         while i + 1 < hb.len() && needle.len() < 64 {
8094:             if let Ok(b) = u8::from_str_radix(&hexq[i..i + 2], 16) {
8095:                 needle.push(b);
8096:             }
8097:             i += 2;
8098:         }
8099:         let max_hits: usize = parse_query(&full_uri, "max").parse().unwrap_or(8);
8100:         let pathq = parse_query(&full_uri, "path");
8101:         let mut targets: Vec<String> = Vec::new();
8102:         if !pathq.is_empty() {
8103:             targets.push(pathq);
8104:         } else if let Ok(maps) = std::fs::read_to_string("/proc/self/maps") {
8105:             for line in maps.lines() {
8106:                 let cols: Vec<&str> = line.split_whitespace().collect();
8107:                 if let Some(name) = cols.get(5) {
8108:                     if (name.ends_with(".so") || name.ends_with(".apk") || name.contains("/dat/"))
8109:                         && !targets.iter().any(|t| t == name)
8110:                     {
8111:                         targets.push(name.to_string());
8112:                     }
8113:                 }
8114:             }
8115:         }
8116:         let mut hits: Vec<String> = Vec::new();
8117:         if needle.is_empty() {
8118:             hits.push("error: empty needle, use ?hex=37a430ec".to_string());
8119:         } else {
8120:             use std::io::Read;
8121:             'files: for t in &targets {
8122:                 if let Ok(mut f) = std::fs::File::open(t) {
8123:                     let mut fbuf: Vec<u8> = Vec::new();
8124:                     if f.read_to_end(&mut fbuf).is_err() {
```

## lines 8135-8202

```rust
8135:                         }
8136:                     }
8137:                 }
8138:             }
8139:         }
8140:         format!(
8141:             r#"{{"targets":{},"hits":{},"locations":[{}]}}"#,
8142:             targets.len(),
8143:             hits.len(),
8144:             hits.iter()
8145:                 .map(|h| format!("\"{}\"", json_escape(h)))
8146:                 .collect::<Vec<_>>()
8147:                 .join(",")
8148:         )
8149:     } else if path == "/debug/maps_list" {
8150:         // ★ v3.24.65: list file-backed memory maps (find libzstd / codec hosts)
8151:         let filter = parse_query(&full_uri, "filter");
8152:         let mut out: Vec<String> = Vec::new();
8153:         if let Ok(maps) = std::fs::read_to_string("/proc/self/maps") {
8154:             for line in maps.lines() {
8155:                 let cols: Vec<&str> = line.split_whitespace().collect();
8156:                 if let Some(name) = cols.get(5) {
8157:                     if name.starts_with('/') && (filter.is_empty() || name.contains(&filter)) {
8158:                         let e = format!("{} {}", cols[0], name);
8159:                         if !out.contains(&e) {
8160:                             out.push(e);
8161:                         }
8162:                     }
8163:                 }
8164:             }
8165:         }
8166:         format!(
8167:             r#"{{"count":{},"maps":[{}]}}"#,
8168:             out.len(),
8169:             out.iter()
8170:                 .map(|h| format!("\"{}\"", json_escape(h)))
8171:                 .collect::<Vec<_>>()
8172:                 .join(",")
8173:         )
8174:     } else if path.starts_with("/debug/file_range_hex") {
8175:         // ★ v3.24.67: read a byte range of a maps-listed file, return hex (chunked RE)
8176:         let want = parse_query(&full_uri, "path");
8177:         let off_str = parse_query(&full_uri, "offset");
8178:         let len_str = parse_query(&full_uri, "len");
8179:         let off = usize::from_str_radix(off_str.trim_start_matches("0x"), 16).unwrap_or(0);
8180:         let max_len: usize = len_str.parse().unwrap_or(65536).min(4 * 1024 * 1024);
8181:         let mut allowed = false;
8182:         if !want.is_empty() {
8183:             if let Ok(maps) = std::fs::read_to_string("/proc/self/maps") {
8184:                 for line in maps.lines() {
8185:                     let cols: Vec<&str> = line.split_whitespace().collect();
8186:                     if cols.get(5).copied() == Some(want.as_str()) {
8187:                         allowed = true;
8188:                         break;
8189:                     }
8190:                 }
8191:             }
8192:         }
8193:         if !allowed {
8194:             format!(r#"{{"error":"not_in_maps"}}"#)
8195:         } else {
8196:             use std::io::{Read, Seek, SeekFrom};
8197:             match std::fs::File::open(&want) {
8198:                 Ok(mut f) => {
8199:                     let mut buf = vec![0u8; max_len];
8200:                     let got = f
8201:                         .seek(SeekFrom::Start(off as u64))
8202:                         .and_then(|_| f.read(&mut buf))
```

## lines 8247-8287

```rust
8247:                 .map(|(n, st)| {
8248:                     format!(
8249:                         r#"{{"hook":"{}","status":"{}"}}"#,
8250:                         json_escape(n),
8251:                         json_escape(st)
8252:                     )
8253:                 })
8254:                 .collect(),
8255:             Err(_) => Vec::new(),
8256:         };
8257:         format!(
8258:             r#"{{"game_initialized":{},"hooks":[{}]}}"#,
8259:             GAME_INITIALIZED.load(Ordering::Relaxed),
8260:             items.join(",")
8261:         )
8262:     } else if path.starts_with("/api/sniff/unity") {
8263:         let after_id = parse_query(&full_uri, "after_id")
8264:             .parse::<u64>()
8265:             .unwrap_or(0);
8266:         let entries = UNITY_OBSERVATIONS.lock().map(|g| {
8267:         g.iter().filter(|x| x.id > after_id).map(|x| format!(
8268:             r#"{{"id":{},"timestamp_ms":{},"method":"{}","path":"{}","body_size":{},"body_hex":"{}","content_type":"{}"}}"#,
8269:             x.id, x.timestamp_ms, json_escape(&x.method), json_escape(&x.path),
8270:             x.body_size, x.body_hex, json_escape(&x.content_type)
8271:         )).collect::<Vec<_>>()
8272:     }).unwrap_or_default();
8273:         format!(
8274:             r#"{{"enabled":{},"unity_send_hooked":{},"count":{},"entries":[{}]}}"#,
8275:             SNIFF_ENABLED.load(Ordering::Relaxed),
8276:             unsafe { UNITY_SEND_ADDR != 0 },
8277:             entries.len(),
8278:             entries.join(",")
8279:         )
8280:     } else if path == "/api/sniff/diag" {
8281:         // v3.23.3: Diagnostic endpoint for hook installation (Interceptor API)
8282:         let req_hooked = unsafe { COMPRESS_REQUEST_ADDR != 0 };
8283:         let resp_hooked = unsafe { DECOMPRESS_RESPONSE_ADDR != 0 };
8284:         let post_hooked = unsafe { POST_ADDR != 0 };
8285:         let req_addr = unsafe { COMPRESS_REQUEST_ADDR };
8286:         let resp_addr = unsafe { DECOMPRESS_RESPONSE_ADDR };
8287:         let post_addr = unsafe { POST_ADDR };
```

## lines 8440-8480

```rust
8440:                 format!(r#"{{"label":"{}","gain_id":{},"next_block_idx":{},"loop_exit_gain_id":{}}}"#,
8441:                     c.label.replace('\\', "\\\\").replace('"', "\\\""),
8442:                     c.gain_id, c.next_block_idx, c.loop_exit_gain_id)
8443:             }).collect();
8444:             let result = format!(
8445:                 r#"{{"generation":{},"story_id":{},"chara_id":{},"selected_idx":{},"choices":[{}]}}"#,
8446:                 EVENT_GENERATION,
8447:                 EVENT_STORY_ID,
8448:                 EVENT_CHARA_ID,
8449:                 EVENT_SELECTED_IDX,
8450:                 choices_json.join(",")
8451:             );
8452:             drop(_lock);
8453:             result
8454:         }
8455:     } else if path == "/api/event/observations" {
8456:         let after_id = parse_query(&full_uri, "after_id")
8457:             .parse::<i64>()
8458:             .unwrap_or(0);
8459:         match EVENT_OBSERVATIONS.lock() {
8460:             Ok(v) => {
8461:                 let selected: Vec<String> = v
8462:                     .iter()
8463:                     .filter(|item| {
8464:                         extract_json_int(item, "\"observation_id\"").unwrap_or(0) > after_id
8465:                     })
8466:                     .cloned()
8467:                     .collect();
8468:                 format!(
8469:                     r#"{{"schema_version":2,"source":"runtime_observation","count":{},"observations":[{}]}}"#,
8470:                     selected.len(),
8471:                     selected.join(",")
8472:                 )
8473:             }
8474:             Err(_) => r#"{"error":"lock_error","observations":[]}"#.to_string(),
8475:         }
8476:     } else if path == "/api/event/observations/clear" {
8477:         if let Ok(mut v) = EVENT_OBSERVATIONS.lock() {
8478:             v.clear();
8479:         }
8480:         r#"{"ok":true,"cleared":"observations"}"#.to_string()
```

## lines 8547-8588

```rust
8547:         .unwrap_or_else(|_| r#"{"error":"ramen_dataset_path_panic"}"#.to_string())
8548:     } else if path == "/debug/ramen_planner_state" {
8549:         std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
8550:             debug_ramen_planner_state()
8551:         }))
8552:         .unwrap_or_else(|_| r#"{"error":"ramen_planner_state_panic"}"#.to_string())
8553:     } else if path == "/debug/ramen_region_select" {
8554:         std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
8555:             debug_ramen_region_select()
8556:         }))
8557:         .unwrap_or_else(|_| r#"{"error":"ramen_region_select_panic"}"#.to_string())
8558:     } else if path == "/debug/race_random_program_exact" {
8559:         unsafe { debug_race_random_program_exact() }
8560:     } else if path.starts_with("/debug/dumpclass") {
8561:         // v3.22.51: Dump all fields of any IL2CPP class by name
8562:         // Usage: /debug/dumpclass?name=WorkSingleModeData
8563:         let class_name = if let Some(q) = full_uri.find("?name=") {
8564:             &full_uri[q + 6..]
8565:         } else {
8566:             ""
8567:         };
8568:         unsafe { debug_dumpclass(class_name) }
8569:     } else if path == "/debug/storydata" {
8570:         // v3.22.35: Discover all DataSet getters, find story/event related arrays
8571:         std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
8572:             debug_storydata()
8573:         }))
8574:         .unwrap_or_else(|_| r#"{"error":"storydata_panic"}"#.to_string())
8575:     } else if path == "/debug/all" {
8576:         // ★ v3.22.35: Aggregate all debug data in one call — summary + scenario + storydata + cmdinfo + rameninfo
8577:         unsafe { debug_all() }
8578:     } else if path == "/debug/ramenfields" {
8579:         // v3.22.51: Dump all ramen array element classes + their fields at runtime
8580:         std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
8581:             debug_ramenfields()
8582:         }))
8583:         .unwrap_or_else(|_| r#"{"error":"ramenfields_panic"}"#.to_string())
8584:     } else if path == "/debug/gauge" {
8585:         // ★ v3.22.39: sigsetjmp + READ_MUTEX protection — prevent game crash on SIGSEGV
8586:         let _lock = READ_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
8587:         let jmp_result = unsafe { sys_sigsetjmp(SIGSEGV_JMP_BUF.as_mut_ptr(), 1) };
8588:         if jmp_result != 0 {
```

## lines 8671-8799

```rust
8671:         }
8672:     } else if path == "/events" {
8673:         read_events_data()
8674:     } else if path == "/debug/unique_skills" {
8675:         debug_unique_skills()
8676:     } else if path == "/debug/mdb_all_tables" {
8677:         debug_mdb_all_tables()
8678:     } else if path == "/debug/mdb_schema_dump" {
8679:         debug_mdb_schema_dump()
8680:     } else if path == "/debug/hint_gain" {
8681:         debug_hint_gain()
8682:     } else if path == "/debug/sc_effect" {
8683:         debug_sc_effect()
8684:     } else if path == "/debug/unique_detail" {
8685:         debug_unique_detail()
8686:     } else if path == "/debug/table" {
8687:         let table_name = if let Some(q) = full_uri.find("?name=") {
8688:             let rest = &full_uri[q + 6..];
8689:             rest.split('&').next().unwrap_or(rest)
8690:         } else {
8691:             ""
8692:         };
8693:         let limit = if let Some(q) = full_uri.find("limit=") {
8694:             full_uri[q + 6..]
8695:                 .split('&')
8696:                 .next()
8697:                 .unwrap_or("100")
8698:                 .parse::<usize>()
8699:                 .unwrap_or(100)
8700:         } else {
8701:             100usize
8702:         };
8703:         let offset = if let Some(q) = full_uri.find("offset=") {
8704:             full_uri[q + 7..]
8705:                 .split("&")
8706:                 .next()
8707:                 .unwrap_or("0")
8708:                 .parse::<usize>()
8709:                 .unwrap_or(0)
8710:         } else {
8711:             0usize
8712:         };
8713:         debug_table_query(table_name, limit.min(1000).max(1), offset)
8714:     } else if path == "/debug/download_table" {
8715:         let table_name = if let Some(q) = full_uri.find("?name=") {
8716:             let rest = &full_uri[q + 6..];
8717:             rest.split('&').next().unwrap_or(rest)
8718:         } else {
8719:             ""
8720:         };
8721:         let batch = if let Some(q) = full_uri.find("batch=") {
8722:             full_uri[q + 6..]
8723:                 .split('&')
8724:                 .next()
8725:                 .unwrap_or("500")
8726:                 .parse::<usize>()
8727:                 .unwrap_or(500)
8728:         } else {
8729:             500usize
8730:         };
8731:         debug_download_table(table_name, batch.min(1000).max(1))
8732:     } else if path == "/debug/push_table" {
8733:         let table_name = if let Some(q) = full_uri.find("?name=") {
8734:             let rest = &full_uri[q + 6..];
8735:             rest.split('&').next().unwrap_or(rest)
8736:         } else {
8737:             ""
8738:         };
8739:         let batch = if let Some(q) = full_uri.find("batch=") {
8740:             full_uri[q + 6..]
8741:                 .split('&')
8742:                 .next()
8743:                 .unwrap_or("500")
8744:                 .parse::<usize>()
8745:                 .unwrap_or(500)
8746:         } else {
8747:             500usize
8748:         };
8749:         let offset = if let Some(q) = full_uri.find("offset=") {
8750:             full_uri[q + 7..]
8751:                 .split('&')
8752:                 .next()
8753:                 .unwrap_or("0")
8754:                 .parse::<usize>()
8755:                 .unwrap_or(0)
8756:         } else {
8757:             0usize
8758:         };
8759:         debug_push_table(table_name, batch.min(1000).max(1), offset)
8760:     } else if path == "/tables" {
8761:         read_mdb_tables()
8762:     } else if path == "/carddb" {
8763:         read_carddb()
8764:     } else if path == "/skilldata" {
8765:         read_skilldata()
8766:     } else if path == "/hall" {
8767:         unsafe { read_hall_data() }
8768:     } else if path == "/event/recommend" {
8769:         unsafe { read_event_recommend() }
8770:     } else if path == "/inherit/selected_parent_records" {
8771:         unsafe { inherit_selected_parent_records_endpoint() }
8772:     } else if path == "/inherit/selected_parent_runtime" {
8773:         unsafe { inherit_selected_parent_runtime_endpoint() }
8774:     } else if path == "/inherit/pair_compat" {
8775:         inherit_pair_compat_endpoint(&full_uri)
8776:     } else if path == "/inherit/compat" {
8777:         r#"{"ok":false,"status":"deprecated","error":"legacy_inherit_contract_unreliable","replacement":"/inherit/pair_compat and /inherit/selected_parent_runtime"}"#.to_string()
8778:     } else if path == "/saddle-analysis" {
8779:         r#"{"ok":false,"status":"unavailable","error":"legacy_saddle_runtime_chain_unverified"}"#.to_string()
8780:     } else if path == "/log/turn" {
8781:         unsafe { read_turn_log() }
8782:     } else if path == "/ranking" {
8783:         unsafe { read_ranking_data() }
8784:     } else if path == "/saddles-dl" {
8785:         read_saddles()
8786:     } else if path == "/saddles" {
8787:         read_saddles()
8788:     } else if path == "/config" {
8789:         let is_post = req.starts_with("POST");
8790:         if is_post {
8791:             // Parse body from request
8792:             let body_start = req.find("\r\n\r\n").map(|i| i + 4).unwrap_or(req.len());
8793:             let post_body = &req[body_start..];
8794:             if let Some(new_cfg) = PluginConfig::from_json(post_body) {
8795:                 let json = new_cfg.to_json();
8796:                 unsafe {
8797:                     update_config(new_cfg);
8798:                 }
8799:                 unsafe {
```

## lines 8825-9369

```rust
8825:             if cfg.push_enabled { "checked" } else { "" },
8826:             if cfg.http_enabled { "checked" } else { "" }
8827:         );
8828:         // Return HTML with text/html content type (handled below)
8829:         html
8830:     } else if path.starts_with("/classes") {
8831:         let search = if path == "/classes" || path == "/classes/" {
8832:             ""
8833:         } else {
8834:             path.strip_prefix("/classes/search/")
8835:                 .or_else(|| path.strip_prefix("/classes/"))
8836:                 .unwrap_or("")
8837:         };
8838:         unsafe { enumerate_all_classes(search) }
8839:     } else if path.starts_with("/mdb/schema") {
8840:         // v3.22.89: 表结构
8841:         let table_name = parse_query(&full_uri, "name");
8842:         mdb_schema(&table_name)
8843:     } else if path.starts_with("/mdb/search") {
8844:         // v3.22.89: 搜索表名和列名
8845:         let keyword = parse_query(&full_uri, "keyword");
8846:         mdb_search(&keyword)
8847:     } else if path.starts_with("/mdb/raw") {
8848:         // v3.22.89: 执行只读SQL
8849:         let sql = parse_query(&full_uri, "sql");
8850:         mdb_raw_query(&sql)
8851:     } else if path.starts_with("/mdb/dl_batch") {
8852:         // ★ 按首字母批量下载 MDB 表数据为 JSON 文件
8853:         // /mdb/dl_batch?prefix=a  → 下载所有 a 开头的表
8854:         // /mdb/dl_batch?prefix=all → 下载全部表（可能很大）
8855:         let prefix = parse_query(&full_uri, "prefix");
8856:         let body = mdb_dl_batch(&prefix);
8857:         let safe_prefix: String = prefix.chars().filter(|c| c.is_alphanumeric()).collect();
8858:         let fname = format!(
8859:             "mdb_{}.json",
8860:             if safe_prefix.is_empty() {
8861:                 "ALL"
8862:             } else {
8863:                 &safe_prefix
8864:             }
8865:         );
8866:         let resp = format!(
8867:             "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Disposition: attachment; filename=\"{}\"\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
8868:             fname, body.len(), body
8869:         );
8870:         let _ = stream.write_all(resp.as_bytes());
8871:         return;
8872:     } else if path.starts_with("/il2cpp/dump_all_methods_dl") {
8873:         // v3.22.91: 暴力dump全部类方法目录（下载JSON，修复：内联下载包装）
8874:         let letter = parse_query(&full_uri, "letter");
8875:         let body = unsafe { il2cpp_dump_all_methods(&letter) };
8876:         let safe_letter: String = letter.chars().filter(|c| c.is_alphanumeric()).collect();
8877:         let fname = format!(
8878:             "dump_all_methods_{}.json",
8879:             if safe_letter.is_empty() {
8880:                 "ALL"
8881:             } else {
8882:                 &safe_letter
8883:             }
8884:         );
8885:         let resp = format!(
8886:             "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Disposition: attachment; filename=\"{}\"\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
8887:             fname, body.len(), body
8888:         );
8889:         let _ = stream.write_all(resp.as_bytes());
8890:         return;
8891:     } else if path.starts_with("/il2cpp/dump_all_methods") {
8892:         // v3.22.89: 暴力dump全部类方法目录（按letter分组避免手机卡死）
8893:         let letter = parse_query(&full_uri, "letter");
8894:         unsafe { il2cpp_dump_all_methods(&letter) }
8895:     } else if path.starts_with("/il2cpp/dump") {
8896:         // v3.22.89: dump单例实例（带运行时值）
8897:         let class_name = parse_query(&full_uri, "name");
8898:         unsafe { il2cpp_dump_singleton(&class_name) }
8899:     } else if path.starts_with("/il2cpp/invoke_instance") {
8900:         // v3.27: 调用实例方法 (基于 il2cpp_runtime_invoke, 需要 object 指针)
8901:         let class_name = parse_query(&full_uri, "class");
8902:         let method_name = parse_query(&full_uri, "method");
8903:         let object_addr = parse_query(&full_uri, "object");
8904:         let p0 = parse_query(&full_uri, "p0").parse::<i64>().unwrap_or(0);
8905:         let p1 = parse_query(&full_uri, "p1").parse::<i64>().unwrap_or(0);
8906:         let p2 = parse_query(&full_uri, "p2").parse::<i64>().unwrap_or(0);
8907:         let p3 = parse_query(&full_uri, "p3").parse::<i64>().unwrap_or(0);
8908:         let p4 = parse_query(&full_uri, "p4").parse::<i64>().unwrap_or(0);
8909:         let param_count = parse_query(&full_uri, "n").parse::<i32>().unwrap_or(0);
8910:         unsafe { il2cpp_invoke_instance_method(&class_name, &method_name, &object_addr, p0, p1, p2, p3, p4, param_count) }
8911:     } else if path.starts_with("/il2cpp/invoke_static") {
8912:         // v3.27: 调用static方法 (基于 il2cpp_runtime_invoke, 安全不崩)
8913:         let class_name = parse_query(&full_uri, "class");
8914:         let method_name = parse_query(&full_uri, "method");
8915:         let p0 = parse_query(&full_uri, "p0").parse::<i64>().unwrap_or(0);
8916:         let p1 = parse_query(&full_uri, "p1").parse::<i64>().unwrap_or(0);
8917:         let p2 = parse_query(&full_uri, "p2").parse::<i64>().unwrap_or(0);
8918:         let p3 = parse_query(&full_uri, "p3").parse::<i64>().unwrap_or(0);
8919:         let p4 = parse_query(&full_uri, "p4").parse::<i64>().unwrap_or(0);
8920:         let param_count = parse_query(&full_uri, "n").parse::<i32>().unwrap_or(0);
8921:         unsafe { il2cpp_invoke_static_method(&class_name, &method_name, p0, p1, p2, p3, p4, param_count) }
8922:     } else if path.starts_with("/il2cpp/call_static") {
8923:         // v3.25: 调用static方法 (无需singleton实例)
8924:         let class_name = parse_query(&full_uri, "class");
8925:         let method_name = parse_query(&full_uri, "method");
8926:         let p0 = parse_query(&full_uri, "p0").parse::<i64>().unwrap_or(0);
8927:         let p1 = parse_query(&full_uri, "p1").parse::<i64>().unwrap_or(0);
8928:         let p2 = parse_query(&full_uri, "p2").parse::<i64>().unwrap_or(0);
8929:         let p3 = parse_query(&full_uri, "p3").parse::<i64>().unwrap_or(0);
8930:         let p4 = parse_query(&full_uri, "p4").parse::<i64>().unwrap_or(0);
8931:         let param_count = parse_query(&full_uri, "n").parse::<i32>().unwrap_or(5);
8932:         unsafe { il2cpp_call_static_method(&class_name, &method_name, p0, p1, p2, p3, p4, param_count) }
8933:     } else if path.starts_with("/il2cpp/call") {
8934:         // v3.22.89: 调用单例上的getter方法
8935:         let class_name = parse_query(&full_uri, "class");
8936:         let method_name = parse_query(&full_uri, "method");
8937:         unsafe { il2cpp_call_method(&class_name, &method_name) }
8938:     } else if path.starts_with("/il2cpp/tree") {
8939:         // v3.22.89: 递归dump引用类型字段
8940:         let class_name = parse_query(&full_uri, "name");
8941:         let depth_str = parse_query(&full_uri, "depth");
8942:         let depth = depth_str.parse::<usize>().unwrap_or(2);
8943:         unsafe { il2cpp_tree_dump(&class_name, depth) }
8944:     } else if path.starts_with("/il2cpp/field") {
8945:         // v3.22.89: 读取单例的指定字段值
8946:         let class_name = parse_query(&full_uri, "class");
8947:         let field_name = parse_query(&full_uri, "field");
8948:         unsafe { il2cpp_read_single_field(&class_name, &field_name) }
8949:     } else if path.starts_with("/il2cpp/classes") {
8950:         // v3.22.89: 搜索IL2CPP类名（方案A）
8951:         let keyword = parse_query(&full_uri, "keyword");
8952:         unsafe { il2cpp_search_classes(&keyword) }
8953:     } else if path.starts_with("/il2cpp/static") {
8954:         // v3.22.89: 读取静态类常量值（方案B）
8955:         let class_name = parse_query(&full_uri, "name");
8956:         unsafe { il2cpp_read_static_fields(&class_name) }
8957:     } else if path == "/runtime/init_status" {
8958:         unsafe { foundation_init_status_endpoint() }
8959:     } else if path == "/hooks/registry" {
8960:         unsafe { foundation_hook_registry_endpoint() }
8961:     } else if path == "/hooks/diagnostics" {
8962:         foundation_hook_diagnostics_endpoint()
8963:     } else if path == "/capture/status" {
8964:         foundation_capture_status_endpoint()
8965:     } else if path == "/storage/files" {
8966:         storage_files_endpoint(&full_uri)
8967:     } else if path == "/storage/download" {
8968:         storage_download(&full_uri)
8969:     } else if path == "/il2cpp/call_targets" {
8970:         unsafe { il2cpp_call_targets(&full_uri) }
8971:     } else if path == "/il2cpp/callers" {
8972:         unsafe { il2cpp_callers(&full_uri) }
8973:     } else if path == "/il2cpp/type_detail" {
8974:         unsafe { il2cpp_type_detail(&full_uri) }
8975:     } else if path == "/il2cpp/object_dump" {
8976:         unsafe { il2cpp_object_dump(&full_uri) }
8977:     } else if path == "/inherit/tree" {
8978:         unsafe { inherit_tree_endpoint() }
8979:     } else if path == "/inherit/parent_records" {
8980:         unsafe { inherit_selected_parent_records_endpoint() }
8981:     } else if path == "/inherit/race_history" {
8982:         k_domain_endpoint(&path, &full_uri)
8983:     } else if path == "/inherit/race_compat" {
8984:         k_domain_endpoint(&path, &full_uri)
8985:     } else if path == "/inherit/full_compat" {
8986:         k_domain_endpoint(&path, &full_uri)
8987:     } else if path == "/inherit/compat_trace" {
8988:         k_domain_endpoint(&path, &full_uri)
8989:     } else if path == "/inherit/factor_tree" {
8990:         k_domain_endpoint(&path, &full_uri)
8991:     } else if path == "/inherit/bonus_params" {
8992:         k_domain_endpoint(&path, &full_uri)
8993:     } else if path == "/inherit/event_trace" {
8994:         k_domain_endpoint(&path, &full_uri)
8995:     } else if path == "/inherit/deck_runtime" {
8996:         k_domain_endpoint(&path, &full_uri)
8997:     } else if path == "/inherit/deck_validate" {
8998:         k_domain_endpoint(&path, &full_uri)
8999:     } else if path == "/inherit/friend_rental_context" {
9000:         k_domain_endpoint(&path, &full_uri)
9001:     } else if path == "/inherit/auto_select_trace" {
9002:         k_domain_endpoint(&path, &full_uri)
9003:     } else if path == "/autoplay/runtime" {
9004:         k_domain_endpoint(&path, &full_uri)
9005:     } else if path == "/autoplay/plan" {
9006:         k_domain_endpoint(&path, &full_uri)
9007:     } else if path == "/autoplay/action_trace" {
9008:         k_domain_endpoint(&path, &full_uri)
9009:     } else if path == "/autoplay/factor_select_trace" {
9010:         k_domain_endpoint(&path, &full_uri)
9011:     } else if path == "/offline_auto/runtime" {
9012:         k_domain_endpoint(&path, &full_uri)
9013:     } else if path == "/offline_auto/start_request" {
9014:         k_domain_endpoint(&path, &full_uri)
9015:     } else if path == "/offline_auto/race_reserve" {
9016:         k_domain_endpoint(&path, &full_uri)
9017:     } else if path == "/offline_auto/result" {
9018:         k_domain_endpoint(&path, &full_uri)
9019:     } else if path == "/generate_succession/status" {
9020:         k_domain_endpoint(&path, &full_uri)
9021:     } else if path == "/generate_succession/limits" {
9022:         k_domain_endpoint(&path, &full_uri)
9023:     } else if path == "/generate_succession/request" {
9024:         k_domain_endpoint(&path, &full_uri)
9025:     } else if path == "/generate_succession/runtime_full" {
9026:         unsafe { generated_succession_runtime_endpoint() }
9027:     } else if path == "/generate_succession/result" {
9028:         k_domain_endpoint(&path, &full_uri)
9029:     } else if path == "/generate_succession/candidates" {
9030:         k_domain_endpoint(&path, &full_uri)
9031:     } else if path == "/generate_succession/race_reserve" {
9032:         k_domain_endpoint(&path, &full_uri)
9033:     } else if path == "/generate_succession/race_validation" {
9034:         k_domain_endpoint(&path, &full_uri)
9035:     } else if path == "/generate_succession/factor_priority" {
9036:         k_domain_endpoint(&path, &full_uri)
9037:     } else if path == "/generate_succession/factor_order" {
9038:         k_domain_endpoint(&path, &full_uri)
9039:     } else if path == "/generate_succession/probability_trace" {
9040:         k_domain_endpoint(&path, &full_uri)
9041:     } else if path == "/generate_succession/cost_trace" {
9042:         k_domain_endpoint(&path, &full_uri)
9043:     } else if path == "/factor/finish_trace" {
9044:         k_domain_endpoint(&path, &full_uri)
9045:     } else if path == "/factor/candidates" {
9046:         k_domain_endpoint(&path, &full_uri)
9047:     } else if path == "/factor/roll_trace" {
9048:         k_domain_endpoint(&path, &full_uri)
9049:     } else if path == "/factor/probability_model" {
9050:         k_domain_endpoint(&path, &full_uri)
9051:     } else if path == "/factor/history" {
9052:         k_domain_endpoint(&path, &full_uri)
9053:     } else if path == "/factor/stats" {
9054:         k_domain_endpoint(&path, &full_uri)
9055:     } else if path == "/factor/breeding_advice" {
9056:         k_domain_endpoint(&path, &full_uri)
9057:     } else if path == "/api/sniff/exchanges" {
9058:         k_domain_endpoint(&path, &full_uri)
9059:     } else if path == "/api/sniff/exchange" {
9060:         k_domain_endpoint(&path, &full_uri)
9061:     } else if path == "/api/hook/install" {
9062:         k_domain_endpoint(&path, &full_uri)
9063:     } else if path == "/api/hook/remove" {
9064:         k_domain_endpoint(&path, &full_uri)
9065:     } else if path == "/api/hook/list" {
9066:         k_domain_endpoint(&path, &full_uri)
9067:     } else if path == "/api/hook/events" {
9068:         k_domain_endpoint(&path, &full_uri)
9069:     } else if path == "/storage/audit" {
9070:         protocol_archive_audit_endpoint(&full_uri)
9071:     } else if path == "/storage/status" {
9072:         storage_status_endpoint()
9073:     } else if path == "/storage/sessions" {
9074:         storage_sessions_endpoint()
9075:     } else if path == "/storage/session" {
9076:         storage_session_endpoint(&full_uri)
9077:     } else if path == "/storage/flush" {
9078:         storage_flush_endpoint()
9079:     } else if path == "/storage/recover" {
9080:         storage_recover_endpoint()
9081:     } else if path == "/il2cpp/method_index_status" {
9082:         method_index_status_endpoint(&full_uri)
9083:     } else if path == "/il2cpp/method_by_addr" {
9084:         unsafe { il2cpp_method_by_addr(&full_uri) }
9085:     } else if path == "/il2cpp/method_detail" {
9086:         unsafe { il2cpp_method_detail(&full_uri) }
9087:     } else if path == "/il2cpp/nested_types" {
9088:         unsafe { il2cpp_nested_types(&full_uri) }
9089:     } else if path == "/il2cpp/enum_values" {
9090:         unsafe { il2cpp_enum_values_capability(&full_uri) }
9091:     } else if path.starts_with("/il2cpp/methods") {
9092:         // v3.22.89: 列出类的所有方法名和参数数量
9093:         let class_name = parse_query(&full_uri, "name");
9094:         unsafe { il2cpp_list_methods(&class_name) }
9095:     } else if path.starts_with("/il2cpp/disassemble_dl") {
9096:         // v3.22.89: 反汇编结果下载JSON文件（手机浏览器复制上限对策）
9097:         let class_name = parse_query(&full_uri, "class");
9098:         let method_name = parse_query(&full_uri, "method");
9099:         let bytes_str = parse_query(&full_uri, "bytes");
9100:         let bytes_limit = bytes_str.parse::<usize>().unwrap_or(2048);
9101:         unsafe { il2cpp_disassemble(&class_name, &method_name, bytes_limit) }
9102:     } else if path.starts_with("/il2cpp/disassemble_addr_dl") {
9103:         // v3.22.91: 按地址反汇编结果下载JSON文件（修复：内联下载包装，避免被starts_with截胡）
9104:         let addr_str = parse_query(&full_uri, "addr");
9105:         let bytes_str = parse_query(&full_uri, "bytes");
9106:         let bytes_limit = bytes_str.parse::<usize>().unwrap_or(2048);
9107:         let body = unsafe { il2cpp_disassemble_addr(&addr_str, bytes_limit) };
9108:         let safe_addr: String = addr_str.chars().filter(|c| c.is_alphanumeric()).collect();
9109:         let fname = format!(
9110:             "disassemble_addr_{}.json",
9111:             if safe_addr.is_empty() {
9112:                 "output"
9113:             } else {
9114:                 &safe_addr
9115:             }
9116:         );
9117:         let resp = format!(
9118:             "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Disposition: attachment; filename=\"{}\"\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
9119:             fname, body.len(), body
9120:         );
9121:         let _ = stream.write_all(resp.as_bytes());
9122:         return;
9123:     } else if path.starts_with("/il2cpp/disassemble_addr") {
9124:         // v3.22.89: 按地址反汇编ARM64指令体（分析ExecTraining等方法的子函数调用目标）
9125:         let addr_str = parse_query(&full_uri, "addr");
9126:         let bytes_str = parse_query(&full_uri, "bytes");
9127:         let bytes_limit = bytes_str.parse::<usize>().unwrap_or(2048);
9128:         unsafe { il2cpp_disassemble_addr(&addr_str, bytes_limit) }
9129:     } else if path.starts_with("/il2cpp/disassemble") {
9130:         // v3.22.89: 反汇编IL2CPP方法的ARM64指令体
9131:         let class_name = parse_query(&full_uri, "class");
9132:         let method_name = parse_query(&full_uri, "method");
9133:         let bytes_str = parse_query(&full_uri, "bytes");
9134:         let bytes_limit = bytes_str.parse::<usize>().unwrap_or(2048);
9135:         unsafe { il2cpp_disassemble(&class_name, &method_name, bytes_limit) }
9136:     } else if path.starts_with("/il2cpp/search_int_dl") {
9137:         // v3.22.91: 搜索整数千分比（下载JSON，修复：内联下载包装）
9138:         let values_str = parse_query(&full_uri, "values");
9139:         let body = unsafe { il2cpp_search_int(&values_str) };
9140:         let safe_vals: String = values_str
9141:             .chars()
9142:             .filter(|c| c.is_alphanumeric() || *c == ',')
9143:             .collect();
9144:         let fname = format!(
9145:             "search_int_{}.json",
9146:             if safe_vals.is_empty() {
9147:                 "all".into()
9148:             } else {
9149:                 safe_vals
9150:             }
9151:         );
9152:         let resp = format!(
9153:             "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Disposition: attachment; filename=\"{}\"\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
9154:             fname, body.len(), body
9155:         );
9156:         let _ = stream.write_all(resp.as_bytes());
9157:         return;
9158:     } else if path.starts_with("/il2cpp/search_int") {
9159:         // v3.22.89: 搜索整数千分比
9160:         let values_str = parse_query(&full_uri, "values");
9161:         unsafe { il2cpp_search_int(&values_str) }
9162:     } else if path.starts_with("/il2cpp/search_float_dl") {
9163:         // v3.22.93: 搜索浮点常量（下载JSON，与search_int_dl对称）
9164:         let value_str = parse_query(&full_uri, "value");
9165:         let body = unsafe { il2cpp_search_float(&value_str) };
9166:         let safe_val: String = value_str
9167:             .chars()
9168:             .filter(|c| c.is_alphanumeric() || *c == '.' || *c == '-')
9169:             .collect();
9170:         let fname = format!(
9171:             "search_float_{}.json",
9172:             if safe_val.is_empty() {
9173:                 "all".into()
9174:             } else {
9175:                 safe_val
9176:             }
9177:         );
9178:         let resp = format!(
9179:             "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Disposition: attachment; filename=\"{}\"\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
9180:             fname, body.len(), body
9181:         );
9182:         let _ = stream.write_all(resp.as_bytes());
9183:         return;
9184:     } else if path.starts_with("/il2cpp/search_float") {
9185:         // v3.22.89: 在代码段搜索浮点常量（方案D）
9186:         let value_str = parse_query(&full_uri, "value");
9187:         unsafe { il2cpp_search_float(&value_str) }
9188:     } else if path.starts_with("/il2cpp/read_mem_dl") {
9189:         // v3.22.91: 读取原始内存（下载hex dump，修复：内联下载包装）
9190:         let addr_str = parse_query(&full_uri, "addr");
9191:         let size_str = parse_query(&full_uri, "size");
9192:         let body = il2cpp_read_mem(&addr_str, &size_str);
9193:         let safe_addr: String = addr_str.chars().filter(|c| c.is_alphanumeric()).collect();
9194:         let fname = format!(
9195:             "read_mem_{}.txt",
9196:             if safe_addr.is_empty() {
9197:                 "output"
9198:             } else {
9199:                 &safe_addr
9200:             }
9201:         );
9202:         let resp = format!(
9203:             "HTTP/1.1 200 OK\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Disposition: attachment; filename=\"{}\"\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
9204:             fname, body.len(), body
9205:         );
9206:         let _ = stream.write_all(resp.as_bytes());
9207:         return;
9208:     } else if path.starts_with("/il2cpp/read_mem") {
9209:         // v3.22.89: 读取原始内存（hex dump）
9210:         let addr_str = parse_query(&full_uri, "addr");
9211:         let size_str = parse_query(&full_uri, "size");
9212:         il2cpp_read_mem(&addr_str, &size_str)
9213:     } else if path.starts_with("/il2cpp/read_string") {
9214:         (|| -> String {
9215:         // ★ Read IL2CPP string at address (or via pointer indirection)
9216:         // ?addr=0x...       → addr points to Il2CppString object directly
9217:         // ?ptr=0x...        → read 8 bytes at ptr to get Il2CppString*, then read string
9218:         let addr_str = parse_query(&full_uri, "addr");
9219:         let ptr_str = parse_query(&full_uri, "ptr");
9220:         let target = if !ptr_str.is_empty() {
9221:             // Indirection: read pointer at ptr_str, then read string
9222:             let ptr_addr = usize::from_str_radix(ptr_str.trim_start_matches("0x"), 16).unwrap_or(0);
9223:             if ptr_addr == 0 {
9224:                 return r#"{"error":"invalid_ptr_addr"}"#.to_string();
9225:             }
9226:             unsafe {
9227:                 let p = std::ptr::read::<u64>(ptr_addr as *const u64);
9228:                 p as usize
9229:             }
9230:         } else if !addr_str.is_empty() {
9231:             usize::from_str_radix(addr_str.trim_start_matches("0x"), 16).unwrap_or(0)
9232:         } else {
9233:             return r#"{"error":"need_addr_or_ptr"}"#.to_string();
9234:         };
9235:         if target == 0 {
9236:             return r#"{"error":"invalid_target"}"#.to_string();
9237:         }
9238:         let s = unsafe { read_il2cpp_string(target as *const c_void) };
9239:         // Also dump raw bytes for debugging
9240:         let raw_len = unsafe {
9241:             std::ptr::read::<i32>((target as *const u8).offset(16) as *const i32)
9242:         };
9243:         format!(
9244:             r#"{{"addr":"0x{:x}","length":{},"string":"{}","raw_len":{}}}"#,
9245:             target,
9246:             s.len(),
9247:             s.replace('\\', "\\\\").replace('"', "\\\""),
9248:             raw_len
9249:         )
9250:         })()
9251:     } else if path == "/il2cpp/search_methods_page" {
9252:         // v3.22.89: 搜索方法名HTML页面（A-Z分组）
9253:         search_methods_page()
9254:     } else if path.starts_with("/il2cpp/search_methods_dl") {
9255:         // v3.22.91: 跨类搜索方法名（下载JSON，修复：内联下载包装）
9256:         let keyword = parse_query(&full_uri, "keyword");
9257:         let letter = parse_query(&full_uri, "letter");
9258:         let body = unsafe { il2cpp_search_methods(&keyword, &letter) };
9259:         let kw = &keyword;
9260:         let safe_kw: String = kw
9261:             .chars()
9262:             .filter(|c| c.is_alphanumeric() || *c == '_')
9263:             .collect();
9264:         let fname = format!(
9265:             "search_methods_{}.json",
9266:             if safe_kw.is_empty() {
9267:                 "all".into()
9268:             } else {
9269:                 safe_kw
9270:             }
9271:         );
9272:         let resp = format!(
9273:             "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Disposition: attachment; filename=\"{}\"\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
9274:             fname, body.len(), body
9275:         );
9276:         let _ = stream.write_all(resp.as_bytes());
9277:         return;
9278:     } else if path.starts_with("/il2cpp/search_methods") {
9279:         // v3.22.89: 跨类搜索方法名关键词
9280:         let keyword = parse_query(&full_uri, "keyword");
9281:         let letter = parse_query(&full_uri, "letter");
9282:         unsafe { il2cpp_search_methods(&keyword, &letter) }
9283:     } else if path == "/debug/private_file_inventory" {
9284:         debug_private_file_inventory(&full_uri)
9285:     } else if path == "/debug/private_file_dl" {
9286:         download_private_file_by_id(&mut stream, &full_uri);
9287:         return;
9288:     } else if path.starts_with("/debug/file_dl") {
9289:         // ★ v3.24.66: download an arbitrary file ONLY if its path currently appears
9290:         // in /proc/self/maps (i.e. a loaded game library) — no free-form path reads.
9291:         let want = parse_query(&full_uri, "path");
9292:         let mut allowed = false;
9293:         if !want.is_empty() {
9294:             if let Ok(maps) = std::fs::read_to_string("/proc/self/maps") {
9295:                 for line in maps.lines() {
9296:                     let cols: Vec<&str> = line.split_whitespace().collect();
9297:                     if cols.get(5).copied() == Some(want.as_str()) {
9298:                         allowed = true;
9299:                         break;
9300:                     }
9301:                 }
9302:             }
9303:         }
9304:         if !allowed {
9305:             format!(
9306:                 r#"{{"error":"not_in_maps","hint":"path must appear in /proc/self/maps (see /debug/maps_list)"}}"#
9307:             )
9308:         } else {
9309:             let fname = std::path::Path::new(&want)
9310:                 .file_name()
9311:                 .and_then(|v| v.to_str())
9312:                 .unwrap_or("file.bin")
9313:                 .to_string();
9314:             stream_private_file(&mut stream, &want, &fname);
9315:             return;
9316:         }
9317:     } else if path == "/debug/resource_storage" {
9318:         debug_resource_storage()
9319:     } else if path == "/debug/resource_meta_schema" {
9320:         debug_resource_meta_schema()
9321:     } else if path == "/debug/resource_meta_probe" {
9322:         debug_resource_meta_probe()
9323:     } else if path == "/debug/resource_crypto_symbols" {
9324:         debug_resource_crypto_symbols()
9325:     } else if path == "/debug/resource_meta_dl" {
9326:         // Allow only the index and its known SQLite sidecars; never an arbitrary path.
9327:         let part = parse_query(&full_uri, "part");
9328:         let (suffix, filename) = match part.as_str() {
9329:             "journal" => ("-journal", "meta-journal"),
9330:             "wal" => ("-wal", "meta-wal"),
9331:             "shm" => ("-shm", "meta-shm"),
9332:             _ => ("", "meta"),
9333:         };
9334:         match find_resource_storage() {
9335:             Ok((meta, _)) => {
9336:                 let target = format!("{}{}", meta, suffix);
9337:                 stream_private_file(&mut stream, &target, filename);
9338:                 return;
9339:             }
9340:             Err(e) => format!(r#"{{"error":"{}"}}"#, json_escape(&e)),
9341:         }
9342:     } else if path == "/debug/resource_file_dl" {
9343:         // v3.24.62: meta `a` 表的 h 是 Base32(A-Z2-7,32字符) 且就是 dat 文件名原文，
9344:         // 与 hex 哈希一并接受；Base32 需保持原样（不做 lowercase）。
9345:         let raw_hash = parse_query(&full_uri, "hash");
9346:         let hash = if raw_hash.len() == 32
9347:             && raw_hash
9348:                 .bytes()
9349:                 .all(|b| matches!(b, b'A'..=b'Z' | b'a'..=b'z' | b'2'..=b'7'))
9350:             && !raw_hash.bytes().all(|b| b.is_ascii_hexdigit())
9351:         {
9352:             raw_hash.to_ascii_uppercase()
9353:         } else {
9354:             raw_hash.to_ascii_lowercase()
9355:         };
9356:         let hash_ok = valid_resource_hash(&hash)
9357:             || (hash.len() == 32 && hash.bytes().all(|b| matches!(b, b'A'..=b'Z' | b'2'..=b'7')));
9358:         if !hash_ok {
9359:             r#"{"error":"invalid_hash","requirement":"8..128 hexadecimal characters"}"#.to_string()
9360:         } else {
9361:             match find_resource_storage() {
9362:                 Ok((_, dat)) => {
9363:                     let target = std::path::Path::new(&dat).join(&hash[..2]).join(&hash);
9364:                     if !target.is_file() {
9365:                         format!(r#"{{"error":"resource_not_found","hash":"{}"}}"#, hash)
9366:                     } else {
9367:                         stream_private_file(&mut stream, &target.to_string_lossy(), &hash);
9368:                         return;
9369:                     }
```

## lines 9380-9505

```rust
9380:         }
9381:     } else {
9382:         format!(
9383:             r#"{{"error":"not_found","path":"{}","available":[\"/inherit/tree\",\"/inherit/parent_records\",\"/inherit/race_history\",\"/inherit/race_compat\",\"/inherit/full_compat\",\"/inherit/compat_trace\",\"/inherit/factor_tree\",\"/inherit/bonus_params\",\"/inherit/event_trace\",\"/inherit/deck_runtime\",\"/inherit/deck_validate\",\"/inherit/friend_rental_context\",\"/inherit/auto_select_trace\",\"/autoplay/runtime\",\"/autoplay/plan\",\"/autoplay/action_trace\",\"/autoplay/factor_select_trace\",\"/offline_auto/runtime\",\"/offline_auto/start_request\",\"/offline_auto/race_reserve\",\"/offline_auto/result\",\"/generate_succession/status\",\"/generate_succession/limits\",\"/generate_succession/request\",\"/generate_succession/result\",\"/generate_succession/candidates\",\"/generate_succession/race_reserve\",\"/generate_succession/race_validation\",\"/generate_succession/factor_priority\",\"/generate_succession/factor_order\",\"/generate_succession/probability_trace\",\"/generate_succession/cost_trace\",\"/factor/finish_trace\",\"/factor/candidates\",\"/factor/roll_trace\",\"/factor/probability_model\",\"/factor/history\",\"/factor/stats\",\"/factor/breeding_advice\",\"/il2cpp/call_targets\",\"/il2cpp/callers\",\"/il2cpp/type_detail\",\"/il2cpp/object_dump\",\"/api/sniff/exchanges\",\"/api/sniff/exchange\",\"/api/hook/install\",\"/api/hook/remove\",\"/api/hook/list\",\"/api/hook/events\",\"/storage/files\",\"/storage/download\","/storage/status","/storage/sessions","/storage/session","/storage/flush","/storage/recover","/il2cpp/method_index_status","/il2cpp/method_by_addr","/il2cpp/method_detail","/il2cpp/nested_types","/il2cpp/enum_values","/inherit/pair_compat","/inherit/selected_parent_runtime","/scan","/data","/status","/health","/scenario","/debug/upload","/debug/rameninfo","/debug/laststep","/event/recommend","/inherit/compat","/saddle-analysis","/log/turn","/log","/debug/params","/fields","/methods","/singletons","/find_method","/classes","/carddb","/skilldata","/hall","/debug/breeders","/debug/cmdinfo","/debug/training_partners","/debug/ramengains","/debug/paramsincdec","/debug/training_seed","/debug/training_log","/debug/training_log_dl","/update","/update/status","/debug/dumpclass","/debug/storydata","/debug/ramenfields","/debug/all","/mdb","/debug/push_table","/debug/download_table","/classes/search/keyword","/mdb/schema","/mdb/search","/mdb/raw","/mdb/dl_batch","/il2cpp/dump","/il2cpp/call","/il2cpp/tree","/il2cpp/field","/il2cpp/classes","/il2cpp/static","/il2cpp/methods","/il2cpp/search_float","/il2cpp/search_float_dl","/il2cpp/search_int","/il2cpp/search_int_dl","/il2cpp/search_methods","/il2cpp/search_methods_dl","/il2cpp/search_methods_page","/il2cpp/read_mem","/il2cpp/read_mem_dl","/training/result","/api/sniff","/api/sniff/metadata","/api/sniff/status","/api/sniff/toggle","/api/sniff/clear","/api/sniff/diag","/api/event/choices","/api/event/clear"]}}"#,
9384:             path
9385:         )
9386:     };
9387: 
9388:     save_endpoint_log(&path, &body);
9389: 
9390:     if body.starts_with("__MDB_BINARY__") {
9391:         // v3.22.51: Serve raw mdb file as binary response
9392:         let mdb_path = &body[14..]; // skip "__MDB_BINARY__"
9393:         match std::fs::read(mdb_path) {
9394:             Ok(data) => {
9395:                 let header = format!(
9396:                     "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Disposition: attachment; filename=\"master.mdb\"\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
9397:                     data.len()
9398:                 );
9399:                 let _ = stream.write_all(header.as_bytes());
9400:                 // Write in chunks to avoid memory spike
9401:                 for chunk in data.chunks(65536) {
9402:                     let _ = stream.write_all(chunk);
9403:                 }
9404:             }
9405:             Err(e) => {
9406:                 let err_json = format!(r#"{{"error":"mdb_read_failed","detail":"{}"}}"#, e);
9407:                 let resp = format!(
9408:                     "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
9409:                     err_json.len(), err_json
9410:                 );
9411:                 let _ = stream.write_all(resp.as_bytes());
9412:             }
9413:         }
9414:     } else if path == "/saddles-dl" {
9415:         let resp = format!(
9416:             "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Disposition: attachment; filename=\"saddles.json\"\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
9417:             body.len(), body
9418:         );
9419:         let _ = stream.write_all(resp.as_bytes());
9420:     } else if path == "/il2cpp/disassemble_dl" {
9421:         // v3.22.89: 反汇编结果下载为JSON文件
9422:         let cn = parse_query(&full_uri, "class");
9423:         let mn = parse_query(&full_uri, "method");
9424:         let safe_name: String = format!(
9425:             "{}_{}",
9426:             cn.chars()
9427:                 .filter(|c| c.is_alphanumeric() || *c == '_')
9428:                 .collect::<String>(),
9429:             mn.chars()
9430:                 .filter(|c| c.is_alphanumeric() || *c == '_')
9431:                 .collect::<String>()
9432:         );
9433:         let fname = format!(
9434:             "disassemble_{}.json",
9435:             if safe_name.is_empty() {
9436:                 "output"
9437:             } else {
9438:                 &safe_name
9439:             }
9440:         );
9441:         let resp = format!(
9442:             "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Disposition: attachment; filename=\"{}\"\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
9443:             fname, body.len(), body
9444:         );
9445:     } else {
9446:         let content_type = if body.starts_with("<!DOCTYPE") || body.starts_with("<html") {
9447:             "text/html; charset=utf-8"
9448:         } else {
9449:             "application/json"
9450:         };
9451:         if dl_enabled {
9452:             // 下载模式：默认按路由生成文件名，?name= 可覆盖
9453:             let safe: String = dl_name
9454:                 .chars()
9455:                 .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
9456:                 .take(64)
9457:                 .collect();
9458:             let fallback = path.trim_matches('/').replace('/', "_");
9459:             let base = if safe.is_empty() { fallback } else { safe };
9460:             let base = if base.is_empty() {
9461:                 "download".to_string()
9462:             } else {
9463:                 base
9464:             };
9465:             let ext = if content_type.starts_with("text/html") {
9466:                 "html"
9467:             } else {
9468:                 "json"
9469:             };
9470:             let fname = format!("{}.{}", base, ext);
9471:             let resp = format!(
9472:                 "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Disposition: attachment; filename=\"{}\"\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
9473:                 fname, body.len(), body
9474:             );
9475:             let _ = stream.write_all(resp.as_bytes());
9476:         } else {
9477:             let resp = format!(
9478:                 "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
9479:                 content_type, body.len(), body
9480:             );
9481:             let _ = stream.write_all(resp.as_bytes());
9482:         }
9483:     }
9484:     let _ = stream.flush();
9485: }
9486: 
9487: // ============================================================
9488: // v3.22.51: Pre-cache all class metadata on game thread
9489: // ============================================================
9490: 
9491: /// Convert PascalCase to snake_case for cache key matching
9492: fn to_snake_case(name: &str) -> String {
9493:     let mut result = String::new();
9494:     for (i, c) in name.chars().enumerate() {
9495:         if c.is_uppercase() && i > 0 {
9496:             result.push('_');
9497:         }
9498:         result.extend(c.to_lowercase());
9499:     }
9500:     result
9501: }
9502: 
9503: /// Pre-cache ALL field offsets for a class (including parent classes)
9504: /// Called on game thread — safe to use IL2CPP API
9505: unsafe fn precache_all_fields(class: *mut c_void) {
```

## lines 13472-13550

```rust
13472:     rows.sort_by(|a, b| a.0.cmp(&b.0));
13473:     Ok(rows
13474:         .into_iter()
13475:         .enumerate()
13476:         .map(
13477:             |(id, (relative_path, absolute_path, size, kind))| PrivateFileEntry {
13478:                 id,
13479:                 relative_path,
13480:                 absolute_path,
13481:                 size,
13482:                 kind,
13483:             },
13484:         )
13485:         .collect())
13486: }
13487: 
13488: fn debug_private_file_inventory(full_uri: &str) -> String {
13489:     if parse_query(full_uri, "confirm") != "1" {
13490:         return r#"{"ok":false,"error":"explicit_confirmation_required","warning":"private files may contain account session cookie or device identifiers","retry":"/debug/private_file_inventory?confirm=1&offset=0&limit=200"}"#.to_string();
13491:     }
13492:     let offset = parse_query(full_uri, "offset")
13493:         .parse::<usize>()
13494:         .unwrap_or(0);
13495:     let limit = parse_query(full_uri, "limit")
13496:         .parse::<usize>()
13497:         .unwrap_or(200)
13498:         .clamp(1, 500);
13499:     match private_file_inventory() {
13500:         Ok(entries) => {
13501:             let total = entries.len();
13502:             let rows: Vec<String> = entries.iter().skip(offset).take(limit).map(|e| format!(
13503:                 r#"{{"id":{},"path":"{}","size":{},"kind":"{}","download":"/debug/private_file_dl?confirm=1&id={}"}}"#,
13504:                 e.id, json_escape(&e.relative_path), e.size, e.kind, e.id)).collect();
13505:             format!(
13506:                 r#"{{"ok":true,"snapshot_rebuilt_each_request":true,"dat_skipped":true,"max_depth":10,"max_files":20000,"total":{},"offset":{},"limit":{},"files":[{}]}}"#,
13507:                 total,
13508:                 offset,
13509:                 limit,
13510:                 rows.join(",")
13511:             )
13512:         }
13513:         Err(e) => format!(r#"{{"ok":false,"error":"{}"}}"#, json_escape(&e)),
13514:     }
13515: }
13516: 
13517: fn download_private_file_by_id(stream: &mut std::net::TcpStream, full_uri: &str) {
13518:     if parse_query(full_uri, "confirm") != "1" {
13519:         stream_private_file(
13520:             stream,
13521:             "/__explicit_confirmation_required__",
13522:             "confirmation_required",
13523:         );
13524:         return;
13525:     }
13526:     let id = match parse_query(full_uri, "id").parse::<usize>() {
13527:         Ok(v) => v,
13528:         Err(_) => {
13529:             stream_private_file(stream, "/__invalid_private_file_id__", "invalid");
13530:             return;
13531:         }
13532:     };
13533:     match private_file_inventory()
13534:         .ok()
13535:         .and_then(|v| v.into_iter().find(|e| e.id == id))
13536:     {
13537:         Some(entry) => {
13538:             let name = std::path::Path::new(&entry.relative_path)
13539:                 .file_name()
13540:                 .and_then(|v| v.to_str())
13541:                 .unwrap_or("private_file.bin");
13542:             stream_private_file(stream, &entry.absolute_path, name);
13543:         }
13544:         None => stream_private_file(stream, "/__private_file_not_found__", "not_found"),
13545:     }
13546: }
13547: 
13548: fn find_resource_storage() -> Result<(String, String), String> {
13549:     use std::collections::{HashSet, VecDeque};
13550:     use std::path::{Path, PathBuf};
```

## lines 14007-14083

```rust
14007:                 std::path::Path::new(&wal).is_file(),
14008:                 std::path::Path::new(&shm).is_file()
14009:             )
14010:         }
14011:         Err(error) => format!(
14012:             r#"{{"ok":false,"error":"{}","read_only":true}}"#,
14013:             json_escape(&error)
14014:         ),
14015:     }
14016: }
14017: 
14018: fn valid_resource_hash(hash: &str) -> bool {
14019:     (8..=128).contains(&hash.len()) && hash.bytes().all(|b| b.is_ascii_hexdigit())
14020: }
14021: 
14022: /// Stream a private game file without loading it into memory.
14023: fn stream_private_file(stream: &mut std::net::TcpStream, path: &str, filename: &str) {
14024:     use std::io::{Read, Write};
14025:     let mut file = match std::fs::File::open(path) {
14026:         Ok(v) => v,
14027:         Err(e) => {
14028:             let body = format!(
14029:                 r#"{{"error":"file_open_failed","detail":"{}"}}"#,
14030:                 json_escape(&e.to_string())
14031:             );
14032:             let response = format!("HTTP/1.1 404 Not Found\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", body.len(), body);
14033:             let _ = stream.write_all(response.as_bytes());
14034:             return;
14035:         }
14036:     };
14037:     let size = match file.metadata() {
14038:         Ok(v) if v.is_file() => v.len(),
14039:         _ => 0,
14040:     };
14041:     let safe_name: String = filename
14042:         .chars()
14043:         .filter(|c| c.is_ascii_alphanumeric() || *c == '.' || *c == '_' || *c == '-')
14044:         .take(140)
14045:         .collect();
14046:     let header = format!(
14047:         "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Disposition: attachment; filename=\"{}\"\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
14048:         if safe_name.is_empty() { "download.bin" } else { &safe_name }, size
14049:     );
14050:     if stream.write_all(header.as_bytes()).is_err() {
14051:         return;
14052:     }
14053:     let mut buffer = [0u8; 65536];
14054:     loop {
14055:         let count = match file.read(&mut buffer) {
14056:             Ok(0) | Err(_) => break,
14057:             Ok(n) => n,
14058:         };
14059:         if stream.write_all(&buffer[..count]).is_err() {
14060:             break;
14061:         }
14062:     }
14063:     let _ = stream.flush();
14064: }
14065: 
14066: fn find_mdb_path() -> Option<String> {
14067:     let paths = [
14068:         "/data/data/jp.pokemon.pokeuma/files/master/master.mdb",
14069:         "/data/user/0/jp.pokemon.pokeuma/files/master/master.mdb",
14070:         "/data/data/jp.pokemon.pokeuma/files/master/master (1).mdb",
14071:         "/data/user/0/jp.pokemon.pokeuma/files/master/master (1).mdb",
14072:         "/storage/emulated/0/Android/data/jp.pokemon.pokeuma/files/master/master.mdb",
14073:     ];
14074: 
14075:     for p in &paths {
14076:         if std::path::Path::new(p).exists() {
14077:             return Some(p.to_string());
14078:         }
14079:     }
14080: 
14081:     // Try to discover from /proc/self/cmdline
14082:     if let Ok(bytes) = std::fs::read("/proc/self/cmdline") {
14083:         let pkg = bytes
```

## lines 14124-14179

```rust
14124:             if let Ok(n) = u8::from_str_radix(hex, 16) {
14125:                 result.push(n as char);
14126:                 i += 3;
14127:             } else {
14128:                 result.push(bytes[i] as char);
14129:                 i += 1;
14130:             }
14131:         } else {
14132:             result.push(bytes[i] as char);
14133:             i += 1;
14134:         }
14135:     }
14136:     result
14137: }
14138: 
14139: /// v3.22.89: 解析query参数值
14140: fn parse_query(full_uri: &str, key: &str) -> String {
14141:     let pattern = format!("{}=", key);
14142:     if let Some(q) = full_uri.find(&format!("?{}", pattern)) {
14143:         let start = q + 1 + pattern.len();
14144:         let end = full_uri[start..]
14145:             .find('&')
14146:             .map(|e| start + e)
14147:             .unwrap_or(full_uri.len());
14148:         url_decode(&full_uri[start..end])
14149:     } else if let Some(q) = full_uri.find(&format!("&{}", pattern)) {
14150:         let start = q + 1 + pattern.len();
14151:         let end = full_uri[start..]
14152:             .find('&')
14153:             .map(|e| start + e)
14154:             .unwrap_or(full_uri.len());
14155:         url_decode(&full_uri[start..end])
14156:     } else {
14157:         String::new()
14158:     }
14159: }
14160: 
14161: /// /tables - List all tables in MasterDB for discovery
14162: /// /tables - List all tables in MasterDB for discovery
14163: /// /debug/unique_skills - Explore mdb tables related to unique skill unlock conditions
14164: /// Dumps table names matching "unique"/"acquisition"/"condition" and their first few rows
14165: fn debug_unique_skills() -> String {
14166:     let mdb_path = match find_mdb_path() {
14167:         Some(p) => p,
14168:         None => return r#"{"error":"mdb_not_found"}"#.to_string(),
14169:     };
14170:     let conn = match Connection::open_with_flags(&mdb_path, OpenFlags::SQLITE_OPEN_READ_ONLY) {
14171:         Ok(c) => c,
14172:         Err(e) => return format!(r#"{{"error":"open_failed","detail":"{}"}}"#, e),
14173:     };
14174: 
14175:     // Step 1: Find all tables that might relate to unique skills
14176:     let all_tables: Vec<String> =
14177:         match conn.prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name") {
14178:             Ok(mut stmt) => stmt
14179:                 .query_map([], |row| Ok(row.get::<_, String>(0).unwrap_or_default()))
```

## lines 14529-14620

```rust
14529:                     pairs.push(format!(r#""{}":{}"#, json_escape(&cn), val));
14530:                 }
14531:                 Ok(format!(r#"{{{}}}"#, pairs.join(",")))
14532:             });
14533:             match rows_result {
14534:                 Ok(rows) => rows.filter_map(|r| r.ok()).collect(),
14535:                 Err(_) => Vec::new(),
14536:             }
14537:         }
14538:         Err(e) => return format!(r#"{{"ok":false,"error":"query_failed","detail":"{}"}}"#, e),
14539:     };
14540: 
14541:     if batch_rows.is_empty() {
14542:         // No more rows at this offset - close JSON and push
14543:         {
14544:             if let Ok(mut f) = std::fs::OpenOptions::new().append(true).open(&tmp_path) {
14545:                 let _ = f.write_all(b"]}");
14546:             }
14547:         }
14548:         // Close JSON - file stays for download
14549:         {
14550:             if let Ok(mut f) = std::fs::OpenOptions::new().append(true).open(&tmp_path) {
14551:                 let _ = f.write_all(b"]}");
14552:             }
14553:         }
14554:         return format!(
14555:             r#"{{"ok":true,"version":"3.22.91","table":"{}","total_rows":{},"offset":{},"rows_queried":0,"complete":true,"download_url":"/debug/download_table?name={}"}}"#,
14556:             json_escape(table_name),
14557:             total,
14558:             offset,
14559:             json_escape(table_name)
14560:         );
14561:     }
14562: 
14563:     // Append rows to file
14564:     let mut append_data = String::new();
14565:     // If offset > 0, we need a comma before the first row of this batch
14566:     if offset > 0 {
14567:         append_data.push(',');
14568:     }
14569:     append_data.push_str(&batch_rows.join(","));
14570: 
14571:     {
14572:         if let Ok(mut f) = std::fs::OpenOptions::new().append(true).open(&tmp_path) {
14573:             let _ = f.write_all(append_data.as_bytes());
14574:         }
14575:     }
14576: 
14577:     let rows_queried = batch_rows.len();
14578:     let next_offset = offset + rows_queried;
14579:     let is_last_batch = (next_offset as i64) >= total || rows_queried < batch;
14580: 
14581:     if !is_last_batch {
14582:         // Not done yet - return progress
14583:         return format!(
14584:             r#"{{"ok":true,"version":"3.22.91","table":"{}","total_rows":{},"offset":{},"rows_queried":{},"next_offset":{},"complete":false}}"#,
14585:             json_escape(table_name),
14586:             total,
14587:             offset,
14588:             rows_queried,
14589:             next_offset
14590:         );
14591:     }
14592: 
14593:     // Last batch! Close JSON - file stays for download
14594:     {
14595:         if let Ok(mut f) = std::fs::OpenOptions::new().append(true).open(&tmp_path) {
14596:             let _ = f.write_all(b"]}");
14597:         }
14598:     }
14599: 
14600:     format!(
14601:         r#"{{"ok":true,"version":"3.22.91","table":"{}","total_rows":{},"offset":{},"rows_queried":{},"complete":true,"download_url":"/debug/download_table?name={}"}}"#,
14602:         json_escape(table_name),
14603:         total,
14604:         offset,
14605:         rows_queried,
14606:         json_escape(table_name)
14607:     )
14608: }
14609: 
14610: /// /debug/mdb_all_tables - Dump ALL table names from mdb with row counts,
14611: /// plus search for tables related to skill unlock conditions (bond thresholds, prerequisites)
14612: /// /debug/hint_gain - Dump single_mode_hint_gain table (support card skill hint acquisition conditions)
14613: /// Plus resolve condition_set_id -> single_mode_story_condition_set details
14614: /// /debug/sc_effect - Dump support_card_effect_table + effect_filter + effect_filter_group
14615: /// These tables likely contain the activation conditions for support card unique effects
14616: /// /debug/unique_detail - Join support_card_data + support_card_unique_effect
14617: /// Shows each card with its unique effect types and values for decoding
14618: 
14619: /// /debug/download_table?name=<table_name>&batch=<N>
14620: /// Auto-batch build + download: queries all rows in batches, writes to local file, returns full JSON.
```

## lines 14667-14707

```rust
14667:             row.get(0)
14668:         }) {
14669:             Ok(t) => t,
14670:             Err(e) => return format!(r#"{{"ok":false,"error":"count_failed","detail":"{}"}}"#, e),
14671:         };
14672: 
14673:     // Write JSON header
14674:     let mut f = match std::fs::File::create(&tmp_path) {
14675:         Ok(file) => file,
14676:         Err(e) => {
14677:             return format!(
14678:                 r#"{{"ok":false,"error":"file_create_failed","detail":"{}"}}"#,
14679:                 e
14680:             )
14681:         }
14682:     };
14683:     if let Err(e) = f.write_all(
14684:         format!(
14685:             r#"{{"table":"{}","total_rows":{},"rows":["#,
14686:             json_escape(table_name),
14687:             total
14688:         )
14689:         .as_bytes(),
14690:     ) {
14691:         let _ = std::fs::remove_file(&tmp_path);
14692:         return format!(
14693:             r#"{{"ok":false,"error":"header_write_failed","detail":"{}"}}"#,
14694:             e
14695:         );
14696:     }
14697: 
14698:     let mut offset = 0usize;
14699:     let mut need_comma = false;
14700:     loop {
14701:         let query = format!(
14702:             "SELECT * FROM [{}] LIMIT {} OFFSET {}",
14703:             table_name, batch, offset
14704:         );
14705:         let rows = match conn.prepare(&query) {
14706:             Ok(mut stmt) => {
14707:                 let column_count = stmt.column_count();
```

## lines 14753-14812

```rust
14753:         let mut append_data = String::new();
14754:         if need_comma {
14755:             append_data.push(',');
14756:         }
14757:         append_data.push_str(&rows.join(","));
14758:         {
14759:             let mut f = match std::fs::OpenOptions::new().append(true).open(&tmp_path) {
14760:                 Ok(file) => file,
14761:                 Err(e) => {
14762:                     let _ = std::fs::remove_file(&tmp_path);
14763:                     return format!(
14764:                         r#"{{"ok":false,"error":"append_open_failed","detail":"{}"}}"#,
14765:                         e
14766:                     );
14767:                 }
14768:             };
14769:             if let Err(e) = f.write_all(append_data.as_bytes()) {
14770:                 let _ = std::fs::remove_file(&tmp_path);
14771:                 return format!(
14772:                     r#"{{"ok":false,"error":"append_write_failed","detail":"{}"}}"#,
14773:                     e
14774:                 );
14775:             }
14776:         }
14777:         need_comma = true;
14778: 
14779:         offset += rows.len();
14780:         if offset as i64 >= total || rows.len() < batch {
14781:             break;
14782:         }
14783:     }
14784: 
14785:     // Close JSON
14786:     {
14787:         if let Ok(mut f) = std::fs::OpenOptions::new().append(true).open(&tmp_path) {
14788:             let _ = f.write_all(b"]}");
14789:         }
14790:     }
14791:     // For very large tables, don't try to read the whole file into memory.
14792:     // Return a pointer instead - the user can access it via the file path.
14793:     let file_size = match std::fs::metadata(&tmp_path) {
14794:         Ok(m) => m.len() as usize,
14795:         Err(e) => return format!(r#"{{"ok":false,"error":"stat_failed","detail":"{}"}}"#, e),
14796:     };
14797:     // If file > 2MB, return metadata instead of reading into memory
14798:     if file_size > 2_000_000 {
14799:         return format!(
14800:             r#"{{"ok":true,"version":"3.22.91","table":"{}","total_rows":{},"file_size":{},"file_path":"{}","hint":"file too large for HTTP response, use push_table batch mode instead"}}"#,
14801:             json_escape(table_name),
14802:             total,
14803:             file_size,
14804:             tmp_path
14805:         );
14806:     }
14807: 
14808:     // Return the file
14809:     match std::fs::read_to_string(&tmp_path) {
14810:         Ok(content) => content,
14811:         Err(e) => format!(r#"{{"ok":false,"error":"read_failed","detail":"{}"}}"#, e),
14812:     }
```

## lines 21704-21744

```rust
21704:                 Ok(_) => {}
21705:                 Err(e) => {
21706:                     return format!(r#"{{"error":"rename_fallback_failed","detail":"{}"}}"#, e)
21707:                 }
21708:             }
21709:         }
21710:         Err(_) => {
21711:             // Last resort: /sdcard/Download
21712:             let sd_path = "/sdcard/Download/libhachimi_ura.so";
21713:             match std::fs::write(sd_path, &data) {
21714:                 Ok(_) => {
21715:                     return format!(
21716:                         r#"{{"status":"downloaded_to_sdcard","old":"{}","new":"{}","path":"{}","hint":"install_manually_then_restart"}}"#,
21717:                         current_ver, tag_name, sd_path
21718:                     );
21719:                 }
21720:                 Err(e) => return format!(r#"{{"error":"write_all_failed","detail":"{}"}}"#, e),
21721:             }
21722:         }
21723:     }
21724:     format!(
21725:         r#"{{"status":"updated_to_fallback","old":"{}","new":"{}","so_path":"{}","hint":"restart_game_to_apply"}}"#,
21726:         current_ver, tag_name, fb_path
21727:     )
21728: }
21729: 
21730: /// Find our own .so file path by scanning /proc/self/maps
21731: fn find_own_so_path() -> Option<String> {
21732:     let maps = std::fs::read_to_string("/proc/self/maps").ok()?;
21733:     for line in maps.lines() {
21734:         let path = line.split_whitespace().last()?;
21735:         if path.contains("libhachimi_") && path.ends_with(".so") {
21736:             return Some(path.to_string());
21737:         }
21738:     }
21739:     None
21740: }
21741: 
21742: /// Extract a JSON string value by key from a JSON body (simple, no parser)
21743: fn extract_json_string(body: &str, key: &str) -> Option<String> {
21744:     let pattern = format!(r##""{}":"##, key);
```

## lines 22589-22629

```rust
22589:                 output.push(value);
22590:                 index += 3;
22591:             }
22592:             b'+' => {
22593:                 output.push(b' ');
22594:                 index += 1;
22595:             }
22596:             value => {
22597:                 output.push(value);
22598:                 index += 1;
22599:             }
22600:         }
22601:     }
22602:     String::from_utf8(output).map_err(|_| "query_not_utf8".to_string())
22603: }
22604: 
22605: fn parse_request_uri(request: &str) -> Result<String, String> {
22606:     let line = request.lines().next().ok_or_else(|| "missing_request_line".to_string())?;
22607:     let mut parts = line.split_whitespace();
22608:     let method = parts.next().ok_or_else(|| "missing_http_method".to_string())?;
22609:     let uri = parts.next().ok_or_else(|| "missing_request_uri".to_string())?;
22610:     let version = parts.next().ok_or_else(|| "missing_http_version".to_string())?;
22611:     if method.is_empty() || !version.starts_with("HTTP/") || parts.next().is_some() {
22612:         return Err("invalid_request_line".to_string());
22613:     }
22614:     Ok(uri.to_string())
22615: }
22616: 
22617: fn parse_query_pairs(uri: &str) -> Result<Vec<(String, String)>, String> {
22618:     let query = match uri.split_once('?') {
22619:         Some((_, value)) => value.split('#').next().unwrap_or(""),
22620:         None => return Ok(Vec::new()),
22621:     };
22622:     let mut pairs = Vec::new();
22623:     for item in query.split('&') {
22624:         if item.is_empty() { continue; }
22625:         let (raw_key, raw_value) = item.split_once('=').unwrap_or((item, ""));
22626:         pairs.push((percent_decode_component(raw_key)?, percent_decode_component(raw_value)?));
22627:     }
22628:     Ok(pairs)
22629: }
```

## lines 23024-23073

```rust
23024:     let _write_guard = GLOBAL_OBSERVATION_WRITE_LOCK
23025:         .lock().map_err(|_| "global_observation_write_lock_poisoned".to_string())?;
23026:     let session_id = ensure_observation_session()?;
23027:     let sequence = GLOBAL_OBSERVATION_SEQUENCE.fetch_add(1, Ordering::SeqCst).saturating_add(1);
23028:     let timestamp_ms = sniff_timestamp_ms();
23029:     let session_directory = observation_storage_root().join("sessions").join(&session_id);
23030:     std::fs::create_dir_all(&session_directory)
23031:         .map_err(|error| format!("create_global_observation_dir:{}", error))?;
23032:     let journal_path = session_directory.join("timeline.ndjson");
23033:     let line = format!(
23034:         r#"{{"session_id":"{}","sequence":{},"timestamp_ms":{},"type":"{}","completeness":"{}","payload":{}}}\n"#,
23035:         json_escape(&session_id), sequence, timestamp_ms, json_escape(observation_type),
23036:         json_escape(completeness), payload_json
23037:     );
23038:     let mut file = std::fs::OpenOptions::new().create(true).append(true).open(&journal_path)
23039:         .map_err(|error| format!("open_global_observation_journal:{}", error))?;
23040:     std::io::Write::write_all(&mut file, line.as_bytes())
23041:         .map_err(|error| format!("append_global_observation:{}", error))?;
23042:     if critical {
23043:         file.sync_data().map_err(|error| format!("sync_global_observation:{}", error))?;
23044:     }
23045:     let byte_length = file.metadata().map_err(|error| format!("stat_global_observation:{}", error))?.len();
23046:     drop(file);
23047:     let connection = open_observation_storage()?;
23048:     connection.execute(
23049:         "INSERT OR REPLACE INTO observation_files(
23050:              session_id, relative_path, content_type, byte_length, sha256, created_at_ms
23051:          ) VALUES(?1, 'timeline.ndjson', 'application/x-ndjson', ?2, NULL, ?3)",
23052:         rusqlite::params![session_id, byte_length as i64, timestamp_ms as i64],
23053:     ).map_err(|error| format!("index_global_observation:{}", error))?;
23054:     STORAGE_LAST_FLUSH_MS.store(timestamp_ms, Ordering::Release);
23055:     storage_clear_error();
23056:     Ok((session_id, sequence, timestamp_ms))
23057: }
23058: 
23059: // ===== Protocol multi-section event timeline O-stage =====
23060: #[derive(Default)]
23061: struct ProtocolSectionScan {
23062:     turn_panel_paths: Vec<String>,
23063:     event_paths: Vec<String>,
23064:     choice_prompt_paths: Vec<String>,
23065:     choice_result_paths: Vec<String>,
23066:     training_paths: Vec<String>,
23067:     choice_index: Option<i64>,
23068:     story_id: Option<i64>,
23069:     event_id: Option<i64>,
23070:     decode_error: Option<String>,
23071: }
23072: 
23073: #[derive(Clone)]
```

## lines 23304-23356

```rust
23304:     let session_id = ensure_observation_session()?;
23305:     let now = sniff_timestamp_ms();
23306:     let suffix = if direction == "response" { format!("{}-{}", request_id, now) } else { request_id.to_string() };
23307:     let relative_base = format!("protocol/{}/{}", direction, suffix);
23308:     let session_dir = observation_storage_root().join("sessions").join(&session_id);
23309:     let target_dir = session_dir.join(&relative_base);
23310:     std::fs::create_dir_all(&target_dir).map_err(|error| format!("create_protocol_dir:{}", error))?;
23311:     let files: [(&str, &[u8], &str); 3] = [
23312:         ("url.txt", url.as_bytes(), "text/plain; charset=utf-8"),
23313:         ("headers.raw", headers, "application/octet-stream"),
23314:         ("payload.bin", payload, "application/octet-stream"),
23315:     ];
23316:     for (name, bytes, _) in &files {
23317:         let temporary = target_dir.join(format!("{}.tmp", name));
23318:         let mut file = std::fs::File::create(&temporary)
23319:             .map_err(|error| format!("create_protocol_file:{}:{}", name, error))?;
23320:         std::io::Write::write_all(&mut file, bytes)
23321:             .map_err(|error| format!("write_protocol_file:{}:{}", name, error))?;
23322:         file.sync_data().map_err(|error| format!("sync_protocol_file:{}:{}", name, error))?;
23323:         drop(file);
23324:         std::fs::rename(&temporary, target_dir.join(name))
23325:             .map_err(|error| format!("commit_protocol_file:{}:{}", name, error))?;
23326:     }
23327:     let mut connection = open_observation_storage()?;
23328:     let transaction = connection.transaction().map_err(|error| format!("protocol_index_transaction:{}", error))?;
23329:     for (name, bytes, content_type) in &files {
23330:         let relative = format!("{}/{}", relative_base, name);
23331:         transaction.execute(
23332:             "INSERT OR REPLACE INTO observation_files(session_id, relative_path, content_type, byte_length, sha256, created_at_ms) VALUES(?1, ?2, ?3, ?4, NULL, ?5)",
23333:             rusqlite::params![session_id, relative, content_type, bytes.len() as i64, now as i64],
23334:         ).map_err(|error| format!("index_protocol_file:{}:{}", name, error))?;
23335:     }
23336:     transaction.commit().map_err(|error| format!("commit_protocol_index:{}", error))?;
23337:     persist_protocol_observation_boundary(
23338:         direction, request_id, url, &relative_base, headers.len(), payload.len()
23339:     )?;
23340:     persist_protocol_semantic_timeline(direction, request_id, url, &relative_base, payload)?;
23341:     storage_clear_error();
23342:     Ok(())
23343: }
23344: 
23345: fn observation_storage_root() -> std::path::PathBuf {
23346:     if let Ok(command_line) = std::fs::read("/proc/self/cmdline") {
23347:         let package_bytes = command_line.split(|byte| *byte == 0).next().unwrap_or(&[]);
23348:         if let Ok(package_name) = std::str::from_utf8(package_bytes) {
23349:             if !package_name.is_empty() {
23350:                 return std::path::PathBuf::from("/data/user/0")
23351:                     .join(package_name)
23352:                     .join("files")
23353:                     .join("hlpatch-observations");
23354:             }
23355:         }
23356:     }
```

## lines 23376-23428

```rust
23376:          PRAGMA foreign_keys=ON;
23377:          CREATE TABLE IF NOT EXISTS storage_meta(
23378:              key TEXT PRIMARY KEY NOT NULL,
23379:              value TEXT NOT NULL
23380:          );
23381:          CREATE TABLE IF NOT EXISTS observation_sessions(
23382:              session_id TEXT PRIMARY KEY NOT NULL,
23383:              process_id INTEGER NOT NULL,
23384:              process_start_token TEXT NOT NULL DEFAULT '',
23385:              plugin_version TEXT NOT NULL,
23386:              started_at_ms INTEGER NOT NULL,
23387:              last_flush_ms INTEGER NOT NULL,
23388:              state TEXT NOT NULL,
23389:              recovered_after_restart INTEGER NOT NULL DEFAULT 0,
23390:              root_path TEXT NOT NULL
23391:          );
23392:          CREATE TABLE IF NOT EXISTS observation_files(
23393:              file_id INTEGER PRIMARY KEY AUTOINCREMENT,
23394:              session_id TEXT NOT NULL,
23395:              relative_path TEXT NOT NULL,
23396:              content_type TEXT NOT NULL,
23397:              byte_length INTEGER NOT NULL,
23398:              sha256 TEXT,
23399:              created_at_ms INTEGER NOT NULL,
23400:              UNIQUE(session_id, relative_path),
23401:              FOREIGN KEY(session_id) REFERENCES observation_sessions(session_id)
23402:          );
23403:          CREATE INDEX IF NOT EXISTS idx_observation_files_session_id_file_id
23404:              ON observation_files(session_id, file_id);"
23405:     ).map_err(|error| format!("initialize_schema:{}", error))?;
23406:     let has_start_token = connection.prepare("PRAGMA table_info(observation_sessions)")
23407:         .and_then(|mut statement| statement.query_map([], |row| row.get::<_, String>(1))
23408:             .map(|rows| rows.filter_map(Result::ok).any(|name| name == "process_start_token")))
23409:         .unwrap_or(false);
23410:     if !has_start_token {
23411:         connection.execute("ALTER TABLE observation_sessions ADD COLUMN process_start_token TEXT NOT NULL DEFAULT ''", [])
23412:             .map_err(|error| format!("migrate_process_start_token:{}", error))?;
23413:     }
23414:     Ok(connection)
23415: }
23416: 
23417: fn observation_process_start_token() -> String {
23418:     let stat = std::fs::read_to_string("/proc/self/stat").unwrap_or_default();
23419:     let start_ticks = stat.rsplit_once(')').map(|(_, tail)| tail.split_whitespace().nth(19).unwrap_or("")).unwrap_or("");
23420:     format!("{}:{}", std::process::id(), start_ticks)
23421: }
23422: 
23423: fn ensure_observation_session() -> Result<String, String> {
23424:     if let Ok(value) = STORAGE_SESSION_ID.lock() {
23425:         if let Some(session_id) = value.as_ref() {
23426:             return Ok(session_id.clone());
23427:         }
23428:     }
```

## lines 23447-23487

```rust
23447:     ).map_err(|error| format!("insert_session:{}", error))?;
23448:     let session_directory = observation_storage_root().join("sessions").join(&session_id);
23449:     if let Err(error) = std::fs::create_dir_all(&session_directory) {
23450:         let _ = connection.execute("DELETE FROM observation_sessions WHERE session_id=?1", rusqlite::params![session_id]);
23451:         return Err(format!("create_session_dir:{}", error));
23452:     }
23453:     let session_json = format!(
23454:         r#"{{"session_id":"{}","process_id":{},"plugin_version":"{}","started_at_ms":{},"state":"open","recovered_after_restart":false,"root_path":"{}"}}"#,
23455:         json_escape(&session_id), process_id, json_escape(PLUGIN_VERSION), now, json_escape(&root_text)
23456:     );
23457:     if let Err(error) = std::fs::write(session_directory.join("session.json"), session_json.as_bytes()) {
23458:         let _ = connection.execute("DELETE FROM observation_sessions WHERE session_id=?1", rusqlite::params![session_id]);
23459:         let _ = std::fs::remove_dir_all(&session_directory);
23460:         return Err(format!("write_session_json:{}", error));
23461:     }
23462:     if let Err(error) = connection.execute(
23463:         "INSERT OR REPLACE INTO observation_files(
23464:              session_id, relative_path, content_type, byte_length, sha256, created_at_ms
23465:          ) VALUES(?1, 'session.json', 'application/json', ?2, NULL, ?3)",
23466:         rusqlite::params![session_id, session_json.as_bytes().len() as i64, now as i64],
23467:     ) {
23468:         let _ = connection.execute("DELETE FROM observation_sessions WHERE session_id=?1", rusqlite::params![session_id]);
23469:         let _ = std::fs::remove_dir_all(&session_directory);
23470:         return Err(format!("index_session_json:{}", error));
23471:     }
23472:     STORAGE_LAST_FLUSH_MS.store(now, Ordering::Relaxed);
23473:     let mut state = STORAGE_SESSION_ID.lock().map_err(|_| "storage_session_lock_poisoned".to_string())?;
23474:     *state = Some(session_id.clone());
23475:     Ok(session_id)
23476: }
23477: 
23478: fn storage_status_endpoint() -> String {
23479:     let root = observation_storage_root();
23480:     let db_path = observation_storage_db_path();
23481:     let session = ensure_observation_session();
23482:     if let Err(error) = session.as_ref() { storage_set_error(error); }
23483:     let current_session = session.ok();
23484:     let writable_probe_path = root.join(".write_probe");
23485:     let writable = std::fs::write(&writable_probe_path, b"hlpatch-storage-probe")
23486:         .and_then(|_| std::fs::remove_file(&writable_probe_path)).is_ok();
23487:     let error = STORAGE_LAST_ERROR.lock().ok().and_then(|value| value.clone());
```

## lines 23901-23990

```rust
23901: }
23902: 
23903: // ===== Unified selected-parent multi-source resolver I-stage =====
23904: // ===== Selected-parent runtime semantics J-stage =====
23905: // ===== Unified K complete observation endpoints =====
23906: fn k_json_error(error: &str) -> String {
23907:     format!(r#"{{"ok":false,"error":"{}"}}"#, json_escape(error))
23908: }
23909: 
23910: fn k_file_sha256(bytes: &[u8]) -> String {
23911:     use sha2::Digest;
23912:     let mut hasher = sha2::Sha256::new();
23913:     hasher.update(bytes);
23914:     hex_encode(&hasher.finalize())
23915: }
23916: 
23917: fn storage_files_endpoint(uri: &str) -> String {
23918:     let pairs = match parse_query_pairs(uri) { Ok(v) => v, Err(e) => return k_json_error(&e) };
23919:     let session_id = query_pair(&pairs, "session_id");
23920:     if session_id.is_empty() { return k_json_error("missing_session_id"); }
23921:     let cursor = query_pair(&pairs, "cursor").parse::<i64>().unwrap_or(0).max(0);
23922:     let limit = query_pair(&pairs, "limit").parse::<i64>().unwrap_or(200).clamp(1, 1000);
23923:     let connection = match open_observation_storage() { Ok(v) => v, Err(e) => return k_json_error(&e) };
23924:     let mut statement = match connection.prepare(
23925:         "SELECT file_id,relative_path,content_type,byte_length,sha256,created_at_ms FROM observation_files WHERE session_id=?1 AND file_id>?2 ORDER BY file_id LIMIT ?3"
23926:     ) { Ok(v) => v, Err(e) => return k_json_error(&format!("prepare_storage_files:{}", e)) };
23927:     let rows = match statement.query_map(rusqlite::params![session_id, cursor, limit], |row| {
23928:         let file_id=row.get::<_,i64>(0)?;
23929:         let path=row.get::<_,String>(1)?;
23930:         let content_type=row.get::<_,String>(2)?;
23931:         let byte_length=row.get::<_,i64>(3)?;
23932:         let sha=row.get::<_,Option<String>>(4)?;
23933:         let created=row.get::<_,i64>(5)?;
23934:         Ok((file_id,path,content_type,byte_length,sha,created))
23935:     }) { Ok(v) => v, Err(e) => return k_json_error(&format!("query_storage_files:{}", e)) };
23936:     let mut items=Vec::new();
23937:     let mut next=cursor;
23938:     for row in rows {
23939:         let (id,path,content_type,len,sha,created)=match row { Ok(v)=>v, Err(e)=>return k_json_error(&format!("decode_storage_file:{}",e)) };
23940:         next=id;
23941:         let sha_json=sha.map(|v|format!("\"{}\"",json_escape(&v))).unwrap_or_else(||"null".to_string());
23942:         items.push(format!(r#"{{"file_id":{},"session_id":"{}","relative_path":"{}","content_type":"{}","byte_length":{},"sha256":{},"created_at_ms":{}}}"#,
23943:             id,json_escape(&session_id),json_escape(&path),json_escape(&content_type),len,sha_json,created));
23944:     }
23945:     format!(r#"{{"ok":true,"session_id":"{}","cursor":{},"next_cursor":{},"count":{},"files":[{}]}}"#,
23946:         json_escape(&session_id),cursor,next,items.len(),items.join(","))
23947: }
23948: 
23949: fn storage_download(uri: &str) -> String {
23950:     let pairs=match parse_query_pairs(uri){Ok(v)=>v,Err(e)=>return k_json_error(&e)};
23951:     let file_id=match query_pair(&pairs,"file_id").parse::<i64>(){Ok(v) if v>0=>v,_=>return k_json_error("invalid_or_missing_file_id")};
23952:     let connection=match open_observation_storage(){Ok(v)=>v,Err(e)=>return k_json_error(&e)};
23953:     let record=connection.query_row(
23954:         "SELECT session_id,relative_path,content_type,byte_length FROM observation_files WHERE file_id=?1",
23955:         rusqlite::params![file_id],|row|Ok((row.get::<_,String>(0)?,row.get::<_,String>(1)?,row.get::<_,String>(2)?,row.get::<_,i64>(3)?)));
23956:     let (session_id,relative,content_type,indexed_len)=match record{
23957:         Ok(v)=>v,Err(rusqlite::Error::QueryReturnedNoRows)=>return k_json_error("file_not_found"),Err(e)=>return k_json_error(&format!("query_file:{}",e))};
23958:     let session_root=observation_storage_root().join("sessions").join(&session_id);
23959:     let target=session_root.join(&relative);
23960:     let canonical_root=match session_root.canonicalize(){Ok(v)=>v,Err(e)=>return k_json_error(&format!("canonical_session_root:{}",e))};
23961:     let canonical_target=match target.canonicalize(){Ok(v)=>v,Err(e)=>return k_json_error(&format!("canonical_file:{}",e))};
23962:     if !canonical_target.starts_with(&canonical_root){return k_json_error("file_outside_session_root");}
23963:     let bytes=match std::fs::read(&canonical_target){Ok(v)=>v,Err(e)=>return k_json_error(&format!("read_file:{}",e))};
23964:     if indexed_len>=0 && indexed_len as usize!=bytes.len(){return k_json_error("indexed_length_mismatch");}
23965:     let sha=k_file_sha256(&bytes);
23966:     let _=connection.execute("UPDATE observation_files SET sha256=?1,byte_length=?2 WHERE file_id=?3",rusqlite::params![sha,bytes.len() as i64,file_id]);
23967:     format!(r#"{{"ok":true,"file_id":{},"session_id":"{}","relative_path":"{}","content_type":"{}","byte_length":{},"sha256":"{}","encoding":"hex","body_hex":"{}"}}"#,
23968:         file_id,json_escape(&session_id),json_escape(&relative),json_escape(&content_type),bytes.len(),sha,hex_encode(&bytes))
23969: }
23970: 
23971: unsafe fn k_resolve_method(uri: &str) -> Result<MethodIndexEntry,String> {
23972:     let pairs=parse_query_pairs(uri)?;
23973:     let requested=query_pair(&pairs,"declaring_type");
23974:     let method_name=query_pair(&pairs,"method");
23975:     let parameter_text=query_pair(&pairs,"parameter_types");
23976:     if requested.is_empty()||method_name.is_empty(){return Err("missing_declaring_type_or_method".to_string());}
23977:     let wanted:Vec<String>=if parameter_text.is_empty(){Vec::new()}else{parameter_text.split(',').map(|v|v.trim().to_string()).collect()};
23978:     let class=find_class_by_full_declaring_name(&requested);
23979:     if class.is_null(){return Err("class_not_found_or_ambiguous".to_string());}
23980:     let names=["il2cpp_class_get_methods","il2cpp_method_get_name","il2cpp_method_get_param_count","il2cpp_method_get_param","il2cpp_type_get_name","il2cpp_method_get_return_type","il2cpp_method_get_flags"];
23981:     let p:Vec<*mut c_void>=names.iter().map(|n|resolve_il2cpp_symbol(n)).collect();
23982:     if let Some(i)=p.iter().position(|v|v.is_null()){return Err(format!("missing_symbol:{}",names[i]));}
23983:     let get_methods:FnClassGetMethods=std::mem::transmute(p[0]);
23984:     let get_name:FnMethodGetName=std::mem::transmute(p[1]);
23985:     let get_count:unsafe extern "C" fn(*const c_void)->u32=std::mem::transmute(p[2]);
23986:     let get_param:unsafe extern "C" fn(*const c_void,u32)->*const c_void=std::mem::transmute(p[3]);
23987:     let get_type_name:unsafe extern "C" fn(*const c_void)->*const c_char=std::mem::transmute(p[4]);
23988:     let get_return:unsafe extern "C" fn(*const c_void)->*const c_void=std::mem::transmute(p[5]);
23989:     let get_flags:unsafe extern "C" fn(*const c_void,*mut u32)->u32=std::mem::transmute(p[6]);
23990:     let mut iterator=ptr::null_mut();
```

## lines 24071-24111

```rust
24071: fn protocol_path_request_id(relative_path: &str, direction: &str) -> Option<u64> {
24072:     let prefix = format!("protocol/{}/", direction);
24073:     let remainder = relative_path.strip_prefix(&prefix)?;
24074:     let component = remainder.split('/').next()?;
24075:     let numeric = if direction == "response" {
24076:         component.split('-').next().unwrap_or(component)
24077:     } else {
24078:         component
24079:     };
24080:     numeric.parse::<u64>().ok()
24081: }
24082: 
24083: fn protocol_archive_rows(session_id: &str) -> Result<Vec<(i64, String, String, i64, Option<String>, i64)>, String> {
24084:     let connection = open_observation_storage()?;
24085:     let mut statement = connection.prepare(
24086:         "SELECT file_id,relative_path,content_type,byte_length,sha256,created_at_ms \
24087:          FROM observation_files WHERE session_id=?1 AND relative_path LIKE 'protocol/%' \
24088:          ORDER BY file_id"
24089:     ).map_err(|error| format!("prepare_protocol_archive:{}", error))?;
24090:     let mapped = statement.query_map(rusqlite::params![session_id], |row| Ok((
24091:         row.get::<_, i64>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?,
24092:         row.get::<_, i64>(3)?, row.get::<_, Option<String>>(4)?, row.get::<_, i64>(5)?,
24093:     ))).map_err(|error| format!("query_protocol_archive:{}", error))?;
24094:     let mut rows = Vec::new();
24095:     for row in mapped {
24096:         rows.push(row.map_err(|error| format!("decode_protocol_archive:{}", error))?);
24097:     }
24098:     Ok(rows)
24099: }
24100: 
24101: fn protocol_file_json(row: &(i64, String, String, i64, Option<String>, i64)) -> String {
24102:     let sha = row.4.as_ref().map(|value| format!("\"{}\"", json_escape(value)))
24103:         .unwrap_or_else(|| "null".to_string());
24104:     format!(r#"{{"file_id":{},"relative_path":"{}","content_type":"{}","byte_length":{},"sha256":{},"created_at_ms":{},"download":"/storage/download?file_id={}"}}"#,
24105:         row.0, json_escape(&row.1), json_escape(&row.2), row.3, sha, row.5, row.0)
24106: }
24107: 
24108: fn protocol_exchange_export_endpoint(uri: &str) -> String {
24109:     let pairs = match parse_query_pairs(uri) { Ok(value) => value, Err(error) => return k_json_error(&error) };
24110:     let session_id = query_pair(&pairs, "session_id");
24111:     if session_id.is_empty() { return k_json_error("missing_session_id"); }
```

## lines 24166-24227

```rust
24166:             Ok(metadata) if metadata.len() != row.3.max(0) as u64 => audit.indexed_length_mismatches.push(row.1.clone()),
24167:             Err(_) => audit.indexed_length_mismatches.push(row.1.clone()),
24168:             _ => {}
24169:         }
24170:     }
24171:     let missing_response: Vec<u64> = audit.request_ids.difference(&audit.response_ids).copied().collect();
24172:     let orphan_response: Vec<u64> = audit.response_ids.difference(&audit.request_ids).copied().collect();
24173:     let u64_json = |values: &[u64]| values.iter().map(|value| value.to_string()).collect::<Vec<_>>().join(",");
24174:     let path_json = |values: &[String]| values.iter().map(|value| format!("\"{}\"", json_escape(value))).collect::<Vec<_>>().join(",");
24175:     format!(r#"{{"ok":true,"session_id":"{}","request_count":{},"response_count":{},"paired_count":{},"request_file_count":{},"response_file_count":{},"request_bytes":{},"response_bytes":{},"missing_response_ids":[{}],"orphan_response_ids":[{}],"zero_length_files":[{}],"indexed_length_mismatches":[{}]}}"#,
24176:         json_escape(&session_id), audit.request_ids.len(), audit.response_ids.len(),
24177:         audit.request_ids.intersection(&audit.response_ids).count(), audit.request_files, audit.response_files,
24178:         audit.request_bytes, audit.response_bytes, u64_json(&missing_response), u64_json(&orphan_response),
24179:         path_json(&audit.zero_length_files), path_json(&audit.indexed_length_mismatches))
24180: }
24181: 
24182: fn k_observation_files(domain: &str, uri: &str) -> String {
24183:     let pairs=match parse_query_pairs(uri){Ok(v)=>v,Err(e)=>return k_json_error(&e)};let requested_session=query_pair(&pairs,"session_id");let connection=match open_observation_storage(){Ok(v)=>v,Err(e)=>return k_json_error(&e)};
24184:     let session_id=if requested_session.is_empty(){match ensure_observation_session(){Ok(v)=>v,Err(e)=>return k_json_error(&e)}}else{requested_session};
24185:     let token=domain.replace('/',"_");let like=format!("%{}%",token);
24186:     let mut statement=match connection.prepare("SELECT file_id,relative_path,content_type,byte_length,created_at_ms FROM observation_files WHERE session_id=?1 AND (relative_path LIKE ?2 OR relative_path LIKE '%protocol%') ORDER BY file_id") {Ok(v)=>v,Err(e)=>return k_json_error(&format!("prepare_domain_history:{}",e))};
24187:     let rows=match statement.query_map(rusqlite::params![session_id,like],|r|Ok((r.get::<_,i64>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?,r.get::<_,i64>(3)?,r.get::<_,i64>(4)?))){Ok(v)=>v,Err(e)=>return k_json_error(&format!("query_domain_history:{}",e))};
24188:     let mut items=Vec::new();for row in rows{let(id,path,ct,len,created)=match row{Ok(v)=>v,Err(e)=>return k_json_error(&format!("decode_domain_history:{}",e))};items.push(format!(r#"{{"file_id":{},"relative_path":"{}","content_type":"{}","byte_length":{},"created_at_ms":{}}}"#,id,json_escape(&path),json_escape(&ct),len,created));}
24189:     format!(r#"{{"ok":true,"domain":"{}","session_id":"{}","evidence_status":"observed_files","count":{},"files":[{}]}}"#,json_escape(domain),json_escape(&session_id),items.len(),items.join(","))
24190: }
24191: 
24192: unsafe fn inherit_tree_endpoint() -> String { inherit_selected_parent_records_endpoint() }
24193: fn factor_history_endpoint(uri:&str)->String{k_observation_files("factor/history",uri)}
24194: 
24195: fn k_domain_endpoint(path:&str,uri:&str)->String{
24196:     match path{
24197:         "/factor/history"=>factor_history_endpoint(uri),
24198:         "/factor/stats"=>k_observation_files("factor/stats",uri),
24199:         "/factor/probability_model"=>k_observation_files("factor/probability_model",uri),
24200:         "/factor/breeding_advice"=>k_observation_files("factor/breeding_advice",uri),
24201:         "/api/sniff/exchanges"=>protocol_exchanges_export_endpoint(uri),
24202:         "/api/sniff/exchange"=>protocol_exchange_export_endpoint(uri),
24203:         _=>k_observation_files(path.trim_start_matches('/'),uri),
24204:     }
24205: }
24206: // ===== Unified K complete observation endpoints =====
24207: // ===== Generated succession runtime L support fix =====
24208: // ===== Generated succession runtime decoder L =====
24209: unsafe fn l_array_objects(array: *mut c_void) -> Result<Vec<*mut c_void>, String> {
24210:     if array.is_null() || !is_readable_range(array as usize + 0x18, 8) { return Err("array_not_readable".to_string()); }
24211:     let len = std::ptr::read_unaligned::<usize>((array as usize + 0x18) as *const usize);
24212:     if len > 10000 { return Err(format!("array_length_out_of_range:{}", len)); }
24213:     if len > 0 && !is_readable_range(array as usize + 0x20, len * 8) { return Err("array_elements_not_readable".to_string()); }
24214:     Ok((0..len).map(|i| std::ptr::read_unaligned::<*mut c_void>((array as usize + 0x20 + i * 8) as *const *mut c_void)).collect())
24215: }
24216: 
24217: unsafe fn l_named_i32(object: *mut c_void, candidates: &[&str]) -> Option<i32> {
24218:     if object.is_null() || !is_readable_range(object as usize, 8) { return None; }
24219:     let class = std::ptr::read_unaligned::<*mut c_void>(object as *const *mut c_void);
24220:     let gf=resolve_il2cpp_symbol("il2cpp_class_get_fields"); let gn=resolve_il2cpp_symbol("il2cpp_field_get_name"); let go=resolve_il2cpp_symbol("il2cpp_field_get_offset");
24221:     if class.is_null() || gf.is_null() || gn.is_null() || go.is_null() { return None; }
24222:     let get_fields: unsafe extern "C" fn(*mut c_void,*mut *mut c_void)->*mut c_void=std::mem::transmute(gf);
24223:     let get_name: unsafe extern "C" fn(*mut c_void)->*const c_char=std::mem::transmute(gn);
24224:     let get_offset: unsafe extern "C" fn(*mut c_void)->i32=std::mem::transmute(go);
24225:     let mut it=ptr::null_mut();
24226:     loop { let f=get_fields(class,&mut it); if f.is_null(){break;} let name=il2cpp_c_string(get_name(f)); if candidates.iter().any(|v|*v==name){let off=get_offset(f);if off>=0&&is_readable_range(object as usize+off as usize,4){return Some(std::ptr::read_unaligned::<i32>((object as usize+off as usize) as *const i32));}} }
24227:     None
```
