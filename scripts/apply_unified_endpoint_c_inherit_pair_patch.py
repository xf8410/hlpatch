from pathlib import Path

SOURCE = Path("hachimi_ura_plugin/src/lib.rs")
s = SOURCE.read_text(encoding="utf-8")

MARKER = "// ===== Unified inheritance pair compatibility C-stage ====="
if MARKER in s:
    print("unified_endpoint_c_inherit_pair_patch=already_applied")
    raise SystemExit(0)

anchor = "/// 辅助函数：IL2CPP类型枚举转可读名称\n"
assert s.count(anchor) == 1, f"inherit insertion anchor count={s.count(anchor)}"

rust = r'''// ===== Unified inheritance pair compatibility C-stage =====
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

'''
s = s.replace(anchor, rust + anchor, 1)

route_anchor = '''    } else if path == "/inherit/compat" {
        unsafe { read_inherit_compat() }
'''
assert s.count(route_anchor) == 1, f"inherit route anchor count={s.count(route_anchor)}"
routes = '''    } else if path == "/inherit/pair_compat" {
        inherit_pair_compat_endpoint(&full_uri)
'''
s = s.replace(route_anchor, routes + route_anchor, 1)

SOURCE.write_text(s, encoding="utf-8")
print("unified_endpoint_c_inherit_pair_patch=applied")
