from pathlib import Path

SOURCE = Path("hachimi_ura_plugin/src/lib.rs")
s = SOURCE.read_text(encoding="utf-8")

MARKER = "// ===== Unified selected inheritance parents D-stage ====="
if MARKER in s:
    print("unified_endpoint_d_selected_parent_patch=already_applied")
    raise SystemExit(0)

anchor = "/// 辅助函数：IL2CPP类型枚举转可读名称\n"
assert s.count(anchor) == 1, f"selected parent insertion anchor count={s.count(anchor)}"

rust = r'''// ===== Unified selected inheritance parents D-stage =====
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
        let trained_chara_id = call_getter_int(
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

'''
s = s.replace(anchor, rust + anchor, 1)

route_anchor = '''    } else if path == "/inherit/pair_compat" {
        inherit_pair_compat_endpoint(&full_uri)
'''
assert s.count(route_anchor) == 1, f"selected parent route anchor count={s.count(route_anchor)}"
routes = '''    } else if path == "/inherit/selected_parent_runtime" {
        unsafe { inherit_selected_parent_runtime_endpoint() }
'''
s = s.replace(route_anchor, routes + route_anchor, 1)

SOURCE.write_text(s, encoding="utf-8")
print("unified_endpoint_d_selected_parent_patch=applied")
