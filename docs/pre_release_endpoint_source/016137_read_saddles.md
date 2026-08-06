# `read_saddles`

source_commit: `ffc3748df2d3c8c57b34aa3fdd64f75d09ed0866`
source_line: `16137`

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
