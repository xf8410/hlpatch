# Pre-release endpoint focused source context

source_commit: `b8afd6ef9fe9cd0856bbb86cf245ba16dd7b40da`

## Route and advertised-endpoint lines

```rust
6819:     if path.starts_with("http://") || path.starts_with("https://") {
7356:         if m.path.starts_with("/dev/") || m.path=="[vvar]" || m.path=="[vdso]" { continue; }
7419:     // that touches game memory; static/self-state endpoints stay available.
7472:         let safe = BOOT_SAFE_EXACT.iter().any(|p| path == *p)
7473:             || BOOT_SAFE_PREFIX.iter().any(|p| path.starts_with(p));
7525:     let dl_enabled = !dl_flag.is_empty() && dl_flag != "0" && DL_ALLOWED.iter().any(|p| path == *p);
7528:     let body = if path == "/debug/global_metadata_probe" {
7530:     } else if path == "/debug/mem_scan_hex" {
7532:     } else if path == "/debug/mem_maps" {
7534:     } else if path == "/" || path == "/health" {
7536:             r#"{{"status":"ok","version":"{}","endpoints":["/summary","/data","/scenario","/debug/rameninfo","/debug/laststep","/event/recommend","/inherit/compat","/saddle-analysis","/log/turn","/debug/params","/debug/breeders","/debug/cmdinfo","/debug/training_partners","/debug/crashlog","/debug/upload","/debug/dumpclass","/debug/storydata","/debug/ramenfields","/debug/gauge","/debug/gauge2","/debug/ramengains","/debug/paramsincdec","/debug/training_seed","/debug/training_log","/debug/training_log_dl","/update","/update/status","/debug/all","/debug/unique_skills","/debug/mdb_all_tables","/debug/mdb_schema_dump","/debug/hint_gain","/debug/sc_effect","/debug/unique_detail","/debug/table","/debug/push_table","/debug/download_table","/mdb","/carddb","/skilldata","/hall","/saddles","/saddles-dl","/log","/status","/health","/mdb/schema","/mdb/search","/mdb/raw","/mdb/dl_batch","/il2cpp/dump","/il2cpp/call","/il2cpp/tree","/il2cpp/field","/il2cpp/classes","/il2cpp/static","/il2cpp/methods","/il2cpp/disassemble","/il2cpp/disassemble_dl","/il2cpp/disassemble_addr","/il2cpp/disassemble_addr_dl","/il2cpp/dump_all_methods","/il2cpp/dump_all_methods_dl","/il2cpp/search_float","/il2cpp/search_float_dl","/il2cpp/search_int","/il2cpp/search_int_dl","/il2cpp/search_methods","/il2cpp/search_methods_dl","/il2cpp/read_mem","/il2cpp/read_mem_dl","/training/result","/api/sniff","/api/sniff/metadata","/api/sniff/status","/api/sniff/toggle","/api/sniff/clear","/api/sniff/diag","/api/event/choices","/api/event/clear","/debug/hooklog","/debug/hookdiag","/debug/resource_meta_key","/debug/resource_db_keys","/debug/resource_reads","/debug/mem_scan_sqlite","/debug/meta_dump","/action/latest","/seed/history","/seed/stats","/debug/ramen_planner_state","/debug/ramen_participants","/debug/ramen_transition","/debug/ramen_dataset_path","/debug/ramen_formula_targets","/debug/event_reward_targets", "/debug/resource_storage","/debug/resource_meta_schema","/debug/resource_meta_probe", "/debug/resource_crypto_symbols","/debug/resource_meta_dl","/debug/resource_file_dl","/debug/private_file_inventory","/debug/private_file_dl"]}}"#,
7539:     } else if path == "/scan" {
7541:     } else if path == "/data" {
7547:     } else if path == "/status" {
7553:     } else if path == "/singletons" {
7555:     } else if path.starts_with("/find_method") {
7556:         let method_name = if path == "/find_method" || path == "/find_method/" {
7563:     } else if path.starts_with("/fields") {
7564:         let class_name = if path == "/fields" || path == "/fields/" {
7582:     } else if path.starts_with("/methods") {
7583:         let class_name = if path == "/methods" || path == "/methods/" {
7601:     } else if path == "/summary" {
7603:     } else if path == "/debug/turn_probe" {
7612:     } else if path == "/debug/ramen_transition" {
7617:     } else if path == "/ramen" {
7627:     } else if path == "/scenario" {
7633:     } else if path == "/log" {
7635:     } else if path == "/debug/params" {
7637:     } else if path == "/debug/breeders" {
7639:     } else if path == "/debug/rameninfo" {
7645:     } else if path == "/debug/laststep" {
7659:     } else if path == "/debug/crashlog" {
7661:     } else if path == "/debug/upload" {
7663:     } else if path == "/debug/cmdinfo" {
7665:     } else if path == "/debug/training_partners" {
7667:     } else if path == "/debug/ramen_participants" {
7669:     } else if path == "/training/result" {
7688:     } else if path == "/api/sniff/status" {
7713:     } else if path == "/api/sniff/metadata" {
7740:     } else if path == "/api/sniff/toggle" {
7773:     } else if path == "/api/sniff/clear" {
7786:     } else if path.starts_with("/debug/hooklog") {
7802:     } else if path == "/debug/resource_reads" {
7818:     } else if path.starts_with("/debug/mem_scan_sqlite") {
7881:     } else if path == "/debug/mem_scan_zdict" {
7956:     } else if path.starts_with("/debug/mem_scan_hex") {
8035:     } else if path.starts_with("/debug/file_scan_hex") {
8099:     } else if path == "/debug/maps_list" {
8124:     } else if path.starts_with("/debug/file_range_hex") {
8166:     } else if path == "/debug/meta_dump" {
8169:     } else if path == "/debug/resource_db_keys" {
8183:     } else if path == "/debug/resource_meta_key" {
8192:     } else if path == "/debug/hookdiag" {
8212:     } else if path.starts_with("/api/sniff/unity") {
8230:     } else if path == "/api/sniff/diag" {
8261:     } else if path == "/api/md5log" {
8275:     } else if path == "/api/md5log/clear" {
8278:     } else if path == "/api/md5log/install" {
8330:     } else if path == "/api/sniff" {
8379:     } else if path == "/api/event/choices" {
8405:     } else if path == "/api/event/observations" {
8426:     } else if path == "/api/event/observations/clear" {
8431:     } else if path == "/api/event/clear" {
8446:     } else if path == "/action/latest" {
8470:     } else if path == "/seed/history" || path == "/seed/stats" {
8472:     } else if path == "/debug/training_log" {
8479:     } else if path == "/debug/training_log_dl" {
8482:     } else if path == "/debug/event_reward_targets" {
8488:     } else if path == "/debug/ramen_formula_targets" {
8493:     } else if path == "/debug/ramen_dataset_path" {
8498:     } else if path == "/debug/ramen_planner_state" {
8503:     } else if path == "/debug/ramen_region_select" {
8508:     } else if path == "/debug/race_random_program_exact" {
8510:     } else if path.starts_with("/debug/dumpclass") {
8519:     } else if path == "/debug/storydata" {
8525:     } else if path == "/debug/all" {
8528:     } else if path == "/debug/ramenfields" {
8534:     } else if path == "/debug/gauge" {
8548:     } else if path == "/debug/gauge2" {
8563:     } else if path == "/debug/ramengains" {
8578:     } else if path == "/debug/paramsincdec" {
8593:     } else if path == "/debug/training_seed" {
8608:     } else if path == "/update" {
8611:     } else if path == "/update/status" {
8622:     } else if path == "/events" {
8624:     } else if path == "/debug/unique_skills" {
8626:     } else if path == "/debug/mdb_all_tables" {
8628:     } else if path == "/debug/mdb_schema_dump" {
8630:     } else if path == "/debug/hint_gain" {
8632:     } else if path == "/debug/sc_effect" {
8634:     } else if path == "/debug/unique_detail" {
8636:     } else if path == "/debug/table" {
8664:     } else if path == "/debug/download_table" {
8682:     } else if path == "/debug/push_table" {
8710:     } else if path == "/tables" {
8712:     } else if path == "/carddb" {
8714:     } else if path == "/skilldata" {
8716:     } else if path == "/hall" {
8718:     } else if path == "/event/recommend" {
8720:     } else if path == "/inherit/selected_parent_runtime" {
8722:     } else if path == "/inherit/pair_compat" {
8724:     } else if path == "/inherit/compat" {
8726:     } else if path == "/saddle-analysis" {
8728:     } else if path == "/log/turn" {
8730:     } else if path == "/ranking" {
8732:     } else if path == "/saddles-dl" {
8734:     } else if path == "/saddles" {
8736:     } else if path == "/config" {
8760:     } else if path == "/debug/dump" {
8764:     } else if path == "/config.html" {
8778:     } else if path.starts_with("/classes") {
8779:         let search = if path == "/classes" || path == "/classes/" {
8787:     } else if path.starts_with("/mdb/schema") {
8791:     } else if path.starts_with("/mdb/search") {
8795:     } else if path.starts_with("/mdb/raw") {
8799:     } else if path.starts_with("/mdb/dl_batch") {
8820:     } else if path.starts_with("/il2cpp/dump_all_methods_dl") {
8839:     } else if path.starts_with("/il2cpp/dump_all_methods") {
8843:     } else if path.starts_with("/il2cpp/dump") {
8847:     } else if path.starts_with("/il2cpp/invoke_instance") {
8859:     } else if path.starts_with("/il2cpp/invoke_static") {
8870:     } else if path.starts_with("/il2cpp/call_static") {
8881:     } else if path.starts_with("/il2cpp/call") {
8886:     } else if path.starts_with("/il2cpp/tree") {
8892:     } else if path.starts_with("/il2cpp/field") {
8897:     } else if path.starts_with("/il2cpp/classes") {
8901:     } else if path.starts_with("/il2cpp/static") {
8905:     } else if path == "/storage/status" {
8907:     } else if path == "/storage/sessions" {
8909:     } else if path == "/storage/session" {
8911:     } else if path == "/storage/flush" {
8913:     } else if path == "/storage/recover" {
8915:     } else if path == "/il2cpp/method_by_addr" {
8917:     } else if path == "/il2cpp/method_detail" {
8919:     } else if path == "/il2cpp/nested_types" {
8921:     } else if path == "/il2cpp/enum_values" {
8923:     } else if path.starts_with("/il2cpp/methods") {
8927:     } else if path.starts_with("/il2cpp/disassemble_dl") {
8934:     } else if path.starts_with("/il2cpp/disassemble_addr_dl") {
8955:     } else if path.starts_with("/il2cpp/disassemble_addr") {
8961:     } else if path.starts_with("/il2cpp/disassemble") {
8968:     } else if path.starts_with("/il2cpp/search_int_dl") {
8990:     } else if path.starts_with("/il2cpp/search_int") {
8994:     } else if path.starts_with("/il2cpp/search_float_dl") {
9016:     } else if path.starts_with("/il2cpp/search_float") {
9020:     } else if path.starts_with("/il2cpp/read_mem_dl") {
9040:     } else if path.starts_with("/il2cpp/read_mem") {
9045:     } else if path.starts_with("/il2cpp/read_string") {
9083:     } else if path == "/il2cpp/search_methods_page" {
9086:     } else if path.starts_with("/il2cpp/search_methods_dl") {
9110:     } else if path.starts_with("/il2cpp/search_methods") {
9115:     } else if path == "/debug/private_file_inventory" {
9117:     } else if path == "/debug/private_file_dl" {
9120:     } else if path.starts_with("/debug/file_dl") {
9149:     } else if path == "/debug/resource_storage" {
9151:     } else if path == "/debug/resource_meta_schema" {
9153:     } else if path == "/debug/resource_meta_probe" {
9155:     } else if path == "/debug/resource_crypto_symbols" {
9157:     } else if path == "/debug/resource_meta_dl" {
9174:     } else if path == "/debug/resource_file_dl" {
9206:     } else if path == "/mdb" {
9215:             r#"{{"error":"not_found","path":"{}","available":["/scan","/data","/status","/health","/scenario","/debug/upload","/debug/rameninfo","/debug/laststep","/event/recommend","/inherit/compat","/saddle-analysis","/log/turn","/log","/debug/params","/fields","/methods","/singletons","/find_method","/classes","/carddb","/skilldata","/hall","/debug/breeders","/debug/cmdinfo","/debug/training_partners","/debug/ramengains","/debug/paramsincdec","/debug/training_seed","/debug/training_log","/debug/training_log_dl","/update","/update/status","/debug/dumpclass","/debug/storydata","/debug/ramenfields","/debug/all","/mdb","/debug/push_table","/debug/download_table","/classes/search/keyword","/mdb/schema","/mdb/search","/mdb/raw","/mdb/dl_batch","/il2cpp/dump","/il2cpp/call","/il2cpp/tree","/il2cpp/field","/il2cpp/classes","/il2cpp/static","/il2cpp/methods","/il2cpp/search_float","/il2cpp/search_float_dl","/il2cpp/search_int","/il2cpp/search_int_dl","/il2cpp/search_methods","/il2cpp/search_methods_dl","/il2cpp/search_methods_page","/il2cpp/read_mem","/il2cpp/read_mem_dl","/training/result","/api/sniff","/api/sniff/metadata","/api/sniff/status","/api/sniff/toggle","/api/sniff/clear","/api/sniff/diag","/api/event/choices","/api/event/clear"]}}"#,
9246:     } else if path == "/saddles-dl" {
9252:     } else if path == "/il2cpp/disassemble_dl" {
16462: // ★ v3.22.0: 4 new endpoints for training prediction, event recommendation,
23907:             r#"{{"error":"insufficient_code_bytes","available":{},"method_addr":"0x{:x}"}}"#,
24129:             r#"{{"error":"insufficient_code_bytes","available":{},"addr":"0x{:x}"}}"#,
```

## `read_summary` (starts at line 4534)

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

## `read_summary_inner` (starts at line 4592)

```rust
unsafe fn read_summary_inner() -> String {
    // v3.22.51: IN_READ_PATH disabled - /debug/ramenfields proves IL2CPP APIs are safe from HTTP thread
    // Keep the wrapper for potential future use, but don't block any APIs
    read_summary_inner_impl()
}
```

## `read_summary_inner_impl` (starts at line 4624)

