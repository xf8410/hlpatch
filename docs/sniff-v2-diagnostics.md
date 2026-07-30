# Sniff v2 patch diagnostics

source_bytes=889313

- globals_literal: `1`
- pending_literal: `1`
- toggle_marker: `1`
- clear_pair_regex: `1`
- response_regex: `1`
- request_literal: `1`

## Exact hit lines

### `SNIFF_MAX` hits=3
```rust
110: // SniffEntry: (id, url, headers_json, body)
111: static mut SNIFF_REQUESTS: Vec<(u64, String, String, Vec<u8>)> = Vec::new();
112: static mut SNIFF_RESPONSES: Vec<(u64, Vec<u8>)> = Vec::new();
113: static SNIFF_MAX: usize = 20;
114: static SNIFF_REQ_ID: AtomicU64 = AtomicU64::new(0);
115: static mut PENDING_URL: String = String::new();
116: static mut PENDING_HEADERS: Vec<(String, String)> = Vec::new();
117: static mut PENDING_REQ_ID: u64 = 0;
118: // CompressRequest/DecompressResponse/Post hook addresses (via Interceptor API)
119: static mut COMPRESS_REQUEST_ADDR: usize = 0;
120: static mut DECOMPRESS_RESPONSE_ADDR: usize = 0;
121: static mut POST_ADDR: usize = 0;
122: // Pending request body parking (CompressRequest → Post matching)
```
```rust
9017:                 let _lock = SNIFF_MUTEX.lock();
9018:                 let rid = PENDING_REQ_ID;
9019:                 SNIFF_RESPONSES.push((rid, bytes));
9020:                 if SNIFF_RESPONSES.len() > SNIFF_MAX {
9021:                     SNIFF_RESPONSES.remove(0);
9022:                 }
9023:             }
9024:         }
9025:         decompressed
9026:     }
9027: }
9028: 
9029: // ★ v3.23.3: Hook handler for WWWRequest.Post(this, url, postData, headers)
```
```rust
9072:                 let url_str = game_url.clone().unwrap_or_default();
9073:                 let _lock = SNIFF_MUTEX.lock();
9074:                 SNIFF_REQUESTS.push((rid, url_str, headers_json, body));
9075:                 if SNIFF_REQUESTS.len() > SNIFF_MAX {
9076:                     SNIFF_REQUESTS.remove(0);
9077:                 }
9078:             }
9079:             PENDING_URL = game_url.clone().unwrap_or_default();
9080:             PENDING_HEADERS = req_headers.clone();
9081:         }
9082: 
9083:         let _ = this;
9084:         original(this, url, post_data, headers)
```
### `SNIFF_REQUESTS.clear` hits=1
```rust
7187:     } else if path == "/api/sniff/clear" {
7188:         let _lock = SNIFF_MUTEX.lock();
7189:         unsafe {
7190:             SNIFF_REQUESTS.clear();
7191:             SNIFF_RESPONSES.clear();
7192:         }
7193:         r#"{"ok":true}"#.to_string()
7194:     } else if path.starts_with("/debug/hooklog") {
7195:         // ★ v3.24.40/42: last HOOK_LOG_MAX lines, optional ?filter=substr
7196:         let filter = parse_query(&full_uri, "filter");
7197:         let entries: Vec<String> = match HOOK_LOG.lock() {
7198:             Ok(g) => g.iter()
7199:                 .filter(|l| filter.is_empty() || l.contains(&filter))
```
### `let rid = PENDING_REQ_ID` hits=1
```rust
9015:         if SNIFF_ENABLED.load(Ordering::Relaxed) {
9016:             if !bytes.is_empty() {
9017:                 let _lock = SNIFF_MUTEX.lock();
9018:                 let rid = PENDING_REQ_ID;
9019:                 SNIFF_RESPONSES.push((rid, bytes));
9020:                 if SNIFF_RESPONSES.len() > SNIFF_MAX {
9021:                     SNIFF_RESPONSES.remove(0);
9022:                 }
9023:             }
9024:         }
9025:         decompressed
9026:     }
9027: }
```
### `Try to match parked request body` hits=1
```rust
9066:         if SNIFF_ENABLED.load(Ordering::Relaxed) {
9067:             let rid = SNIFF_REQ_ID.fetch_add(1, Ordering::Relaxed);
9068:             PENDING_REQ_ID = rid;
9069:             // Try to match parked request body
9070:             if let Some(body) = PENDING_REQ_BODY.take() {
9071:                 let headers_json = format_headers_json(&req_headers);
9072:                 let url_str = game_url.clone().unwrap_or_default();
9073:                 let _lock = SNIFF_MUTEX.lock();
9074:                 SNIFF_REQUESTS.push((rid, url_str, headers_json, body));
9075:                 if SNIFF_REQUESTS.len() > SNIFF_MAX {
9076:                     SNIFF_REQUESTS.remove(0);
9077:                 }
9078:             }
```
### `/api/sniff/toggle` hits=4
```rust
6978:             "/", "/health", "/status", "/config", "/config.html",
6979:             "/update", "/update/status",
6980:             "/debug/hookdiag", "/debug/hooklog", "/debug/crashlog", "/debug/upload",
6981:             "/api/sniff", "/api/sniff/diag", "/api/sniff/toggle", "/api/sniff/clear",
6982:             "/api/event/choices", "/api/event/observations", "/api/event/observations/clear", "/api/event/clear",
6983:             "/action/latest", "/seed/history", "/seed/stats",
6984:             "/log", "/carddb", "/skilldata",
6985:             "/debug/table", "/debug/push_table", "/debug/download_table",
6986:             "/debug/mdb_all_tables", "/debug/mdb_schema_dump",
6987:         ];
6988:         const BOOT_SAFE_PREFIX: &[&str] = &[
6989:             "/mdb", "/debug/resource_", "/debug/private_file", "/debug/mem_scan_sqlite", "/debug/mem_scan_zdict", "/debug/mem_scan_hex", "/debug/file_scan_hex", "/debug/maps_list", "/debug/file_dl", "/debug/file_range_hex",
6990:         ];
```
```rust
7021:         && DL_ALLOWED.iter().any(|p| path == *p);
7022: 
7023:     let body = if path == "/" || path == "/health" {
7024:         format!(r#"{{"status":"ok","version":"{}","endpoints":["/summary","/data","/scenario","/debug/rameninfo","/debug/laststep","/event/recommend","/inherit/compat","/saddle-analysis","/log/turn","/debug/params","/debug/breeders","/debug/cmdinfo","/debug/training_partners","/debug/crashlog","/debug/upload","/debug/dumpclass","/debug/storydata","/debug/ramenfields","/debug/gauge","/debug/gauge2","/debug/ramengains","/debug/paramsincdec","/debug/training_seed","/debug/training_log","/debug/training_log_dl","/update","/update/status","/debug/all","/debug/unique_skills","/debug/mdb_all_tables","/debug/mdb_schema_dump","/debug/hint_gain","/debug/sc_effect","/debug/unique_detail","/debug/table","/debug/push_table","/debug/download_table","/mdb","/carddb","/skilldata","/hall","/saddles","/saddles-dl","/log","/status","/health","/mdb/schema","/mdb/search","/mdb/raw","/mdb/dl_batch","/il2cpp/dump","/il2cpp/call","/il2cpp/tree","/il2cpp/field","/il2cpp/classes","/il2cpp/static","/il2cpp/methods","/il2cpp/disassemble","/il2cpp/disassemble_dl","/il2cpp/disassemble_addr","/il2cpp/disassemble_addr_dl","/il2cpp/dump_all_methods","/il2cpp/dump_all_methods_dl","/il2cpp/search_float","/il2cpp/search_float_dl","/il2cpp/search_int","/il2cpp/search_int_dl","/il2cpp/search_methods","/il2cpp/search_methods_dl","/il2cpp/read_mem","/il2cpp/read_mem_dl","/training/result","/api/sniff","/api/sniff/toggle","/api/sniff/clear","/api/sniff/diag","/api/event/choices","/api/event/clear","/debug/hooklog","/debug/hookdiag","/debug/resource_meta_key","/debug/resource_db_keys","/debug/resource_reads","/debug/mem_scan_sqlite","/debug/meta_dump","/action/latest","/seed/history","/seed/stats","/debug/ramen_planner_state","/debug/ramen_participants","/debug/ramen_transition","/debug/ramen_dataset_path","/debug/ramen_formula_targets","/debug/event_reward_targets", "/debug/resource_storage","/debug/resource_meta_schema","/debug/resource_meta_probe", "/debug/resource_crypto_symbols","/debug/resource_meta_dl","/debug/resource_file_dl","/debug/private_file_inventory","/debug/private_file_dl"]}}"#, PLUGIN_VERSION)
7025:     } else if path == "/scan" {
7026:         unsafe { scan_il2cpp_classes() }
7027:     } else if path == "/data" {
7028:         let result = unsafe { read_training_data() };
7029:         unsafe {
7030:             log_snapshot("data", &result);
7031:         }
7032:         result
7033:     } else if path == "/status" {
```
```rust
7170:                 _ => "Unknown",
7171:             }
7172:         )
7173:     } else if path == "/api/sniff/toggle" {
7174:         // ★ v3.24.40: lazy retry for fallback-mode installs.
7175:         unsafe {
7176:             install_api_sniff_hooks();
7177:         }
7178:         let new_val = !SNIFF_ENABLED.load(Ordering::Relaxed);
7179:         SNIFF_ENABLED.store(new_val, Ordering::Relaxed);
7180:         let req_hooked = unsafe { COMPRESS_REQUEST_ADDR != 0 };
7181:         let resp_hooked = unsafe { DECOMPRESS_RESPONSE_ADDR != 0 };
7182:         let post_hooked = unsafe { POST_ADDR != 0 };
```
```rust
8293:         }
8294:     } else {
8295:         format!(
8296:             r#"{{"error":"not_found","path":"{}","available":["/scan","/data","/status","/health","/scenario","/debug/upload","/debug/rameninfo","/debug/laststep","/event/recommend","/inherit/compat","/saddle-analysis","/log/turn","/log","/debug/params","/fields","/methods","/singletons","/find_method","/classes","/carddb","/skilldata","/hall","/debug/breeders","/debug/cmdinfo","/debug/training_partners","/debug/ramengains","/debug/paramsincdec","/debug/training_seed","/debug/training_log","/debug/training_log_dl","/update","/update/status","/debug/dumpclass","/debug/storydata","/debug/ramenfields","/debug/all","/mdb","/debug/push_table","/debug/download_table","/classes/search/keyword","/mdb/schema","/mdb/search","/mdb/raw","/mdb/dl_batch","/il2cpp/dump","/il2cpp/call","/il2cpp/tree","/il2cpp/field","/il2cpp/classes","/il2cpp/static","/il2cpp/methods","/il2cpp/search_float","/il2cpp/search_float_dl","/il2cpp/search_int","/il2cpp/search_int_dl","/il2cpp/search_methods","/il2cpp/search_methods_dl","/il2cpp/search_methods_page","/il2cpp/read_mem","/il2cpp/read_mem_dl","/training/result","/api/sniff","/api/sniff/toggle","/api/sniff/clear","/api/sniff/diag","/api/event/choices","/api/event/clear"]}}"#,
8297:             path
8298:         )
8299:     };
8300: 
8301:     save_endpoint_log(&path, &body);
8302: 
8303:     if body.starts_with("__MDB_BINARY__") {
8304:         // v3.22.51: Serve raw mdb file as binary response
8305:         let mdb_path = &body[14..]; // skip "__MDB_BINARY__"
```