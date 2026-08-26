from pathlib import Path

SOURCE=Path('hachimi_ura_plugin/src/lib.rs')
s=SOURCE.read_text(encoding='utf-8')
MARKER='// ===== Succession per-factor bonus delta R-stage ====='
if MARKER in s:
    print('succession_audit_bonus_delta=already_applied')
    raise SystemExit(0)

anchor='extern "C" fn succession_apply_factor_hook_handler(this: *mut c_void, factor: *mut c_void, skill: *mut c_void, factor_lv: i32) {\n'
assert s.count(anchor)==1
helper=r'''// ===== Succession per-factor bonus delta R-stage =====
unsafe fn succession_bonus_snapshot(this: *mut c_void) -> Vec<(i32,i32)> {
    let trampoline=interceptor_get_trampoline(succession_get_bonus_hook_handler as usize);
    if trampoline==0 || this.is_null(){return Vec::new();}
    type FnType=unsafe extern "C" fn(*mut c_void,i32)->i32;
    let original:FnType=std::mem::transmute(trampoline);
    const TARGETS:&[i32]=&[1,2,3,4,5,6,7,11,12,21,22,23,24,31,32,33,34,41,51,61,62,63,64,65];
    TARGETS.iter().map(|target|(*target,original(this,*target))).collect()
}

fn succession_bonus_delta_json(before:&[(i32,i32)],after:&[(i32,i32)])->String{
    let mut items=Vec::new();
    for (target,after_value) in after {
        let before_value=before.iter().find(|(value,_)|value==target).map(|(_,value)|*value).unwrap_or(0);
        let delta=after_value.saturating_sub(before_value);
        if delta!=0 {
            items.push(format!(r#"{{"FactorTargetType":{},"FactorTargetTypeName":"{}","bonus_before":{},"bonus_after":{},"bonus_delta":{}}}"#,
                target,succession_target_type_name(*target),before_value,after_value,delta));
        }
    }
    format!("[{}]",items.join(","))
}

'''
s=s.replace(anchor,helper+anchor,1)

old='''        SUCCESSION_ACTIVE_FACTOR.with(|slot| *slot.borrow_mut() = Some(context.clone()));
        original(this,factor,skill,factor_lv);
        SUCCESSION_ACTIVE_FACTOR.with(|slot| *slot.borrow_mut() = None);
        succession_push_audit_event(format!(r#"{{"sequence":{},"timestamp_ms":{},"phase":"return","method_hit":"SuccessionBonusParams.ApplyFactor","factor_id":{},"factor_group_id":{},"factor_type":{},"factor_category":"{}","factorLv":{},"source_position":{},"attribute_after":null,"attribute_delta":null,"attribute_snapshot_status":"event_level_protocol_snapshot_required","result":"apply_factor_returned"}}"#,
            sequence,sniff_timestamp_ms(),factor_id,factor_group_id,factor_type,succession_factor_category(factor_type),factor_lv,source_position));
'''
new='''        let bonus_before=succession_bonus_snapshot(this);
        SUCCESSION_ACTIVE_FACTOR.with(|slot| *slot.borrow_mut() = Some(context.clone()));
        original(this,factor,skill,factor_lv);
        SUCCESSION_ACTIVE_FACTOR.with(|slot| *slot.borrow_mut() = None);
        let bonus_after=succession_bonus_snapshot(this);
        let bonus_deltas=succession_bonus_delta_json(&bonus_before,&bonus_after);
        succession_push_audit_event(format!(r#"{{"sequence":{},"timestamp_ms":{},"phase":"return","method_hit":"SuccessionBonusParams.ApplyFactor","factor_id":{},"factor_group_id":{},"factor_type":{},"factor_category":"{}","factorLv":{},"source_position":{},"bonus_deltas":{},"bonus_measurement":"GetBonusValueByType_before_after_original_ApplyFactor","attribute_before":null,"attribute_after":null,"attribute_delta":null,"attribute_snapshot_status":"event_level_protocol_snapshot_required","result":"apply_factor_returned"}}"#,
            sequence,sniff_timestamp_ms(),factor_id,factor_group_id,factor_type,succession_factor_category(factor_type),factor_lv,source_position,bonus_deltas));
'''
assert s.count(old)==1
s=s.replace(old,new,1)
SOURCE.write_text(s,encoding='utf-8')
print('succession_audit_bonus_delta=applied')
