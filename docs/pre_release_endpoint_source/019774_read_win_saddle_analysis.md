# `read_win_saddle_analysis`

source_commit: `a340a147acf13672b2fbc64925bfa321d08091fd`
source_line: `19774`

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
