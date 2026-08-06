# `read_summary_inner_impl`

source_commit: `ffc3748df2d3c8c57b34aa3fdd64f75d09ed0866`
source_line: `4624`

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
