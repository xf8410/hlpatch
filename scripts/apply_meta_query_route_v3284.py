#!/usr/bin/env python3
"""v3.28.4 meta query route.

Adds a read-only SQL console over the game's SQLCipher-encrypted resource
index (`/data/user/0/<pkg>/files/meta`, ~164 MB):

    GET /debug/resource_meta_query                          -> table name list
    GET /debug/resource_meta_query?table=NAME&limit=50      -> rows of a table
    GET /debug/resource_meta_query?sql=SELECT%20...         -> one read-only statement

Design mirrors the proven `meta_dump_endpoint` machinery (libnative.so's own
sqlite3mc + the captured key from files/ura_meta_key.txt, falling back to the
in-memory META_KEY_HEX capture), but:

  * opens SQLITE_OPEN_READONLY only — never creates journals on the live file;
  * tries three key-call variants (sqlite3_key v1 / sqlite3_key_v2 with NULL
    db-name / sqlite3_key_v2 with "main") and verifies each by preparing
    `SELECT count(*) FROM sqlite_master`; only the variant that actually
    decrypts is accepted (the winner is reported as key_used);
  * caps rows (limit<=500, hard byte budget ~900 KB) and cell text (300 chars)
    so one query can never blow up the 18765 response path;
  * adds the route to /health and the not-found endpoint lists, and to the
    ?dl=1 download whitelist.

Idempotent: marker present -> already_applied. Anchors fail closed.

Chain position: after strip_ai_output_v3283 + apply_dynamic_version, before
the final bump_plugin_version to 3.28.4. Baseline = v3.28.2 lineage.
"""
from pathlib import Path

SOURCE = Path("hachimi_ura_plugin/src/lib.rs")
MARK = "// ===== v3.28.4 meta query route ====="

