from pathlib import Path

SOURCE = Path("hachimi_ura_plugin/src/lib.rs")
s = SOURCE.read_text(encoding="utf-8")
MARKER = "// ===== Generated succession runtime decoder L ====="
if MARKER in s:
    print("generated_succession_runtime_l=already_applied")
    raise SystemExit(0)

anchor = "/// 辅助函数：IL2CPP类型枚举转可读名称\n"
assert s.count(anchor) == 1
rust = r'''// ===== Generated succession runtime decoder L =====
unsafe fn l_array_objects(array: *mut c_void) -> Result<Vec<*mut c_void>, String> {
    if array.is_null() || !is_readable_range(array as usize + 0x18, 8) { return Err("array_not_readable".to_string()); }
    let len = std::ptr::read_unaligned::<usize>((array as usize + 0x18) as *const usize);
    if len > 10000 { return Err(format!("array_length_out_of_range:{}", len)); }
    if len > 0 && !is_readable_range(array as usize + 0x20, len * 8) { return Err("array_elements_not_readable".to_string()); }
    Ok((0..len).map(|i| std::ptr::read_unaligned::<*mut c_void>((array as usize + 0x20 + i * 8) as *const *mut c_void)).collect())
}

unsafe fn l_named_i32(object: *mut c_void, candidates: &[&str]) -> Option<i32> {
    if object.is_null() || !is_readable_range(object as usize, 8) { return None; }
    let class = std::ptr::read_unaligned::<*mut c_void>(object as *const *mut c_void);
    let gf=resolve_il2cpp_symbol("il2cpp_class_get_fields"); let gn=resolve_il2cpp_symbol("il2cpp_field_get_name"); let go=resolve_il2cpp_symbol("il2cpp_field_get_offset");
    if class.is_null() || gf.is_null() || gn.is_null() || go.is_null() { return None; }
    let get_fields: unsafe extern "C" fn(*mut c_void,*mut *mut c_void)->*mut c_void=std::mem::transmute(gf);
    let get_name: unsafe extern "C" fn(*mut c_void)->*const c_char=std::mem::transmute(gn);
    let get_offset: unsafe extern "C" fn(*mut c_void)->i32=std::mem::transmute(go);
    let mut it=ptr::null_mut();
    loop { let f=get_fields(class,&mut it); if f.is_null(){break;} let name=il2cpp_c_string(get_name(f)); if candidates.iter().any(|v|*v==name){let off=get_offset(f);if off>=0&&is_readable_range(object as usize+off as usize,4){return Some(std::ptr::read_unaligned::<i32>((object as usize+off as usize) as *const i32));}} }
    None
}

unsafe fn l_factor_json(factor_class:*mut c_void, object:*mut c_void)->String{
    format!(r#"{{"factor_id":{},"factor_lv":{}}}"#,call_getter_obscured_int(factor_class,object,"get_FactorId"),call_getter_obscured_int(factor_class,object,"get_FactorLv"))
}
unsafe fn l_race_json(race_class:*mut c_void, object:*mut c_void)->String{
    format!(r#"{{"turn":{},"program_id":{},"race_instance_id":{},"frame_order":{},"entry_num":{},"weather":{},"ground_condition":{},"running_style":{},"result_rank":{},"scenario_id":{}}}"#,
      call_getter_obscured_int(race_class,object,"get_Turn"),call_getter_obscured_int(race_class,object,"get_ProgramId"),call_getter_obscured_int(race_class,object,"get_RaceInstanceId"),call_getter_obscured_int(race_class,object,"get_FrameOrder"),call_getter_obscured_int(race_class,object,"get_EntryNum"),call_getter_obscured_int(race_class,object,"get_Weather"),call_getter_obscured_int(race_class,object,"get_GroundCondition"),call_getter_obscured_int(race_class,object,"get_RunningStyle"),call_getter_obscured_int(race_class,object,"get_ResultRank"),call_getter_obscured_int(race_class,object,"get_ScenarioId"))
}
unsafe fn generated_succession_runtime_endpoint()->String{
    if API.is_null(){return k_json_error("api_null");} let image=get_image();if image.is_null(){return k_json_error("image_null");}
    let wdm_class=find_class(image,to_cstr("Gallop").as_ptr(),to_cstr("WorkDataManager").as_ptr());
    let store_class=find_class(image,to_cstr("Gallop").as_ptr(),to_cstr("WorkSuccessionOnlyCharaData").as_ptr());
    let wrapper_class=find_class_by_short_name(image,"GenerateSuccessionCharaData"); let trained_class=find_class_by_short_name(image,"TrainedCharaData");let race_class=find_class_by_short_name(image,"RaceHistoryInfo");let factor_class=find_class_by_short_name(image,"FactorData");
    if wdm_class.is_null()||store_class.is_null()||wrapper_class.is_null()||trained_class.is_null()||race_class.is_null()||factor_class.is_null(){return k_json_error("required_class_not_found");}
    let wdm=get_singleton(wdm_class);if wdm.is_null(){return k_json_error("work_data_manager_instance_not_found");}let store=call_getter_ref(wdm_class,wdm,"get_SuccessionOnlyCharaData");if store.is_null(){return k_json_error("succession_only_store_not_found");}
    let list=call_getter_ref(store_class,store,"get_GeneratedList");if list.is_null()||!is_readable_range(list as usize+0x18,4){return k_json_error("generated_list_not_found");}let size=std::ptr::read_unaligned::<i32>((list as usize+0x18) as *const i32);if !(0..=100).contains(&size){return k_json_error("generated_list_size_out_of_range");}let items=std::ptr::read_unaligned::<*mut c_void>((list as usize+0x10) as *const *mut c_void);if items.is_null(){return k_json_error("generated_list_items_null");}
    let mut generated=Vec::new();for i in 0..size as usize{let wrapper=std::ptr::read_unaligned::<*mut c_void>((items as usize+0x20+i*8) as *const *mut c_void);if wrapper.is_null(){return k_json_error("generated_wrapper_null");}let position=std::ptr::read_unaligned::<i32>((wrapper as usize+0x10) as *const i32);let trained=call_getter_ref(wrapper_class,wrapper,"get_TrainedCharaData");if trained.is_null(){return k_json_error("trained_chara_null");}
      let factor_array=call_getter_ref(trained_class,trained,"get_FactorDataArray");let race_array=call_getter_ref(trained_class,trained,"get_SingleModeRaceResultArray");let support_array=call_getter_ref(trained_class,trained,"get_SupportCardArray");
      let factors=match l_array_objects(factor_array){Ok(v)=>v.into_iter().map(|x|l_factor_json(factor_class,x)).collect::<Vec<_>>(),Err(e)=>return k_json_error(&format!("factor_{}",e))};let races=match l_array_objects(race_array){Ok(v)=>v.into_iter().map(|x|l_race_json(race_class,x)).collect::<Vec<_>>(),Err(e)=>return k_json_error(&format!("race_{}",e))};let supports=match l_array_objects(support_array){Ok(v)=>v.into_iter().map(|x|format!(r#"{{"support_card_id":{},"limit_break_count":{}}}"#,l_named_i32(x,&["SupportCardId","supportCardId","Id","id"]).unwrap_or(0),l_named_i32(x,&["LimitBreakCount","limitBreakCount","limit_break_count"]).unwrap_or(-1))).collect::<Vec<_>>(),Err(e)=>return k_json_error(&format!("support_{}",e))};
      generated.push(format!(r#"{{"position":{},"card_id":{},"chara_id":{},"scenario_id":{},"single_total_race_num":{},"single_win_num":{},"proper":{{"ground_turf":{},"ground_dirt":{},"distance_short":{},"distance_mile":{},"distance_middle":{},"distance_long":{},"running_nige":{},"running_senko":{},"running_sashi":{},"running_oikomi":{}}},"factors":[{}],"support_cards":[{}],"races":[{}]}}"#,position,call_getter_int(trained_class,trained,"get_CardId"),call_getter_obscured_int(trained_class,trained,"get_CharaId"),call_getter_obscured_int(trained_class,trained,"get_ScenarioId"),call_getter_int(trained_class,trained,"get_SingleTotalRaceNum"),call_getter_obscured_int(trained_class,trained,"get_SingleWinNum"),call_getter_obscured_int(trained_class,trained,"get_ProperGroundTurf"),call_getter_obscured_int(trained_class,trained,"get_ProperGroundDirt"),call_getter_obscured_int(trained_class,trained,"get_ProperDistanceShort"),call_getter_obscured_int(trained_class,trained,"get_ProperDistanceMile"),call_getter_obscured_int(trained_class,trained,"get_ProperDistanceMiddle"),call_getter_obscured_int(trained_class,trained,"get_ProperDistanceLong"),call_getter_obscured_int(trained_class,trained,"get_ProperRunningStyleNige"),call_getter_obscured_int(trained_class,trained,"get_ProperRunningStyleSenko"),call_getter_obscured_int(trained_class,trained,"get_ProperRunningStyleSashi"),call_getter_obscured_int(trained_class,trained,"get_ProperRunningStyleOikomi"),factors.join(","),supports.join(","),races.join(",")));
    }
    format!(r#"{{"ok":true,"scope":"generated_succession_runtime_full","count":{},"storage_order":"generated_list_index","generated":[{}]}}"#,generated.len(),generated.join(","))
}
'''
s=s.replace(anchor,rust+MARKER+"\n"+anchor,1)
route='    } else if path == "/generate_succession/result" {\n'
assert s.count(route)==1
s=s.replace(route,'    } else if path == "/generate_succession/runtime_full" {\n        unsafe { generated_succession_runtime_endpoint() }\n'+route,1)
SOURCE.write_text(s,encoding="utf-8")
print("generated_succession_runtime_l=applied")
