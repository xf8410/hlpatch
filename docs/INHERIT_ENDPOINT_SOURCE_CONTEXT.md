# Focused inheritance source context

hit_count=25

## lines 7435..7635

```rust
00007435:             "/api/sniff/status",
00007436:             "/api/sniff/diag",
00007437:             "/api/sniff/toggle",
00007438:             "/api/sniff/clear",
00007439:             "/api/md5log",
00007440:             "/api/md5log/clear",
00007441:             "/api/md5log/install",
00007442:             "/api/event/choices",
00007443:             "/api/event/observations",
00007444:             "/api/event/observations/clear",
00007445:             "/api/event/clear",
00007446:             "/action/latest",
00007447:             "/seed/history",
00007448:             "/seed/stats",
00007449:             "/log",
00007450:             "/carddb",
00007451:             "/skilldata",
00007452:             "/debug/table",
00007453:             "/debug/push_table",
00007454:             "/debug/download_table",
00007455:             "/debug/mdb_all_tables",
00007456:             "/debug/mdb_schema_dump",
00007457:         ];
00007458:         const BOOT_SAFE_PREFIX: &[&str] = &[
00007459:             "/mdb",
00007460:             "/debug/resource_",
00007461:             "/debug/private_file",
00007462:             "/debug/mem_scan_sqlite",
00007463:             "/debug/mem_scan_zdict",
00007464:             "/debug/mem_scan_hex",
00007465:             "/debug/file_scan_hex",
00007466:             "/debug/maps_list",
00007467:             "/debug/file_dl",
00007468:             "/debug/file_range_hex",
00007469:             "/il2cpp/read_string",
00007470:             "/il2cpp/read_mem",
00007471:         ];
00007472:         let safe = BOOT_SAFE_EXACT.iter().any(|p| path == *p)
00007473:             || BOOT_SAFE_PREFIX.iter().any(|p| path.starts_with(p));
00007474:         if !safe {
00007475:             let b = r#"{"status":"booting","game_initialized":false}"#;
00007476:             let resp = format!(
00007477:                 "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
00007478:                 b.len(), b
00007479:             );
00007480:             let _ = stream.write_all(resp.as_bytes());
00007481:             return;
00007482:         }
00007483:     }
00007484: 
00007485:     // ★ 白名单下载开关：名单内端点追加 ?dl=1 即以附件形式返回（解决手机复制长度上限）
00007486:     //    ?dl=1&name=xxx 可自定义文件名（仅保留字母数字和下划线/连字符）
00007487:     //    大文件仍走各专用流式 _dl 端点，避免此路径内存翻倍
00007488:     const DL_ALLOWED: &[&str] = &[
00007489:         "/summary",
00007490:         "/scenario",
00007491:         "/data",
00007492:         "/ramen",
00007493:         "/debug/ramen_transition",
00007494:         "/api/sniff",
00007495:         "/api/sniff/metadata",
00007496:         "/api/sniff/diag",
00007497:         "/api/event/choices",
00007498:         "/api/event/observations",
00007499:         "/debug/event_reward_targets",
00007500:         "/debug/resource_meta_schema",
00007501:         "/debug/resource_meta_probe",
00007502:         "/debug/resource_crypto_symbols",
00007503:         "/debug/all",
00007504:         "/debug/params",
00007505:         "/debug/cmdinfo",
00007506:         "/debug/breeders",
00007507:         "/debug/training_partners",
00007508:         "/debug/rameninfo",
00007509:         "/debug/laststep",
00007510:         "/debug/storydata",
00007511:         "/debug/ramenfields",
00007512:         "/debug/gauge",
00007513:         "/debug/gauge2",
00007514:         "/debug/ramengains",
00007515:         "/debug/paramsincdec",
00007516:         "/debug/training_seed",
00007517:         "/debug/unique_skills",
00007518:         "/debug/hint_gain",
00007519:         "/debug/sc_effect",
00007520:         "/debug/unique_detail",
00007521:         "/classes",
00007522:     ];
00007523:     let dl_flag = parse_query(&full_uri, "dl");
00007524:     let dl_name = parse_query(&full_uri, "name");
00007525:     let dl_enabled = !dl_flag.is_empty() && dl_flag != "0" && DL_ALLOWED.iter().any(|p| path == *p);
00007526: 
00007527:     let body = if path == "/debug/global_metadata_probe" {
00007528:         safe_mem_scan(req, true)
00007529:     } else if path == "/debug/mem_scan_hex" {
00007530:         safe_mem_scan(req, false)
00007531:     } else if path == "/debug/mem_maps" {
00007532:         safe_maps_summary()
00007533:     } else if path == "/" || path == "/health" {
00007534:         format!(
00007535:             r#"{{"status":"ok","version":"{}","endpoints":["/summary","/data","/scenario","/debug/rameninfo","/debug/laststep","/event/recommend","/inherit/compat","/saddle-analysis","/log/turn","/debug/params","/debug/breeders","/debug/cmdinfo","/debug/training_partners","/debug/crashlog","/debug/upload","/debug/dumpclass","/debug/storydata","/debug/ramenfields","/debug/gauge","/debug/gauge2","/debug/ramengains","/debug/paramsincdec","/debug/training_seed","/debug/training_log","/debug/training_log_dl","/update","/update/status","/debug/all","/debug/unique_skills","/debug/mdb_all_tables","/debug/mdb_schema_dump","/debug/hint_gain","/debug/sc_effect","/debug/unique_detail","/debug/table","/debug/push_table","/debug/download_table","/mdb","/carddb","/skilldata","/hall","/saddles","/saddles-dl","/log","/status","/health","/mdb/schema","/mdb/search","/mdb/raw","/mdb/dl_batch","/il2cpp/dump","/il2cpp/call","/il2cpp/tree","/il2cpp/field","/il2cpp/classes","/il2cpp/static","/il2cpp/methods","/il2cpp/disassemble","/il2cpp/disassemble_dl","/il2cpp/disassemble_addr","/il2cpp/disassemble_addr_dl","/il2cpp/dump_all_methods","/il2cpp/dump_all_methods_dl","/il2cpp/search_float","/il2cpp/search_float_dl","/il2cpp/search_int","/il2cpp/search_int_dl","/il2cpp/search_methods","/il2cpp/search_methods_dl","/il2cpp/read_mem","/il2cpp/read_mem_dl","/training/result","/api/sniff","/api/sniff/metadata","/api/sniff/status","/api/sniff/toggle","/api/sniff/clear","/api/sniff/diag","/api/event/choices","/api/event/clear","/debug/hooklog","/debug/hookdiag","/debug/resource_meta_key","/debug/resource_db_keys","/debug/resource_reads","/debug/mem_scan_sqlite","/debug/meta_dump","/action/latest","/seed/history","/seed/stats","/debug/ramen_planner_state","/debug/ramen_participants","/debug/ramen_transition","/debug/ramen_dataset_path","/debug/ramen_formula_targets","/debug/event_reward_targets", "/debug/resource_storage","/debug/resource_meta_schema","/debug/resource_meta_probe", "/debug/resource_crypto_symbols","/debug/resource_meta_dl","/debug/resource_file_dl","/debug/private_file_inventory","/debug/private_file_dl"]}}"#,
00007536:             PLUGIN_VERSION
00007537:         )
00007538:     } else if path == "/scan" {
00007539:         unsafe { scan_il2cpp_classes() }
00007540:     } else if path == "/data" {
00007541:         let result = unsafe { read_training_data() };
00007542:         unsafe {
00007543:             log_snapshot("data", &result);
00007544:         }
00007545:         result
00007546:     } else if path == "/status" {
00007547:         format!(
00007548:             r#"{{"game_initialized":{},"http_running":{}}}"#,
00007549:             GAME_INITIALIZED.load(Ordering::Relaxed),
00007550:             HTTP_RUNNING.load(Ordering::Relaxed)
00007551:         )
00007552:     } else if path == "/singletons" {
00007553:         unsafe { find_all_singletons() }
00007554:     } else if path.starts_with("/find_method") {
00007555:         let method_name = if path == "/find_method" || path == "/find_method/" {
00007556:             "get_SingleMode"
00007557:         } else {
00007558:             path.strip_prefix("/find_method/")
00007559:                 .unwrap_or("get_SingleMode")
00007560:         };
00007561:         unsafe { find_method_in_all_classes(method_name) }
00007562:     } else if path.starts_with("/fields") {
00007563:         let class_name = if path == "/fields" || path == "/fields/" {
00007564:             "WorkDataManager"
00007565:         } else {
00007566:             path.strip_prefix("/fields/").unwrap_or("WorkDataManager")
00007567:         };
00007568:         unsafe {
00007569:             let image = get_image();
00007570:             if image.is_null() {
00007571:                 r#"{"error":"image_null"}"#.to_string()
00007572:             } else {
00007573:                 let cls = find_class_by_short_name(image, class_name);
00007574:                 if cls.is_null() {
00007575:                     format!(r#"{{"error":"class_not_found","name":"{}"}}"#, class_name)
00007576:                 } else {
00007577:                     enumerate_class_fields(cls)
00007578:                 }
00007579:             }
00007580:         }
00007581:     } else if path.starts_with("/methods") {
00007582:         let class_name = if path == "/methods" || path == "/methods/" {
00007583:             "WorkDataManager"
00007584:         } else {
00007585:             path.strip_prefix("/methods/").unwrap_or("WorkDataManager")
00007586:         };
00007587:         unsafe {
00007588:             let image = get_image();
00007589:             if image.is_null() {
00007590:                 r#"{"error":"image_null"}"#.to_string()
00007591:             } else {
00007592:                 let cls = find_class_by_short_name(image, class_name);
00007593:                 if cls.is_null() {
00007594:                     format!(r#"{{"error":"class_not_found","name":"{}"}}"#, class_name)
00007595:                 } else {
00007596:                     enumerate_class_methods(cls)
00007597:                 }
00007598:             }
00007599:         }
00007600:     } else if path == "/summary" {
00007601:         read_summary()
00007602:     } else if path == "/debug/turn_probe" {
00007603:         // v3.24.72: expose decrypted raw field only; UI is a countdown and mapping is unknown.
00007604:         let s = read_summary();
00007605:         format!(
00007606:             r#"{{"status":"ok","raw_total_turn_num":{},"ui_turn_semantics":"countdown","raw_field_mapping":"unverified","year":null,"month":{},"half":{},"derived_turn":null}}"#,
00007607:             extract_json_int(&s, "\"raw_total_turn_num\"").unwrap_or(-1),
00007608:             extract_json_int(&s, "\"month\"").unwrap_or(-1),
00007609:             extract_json_int(&s, "\"half\"").unwrap_or(-1)
00007610:         )
00007611:     } else if path == "/debug/ramen_transition" {
00007612:         // v3.24.71: compact before/after observation; causality is always unknown.
00007613:         // Refresh summary first so this endpoint can itself advance the probe.
00007614:         let _ = read_summary();
00007615:         ramen_transition_probe()
00007616:     } else if path == "/ramen" {
00007617:         // v3.24.72: lightweight Ramen data; raw field semantics remain unverified.
00007618:         let s = read_summary();
00007619:         let ramen = extract_json_object(&s, "\"ramen\"");
00007620:         let raw_total_turn_num = extract_json_int(&s, "\"raw_total_turn_num\"");
00007621:         format!(
00007622:             r#"{{"status":"ok","raw_total_turn_num":{},"ui_turn_semantics":"countdown","raw_field_mapping":"unverified","ramen":{}}}"#,
00007623:             raw_total_turn_num.unwrap_or(-1),
00007624:             ramen.unwrap_or_else(|| "null".to_string())
00007625:         )
00007626:     } else if path == "/scenario" {
00007627:         let result = unsafe { read_scenario_detail() };
00007628:         unsafe {
00007629:             log_snapshot("scenario", &result);
00007630:         }
00007631:         result
00007632:     } else if path == "/log" {
00007633:         unsafe { get_training_log() }
00007634:     } else if path == "/debug/params" {
00007635:         unsafe { debug_params_inc_dec() }
```