FUNC = r'''// ===== v3.28.4 meta query route =====
// Read-only SQL console over the game's SQLCipher-encrypted resource index
// (/data/user/0/<pkg>/files/meta, ~164 MB) using libnative's own sqlite3mc
// with the captured key (files/ura_meta_key.txt, falling back to the
// in-memory capture). GET /debug/resource_meta_query with no params lists
// table names; ?table=NAME&limit=N&offset=M reads rows; ?sql=SELECT... runs
// one read-only statement. Opens SQLITE_OPEN_READONLY, so it never creates
// journals or touches the original file. Three key-call variants (key v1,
// key_v2 with NULL db name, key_v2 with "main") are each verified by
// preparing `SELECT count(*) FROM sqlite_master`; only the variant that
// actually decrypts is used, and the outcome is reported in the response.
fn debug_resource_meta_query(full_uri: &str) -> String {
    unsafe {
        type QOpenV2 =
            extern "C" fn(*const i8, *mut *mut c_void, libc::c_int, *const i8) -> libc::c_int;
        type QKeyV1 = extern "C" fn(*mut c_void, *const c_void, libc::c_int) -> libc::c_int;
        type QKeyV2 =
            extern "C" fn(*mut c_void, *const i8, *const c_void, libc::c_int) -> libc::c_int;
        type QPrepV2 = extern "C" fn(
            *mut c_void,
            *const i8,
            libc::c_int,
            *mut *mut c_void,
            *mut *const i8,
        ) -> libc::c_int;
        type QStep = extern "C" fn(*mut c_void) -> libc::c_int;
        type QText = extern "C" fn(*mut c_void, libc::c_int) -> *const u8;
        type QColName = extern "C" fn(*mut c_void, libc::c_int) -> *const u8;
        type QColCount = extern "C" fn(*mut c_void) -> libc::c_int;
        type QColType = extern "C" fn(*mut c_void, libc::c_int) -> libc::c_int;
        type QColInt = extern "C" fn(*mut c_void, libc::c_int) -> i64;
        type QColDbl = extern "C" fn(*mut c_void, libc::c_int) -> f64;
        type QColBytes = extern "C" fn(*mut c_void, libc::c_int) -> libc::c_int;
        type QErrMsg = extern "C" fn(*mut c_void) -> *const u8;
        type QFin = extern "C" fn(*mut c_void) -> libc::c_int;

        let p_open = resolve_module_symbol("libnative.so", "sqlite3_open_v2");
        let p_key1 = resolve_module_symbol("libnative.so", "sqlite3_key");
        let p_key2 = resolve_module_symbol("libnative.so", "sqlite3_key_v2");
        let p_prep = resolve_module_symbol("libnative.so", "sqlite3_prepare_v2");
        let p_step = resolve_module_symbol("libnative.so", "sqlite3_step");
        let p_text = resolve_module_symbol("libnative.so", "sqlite3_column_text");
        let p_cname = resolve_module_symbol("libnative.so", "sqlite3_column_name");
        let p_cnum = resolve_module_symbol("libnative.so", "sqlite3_column_count");
        let p_ctype = resolve_module_symbol("libnative.so", "sqlite3_column_type");
        let p_cint = resolve_module_symbol("libnative.so", "sqlite3_column_int64");
        let p_cdbl = resolve_module_symbol("libnative.so", "sqlite3_column_double");
        let p_cbytes = resolve_module_symbol("libnative.so", "sqlite3_column_bytes");
        let p_errmsg = resolve_module_symbol("libnative.so", "sqlite3_errmsg");
        let p_fin = resolve_module_symbol("libnative.so", "sqlite3_finalize");
        let p_close = resolve_module_symbol("libnative.so", "sqlite3_close");
        if p_open == 0
            || (p_key1 == 0 && p_key2 == 0)
            || p_prep == 0
            || p_step == 0
            || p_text == 0
            || p_cnum == 0
            || p_fin == 0
            || p_close == 0
        {
            return r#"{"ok":false,"error":"symbol_resolve_failed"}"#.to_string();
        }
        let open_v2: QOpenV2 = std::mem::transmute(p_open);
        let prep: QPrepV2 = std::mem::transmute(p_prep);
        let step: QStep = std::mem::transmute(p_step);
        let coltext: QText = std::mem::transmute(p_text);
        let colname: Option<QColName> = if p_cname != 0 {
            let f: QColName = std::mem::transmute(p_cname);
            Some(f)
        } else {
            None
        };
        let colcount: QColCount = std::mem::transmute(p_cnum);
        let coltype: Option<QColType> = if p_ctype != 0 {
            let f: QColType = std::mem::transmute(p_ctype);
            Some(f)
        } else {
            None
        };
        let colint: Option<QColInt> = if p_cint != 0 {
            let f: QColInt = std::mem::transmute(p_cint);
            Some(f)
        } else {
            None
        };
        let coldbl: Option<QColDbl> = if p_cdbl != 0 {
            let f: QColDbl = std::mem::transmute(p_cdbl);
            Some(f)
        } else {
            None
        };
        let colbytes: Option<QColBytes> = if p_cbytes != 0 {
            let f: QColBytes = std::mem::transmute(p_cbytes);
            Some(f)
        } else {
            None
        };
        let errmsg: Option<QErrMsg> = if p_errmsg != 0 {
            let f: QErrMsg = std::mem::transmute(p_errmsg);
            Some(f)
        } else {
            None
        };
        let fin: QFin = std::mem::transmute(p_fin);
        let close: QFin = std::mem::transmute(p_close);

        let pkg_raw = std::fs::read("/proc/self/cmdline").unwrap_or_default();
        let pkg = String::from_utf8_lossy(&pkg_raw)
            .trim_matches(char::from(0))
            .trim()
            .to_string();
        if pkg.is_empty() {
            return r#"{"ok":false,"error":"pkg"}"#.to_string();
        }
        let db_path = format!("/data/user/0/{}/files/meta", pkg);
        let key_file = format!("/data/user/0/{}/files/ura_meta_key.txt", pkg);
        let key_from_file = std::fs::read_to_string(&key_file)
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        let key_source = if key_from_file.is_some() { "file" } else { "memory" };
        let key_hex = match key_from_file.or_else(|| {
            META_KEY_HEX
                .lock()
                .ok()
                .map(|g| g.clone())
                .filter(|v| !v.is_empty())
        }) {
            Some(h) => h,
            None => {
                return r#"{"ok":false,"error":"no_key_captured","hint":"restart once with ura_sqlcipher_hooks.flag present so the key is captured"}"#
                    .to_string()
            }
        };
        let key_bytes = hex_decode(&key_hex);
        if key_bytes.is_empty() {
            return r#"{"ok":false,"error":"bad_key_hex"}"#.to_string();
        }

        let c_path = std::ffi::CString::new(db_path.clone()).unwrap();
        let c_main = std::ffi::CString::new("main").unwrap();
        let mut db: *mut c_void = std::ptr::null_mut();
        let mut key_used = "";
        let mut attempts: Vec<String> = Vec::new();
        for variant in 1u8..=3u8 {
            let mut candidate: *mut c_void = std::ptr::null_mut();
            let rc = open_v2(c_path.as_ptr() as *const i8, &mut candidate, 0x1, std::ptr::null());
            if rc != 0 || candidate.is_null() {
                attempts.push(format!("open_rc={}", rc));
                continue;
            }
            let krc = match variant {
                1 => {
                    if p_key1 == 0 {
                        -1
                    } else {
                        let f: QKeyV1 = std::mem::transmute(p_key1);
                        f(
                            candidate,
                            key_bytes.as_ptr() as *const c_void,
                            key_bytes.len() as libc::c_int,
                        )
                    }
                }
                2 => {
                    if p_key2 == 0 {
                        -1
                    } else {
                        let f: QKeyV2 = std::mem::transmute(p_key2);
                        f(
                            candidate,
                            std::ptr::null(),
                            key_bytes.as_ptr() as *const c_void,
                            key_bytes.len() as libc::c_int,
                        )
                    }
                }
                _ => {
                    if p_key2 == 0 {
                        -1
                    } else {
                        let f: QKeyV2 = std::mem::transmute(p_key2);
                        f(
                            candidate,
                            c_main.as_ptr() as *const i8,
                            key_bytes.as_ptr() as *const c_void,
                            key_bytes.len() as libc::c_int,
                        )
                    }
                }
            };
            if krc != 0 {
                attempts.push(format!("v{}_key_rc={}", variant, krc));
                close(candidate);
                continue;
            }
            let c_verify = std::ffi::CString::new("SELECT count(*) FROM sqlite_master").unwrap();
            let mut stv: *mut c_void = std::ptr::null_mut();
            let prc = prep(
                candidate,
                c_verify.as_ptr() as *const i8,
                -1,
                &mut stv,
                std::ptr::null_mut(),
            );
            let verified = if prc == 0 && !stv.is_null() {
                let ok = step(stv) == 100;
                fin(stv);
                ok
            } else {
                false
            };
            if verified {
                db = candidate;
                key_used = match variant {
                    1 => "key_v1",
                    2 => "key_v2_null",
                    _ => "key_v2_main",
                };
                break;
            }
            attempts.push(format!("v{}_verify_failed", variant));
            close(candidate);
        }
        if db.is_null() {
            return format!(
                r#"{{"ok":false,"error":"no_key_variant_worked","key_source":"{}","attempts":"{}"}}"#,
                key_source,
                json_escape(&attempts.join(","))
            );
        }

        let c_bt = std::ffi::CString::new("PRAGMA busy_timeout=3000").unwrap();
        let mut st_bt: *mut c_void = std::ptr::null_mut();
        if prep(
            db,
            c_bt.as_ptr() as *const i8,
            -1,
            &mut st_bt,
            std::ptr::null_mut(),
        ) == 0
            && !st_bt.is_null()
        {
            let _ = step(st_bt);
            fin(st_bt);
        }

        let sql_param = parse_query(full_uri, "sql");
        let table_param = parse_query(full_uri, "table");
        let limit: i64 = parse_query(full_uri, "limit")
            .trim()
            .parse::<i64>()
            .unwrap_or(50)
            .clamp(1, 500);
        let offset: i64 = parse_query(full_uri, "offset")
            .trim()
            .parse::<i64>()
            .unwrap_or(0)
            .max(0);
        let (mode, sql) = if !sql_param.trim().is_empty() {
            ("sql".to_string(), sql_param.trim().to_string())
        } else if !table_param.trim().is_empty() {
            let t = table_param.trim().replace('"', "\"\"");
            (
                "table".to_string(),
                format!(
                    "SELECT * FROM \"{}\" LIMIT {} OFFSET {}",
                    t, limit, offset
                ),
            )
        } else {
            (
                "tables".to_string(),
                "SELECT name FROM sqlite_master WHERE type IN ('table','view') ORDER BY name"
                    .to_string(),
            )
        };
        let row_cap: usize = if mode == "tables" { 1000 } else { limit as usize };

        let c_sql = match std::ffi::CString::new(sql.clone()) {
            Ok(v) => v,
            Err(_) => {
                close(db);
                return r#"{"ok":false,"error":"sql_contains_nul"}"#.to_string();
            }
        };
        let mut st: *mut c_void = std::ptr::null_mut();
        let prc = prep(
            db,
            c_sql.as_ptr() as *const i8,
            -1,
            &mut st,
            std::ptr::null_mut(),
        );
        if prc != 0 || st.is_null() {
            let detail = match errmsg {
                Some(f) => {
                    let m = f(db);
                    if m.is_null() {
                        String::new()
                    } else {
                        CStr::from_ptr(m as *const c_char)
                            .to_string_lossy()
                            .into_owned()
                    }
                }
                None => String::new(),
            };
            close(db);
            return format!(
                r#"{{"ok":false,"error":"prepare_rc={}","detail":"{}","sql":"{}"}}"#,
                prc,
                json_escape(&detail),
                json_escape(&sql)
            );
        }
        let ncols = colcount(st).max(0);
        let mut columns: Vec<String> = Vec::new();
        for i in 0..ncols {
            let name = match colname {
                Some(f) => {
                    let p = f(st, i);
                    if p.is_null() {
                        format!("col{}", i)
                    } else {
                        CStr::from_ptr(p as *const c_char)
                            .to_string_lossy()
                            .into_owned()
                    }
                }
                None => format!("col{}", i),
            };
            columns.push(name);
        }
        let mut rows: Vec<String> = Vec::new();
        let mut truncated = false;
        let mut approx_bytes: usize = 0;
        loop {
            let s = step(st);
            if s == 100 {
                if rows.len() >= row_cap {
                    truncated = true;
                    break;
                }
                let mut cells: Vec<String> = Vec::new();
                let mut row_bytes: usize = 8;
                for j in 0..ncols {
                    let t = coltype.map(|f| f(st, j)).unwrap_or(3);
                    let text = match t {
                        5 => "null".to_string(),
                        4 => {
                            let n = colbytes.map(|f| f(st, j)).unwrap_or(-1);
                            if n >= 0 {
                                format!("<blob:{} bytes>", n)
                            } else {
                                "<blob>".to_string()
                            }
                        }
                        1 => match colint {
                            Some(f) => f(st, j).to_string(),
                            None => {
                                let p = coltext(st, j);
                                if p.is_null() {
                                    String::new()
                                } else {
                                    CStr::from_ptr(p as *const c_char)
                                        .to_string_lossy()
                                        .into_owned()
                                }
                            }
                        },
                        2 => match coldbl {
                            Some(f) => format!("{}", f(st, j)),
                            None => {
                                let p = coltext(st, j);
                                if p.is_null() {
                                    String::new()
                                } else {
                                    CStr::from_ptr(p as *const c_char)
                                        .to_string_lossy()
                                        .into_owned()
                                }
                            }
                        },
                        _ => {
                            let p = coltext(st, j);
                            if p.is_null() {
                                String::new()
                            } else {
                                CStr::from_ptr(p as *const c_char)
                                    .to_string_lossy()
                                    .into_owned()
                            }
                        }
                    };
                    let clipped: String = if text.chars().count() > 300 {
                        text.chars().take(300).collect::<String>() + "..."
                    } else {
                        text
                    };
                    row_bytes += clipped.len() + 8;
                    cells.push(format!("\"{}\"", json_escape(&clipped)));
                }
                if approx_bytes + row_bytes > 900_000 {
                    truncated = true;
                    break;
                }
                approx_bytes += row_bytes;
                rows.push(format!("[{}]", cells.join(",")));
            } else if s == 101 {
                break;
            } else {
                let detail = match errmsg {
                    Some(f) => {
                        let m = f(db);
                        if m.is_null() {
                            String::new()
                        } else {
                            CStr::from_ptr(m as *const c_char)
                                .to_string_lossy()
                                .into_owned()
                        }
                    }
                    None => String::new(),
                };
                fin(st);
                close(db);
                return format!(
                    r#"{{"ok":false,"error":"step_rc={}","detail":"{}","rows_read":{}}}"#,
                    s,
                    json_escape(&detail),
                    rows.len()
                );
            }
        }
        fin(st);
        close(db);

        let columns_json: Vec<String> = columns
            .iter()
            .map(|c| format!("\"{}\"", json_escape(c)))
            .collect();
        format!(
            r#"{{"ok":true,"read_only":true,"mode":"{}","key_source":"{}","key_used":"{}","db":"{}","sql":"{}","columns":[{}],"rows":[{}],"row_count":{},"truncated":{},"hint":"plain=table list | ?table=NAME&limit=50 | ?sql=SELECT ..."}}"#,
            mode,
            key_source,
            key_used,
            json_escape(&db_path),
            json_escape(&sql),
            columns_json.join(","),
            rows.join(","),
            rows.len(),
            truncated
        )
    }
}
'''


