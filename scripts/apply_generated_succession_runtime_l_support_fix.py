from pathlib import Path
p=Path('hachimi_ura_plugin/src/lib.rs')
s=p.read_text(encoding='utf-8')
marker='// ===== Generated succession runtime L support fix ====='
if marker in s:
    print('generated_succession_runtime_l_support_fix=already_applied')
    raise SystemExit(0)
old='''    let wrapper_class=find_class_by_short_name(image,"GenerateSuccessionCharaData"); let trained_class=find_class_by_short_name(image,"TrainedCharaData");let race_class=find_class_by_short_name(image,"RaceHistoryInfo");let factor_class=find_class_by_short_name(image,"FactorData");
    if wdm_class.is_null()||store_class.is_null()||wrapper_class.is_null()||trained_class.is_null()||race_class.is_null()||factor_class.is_null(){return k_json_error("required_class_not_found");}'''
new='''    let wrapper_class=find_class_by_short_name(image,"GenerateSuccessionCharaData"); let trained_class=find_class_by_short_name(image,"TrainedCharaData");let race_class=find_class_by_short_name(image,"RaceHistoryInfo");let factor_class=find_class_by_short_name(image,"FactorData");let support_class=find_class_by_full_declaring_name("Gallop.WorkTrainedCharaData/SupportCardData");
    if wdm_class.is_null()||store_class.is_null()||wrapper_class.is_null()||trained_class.is_null()||race_class.is_null()||factor_class.is_null()||support_class.is_null(){return k_json_error("required_class_not_found");}'''
assert s.count(old)==1, f'class_anchor_count={s.count(old)}'
s=s.replace(old,new,1)
old='''let supports=match l_array_objects(support_array){Ok(v)=>v.into_iter().map(|x|format!(r#"{{"support_card_id":{},"limit_break_count":{}}}"#,l_named_i32(x,&["SupportCardId","supportCardId","Id","id"]).unwrap_or(0),l_named_i32(x,&["LimitBreakCount","limitBreakCount","limit_break_count"]).unwrap_or(-1))).collect::<Vec<_>>(),Err(e)=>return k_json_error(&format!("support_{}",e))};'''
new='''let supports=match l_array_objects(support_array){Ok(v)=>v.into_iter().map(|x|format!(r#"{{"position":{},"support_card_id":{},"limit_break_count":{}}}"#,call_getter_obscured_int(support_class,x,"get_Position"),call_getter_obscured_int(support_class,x,"get_SupportCardId"),call_getter_obscured_int(support_class,x,"get_LimitBreakCount"))).collect::<Vec<_>>(),Err(e)=>return k_json_error(&format!("support_{}",e))};'''
assert s.count(old)==1, f'support_anchor_count={s.count(old)}'
s=s.replace(old,new,1)
s=s.replace('// ===== Generated succession runtime decoder L =====',marker+'\n// ===== Generated succession runtime decoder L =====',1)
p.write_text(s,encoding='utf-8')
print('generated_succession_runtime_l_support_fix=applied')