## lines 8619..8820

```rust
00008619:             Err(_) => r#"{"status":"error","detail":"lock_failed"}"#.to_string(),
00008620:         }
00008621:     } else if path == "/events" {
00008622:         read_events_data()
00008623:     } else if path == "/debug/unique_skills" {
00008624:         debug_unique_skills()
00008625:     } else if path == "/debug/mdb_all_tables" {
00008626:         debug_mdb_all_tables()
00008627:     } else if path == "/debug/mdb_schema_dump" {
00008628:         debug_mdb_schema_dump()
00008629:     } else if path == "/debug/hint_gain" {
00008630:         debug_hint_gain()
00008631:     } else if path == "/debug/sc_effect" {
00008632:         debug_sc_effect()
00008633:     } else if path == "/debug/unique_detail" {
00008634:         debug_unique_detail()
00008635:     } else if path == "/debug/table" {
00008636:         let table_name = if let Some(q) = full_uri.find("?name=") {
00008637:             let rest = &full_uri[q + 6..];
00008638:             rest.split('&').next().unwrap_or(rest)
00008639:         } else {
00008640:             ""
00008641:         };
00008642:         let limit = if let Some(q) = full_uri.find("limit=") {
00008643:             full_uri[q + 6..]
00008644:                 .split('&')
00008645:                 .next()
00008646:                 .unwrap_or("100")
00008647:                 .parse::<usize>()
00008648:                 .unwrap_or(100)
00008649:         } else {
00008650:             100usize
00008651:         };
00008652:         let offset = if let Some(q) = full_uri.find("offset=") {
00008653:             full_uri[q + 7..]
00008654:                 .split("&")
00008655:                 .next()
00008656:                 .unwrap_or("0")
00008657:                 .parse::<usize>()
00008658:                 .unwrap_or(0)
00008659:         } else {
00008660:             0usize
00008661:         };
00008662:         debug_table_query(table_name, limit.min(1000).max(1), offset)
00008663:     } else if path == "/debug/download_table" {
00008664:         let table_name = if let Some(q) = full_uri.find("?name=") {
00008665:             let rest = &full_uri[q + 6..];
00008666:             rest.split('&').next().unwrap_or(rest)
00008667:         } else {
00008668:             ""
00008669:         };
00008670:         let batch = if let Some(q) = full_uri.find("batch=") {
00008671:             full_uri[q + 6..]
00008672:                 .split('&')
00008673:                 .next()
00008674:                 .unwrap_or("500")
00008675:                 .parse::<usize>()
00008676:                 .unwrap_or(500)
00008677:         } else {
00008678:             500usize
00008679:         };
00008680:         debug_download_table(table_name, batch.min(1000).max(1))
00008681:     } else if path == "/debug/push_table" {
00008682:         let table_name = if let Some(q) = full_uri.find("?name=") {
00008683:             let rest = &full_uri[q + 6..];
00008684:             rest.split('&').next().unwrap_or(rest)
00008685:         } else {
00008686:             ""
00008687:         };
00008688:         let batch = if let Some(q) = full_uri.find("batch=") {
00008689:             full_uri[q + 6..]
00008690:                 .split('&')
00008691:                 .next()
00008692:                 .unwrap_or("500")
00008693:                 .parse::<usize>()
00008694:                 .unwrap_or(500)
00008695:         } else {
00008696:             500usize
00008697:         };
00008698:         let offset = if let Some(q) = full_uri.find("offset=") {
00008699:             full_uri[q + 7..]
00008700:                 .split('&')
00008701:                 .next()
00008702:                 .unwrap_or("0")
00008703:                 .parse::<usize>()
00008704:                 .unwrap_or(0)
00008705:         } else {
00008706:             0usize
00008707:         };
00008708:         debug_push_table(table_name, batch.min(1000).max(1), offset)
00008709:     } else if path == "/tables" {
00008710:         read_mdb_tables()
00008711:     } else if path == "/carddb" {
00008712:         read_carddb()
00008713:     } else if path == "/skilldata" {
00008714:         read_skilldata()
00008715:     } else if path == "/hall" {
00008716:         unsafe { read_hall_data() }
00008717:     } else if path == "/event/recommend" {
00008718:         unsafe { read_event_recommend() }
00008719:     } else if path == "/inherit/compat" {
00008720:         unsafe { read_inherit_compat() }
00008721:     } else if path == "/saddle-analysis" {
00008722:         unsafe { read_win_saddle_analysis() }
00008723:     } else if path == "/log/turn" {
00008724:         unsafe { read_turn_log() }
00008725:     } else if path == "/ranking" {
00008726:         unsafe { read_ranking_data() }
00008727:     } else if path == "/saddles-dl" {
00008728:         read_saddles()
00008729:     } else if path == "/saddles" {
00008730:         read_saddles()
00008731:     } else if path == "/config" {
00008732:         let is_post = req.starts_with("POST");
00008733:         if is_post {
00008734:             // Parse body from request
00008735:             let body_start = req.find("\r\n\r\n").map(|i| i + 4).unwrap_or(req.len());
00008736:             let post_body = &req[body_start..];
00008737:             if let Some(new_cfg) = PluginConfig::from_json(post_body) {
00008738:                 let json = new_cfg.to_json();
00008739:                 unsafe {
00008740:                     update_config(new_cfg);
00008741:                 }
00008742:                 unsafe {
00008743:                     ura_log(3, &format!("Config updated: {}", json));
00008744:                 }
00008745:                 format!(r#"{{"ok":true,"config":{}}}"#, json)
00008746:             } else {
00008747:                 r#"{"ok":false,"error":"invalid_json"}"#.to_string()
00008748:             }
00008749:         } else {
00008750:             format!(
00008751:                 r#"{{"ok":true,"config":{}}}"#,
00008752:                 unsafe { get_config() }.to_json()
00008753:             )
00008754:         }
00008755:     } else if path == "/debug/dump" {
00008756:         // v3.22.89: Dump tool - group tables by first letter, one file per group
00008757:         let html = r#"<html><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Dump</title><style>body{font-family:system-ui;max-width:600px;margin:12px auto;padding:0 8px;background:#1a1a2e;color:#e0e0e0}h1{color:#4fc3f7;font-size:1.2em;margin:8px 0}.g{display:inline-block;margin:4px 2px;padding:8px 12px;background:#16213e;border:1px solid #333;border-radius:4px;color:#fff;cursor:pointer;font-size:14px;min-width:36px;text-align:center}.g:disabled{background:#555;color:#333;cursor:default}.g.ok{background:#2e7d32;border-color:#4caf50}.g.err{background:#b71c1c;border-color:#ff5252}.g.run{background:#e65100;border-color:#ff9800}select{padding:8px;background:#16213e;border:1px solid #333;border-radius:4px;color:#fff;font-size:16px;width:100%}button{padding:12px 24px;border:none;border-radius:4px;color:#000;font-weight:bold;cursor:pointer;font-size:16px;margin:4px}#btn{background:#4fc3f7}#btn:disabled{background:#555;color:#333}.p{margin:8px 0;font-size:0.95em}.ok{color:#4caf50}.err{color:#ff5252}progress{width:100%;height:20px;margin:8px 0}#lst{margin:8px 0;font-size:0.8em;color:#aaa;max-height:300px;overflow-y:auto}</style></head><body><h1>MDB Dump Tool</h1><div class="p" id="pg2">Loading table list...</div><div id="groups"></div><hr><select id="tn"><option value="">-- loading --</option></select><button id="btn" onclick="goOne()" disabled>Dump 1 Table</button><div class="p" id="pg">Press a letter group to dump all tables in that group as one file</div><progress id="pb" value="0" max="100"></progress><div id="lst"></div><script>function safeJson(t){try{return JSON.parse(t)}catch(e){return JSON.parse(t.replace(/[\x00-\x1f]/g,function(c){return"\\u"+("0000"+c.charCodeAt(0).toString(16)).slice(-4)}))}}var tables=[];var groups={};async function loadTables(){try{var r=await fetch("/debug/mdb_all_tables");var j=safeJson(await r.text());if(!j.ok){document.getElementById("pg2").innerHTML=`<span class="err">Error: ${j.error||"unknown"}</span>`;return;}tables=j.all_tables||[];var sel=document.getElementById("tn");sel.innerHTML="";groups={};for(var i=0;i<tables.length;i++){var t=tables[i];var o=document.createElement("option");o.value=t.name;o.textContent=t.name+" ("+t.rows+")";sel.appendChild(o);var fl=t.name[0].toUpperCase();if(!groups[fl])groups[fl]=[];groups[fl].push(t);}document.getElementById("btn").disabled=false;document.getElementById("pg2").innerHTML=`<span class="ok">${tables.length} tables in ${Object.keys(groups).length} groups</span>`;renderGroups();}catch(e){document.getElementById("pg2").innerHTML=`<span class="err">Fetch error: ${e}</span>`;}}function renderGroups(){var div=document.getElementById("groups");div.innerHTML="";var keys=Object.keys(groups).sort();for(var k=0;k<keys.length;k++){var key=keys[k];var btn=document.createElement("button");btn.className="g";btn.textContent=key+" ("+groups[key].length+")";btn.setAttribute("data-key",key);btn.onclick=function(){goGroup(this.getAttribute("data-key"),this);};div.appendChild(btn);}}async function dumpTable(n,onProgress){var allRows=[];var off=0;var total=0;var batch=100;var done=false;while(!done){try{var r=await fetch("/debug/table?name="+n+"&limit="+batch+"&offset="+off);var j=safeJson(await r.text());if(!j.ok){return{ok:false,error:j.error||"unknown"};}total=j.row_count||total;var nr=j.rows?j.rows.length:0;if(nr===0){done=true;break;}allRows=allRows.concat(j.rows);off+=nr;if(onProgress)onProgress(off,total);done=off>=total||nr<batch;}catch(e){return{ok:false,error:""+e};}}return{ok:true,table:n,row_count:total,rows_merged:allRows.length,rows:allRows};}function downloadJson(data,filename){var result=JSON.stringify(data);var blob=new Blob([result],{type:"application/json"});var url=URL.createObjectURL(blob);var a=document.createElement("a");a.href=url;a.download=filename;a.click();URL.revokeObjectURL(url);}async function goGroup(key,btn){btn.disabled=true;btn.className="g run";var tbls=groups[key];var result={group:key,tables:{}};var log=document.getElementById("lst");log.innerHTML="";var ok=0,fail=0;for(var i=0;i<tbls.length;i++){var t=tbls[i];document.getElementById("pg").innerHTML=`<span class="ok">[${key}] ${(i+1)}/${tbls.length} ${t.name} (${t.rows} rows)...</span>`;document.getElementById("pb").value=Math.round((i+1)/tbls.length*100);if(t.rows===0){result.tables[t.name]={ok:true,rows:0,data:[]};log.innerHTML+=t.name+": skip (0)<br>";ok++;continue;}var res=await dumpTable(t.name);if(res.ok&&res.rows_merged>0){result.tables[t.name]={ok:true,row_count:res.row_count,rows_merged:res.rows_merged,rows:res.rows};log.innerHTML+=t.name+`: <span class="ok">${res.rows_merged}</span><br>`;ok++;}else{result.tables[t.name]={ok:false,error:res.error||"no rows"};log.innerHTML+=t.name+`: <span class="err">${res.error||"no rows"}</span><br>`;fail++;}}var fname="mdb_"+key.toLowerCase()+".json";downloadJson(result,fname);btn.className=ok>0&&fail===0?"g ok":"g err";btn.disabled=false;document.getElementById("pg").innerHTML=`<span class="ok">${key}: ${ok} OK, ${fail} fail -> ${fname}</span>`;document.getElementById("pb").value=0;}async function goOne(){var b=document.getElementById("btn");var n=document.getElementById("tn").value;if(!n)return;b.disabled=true;document.getElementById("pg").innerHTML=`<span class="ok">Dumping ${n}...</span>`;var res=await dumpTable(n,function(off,total){var pct=total>0?Math.round(off/total*100):0;document.getElementById("pb").value=pct;document.getElementById("pg").innerHTML="Dumping "+n+": "+off+"/"+total+" ("+pct+"%)";});if(res.ok&&res.rows_merged>0){downloadJson(res,n+".json");document.getElementById("pg").innerHTML=`<span class="ok">Done! ${res.rows_merged}/${res.row_count} -> ${n}.json</span>`;}else{document.getElementById("pg").innerHTML=`<span class="err">${res.error?"Error: "+res.error:"No rows found"}</span>`;}document.getElementById("pb").value=0;b.disabled=false;}loadTables();</script></body></html>"#.to_string();
00008758:         html
00008759:     } else if path == "/config.html" {
00008760:         // Serve a simple HTML form for config editing - open in any browser
00008761:         let cfg = unsafe { get_config() };
00008762:         let html = format!(
00008763:             r#"<!DOCTYPE html><html><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>URA Plugin Config</title><style>body{{font-family:system-ui;max-width:500px;margin:20px auto;padding:0 16px;background:#1a1a2e;color:#e0e0e0}}h1{{color:#4fc3f7;font-size:1.3em}}label{{display:block;margin:12px 0 4px;color:#aaa;font-size:0.85em}}input{{width:100%;padding:8px;background:#16213e;border:1px solid #333;border-radius:4px;color:#fff;box-sizing:border-box}}button{{margin-top:16px;padding:10px 24px;background:#4fc3f7;border:none;border-radius:4px;color:#000;font-weight:bold;cursor:pointer}}.ok{{color:#4caf50;margin-top:8px}}</style></head><body><h1>URA Plugin Config</h1><form id="f"><label>Push Host</label><input id="push_host" value="{}"><label>Push Port</label><input id="push_port" type="number" value="{}"><label>HTTP Port</label><input id="http_port" type="number" value="{}"><label>Push Interval (sec)</label><input id="push_interval_secs" type="number" value="{}" min="1"><label>Push Enabled</label><input id="push_enabled" type="checkbox" {}><label>HTTP Enabled</label><input id="http_enabled" type="checkbox" {}><button type="submit">Save</button></form><div id="r"></div><script>document.getElementById('f').onsubmit=async(e)=>{{e.preventDefault();const d={{push_host:push_host.value,push_port:+push_port.value,http_port:+http_port.value,push_interval_secs:+push_interval_secs.value,push_enabled:push_enabled.checked,http_enabled:http_enabled.checked}};const r=await fetch('/config',{{method:'POST',headers:{{'Content-Type':'application/json'}},body:JSON.stringify(d)}});const j=await r.json();document.getElementById('r').innerHTML=j.ok?'<p class="ok">Saved!</p>':'<p style="color:red">Error: '+j.error+'</p>';}};</script></body></html>"#,
00008764:             cfg.push_host,
00008765:             cfg.push_port,
00008766:             cfg.http_port,
00008767:             cfg.push_interval_secs,
00008768:             if cfg.push_enabled { "checked" } else { "" },
00008769:             if cfg.http_enabled { "checked" } else { "" }
00008770:         );
00008771:         // Return HTML with text/html content type (handled below)
00008772:         html
00008773:     } else if path.starts_with("/classes") {
00008774:         let search = if path == "/classes" || path == "/classes/" {
00008775:             ""
00008776:         } else {
00008777:             path.strip_prefix("/classes/search/")
00008778:                 .or_else(|| path.strip_prefix("/classes/"))
00008779:                 .unwrap_or("")
00008780:         };
00008781:         unsafe { enumerate_all_classes(search) }
00008782:     } else if path.starts_with("/mdb/schema") {
00008783:         // v3.22.89: 表结构
00008784:         let table_name = parse_query(&full_uri, "name");
00008785:         mdb_schema(&table_name)
00008786:     } else if path.starts_with("/mdb/search") {
00008787:         // v3.22.89: 搜索表名和列名
00008788:         let keyword = parse_query(&full_uri, "keyword");
00008789:         mdb_search(&keyword)
00008790:     } else if path.starts_with("/mdb/raw") {
00008791:         // v3.22.89: 执行只读SQL
00008792:         let sql = parse_query(&full_uri, "sql");
00008793:         mdb_raw_query(&sql)
00008794:     } else if path.starts_with("/mdb/dl_batch") {
00008795:         // ★ 按首字母批量下载 MDB 表数据为 JSON 文件
00008796:         // /mdb/dl_batch?prefix=a  → 下载所有 a 开头的表
00008797:         // /mdb/dl_batch?prefix=all → 下载全部表（可能很大）
00008798:         let prefix = parse_query(&full_uri, "prefix");
00008799:         let body = mdb_dl_batch(&prefix);
00008800:         let safe_prefix: String = prefix.chars().filter(|c| c.is_alphanumeric()).collect();
00008801:         let fname = format!(
00008802:             "mdb_{}.json",
00008803:             if safe_prefix.is_empty() {
00008804:                 "ALL"
00008805:             } else {
00008806:                 &safe_prefix
00008807:             }
00008808:         );
00008809:         let resp = format!(
00008810:             "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Disposition: attachment; filename=\"{}\"\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
00008811:             fname, body.len(), body
00008812:         );
00008813:         let _ = stream.write_all(resp.as_bytes());
00008814:         return;
00008815:     } else if path.starts_with("/il2cpp/dump_all_methods_dl") {
00008816:         // v3.22.91: 暴力dump全部类方法目录（下载JSON，修复：内联下载包装）
00008817:         let letter = parse_query(&full_uri, "letter");
00008818:         let body = unsafe { il2cpp_dump_all_methods(&letter) };
00008819:         let safe_letter: String = letter.chars().filter(|c| c.is_alphanumeric()).collect();
00008820:         let fname = format!(
```