```rust
unsafe fn read_summary_inner_impl() -> String {
    if API.is_null() {
        return r#"{"error":"api_null"}"#.to_string();
    }
    let image = match get_image() {
        img if !img.is_null() => img,
        _ => return r#"{"error":"image_null"}"#.to_string(),
    };

    // --- Chara stats ---
    boot_trace("summary_p1");
    ura_log(3, "★ read_summary phase1: chara stats");
    let wdm_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkDataManager").as_ptr(),
    );
    if wdm_class.is_null() {
        return r#"{"error":"no_wdm"}"#.to_string();
    }
    let wdm_inst = get_singleton(wdm_class);
    if wdm_inst.is_null() {
        return r#"{"error":"no_wdm_inst"}"#.to_string();
    }
    log_predict_step("S:wdm");

    log_predict_step("S:before_sm_class");
    let sm_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkSingleModeData").as_ptr(),
    );
    log_predict_step("S:after_sm_class");
    if sm_class.is_null() {
        return r#"{"error":"no_sm_class"}"#.to_string();
    }

    log_predict_step("S:before_get_single_mode");
    let sm_obj = call_getter_ref(wdm_class, wdm_inst, "get_SingleMode"); // [INVOKE-01]
    log_predict_step("S:after_get_single_mode");
    if sm_obj.is_null() {
        return r#"{"error":"no_sm"}"#.to_string();
    }

    log_predict_step("S:before_chara_class");
    let chara_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkSingleModeCharaData").as_ptr(),
    );
    log_predict_step("S:after_chara_class");
    if chara_class.is_null() {
        return r#"{"error":"no_chara_class"}"#.to_string();
    }

    log_predict_step("S:before_get_character");
    let chara_obj = call_getter_ref(sm_class, sm_obj, "get_Character"); // [INVOKE-02] get_Character — 唯一调用
    log_predict_step("S:after_get_character");
    if chara_obj.is_null() {
        return r#"{"error":"no_chara"}"#.to_string();
    }

    log_predict_step("S:before_read_speed");
    let spd = read_obscured_int_at(chara_obj, 248); // _speed
    log_predict_step("S:after_read_speed");
    let sta = read_obscured_int_at(chara_obj, 268); // _stamina
    let pow_ = read_obscured_int_at(chara_obj, 288); // _power
    let gut = read_obscured_int_at(chara_obj, 308); // _guts
    let wiz = read_obscured_int_at(chara_obj, 328); // _wiz
    let vit = read_obscured_int_at(chara_obj, 208); // _hp
    let mvit = read_obscured_int_at(chara_obj, 228); // _maxHp
    let mot = read_obscured_int_at(chara_obj, 1056); // _motivation
    let spt = read_obscured_int_at(chara_obj, 704); // _skillPoint
    let fan = read_obscured_int_at(chara_obj, 996); // _fanCount
                                                    // ★ v3.24.9: Month/Half are computed properties — must use getter (no offset available)
    let mon = if !sm_class.is_null() {
        call_getter_int(sm_class, sm_obj, "get_Month")
    } else {
        1
    }; // [INVOKE-03] get_Month — 唯一调用
    let half = if !sm_class.is_null() {
        call_getter_int(sm_class, sm_obj, "get_Half")
    } else {
        1
    }; // [INVOKE-04] get_Half — 唯一调用
       // v3.24.72: decrypt the field, but do NOT assign cumulative/countdown semantics yet.
       // 444444 is the ObscuredInt XOR key. The decrypted value is exposed only as raw data
       // until it has been compared with the in-game countdown display across boundaries.
    let raw_total_turn_num = read_obscured_int_at(sm_obj as *const c_void, 68); // _totalTurnNum
    let sid = read_obscured_int_at(chara_obj, 568); // _scenarioId
                                                    // Only Ramen is quarantined: its UI is countdown-based and the raw field mapping is unverified.
                                                    // Preserve the pre-existing behavior for other scenarios.
    let (year, cumulative_turn) = if sid == 14 {
        (-1, -1)
    } else if raw_total_turn_num > 0 {
        let y = if raw_total_turn_num <= 18 {
            1
        } else if raw_total_turn_num <= 42 {
            2
        } else if raw_total_turn_num <= 66 {
            3
        } else {
            4
        };
        (y, raw_total_turn_num)
    } else {
        let y = if mon >= 4 { 1 } else { 2 };
        (y, (y - 1) * 24 + (mon - 1) * 2 + half)
    };
    let chara_id = read_obscured_int_at(chara_obj, 36); // _cardId

    // ★ v3.24.9: New fields — attribute caps + scenario progress + running style
    let max_spd = read_obscured_int_at(chara_obj, 348); // MaxSpeed
    let max_sta = read_obscured_int_at(chara_obj, 368); // MaxStamina
    let max_pow = read_obscured_int_at(chara_obj, 388); // MaxPower
    let max_gut = read_obscured_int_at(chara_obj, 408); // MaxGuts
    let max_wiz = read_obscured_int_at(chara_obj, 428); // MaxWiz
    let scenario_progress = read_obscured_int_at(chara_obj, 1116); // ScenarioProgress
    let running_style = read_obscured_int_at(chara_obj, 944); // RunningStyle
    let training_event_type = read_obscured_int_at(chara_obj, 672); // TrainingEventType

    // ★ v3.24.9: Static info (read every time, but rarely changes)
    let talent_level = read_obscured_int_at(chara_obj, 88); // TalentLevel
    let limit_break = read_obscured_int_at(chara_obj, 108); // LimitBreakCount
    let chara_grade = read_obscured_int_at(chara_obj, 168); // CharaGrade
    let difficulty = read_obscured_int_at(chara_obj, 608); // Difficulty

    // ★ v3.24.9: Proper (适性) — A=6,B=5,C=4,D=3,E=2,F=1,G=0
    let proper_dist_short = read_obscured_int_at(chara_obj, 744);
    let proper_dist_mile = read_obscured_int_at(chara_obj, 764);
    let proper_dist_mid = read_obscured_int_at(chara_obj, 784);
    let proper_dist_long = read_obscured_int_at(chara_obj, 804);
    let proper_ground_turf = read_obscured_int_at(chara_obj, 904);
    let proper_ground_dirt = read_obscured_int_at(chara_obj, 924);

    // Runtime reflection confirms offset 0x198 is ObscuredInt _fixedTurnCharaSeed.
    // This is a named game field, not a complete PRNG state.
    let fixed_turn_chara_seed = if !sm_obj.is_null() {
        read_obscured_int_at(sm_obj, 408)
    } else {
        0
    };
    let chara_effect_ids_arr = read_ptr_at(chara_obj, 1080); // _charaEffectIdArray
    let chara_effect_ids: Vec<i32> = if !chara_effect_ids_arr.is_null() {
        read_il2cpp_int_list(chara_effect_ids_arr)
    } else {
        Vec::new()
    };
    let effect_ids_str: Vec<String> = chara_effect_ids.iter().map(|x| x.to_string()).collect();
    log_predict_step(&format!("S:stats sid={}", sid));

    // ★ v3.22.0: Read learned skills and compute skill evaluation
    boot_trace("summary_p1b");
    ura_log(3, "★ read_summary phase1b: skill eval");
    let (skill_eval, skill_count, skills_json) = {
        let learned_skills = read_chara_skills(chara_class, chara_obj, image);
        compute_skill_eval(&learned_skills)
    };
    ura_log(
        2,
        &format!("skill_eval={} count={}", skill_eval, skill_count),
    );
    log_predict_step("S:skills");

    let mot_s = match mot {
        5 => "Best",
        4 => "Good",
        3 => "Normal",
        2 => "Bad",
        1 => "Worst",
        _ => "?",
    };
    let scn_s = match sid {
        1 => "URA",
        2 => "TeamRace",
        3 => "Live",
        4 => "Free",
        5 => "Venus",
        6 => "Arc",
        7 => "Sport",
        8 => "Cook",
        9 => "Mecha",
        10 => "Legend",
        11 => "Pioneer",
        12 => "Onsen",
        13 => "Breeders",
        14 => "Ramen",
        _ => "Unknown",
    };

    // ★ v3.18.2: Pre-read Ramen CommandInfoArray gains (scenario_id == 14)
    // HomeInfoData.ParamsIncDecInfoArray is empty for Ramen scenario.
    // Real gains are in WorkSingleModeScenarioRamenDataSet.CommandInfoArray
    // → ObscuredSingleModeRamenCommandInfo.ParamsIncDecInfoArray
    // Uses same plain Int32 format as Breeders: SingleModeParamsIncDecInfo at 0x10, 0x14
    let mut ramen_gains_map: std::collections::HashMap<i32, String> =
        std::collections::HashMap::new();
    let mut ramen_stat_gains_map: std::collections::HashMap<i32, [i32; 5]> =
        std::collections::HashMap::new();
    let mut ramen_skill_pt_map: std::collections::HashMap<i32, i32> =
        std::collections::HashMap::new();
    let mut ramen_vital_cost_map: std::collections::HashMap<i32, i32> =
        std::collections::HashMap::new();
    let mut ramen_gauge_gains_map: std::collections::HashMap<i32, i32> =
        std::collections::HashMap::new();
    // ★ v3.18.4: Ramen scenario-specific data for /summary
    let mut ramen_checkpoint_pt: i32 = -1;
    let mut ramen_special_feeling_num: i32 = -1;
    let mut ramen_recommend_type: i32 = -1;
    let mut ramen_feeling_info_json = String::new();
    let mut ramen_acquisition_gauges_json = String::new();
    let mut ramen_command_feelings_json = String::new();
    let mut ramen_command_gauge_vectors_json = String::new();
    // ★ v3.22.39: Aggregate sozai counts while reading FeelingInfo
    let mut ramen_sozai_counts: [i32; 3] = [0, 0, 0]; // [麺=1, スープ=2, トッピング=3]
    let mut ramen_selected_region_ids_json = String::new();
    // ★ v3.24.30: MDB-derived candidate pool (labeled; not a runtime-native read)
    let mut ramen_selectable_region_ids_derived_json = String::new();
    // ★ v3.24.32: stale phase pool between selection rounds (not selectable now)
    let mut ramen_region_pool_phase_derived_json = String::new();
    let mut ramen_active_effects_raw_json = String::new();
    let mut ramen_uraf_type: i32 = -1;
    let mut ramen_uraf_state: i32 = -1;
    // ★ v3.22.89: Gauge gains per training command (from DataSet CommandInfoArray, target_type=30)
    let mut ramen_gauge_gains_json = String::new();
    // ★ v3.22.51: Ramen direct memory read — only 2 il2cpp_runtime_invoke calls
    // (try_get_scenario_obj + get_DataSet), then zero il2cpp calls
    if sid == 14 {
        ura_log(3, "v3.22.51 ramen: direct memory read");
        log_predict_step("S:ramen start");
        log_predict_step("S:ramen dataset before scenario");
        let scenario_obj = try_get_scenario_obj(chara_class, chara_obj, 14);
        if !scenario_obj.is_null() {
            let sc_class =
                std::ptr::read_unaligned::<*mut c_void>(scenario_obj as *const *mut c_void);
            log_predict_step("S:ramen sc_obj");
            log_predict_step("S:ramen dataset before getter");
            let dataset_obj = call_getter_ref(sc_class, scenario_obj, "get_DataSet"); // [INVOKE-05] get_DataSet (Ramen) — ★ 与 INVOKE-09 重复，待去重
            log_predict_step("S:ramen dataset after getter");
            if !dataset_obj.is_null() {
                let ds_class =
                    std::ptr::read_unaligned::<*mut c_void>(dataset_obj as *const *mut c_void);
                // Read 5 scalar ObscuredInt fields (zero il2cpp calls)
                let (cp_pt, sf_num, rec_type, uraf_t, uraf_s) =
                    read_ramen_scalar_fields(ds_class, dataset_obj);
                log_predict_step("S:ramen ds");
                ramen_checkpoint_pt = cp_pt;
                ramen_special_feeling_num = sf_num;
                ramen_recommend_type = rec_type;
                ramen_uraf_type = uraf_t;
                ramen_uraf_state = uraf_s;
                ura_log(
                    3,
                    &format!(
                        "ramen scalar: cp={} sf={} rec={} uraf_t={} uraf_s={}",
                        cp_pt, sf_num, rec_type, uraf_t, uraf_s
                    ),
                );
                log_predict_step("S:ramen dataset scalars done");
                // SelectedRegionIdArray (List<ObscuredInt>)
                let sra_off = cached_find_field_offset(ds_class, "SelectedRegionIdArray");
                if sra_off >= 0 {
                    let list_obj = read_ptr_at(dataset_obj, sra_off);
                    if !list_obj.is_null() {
                        let lb = list_obj as *const u8;
                        let llen = std::ptr::read_unaligned::<usize>(
                            lb.add(IL2CPP_LIST_COUNT_OFF) as *const usize
                        );
                        if llen > 0 && llen < 100 {
                            let mut ids: Vec<String> = Vec::new();
                            for i in 0..llen {
                                let elem = lb.add(IL2CPP_LIST_ITEMS_OFF + i * 0x14);
                                let val = read_obscured_int_at(elem as *const c_void, 0);
                                ids.push(val.to_string());
                            }
                            ramen_selected_region_ids_json = ids.join(",");
                        }
                    }
                }
                log_predict_step("S:ramen regions done");
                // ★ v3.24.30: AllSelectedRegionIdArray + MDB-derived candidate pool.
                // Labeled as derivation; the runtime-native selectable list stays unknown.
                let asra_off = cached_find_field_offset(ds_class, "AllSelectedRegionIdArray");
                if asra_off >= 0 {
                    let list_obj = read_ptr_at(dataset_obj, asra_off);
                    let _all_selected = inline_obscured_int_list_vec(list_obj);
                    // Disabled in v3.24.72: MDB round numbers cannot be compared with
                    // raw_total_turn_num until the latter's countdown/cumulative mapping is verified.
                }
                // ActiveEffectArray (List<ActiveEffectInfo>)
                let ae_off = cached_find_field_offset(ds_class, "ActiveEffectArray");
                if ae_off >= 0 {
                    let list_obj = read_ptr_at(dataset_obj, ae_off);
                    if !list_obj.is_null() {
                        let lb = list_obj as *const u8;
                        let llen = std::ptr::read_unaligned::<usize>(
                            lb.add(IL2CPP_LIST_COUNT_OFF) as *const usize
                        );
                        if llen > 0 && llen < 100 {
                            let first_elem = std::ptr::read_unaligned::<*mut c_void>(
                                lb.add(IL2CPP_LIST_ITEMS_OFF) as *const *mut c_void,
                            );
                            if !first_elem.is_null() {
                                let elem_class = std::ptr::read_unaligned::<*mut c_void>(
                                    first_elem as *const *mut c_void,
                                );
                                let cat_off =
                                    cached_find_field_offset(elem_class, "EffectCategory");
                                let eid_off = cached_find_field_offset(elem_class, "EffectId");
                                let val_off = cached_find_field_offset(elem_class, "EffectValue");
                                let mut effects: Vec<String> = Vec::new();
                                for i in 0..llen {
                                    let ep = std::ptr::read_unaligned::<*mut c_void>(
                                        lb.add(IL2CPP_LIST_ITEMS_OFF + i * IL2CPP_LIST_ITEM_SIZE)
                                            as *const *mut c_void,
                                    );
                                    if ep.is_null() {
                                        continue;
                                    }
                                    let cat = if cat_off >= 0 {
                                        read_obscured_int_at(ep, cat_off)
                                    } else {
                                        -1
                                    };
                                    let eid = if eid_off >= 0 {
                                        read_obscured_int_at(ep, eid_off)
                                    } else {
                                        -1
                                    };
                                    let val = if val_off >= 0 {
                                        read_obscured_int_at(ep, val_off)
                                    } else {
                                        -1
                                    };
                                    effects.push(format!(
                                        r#"{{"category":{},"id":{},"value":{}}}"#,
                                        cat, eid, val
                                    ));
                                }
                                ramen_active_effects_raw_json = effects.join(",");
                            }
                        }
                    }
                }
                log_predict_step("S:ramen effects done");
                // ★ v3.22.39: CommandFeelingInfoArray — dump element class name + gauge data
                // Skip in /summary for now, use /debug/gauge for safe testing
                // TODO: re-enable after /debug/gauge confirms element type and GetGainCount works
                // FeelingInfoArray (List<FeelingInfo>)
                let fi_off = cached_find_field_offset(ds_class, "FeelingInfoArray");
                if fi_off >= 0 {
                    let list_obj = read_ptr_at(dataset_obj, fi_off);
                    if !list_obj.is_null() {
                        let lb = list_obj as *const u8;
                        let llen = std::ptr::read_unaligned::<usize>(
                            lb.add(IL2CPP_LIST_COUNT_OFF) as *const usize
                        );
                        if llen > 0 && llen < 100 {
                            let first_elem = std::ptr::read_unaligned::<*mut c_void>(
                                lb.add(IL2CPP_LIST_ITEMS_OFF) as *const *mut c_void,
                            );
                            if !first_elem.is_null() {
                                let elem_class = std::ptr::read_unaligned::<*mut c_void>(
                                    first_elem as *const *mut c_void,
                                );
                                let ft_off = cached_find_field_offset(elem_class, "FeelingIndex");
                                let fv_off = cached_find_field_offset(elem_class, "FeelingId");
                                let mut feelings: Vec<String> = Vec::new();
                                for i in 0..llen {
                                    let ep = std::ptr::read_unaligned::<*mut c_void>(
                                        lb.add(IL2CPP_LIST_ITEMS_OFF + i * IL2CPP_LIST_ITEM_SIZE)
                                            as *const *mut c_void,
                                    );
                                    if ep.is_null() {
                                        continue;
                                    }
                                    let ft = if ft_off >= 0 {
                                        read_obscured_int_at(ep, ft_off)
                                    } else {
                                        -1
                                    };
                                    let fv = if fv_off >= 0 {
                                        read_obscured_int_at(ep, fv_off)
                                    } else {
                                        -1
                                    };
                                    // ★ v3.22.39: Count sozai by FeelingId (1=麺, 2=スープ, 3=トッピング)
                                    if fv >= 1 && fv <= 3 {
                                        ramen_sozai_counts[(fv - 1) as usize] += 1;
                                    }
                                    feelings.push(format!(
                                        r#"{{"FeelingIndex":{},"FeelingId":{}}}"#,
                                        ft, fv
                                    ));
                                }
                                ramen_feeling_info_json = feelings.join(",");
                            }
                        }
                    }
                }
                // Resource acquisition state. Read the two proven bounded lists
                // directly from DataSet memory; do not call unsafe GetGainCount.
                let aft_off = cached_find_field_offset(ds_class, "FeelingTurnInfoArray");
                if aft_off >= 0 {
                    let list_obj = read_ptr_at(dataset_obj, aft_off);
                    if !list_obj.is_null() {
                        let lb = list_obj as *const u8;
                        let count = std::ptr::read_unaligned::<usize>(
                            lb.add(IL2CPP_LIST_COUNT_OFF) as *const usize,
                        );
                        if count > 0 && count <= 3 {
                            let mut parts = Vec::new();
                            for i in 0..count {
                                let item = std::ptr::read_unaligned::<*mut c_void>(
                                    lb.add(IL2CPP_LIST_ITEMS_OFF + i * IL2CPP_LIST_ITEM_SIZE)
                                        as *const *mut c_void,
                                );
                                if item.is_null() {
                                    continue;
                                }
                                let class = std::ptr::read_unaligned::<*mut c_void>(
                                    item as *const *mut c_void,
                                );
                                let remain_off = cached_find_field_offset(class, "RemainTurn");
                                let feeling_off = cached_find_field_offset(class, "FeelingId");
                                let remain = if remain_off >= 0 {
                                    read_obscured_int_at(item, remain_off)
                                } else {
                                    -1
                                };
                                let feeling_id = if feeling_off >= 0 {
                                    read_obscured_int_at(item, feeling_off)
                                } else {
                                    -1
                                };
                                parts.push(format!(
                                    r#"{{"feeling_id":{},"remaining":{}}}"#,
                                    feeling_id, remain
                                ));
                            }
                            ramen_acquisition_gauges_json = parts.join(",");
                        }
                    }
                }

                let cf_off = cached_find_field_offset(ds_class, "CommandFeelingInfoArray");
                if cf_off >= 0 {
                    let list_obj = read_ptr_at(dataset_obj, cf_off);
                    if !list_obj.is_null() {
                        let lb = list_obj as *const u8;
                        let count = std::ptr::read_unaligned::<usize>(
                            lb.add(IL2CPP_LIST_COUNT_OFF) as *const usize,
                        );
                        if count > 0 && count <= 20 {
                            let mut parts = Vec::new();
                            for i in 0..count {
                                let item = std::ptr::read_unaligned::<*mut c_void>(
                                    lb.add(IL2CPP_LIST_ITEMS_OFF + i * IL2CPP_LIST_ITEM_SIZE)
                                        as *const *mut c_void,
                                );
                                if item.is_null() {
                                    continue;
                                }
                                let class = std::ptr::read_unaligned::<*mut c_void>(
                                    item as *const *mut c_void,
                                );
                                let type_off = cached_find_field_offset(class, "CommandType");
                                let id_off = cached_find_field_offset(class, "CommandId");
                                let feeling_off = cached_find_field_offset(class, "FeelingId");
                                let command_type = if type_off >= 0 {
                                    read_obscured_int_at(item, type_off)
                                } else {
                                    -1
                                };
                                let command_id = if id_off >= 0 {
                                    read_obscured_int_at(item, id_off)
                                } else {
                                    -1
                                };
                                let feeling_id = if feeling_off >= 0 {
                                    read_obscured_int_at(item, feeling_off)
                                } else {
                                    -1
                                };
                                parts.push(format!(
                                    r#"{{"command_type":{},"command_id":{},"feeling_id":{}}}"#,
                                    command_type, command_id, feeling_id
                                ));
                            }
                            ramen_command_feelings_json = parts.join(",");
                        }
                    }
                }

                // Final per-command acquisition vectors for the resource planner.
                // This reads the proven bounded FeelingReduceTurnInfoArray and its
                // nested FeelingTurnArray only; it does not reconstruct any formula.
                let fr_off = cached_find_field_offset(ds_class, "FeelingReduceTurnInfoArray");
                if fr_off >= 0 {
                    let list_obj = read_ptr_at(dataset_obj, fr_off);
                    if !list_obj.is_null() {
                        let lb = list_obj as *const u8;
                        let count = std::ptr::read_unaligned::<usize>(
                            lb.add(IL2CPP_LIST_COUNT_OFF) as *const usize,
                        );
                        if count > 0 && count <= 20 {
                            let mut vectors = Vec::new();
                            for i in 0..count {
                                let item = std::ptr::read_unaligned::<*mut c_void>(
                                    lb.add(IL2CPP_LIST_ITEMS_OFF + i * IL2CPP_LIST_ITEM_SIZE)
                                        as *const *mut c_void,
                                );
                                if item.is_null() {
                                    continue;
                                }
                                let class = std::ptr::read_unaligned::<*mut c_void>(
                                    item as *const *mut c_void,
                                );
                                let type_off = cached_find_field_offset(class, "CommandType");
                                let id_off = cached_find_field_offset(class, "CommandId");
                                let turns_off = cached_find_field_offset(class, "FeelingTurnArray");
                                let command_type = if type_off >= 0 {
                                    read_obscured_int_at(item, type_off)
                                } else {
                                    -1
                                };
                                let command_id = if id_off >= 0 {
                                    read_obscured_int_at(item, id_off)
                                } else {
                                    -1
                                };
                                if turns_off < 0 {
                                    continue;
                                }
                                let turns = read_ptr_at(item, turns_off);
                                if turns.is_null() {
                                    continue;
                                }
                                let tb = turns as *const u8;
                                let turn_count = std::ptr::read_unaligned::<usize>(
                                    tb.add(IL2CPP_LIST_COUNT_OFF) as *const usize,
                                );
                                if turn_count == 0 || turn_count > 3 {
                                    continue;
                                }
                                let mut progress = Vec::new();
                                for j in 0..turn_count {
                                    let turn_item = std::ptr::read_unaligned::<*mut c_void>(
                                        tb.add(IL2CPP_LIST_ITEMS_OFF + j * IL2CPP_LIST_ITEM_SIZE)
                                            as *const *mut c_void,
                                    );
                                    if turn_item.is_null() {
                                        continue;
                                    }
                                    let turn_class = std::ptr::read_unaligned::<*mut c_void>(
                                        turn_item as *const *mut c_void,
                                    );
                                    let feeling_off =
                                        cached_find_field_offset(turn_class, "FeelingId");
                                    let remain_off =
                                        cached_find_field_offset(turn_class, "RemainTurn");
                                    let feeling_id = if feeling_off >= 0 {
                                        read_obscured_int_at(turn_item, feeling_off)
                                    } else {
                                        -1
                                    };
                                    let remaining = if remain_off >= 0 {
                                        read_obscured_int_at(turn_item, remain_off)
                                    } else {
                                        -1
                                    };
                                    progress.push(format!(
                                        r#"{{"feeling_id":{},"remaining":{}}}"#,
                                        feeling_id, remaining
                                    ));
                                }
                                if !progress.is_empty() {
                                    vectors.push(format!(
                                        r#"{{"command_type":{},"command_id":{},"progress":[{}]}}"#,
                                        command_type,
                                        command_id,
                                        progress.join(",")
                                    ));
                                }
                            }
                            ramen_command_gauge_vectors_json = vectors.join(",");
                        }
                    }
                }

                // ★ v3.22.52: Read CommandInfoArray from DataSet for Ramen gains
                // HomeInfoData.ParamsIncDecInfoArray is empty for Ramen,
                // real gains are in DataSet.CommandInfoArray[].ParamsIncDecInfoArray
                // Same direct memory read as /debug/paramsincdec
                // ★ v3.24.9: Reverted to read_ptr_at — call_getter_ref caused crash during loading
                // The offset 16 is confirmed correct by /debug/dumpclass
                // Original code worked in v3.24.2, crash was introduced by getter call
                log_predict_step("S:ramen feelings done");
                let cmd_list = read_ptr_at(dataset_obj, RAMEN_DATASET_CMD_ARRAY_OFF as i32);
                if !cmd_list.is_null() {
                    let cmd_lb = cmd_list as *const u8;
                    let cmd_count = std::ptr::read_unaligned::<usize>(
                        cmd_lb.add(IL2CPP_LIST_COUNT_OFF) as *const usize,
                    );
                    if cmd_count > 0 && cmd_count < 100 {
                        // ObscuredSingleModeRamenCommandInfo field offsets (confirmed by /debug/dumpclass):
                        //   0x10 (16): CommandType (ObscuredInt, 20 bytes inline)
                        //   0x24 (36): CommandId (ObscuredInt, 20 bytes inline)
                        //   0x38 (56): ParamsIncDecInfoArray (List ptr)
                        // read_obscured_int_at reads key^hidden at the given offset
                        for ci in 0..cmd_count {
                            let ce = std::ptr::read_unaligned::<*mut c_void>(
                                cmd_lb.add(IL2CPP_LIST_ITEMS_OFF + ci * IL2CPP_LIST_ITEM_SIZE)
                                    as *const *mut c_void,
                            );
                            if ce.is_null() {
                                continue;
                            }
                            let cmd_id = read_obscured_int_at(ce, RAMEN_CMD_COMMAND_ID_OFF as i32);
                            let ce_params = read_ptr_at(ce, RAMEN_CMD_PARAMS_ARRAY_OFF as i32);
                            if cmd_id < 0 || ce_params.is_null() {
                                continue;
                            }
                            let ce_plb = ce_params as *const u8;
                            let ce_plen = std::ptr::read_unaligned::<usize>(
                                ce_plb.add(IL2CPP_LIST_COUNT_OFF) as *const usize,
                            );
                            if ce_plen == 0 || ce_plen > 1000 {
                                continue;
                            }
                            let mut gains_parts: Vec<String> = Vec::new();
                            let mut sg = [0i32; 5]; // [Speed, Stamina, Power, Guts, Wisdom]
                            let mut spt = 0i32;
                            let mut vc = 0i32;
                            // ★ v3.22.89: Merged gauge_gain into single loop (was separate redundant loop)
                            let mut gauge_gain = 0i32;
                            // ★ Confirmed by /debug/params: elements are SingleModeParamsIncDecInfo (plain int32)
                            // NOT SingleModeParamsIncDecInfoData (ObscuredInt)
                            // Plain int32 read at 0x10/0x14 gives correct values
                            for pi in 0..ce_plen {
                                let pe = std::ptr::read_unaligned::<*mut c_void>(
                                    ce_plb.add(IL2CPP_LIST_ITEMS_OFF + pi * IL2CPP_LIST_ITEM_SIZE)
                                        as *const *mut c_void,
                                );
                                if pe.is_null() {
                                    continue;
                                }
                                let tt = std::ptr::read_unaligned::<i32>(
                                    (pe as *const u8).add(PARAMS_INCDEC_TARGET_TYPE_OFF)
                                        as *const i32,
                                );
                                let vv = std::ptr::read_unaligned::<i32>(
                                    (pe as *const u8).add(PARAMS_INCDEC_VALUE_OFF) as *const i32,
                                );
                                if tt == 30 {
                                    gauge_gain += vv;
                                }
                                if vv == 0 {
                                    continue;
                                }
                                let tn = match tt {
                                    1 => "Speed",
                                    2 => "Stamina",
                                    3 => "Power",
                                    4 => "Guts",
                                    5 => "Wiz",
                                    10 => "HP",
                                    20 => "Motivation",
                                    30 => "Gauge",
                                    40 => "SkillPt",
                                    _ => "Unknown",
                                };
                                gains_parts.push(format!(r#""{}":{}"#, tn, vv));
                                match tt {
                                    1 => sg[0] += vv,
                                    2 => sg[1] += vv,
                                    4 => sg[2] += vv,
                                    3 => sg[3] += vv,
                                    5 => sg[4] += vv,
                                    10 => vc += vv,
                                    40 => spt += vv,
                                    _ => {}
                                }
                            }
                            if !gains_parts.is_empty() {
                                // ★ FIX: Store under both cmd_id variants (601→101, 602→102, etc.)
                                // so lookup works regardless of which command_id space HomeInfoData uses
                                ramen_gains_map.insert(cmd_id, gains_parts.join(","));
                                ramen_stat_gains_map.insert(cmd_id, sg);
                                ramen_skill_pt_map.insert(cmd_id, spt);
                                ramen_vital_cost_map.insert(cmd_id, vc);
                                let alt_id = match cmd_id {
                                    601 => Some(101),
                                    602 => Some(105),
                                    603 => Some(102),
                                    604 => Some(103),
                                    605 => Some(106),
                                    101 => Some(601),
                                    102 => Some(603),
                                    103 => Some(604),
                                    105 => Some(602),
                                    106 => Some(605),
                                    _ => None,
                                };
                                if let Some(aid) = alt_id {
                                    ramen_gains_map
                                        .insert(aid, ramen_gains_map.get(&cmd_id).unwrap().clone());
                                    ramen_stat_gains_map.insert(aid, sg);
                                    ramen_skill_pt_map.insert(aid, spt);
                                    ramen_vital_cost_map.insert(aid, vc);
                                }
                                ura_log(
                                    4,
                                    &format!(
                                        "ramen gains: cmd_id={} gains={} alt={:?}",
                                        cmd_id,
                                        gains_parts.join(","),
                                        alt_id
                                    ),
                                );
                            }
                            if gauge_gain > 0 {
                                ramen_gauge_gains_map.insert(cmd_id, gauge_gain);
                            }
                        }
                    }
                }
                // ★ v3.22.89: Build gauge_gains JSON from ramen_gauge_gains_map
                log_predict_step("S:ramen commands done");
                if !ramen_gauge_gains_map.is_empty() {
                    let mut gg_parts: Vec<String> = Vec::new();
                    for (&cmd_id, &gauge_val) in &ramen_gauge_gains_map {
                        let cname = match cmd_id {
                            101 | 601 => "Speed",
                            102 | 603 => "Power",
                            103 | 604 => "Guts",
                            105 | 602 => "Stamina",
                            106 | 605 => "Wiz",
                            _ => "Unknown",
                        };
                        gg_parts.push(format!(
                            r#"{{"command_id":{},"name":"{}","gauge":{}}}"#,
                            cmd_id, cname, gauge_val
                        ));
                    }
                    ramen_gauge_gains_json = gg_parts.join(",");
                }
                ura_log(
                    3,
                    &format!(
                    "ramen arrays: regions={} effects={} feelings={} gains_map={} gauge_gains={}",
                    !ramen_selected_region_ids_json.is_empty(),
                    !ramen_active_effects_raw_json.is_empty(),
                    !ramen_feeling_info_json.is_empty(),
                    !ramen_gains_map.is_empty(),
                    !ramen_gauge_gains_map.is_empty()
                ),
                );
                log_predict_step("S:ramen arrays");
            } else {
                ura_log(2, "ramen: dataset_obj null");
            }
        } else {
            ura_log(2, "ramen: scenario_obj null");
        }
    }

    // Partner ID -> current bond.
    //
    // _evaluationList is List<Evaluation>, not Evaluation[]:
    //
    // List<Evaluation> + 0x10 = _items (Evaluation[])
    // List<Evaluation> + 0x18 = _size  (Int32)
    // Evaluation[]     + 0x20 = first element
    //
    // Each Evaluation object contains two inline ObscuredInt fields:
    //
    // Evaluation + 0x10 = partner ID
    // Evaluation + 0x24 = current bond
    let mut partner_evaluation: std::collections::HashMap<i32, i32> =
        std::collections::HashMap::new();

    let evaluation_list = read_ptr_at(chara_obj as *const c_void, EVALUATION_LIST_OFF);

    if !evaluation_list.is_null() {
        // List<T>._size is Int32 at +0x18. Do not read this as usize,
        // because +0x1c contains List<T>._version.
        let count = read_int_at(evaluation_list as *const c_void, 0x18);

        // List<T>._items is the backing T[] array at +0x10.
        let evaluation_items = read_ptr_at(evaluation_list as *const c_void, 0x10);

        if count > 0 && count < 1000 && !evaluation_items.is_null() {
            let items_base = evaluation_items as *const u8;

            for i in 0..count as usize {
                // Evaluation is a reference type, so the backing array
                // contains object pointers beginning at array + 0x20.
                let item = std::ptr::read_unaligned::<*mut c_void>(
                    items_base.add(IL2CPP_LIST_ITEMS_OFF + i * IL2CPP_LIST_ITEM_SIZE)
                        as *const *mut c_void,
                );

                if item.is_null() {
                    continue;
                }

                let partner_id =
                    read_obscured_int_at(item as *const c_void, EVALUATION_PARTNER_ID_OFF);

                let current_bond =
                    read_obscured_int_at(item as *const c_void, EVALUATION_VALUE_OFF);

                // Guard against corrupt or misread entries.
                if partner_id > 0
                    && partner_id < 100_000
                    && current_bond >= 0
                    && current_bond <= 100
                {
                    partner_evaluation.insert(partner_id, current_bond);
                }
            }

            ura_log(
                3,
                &format!(
                    "evaluation_list: size={}, decoded={} entries",
                    count,
                    partner_evaluation.len()
                ),
            );
        } else {
            ura_log(
                2,
                &format!(
                    "evaluation_list unavailable: size={}, items_null={}",
                    count,
                    evaluation_items.is_null()
                ),
            );
        }
    } else {
        ura_log(2, "evaluation_list is null");
    }

    // Runtime support-card position -> support_card_data.command_id.
    //
    // Position is read dynamically from the equipped-card object.
    // It is not assumed that a particular card is always in a fixed slot.
    let mut support_command_by_position: std::collections::HashMap<i32, i32> =
        std::collections::HashMap::new();
    /// ★ v3.24.14: position → bond_threshold from MasterDB unique_effect
    let mut bond_threshold_by_position: std::collections::HashMap<i32, i32> =
        std::collections::HashMap::new();
    /// ★ v3.24.15: position → support_card_type (1=普通, 2=友人, 3=团体)
    let mut support_card_type_by_position: std::collections::HashMap<i32, i32> =
        std::collections::HashMap::new();

    // First collect equipped (position, support_card_id) pairs.
    let mut equipped_support_cards: Vec<(i32, i32)> = Vec::new();

    log_predict_step("S:support equip before getter");
    let support_array_for_shining =
        call_getter_on_instance(chara_class, chara_obj, "get_EquipSupportCardArray"); // [INVOKE-06] get_EquipSupportCardArray — ★ 结果复用到 support cards 段
    log_predict_step("S:support equip after getter");

    if !support_array_for_shining.is_null() {
        let support_array_base = support_array_for_shining as *const u8;

        let support_count = std::ptr::read_unaligned::<usize>(
            support_array_base.add(IL2CPP_LIST_COUNT_OFF) as *const usize,
        );

        if support_count > 0 && support_count <= 6 {
            for i in 0..support_count {
                let support = std::ptr::read_unaligned::<*mut c_void>(
                    support_array_base.add(IL2CPP_LIST_ITEMS_OFF + i * IL2CPP_LIST_ITEM_SIZE)
                        as *const *mut c_void,
                );

                if support.is_null() {
                    continue;
                }

                // SingleModeEquipSupportCard:
                //   +0x10 Position      (inline ObscuredInt)
                //   +0x24 SupportCardId (inline ObscuredInt)
                let position = read_obscured_int_at(support as *const c_void, 0x10);

                let support_card_id = read_obscured_int_at(support as *const c_void, 0x24);

                if (1..=6).contains(&position) && support_card_id > 0 {
                    equipped_support_cards.push((position, support_card_id));
                }
            }
        }
    }

    // Resolve each ordinary card's training specialty from MasterDB:
    //
    // command_id=0 is intentionally retained as an unclassified special card.
    //
    // ★ v3.24.14: Also read bond_threshold from support_card_unique_effect.
    //   type_0=101 → value_0 = bond threshold for unique effect / friendship training.
    //   Cards without unique_effect_id get threshold = i32::MAX (never shines).
    log_predict_step("S:support mdb before");
    /// position → (command_id, bond_threshold, support_card_type)
    static SUPPORT_CARD_INFO_CACHE: std::sync::Mutex<
        Option<std::collections::HashMap<i32, (i32, i32, i32)>>,
    > = std::sync::Mutex::new(None);

    // Try cache first; rebuild if empty.
    let mut info_map: std::collections::HashMap<i32, (i32, i32, i32)> = SUPPORT_CARD_INFO_CACHE
        .lock()
        .unwrap()
        .clone()
        .unwrap_or_default();

    if info_map.is_empty() {
        if let Some(mdb_path) = find_mdb_path() {
            if let Ok(connection) =
                Connection::open_with_flags(&mdb_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
            {
                // Query command_id, unique effect threshold, and support_card_type in one JOIN.
                // type_0=101 is the bond-threshold marker in support_card_unique_effect.
                if let Ok(mut statement) = connection.prepare(
                    "SELECT sc.id, sc.command_id, sc.support_card_type, \
                     COALESCE(ue.value_0, 999999) AS threshold \
                     FROM support_card_data sc \
                     LEFT JOIN support_card_unique_effect ue \
                       ON sc.unique_effect_id = ue.id AND ue.type_0 = 101 \
                     WHERE sc.id = ?1",
                ) {
                    for &(position, support_card_id) in &equipped_support_cards {
                        let result = statement.query_row([support_card_id], |row| {
                            Ok((
                                row.get::<_, i32>(0)?, // id
                                row.get::<_, i32>(1)?, // command_id
                                row.get::<_, i32>(2)?, // support_card_type
                                row.get::<_, i32>(3)?, // threshold
                            ))
                        });

                        if let Ok((_id, support_command_id, sc_type, threshold)) = result {
                            support_command_by_position.insert(position, support_command_id);
                            bond_threshold_by_position.insert(position, threshold);
                            support_card_type_by_position.insert(position, sc_type);
                            info_map
                                .insert(support_card_id, (support_command_id, threshold, sc_type));
                        }
                    }
                }

                // Cache for next call.
                if !info_map.is_empty() {
                    *SUPPORT_CARD_INFO_CACHE.lock().unwrap() = Some(info_map.clone());
                }
            }
        }
    } else {
        // Cache hit — populate from cache without DB access.
        for &(position, support_card_id) in &equipped_support_cards {
            if let Some(&(cmd_id, threshold, sc_type)) = info_map.get(&support_card_id) {
                support_command_by_position.insert(position, cmd_id);
                bond_threshold_by_position.insert(position, threshold);
                support_card_type_by_position.insert(position, sc_type);
            }
        }
    }

    log_predict_step("S:support mdb done");
    // --- Training data via HomeInfoData (ALL scenarios) ---
    log_predict_step("S:ramen end");
    boot_trace("summary_p2");
    ura_log(3, "★ read_summary phase2: training data");
    log_predict_step("S:p2 training");
    let mut tr_json = "[]".to_string();
    // ★ v3.15.1: collect eval_trainings in same pass (eliminate dangerous double-read)
    let mut eval_trainings: Vec<(i32, [i32; 5], i32, i32, i32, i32, i32, i32)> = Vec::new();
    log_predict_step("S:homeinfo before getter");
    let home_info_obj = call_getter_on_instance(sm_class, sm_obj, "get_HomeInfoData"); // [INVOKE-07] get_HomeInfoData — 唯一调用
    log_predict_step("S:homeinfo after getter");
    if !home_info_obj.is_null() {
        let hi_class = find_class(
            image,
            to_cstr("Gallop").as_ptr(),
            to_cstr("WorkSingleModeHomeInfoData").as_ptr(),
        );
        if !hi_class.is_null() {
            // CommandInfoArray is a public field (not a getter), at offset 0x10
            log_predict_step("S:homeinfo commands before");
            let cmd_arr = read_field_value(hi_class, home_info_obj, "CommandInfoArray");
            log_predict_step("S:homeinfo commands after");
            if !cmd_arr.is_null() {
                let cmd_base = cmd_arr as *const u8;
                let cmd_len = std::ptr::read_unaligned::<usize>(
                    cmd_base.add(IL2CPP_LIST_COUNT_OFF) as *const usize,
                );
                if cmd_len > 0 && cmd_len < 100 {
                    let mut trs = Vec::new();
                    for i in 0..cmd_len {
                        let ep = std::ptr::read_unaligned::<*mut c_void>(
                            cmd_base.add(IL2CPP_LIST_ITEMS_OFF + i * IL2CPP_LIST_ITEM_SIZE)
                                as *const *mut c_void,
                        );
                        if ep.is_null() {
                            continue;
                        }

                        // ★ v3.24.9: Direct memory read — zero il2cpp_runtime_invoke
                        // SingleModeCommandInfoData offsets (confirmed by /debug/dumpclass):
                        //   CommandType=16  CommandId=36  IsEnable=56
                        //   TrainingPartnerArray=80  TipsEventPartnerArray=88  FailureRate=104
                        let cid = read_obscured_int_at(ep as *const c_void, 36); // CommandId
                        let cname = match cid {
                            CMD_SPEED => "Speed",
                            CMD_STAMINA => "Stamina",
                            CMD_GUTS => "Guts",
                            CMD_POWER => "Power",
                            CMD_WISDOM => "Wiz",
                            CMD_URA_SPEED => "Speed",
                            CMD_URA_STAMINA => "Stamina",
                            CMD_URA_GUTS => "Guts",
                            CMD_URA_POWER => "Power",
                            CMD_URA_WISDOM => "Wiz",
                            CMD_KAKUSHIMI => "Kakushimi",
                            301 => "Outing",
                            390 => "Rest",
                            401 => "Outing2",
                            701 => "Outing3",
                            801 => "Outing4",
                            _ => "Unknown",
                        };
                        let is_enable = read_obscured_int_at(ep as *const c_void, 56); // IsEnable
                        let failure_rate = read_obscured_int_at(ep as *const c_void, 104); // FailureRate

                        // TrainingPartnerArray is ObscuredInt[] with
                        // inline 20-byte values.
                        let tp_arr = read_ptr_at(ep as *const c_void, TRAINING_PARTNER_ARRAY_OFF);

                        // ★ v3.24.15: Read TipsEventPartnerArray for group card shining.
                        // Group cards (support_card_type=3) shine when they trigger a
                        // special tips event, not based on bond threshold.
                        // TipsEventPartnerArray is at offset 0x58 (88), same ObscuredInt[] format.
                        let tips_arr =
                            read_ptr_at(ep as *const c_void, TIPS_EVENT_PARTNER_ARRAY_OFF);
                        let mut tips_partner_ids: std::collections::HashSet<i32> =
                            std::collections::HashSet::new();
                        if !tips_arr.is_null() {
                            let tips_base = tips_arr as *const u8;
                            let tips_len = std::ptr::read_unaligned::<usize>(
                                tips_base.add(IL2CPP_LIST_COUNT_OFF) as *const usize,
                            );
                            if tips_len <= 100 {
                                for ti in 0..tips_len {
                                    let tval = tips_base
                                        .add(IL2CPP_LIST_ITEMS_OFF + ti * OBSCURED_INT_SIZE);
                                    let tips_id = read_obscured_int_at(tval as *const c_void, 0);
                                    if tips_id > 0 {
                                        tips_partner_ids.insert(tips_id);
                                    }
                                }
                            }
                        }

                        let mut partner_ids: Vec<i32> = Vec::new();
                        let mut partners_json: Vec<String> = Vec::new();

                        // Number of confirmed shining support cards.
                        let mut shining_count = 0i32;

                        // This remains true when every present support
                        // partner can be classified conclusively.
                        //
                        // NPC/scenario partners do not affect completeness.
                        let mut shining_complete = true;

                        if !tp_arr.is_null() {
                            let array_base = tp_arr as *const u8;

                            let partner_count = std::ptr::read_unaligned::<usize>(
                                array_base.add(IL2CPP_LIST_COUNT_OFF) as *const usize,
                            );

                            if partner_count <= 100 {
                                for pi in 0..partner_count {
                                    let value = array_base
                                        .add(IL2CPP_LIST_ITEMS_OFF + pi * OBSCURED_INT_SIZE);

                                    let partner_id =
                                        read_obscured_int_at(value as *const c_void, 0);

                                    if partner_id <= 0 {
                                        continue;
                                    }

                                    partner_ids.push(partner_id);

                                    let current_bond = partner_evaluation.get(&partner_id).copied();

                                    // Classify support cards only through the actual equipped
                                    // position map. A numeric partner_id range alone is not proof.
                                    let support_card_id = equipped_support_cards
                                        .iter()
                                        .find(|&&(position, _)| position == partner_id)
                                        .map(|&(_, card_id)| card_id);
                                    let is_support_card = support_card_id.is_some();

                                    let support_position = if is_support_card {
                                        partner_id.to_string()
                                    } else {
                                        "null".to_string()
                                    };
                                    let support_card_id_json = support_card_id
                                        .map(|value| value.to_string())
                                        .unwrap_or_else(|| "null".to_string());

                                    let bond_json = current_bond
                                        .map(|value| value.to_string())
                                        .unwrap_or_else(|| "null".to_string());

                                    // ★ v3.24.15: Card-type-aware shining logic.
                                    //
                                    //   support_card_type=1 (普通卡): bond >= threshold && training match
                                    //   support_card_type=2 (友人卡): always false (友人卡不彩圈)
                                    //   support_card_type=3 (团体卡): partner_id in TipsEventPartnerArray
                                    //     (触发特殊启示事件就彩圈，不管 bond)
                                    //   Unknown type: null (conservative)
                                    let sc_type =
                                        support_card_type_by_position.get(&partner_id).copied();

                                    let bond_threshold = bond_threshold_by_position
                                        .get(&partner_id)
                                        .copied()
                                        .unwrap_or(999999);

                                    let is_shining: Option<bool> = if is_support_card {
                                        match sc_type {
                                            // 普通卡: bond >= threshold && training match
                                            Some(1) => {
                                                match (
                                                    current_bond,
                                                    support_command_by_position
                                                        .get(&partner_id)
                                                        .copied(),
                                                    normalize_training_command_id(cid),
                                                ) {
                                                    (
                                                        Some(bond),
                                                        Some(support_command_id),
                                                        Some(current_training),
                                                    ) => {
                                                        match support_card_command_id_to_training_id(
                                                            support_command_id,
                                                        ) {
                                                            Some(card_training) => Some(
                                                                bond >= bond_threshold
                                                                    && card_training
                                                                        == current_training,
                                                            ),
                                                            None => None,
                                                        }
                                                    }
                                                    _ => None,
                                                }
                                            }
                                            // 友人卡: 永远不彩圈
                                            Some(2) => Some(false),
                                            // 团体卡: 启示事件触发就彩圈
                                            Some(3) => Some(tips_partner_ids.contains(&partner_id)),
                                            // 未知类型: 保守 null
                                            _ => {
                                                // Fallback to old logic for untyped cards
                                                match (
                                                    current_bond,
                                                    support_command_by_position
                                                        .get(&partner_id)
                                                        .copied(),
                                                    normalize_training_command_id(cid),
                                                ) {
                                                    (
                                                        Some(bond),
                                                        Some(support_command_id),
                                                        Some(current_training),
                                                    ) => {
                                                        match support_card_command_id_to_training_id(
                                                            support_command_id,
                                                        ) {
                                                            Some(card_training) => Some(
                                                                bond >= bond_threshold
                                                                    && card_training
                                                                        == current_training,
                                                            ),
                                                            None => None,
                                                        }
                                                    }
                                                    _ => None,
                                                }
                                            }
                                        }
                                    } else {
                                        // NPC and scenario partners are not equipped support cards.
                                        None
                                    };

                                    // ★ v3.24.14: Unique effect active = bond >= threshold.
                                    //   Triggers on ANY training, not just得意训练.
                                    //   友人卡: threshold=60, 团体卡: threshold=80/100
                                    let is_unique_active: Option<bool> = if is_support_card {
                                        if sc_type == Some(2) {
                                            // 友人卡固有: bond >= 60
                                            current_bond.map(|bond| bond >= bond_threshold)
                                        } else if sc_type == Some(3) {
                                            // 团体卡固有: bond >= threshold (80 or 100)
                                            current_bond.map(|bond| bond >= bond_threshold)
                                        } else if sc_type == Some(1) {
                                            // 普通卡固有: bond >= threshold (80 or 100)
                                            current_bond.map(|bond| bond >= bond_threshold)
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    };

                                    if is_support_card && is_shining.is_none() {
                                        shining_complete = false;
                                    }

                                    if is_shining == Some(true) {
                                        shining_count += 1;
                                    }

                                    let is_shining_json = match is_shining {
                                        Some(true) => "true",
                                        Some(false) => "false",
                                        None => "null",
                                    };

                                    let is_unique_json = match is_unique_active {
                                        Some(true) => "true",
                                        Some(false) => "false",
                                        None => "null",
                                    };

                                    let sc_type_json = match sc_type {
                                        Some(t) => t.to_string(),
                                        None => "null".to_string(),
                                    };

                                    let is_tips_event = tips_partner_ids.contains(&partner_id);

                                    // ★ v2.3: partner_type 和 name（照 PC 版小黑板 personType 映射）
                                    // personType: 0=未加载, 1=友人卡, 2=普通支援卡, 3=NPC, 4=理事长, 5=记者, 6=不带卡佐岳
                                    let (partner_type, partner_name) = if is_support_card {
                                        let sc_type_val = sc_type.unwrap_or(1);
                                        let ptype = match sc_type_val {
                                            2 => 1, // 友人卡
                                            3 => 2, // 团体卡 → 当普通支援卡显示
                                            _ => 2, // 普通支援卡
                                        };
                                        // 名称从 MDB 查（后续优化），暂时用位置
                                        let name = format!("支援位{}", partner_id);
                                        (ptype, name)
                                    } else {
                                        // NPC/理事长/记者 — 按常见 ID 范围判断
                                        // 暂时全部标为 NPC
                                        (0, format!("伙伴{}", partner_id))
                                    };

                                    partners_json.push(format!(
                                        r#"{{"partner_id":{},"support_position":{},"support_card_id":{},"current_bond":{},"is_shining":{},"is_unique_active":{},"bond_threshold":{},"support_card_type":{},"is_tips_event":{},"partner_type":{},"name":"{}","bond_gain":null}}"#,
                                        partner_id,
                                        support_position,
                                        support_card_id_json,
                                        bond_json,
                                        is_shining_json,
                                        is_unique_json,
                                        bond_threshold,
                                        sc_type_json,
                                        is_tips_event,
                                        partner_type,
                                        json_escape(&partner_name),
                                    ));
                                }
                            }
                        }

                        let heads = partner_ids.len() as i32;

                        // Training-level shining count:
                        //
                        //   >= 0: confirmed number of shining cards
                        //     -1: unknown because a present support card could
                        //         not be classified safely
                        //
                        // TipsEventPartnerArray is intentionally not used.
                        let is_attribute_training = normalize_training_command_id(cid).is_some();

                        let shining = if !is_attribute_training {
                            // Rest, outing and other non-training commands do not have
                            // an ordinary friendship-training count.
                            -1
                        } else if shining_complete {
                            shining_count
                        } else {
                            -1
                        };

                        let shining_json = if shining >= 0 {
                            shining.to_string()
                        } else {
                            "null".to_string()
                        };

                        // Training gains from HomeInfoData.
                        //
                        // Runtime capture confirmed that each
                        // SingleModeParamsIncDecInfoData object contains:
                        //
                        //   +0x10 TargetType (inline ObscuredInt)
                        //   +0x24 Value      (inline ObscuredInt)
                        //
                        // The array itself is an IL2CPP reference array.
                        let mut gains = Vec::new();
                        let mut stat_gains = [0i32; 5];
                        let mut skill_pt_gain = 0i32;
                        let mut vital_cost = 0i32;

                        let params_array = read_ptr_at(ep as *const c_void, 96);

                        if !params_array.is_null() {
                            let array_base = params_array as *const u8;

                            let params_len = std::ptr::read_unaligned::<usize>(
                                array_base.add(IL2CPP_LIST_COUNT_OFF) as *const usize,
                            );

                            if params_len > 0 && params_len < 100 {
                                for j in 0..params_len {
                                    let param = std::ptr::read_unaligned::<*mut c_void>(
                                        array_base
                                            .add(IL2CPP_LIST_ITEMS_OFF + j * IL2CPP_LIST_ITEM_SIZE)
                                            as *const *mut c_void,
                                    );

                                    if param.is_null() {
                                        continue;
                                    }

                                    let target_type = read_obscured_int_at(
                                        param as *const c_void,
                                        PARAMS_INCDEC_DATA_TARGET_TYPE_OFF,
                                    );

                                    let value = read_obscured_int_at(
                                        param as *const c_void,
                                        PARAMS_INCDEC_DATA_VALUE_OFF,
                                    );

                                    if value == 0 {
                                        continue;
                                    }

                                    let target_name = match target_type {
                                        1 => "Speed",
                                        2 => "Stamina",
                                        3 => "Power",
                                        4 => "Guts",
                                        5 => "Wiz",
                                        10 => "HP",
                                        20 => "Motivation",
                                        30 => "SkillPt",
                                        _ => "Unknown",
                                    };

                                    // Include the numeric type in unknown keys
                                    // so malformed/unrecognised entries cannot
                                    // produce duplicate "Unknown" JSON keys.
                                    if target_name == "Unknown" {
                                        gains.push(format!(
                                            r#""Unknown_{}":{}"#,
                                            target_type, value
                                        ));
                                    } else {
                                        gains.push(format!(r#""{}":{}"#, target_name, value));
                                    }

                                    match target_type {
                                        1 => stat_gains[0] += value,
                                        2 => stat_gains[1] += value,
                                        3 => stat_gains[3] += value,
                                        4 => stat_gains[2] += value,
                                        5 => stat_gains[4] += value,
                                        10 => vital_cost += value,
                                        30 => skill_pt_gain += value,
                                        _ => {}
                                    }
                                }
                            }
                        }

                        // ★ v3.24.10: Ramen gains 直接用 HomeInfoData 读到的值
                        // 诊断确认: DataSet.CommandInfoArray.ParamsIncDecInfoArray 为空
                        // HomeInfoData.ParamsIncDecInfoArray 有数据 (params_len=4)

                        trs.push(format!(
                            r#"{{"name":"{}","command_id":{},"is_enable":{},"failure_rate":{},"heads":{},"shining":{},"partner_ids":[{}],"partners":[{}],"gains":{{{}}}}}"#,
                            cname,
                            cid,
                            is_enable,
                            failure_rate,
                            heads,
                            shining_json,
                            partner_ids
                                .iter()
                                .map(|value| value.to_string())
                                .collect::<Vec<_>>()
                                .join(","),
                            partners_json.join(","),
                            gains.join(","),
                        ));

                        // ★ v3.15.1: collect eval training data in same pass
                        if cmd_id_to_train_idx(cid).is_some() {
                            eval_trainings.push((
                                cid,
                                stat_gains,
                                skill_pt_gain,
                                vital_cost,
                                failure_rate,
                                is_enable,
                                shining,
                                heads,
                            ));
                        }
                    }
                    tr_json = format!("[{}]", trs.join(","));
                }
            }
        }
    }

    log_predict_step("S:training partners done");
    // --- Support cards (graceful fallback) ---
    log_predict_step("S:p2 done");
    boot_trace("summary_p3");
    ura_log(3, "★ read_summary phase3: support cards");
    log_predict_step("S:p3 cards");
    let mut sc_json = "[]".to_string();
    // ★ v3.22.89: Fix support_cards — use get_EquipSupportCardArray getter
    // Root cause: field name is "EquipSupportCardArray" not "SupportCardArray"
    // v3.22.89's cached_find_field_offset("SupportCardArray") hit wrong field via substring match
    // Also: position/supportCardId/limitBreakCount are ObscuredInt, not plain int
    // ★ v3.24.13: Reuse the array already fetched for shining detection —
    // eliminates a duplicate il2cpp_runtime_invoke that caused SIGSEGV.
    let mut sc_arr: *mut c_void = support_array_for_shining;
    ura_log(
        3,
        &format!("sc: reused shining array ptr={}", !sc_arr.is_null()),
    );
    // Method 2: direct field offset on chara_class (fallback)
    if sc_arr.is_null() {
        let sc_off = cached_find_field_offset(chara_class, "EquipSupportCardArray");
        if sc_off >= 0 {
            sc_arr = read_ptr_at(chara_obj as *const c_void, sc_off);
            ura_log(
                3,
                &format!("sc: offset={} ptr={}", sc_off, !sc_arr.is_null()),
            );
        }
    }
    // Parse the List<SingleModeEquipSupportCard>
    if !sc_arr.is_null() {
        let ab = sc_arr as *const u8;
        let al = std::ptr::read_unaligned::<usize>(ab.add(IL2CPP_LIST_COUNT_OFF) as *const usize);
        if al > 0 && al < 100 {
            let mut scs = Vec::new();
            for i in 0..al {
                let ep = std::ptr::read_unaligned::<*mut c_void>(
                    ab.add(IL2CPP_LIST_ITEMS_OFF + i * IL2CPP_LIST_ITEM_SIZE) as *const *mut c_void,
                );
                if ep.is_null() {
                    continue;
                }
                // ★ v3.24.9: Direct memory read for EquipSupportCard (zero invoke)
                // Offsets confirmed by /debug/dumpclass:
                //   Position=16  SupportCardId=36  LimitBreakCount=56  Exp=76  RentalType=136
                let sc_ep = ep as *const c_void;
                let position = read_obscured_int_at(sc_ep, 16);
                let support_card_id = read_obscured_int_at(sc_ep, 36);
                let limit_break_count = read_obscured_int_at(sc_ep, 56);
                let sc_exp = read_obscured_int_at(sc_ep, 76); // ★ 新增: 支援卡经验值
                let rental_type = read_obscured_int_at(sc_ep, 136);
                // TrainingPartnerState is not in EquipSupportCard fields — it's on a different object
                // Skip it (set to -1) to avoid invoke
                let training_partner_state = -1;
                // CharaId is a computed property. The app can resolve it
                // through support_card_id and card_db.json.
                let sc_chara_id = -1;

                // Runtime capture confirmed that support-card positions
                // 1..=6 are also the corresponding partner IDs.
                let kizuna = partner_evaluation.get(&position).copied().unwrap_or(-1);
                scs.push(format!(
                    r#"{{"position":{},"support_card_id":{},"limit_break_count":{},"training_partner_state":{},"chara_id":{},"kizuna":{},"exp":{},"rental_type":{}}}"#,
                    position, support_card_id, limit_break_count, training_partner_state, sc_chara_id, kizuna, sc_exp, rental_type
                ));
            }
            sc_json = format!("[{}]", scs.join(","));
            ura_log(
                3,
                &format!(
                    "sc: {} cards found, partner_evaluation: {} entries",
                    scs.len(),
                    partner_evaluation.len()
                ),
            );
        }
    }

    // --- Partner evaluation/bond (confirmed Evaluation layout) ---
    log_predict_step("S:p3 done");
    boot_trace("summary_p4");
    ura_log(3, "★ read_summary phase4: partner evaluation");
    log_predict_step("S:p4 eval");

    let mut evaluation_entries: Vec<(i32, i32)> = partner_evaluation
        .iter()
        .map(|(&partner_id, &evaluation)| (partner_id, evaluation))
        .collect();

    // HashMap iteration order is undefined, so sort by partner ID
    // to keep /summary stable between requests.
    evaluation_entries.sort_unstable_by_key(|&(partner_id, _)| partner_id);

    let ev_json = format!(
        "[{}]",
        evaluation_entries
            .iter()
            .map(|&(partner_id, evaluation)| {
                format!(
                    r#"{{"target_id":{},"partner_id":{},"evaluation":{},"current_bond":{}}}"#,
                    partner_id, partner_id, evaluation, evaluation
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    );

    // --- Training levels (graceful fallback) ---
    log_predict_step("S:p4 done");
    boot_trace("summary_p5");
    ura_log(3, "★ read_summary phase5: training_levels");
    log_predict_step("S:p5 levels");
    let mut tl_json = "[]".to_string();
    let tl_arr = read_field_value(chara_class, chara_obj, "training_level_info_array");
    if tl_arr.is_null() {
        let arr = call_getter_on_instance(chara_class, chara_obj, "get_TrainingLevelInfoArray"); // [INVOKE-08] get_TrainingLevelInfoArray — 唯一调用
        if !arr.is_null() {
            let ab = arr as *const u8;
            let al =
                std::ptr::read_unaligned::<usize>(ab.add(IL2CPP_LIST_COUNT_OFF) as *const usize);
            if al > 0 && al < 100 {
                let mut tls = Vec::new();
                for i in 0..al {
                    let ep = std::ptr::read_unaligned::<*mut c_void>(
                        ab.add(IL2CPP_LIST_ITEMS_OFF + i * IL2CPP_LIST_ITEM_SIZE)
                            as *const *mut c_void,
                    );
                    if ep.is_null() {
                        continue;
                    }
                    let b = ep as *const u8;
                    let command_id =
                        std::ptr::read_unaligned::<i32>(b.add(IL2CPP_COMMAND_ID_OFF) as *const i32);
                    let level = std::ptr::read_unaligned::<i32>(
                        b.add(IL2CPP_COMMAND_LEVEL_OFF) as *const i32
                    );
                    tls.push(format!(
                        r#"{{"command_id":{},"level":{}}}"#,
                        command_id, level
                    ));
                }
                tl_json = format!("[{}]", tls.join(","));
            }
        }
    } else {
        let ab = tl_arr as *const u8;
        let al = std::ptr::read_unaligned::<usize>(ab.add(IL2CPP_LIST_COUNT_OFF) as *const usize);
        if al > 0 && al < 100 {
            let mut tls = Vec::new();
            for i in 0..al {
                let ep = std::ptr::read_unaligned::<*mut c_void>(
                    ab.add(IL2CPP_LIST_ITEMS_OFF + i * IL2CPP_LIST_ITEM_SIZE) as *const *mut c_void,
                );
                if ep.is_null() {
                    continue;
                }
                let b = ep as *const u8;
                let command_id =
                    std::ptr::read_unaligned::<i32>(b.add(IL2CPP_COMMAND_ID_OFF) as *const i32);
                let level =
                    std::ptr::read_unaligned::<i32>(b.add(IL2CPP_COMMAND_LEVEL_OFF) as *const i32);
                tls.push(format!(
                    r#"{{"command_id":{},"level":{}}}"#,
                    command_id, level
                ));
            }
            tl_json = format!("[{}]", tls.join(","));
        }
    }

    // --- Buffs: chara_effect_ids → readable names (ALL scenarios) + EnhanceGroup (Breeders) ---
    log_predict_step("S:p5 done");
    boot_trace("summary_p6");
    ura_log(3, "★ read_summary phase6: buffs");
    log_predict_step("S:p6 buffs");
    // ★ v3.14.2: Always generate buffs from chara_effect_ids first
    let mut buff_json = effects_to_buffs_json(&chara_effect_ids);
    // ★ v3.22.51: sid==14 skips try_get_scenario_obj (data pre-read in ramen section)
    let scenario_obj = if sid == 14 {
        ptr::null_mut()
    } else {
        try_get_scenario_obj(chara_class, chara_obj, sid)
    };
    if !scenario_obj.is_null() {
        let sc_name = match sid {
            1 => "WorkSingleModeScenarioURA",
            2 => "WorkSingleModeScenarioTeamRace",
            3 => "WorkSingleModeScenarioLive",
            4 => "WorkSingleModeScenarioFree",
            5 => "WorkSingleModeScenarioVenus",
            6 => "WorkSingleModeScenarioArc",
            7 => "WorkSingleModeScenarioSport",
            8 => "WorkSingleModeScenarioCook",
            9 => "WorkSingleModeScenarioMecha",
            10 => "WorkSingleModeScenarioLegend",
            11 => "WorkSingleModeScenarioPioneer",
            12 => "WorkSingleModeScenarioOnsen",
            13 => "WorkSingleModeScenarioBreeders",
            14 => "WorkSingleModeScenarioRamen",
            _ => "",
        };
        if !sc_name.is_empty() {
            let sc_class = find_class_by_short_name(image, sc_name);
            if !sc_class.is_null() {
                let ds_obj = call_getter_on_instance(sc_class, scenario_obj, "get_DataSet"); // [INVOKE-09] get_DataSet — ★ 与 INVOKE-05 重复，待去重
                if !ds_obj.is_null() {
                    let ds_name = format!("{}DataSet", sc_name);
                    let ds_class = find_class_by_short_name(image, &ds_name);
                    if !ds_class.is_null() {
                        // ★ Breeders EnhanceGroups → override chara_effect_ids buffs
                        if sid == 13 {
                            let enhance_cls = find_class_by_short_name(
                                image,
                                "ObscuredSingleModeBreedersEnhanceGroup",
                            );
                            if !enhance_cls.is_null() {
                                let enhance_arr = call_getter_on_instance(
                                    // [INVOKE-10] get_EnhanceGroupArray — 循环外
                                    ds_class,
                                    ds_obj,
                                    "get_EnhanceGroupArray",
                                );
                                if !enhance_arr.is_null() {
                                    let eb = enhance_arr as *const u8;
                                    let el = std::ptr::read_unaligned::<usize>(
                                        eb.add(IL2CPP_LIST_COUNT_OFF) as *const usize,
                                    );
                                    if el > 0 && el < 20 {
                                        let mut buffs = Vec::new();
                                        for i in 0..el {
                                            let ep =
                                                std::ptr::read_unaligned::<*mut c_void>(eb.add(
                                                    IL2CPP_LIST_ITEMS_OFF
                                                        + i * IL2CPP_LIST_ITEM_SIZE,
                                                )
                                                    as *const *mut c_void);
                                            if ep.is_null() {
                                                continue;
                                            }
                                            let gt = call_getter_obscured_int(
                                                // [INVOKE-11] get_GainTotal (obscured) — 循环内倍增
                                                enhance_cls,
                                                ep,
                                                "get_GroupType",
                                            );
                                            let lv = call_getter_obscured_int(
                                                // [INVOKE-12] get_Level (obscured) — 循环内倍增
                                                enhance_cls,
                                                ep,
                                                "get_Level",
                                            );
                                            let (gtn, desc) = breeders_buff_desc(gt, lv);
                                            buffs.push(format!(r#"{{"name":"{}","level":{},"desc":"{}","type":"Breeders"}}"#, gtn, lv, desc));
                                        }
                                        if !buffs.is_empty() {
                                            buff_json = format!("[{}]", buffs.join(","));
                                        }
                                    }
                                }
                            }
                        }
                        // ★ v3.22.89: Removed dead Ramen buffs code here
                        // (sid==14 sets scenario_obj=null, so this block never executes for Ramen.
                        //  Ramen buffs are handled below after the scenario_obj block.)
                    }
                }
            }
        }
    }

    // ★ v3.22.51: Ramen buffs — extracted outside nested block (uses pre-read data only)
    if sid == 14 && !ramen_active_effects_raw_json.is_empty() {
        let mut buffs = Vec::new();
        for ae_part in ramen_active_effects_raw_json.split("},{") {
            let mut cat: i32 = -1;
            let mut eid: i32 = 0;
            let mut val: i32 = 0;
            for field in ae_part
                .trim_start_matches('{')
                .trim_end_matches('}')
                .split(',')
            {
                let fv: Vec<&str> = field.splitn(2, ':').collect();
                if fv.len() == 2 {
                    let key = fv[0].trim();
                    if key.contains("category") {
                        cat = fv[1].parse().unwrap_or(-1);
                    } else if key.contains("id") && !key.contains("Eff") {
                        eid = fv[1].parse().unwrap_or(0);
                    } else if key.contains("value") {
                        val = fv[1].parse().unwrap_or(0);
                    }
                }
            }
            if cat >= 0 {
                // Runtime/MDB evidence: category 1 IDs address
                // single_mode_14_region_effect rows; category 2 IDs address
                // single_mode_14_basic_effect rows. Keep category 4 generic
                // until its ID domain is independently proven.
                let cat_name = match cat {
                    1 => "地区效果",
                    2 => "吃面效果",
                    4 => "特殊效果",
                    _ => "未解析效果",
                };
                let name = format!("{}#{}", cat_name, eid);
                // EffectValue is not universally a percentage (some effects
                // add a character, trigger hints, or raise a cap), so the SO
                // must not invent a display unit. Consumers resolve semantics
                // from EffectCategory + EffectId and retain this raw value.
                buffs.push(format!(
                    r#"{{"name":"{}","EffectCategory":{},"EffectId":{},"EffectValue":{},"desc":"","type":"Ramen"}}"#,
                    name, cat, eid, val
                ));
            }
        }
        if ramen_uraf_type >= 0 {
            // UrafEffectType is a separate enum. Do not reuse ActiveEffect
            // category labels until its values are independently mapped.
            let state_name = match ramen_uraf_state {
                0 => "无效",
                1 => "有效",
                _ => "未知",
            };
            buffs.push(format!(
                r#"{{"name":"特殊机制","UrafEffectType":{},"type":"Ramen"}}"#,
                ramen_uraf_type
            ));
            buffs.push(format!(
                r#"{{"name":"特殊机制状态","state":"{}","UrafEffectState":{},"type":"Ramen"}}"#,
                state_name, ramen_uraf_state
            ));
        }
        if !buffs.is_empty() {
            buff_json = format!("[{}]", buffs.join(","));
        }
    }

    // ★ state field removed: get_State() doesn't exist on WorkSingleModeCharaData
    // Health condition is now detected via chara_effect_ids (top-level array)
    // ★ AI Evaluation (v3.15.1): compute score and training recommendation
    // FIXED: no more double-read of CommandInfoArray — eval_trainings collected in phase2
    log_predict_step("S:buffs done");
    let ai_json = if sid == 14 {
        // Ramen UI uses a countdown. Until its mapping to MDB/internal progress is verified,
        // suppress recommendations whose score changes with elapsed turn/year/next race.
        r#"{"status":"unavailable","reason":"ramen_turn_semantics_unverified","timing_dependent_recommendation":false}"#.to_string()
    } else {
        let turn = std::cmp::min((mon - 1) * 2 + (half - 1), 71);
        let stats = [spd, sta, pow_, gut, wiz];

        // Detect buffs from chara_effect_ids
        let has_ai_jiao = chara_effect_ids.iter().any(|&id| id == 8);
        let has_renshou_jouzu = chara_effect_ids.iter().any(|&id| id == 10 || id == 11);

        // Non-Ramen path only. Ramen next-race lookup is disabled until turn mapping is verified.
        let next_race = false;

        let result = evaluate_ai(
            turn,
            stats,
            vit,
            mvit,
            mot,
            sid,
            &eval_trainings,
            has_ai_jiao,
            has_renshou_jouzu,
            skill_eval,
            skill_count, // ★ v3.22.0
            &ramen_gauge_gains_map,
            ramen_special_feeling_num,
            next_race,
        );
        ai_result_to_json(&result)
    };

    // ★ Breeders team member data (v3.15.4)
    let team_json = if sid == 13 {
        let team_result = read_breeders_team();
        if team_result.contains("\"team_members\"") {
            format!(r#","team_data":{}"#, team_result)
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    // ★ v3.22.39: Ramen scenario data — sozai counts aggregated during read
    let ramen_json = if sid == 14 && ramen_checkpoint_pt >= 0 {
        // ★ v3.24.68: moriagari_level 改用 MDB check_point_pt_effect 真实11档
        // (旧阈值480/330/210/120/50为猜测值，错误)
        // [MDB] 0-249=0, 250-499=1, 500-999=2, 1000-1499=3, 1500-1999=4,
        //       2000-2499=5, 2500-2999=6, 3000-3499=7, 3500-3999=8, 4000-4999=9, 5000+=10
        let moriagari_level = if ramen_checkpoint_pt >= 5000 {
            10
        } else if ramen_checkpoint_pt >= 4000 {
            9
        } else if ramen_checkpoint_pt >= 3500 {
            8
        } else if ramen_checkpoint_pt >= 3000 {
            7
        } else if ramen_checkpoint_pt >= 2500 {
            6
        } else if ramen_checkpoint_pt >= 2000 {
            5
        } else if ramen_checkpoint_pt >= 1500 {
            4
        } else if ramen_checkpoint_pt >= 1000 {
            3
        } else if ramen_checkpoint_pt >= 500 {
            2
        } else if ramen_checkpoint_pt >= 250 {
            1
        } else {
            0
        };
        format!(
            r#","ramen":{{"checkpoint_pt":{},"moriagari_level":{},"special_feeling_num":{},"recommend_type":{},"sozai":[{},{},{}],"feeling_info":[{}],"acquisition_gauges":[{}],"command_feelings":[{}],"command_gauge_vectors":[{}],"selected_region_ids":[{}],"selectable_region_ids_derived":[{}],"selectable_region_ids_source":"{}","region_pool_for_latest_selection_phase_derived":[{}],"currently_selectable_status":"{}","active_effects":[{}],"gauge_gains":[{}]}}"#,
            ramen_checkpoint_pt,
            moriagari_level,
            ramen_special_feeling_num,
            ramen_recommend_type,
            ramen_sozai_counts[0],
            ramen_sozai_counts[1],
            ramen_sozai_counts[2],
            ramen_feeling_info_json,
            ramen_acquisition_gauges_json,
            ramen_command_feelings_json,
            ramen_command_gauge_vectors_json,
            ramen_selected_region_ids_json,
            ramen_selectable_region_ids_derived_json,
            if ramen_selectable_region_ids_derived_json.is_empty() {
                "unknown"
            } else {
                "mdb_pool_minus_all_selected_derivation"
            },
            ramen_region_pool_phase_derived_json,
            if !ramen_selectable_region_ids_derived_json.is_empty() {
                "selection_turn_pool_is_current_derived"
            } else if !ramen_region_pool_phase_derived_json.is_empty() {
                "unknown_between_selection_rounds"
            } else {
                "unknown"
            },
            ramen_active_effects_raw_json,
            ramen_gauge_gains_json
        )
    } else {
        String::new()
    };

    // ★ v2.2: last_action 字段 — 从缓存读取，不调用 IL2CPP
    let last_action_json = {
        let (cmd_id, seq) = ACTION_STATE
            .lock()
            .map(|state| (state.command_id, state.sequence))
            .unwrap_or((-1, 0));
        if cmd_id >= 0 {
            let (action, normalized) = match cmd_id {
                101 => ("Speed", 101),
                102 => ("Power", 102),
                103 => ("Guts", 103),
                105 => ("Stamina", 105),
                106 => ("Wiz", 106),
                601 => ("Speed", 101),
                602 => ("Stamina", 105),
                603 => ("Power", 102),
                604 => ("Guts", 103),
                605 => ("Wiz", 106),
                _ => ("Unknown", cmd_id),
            };
            format!(
                r#","last_action":{{"sequence":{},"raw_command_id":{},"normalized_command_id":{},"action":"{}","source":"training_hook"}}"#,
                seq, cmd_id, normalized, action
            )
        } else {
            String::new()
        }
    };

    log_predict_step("S:json");
    format!(
        r#"{{"version":"{}","year":{},"turn":{},"raw_total_turn_num":{},"ui_turn_semantics":"countdown","raw_field_mapping":"unverified","month":{},"half":{},"scenario":"{}","chara_id":{},"stats":{{"speed":{},"stamina":{},"power":{},"guts":{},"wiz":{},"vital":{},"max_vital":{},"motivation":"{}","skill_point":{},"fan":{}}},"max_stats":{{"speed":{},"stamina":{},"power":{},"guts":{},"wiz":{}}},"proper":{{"dist_short":{},"dist_mile":{},"dist_mid":{},"dist_long":{},"ground_turf":{},"ground_dirt":{}}},"running_style":{},"scenario_progress":{},"training_event_type":{},"talent_level":{},"chara_grade":{},"difficulty":{},"fixed_turn_chara_seed":{},"trainings":{},"support_cards":{},"evaluation":{},"training_levels":{},"buffs":{},"chara_effect_ids":[{}],"skills":{{"eval":{},"count":{},"list":{}}},"ai":{}{}{}{} }}"#,
        PLUGIN_VERSION,
        year,
        cumulative_turn,
        raw_total_turn_num,
        mon,
        half,
        scn_s,
        chara_id,
        spd,
        sta,
        pow_,
        gut,
        wiz,
        vit,
        mvit,
        mot_s,
        spt,
        fan,
        max_spd,
        max_sta,
        max_pow,
        max_gut,
        max_wiz,
        proper_dist_short,
        proper_dist_mile,
        proper_dist_mid,
        proper_dist_long,
        proper_ground_turf,
        proper_ground_dirt,
        running_style,
        scenario_progress,
        training_event_type,
        talent_level,
        chara_grade,
        difficulty,
        fixed_turn_chara_seed,
        tr_json,
        sc_json,
        ev_json,
        tl_json,
        buff_json,
        effect_ids_str.join(","),
        skill_eval,
        skill_count,
        skills_json,
        ai_json,
        team_json,
        ramen_json,
        last_action_json
    )
}
```