def apply_once() -> str:
    text = SOURCE.read_text(encoding="utf-8")
    if MARK in text:
        return "already_applied"

    # 1) function insert right before the existing schema debug fn
    anchor_fn = "fn debug_resource_meta_schema() -> String {"
    c = text.count(anchor_fn)
    if c != 1:
        raise RuntimeError(f"schema fn anchor count={c} (expect 1)")
    text = text.replace(anchor_fn, FUNC.strip("\n") + "\n\n" + anchor_fn, 1)

    # 2) route dispatch entry
    anchor_route = (
        '    } else if path == "/debug/resource_meta_schema" {\n'
        "        debug_resource_meta_schema()"
    )
    c = text.count(anchor_route)
    if c != 1:
        raise RuntimeError(f"route anchor count={c} (expect 1)")
    text = text.replace(
        anchor_route,
        anchor_route
        + '\n    } else if path == "/debug/resource_meta_query" {\n'
        + "        debug_resource_meta_query(&full_uri)",
        1,
    )

    # 3) DL_ALLOWED entry (plain string literal, exactly one)
    plain = '"/debug/resource_meta_schema",'
    c = text.count(plain)
    if c != 1:
        raise RuntimeError(f"DL_ALLOWED anchor count={c} (expect 1)")
    text = text.replace(
        plain, '"/debug/resource_meta_schema","/debug/resource_meta_query",', 1
    )

    # 4) endpoint lists inside the raw JSON strings (escaped form, one or more)
    esc = '\\"/debug/resource_meta_schema\\",'
    c = text.count(esc)
    if c < 1:
        raise RuntimeError(f"escaped endpoint-list anchor missing (count={c})")
    text = text.replace(
        esc, '\\"/debug/resource_meta_schema\\",\\"/debug/resource_meta_query\\",'
    )

    # post conditions — refuse to write a half-applied tree
    if "fn debug_resource_meta_query(" not in text:
        raise RuntimeError("query fn missing after patch")
    if '} else if path == "/debug/resource_meta_query" {' not in text:
        raise RuntimeError("query route missing after patch")
    if '\\"/debug/resource_meta_query\\",' not in text:
        raise RuntimeError("endpoint list entry missing after patch")
    if MARK not in text:
        raise RuntimeError("marker missing after patch")

    SOURCE.write_text(text, encoding="utf-8")
    return "applied"


if __name__ == "__main__":
    print("meta_query_route=" + apply_once())