## lines 9092..9292

```rust
00009092:     } else if path == "/debug/private_file_inventory" {
00009093:         debug_private_file_inventory(&full_uri)
00009094:     } else if path == "/debug/private_file_dl" {
00009095:         download_private_file_by_id(&mut stream, &full_uri);
00009096:         return;
00009097:     } else if path.starts_with("/debug/file_dl") {
00009098:         // ★ v3.24.66: download an arbitrary file ONLY if its path currently appears
00009099:         // in /proc/self/maps (i.e. a loaded game library) — no free-form path reads.
00009100:         let want = parse_query(&full_uri, "path");
00009101:         let mut allowed = false;
00009102:         if !want.is_empty() {
00009103:             if let Ok(maps) = std::fs::read_to_string("/proc/self/maps") {
00009104:                 for line in maps.lines() {
00009105:                     let cols: Vec<&str> = line.split_whitespace().collect();
00009106:                     if cols.get(5).copied() == Some(want.as_str()) {
00009107:                         allowed = true;
00009108:                         break;
00009109:                     }
00009110:                 }
00009111:             }
00009112:         }
00009113:         if !allowed {
00009114:             format!(
00009115:                 r#"{{"error":"not_in_maps","hint":"path must appear in /proc/self/maps (see /debug/maps_list)"}}"#
00009116:             )
00009117:         } else {
00009118:             let fname = std::path::Path::new(&want)
00009119:                 .file_name()
00009120:                 .and_then(|v| v.to_str())
00009121:                 .unwrap_or("file.bin")
00009122:                 .to_string();
00009123:             stream_private_file(&mut stream, &want, &fname);
00009124:             return;
00009125:         }
00009126:     } else if path == "/debug/resource_storage" {
00009127:         debug_resource_storage()
00009128:     } else if path == "/debug/resource_meta_schema" {
00009129:         debug_resource_meta_schema()
00009130:     } else if path == "/debug/resource_meta_probe" {
00009131:         debug_resource_meta_probe()
00009132:     } else if path == "/debug/resource_crypto_symbols" {
00009133:         debug_resource_crypto_symbols()
00009134:     } else if path == "/debug/resource_meta_dl" {
00009135:         // Allow only the index and its known SQLite sidecars; never an arbitrary path.
00009136:         let part = parse_query(&full_uri, "part");
00009137:         let (suffix, filename) = match part.as_str() {
00009138:             "journal" => ("-journal", "meta-journal"),
00009139:             "wal" => ("-wal", "meta-wal"),
00009140:             "shm" => ("-shm", "meta-shm"),
00009141:             _ => ("", "meta"),
00009142:         };
00009143:         match find_resource_storage() {
00009144:             Ok((meta, _)) => {
00009145:                 let target = format!("{}{}", meta, suffix);
00009146:                 stream_private_file(&mut stream, &target, filename);
00009147:                 return;
00009148:             }
00009149:             Err(e) => format!(r#"{{"error":"{}"}}"#, json_escape(&e)),
00009150:         }
00009151:     } else if path == "/debug/resource_file_dl" {
00009152:         // v3.24.62: meta `a` 表的 h 是 Base32(A-Z2-7,32字符) 且就是 dat 文件名原文，
00009153:         // 与 hex 哈希一并接受；Base32 需保持原样（不做 lowercase）。
00009154:         let raw_hash = parse_query(&full_uri, "hash");
00009155:         let hash = if raw_hash.len() == 32
00009156:             && raw_hash
00009157:                 .bytes()
00009158:                 .all(|b| matches!(b, b'A'..=b'Z' | b'a'..=b'z' | b'2'..=b'7'))
00009159:             && !raw_hash.bytes().all(|b| b.is_ascii_hexdigit())
00009160:         {
00009161:             raw_hash.to_ascii_uppercase()
00009162:         } else {
00009163:             raw_hash.to_ascii_lowercase()
00009164:         };
00009165:         let hash_ok = valid_resource_hash(&hash)
00009166:             || (hash.len() == 32 && hash.bytes().all(|b| matches!(b, b'A'..=b'Z' | b'2'..=b'7')));
00009167:         if !hash_ok {
00009168:             r#"{"error":"invalid_hash","requirement":"8..128 hexadecimal characters"}"#.to_string()
00009169:         } else {
00009170:             match find_resource_storage() {
00009171:                 Ok((_, dat)) => {
00009172:                     let target = std::path::Path::new(&dat).join(&hash[..2]).join(&hash);
00009173:                     if !target.is_file() {
00009174:                         format!(r#"{{"error":"resource_not_found","hash":"{}"}}"#, hash)
00009175:                     } else {
00009176:                         stream_private_file(&mut stream, &target.to_string_lossy(), &hash);
00009177:                         return;
00009178:                     }
00009179:                 }
00009180:                 Err(e) => format!(r#"{{"error":"{}"}}"#, json_escape(&e)),
00009181:             }
00009182:         }
00009183:     } else if path == "/mdb" {
00009184:         // v3.22.51: Serve raw MasterDB file for client-side processing
00009185:         // Uses marker string; binary file sent in response handler below
00009186:         match find_mdb_path() {
00009187:             Some(mdb_path) => format!("__MDB_BINARY__{}", mdb_path),
00009188:             None => r#"{"error":"mdb_not_found"}"#.to_string(),
00009189:         }
00009190:     } else {
00009191:         format!(
00009192:             r#"{{"error":"not_found","path":"{}","available":["/scan","/data","/status","/health","/scenario","/debug/upload","/debug/rameninfo","/debug/laststep","/event/recommend","/inherit/compat","/saddle-analysis","/log/turn","/log","/debug/params","/fields","/methods","/singletons","/find_method","/classes","/carddb","/skilldata","/hall","/debug/breeders","/debug/cmdinfo","/debug/training_partners","/debug/ramengains","/debug/paramsincdec","/debug/training_seed","/debug/training_log","/debug/training_log_dl","/update","/update/status","/debug/dumpclass","/debug/storydata","/debug/ramenfields","/debug/all","/mdb","/debug/push_table","/debug/download_table","/classes/search/keyword","/mdb/schema","/mdb/search","/mdb/raw","/mdb/dl_batch","/il2cpp/dump","/il2cpp/call","/il2cpp/tree","/il2cpp/field","/il2cpp/classes","/il2cpp/static","/il2cpp/methods","/il2cpp/search_float","/il2cpp/search_float_dl","/il2cpp/search_int","/il2cpp/search_int_dl","/il2cpp/search_methods","/il2cpp/search_methods_dl","/il2cpp/search_methods_page","/il2cpp/read_mem","/il2cpp/read_mem_dl","/training/result","/api/sniff","/api/sniff/metadata","/api/sniff/status","/api/sniff/toggle","/api/sniff/clear","/api/sniff/diag","/api/event/choices","/api/event/clear"]}}"#,
00009193:             path
00009194:         )
00009195:     };
00009196: 
00009197:     save_endpoint_log(&path, &body);
00009198: 
00009199:     if body.starts_with("__MDB_BINARY__") {
00009200:         // v3.22.51: Serve raw mdb file as binary response
00009201:         let mdb_path = &body[14..]; // skip "__MDB_BINARY__"
00009202:         match std::fs::read(mdb_path) {
00009203:             Ok(data) => {
00009204:                 let header = format!(
00009205:                     "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Disposition: attachment; filename=\"master.mdb\"\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
00009206:                     data.len()
00009207:                 );
00009208:                 let _ = stream.write_all(header.as_bytes());
00009209:                 // Write in chunks to avoid memory spike
00009210:                 for chunk in data.chunks(65536) {
00009211:                     let _ = stream.write_all(chunk);
00009212:                 }
00009213:             }
00009214:             Err(e) => {
00009215:                 let err_json = format!(r#"{{"error":"mdb_read_failed","detail":"{}"}}"#, e);
00009216:                 let resp = format!(
00009217:                     "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
00009218:                     err_json.len(), err_json
00009219:                 );
00009220:                 let _ = stream.write_all(resp.as_bytes());
00009221:             }
00009222:         }
00009223:     } else if path == "/saddles-dl" {
00009224:         let resp = format!(
00009225:             "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Disposition: attachment; filename=\"saddles.json\"\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
00009226:             body.len(), body
00009227:         );
00009228:         let _ = stream.write_all(resp.as_bytes());
00009229:     } else if path == "/il2cpp/disassemble_dl" {
00009230:         // v3.22.89: 反汇编结果下载为JSON文件
00009231:         let cn = parse_query(&full_uri, "class");
00009232:         let mn = parse_query(&full_uri, "method");
00009233:         let safe_name: String = format!(
00009234:             "{}_{}",
00009235:             cn.chars()
00009236:                 .filter(|c| c.is_alphanumeric() || *c == '_')
00009237:                 .collect::<String>(),
00009238:             mn.chars()
00009239:                 .filter(|c| c.is_alphanumeric() || *c == '_')
00009240:                 .collect::<String>()
00009241:         );
00009242:         let fname = format!(
00009243:             "disassemble_{}.json",
00009244:             if safe_name.is_empty() {
00009245:                 "output"
00009246:             } else {
00009247:                 &safe_name
00009248:             }
00009249:         );
00009250:         let resp = format!(
00009251:             "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Disposition: attachment; filename=\"{}\"\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
00009252:             fname, body.len(), body
00009253:         );
00009254:     } else {
00009255:         let content_type = if body.starts_with("<!DOCTYPE") || body.starts_with("<html") {
00009256:             "text/html; charset=utf-8"
00009257:         } else {
00009258:             "application/json"
00009259:         };
00009260:         if dl_enabled {
00009261:             // 下载模式：默认按路由生成文件名，?name= 可覆盖
00009262:             let safe: String = dl_name
00009263:                 .chars()
00009264:                 .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
00009265:                 .take(64)
00009266:                 .collect();
00009267:             let fallback = path.trim_matches('/').replace('/', "_");
00009268:             let base = if safe.is_empty() { fallback } else { safe };
00009269:             let base = if base.is_empty() {
00009270:                 "download".to_string()
00009271:             } else {
00009272:                 base
00009273:             };
00009274:             let ext = if content_type.starts_with("text/html") {
00009275:                 "html"
00009276:             } else {
00009277:                 "json"
00009278:             };
00009279:             let fname = format!("{}.{}", base, ext);
00009280:             let resp = format!(
00009281:                 "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Disposition: attachment; filename=\"{}\"\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
00009282:                 fname, body.len(), body
00009283:             );
00009284:             let _ = stream.write_all(resp.as_bytes());
00009285:         } else {
00009286:             let resp = format!(
00009287:                 "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
00009288:                 content_type, body.len(), body
00009289:             );
00009290:             let _ = stream.write_all(resp.as_bytes());
00009291:         }
00009292:     }
```