## `safe_maps_summary` (starts at line 7391)

```rust
fn safe_maps_summary() -> String {
    let maps=match safe_maps(){Ok(v)=>v,Err(e)=>return format!(r#"{{"ok":false,"error":"maps_read_failed","detail":"{}"}}"#,safe_json(&e.to_string()))};
    let readable=maps.iter().filter(|m|m.perms.starts_with('r')).count();
    let sample=maps.iter().filter(|m|m.perms.starts_with('r')).take(64).map(|m|format!(r#"{{"start":"0x{:x}","end":"0x{:x}","size":{},"perms":"{}","path":"{}"}}"#,m.start,m.end,m.end-m.start,safe_json(&m.perms),safe_json(&m.path))).collect::<Vec<_>>().join(",");
    format!(r#"{{"ok":true,"maps_total":{},"readable":{},"sample_limited":true,"maps":[{}]}}"#,maps.len(),readable,sample)
}
```

## `parse_query` (starts at line 13917)

```rust
fn parse_query(full_uri: &str, key: &str) -> String {
    let pattern = format!("{}=", key);
    if let Some(q) = full_uri.find(&format!("?{}", pattern)) {
        let start = q + 1 + pattern.len();
        let end = full_uri[start..]
            .find('&')
            .map(|e| start + e)
            .unwrap_or(full_uri.len());
        url_decode(&full_uri[start..end])
    } else if let Some(q) = full_uri.find(&format!("&{}", pattern)) {
        let start = q + 1 + pattern.len();
        let end = full_uri[start..]
            .find('&')
            .map(|e| start + e)
            .unwrap_or(full_uri.len());
        url_decode(&full_uri[start..end])
    } else {
        String::new()
    }
}
```

