# `read_inherit_compat`

source_commit: `ffc3748df2d3c8c57b34aa3fdd64f75d09ed0866`
source_line: `19575`

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