## lines 9344..9544

```rust
00009344:         let mut iter: *mut c_void = ptr::null_mut();
00009345:         loop {
00009346:             let field_info = get_fields_fn.unwrap()(current_class, &mut iter);
00009347:             if field_info.is_null() {
00009348:                 break;
00009349:             }
00009350:             if !(*field_info).name.is_null() {
00009351:                 let s = std::ffi::CStr::from_ptr((*field_info).name);
00009352:                 let fname = s.to_string_lossy().to_string();
00009353:                 let offset = (*field_info).offset;
00009354:                 // Extract property name from <PropName>k__BackingField
00009355:                 let prop_name = if fname.starts_with('<') {
00009356:                     if let Some(end) = fname.find('>') {
00009357:                         &fname[1..end]
00009358:                     } else {
00009359:                         &fname
00009360:                     }
00009361:                 } else {
00009362:                     &fname
00009363:                 };
00009364:                 // Store multiple cache keys for robust lookup
00009365:                 let keys = [
00009366:                     format!("{:p}_{}", class, prop_name),
00009367:                     format!("{:p}_{}", class, prop_name.to_lowercase()),
00009368:                     format!("{:p}_{}", class, to_snake_case(prop_name)),
00009369:                 ];
00009370:                 if let Ok(mut guard) = FIELD_OFFSET_CACHE.lock() {
00009371:                     if guard.is_none() {
00009372:                         *guard = Some(HashMap::new());
00009373:                     }
00009374:                     if let Some(ref mut map) = *guard {
00009375:                         for k in &keys {
00009376:                             map.insert(k.clone(), offset);
00009377:                         }
00009378:                     }
00009379:                 }
00009380:             }
00009381:         }
00009382:         if let Some(ref get_parent) = get_parent_fn {
00009383:             let parent = get_parent(current_class);
00009384:             if parent.is_null() || parent == current_class {
00009385:                 break;
00009386:             }
00009387:             current_class = parent;
00009388:         } else {
00009389:             break;
00009390:         }
00009391:         depth += 1;
00009392:     }
00009393: }
00009394: 
00009395: /// Pre-cache all known classes and field offsets on game thread
00009396: unsafe fn precache_metadata() {
00009397:     ura_log(2, "v3.22.51 precache_metadata: starting");
00009398:     let image = match get_image() {
00009399:         img if !img.is_null() => img,
00009400:         _ => {
00009401:             ura_log(1, "precache_metadata: image null");
00009402:             return;
00009403:         }
00009404:     };
00009405: 
00009406:     // Classes found via find_class(image, "Gallop", X)
00009407:     let gallop_classes = [
00009408:         "WorkDataManager",
00009409:         "WorkSingleModeData",
00009410:         "WorkSingleModeCharaData",
00009411:         "WorkSingleModeHomeInfoData",
00009412:         "WorkSingleModeScenarioRamen",
00009413:         "WorkSingleModeScenarioURA",
00009414:         "WorkSingleModeScenarioTeamRace",
00009415:         "WorkSingleModeScenarioLive",
00009416:         "WorkSingleModeScenarioFree",
00009417:         "WorkSingleModeScenarioVenus",
00009418:         "WorkSingleModeScenarioArc",
00009419:         "WorkSingleModeScenarioSport",
00009420:         "WorkSingleModeScenarioCook",
00009421:         "WorkSingleModeScenarioMecha",
00009422:         "WorkSingleModeScenarioLegend",
00009423:         "WorkSingleModeScenarioPioneer",
00009424:         "WorkSingleModeScenarioOnsen",
00009425:         "WorkSingleModeScenarioBreeders",
00009426:     ];
00009427: 
00009428:     // Classes found via find_class_by_short_name
00009429:     let short_name_classes = [
00009430:         "SingleModeSkillData",
00009431:         "SingleModeCommandInfoData",
00009432:         "SingleModeParamsIncDecInfoData",
00009433:         "ObscuredSingleModeBreedersEnhanceGroup",
00009434:         "ObscuredSingleModeBreedersCommandInfo",
00009435:         "WorkSingleModeScenarioRamenDataSet",
00009436:         "ObscuredSingleModeRamenFeeling",
00009437:         "ObscuredSingleModeRamenFeelingTurnInfo",
00009438:         "ObscuredSingleModeRamenCommandFeelingInfo",
00009439:         "ObscuredSingleModeRamenFeelingReduceTurnInfo",
00009440:         "ObscuredSingleModeRamenUrafEffectInfo",
00009441:         "ObscuredSingleModeRamenActiveEffectInfo",
00009442:         "WorkTrainedCharaData",
00009443:         "TrainedCharaData",
00009444:         "SuccessionCharaInfo",
00009445:         "WorkSingleModeScenarioURADataSet",
00009446:         "WorkSingleModeScenarioTeamRaceDataSet",
00009447:         "WorkSingleModeScenarioLiveDataSet",
00009448:         "WorkSingleModeScenarioFreeDataSet",
00009449:         "WorkSingleModeScenarioVenusDataSet",
00009450:         "WorkSingleModeScenarioArcDataSet",
00009451:         "WorkSingleModeScenarioSportDataSet",
00009452:         "WorkSingleModeScenarioCookDataSet",
00009453:         "WorkSingleModeScenarioMechaDataSet",
00009454:         "WorkSingleModeScenarioLegendDataSet",
00009455:         "WorkSingleModeScenarioPioneerDataSet",
00009456:         "WorkSingleModeScenarioOnsenDataSet",
00009457:         "WorkSingleModeScenarioBreedersDataSet",
00009458:     ];
00009459: 
00009460:     let mut cached_count = 0i32;
00009461: 
00009462:     // Cache Gallop namespace classes
00009463:     for name in &gallop_classes {
00009464:         let cls = find_class(image, to_cstr("Gallop").as_ptr(), to_cstr(name).as_ptr());
00009465:         if !cls.is_null() {
00009466:             if let Ok(mut guard) = CLASS_CACHE.lock() {
00009467:                 if guard.is_none() {
00009468:                     *guard = Some(HashMap::new());
00009469:                 }
00009470:                 if let Some(ref mut map) = *guard {
00009471:                     map.insert(name.to_string(), cls as usize);
00009472:                 }
00009473:             }
00009474:             precache_all_fields(cls);
00009475:             cached_count += 1;
00009476:         }
00009477:     }
00009478: 
00009479:     // Cache short-name classes
00009480:     for name in &short_name_classes {
00009481:         let cls = find_class_by_short_name(image, name);
00009482:         if !cls.is_null() {
00009483:             if let Ok(mut guard) = CLASS_CACHE.lock() {
00009484:                 if guard.is_none() {
00009485:                     *guard = Some(HashMap::new());
00009486:                 }
00009487:                 if let Some(ref mut map) = *guard {
00009488:                     map.insert(name.to_string(), cls as usize);
00009489:                 }
00009490:             }
00009491:             precache_all_fields(cls);
00009492:             cached_count += 1;
00009493:         }
00009494:     }
00009495: 
00009496:     // Cache WorkDataManager singleton
00009497:     if let Some(wdm_cls) = CLASS_CACHE
00009498:         .lock()
00009499:         .ok()
00009500:         .and_then(|g| g.as_ref().and_then(|m| m.get("WorkDataManager").copied()))
00009501:     {
00009502:         let wdm_ptr = wdm_cls as *mut c_void;
00009503:         let inst = get_singleton(wdm_ptr);
00009504:         if !inst.is_null() {
00009505:             if let Ok(mut guard) = SINGLETON_CACHE.lock() {
00009506:                 if guard.is_none() {
00009507:                     *guard = Some(HashMap::new());
00009508:                 }
00009509:                 if let Some(ref mut map) = *guard {
00009510:                     map.insert(wdm_cls, inst as usize);
00009511:                 }
00009512:             }
00009513:             ura_log(
00009514:                 2,
00009515:                 &format!("precache_metadata: WDM singleton cached at {:p}", inst),
00009516:             );
00009517:         }
00009518:     }
00009519: 
00009520:     // Count cached field offsets
00009521:     let field_count = FIELD_OFFSET_CACHE
00009522:         .lock()
00009523:         .ok()
00009524:         .and_then(|g| g.as_ref().map(|m| m.len()))
00009525:         .unwrap_or(0);
00009526: 
00009527:     ura_log(
00009528:         2,
00009529:         &format!(
00009530:             "v3.22.51 precache_metadata: done — {} classes, {} field offsets cached",
00009531:             cached_count, field_count
00009532:         ),
00009533:     );
00009534: }
00009535: 
00009536: // ============================================================
00009537: // Menu Callbacks
00009538: // ============================================================
00009539: 
00009540: extern "C" fn on_menu_item_click(_userdata: *mut c_void) {
00009541:     unsafe {
00009542:         ura_log(3, "URA menu item clicked");
00009543:     }
00009544: }
```