## `read_saddles` (starts at line 16137)

```rust
fn read_saddles() -> String {
    let mdb_path = match find_mdb_path() {
        Some(p) => p,
        None => {
            return r#"{"error":"mdb_not_found","hint":"MasterDB file not found on device"}"#
                .to_string()
        }
    };

    let conn = match Connection::open_with_flags(&mdb_path, OpenFlags::SQLITE_OPEN_READ_ONLY) {
        Ok(c) => c,
        Err(e) => return format!(r#"{{"error":"open_failed","detail":"{}"}}"#, e),
    };

    // Collect G1 win saddles (win_saddle_type=3)
    let saddles: Vec<String> = match conn.prepare(
        "SELECT id, priority, group_id, relation_group_id, condition, win_saddle_type, race_instance_id_1, race_instance_id_2, race_instance_id_3, race_instance_id_4, race_instance_id_5, race_instance_id_6, race_instance_id_7, race_instance_id_8 FROM single_mode_wins_saddle WHERE win_saddle_type=3 ORDER BY id"
    ) {
        Ok(mut stmt) => stmt.query_map([], |row| {
            Ok(format!(
                r#"{{"id":{},"priority":{},"group_id":{},"relation_group_id":{},"condition":{},"race_ids":[{},{},{},{},{},{},{},{}]}}"#,
                row.get::<_, i32>(0).unwrap_or(0),
                row.get::<_, i32>(1).unwrap_or(0),
                row.get::<_, i32>(2).unwrap_or(0),
                row.get::<_, i32>(3).unwrap_or(0),
                row.get::<_, i32>(4).unwrap_or(0),
                row.get::<_, i32>(6).unwrap_or(0),
                row.get::<_, i32>(7).unwrap_or(0),
                row.get::<_, i32>(8).unwrap_or(0),
                row.get::<_, i32>(9).unwrap_or(0),
                row.get::<_, i32>(10).unwrap_or(0),
                row.get::<_, i32>(11).unwrap_or(0),
                row.get::<_, i32>(12).unwrap_or(0),
                row.get::<_, i32>(13).unwrap_or(0),
            ))
        }).unwrap().filter_map(|r| r.ok()).collect(),
        Err(e) => return format!(r#"{{"error":"saddle_prepare_failed","detail":"{}"}}"#, e),
    };

    // Collect chara_program (which chara runs which program_group)
    let chara_programs: Vec<String> = match conn.prepare(
        "SELECT chara_id, program_group, program_group_2 FROM single_mode_chara_program ORDER BY program_group, chara_id"
    ) {
        Ok(mut stmt) => stmt.query_map([], |row| {
            Ok(format!(
                r#"{{"chara_id":{},"program_group":{},"program_group_2":{}}}"#,
                row.get::<_, i32>(0).unwrap_or(0),
                row.get::<_, i32>(1).unwrap_or(0),
                row.get::<_, i32>(2).unwrap_or(0),
            ))
        }).unwrap().filter_map(|r| r.ok()).collect(),
        Err(e) => return format!(r#"{{"error":"program_prepare_failed","detail":"{}"}}"#, e),
    };

    // Collect program race mapping
    let programs: Vec<String> = match conn.prepare(
        "SELECT id, program_group, race_instance_id, month, half FROM single_mode_program ORDER BY program_group, month, half"
    ) {
        Ok(mut stmt) => stmt.query_map([], |row| {
            Ok(format!(
                r#"{{"id":{},"program_group":{},"race_instance_id":{},"month":{},"half":{}}}"#,
                row.get::<_, i32>(0).unwrap_or(0),
                row.get::<_, i32>(1).unwrap_or(0),
                row.get::<_, i32>(2).unwrap_or(0),
                row.get::<_, i32>(3).unwrap_or(0),
                row.get::<_, i32>(4).unwrap_or(0),
            ))
        }).unwrap().filter_map(|r| r.ok()).collect(),
        Err(e) => return format!(r#"{{"error":"prog_prepare_failed","detail":"{}"}}"#, e),
    };

    // Collect race names (category=32 = race name in text_data)
    let race_names: Vec<String> = match conn.prepare(&format!(
        "SELECT [index], text FROM text_data WHERE category={} ORDER BY [index]",
        TEXT_DATA_CATEGORY_RACE_NAME
    )) {
        Ok(mut stmt) => stmt
            .query_map([], |row| {
                let text: String = row
                    .get::<_, Option<String>>(1)
                    .unwrap_or(None)
                    .unwrap_or_default();
                Ok(format!(
                    r#"{{"race_id":{},"name":"{}"}}"#,
                    row.get::<_, i32>(0).unwrap_or(0),
                    json_escape(&text),
                ))
            })
            .unwrap()
            .filter_map(|r| r.ok())
            .collect(),
        Err(e) => return format!(r#"{{"error":"race_name_prepare_failed","detail":"{}"}}"#, e),
    };

    // Collect chara names (category=6 = chara name in text_data)
    let chara_names: Vec<String> = match conn.prepare(&format!(
        "SELECT [index], text FROM text_data WHERE category={} ORDER BY [index]",
        TEXT_DATA_CATEGORY_CHARA_NAME
    )) {
        Ok(mut stmt) => stmt
            .query_map([], |row| {
                let text: String = row
                    .get::<_, Option<String>>(1)
                    .unwrap_or(None)
                    .unwrap_or_default();
                Ok(format!(
                    r#"{{"chara_id":{},"name":"{}"}}"#,
                    row.get::<_, i32>(0).unwrap_or(0),
                    json_escape(&text),
                ))
            })
            .unwrap()
            .filter_map(|r| r.ok())
            .collect(),
        Err(e) => {
            return format!(
                r#"{{"error":"chara_name_prepare_failed","detail":"{}"}}"#,
                e
            )
        }
    };

    // Collect succession_relation (fixed compatibility scores)
    let relations: Vec<String> = match conn.prepare(
        "SELECT relation_type, relation_point FROM succession_relation ORDER BY relation_type, relation_point"
    ) {
        Ok(mut stmt) => stmt.query_map([], |row| {
            Ok(format!(
                r#"{{"relation_type":{},"relation_point":{}}}"#,
                row.get::<_, i32>(0).unwrap_or(0),
                row.get::<_, i32>(1).unwrap_or(0),
            ))
        }).unwrap().filter_map(|r| r.ok()).collect(),
        Err(e) => return format!(r#"{{"error":"relation_prepare_failed","detail":"{}"}}"#, e),
    };

    // Collect succession_relation_member
    let relation_members: Vec<String> = match conn.prepare(
        "SELECT id, relation_type, chara_id FROM succession_relation_member ORDER BY relation_type, chara_id"
    ) {
        Ok(mut stmt) => stmt.query_map([], |row| {
            Ok(format!(
                r#"{{"id":{},"relation_type":{},"chara_id":{}}}"#,
                row.get::<_, i32>(0).unwrap_or(0),
                row.get::<_, i32>(1).unwrap_or(0),
                row.get::<_, i32>(2).unwrap_or(0),
            ))
        }).unwrap().filter_map(|r| r.ok()).collect(),
        Err(e) => return format!(r#"{{"error":"member_prepare_failed","detail":"{}"}}"#, e),
    };

    // Collect race_instance to race_course_set mapping (for venue info)
    let race_instances: Vec<String> = match conn.prepare(
        "SELECT ri.id, ri.race_id, r.grade, r.course_set, cs.race_track_id, cs.distance, cs.ground FROM race_instance ri JOIN race r ON ri.race_id=r.id JOIN race_course_set cs ON r.course_set=cs.id WHERE r.grade=100 ORDER BY ri.id"
    ) {
        Ok(mut stmt) => stmt.query_map([], |row| {
            Ok(format!(
                r#"{{"id":{},"race_id":{},"grade":{},"course_set":{},"race_track_id":{},"distance":{},"ground":{}}}"#,
                row.get::<_, i32>(0).unwrap_or(0),
                row.get::<_, i32>(1).unwrap_or(0),
                row.get::<_, i32>(2).unwrap_or(0),
                row.get::<_, i32>(3).unwrap_or(0),
                row.get::<_, i32>(4).unwrap_or(0),
                row.get::<_, i32>(5).unwrap_or(0),
                row.get::<_, i32>(6).unwrap_or(0),
            ))
        }).unwrap().filter_map(|r| r.ok()).collect(),
        Err(e) => return format!(r#"{{"error":"race_inst_prepare_failed","detail":"{}"}}"#, e),
    };

    drop(conn);

    format!(
        r#"{{"ok":true,"version":"3.22.91","mdb":"{}","saddle_count":{},"program_chara_count":{},"program_count":{},"race_name_count":{},"chara_name_count":{},"relation_count":{},"member_count":{},"race_instance_count":{},"saddles":[{}],"chara_programs":[{}],"programs":[{}],"race_names":[{}],"chara_names":[{}],"relations":[{}],"relation_members":[{}],"race_instances":[{}]}}"#,
        mdb_path,
        saddles.len(),
        chara_programs.len(),
        programs.len(),
        race_names.len(),
        chara_names.len(),
        relations.len(),
        relation_members.len(),
        race_instances.len(),
        saddles.join(","),
        chara_programs.join(","),
        programs.join(","),
        race_names.join(","),
        chara_names.join(","),
        relations.join(","),
        relation_members.join(","),
        race_instances.join(","),
    )
}
```

