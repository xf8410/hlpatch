from pathlib import Path

SOURCE = Path("hachimi_ura_plugin/src/lib.rs")
s = SOURCE.read_text(encoding="utf-8")
MARKER = "// ===== Unified selected-parent multi-source resolver I-stage ====="
if MARKER in s:
    print("unified_endpoint_i_parent_multisource=already_applied")
    raise SystemExit(0)

anchor = "/// 辅助函数：IL2CPP类型枚举转可读名称\n"
assert s.count(anchor) == 1, f"I insertion anchor count={s.count(anchor)}"

rust = r'''// ===== Unified selected-parent multi-source resolver I-stage =====
unsafe fn find_exact_instance_method(
    class: *mut c_void,
    name: &str,
    parameter_types: &[&str],
) -> *const c_void {
    let get_methods_ptr = resolve_il2cpp_symbol("il2cpp_class_get_methods");
    let get_name_ptr = resolve_il2cpp_symbol("il2cpp_method_get_name");
    let get_param_count_ptr = resolve_il2cpp_symbol("il2cpp_method_get_param_count");
    let get_param_ptr = resolve_il2cpp_symbol("il2cpp_method_get_param");
    let get_type_name_ptr = resolve_il2cpp_symbol("il2cpp_type_get_name");
    if class.is_null() || get_methods_ptr.is_null() || get_name_ptr.is_null()
        || get_param_count_ptr.is_null() || get_param_ptr.is_null() || get_type_name_ptr.is_null() {
        return ptr::null();
    }
    let get_methods: FnClassGetMethods = std::mem::transmute(get_methods_ptr);
    let get_name: FnMethodGetName = std::mem::transmute(get_name_ptr);
    let get_param_count: unsafe extern "C" fn(*const c_void) -> u32 = std::mem::transmute(get_param_count_ptr);
    let get_param: unsafe extern "C" fn(*const c_void, u32) -> *const c_void = std::mem::transmute(get_param_ptr);
    let get_type_name: unsafe extern "C" fn(*const c_void) -> *const c_char = std::mem::transmute(get_type_name_ptr);
    let mut iterator = ptr::null_mut();
    let mut found: *const c_void = ptr::null();
    loop {
        let method = get_methods(class, &mut iterator);
        if method.is_null() { break; }
        if il2cpp_c_string(get_name(method)) != name || get_param_count(method) as usize != parameter_types.len() { continue; }
        let mut exact = true;
        for (index, expected) in parameter_types.iter().enumerate() {
            let parameter = get_param(method, index as u32);
            if parameter.is_null() || il2cpp_c_string(get_type_name(parameter)) != *expected { exact = false; break; }
        }
        if exact {
            if !found.is_null() { return ptr::null(); }
            found = method;
        }
    }
    found
}

unsafe fn invoke_parent_store_get(store_class: *mut c_void, store: *mut c_void, trained_chara_id: i32) -> *mut c_void {
    let method = find_exact_instance_method(store_class, "Get", &["System.Int32", "System.Boolean"]);
    let invoke_ptr = resolve_il2cpp_symbol("il2cpp_runtime_invoke");
    if method.is_null() || invoke_ptr.is_null() { return ptr::null_mut(); }
    let invoke: unsafe extern "C" fn(*const c_void, *mut c_void, *mut *mut c_void, *mut *mut c_void) -> *mut c_void = std::mem::transmute(invoke_ptr);
    let mut id = trained_chara_id;
    let mut all = true;
    let mut arguments = [
        (&mut id as *mut i32).cast::<c_void>(),
        (&mut all as *mut bool).cast::<c_void>(),
    ];
    let mut exception = ptr::null_mut();
    let result = invoke(method, store, arguments.as_mut_ptr(), &mut exception);
    if exception.is_null() { result } else { ptr::null_mut() }
}

unsafe fn selected_parent_record_json(
    slot: &str,
    trained_chara_id: i32,
    own_store_class: *mut c_void,
    own_store: *mut c_void,
    succession_store_class: *mut c_void,
    succession_store: *mut c_void,
    record_class: *mut c_void,
) -> String {
    let mut source = "not_found";
    let mut record = if own_store.is_null() { ptr::null_mut() } else {
        invoke_parent_store_get(own_store_class, own_store, trained_chara_id)
    };
    if !record.is_null() { source = "trained_chara_data"; }
    if record.is_null() && !succession_store.is_null() {
        record = invoke_parent_store_get(succession_store_class, succession_store, trained_chara_id);
        if !record.is_null() { source = "succession_only_chara_data"; }
    }
    if record.is_null() {
        return format!(r#"{{"slot":"{}","trained_chara_id":{},"resolved":false,"source":"{}","record":null}}"#,
            slot, trained_chara_id, source);
    }
    let id = call_getter_obscured_int(record_class, record, "get_Id");
    let card_id = call_getter_int(record_class, record, "get_CardId");
    let chara_id = call_getter_obscured_int(record_class, record, "get_CharaId");
    let is_player = call_getter_bool(record_class, record, "get_IsPlayer");
    let is_rental = call_getter_bool(record_class, record, "get_IsRental");
    let is_others = call_getter_bool(record_class, record, "get_IsOthers");
    let is_succession_only = call_getter_bool(record_class, record, "get_IsSuccessionOnly");
    format!(r#"{{"slot":"{}","trained_chara_id":{},"resolved":true,"source":"{}","record":{{"id":{},"card_id":{},"chara_id":{},"is_player":{},"is_rental":{},"is_others":{},"is_succession_only":{}}}}}"#,
        slot, trained_chara_id, source, id, card_id, chara_id, is_player, is_rental, is_others, is_succession_only)
}

unsafe fn inherit_selected_parent_records_endpoint() -> String {
    if API.is_null() { return r#"{"ok":false,"error":"api_null"}"#.to_string(); }
    let image = get_image();
    if image.is_null() { return r#"{"ok":false,"error":"image_null"}"#.to_string(); }
    let wdm_class = find_class(image, to_cstr("Gallop").as_ptr(), to_cstr("WorkDataManager").as_ptr());
    let single_mode_class = find_class(image, to_cstr("Gallop").as_ptr(), to_cstr("WorkSingleModeData").as_ptr());
    let chara_class = find_class(image, to_cstr("Gallop").as_ptr(), to_cstr("WorkSingleModeCharaData").as_ptr());
    let own_store_class = find_class(image, to_cstr("Gallop").as_ptr(), to_cstr("WorkTrainedCharaData").as_ptr());
    let succession_store_class = find_class(image, to_cstr("Gallop").as_ptr(), to_cstr("WorkSuccessionOnlyCharaData").as_ptr());
    let record_class = find_class_by_short_name(image, "TrainedCharaData");
    let succession_info_class = find_class_by_short_name(image, "SuccessionCharaInfo");
    if wdm_class.is_null() || single_mode_class.is_null() || chara_class.is_null()
        || own_store_class.is_null() || succession_store_class.is_null()
        || record_class.is_null() || succession_info_class.is_null() {
        return r#"{"ok":false,"error":"required_class_not_found"}"#.to_string();
    }
    let wdm = get_singleton(wdm_class);
    if wdm.is_null() { return r#"{"ok":false,"error":"work_data_manager_instance_not_found"}"#.to_string(); }
    let single_mode = call_getter_ref(wdm_class, wdm, "get_SingleMode");
    let chara = call_getter_ref(single_mode_class, single_mode, "get_Character");
    if single_mode.is_null() || chara.is_null() { return r#"{"ok":false,"error":"single_mode_character_not_found"}"#.to_string(); }
    let own_store = call_getter_ref(wdm_class, wdm, "get_TrainedCharaData");
    let succession_store = call_getter_ref(wdm_class, wdm, "get_SuccessionOnlyCharaData");
    let first_info = call_getter_ref(chara_class, chara, "get_SuccessionTrainedCharaInfoFirst");
    let second_info = call_getter_ref(chara_class, chara, "get_SuccessionTrainedCharaInfoSecond");
    let first_id = if first_info.is_null() { 0 } else { call_getter_obscured_int(succession_info_class, first_info, "get_TrainedCharaId") };
    let second_id = if second_info.is_null() { 0 } else { call_getter_obscured_int(succession_info_class, second_info, "get_TrainedCharaId") };
    let first = selected_parent_record_json("first", first_id, own_store_class, own_store, succession_store_class, succession_store, record_class);
    let second = selected_parent_record_json("second", second_id, own_store_class, own_store, succession_store_class, succession_store, record_class);
    format!(r#"{{"ok":true,"scope":"selected_parent_record_multisource","lookup_order":["trained_chara_data","succession_only_chara_data"],"selected_temp_coverage":"included_by_succession_only_all_lookup","parents":[{},{}],"ancestor_tree":null,"race_bonus":null,"full_compatibility":null,"runtime_validation":"pending_device_execution"}}"#, first, second)
}

'''
s = s.replace(anchor, rust + MARKER + "\n" + anchor, 1)
route_anchor = '    } else if path == "/inherit/selected_parent_runtime" {\n'
assert s.count(route_anchor) == 1, f"I route anchor count={s.count(route_anchor)}"
s = s.replace(route_anchor, '    } else if path == "/inherit/selected_parent_records" {\n        unsafe { inherit_selected_parent_records_endpoint() }\n' + route_anchor, 1)
SOURCE.write_text(s, encoding="utf-8")
print("unified_endpoint_i_parent_multisource=applied")