## lines 16136..16352

```rust
00016136:                 row.get::<_, i32>(1).unwrap_or(0),
00016137:                 row.get::<_, i32>(2).unwrap_or(0),
00016138:                 row.get::<_, i32>(3).unwrap_or(0),
00016139:                 row.get::<_, i32>(4).unwrap_or(0),
00016140:                 row.get::<_, i32>(6).unwrap_or(0),
00016141:                 row.get::<_, i32>(7).unwrap_or(0),
00016142:                 row.get::<_, i32>(8).unwrap_or(0),
00016143:                 row.get::<_, i32>(9).unwrap_or(0),
00016144:                 row.get::<_, i32>(10).unwrap_or(0),
00016145:                 row.get::<_, i32>(11).unwrap_or(0),
00016146:                 row.get::<_, i32>(12).unwrap_or(0),
00016147:                 row.get::<_, i32>(13).unwrap_or(0),
00016148:             ))
00016149:         }).unwrap().filter_map(|r| r.ok()).collect(),
00016150:         Err(e) => return format!(r#"{{"error":"saddle_prepare_failed","detail":"{}"}}"#, e),
00016151:     };
00016152: 
00016153:     // Collect chara_program (which chara runs which program_group)
00016154:     let chara_programs: Vec<String> = match conn.prepare(
00016155:         "SELECT chara_id, program_group, program_group_2 FROM single_mode_chara_program ORDER BY program_group, chara_id"
00016156:     ) {
00016157:         Ok(mut stmt) => stmt.query_map([], |row| {
00016158:             Ok(format!(
00016159:                 r#"{{"chara_id":{},"program_group":{},"program_group_2":{}}}"#,
00016160:                 row.get::<_, i32>(0).unwrap_or(0),
00016161:                 row.get::<_, i32>(1).unwrap_or(0),
00016162:                 row.get::<_, i32>(2).unwrap_or(0),
00016163:             ))
00016164:         }).unwrap().filter_map(|r| r.ok()).collect(),
00016165:         Err(e) => return format!(r#"{{"error":"program_prepare_failed","detail":"{}"}}"#, e),
00016166:     };
00016167: 
00016168:     // Collect program race mapping
00016169:     let programs: Vec<String> = match conn.prepare(
00016170:         "SELECT id, program_group, race_instance_id, month, half FROM single_mode_program ORDER BY program_group, month, half"
00016171:     ) {
00016172:         Ok(mut stmt) => stmt.query_map([], |row| {
00016173:             Ok(format!(
00016174:                 r#"{{"id":{},"program_group":{},"race_instance_id":{},"month":{},"half":{}}}"#,
00016175:                 row.get::<_, i32>(0).unwrap_or(0),
00016176:                 row.get::<_, i32>(1).unwrap_or(0),
00016177:                 row.get::<_, i32>(2).unwrap_or(0),
00016178:                 row.get::<_, i32>(3).unwrap_or(0),
00016179:                 row.get::<_, i32>(4).unwrap_or(0),
00016180:             ))
00016181:         }).unwrap().filter_map(|r| r.ok()).collect(),
00016182:         Err(e) => return format!(r#"{{"error":"prog_prepare_failed","detail":"{}"}}"#, e),
00016183:     };
00016184: 
00016185:     // Collect race names (category=32 = race name in text_data)
00016186:     let race_names: Vec<String> = match conn.prepare(&format!(
00016187:         "SELECT [index], text FROM text_data WHERE category={} ORDER BY [index]",
00016188:         TEXT_DATA_CATEGORY_RACE_NAME
00016189:     )) {
00016190:         Ok(mut stmt) => stmt
00016191:             .query_map([], |row| {
00016192:                 let text: String = row
00016193:                     .get::<_, Option<String>>(1)
00016194:                     .unwrap_or(None)
00016195:                     .unwrap_or_default();
00016196:                 Ok(format!(
00016197:                     r#"{{"race_id":{},"name":"{}"}}"#,
00016198:                     row.get::<_, i32>(0).unwrap_or(0),
00016199:                     json_escape(&text),
00016200:                 ))
00016201:             })
00016202:             .unwrap()
00016203:             .filter_map(|r| r.ok())
00016204:             .collect(),
00016205:         Err(e) => return format!(r#"{{"error":"race_name_prepare_failed","detail":"{}"}}"#, e),
00016206:     };
00016207: 
00016208:     // Collect chara names (category=6 = chara name in text_data)
00016209:     let chara_names: Vec<String> = match conn.prepare(&format!(
00016210:         "SELECT [index], text FROM text_data WHERE category={} ORDER BY [index]",
00016211:         TEXT_DATA_CATEGORY_CHARA_NAME
00016212:     )) {
00016213:         Ok(mut stmt) => stmt
00016214:             .query_map([], |row| {
00016215:                 let text: String = row
00016216:                     .get::<_, Option<String>>(1)
00016217:                     .unwrap_or(None)
00016218:                     .unwrap_or_default();
00016219:                 Ok(format!(
00016220:                     r#"{{"chara_id":{},"name":"{}"}}"#,
00016221:                     row.get::<_, i32>(0).unwrap_or(0),
00016222:                     json_escape(&text),
00016223:                 ))
00016224:             })
00016225:             .unwrap()
00016226:             .filter_map(|r| r.ok())
00016227:             .collect(),
00016228:         Err(e) => {
00016229:             return format!(
00016230:                 r#"{{"error":"chara_name_prepare_failed","detail":"{}"}}"#,
00016231:                 e
00016232:             )
00016233:         }
00016234:     };
00016235: 
00016236:     // Collect succession_relation (fixed compatibility scores)
00016237:     let relations: Vec<String> = match conn.prepare(
00016238:         "SELECT relation_type, relation_point FROM succession_relation ORDER BY relation_type, relation_point"
00016239:     ) {
00016240:         Ok(mut stmt) => stmt.query_map([], |row| {
00016241:             Ok(format!(
00016242:                 r#"{{"relation_type":{},"relation_point":{}}}"#,
00016243:                 row.get::<_, i32>(0).unwrap_or(0),
00016244:                 row.get::<_, i32>(1).unwrap_or(0),
00016245:             ))
00016246:         }).unwrap().filter_map(|r| r.ok()).collect(),
00016247:         Err(e) => return format!(r#"{{"error":"relation_prepare_failed","detail":"{}"}}"#, e),
00016248:     };
00016249: 
00016250:     // Collect succession_relation_member
00016251:     let relation_members: Vec<String> = match conn.prepare(
00016252:         "SELECT id, relation_type, chara_id FROM succession_relation_member ORDER BY relation_type, chara_id"
00016253:     ) {
00016254:         Ok(mut stmt) => stmt.query_map([], |row| {
00016255:             Ok(format!(
00016256:                 r#"{{"id":{},"relation_type":{},"chara_id":{}}}"#,
00016257:                 row.get::<_, i32>(0).unwrap_or(0),
00016258:                 row.get::<_, i32>(1).unwrap_or(0),
00016259:                 row.get::<_, i32>(2).unwrap_or(0),
00016260:             ))
00016261:         }).unwrap().filter_map(|r| r.ok()).collect(),
00016262:         Err(e) => return format!(r#"{{"error":"member_prepare_failed","detail":"{}"}}"#, e),
00016263:     };
00016264: 
00016265:     // Collect race_instance to race_course_set mapping (for venue info)
00016266:     let race_instances: Vec<String> = match conn.prepare(
00016267:         "SELECT ri.id, ri.race_id, r.grade, r.course_set, cs.race_track_id, cs.distance, cs.ground FROM race_instance ri JOIN race r ON ri.race_id=r.id JOIN race_course_set cs ON r.course_set=cs.id WHERE r.grade=100 ORDER BY ri.id"
00016268:     ) {
00016269:         Ok(mut stmt) => stmt.query_map([], |row| {
00016270:             Ok(format!(
00016271:                 r#"{{"id":{},"race_id":{},"grade":{},"course_set":{},"race_track_id":{},"distance":{},"ground":{}}}"#,
00016272:                 row.get::<_, i32>(0).unwrap_or(0),
00016273:                 row.get::<_, i32>(1).unwrap_or(0),
00016274:                 row.get::<_, i32>(2).unwrap_or(0),
00016275:                 row.get::<_, i32>(3).unwrap_or(0),
00016276:                 row.get::<_, i32>(4).unwrap_or(0),
00016277:                 row.get::<_, i32>(5).unwrap_or(0),
00016278:                 row.get::<_, i32>(6).unwrap_or(0),
00016279:             ))
00016280:         }).unwrap().filter_map(|r| r.ok()).collect(),
00016281:         Err(e) => return format!(r#"{{"error":"race_inst_prepare_failed","detail":"{}"}}"#, e),
00016282:     };
00016283: 
00016284:     drop(conn);
00016285: 
00016286:     format!(
00016287:         r#"{{"ok":true,"version":"3.22.91","mdb":"{}","saddle_count":{},"program_chara_count":{},"program_count":{},"race_name_count":{},"chara_name_count":{},"relation_count":{},"member_count":{},"race_instance_count":{},"saddles":[{}],"chara_programs":[{}],"programs":[{}],"race_names":[{}],"chara_names":[{}],"relations":[{}],"relation_members":[{}],"race_instances":[{}]}}"#,
00016288:         mdb_path,
00016289:         saddles.len(),
00016290:         chara_programs.len(),
00016291:         programs.len(),
00016292:         race_names.len(),
00016293:         chara_names.len(),
00016294:         relations.len(),
00016295:         relation_members.len(),
00016296:         race_instances.len(),
00016297:         saddles.join(","),
00016298:         chara_programs.join(","),
00016299:         programs.join(","),
00016300:         race_names.join(","),
00016301:         chara_names.join(","),
00016302:         relations.join(","),
00016303:         relation_members.join(","),
00016304:         race_instances.join(","),
00016305:     )
00016306: }
00016307: 
00016308: /// /hall - Read 殿堂 (Hall of Fame) data via TrainedCharaData
00016309: /// Path: WDM -> get_TrainedCharaData -> WorkTrainedCharaData -> get_List -> List<TrainedCharaData>
00016310: /// Each TrainedCharaData has get_RankScore (評価点), get_Speed/Stamina/Power/Guts/Wiz, etc.
00016311: /// _rankScore is the game's own calculated評価点 (gold standard for verification)
00016312: unsafe fn read_hall_data() -> String {
00016313:     if API.is_null() {
00016314:         return r#"{"error":"api_null"}"#.to_string();
00016315:     }
00016316:     let image = match get_image() {
00016317:         img if !img.is_null() => img,
00016318:         _ => return r#"{"error":"image_null"}"#.to_string(),
00016319:     };
00016320: 
00016321:     // 1. Get WDM singleton
00016322:     let wdm_class = find_class(
00016323:         image,
00016324:         to_cstr("Gallop").as_ptr(),
00016325:         to_cstr("WorkDataManager").as_ptr(),
00016326:     );
00016327:     if wdm_class.is_null() {
00016328:         return r#"{"error":"no_wdm"}"#.to_string();
00016329:     }
00016330:     let wdm_inst = get_singleton(wdm_class);
00016331:     if wdm_inst.is_null() {
00016332:         return r#"{"error":"no_wdm_inst"}"#.to_string();
00016333:     }
00016334: 
00016335:     // 2. Get WorkTrainedCharaData from WDM
00016336:     let wtcd_inst = call_getter_ref(wdm_class, wdm_inst, "get_TrainedCharaData");
00016337:     if wtcd_inst.is_null() {
00016338:         ura_log(1, "/hall: get_TrainedCharaData returned null");
00016339:         return r#"{"error":"no_trained_chara_data"}"#.to_string();
00016340:     }
00016341:     ura_log(2, "/hall: got WorkTrainedCharaData instance");
00016342: 
00016343:     // 3. Find WorkTrainedCharaData class for calling get_List
00016344:     let wtcd_class = find_class_by_short_name(image, "WorkTrainedCharaData");
00016345: 
00016346:     // 4. Get List<TrainedCharaData> from WorkTrainedCharaData
00016347:     let list_obj = call_getter_ref(wtcd_class, wtcd_inst, "get_List");
00016348:     if list_obj.is_null() {
00016349:         ura_log(1, "/hall: get_List returned null");
00016350:         return r#"{"error":"no_list"}"#.to_string();
00016351:     }
00016352: 
```