## `read_hall_data` (starts at line 16335)

```rust
unsafe fn read_hall_data() -> String {
    if API.is_null() {
        return r#"{"error":"api_null"}"#.to_string();
    }
    let image = match get_image() {
        img if !img.is_null() => img,
        _ => return r#"{"error":"image_null"}"#.to_string(),
    };

    // 1. Get WDM singleton
    let wdm_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkDataManager").as_ptr(),
    );
    if wdm_class.is_null() {
        return r#"{"error":"no_wdm"}"#.to_string();
    }
    let wdm_inst = get_singleton(wdm_class);
    if wdm_inst.is_null() {
        return r#"{"error":"no_wdm_inst"}"#.to_string();
    }

    // 2. Get WorkTrainedCharaData from WDM
    let wtcd_inst = call_getter_ref(wdm_class, wdm_inst, "get_TrainedCharaData");
    if wtcd_inst.is_null() {
        ura_log(1, "/hall: get_TrainedCharaData returned null");
        return r#"{"error":"no_trained_chara_data"}"#.to_string();
    }
    ura_log(2, "/hall: got WorkTrainedCharaData instance");

    // 3. Find WorkTrainedCharaData class for calling get_List
    let wtcd_class = find_class_by_short_name(image, "WorkTrainedCharaData");

    // 4. Get List<TrainedCharaData> from WorkTrainedCharaData
    let list_obj = call_getter_ref(wtcd_class, wtcd_inst, "get_List");
    if list_obj.is_null() {
        ura_log(1, "/hall: get_List returned null");
        return r#"{"error":"no_list"}"#.to_string();
    }

    // 5. Read List<TrainedCharaData> internals
    // List<T> IL2CPP layout (64-bit):
    //   +0x00: Il2CppObject header (16 bytes)
    //   +0x10: _items (Il2CppArray* pointer, 8 bytes)
    //   +0x18: _size (int32, 4 bytes)
    let list_base = list_obj as *const u8;
    let items_arr = std::ptr::read_unaligned::<*mut c_void>(
        list_base.add(IL2CPP_LIST_ARRAY_OFF) as *const *mut c_void
    );
    let list_size =
        std::ptr::read_unaligned::<usize>(list_base.add(IL2CPP_LIST_COUNT_OFF) as *const usize)
            as i32;

    if items_arr.is_null() || list_size <= 0 {
        ura_log(1, &format!("/hall: List null or empty, size={}", list_size));
        return format!(r#"{{"error":"empty_list","list_size":{}}}"#, list_size);
    }
    ura_log(2, &format!("/hall: List has {} entries", list_size));

    // 6. Find TrainedCharaData class
    let tcd_class = find_class_by_short_name(image, "TrainedCharaData");
    if tcd_class.is_null() {
        ura_log(1, "/hall: TrainedCharaData class not found");
        return r#"{"error":"no_tcd_class"}"#.to_string();
    }

    // 7. Read array elements from List._items
    // Il2CppArray layout: +0x18: max_length (usize), +0x20: data[0]
    let arr_base = items_arr as *const u8;
    let arr_len =
        std::ptr::read_unaligned::<usize>(arr_base.add(IL2CPP_LIST_COUNT_OFF) as *const usize);

    let mut entries = Vec::new();
    let count = std::cmp::min(list_size as usize, std::cmp::min(arr_len, 200));

    for i in 0..count {
        let elem_ptr = std::ptr::read_unaligned::<*mut c_void>(
            arr_base.add(IL2CPP_LIST_ITEMS_OFF + i * IL2CPP_LIST_ITEM_SIZE) as *const *mut c_void,
        );
        if elem_ptr.is_null() {
            continue;
        }

        // Read fields via getter methods
        let card_id = call_getter_int(tcd_class, elem_ptr, "get_CardId");
        let speed = call_getter_int(tcd_class, elem_ptr, "get_Speed");
        let stamina = call_getter_int(tcd_class, elem_ptr, "get_Stamina");
        let power = call_getter_int(tcd_class, elem_ptr, "get_Power");
        let guts = call_getter_int(tcd_class, elem_ptr, "get_Guts");
        let wiz = call_getter_int(tcd_class, elem_ptr, "get_Wiz");
        let rank_score = call_getter_int(tcd_class, elem_ptr, "get_RankScore");
        let rank = call_getter_int(tcd_class, elem_ptr, "get_Rank");
        let scenario_id = call_getter_obscured_int(tcd_class, elem_ptr, "get_ScenarioId");
        let fans = call_getter_int(tcd_class, elem_ptr, "get_Fans");
        let rarity = call_getter_obscured_int(tcd_class, elem_ptr, "get_Rarity");

        // Skip entries with no meaningful data
        if speed <= 0 && stamina <= 0 && rank_score <= 0 {
            continue;
        }

        entries.push(format!(
            r#"{{"idx":{},"card_id":{},"speed":{},"stamina":{},"power":{},"guts":{},"wiz":{},"rank_score":{},"rank":{},"scenario_id":{},"fans":{},"rarity":{}}}"#,
            i, card_id, speed, stamina, power, guts, wiz, rank_score, rank, scenario_id, fans, rarity
        ));
    }

    if entries.is_empty() {
        return r#"{"error":"no_valid_entries"}"#.to_string();
    }

    ura_log(2, &format!("/hall: {} valid entries", entries.len()));
    format!(
        r#"{{"count":{},"entries":[{}]}}"#,
        entries.len(),
        entries.join(",")
    )
}
```

