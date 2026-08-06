# Routes and advertised endpoints

source_commit: `a340a147acf13672b2fbc64925bfa321d08091fd`

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