## lines 19444..19765

```rust
00019444: 
00019445: /// /training/predict — Detailed training prediction with NPC partner breakdown
00019446: /// Returns per-command: gains, partner details (support card vs NPC), buffs, failure risk
00019447: /// Key data sources:
00019448: ///   - WorkSingleModeData -> get_HomeInfoData -> CommandInfoArray (training layout + partners)
00019449: ///   - WorkSingleModeCharaData -> CharaEffectBuffArray (active buffs)
00019450: ///   - WorkSingleModeScenarioRamenDataSet (ramen-specific data, scenario_id==14)
00019451: unsafe fn read_ramen_info() -> String {
00019452:     if API.is_null() {
00019453:         return r#"{"error":"api_null"}"#.to_string();
00019454:     }
00019455:     let image = match get_image() {
00019456:         img if !img.is_null() => img,
00019457:         _ => return r#"{"error":"image_null"}"#.to_string(),
00019458:     };
00019459:     let wdm_class = find_class(
00019460:         image,
00019461:         to_cstr("Gallop").as_ptr(),
00019462:         to_cstr("WorkDataManager").as_ptr(),
00019463:     );
00019464:     if wdm_class.is_null() {
00019465:         return r#"{"error":"no_wdm"}"#.to_string();
00019466:     }
00019467:     let wdm_inst = get_singleton(wdm_class);
00019468:     if wdm_inst.is_null() {
00019469:         return r#"{"error":"no_wdm_inst"}"#.to_string();
00019470:     }
00019471:     let sm_class = find_class(
00019472:         image,
00019473:         to_cstr("Gallop").as_ptr(),
00019474:         to_cstr("WorkSingleModeData").as_ptr(),
00019475:     );
00019476:     let sm_obj = call_getter_ref(wdm_class, wdm_inst, "get_SingleMode");
00019477:     if sm_obj.is_null() {
00019478:         return r#"{"error":"no_sm"}"#.to_string();
00019479:     }
00019480:     let chara_class = find_class(
00019481:         image,
00019482:         to_cstr("Gallop").as_ptr(),
00019483:         to_cstr("WorkSingleModeCharaData").as_ptr(),
00019484:     );
00019485:     let chara_obj = call_getter_ref(sm_class, sm_obj, "get_Character");
00019486:     if chara_obj.is_null() {
00019487:         return r#"{"error":"no_chara"}"#.to_string();
00019488:     }
00019489: 
00019490:     let ramen_sc_class = find_class(
00019491:         image,
00019492:         to_cstr("Gallop").as_ptr(),
00019493:         to_cstr("WorkSingleModeScenarioRamen").as_ptr(),
00019494:     );
00019495:     if ramen_sc_class.is_null() {
00019496:         return r#"{"error":"no_ramen_sc_class"}"#.to_string();
00019497:     }
00019498:     let ramen_sc_obj = try_get_scenario_obj(chara_class, chara_obj, 14);
00019499:     if ramen_sc_obj.is_null() {
00019500:         return r#"{"error":"no_ramen_sc_obj"}"#.to_string();
00019501:     }
00019502:     let ramen_ds_obj = call_getter_ref(ramen_sc_class, ramen_sc_obj, "get_DataSet");
00019503:     if ramen_ds_obj.is_null() {
00019504:         return r#"{"error":"no_ramen_ds"}"#.to_string();
00019505:     }
00019506: 
00019507:     // Read class pointer from object header (offset 0 on 64-bit = Il2CppObject.klass)
00019508:     let ds_base = ramen_ds_obj as *const u8;
00019509:     let ds_class_ptr = std::ptr::read_unaligned::<*mut c_void>(ds_base as *const *mut c_void);
00019510: 
00019511:     // Hex dump first 256 bytes
00019512:     let mut hex = String::new();
00019513:     for i in 0..256usize {
00019514:         let b = std::ptr::read_unaligned::<u8>(ds_base.add(i));
00019515:         hex.push_str(&format!("{:02x}", b));
00019516:         if (i + 1) % 16 == 0 {
00019517:             hex.push('\n');
00019518:         } else if (i + 1) % 8 == 0 {
00019519:             hex.push(' ');
00019520:         }
00019521:     }
00019522: 
00019523:     // Try to read class name via il2cpp class API
00019524:     let mut class_name = String::new();
00019525:     if !ds_class_ptr.is_null() {
00019526:         let get_name_fn = resolve_il2cpp_symbol("il2cpp_class_get_name");
00019527:         if !get_name_fn.is_null() {
00019528:             let fn_ptr: unsafe extern "C" fn(*mut c_void) -> *const u8 =
00019529:                 std::mem::transmute(get_name_fn);
00019530:             let name_ptr = fn_ptr(ds_class_ptr);
00019531:             if !name_ptr.is_null() {
00019532:                 let cstr = std::ffi::CStr::from_ptr(name_ptr as *const c_char);
00019533:                 class_name = cstr.to_string_lossy().into_owned();
00019534:             }
00019535:         }
00019536:     }
00019537: 
00019538:     format!(
00019539:         r#"{{"ds_ptr":"0x{:x}","ds_class":"0x{:x}","class_name":"{}","hex_dump":"{}"}}"#,
00019540:         ramen_ds_obj as usize, ds_class_ptr as usize, class_name, hex
00019541:     )
00019542: }
00019543: 
00019544: /// /inherit/compat — Inheritance compatibility calculation
00019545: /// Shows exact compatibility values (not just ○△×), split by parent gender,
00019546: /// and detects target race overlap
00019547: /// Data sources:
00019548: ///   - SuccessionCharaInfo (parent chara IDs)
00019549: ///   - SuccessionRelationMember + SuccessionRelation (compatibility data)
00019550: ///   - mdb succession_relation tables
00019551: ///   - SingleModeTargetRace (current target races)
00019552: unsafe fn read_inherit_compat() -> String {
00019553:     if API.is_null() {
00019554:         return r#"{"error":"api_null"}"#.to_string();
00019555:     }
00019556:     let image = match get_image() {
00019557:         img if !img.is_null() => img,
00019558:         _ => return r#"{"error":"image_null"}"#.to_string(),
00019559:     };
00019560: 
00019561:     let wdm_class = find_class(
00019562:         image,
00019563:         to_cstr("Gallop").as_ptr(),
00019564:         to_cstr("WorkDataManager").as_ptr(),
00019565:     );
00019566:     if wdm_class.is_null() {
00019567:         return r#"{"error":"no_wdm"}"#.to_string();
00019568:     }
00019569:     let wdm_inst = get_singleton(wdm_class);
00019570:     if wdm_inst.is_null() {
00019571:         return r#"{"error":"no_wdm_inst"}"#.to_string();
00019572:     }
00019573:     log_predict_step("P:wdm");
00019574: 
00019575:     let sm_class = find_class(
00019576:         image,
00019577:         to_cstr("Gallop").as_ptr(),
00019578:         to_cstr("WorkSingleModeData").as_ptr(),
00019579:     );
00019580:     let sm_obj = call_getter_ref(wdm_class, wdm_inst, "get_SingleMode");
00019581:     if sm_obj.is_null() {
00019582:         return r#"{"error":"no_sm"}"#.to_string();
00019583:     }
00019584: 
00019585:     let chara_class = find_class(
00019586:         image,
00019587:         to_cstr("Gallop").as_ptr(),
00019588:         to_cstr("WorkSingleModeCharaData").as_ptr(),
00019589:     );
00019590:     let chara_obj = call_getter_ref(sm_class, sm_obj, "get_Character");
00019591:     if chara_obj.is_null() {
00019592:         return r#"{"error":"no_chara"}"#.to_string();
00019593:     }
00019594: 
00019595:     // 1. Read succession parent info
00019596:     // WorkSingleModeCharaData.SuccessionTrainedCharaInfoFirst (offset 0x48)
00019597:     // WorkSingleModeCharaData.SuccessionTrainedCharaInfoSecond (offset 0x50)
00019598:     let sci_class = find_class_by_short_name(image, "SuccessionCharaInfo");
00019599:     let first_sci = call_getter_ref(
00019600:         chara_class,
00019601:         chara_obj,
00019602:         "get_SuccessionTrainedCharaInfoFirst",
00019603:     );
00019604:     let second_sci = call_getter_ref(
00019605:         chara_class,
00019606:         chara_obj,
00019607:         "get_SuccessionTrainedCharaInfoSecond",
00019608:     );
00019609: 
00019610:     let mut first_chara_id: i32 = -1;
00019611:     let mut second_chara_id: i32 = -1;
00019612:     if !first_sci.is_null() && !sci_class.is_null() {
00019613:         first_chara_id = call_getter_int(sci_class, first_sci, "get_TrainedCharaId");
00019614:     }
00019615:     if !second_sci.is_null() && !sci_class.is_null() {
00019616:         second_chara_id = call_getter_int(sci_class, second_sci, "get_TrainedCharaId");
00019617:     }
00019618: 
00019619:     // 2. Read SuccessionFactor (offset 0x448 on CharaData) — factor count for compatibility
00019620:     let factor_arr = call_getter_on_instance(chara_class, chara_obj, "get_SuccessionFactor");
00019621:     let mut factor_count: i32 = 0;
00019622:     if !factor_arr.is_null() {
00019623:         let fb = factor_arr as *const u8;
00019624:         factor_count =
00019625:             std::ptr::read_unaligned::<usize>(fb.add(IL2CPP_LIST_COUNT_OFF) as *const usize) as i32;
00019626:     }
00019627: 
00019628:     // 3. Read relation data from mdb
00019629:     let mut relations_json: Vec<String> = Vec::new();
00019630:     let mut relation_members_json: Vec<String> = Vec::new();
00019631:     let mut relation_ranks_json: Vec<String> = Vec::new();
00019632: 
00019633:     if let Some(mdb_path) = find_mdb_path() {
00019634:         if let Ok(conn) = Connection::open_with_flags(&mdb_path, OpenFlags::SQLITE_OPEN_READ_ONLY) {
00019635:             // succession_relation: type + point pairs
00019636:             if let Ok(mut stmt) = conn.prepare("SELECT relation_type, relation_point FROM succession_relation ORDER BY relation_type") {
00019637:                 let rows: Vec<String> = stmt.query_map([], |row| {
00019638:                     Ok(format!(r#"{{"type":{},"point":{}}}"#,
00019639:                         row.get::<_, i32>(0).unwrap_or(0),
00019640:                         row.get::<_, i32>(1).unwrap_or(0)))
00019641:                 }).unwrap().filter_map(|r| r.ok()).collect();
00019642:                 relations_json = rows;
00019643:             }
00019644: 
00019645:             // succession_relation_member: id + type + chara_id
00019646:             if let Ok(mut stmt) = conn.prepare(
00019647:                 "SELECT id, relation_type, chara_id FROM succession_relation_member ORDER BY id",
00019648:             ) {
00019649:                 let rows: Vec<String> = stmt
00019650:                     .query_map([], |row| {
00019651:                         Ok(format!(
00019652:                             r#"{{"id":{},"type":{},"chara_id":{}}}"#,
00019653:                             row.get::<_, i32>(0).unwrap_or(0),
00019654:                             row.get::<_, i32>(1).unwrap_or(0),
00019655:                             row.get::<_, i32>(2).unwrap_or(0)
00019656:                         ))
00019657:                     })
00019658:                     .unwrap()
00019659:                     .filter_map(|r| r.ok())
00019660:                     .collect();
00019661:                 relation_members_json = rows;
00019662:             }
00019663: 
00019664:             // succession_relation_rank: rank + min + max
00019665:             if let Ok(mut stmt) = conn.prepare("SELECT relation_rank, rank_value_min, rank_value_max FROM succession_relation_rank ORDER BY relation_rank") {
00019666:                 let rows: Vec<String> = stmt.query_map([], |row| {
00019667:                     Ok(format!(r#"{{"rank":{},"min":{},"max":{}}}"#,
00019668:                         row.get::<_, i32>(0).unwrap_or(0),
00019669:                         row.get::<_, i32>(1).unwrap_or(0),
00019670:                         row.get::<_, i32>(2).unwrap_or(0)))
00019671:                 }).unwrap().filter_map(|r| r.ok()).collect();
00019672:                 relation_ranks_json = rows;
00019673:             }
00019674: 
00019675:             drop(conn);
00019676:         }
00019677:     }
00019678: 
00019679:     // 4. Read target races for overlap detection
00019680:     let mut target_races_json: Vec<String> = Vec::new();
00019681:     let tr_arr = call_getter_on_instance(chara_class, chara_obj, "get_TargetRaceArray");
00019682:     if !tr_arr.is_null() {
00019683:         let trb = tr_arr as *const u8;
00019684:         let trl = std::ptr::read_unaligned::<usize>(trb.add(IL2CPP_LIST_COUNT_OFF) as *const usize);
00019685:         if trl > 0 && trl < 50 {
00019686:             for ti in 0..trl {
00019687:                 let tp = std::ptr::read_unaligned::<*mut c_void>(
00019688:                     trb.add(IL2CPP_LIST_ITEMS_OFF + ti * IL2CPP_LIST_ITEM_SIZE)
00019689:                         as *const *mut c_void,
00019690:                 );
00019691:                 if tp.is_null() {
00019692:                     continue;
00019693:                 }
00019694:                 // TargetRace: targetId at offset 0x10, evaluation at 0x14
00019695:                 let bytes = tp as *const u8;
00019696:                 let tid = std::ptr::read_unaligned::<i32>(
00019697:                     bytes.add(IL2CPP_TARGET_RACE_ID_OFF) as *const i32
00019698:                 );
00019699:                 let teval = std::ptr::read_unaligned::<i32>(
00019700:                     bytes.add(IL2CPP_TARGET_RACE_EVAL_OFF) as *const i32
00019701:                 );
00019702:                 target_races_json
00019703:                     .push(format!(r#"{{"target_id":{},"evaluation":{}}}"#, tid, teval));
00019704:             }
00019705:         }
00019706:     }
00019707: 
00019708:     // 5. Read route_race from mdb for race name resolution
00019709:     let mut race_names_json: Vec<String> = Vec::new();
00019710:     if let Some(mdb_path) = find_mdb_path() {
00019711:         if let Ok(conn) = Connection::open_with_flags(&mdb_path, OpenFlags::SQLITE_OPEN_READ_ONLY) {
00019712:             if let Ok(mut stmt) = conn.prepare(
00019713:                 "SELECT id, race_id, race_grade FROM single_mode_route_race ORDER BY id LIMIT 200",
00019714:             ) {
00019715:                 let rows: Vec<String> = stmt
00019716:                     .query_map([], |row| {
00019717:                         Ok(format!(
00019718:                             r#"{{"id":{},"race_id":{},"grade":{}}}"#,
00019719:                             row.get::<_, i32>(0).unwrap_or(0),
00019720:                             row.get::<_, i32>(1).unwrap_or(0),
00019721:                             row.get::<_, i32>(2).unwrap_or(0)
00019722:                         ))
00019723:                     })
00019724:                     .unwrap()
00019725:                     .filter_map(|r| r.ok())
00019726:                     .collect();
00019727:                 race_names_json = rows;
00019728:             }
00019729:             drop(conn);
00019730:         }
00019731:     }
00019732: 
00019733:     format!(
00019734:         r#"{{"version":"3.22.91","parents":{{"first_chara_id":{},"second_chara_id":{}}},"factor_count":{},"relations":[{}],"relation_members":[{}],"relation_ranks":[{}],"target_races":[{}],"route_races":[{}]}}"#,
00019735:         first_chara_id,
00019736:         second_chara_id,
00019737:         factor_count,
00019738:         relations_json.join(","),
00019739:         relation_members_json.join(","),
00019740:         relation_ranks_json.join(","),
00019741:         target_races_json.join(","),
00019742:         race_names_json.join(",")
00019743:     )
00019744: }
00019745: 
00019746: /// /saddle-analysis — WinSaddleAnalyzer
00019747: /// Reads current trained chara's win saddles from game memory,
00019748: /// cross-references with MDB to map each G1 win to its relation_group_id,
00019749: /// then outputs which relation groups (compatibility bonuses) the chara has earned.
00019750: /// Also reads parent candidates' win saddles for cross-comparison.
00019751: unsafe fn read_win_saddle_analysis() -> String {
00019752:     if API.is_null() {
00019753:         return r#"{"error":"api_null"}"#.to_string();
00019754:     }
00019755:     let api = &*API;
00019756:     let image = match get_image() {
00019757:         img if !img.is_null() => img,
00019758:         _ => return r#"{"error":"image_null"}"#.to_string(),
00019759:     };
00019760: 
00019761:     // 1. Get WorkSingleModeData
00019762:     let wdm_class = find_class(
00019763:         image,
00019764:         to_cstr("Gallop").as_ptr(),
00019765:         to_cstr("WorkDataManager").as_ptr(),
```