## `read_inherit_compat` (starts at line 19575)

```rust
unsafe fn read_inherit_compat() -> String {
    if API.is_null() {
        return r#"{"error":"api_null"}"#.to_string();
    }
    let image = match get_image() {
        img if !img.is_null() => img,
        _ => return r#"{"error":"image_null"}"#.to_string(),
    };

    let wdm_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkDataManager").as_ptr(),
    );
    if wdm_class.is_null() {
        return r#"{"error":"no_wdm"}"#.to_string();
    }
    let wdm_inst = get_singleton(wdm_class);
    if wdm_inst.is_null() {
        return r#"{"error":"no_wdm_inst"}"#.to_string();
    }
    log_predict_step("P:wdm");

    let sm_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkSingleModeData").as_ptr(),
    );
    let sm_obj = call_getter_ref(wdm_class, wdm_inst, "get_SingleMode");
    if sm_obj.is_null() {
        return r#"{"error":"no_sm"}"#.to_string();
    }

    let chara_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkSingleModeCharaData").as_ptr(),
    );
    let chara_obj = call_getter_ref(sm_class, sm_obj, "get_Character");
    if chara_obj.is_null() {
        return r#"{"error":"no_chara"}"#.to_string();
    }

    // 1. Read succession parent info
    // WorkSingleModeCharaData.SuccessionTrainedCharaInfoFirst (offset 0x48)
    // WorkSingleModeCharaData.SuccessionTrainedCharaInfoSecond (offset 0x50)
    let sci_class = find_class_by_short_name(image, "SuccessionCharaInfo");
    let first_sci = call_getter_ref(
        chara_class,
        chara_obj,
        "get_SuccessionTrainedCharaInfoFirst",
    );
    let second_sci = call_getter_ref(
        chara_class,
        chara_obj,
        "get_SuccessionTrainedCharaInfoSecond",
    );

    let mut first_chara_id: i32 = -1;
    let mut second_chara_id: i32 = -1;
    if !first_sci.is_null() && !sci_class.is_null() {
        first_chara_id = call_getter_int(sci_class, first_sci, "get_TrainedCharaId");
    }
    if !second_sci.is_null() && !sci_class.is_null() {
        second_chara_id = call_getter_int(sci_class, second_sci, "get_TrainedCharaId");
    }

    // 2. Read SuccessionFactor (offset 0x448 on CharaData) — factor count for compatibility
    let factor_arr = call_getter_on_instance(chara_class, chara_obj, "get_SuccessionFactor");
    let mut factor_count: i32 = 0;
    if !factor_arr.is_null() {
        let fb = factor_arr as *const u8;
        factor_count =
            std::ptr::read_unaligned::<usize>(fb.add(IL2CPP_LIST_COUNT_OFF) as *const usize) as i32;
    }

    // 3. Read relation data from mdb
    let mut relations_json: Vec<String> = Vec::new();
    let mut relation_members_json: Vec<String> = Vec::new();
    let mut relation_ranks_json: Vec<String> = Vec::new();

    if let Some(mdb_path) = find_mdb_path() {
        if let Ok(conn) = Connection::open_with_flags(&mdb_path, OpenFlags::SQLITE_OPEN_READ_ONLY) {
            // succession_relation: type + point pairs
            if let Ok(mut stmt) = conn.prepare("SELECT relation_type, relation_point FROM succession_relation ORDER BY relation_type") {
                let rows: Vec<String> = stmt.query_map([], |row| {
                    Ok(format!(r#"{{"type":{},"point":{}}}"#,
                        row.get::<_, i32>(0).unwrap_or(0),
                        row.get::<_, i32>(1).unwrap_or(0)))
                }).unwrap().filter_map(|r| r.ok()).collect();
                relations_json = rows;
            }

            // succession_relation_member: id + type + chara_id
            if let Ok(mut stmt) = conn.prepare(
                "SELECT id, relation_type, chara_id FROM succession_relation_member ORDER BY id",
            ) {
                let rows: Vec<String> = stmt
                    .query_map([], |row| {
                        Ok(format!(
                            r#"{{"id":{},"type":{},"chara_id":{}}}"#,
                            row.get::<_, i32>(0).unwrap_or(0),
                            row.get::<_, i32>(1).unwrap_or(0),
                            row.get::<_, i32>(2).unwrap_or(0)
                        ))
                    })
                    .unwrap()
                    .filter_map(|r| r.ok())
                    .collect();
                relation_members_json = rows;
            }

            // succession_relation_rank: rank + min + max
            if let Ok(mut stmt) = conn.prepare("SELECT relation_rank, rank_value_min, rank_value_max FROM succession_relation_rank ORDER BY relation_rank") {
                let rows: Vec<String> = stmt.query_map([], |row| {
                    Ok(format!(r#"{{"rank":{},"min":{},"max":{}}}"#,
                        row.get::<_, i32>(0).unwrap_or(0),
                        row.get::<_, i32>(1).unwrap_or(0),
                        row.get::<_, i32>(2).unwrap_or(0)))
                }).unwrap().filter_map(|r| r.ok()).collect();
                relation_ranks_json = rows;
            }

            drop(conn);
        }
    }

    // 4. Read target races for overlap detection
    let mut target_races_json: Vec<String> = Vec::new();
    let tr_arr = call_getter_on_instance(chara_class, chara_obj, "get_TargetRaceArray");
    if !tr_arr.is_null() {
        let trb = tr_arr as *const u8;
        let trl = std::ptr::read_unaligned::<usize>(trb.add(IL2CPP_LIST_COUNT_OFF) as *const usize);
        if trl > 0 && trl < 50 {
            for ti in 0..trl {
                let tp = std::ptr::read_unaligned::<*mut c_void>(
                    trb.add(IL2CPP_LIST_ITEMS_OFF + ti * IL2CPP_LIST_ITEM_SIZE)
                        as *const *mut c_void,
                );
                if tp.is_null() {
                    continue;
                }
                // TargetRace: targetId at offset 0x10, evaluation at 0x14
                let bytes = tp as *const u8;
                let tid = std::ptr::read_unaligned::<i32>(
                    bytes.add(IL2CPP_TARGET_RACE_ID_OFF) as *const i32
                );
                let teval = std::ptr::read_unaligned::<i32>(
                    bytes.add(IL2CPP_TARGET_RACE_EVAL_OFF) as *const i32
                );
                target_races_json
                    .push(format!(r#"{{"target_id":{},"evaluation":{}}}"#, tid, teval));
            }
        }
    }

    // 5. Read route_race from mdb for race name resolution
    let mut race_names_json: Vec<String> = Vec::new();
    if let Some(mdb_path) = find_mdb_path() {
        if let Ok(conn) = Connection::open_with_flags(&mdb_path, OpenFlags::SQLITE_OPEN_READ_ONLY) {
            if let Ok(mut stmt) = conn.prepare(
                "SELECT id, race_id, race_grade FROM single_mode_route_race ORDER BY id LIMIT 200",
            ) {
                let rows: Vec<String> = stmt
                    .query_map([], |row| {
                        Ok(format!(
                            r#"{{"id":{},"race_id":{},"grade":{}}}"#,
                            row.get::<_, i32>(0).unwrap_or(0),
                            row.get::<_, i32>(1).unwrap_or(0),
                            row.get::<_, i32>(2).unwrap_or(0)
                        ))
                    })
                    .unwrap()
                    .filter_map(|r| r.ok())
                    .collect();
                race_names_json = rows;
            }
            drop(conn);
        }
    }

    format!(
        r#"{{"version":"3.22.91","parents":{{"first_chara_id":{},"second_chara_id":{}}},"factor_count":{},"relations":[{}],"relation_members":[{}],"relation_ranks":[{}],"target_races":[{}],"route_races":[{}]}}"#,
        first_chara_id,
        second_chara_id,
        factor_count,
        relations_json.join(","),
        relation_members_json.join(","),
        relation_ranks_json.join(","),
        target_races_json.join(","),
        race_names_json.join(",")
    )
}
```

## `read_win_saddle_analysis` (starts at line 19774)