## lines 19828..20148

```rust
00019828:                     read_il2cpp_string(n)
00019829:                 }
00019830:             } else {
00019831:                 String::new()
00019832:             };
00019833: 
00019834:             // Call get_Type
00019835:             let stype = if !saddle_class.is_null() {
00019836:                 call_getter_int(saddle_class, elem_ptr as *mut c_void, "get_Type")
00019837:             } else {
00019838:                 -1
00019839:             };
00019840: 
00019841:             // Call IsRelationBonusWinSaddle (returns bool)
00019842:             let is_relation_bonus = if !saddle_class.is_null() {
00019843:                 let get_method_fn = resolve_il2cpp_symbol("il2cpp_class_get_method_from_name");
00019844:                 let invoke_fn = resolve_il2cpp_symbol("il2cpp_runtime_invoke");
00019845:                 if !get_method_fn.is_null() && !invoke_fn.is_null() {
00019846:                     type FnGetMethod =
00019847:                         unsafe extern "C" fn(*mut c_void, *const c_char, i32) -> *mut c_void;
00019848:                     type FnInvoke = unsafe extern "C" fn(
00019849:                         *mut c_void,
00019850:                         *mut c_void,
00019851:                         *mut c_void,
00019852:                         *mut c_void,
00019853:                     ) -> *mut c_void;
00019854:                     let f: FnGetMethod = std::mem::transmute(get_method_fn);
00019855:                     let inv: FnInvoke = std::mem::transmute(invoke_fn);
00019856:                     let m = f(
00019857:                         saddle_class,
00019858:                         to_cstr("IsRelationBonusWinSaddle").as_ptr(),
00019859:                         0,
00019860:                     );
00019861:                     if !m.is_null() {
00019862:                         let ret = inv(
00019863:                             m,
00019864:                             elem_ptr as *mut c_void,
00019865:                             std::ptr::null_mut(),
00019866:                             std::ptr::null_mut(),
00019867:                         );
00019868:                         ret as i32 != 0
00019869:                     } else {
00019870:                         false
00019871:                     }
00019872:                 } else {
00019873:                     false
00019874:                 }
00019875:             } else {
00019876:                 false
00019877:             };
00019878: 
00019879:             // Call GetRelationPoint
00019880:             let relation_point = if !saddle_class.is_null() {
00019881:                 let get_method_fn = resolve_il2cpp_symbol("il2cpp_class_get_method_from_name");
00019882:                 let invoke_fn = resolve_il2cpp_symbol("il2cpp_runtime_invoke");
00019883:                 if !get_method_fn.is_null() && !invoke_fn.is_null() {
00019884:                     type FnGetMethod =
00019885:                         unsafe extern "C" fn(*mut c_void, *const c_char, i32) -> *mut c_void;
00019886:                     type FnInvoke = unsafe extern "C" fn(
00019887:                         *mut c_void,
00019888:                         *mut c_void,
00019889:                         *mut c_void,
00019890:                         *mut c_void,
00019891:                     ) -> *mut c_void;
00019892:                     let f: FnGetMethod = std::mem::transmute(get_method_fn);
00019893:                     let inv: FnInvoke = std::mem::transmute(invoke_fn);
00019894:                     let m = f(saddle_class, to_cstr("GetRelationPoint").as_ptr(), 0);
00019895:                     if !m.is_null() {
00019896:                         let ret = inv(
00019897:                             m,
00019898:                             elem_ptr as *mut c_void,
00019899:                             std::ptr::null_mut(),
00019900:                             std::ptr::null_mut(),
00019901:                         );
00019902:                         if !ret.is_null() {
00019903:                             std::ptr::read_unaligned::<i32>(ret as *const i32)
00019904:                         } else {
00019905:                             0
00019906:                         }
00019907:                     } else {
00019908:                         0
00019909:                     }
00019910:                 } else {
00019911:                     0
00019912:                 }
00019913:             } else {
00019914:                 0
00019915:             };
00019916: 
00019917:             saddle_entries.push(format!(
00019918:                 r#"{{"index":{},"name":"{}","type":{},"is_relation_bonus":{},"relation_point":{}}}"#,
00019919:                 i,
00019920:                 json_escape(&name),
00019921:                 stype,
00019922:                 is_relation_bonus,
00019923:                 relation_point,
00019924:             ));
00019925:         }
00019926:     }
00019927: 
00019928:     // 4. Read parent candidates' WinSaddleArray via SuccessionCharaData
00019929:     // Get WorkSingleModeCharaData → SuccessionTrainedCharaInfo
00019930:     let chara_class = find_class(
00019931:         image,
00019932:         to_cstr("Gallop").as_ptr(),
00019933:         to_cstr("WorkSingleModeCharaData").as_ptr(),
00019934:     );
00019935:     let chara_obj = if !chara_class.is_null() {
00019936:         call_getter_ref(wdm_class, wdm, "get_WorkSingleModeCharaData")
00019937:     } else {
00019938:         std::ptr::null_mut()
00019939:     };
00019940: 
00019941:     let mut parent_saddles_json: Vec<String> = Vec::new();
00019942:     if !chara_obj.is_null() && !chara_class.is_null() {
00019943:         let sci_class = find_class(
00019944:             image,
00019945:             to_cstr("Gallop").as_ptr(),
00019946:             to_cstr("SuccessionCharaInfo").as_ptr(),
00019947:         );
00019948: 
00019949:         for (label, getter_name) in [
00019950:             ("parent1", "get_SuccessionTrainedCharaInfoFirst"),
00019951:             ("parent2", "get_SuccessionTrainedCharaInfoSecond"),
00019952:         ] {
00019953:             let sci = call_getter_ref(chara_class, chara_obj, getter_name);
00019954:             if sci.is_null() {
00019955:                 continue;
00019956:             }
00019957: 
00019958:             let chara_id = if !sci_class.is_null() {
00019959:                 call_getter_int(sci_class, sci, "get_TrainedCharaId")
00019960:             } else {
00019961:                 0
00019962:             };
00019963: 
00019964:             // Try to get WinSaddleArray from SuccessionCharaInfo
00019965:             let p_saddles = call_getter_on_instance(sci_class, sci, "get_WinSaddleArray");
00019966:             let mut p_count = 0i32;
00019967:             let mut p_entries: Vec<String> = Vec::new();
00019968: 
00019969:             if !p_saddles.is_null() {
00019970:                 let pb = p_saddles as *const u8;
00019971:                 let p_items = std::ptr::read_unaligned::<usize>(pb.add(0x10) as *const usize);
00019972:                 p_count = std::ptr::read_unaligned::<i32>(pb.add(0x18) as *const i32);
00019973: 
00019974:                 let saddle_class = find_class(
00019975:                     image,
00019976:                     to_cstr("Gallop").as_ptr(),
00019977:                     to_cstr("SingleModeWinsSaddle").as_ptr(),
00019978:                 );
00019979: 
00019980:                 for i in 0..p_count.min(30) {
00019981:                     let elem_ptr = std::ptr::read_unaligned::<usize>(
00019982:                         (p_items + (i as usize) * std::mem::size_of::<usize>()) as *const usize,
00019983:                     );
00019984:                     if elem_ptr == 0 {
00019985:                         continue;
00019986:                     }
00019987:                     let name = if !saddle_class.is_null() {
00019988:                         let n =
00019989:                             call_getter_string(saddle_class, elem_ptr as *mut c_void, "get_Name");
00019990:                         if n.is_null() {
00019991:                             String::new()
00019992:                         } else {
00019993:                             read_il2cpp_string(n)
00019994:                         }
00019995:                     } else {
00019996:                         String::new()
00019997:                     };
00019998:                     let stype = if !saddle_class.is_null() {
00019999:                         call_getter_int(saddle_class, elem_ptr as *mut c_void, "get_Type")
00020000:                     } else {
00020001:                         -1
00020002:                     };
00020003:                     p_entries.push(format!(
00020004:                         r#"{{"name":"{}","type":{}}}"#,
00020005:                         json_escape(&name),
00020006:                         stype,
00020007:                     ));
00020008:                 }
00020009:             }
00020010: 
00020011:             parent_saddles_json.push(format!(
00020012:                 r#"{{"label":"{}","chara_id":{},"saddle_count":{},"saddles":[{}]}}"#,
00020013:                 label,
00020014:                 chara_id,
00020015:                 p_count,
00020016:                 p_entries.join(","),
00020017:             ));
00020018:         }
00020019:     }
00020020: 
00020021:     // 5. Cross-reference with MDB for relation_group_id mapping
00020022:     let mut mdb_saddle_map_json: Vec<String> = Vec::new();
00020023:     let mut relation_groups_json: Vec<String> = Vec::new();
00020024: 
00020025:     if let Some(mdb_path) = find_mdb_path() {
00020026:         if let Ok(conn) = Connection::open_with_flags(&mdb_path, OpenFlags::SQLITE_OPEN_READ_ONLY) {
00020027:             // Map: win_saddle entries from MDB with their relation_group_id
00020028:             if let Ok(mut stmt) = conn.prepare(
00020029:                 "SELECT id, relation_group_id, condition, win_saddle_type, race_instance_id_1, race_instance_id_2 FROM single_mode_wins_saddle WHERE win_saddle_type=3 AND relation_group_id > 0 ORDER BY relation_group_id"
00020030:             ) {
00020031:                 let rows: Vec<String> = stmt.query_map([], |row| {
00020032:                     Ok(format!(
00020033:                         r#"{{"id":{},"rel_group":{},"cond":{},"type":{},"race1":{},"race2":{}}}"#,
00020034:                         row.get::<_, i32>(0).unwrap_or(0),
00020035:                         row.get::<_, i32>(1).unwrap_or(0),
00020036:                         row.get::<_, i32>(2).unwrap_or(0),
00020037:                         row.get::<_, i32>(3).unwrap_or(0),
00020038:                         row.get::<_, i32>(4).unwrap_or(0),
00020039:                         row.get::<_, i32>(5).unwrap_or(0),
00020040:                     ))
00020041:                 }).unwrap().filter_map(|r| r.ok()).collect();
00020042:                 mdb_saddle_map_json = rows;
00020043:             }
00020044: 
00020045:             // succession_relation: check which relation_types give points
00020046:             // The G1 win groups are type 1-34 (1pt each)
00020047:             if let Ok(mut stmt) = conn.prepare(
00020048:                 "SELECT relation_type, relation_point FROM succession_relation WHERE relation_type BETWEEN 1 AND 200 ORDER BY relation_type"
00020049:             ) {
00020050:                 let rows: Vec<String> = stmt.query_map([], |row| {
00020051:                     Ok(format!(
00020052:                         r#"{{"type":{},"point":{}}}"#,
00020053:                         row.get::<_, i32>(0).unwrap_or(0),
00020054:                         row.get::<_, i32>(1).unwrap_or(0),
00020055:                     ))
00020056:                 }).unwrap().filter_map(|r| r.ok()).collect();
00020057:                 relation_groups_json = rows;
00020058:             }
00020059: 
00020060:             // Get race names for G1 race_instance_ids
00020061:             // race_instance_id 100301 → race_id → text_data category=32
00020062:         }
00020063:     }
00020064: 
00020065:     // 6. Build output
00020066:     format!(
00020067:         r#"{{"ok":true,"total_races":{},"win_count":{},"saddle_count":{},"win_saddles":[{}],"parent_saddles":[{}],"mdb_saddle_map":[{}],"relation_groups":[{}]}}"#,
00020068:         total_races,
00020069:         win_count,
00020070:         saddle_count,
00020071:         saddle_entries.join(","),
00020072:         parent_saddles_json.join(","),
00020073:         mdb_saddle_map_json.join(","),
00020074:         relation_groups_json.join(","),
00020075:     )
00020076: }
00020077: 
00020078: /// Returns current turn info + history from training log
00020079: /// Data sources:
00020080: ///   - WorkSingleModeData: Month, Half, Turn
00020081: ///   - WorkSingleModeCharaData: all stats, motivation
00020082: ///   - SingleModeTurn (mdb): turn config (year, period, training set)
00020083: ///   - Training log snapshots
00020084: unsafe fn read_turn_log() -> String {
00020085:     if API.is_null() {
00020086:         return r#"{"error":"api_null"}"#.to_string();
00020087:     }
00020088:     let image = match get_image() {
00020089:         img if !img.is_null() => img,
00020090:         _ => return r#"{"error":"image_null"}"#.to_string(),
00020091:     };
00020092: 
00020093:     let wdm_class = find_class(
00020094:         image,
00020095:         to_cstr("Gallop").as_ptr(),
00020096:         to_cstr("WorkDataManager").as_ptr(),
00020097:     );
00020098:     if wdm_class.is_null() {
00020099:         return r#"{"error":"no_wdm"}"#.to_string();
00020100:     }
00020101:     let wdm_inst = get_singleton(wdm_class);
00020102:     if wdm_inst.is_null() {
00020103:         return r#"{"error":"no_wdm_inst"}"#.to_string();
00020104:     }
00020105:     log_predict_step("P:wdm");
00020106: 
00020107:     let sm_class = find_class(
00020108:         image,
00020109:         to_cstr("Gallop").as_ptr(),
00020110:         to_cstr("WorkSingleModeData").as_ptr(),
00020111:     );
00020112:     let sm_obj = call_getter_ref(wdm_class, wdm_inst, "get_SingleMode");
00020113:     if sm_obj.is_null() {
00020114:         return r#"{"error":"no_sm"}"#.to_string();
00020115:     }
00020116: 
00020117:     let chara_class = find_class(
00020118:         image,
00020119:         to_cstr("Gallop").as_ptr(),
00020120:         to_cstr("WorkSingleModeCharaData").as_ptr(),
00020121:     );
00020122:     let chara_obj = call_getter_ref(sm_class, sm_obj, "get_Character");
00020123:     if chara_obj.is_null() {
00020124:         return r#"{"error":"no_chara"}"#.to_string();
00020125:     }
00020126: 
00020127:     // Current state
00020128:     let mon = call_getter_int(sm_class, sm_obj, "get_Month");
00020129:     let half = call_getter_int(sm_class, sm_obj, "get_Half");
00020130:     let sid = call_getter_int(chara_class, chara_obj, "get_ScenarioId");
00020131:     let spd = call_getter_int(chara_class, chara_obj, "get_Speed");
00020132:     let sta = call_getter_int(chara_class, chara_obj, "get_Stamina");
00020133:     let pow_ = call_getter_int(chara_class, chara_obj, "get_Power");
00020134:     let gut = call_getter_int(chara_class, chara_obj, "get_Guts");
00020135:     let wiz = call_getter_int(chara_class, chara_obj, "get_Wiz");
00020136:     let vit = call_getter_int(chara_class, chara_obj, "get_Hp");
00020137:     let mvit = call_getter_int(chara_class, chara_obj, "get_MaxHp");
00020138:     let mot = call_getter_int(chara_class, chara_obj, "get_Motivation");
00020139:     let spt = call_getter_obscured_int(chara_class, chara_obj, "get_SkillPoint");
00020140:     let fan = call_getter_int(chara_class, chara_obj, "get_FanCount");
00020141: 
00020142:     // Turn config from mdb
00020143:     let mut turn_config_json = String::new();
00020144:     if let Some(mdb_path) = find_mdb_path() {
00020145:         if let Ok(conn) = Connection::open_with_flags(&mdb_path, OpenFlags::SQLITE_OPEN_READ_ONLY) {
00020146:             if let Ok(mut stmt) = conn.prepare("SELECT id, turn, year, month, half, period, unique_command, training_set_id, race_entry_type FROM single_mode_turn ORDER BY id") {
00020147:                 let rows: Vec<String> = stmt.query_map([], |row| {
00020148:                     Ok(format!(
```