```rust
unsafe fn read_win_saddle_analysis() -> String {
    if API.is_null() {
        return r#"{"error":"api_null"}"#.to_string();
    }
    let api = &*API;
    let image = match get_image() {
        img if !img.is_null() => img,
        _ => return r#"{"error":"image_null"}"#.to_string(),
    };

    // 1. Get WorkSingleModeData
    let wdm_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkDataManager").as_ptr(),
    );
    if wdm_class.is_null() {
        return r#"{"error":"wdm_class_null"}"#.to_string();
    }
    let get_instance = match api.il2cpp_get_singleton_like_instance_fn {
        Some(f) => f,
        None => return r#"{"error":"no_singleton_fn"}"#.to_string(),
    };
    let wdm = get_instance(wdm_class as *mut c_void);
    if wdm.is_null() {
        return r#"{"error":"wdm_null"}"#.to_string();
    }

    // get_WorkSingleModeData
    let wsm_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkSingleModeData").as_ptr(),
    );
    if wsm_class.is_null() {
        return r#"{"error":"wsm_class_null"}"#.to_string();
    }
    let wsm = call_getter_ref(wdm_class, wdm, "get_WorkSingleModeData");
    if wsm.is_null() {
        return r#"{"error":"wsm_null"}"#.to_string();
    }

    // 2. Read total_race_count and win_count
    let total_races = call_getter_int(wsm_class, wsm, "get_TotalRaceCount");
    let win_count = call_getter_int(wsm_class, wsm, "get_WinCount");

    // 3. Read WinSaddleArray — List<SingleModeWinsSaddle>
    let saddle_arr = call_getter_on_instance(wsm_class, wsm, "get_WinSaddleArray");
    let mut saddle_count = 0i32;
    let mut saddle_entries: Vec<String> = Vec::new();
    if !saddle_arr.is_null() {
        let ab = saddle_arr as *const u8;
        // IL2CPP List<T>: _items (T[] at +0x10), _size (int at +0x18)
        let items_ptr = std::ptr::read_unaligned::<usize>(ab.add(0x10) as *const usize);
        saddle_count = std::ptr::read_unaligned::<i32>(ab.add(0x18) as *const i32);

        // Find SingleModeWinsSaddle class for method calls
        let saddle_class = find_class(
            image,
            to_cstr("Gallop").as_ptr(),
            to_cstr("SingleModeWinsSaddle").as_ptr(),
        );

        for i in 0..saddle_count {
            let elem_ptr = std::ptr::read_unaligned::<usize>(
                (items_ptr + (i as usize) * std::mem::size_of::<usize>()) as *const usize,
            );
            if elem_ptr == 0 {
                continue;
            }

            // Call get_Name on the saddle object
            let name = if !saddle_class.is_null() {
                let n = call_getter_string(saddle_class, elem_ptr as *mut c_void, "get_Name");
                if n.is_null() {
                    String::new()
                } else {
                    read_il2cpp_string(n)
                }
            } else {
                String::new()
            };

            // Call get_Type
            let stype = if !saddle_class.is_null() {
                call_getter_int(saddle_class, elem_ptr as *mut c_void, "get_Type")
            } else {
                -1
            };

            // Call IsRelationBonusWinSaddle (returns bool)
            let is_relation_bonus = if !saddle_class.is_null() {
                let get_method_fn = resolve_il2cpp_symbol("il2cpp_class_get_method_from_name");
                let invoke_fn = resolve_il2cpp_symbol("il2cpp_runtime_invoke");
                if !get_method_fn.is_null() && !invoke_fn.is_null() {
                    type FnGetMethod =
                        unsafe extern "C" fn(*mut c_void, *const c_char, i32) -> *mut c_void;
                    type FnInvoke = unsafe extern "C" fn(
                        *mut c_void,
                        *mut c_void,
                        *mut c_void,
                        *mut c_void,
                    ) -> *mut c_void;
                    let f: FnGetMethod = std::mem::transmute(get_method_fn);
                    let inv: FnInvoke = std::mem::transmute(invoke_fn);
                    let m = f(
                        saddle_class,
                        to_cstr("IsRelationBonusWinSaddle").as_ptr(),
                        0,
                    );
                    if !m.is_null() {
                        let ret = inv(
                            m,
                            elem_ptr as *mut c_void,
                            std::ptr::null_mut(),
                            std::ptr::null_mut(),
                        );
                        ret as i32 != 0
                    } else {
                        false
                    }
                } else {
                    false
                }
            } else {
                false
            };

            // Call GetRelationPoint
            let relation_point = if !saddle_class.is_null() {
                let get_method_fn = resolve_il2cpp_symbol("il2cpp_class_get_method_from_name");
                let invoke_fn = resolve_il2cpp_symbol("il2cpp_runtime_invoke");
                if !get_method_fn.is_null() && !invoke_fn.is_null() {
                    type FnGetMethod =
                        unsafe extern "C" fn(*mut c_void, *const c_char, i32) -> *mut c_void;
                    type FnInvoke = unsafe extern "C" fn(
                        *mut c_void,
                        *mut c_void,
                        *mut c_void,
                        *mut c_void,
                    ) -> *mut c_void;
                    let f: FnGetMethod = std::mem::transmute(get_method_fn);
                    let inv: FnInvoke = std::mem::transmute(invoke_fn);
                    let m = f(saddle_class, to_cstr("GetRelationPoint").as_ptr(), 0);
                    if !m.is_null() {
                        let ret = inv(
                            m,
                            elem_ptr as *mut c_void,
                            std::ptr::null_mut(),
                            std::ptr::null_mut(),
                        );
                        if !ret.is_null() {
                            std::ptr::read_unaligned::<i32>(ret as *const i32)
                        } else {
                            0
                        }
                    } else {
                        0
                    }
                } else {
                    0
                }
            } else {
                0
            };

            saddle_entries.push(format!(
                r#"{{"index":{},"name":"{}","type":{},"is_relation_bonus":{},"relation_point":{}}}"#,
                i,
                json_escape(&name),
                stype,
                is_relation_bonus,
                relation_point,
            ));
        }
    }

    // 4. Read parent candidates' WinSaddleArray via SuccessionCharaData
    // Get WorkSingleModeCharaData → SuccessionTrainedCharaInfo
    let chara_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkSingleModeCharaData").as_ptr(),
    );
    let chara_obj = if !chara_class.is_null() {
        call_getter_ref(wdm_class, wdm, "get_WorkSingleModeCharaData")
    } else {
        std::ptr::null_mut()
    };

    let mut parent_saddles_json: Vec<String> = Vec::new();
    if !chara_obj.is_null() && !chara_class.is_null() {
        let sci_class = find_class(
            image,
            to_cstr("Gallop").as_ptr(),
            to_cstr("SuccessionCharaInfo").as_ptr(),
        );

        for (label, getter_name) in [
            ("parent1", "get_SuccessionTrainedCharaInfoFirst"),
            ("parent2", "get_SuccessionTrainedCharaInfoSecond"),
        ] {
            let sci = call_getter_ref(chara_class, chara_obj, getter_name);
            if sci.is_null() {
                continue;
            }

            let chara_id = if !sci_class.is_null() {
                call_getter_int(sci_class, sci, "get_TrainedCharaId")
            } else {
                0
            };

            // Try to get WinSaddleArray from SuccessionCharaInfo
            let p_saddles = call_getter_on_instance(sci_class, sci, "get_WinSaddleArray");
            let mut p_count = 0i32;
            let mut p_entries: Vec<String> = Vec::new();

            if !p_saddles.is_null() {
                let pb = p_saddles as *const u8;
                let p_items = std::ptr::read_unaligned::<usize>(pb.add(0x10) as *const usize);
                p_count = std::ptr::read_unaligned::<i32>(pb.add(0x18) as *const i32);

                let saddle_class = find_class(
                    image,
                    to_cstr("Gallop").as_ptr(),
                    to_cstr("SingleModeWinsSaddle").as_ptr(),
                );

                for i in 0..p_count.min(30) {
                    let elem_ptr = std::ptr::read_unaligned::<usize>(
                        (p_items + (i as usize) * std::mem::size_of::<usize>()) as *const usize,
                    );
                    if elem_ptr == 0 {
                        continue;
                    }
                    let name = if !saddle_class.is_null() {
                        let n =
                            call_getter_string(saddle_class, elem_ptr as *mut c_void, "get_Name");
                        if n.is_null() {
                            String::new()
                        } else {
                            read_il2cpp_string(n)
                        }
                    } else {
                        String::new()
                    };
                    let stype = if !saddle_class.is_null() {
                        call_getter_int(saddle_class, elem_ptr as *mut c_void, "get_Type")
                    } else {
                        -1
                    };
                    p_entries.push(format!(
                        r#"{{"name":"{}","type":{}}}"#,
                        json_escape(&name),
                        stype,
                    ));
                }
            }

            parent_saddles_json.push(format!(
                r#"{{"label":"{}","chara_id":{},"saddle_count":{},"saddles":[{}]}}"#,
                label,
                chara_id,
                p_count,
                p_entries.join(","),
            ));
        }
    }

    // 5. Cross-reference with MDB for relation_group_id mapping
    let mut mdb_saddle_map_json: Vec<String> = Vec::new();
    let mut relation_groups_json: Vec<String> = Vec::new();

    if let Some(mdb_path) = find_mdb_path() {
        if let Ok(conn) = Connection::open_with_flags(&mdb_path, OpenFlags::SQLITE_OPEN_READ_ONLY) {
            // Map: win_saddle entries from MDB with their relation_group_id
            if let Ok(mut stmt) = conn.prepare(
                "SELECT id, relation_group_id, condition, win_saddle_type, race_instance_id_1, race_instance_id_2 FROM single_mode_wins_saddle WHERE win_saddle_type=3 AND relation_group_id > 0 ORDER BY relation_group_id"
            ) {
                let rows: Vec<String> = stmt.query_map([], |row| {
                    Ok(format!(
                        r#"{{"id":{},"rel_group":{},"cond":{},"type":{},"race1":{},"race2":{}}}"#,
                        row.get::<_, i32>(0).unwrap_or(0),
                        row.get::<_, i32>(1).unwrap_or(0),
                        row.get::<_, i32>(2).unwrap_or(0),
                        row.get::<_, i32>(3).unwrap_or(0),
                        row.get::<_, i32>(4).unwrap_or(0),
                        row.get::<_, i32>(5).unwrap_or(0),
                    ))
                }).unwrap().filter_map(|r| r.ok()).collect();
                mdb_saddle_map_json = rows;
            }

            // succession_relation: check which relation_types give points
            // The G1 win groups are type 1-34 (1pt each)
            if let Ok(mut stmt) = conn.prepare(
                "SELECT relation_type, relation_point FROM succession_relation WHERE relation_type BETWEEN 1 AND 200 ORDER BY relation_type"
            ) {
                let rows: Vec<String> = stmt.query_map([], |row| {
                    Ok(format!(
                        r#"{{"type":{},"point":{}}}"#,
                        row.get::<_, i32>(0).unwrap_or(0),
                        row.get::<_, i32>(1).unwrap_or(0),
                    ))
                }).unwrap().filter_map(|r| r.ok()).collect();
                relation_groups_json = rows;
            }

            // Get race names for G1 race_instance_ids
            // race_instance_id 100301 → race_id → text_data category=32
        }
    }

    // 6. Build output
    format!(
        r#"{{"ok":true,"total_races":{},"win_count":{},"saddle_count":{},"win_saddles":[{}],"parent_saddles":[{}],"mdb_saddle_map":[{}],"relation_groups":[{}]}}"#,
        total_races,
        win_count,
        saddle_count,
        saddle_entries.join(","),
        parent_saddles_json.join(","),
        mdb_saddle_map_json.join(","),
        relation_groups_json.join(","),
    )
}
```

## `parse_query_pairs` (starts at line 22381)

```rust
fn parse_query_pairs(uri: &str) -> Result<Vec<(String, String)>, String> {
    let query = match uri.split_once('?') {
        Some((_, value)) => value.split('#').next().unwrap_or(""),
        None => return Ok(Vec::new()),
    };
    let mut pairs = Vec::new();
    for item in query.split('&') {
        if item.is_empty() { continue; }
        let (raw_key, raw_value) = item.split_once('=').unwrap_or((item, ""));
        pairs.push((percent_decode_component(raw_key)?, percent_decode_component(raw_value)?));
    }
    Ok(pairs)
}
```

## `query_pair` (starts at line 22395)

```rust
fn query_pair(pairs: &[(String, String)], name: &str) -> String {
    pairs.iter().find(|(key, _)| key == name).map(|(_, value)| value.clone()).unwrap_or_default()
}
```

## `build_method_index` (starts at line 22470)

```rust
unsafe fn build_method_index() -> Result<Vec<MethodIndexEntry>, String> {
    let image = get_image();
    if image.is_null() { return Err("image_null".to_string()); }
    let symbols = [
        "il2cpp_image_get_class_count", "il2cpp_image_get_class", "il2cpp_class_get_methods",
        "il2cpp_method_get_name", "il2cpp_method_get_param_count", "il2cpp_method_get_param",
        "il2cpp_method_get_param_name", "il2cpp_method_get_return_type", "il2cpp_type_get_name",
        "il2cpp_method_get_flags", "il2cpp_method_get_class",
    ];
    let resolved: Vec<*mut c_void> = symbols.iter().map(|name| resolve_il2cpp_symbol(name)).collect();
    if let Some(index) = resolved.iter().position(|value| value.is_null()) {
        return Err(format!("missing_symbol:{}", symbols[index]));
    }
    let get_class_count: FnImageGetClassCount = std::mem::transmute(resolved[0]);
    let get_class: FnImageGetClass = std::mem::transmute(resolved[1]);
    let get_methods: FnClassGetMethods = std::mem::transmute(resolved[2]);
    let get_method_name: FnMethodGetName = std::mem::transmute(resolved[3]);
    let get_param_count: unsafe extern "C" fn(*const c_void) -> u32 = std::mem::transmute(resolved[4]);
    let get_param: unsafe extern "C" fn(*const c_void, u32) -> *const c_void = std::mem::transmute(resolved[5]);
    let get_param_name: unsafe extern "C" fn(*const c_void, u32) -> *const c_char = std::mem::transmute(resolved[6]);
    let get_return_type: unsafe extern "C" fn(*const c_void) -> *const c_void = std::mem::transmute(resolved[7]);
    let get_type_name: unsafe extern "C" fn(*const c_void) -> *const c_char = std::mem::transmute(resolved[8]);
    let get_flags: unsafe extern "C" fn(*const c_void, *mut u32) -> u32 = std::mem::transmute(resolved[9]);
    let get_method_class: unsafe extern "C" fn(*const c_void) -> *mut c_void = std::mem::transmute(resolved[10]);
    let mut entries = Vec::new();
    let class_count = get_class_count(image);
    for class_index in 0..class_count {
        let class = get_class(image, class_index);
        if class.is_null() { continue; }
        let mut iterator = ptr::null_mut();
        loop {
            let method_info = get_methods(class, &mut iterator);
            if method_info.is_null() { break; }
            let declaring_class = get_method_class(method_info);
            let declaring_type = class_full_declaring_name(declaring_class);
            let namespace = declaring_type.split_once('.').map(|(value, _)| value.to_string()).unwrap_or_default();
            let parameter_count = get_param_count(method_info);
            let mut parameter_names = Vec::with_capacity(parameter_count as usize);
            let mut parameter_types = Vec::with_capacity(parameter_count as usize);
            for parameter_index in 0..parameter_count {
                let parameter_type = get_param(method_info, parameter_index);
                parameter_types.push(if parameter_type.is_null() { "unresolved".to_string() } else { il2cpp_c_string(get_type_name(parameter_type)) });
                let parameter_name = il2cpp_c_string(get_param_name(method_info, parameter_index));
                parameter_names.push(if parameter_name.is_empty() { None } else { Some(parameter_name) });
            }
            let return_type_pointer = get_return_type(method_info);
            let return_type = if return_type_pointer.is_null() { "unresolved".to_string() } else { il2cpp_c_string(get_type_name(return_type_pointer)) };
            let mut iflags = 0u32;
            let flags = get_flags(method_info, &mut iflags);
            let method_pointer = if is_readable_range(method_info as usize, std::mem::size_of::<usize>()) {
                std::ptr::read_unaligned::<usize>(method_info as *const usize)
            } else { 0 };
            entries.push(MethodIndexEntry {
                method_info: method_info as usize,
                method_pointer,
                namespace,
                declaring_type,
                method_name: il2cpp_c_string(get_method_name(method_info)),
                return_type,
                parameter_names,
                parameter_types,
                flags,
            });
        }
    }
    entries.sort_by(|left, right| left.method_pointer.cmp(&right.method_pointer).then(left.method_info.cmp(&right.method_info)));
    Ok(entries)
}
```

## `ensure_method_index` (starts at line 22539)

```rust
unsafe fn ensure_method_index() -> Result<(), String> {
    {
        let mut state = METHOD_INDEX.lock().map_err(|_| "method_index_lock_poisoned".to_string())?;
        match state.status {
            "ready" => return Ok(()),
            "building" => return Err("method_index_building".to_string()),
            "failed" => return Err(state.error.clone()),
            _ => state.status = "building",
        }
    }
    let result = build_method_index();
    let mut state = METHOD_INDEX.lock().map_err(|_| "method_index_lock_poisoned".to_string())?;
    match result {
        Ok(entries) => {
            let class_count = {
                let image = get_image();
                let pointer = resolve_il2cpp_symbol("il2cpp_image_get_class_count");
                if image.is_null() || pointer.is_null() { 0 } else {
                    let function: FnImageGetClassCount = std::mem::transmute(pointer);
                    function(image)
                }
            };
            let null_count = entries.iter().filter(|entry| entry.method_pointer == 0).count();
            let mut duplicate_count = 0usize;
            let mut previous = 0usize;
            for entry in entries.iter().filter(|entry| entry.method_pointer != 0) {
                if entry.method_pointer == previous { duplicate_count += 1; }
                previous = entry.method_pointer;
            }
            state.image_class_count = class_count;
            state.indexed_class_count = class_count;
            state.indexed_method_count = entries.len();
            state.null_method_pointer_count = null_count;
            state.duplicate_method_pointer_count = duplicate_count;
            state.entries = entries;
            state.error.clear();
            state.status = "ready";
            Ok(())
        }
        Err(error) => {
            state.status = "failed";
            state.error = error.clone();
            Err(error)
        }
    }
}
```

## `il2cpp_method_by_addr` (starts at line 22599)

```rust
unsafe fn il2cpp_method_by_addr(uri: &str) -> String {
    let pairs = match parse_query_pairs(uri) { Ok(value) => value, Err(error) => return format!(r#"{{"ok":false,"error":"{}"}}"#, json_escape(&error)) };
    let raw = query_pair(&pairs, "addr");
    let address = match parse_address(&raw) { Some(value) if value != 0 => value, _ => return r#"{"ok":false,"error":"invalid_or_missing_addr"}"#.to_string() };
    if let Err(error) = ensure_method_index() { return format!(r#"{{"ok":false,"error":"{}"}}"#, json_escape(&error)); }
    let state = match METHOD_INDEX.lock() { Ok(value) => value, Err(_) => return r#"{"ok":false,"error":"method_index_lock_poisoned"}"#.to_string() };
    let method_info_matches: Vec<&MethodIndexEntry> = state.entries.iter().filter(|entry| entry.method_info == address).collect();
    let exact_pointer_matches: Vec<&MethodIndexEntry> = state.entries.iter().filter(|entry| entry.method_pointer == address && entry.method_pointer != 0).collect();
    let (kind, matches): (&str, Vec<&MethodIndexEntry>) = if !method_info_matches.is_empty() {
        (if method_info_matches.len() == 1 { "exact_method_info" } else { "ambiguous" }, method_info_matches)
    } else if !exact_pointer_matches.is_empty() {
        (if exact_pointer_matches.len() == 1 { "exact_method_pointer" } else { "ambiguous" }, exact_pointer_matches)
    } else {
        let mut distinct: Vec<usize> = state.entries.iter().map(|entry| entry.method_pointer).filter(|value| *value != 0).collect();
        distinct.dedup();
        match distinct.binary_search(&address) {
            Ok(_) => ("none", Vec::new()),
            Err(position) if position > 0 && position < distinct.len() => {
                let start = distinct[position - 1];
                let candidates: Vec<&MethodIndexEntry> = state.entries.iter().filter(|entry| entry.method_pointer == start).collect();
                (if candidates.len() == 1 { "upper_bound_candidate" } else { "ambiguous" }, candidates)
            }
            _ => ("none", Vec::new()),
        }
    };
    let items = matches.iter().map(|entry| {
        let upper = state.entries.iter().map(|candidate| candidate.method_pointer).filter(|pointer| *pointer > entry.method_pointer).min();
        method_entry_json(entry, upper)
    }).collect::<Vec<_>>().join(",");
    format!(r#"{{"ok":true,"query":"0x{:x}","status":"{}","ambiguous":{},"matches":[{}],"index":{{"status":"{}","classes":{},"methods":{},"null_method_pointers":{},"duplicate_method_pointers":{}}}}}"#,
        address, kind, kind == "ambiguous", items, state.status, state.indexed_class_count,
        state.indexed_method_count, state.null_method_pointer_count, state.duplicate_method_pointer_count)
}
```

## `il2cpp_method_detail` (starts at line 22633)

```rust
unsafe fn il2cpp_method_detail(uri: &str) -> String {
    let pairs = match parse_query_pairs(uri) { Ok(value) => value, Err(error) => return format!(r#"{{"ok":false,"error":"{}"}}"#, json_escape(&error)) };
    let namespace = query_pair(&pairs, "namespace");
    let declaring_type = query_pair(&pairs, "declaring_type");
    let method = query_pair(&pairs, "method");
    let parameter_text = query_pair(&pairs, "parameter_types");
    if declaring_type.is_empty() || method.is_empty() { return r#"{"ok":false,"error":"missing_declaring_type_or_method"}"#.to_string(); }
    let parameter_types: Vec<String> = if parameter_text.is_empty() { Vec::new() } else { parameter_text.split(',').map(|value| value.trim().to_string()).collect() };
    if let Err(error) = ensure_method_index() { return format!(r#"{{"ok":false,"error":"{}"}}"#, json_escape(&error)); }
    let state = match METHOD_INDEX.lock() { Ok(value) => value, Err(_) => return r#"{"ok":false,"error":"method_index_lock_poisoned"}"#.to_string() };
    let matches: Vec<&MethodIndexEntry> = state.entries.iter().filter(|entry| {
        (namespace.is_empty() || entry.namespace == namespace) && entry.declaring_type == declaring_type &&
        entry.method_name == method && entry.parameter_types == parameter_types
    }).collect();
    let status = if matches.is_empty() { "none" } else if matches.len() == 1 { "exact" } else { "ambiguous" };
    let items = matches.iter().map(|entry| {
        let upper = state.entries.iter().map(|candidate| candidate.method_pointer).filter(|pointer| *pointer > entry.method_pointer).min();
        method_entry_json(entry, upper)
    }).collect::<Vec<_>>().join(",");
    format!(r#"{{"ok":true,"status":"{}","ambiguous":{},"matches":[{}]}}"#, status, status == "ambiguous", items)
}
```

## `il2cpp_nested_types` (starts at line 22655)

```rust
unsafe fn il2cpp_nested_types(uri: &str) -> String {
    let pairs = match parse_query_pairs(uri) { Ok(value) => value, Err(error) => return format!(r#"{{"ok":false,"error":"{}"}}"#, json_escape(&error)) };
    let requested = query_pair(&pairs, "type");
    if requested.is_empty() { return r#"{"ok":false,"error":"missing_type"}"#.to_string(); }
    let class = find_class_by_full_declaring_name(&requested);
    if class.is_null() { return format!(r#"{{"ok":false,"error":"class_not_found_or_ambiguous","type":"{}"}}"#, json_escape(&requested)); }
    let nested_ptr = resolve_il2cpp_symbol("il2cpp_class_get_nested_types");
    if nested_ptr.is_null() { return r#"{"ok":false,"error":"il2cpp_class_get_nested_types_unavailable"}"#.to_string(); }
    let get_nested: unsafe extern "C" fn(*mut c_void, *mut *mut c_void) -> *mut c_void = std::mem::transmute(nested_ptr);
    let mut iterator = ptr::null_mut();
    let mut items = Vec::new();
    loop {
        let nested = get_nested(class, &mut iterator);
        if nested.is_null() { break; }
        items.push(format!(r#"{{"type":"{}","class_pointer":"0x{:x}"}}"#, json_escape(&class_full_declaring_name(nested)), nested as usize));
    }
    format!(r#"{{"ok":true,"requested":"{}","direct_only":true,"count":{},"nested_types":[{}]}}"#, json_escape(&requested), items.len(), items.join(","))
}
```

## `il2cpp_enum_values_capability` (starts at line 22674)

```rust
unsafe fn il2cpp_enum_values_capability(uri: &str) -> String {
    let pairs = match parse_query_pairs(uri) { Ok(value) => value, Err(error) => return format!(r#"{{"ok":false,"error":"{}"}}"#, json_escape(&error)) };
    let requested = query_pair(&pairs, "type");
    let required = ["il2cpp_class_get_fields", "il2cpp_field_get_flags", "il2cpp_field_static_get_value"];
    let available: Vec<bool> = required.iter().map(|name| !resolve_il2cpp_symbol(name).is_null()).collect();
    format!(r#"{{"ok":true,"requested":"{}","value_status":"unresolved","integer_values":null,"declaration_order_inference":false,"runtime_api":{{"il2cpp_class_get_fields":{},"il2cpp_field_get_flags":{},"il2cpp_field_static_get_value":{}}}}}"#,
        json_escape(&requested), available[0], available[1], available[2])
}
```

## `storage_set_error` (starts at line 22688)

```rust
fn storage_set_error(error: &str) {
    if let Ok(mut value) = STORAGE_LAST_ERROR.lock() {
        *value = Some(error.to_string());
    }
}
```

## `observation_storage_root` (starts at line 22694)

```rust
fn observation_storage_root() -> std::path::PathBuf {
    if let Ok(command_line) = std::fs::read("/proc/self/cmdline") {
        let package_bytes = command_line.split(|byte| *byte == 0).next().unwrap_or(&[]);
        if let Ok(package_name) = std::str::from_utf8(package_bytes) {
            if !package_name.is_empty() {
                return std::path::PathBuf::from("/data/user/0")
                    .join(package_name)
                    .join("files")
                    .join("hlpatch-observations");
            }
        }
    }
    std::path::PathBuf::from("/data/user/0/jp.co.cygames.umamusume/files/hlpatch-observations")
}
```

## `observation_storage_db_path` (starts at line 22709)

```rust
fn observation_storage_db_path() -> std::path::PathBuf {
    observation_storage_root().join("index.sqlite")
}
```

## `storage_status_endpoint` (starts at line 22801)

```rust
fn storage_status_endpoint() -> String {
    let root = observation_storage_root();
    let db_path = observation_storage_db_path();
    let session = ensure_observation_session();
    if let Err(error) = session.as_ref() { storage_set_error(error); }
    let current_session = session.ok();
    let writable_probe_path = root.join(".write_probe");
    let writable = std::fs::write(&writable_probe_path, b"hlpatch-storage-probe")
        .and_then(|_| std::fs::remove_file(&writable_probe_path)).is_ok();
    let error = STORAGE_LAST_ERROR.lock().ok().and_then(|value| value.clone());
    let session_json = current_session.as_ref().map(|value| format!("\"{}\"", json_escape(value))).unwrap_or_else(|| "null".to_string());
    let error_json = error.map(|value| format!("\"{}\"", json_escape(&value))).unwrap_or_else(|| "null".to_string());
    format!(
        r#"{{"ok":{},"schema_version":1,"root_path":"{}","index_path":"{}","writable":{},"current_session_id":{},"last_flush_ms":{},"last_error":{},"storage_format":{{"index":"sqlite","timeline":"ndjson","payloads":"raw_files"}}}}"#,
        writable && current_session.is_some(), json_escape(&root.to_string_lossy()),
        json_escape(&db_path.to_string_lossy()), writable, session_json,
        STORAGE_LAST_FLUSH_MS.load(Ordering::Relaxed), error_json
    )
}
```

## `storage_sessions_endpoint` (starts at line 22821)

```rust
fn storage_sessions_endpoint() -> String {
    if let Err(error) = ensure_observation_session() {
        storage_set_error(&error);
        return format!(r#"{{"ok":false,"error":"{}","sessions":[]}}"#, json_escape(&error));
    }
    let connection = match open_observation_storage() {
        Ok(value) => value,
        Err(error) => return format!(r#"{{"ok":false,"error":"{}","sessions":[]}}"#, json_escape(&error)),
    };
    let mut statement = match connection.prepare(
        "SELECT session_id, process_id, plugin_version, started_at_ms, last_flush_ms,
                state, recovered_after_restart, root_path
         FROM observation_sessions ORDER BY started_at_ms, session_id"
    ) {
        Ok(value) => value,
        Err(error) => return format!(r#"{{"ok":false,"error":"prepare_sessions:{}","sessions":[]}}"#, json_escape(&error.to_string())),
    };
    let rows = match statement.query_map([], |row| {
        Ok(format!(
            r#"{{"session_id":"{}","process_id":{},"plugin_version":"{}","started_at_ms":{},"last_flush_ms":{},"state":"{}","recovered_after_restart":{},"root_path":"{}"}}"#,
            json_escape(&row.get::<_, String>(0)?), row.get::<_, i64>(1)?,
            json_escape(&row.get::<_, String>(2)?), row.get::<_, i64>(3)?, row.get::<_, i64>(4)?,
            json_escape(&row.get::<_, String>(5)?), row.get::<_, i64>(6)? != 0,
            json_escape(&row.get::<_, String>(7)?)
        ))
    }) {
        Ok(value) => value,
        Err(error) => return format!(r#"{{"ok":false,"error":"query_sessions:{}","sessions":[]}}"#, json_escape(&error.to_string())),
    };
    let sessions: Vec<String> = rows.filter_map(Result::ok).collect();
    format!(r#"{{"ok":true,"count":{},"sessions":[{}]}}"#, sessions.len(), sessions.join(","))
}
```

## `storage_session_endpoint` (starts at line 22854)

```rust
fn storage_session_endpoint(uri: &str) -> String {
    let pairs = match parse_query_pairs(uri) {
        Ok(value) => value,
        Err(error) => return format!(r#"{{"ok":false,"error":"{}"}}"#, json_escape(&error)),
    };
    let session_id = query_pair(&pairs, "id");
    if session_id.is_empty() { return r#"{"ok":false,"error":"missing_id"}"#.to_string(); }
    let connection = match open_observation_storage() {
        Ok(value) => value,
        Err(error) => return format!(r#"{{"ok":false,"error":"{}"}}"#, json_escape(&error)),
    };
    let result = connection.query_row(
        "SELECT process_id, plugin_version, started_at_ms, last_flush_ms,
                state, recovered_after_restart, root_path
         FROM observation_sessions WHERE session_id=?1",
        rusqlite::params![session_id],
        |row| Ok(format!(
            r#"{{"ok":true,"session":{{"session_id":"{}","process_id":{},"plugin_version":"{}","started_at_ms":{},"last_flush_ms":{},"state":"{}","recovered_after_restart":{},"root_path":"{}"}}}}"#,
            json_escape(&session_id), row.get::<_, i64>(0)?, json_escape(&row.get::<_, String>(1)?),
            row.get::<_, i64>(2)?, row.get::<_, i64>(3)?, json_escape(&row.get::<_, String>(4)?),
            row.get::<_, i64>(5)? != 0, json_escape(&row.get::<_, String>(6)?)
        )),
    );
    match result {
        Ok(value) => value,
        Err(rusqlite::Error::QueryReturnedNoRows) => r#"{"ok":true,"session":null,"status":"none"}"#.to_string(),
        Err(error) => format!(r#"{{"ok":false,"error":"query_session:{}"}}"#, json_escape(&error.to_string())),
    }
}
```

## `storage_flush_endpoint` (starts at line 22884)

```rust
fn storage_flush_endpoint() -> String {
    let session_id = match ensure_observation_session() {
        Ok(value) => value,
        Err(error) => return format!(r#"{{"ok":false,"error":"{}"}}"#, json_escape(&error)),
    };
    let connection = match open_observation_storage() {
        Ok(value) => value,
        Err(error) => return format!(r#"{{"ok":false,"error":"{}"}}"#, json_escape(&error)),
    };
    let now = sniff_timestamp_ms();
    if let Err(error) = connection.execute(
        "UPDATE observation_sessions SET last_flush_ms=?1 WHERE session_id=?2",
        rusqlite::params![now as i64, session_id],
    ) {
        return format!(r#"{{"ok":false,"error":"update_flush:{}"}}"#, json_escape(&error.to_string()));
    }
    if let Err(error) = connection.execute_batch("PRAGMA wal_checkpoint(FULL);") {
        return format!(r#"{{"ok":false,"error":"checkpoint:{}"}}"#, json_escape(&error.to_string()));
    }
    STORAGE_LAST_FLUSH_MS.store(now, Ordering::Relaxed);
    format!(r#"{{"ok":true,"session_id":"{}","last_flush_ms":{},"checkpoint":"full"}}"#, json_escape(&session_id), now)
}
```

## `storage_recover_endpoint` (starts at line 22907)

```rust
fn storage_recover_endpoint() -> String {
    let connection = match open_observation_storage() {
        Ok(value) => value,
        Err(error) => return format!(r#"{{"ok":false,"error":"{}"}}"#, json_escape(&error)),
    };
    let process_id = std::process::id() as i64;
    let recovered = match connection.execute(
        "UPDATE observation_sessions
         SET state='interrupted', recovered_after_restart=1
         WHERE state='open' AND process_id<>?1",
        rusqlite::params![process_id],
    ) {
        Ok(value) => value,
        Err(error) => return format!(r#"{{"ok":false,"error":"recover:{}"}}"#, json_escape(&error.to_string())),
    };
    match ensure_observation_session() {
        Ok(session_id) => format!(r#"{{"ok":true,"recovered_session_count":{},"current_session_id":"{}"}}"#, recovered, json_escape(&session_id)),
        Err(error) => format!(r#"{{"ok":false,"error":"{}","recovered_session_count":{}}}"#, json_escape(&error), recovered),
    }
}
```

## `inherit_pair_compat_endpoint` (starts at line 22929)

```rust
fn inherit_pair_compat_endpoint(uri: &str) -> String {
    let pairs = match parse_query_pairs(uri) {
        Ok(value) => value,
        Err(error) => return format!(r#"{{"ok":false,"error":"{}"}}"#, json_escape(&error)),
    };
    let chara_id_a = match query_pair(&pairs, "chara_id_a").parse::<i32>() {
        Ok(value) if value > 0 => value,
        _ => return r#"{"ok":false,"error":"invalid_or_missing_chara_id_a"}"#.to_string(),
    };
    let chara_id_b = match query_pair(&pairs, "chara_id_b").parse::<i32>() {
        Ok(value) if value > 0 => value,
        _ => return r#"{"ok":false,"error":"invalid_or_missing_chara_id_b"}"#.to_string(),
    };
    let mdb_path = match find_mdb_path() {
        Some(value) => value,
        None => return r#"{"ok":false,"error":"mdb_not_found"}"#.to_string(),
    };
    let connection = match Connection::open_with_flags(&mdb_path, OpenFlags::SQLITE_OPEN_READ_ONLY) {
        Ok(value) => value,
        Err(error) => return format!(r#"{{"ok":false,"error":"mdb_open_failed","detail":"{}"}}"#, json_escape(&error.to_string())),
    };
    let mut statement = match connection.prepare(
        "SELECT DISTINCT r.relation_type, r.relation_point
         FROM succession_relation r
         INNER JOIN succession_relation_member a
             ON a.relation_type = r.relation_type AND a.chara_id = ?1
         INNER JOIN succession_relation_member b
             ON b.relation_type = r.relation_type AND b.chara_id = ?2
         ORDER BY r.relation_type, r.relation_point"
    ) {
        Ok(value) => value,
        Err(error) => return format!(r#"{{"ok":false,"error":"pair_query_prepare_failed","detail":"{}"}}"#, json_escape(&error.to_string())),
    };
    let mapped = match statement.query_map(rusqlite::params![chara_id_a, chara_id_b], |row| {
        Ok((row.get::<_, i32>(0)?, row.get::<_, i32>(1)?))
    }) {
        Ok(value) => value,
        Err(error) => return format!(r#"{{"ok":false,"error":"pair_query_failed","detail":"{}"}}"#, json_escape(&error.to_string())),
    };
    let mut relation_items = Vec::new();
    let mut base_compatibility = 0i64;
    for row in mapped {
        let (relation_type, relation_point) = match row {
            Ok(value) => value,
            Err(error) => return format!(r#"{{"ok":false,"error":"pair_row_decode_failed","detail":"{}"}}"#, json_escape(&error.to_string())),
        };
        base_compatibility += i64::from(relation_point);
        relation_items.push(format!(
            r#"{{"relation_type":{},"relation_point":{},"chara_id_a_member":true,"chara_id_b_member":true}}"#,
            relation_type, relation_point
        ));
    }
    format!(
        r#"{{"ok":true,"source":"current_mdb","calculation":"sum_shared_succession_relation_points","chara_id_a":{},"chara_id_b":{},"shared_relation_count":{},"base_compatibility":{},"shared_relations":[{}],"race_bonus":null,"specific_trained_chara_adjustments":null,"runtime_consumer_result":null,"scope":"character_pair_base_only"}}"#,
        chara_id_a, chara_id_b, relation_items.len(), base_compatibility, relation_items.join(",")
    )
}
```

## `inherit_selected_parent_runtime_endpoint` (starts at line 22988)

```rust
unsafe fn inherit_selected_parent_runtime_endpoint() -> String {
    if API.is_null() {
        return r#"{"ok":false,"error":"api_null"}"#.to_string();
    }
    let image = get_image();
    if image.is_null() {
        return r#"{"ok":false,"error":"image_null"}"#.to_string();
    }
    let wdm_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkDataManager").as_ptr(),
    );
    if wdm_class.is_null() {
        return r#"{"ok":false,"error":"work_data_manager_class_not_found"}"#.to_string();
    }
    let wdm = get_singleton(wdm_class);
    if wdm.is_null() {
        return r#"{"ok":false,"error":"work_data_manager_instance_not_found"}"#.to_string();
    }
    let single_mode_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkSingleModeData").as_ptr(),
    );
    if single_mode_class.is_null() {
        return r#"{"ok":false,"error":"work_single_mode_data_class_not_found"}"#.to_string();
    }
    let single_mode = call_getter_ref(wdm_class, wdm, "get_SingleMode");
    if single_mode.is_null() {
        return r#"{"ok":false,"error":"single_mode_instance_not_found"}"#.to_string();
    }
    let chara_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkSingleModeCharaData").as_ptr(),
    );
    if chara_class.is_null() {
        return r#"{"ok":false,"error":"work_single_mode_chara_data_class_not_found"}"#.to_string();
    }
    let chara = call_getter_ref(single_mode_class, single_mode, "get_Character");
    if chara.is_null() {
        return r#"{"ok":false,"error":"single_mode_character_instance_not_found"}"#.to_string();
    }
    let succession_info_class = find_class_by_short_name(image, "SuccessionCharaInfo");
    if succession_info_class.is_null() {
        return r#"{"ok":false,"error":"succession_chara_info_class_not_found"}"#.to_string();
    }

    let target_card_id = call_getter_int(chara_class, chara, "get_CardId");
    let target_chara_id = call_getter_int(chara_class, chara, "get_CharaId");
    let first = call_getter_ref(
        chara_class,
        chara,
        "get_SuccessionTrainedCharaInfoFirst",
    );
    let second = call_getter_ref(
        chara_class,
        chara,
        "get_SuccessionTrainedCharaInfoSecond",
    );

    let render_slot = |slot: &str, info: *mut c_void| -> String {
        if info.is_null() {
            return format!(
                r#"{{"slot":"{}","selected":false,"trained_chara_id":null,"trained_chara_record":null}}"#,
                slot
            );
        }
        let trained_chara_id = call_getter_obscured_int(
            succession_info_class,
            info,
            "get_TrainedCharaId",
        );
        format!(
            r#"{{"slot":"{}","selected":true,"trained_chara_id":{},"trained_chara_record":null}}"#,
            slot, trained_chara_id
        )
    };

    format!(
        r#"{{"ok":true,"source":"current_work_single_mode_character","scope":"selected_parent_ids_only","target":{{"card_id":{},"chara_id":{}}},"parents":[{},{}],"trained_chara_record_resolution":null,"ancestor_tree":null,"pair_compatibility":null,"race_bonus":null,"runtime_consumer_result":null,"id_semantics":"trained_chara_id","getter_decode":"existing_runtime_invoke_int_path","runtime_validation":"pending_device_execution"}}"#,
        target_card_id,
        target_chara_id,
        render_slot("first", first),
        render_slot("second", second),
    )
}
```
