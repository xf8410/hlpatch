from pathlib import Path
import re

SOURCE = Path("hachimi_ura_plugin/src/lib.rs")
s = SOURCE.read_text(encoding="utf-8")
MARKER = "// ===== Succession white-factor static lookup and runtime audit Q-stage ====="
if MARKER in s:
    print("succession_white_factor_audit=already_applied")
    raise SystemExit(0)


def replace_once(old: str, new: str, label: str) -> None:
    global s
    count = s.count(old)
    assert count == 1, f"{label} anchor count={count}"
    s = s.replace(old, new, 1)

anchor = "/// 辅助函数：IL2CPP类型枚举转可读名称\n"
assert s.count(anchor) == 1
rust = r'''// ===== Succession white-factor static lookup and runtime audit Q-stage =====
#[derive(Clone, Default)]
struct SuccessionAuditContext {
    sequence: u64,
    factor_id: i32,
    factor_group_id: i32,
    factor_type: i32,
    factor_lv: i32,
    source_position: i32,
    source_trained_chara_id: i64,
    target_trained_chara_id: i64,
    apply_this: usize,
    factor_argument: usize,
    skill_argument: usize,
    started_at_ms: u64,
}

static SUCCESSION_AUDIT_SEQUENCE: AtomicU64 = AtomicU64::new(0);
static SUCCESSION_AUDIT_EVENTS: Mutex<Vec<String>> = Mutex::new(Vec::new());
static mut SUCCESSION_APPLY_FACTOR_ADDR: usize = 0;
static mut SUCCESSION_GET_BONUS_ADDR: usize = 0;

thread_local! {
    static SUCCESSION_ACTIVE_FACTOR: std::cell::RefCell<Option<SuccessionAuditContext>> = std::cell::RefCell::new(None);
}

fn succession_target_type_name(value: i32) -> &'static str {
    match value {
        1 => "speed", 2 => "stamina", 3 => "power", 4 => "guts", 5 => "wisdom",
        6 => "skill_point", 7 => "skill_hint", 11 => "turf", 12 => "dirt",
        21 => "short", 22 => "mile", 23 => "middle", 24 => "long",
        31 => "nige", 32 => "senko", 33 => "sashi", 34 => "oikomi",
        41 => "race_factor", 51 => "scenario_factor", 61 => "parent_star_speed",
        62 => "parent_star_stamina", 63 => "parent_star_power", 64 => "parent_star_guts",
        65 => "parent_star_wisdom", _ => "unknown",
    }
}

fn succession_factor_type_name(value: i32) -> &'static str {
    match value {
        1 => "parameter", 2 => "proper", 3 => "chara_green", 4 => "skill_white",
        5 => "race_white", 6 => "scenario_white", 7 => "match_bonus",
        8 => "parent_star", 9 => "special_condition", _ => "unknown",
    }
}

fn succession_factor_category(value: i32) -> &'static str {
    match value {
        1 => "blue", 2 => "red_or_pink", 3 => "green", 4 | 5 | 6 | 7 | 8 | 9 => "white",
        _ => "unknown",
    }
}

fn succession_push_audit_event(value: String) {
    if let Ok(mut events) = SUCCESSION_AUDIT_EVENTS.lock() {
        events.push(value.clone());
        if events.len() > 4096 { let remove = events.len() - 4096; events.drain(0..remove); }
    }
    if let Err(error) = append_global_observation("succession_factor_audit", "complete", &value, true) {
        storage_set_error(&format!("persist_succession_factor_audit:{}", error));
    }
}

fn succession_master_connection() -> Result<rusqlite::Connection, String> {
    let candidates = [
        "/data/data/jp.co.cygames.umamusume/files/master/master.mdb",
        "/data/user/0/jp.co.cygames.umamusume/files/master/master.mdb",
    ];
    for candidate in candidates {
        if std::path::Path::new(candidate).is_file() {
            return rusqlite::Connection::open_with_flags(candidate, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
                .map_err(|error| format!("open_master_mdb:{}", error));
        }
    }
    Err("master_mdb_not_found".to_string())
}

fn succession_factor_static_lookup(uri: &str) -> String {
    let pairs = match parse_query_pairs(uri) { Ok(value) => value, Err(error) => return k_json_error(&error) };
    let factor_id = match query_pair(&pairs, "factor_id").parse::<i64>() {
        Ok(value) if value > 0 => value,
        _ => return k_json_error("invalid_or_missing_factor_id"),
    };
    let requested_lv = query_pair(&pairs, "factor_lv").parse::<i64>().ok().filter(|value| *value >= 0);
    let connection = match succession_master_connection() { Ok(value) => value, Err(error) => return k_json_error(&error) };
    let factor = connection.query_row(
        "SELECT factor_group_id,rarity,grade,factor_type,effect_group_id FROM succession_factor WHERE factor_id=?1",
        rusqlite::params![factor_id],
        |row| Ok((row.get::<_,i64>(0)?,row.get::<_,i64>(1)?,row.get::<_,i64>(2)?,row.get::<_,i64>(3)?,row.get::<_,i64>(4)?))
    );
    let (group_id, rarity, grade, factor_type, effect_group_id) = match factor {
        Ok(value) => value,
        Err(rusqlite::Error::QueryReturnedNoRows) => return k_json_error("factor_id_not_found"),
        Err(error) => return k_json_error(&format!("query_succession_factor:{}", error)),
    };
    let mut statement = match connection.prepare(
        "SELECT id,effect_id,target_type,value_1,value_2 FROM succession_factor_effect WHERE factor_group_id=?1 ORDER BY effect_id,target_type,id"
    ) { Ok(value) => value, Err(error) => return k_json_error(&format!("prepare_factor_effect:{}", error)) };
    let rows = match statement.query_map(rusqlite::params![group_id], |row| Ok((
        row.get::<_,i64>(0)?,row.get::<_,i64>(1)?,row.get::<_,i64>(2)?,row.get::<_,i64>(3)?,row.get::<_,i64>(4)?
    ))) { Ok(value) => value, Err(error) => return k_json_error(&format!("query_factor_effect:{}", error)) };
    let mut all = Vec::new();
    let mut selected = Vec::new();
    for row in rows {
        let (id,effect_id,target_type,value_1,value_2) = match row { Ok(value) => value, Err(error) => return k_json_error(&format!("decode_factor_effect:{}", error)) };
        let item = format!(r#"{{"id":{},"effect_id":{},"target_type":{},"target_name":"{}","value_1":{},"value_2":{}}}"#,
            id,effect_id,target_type,succession_target_type_name(target_type as i32),value_1,value_2);
        if requested_lv == Some(effect_id) { selected.push(item.clone()); }
        all.push(item);
    }
    let lv_json = requested_lv.map(|value| value.to_string()).unwrap_or_else(|| "null".to_string());
    format!(r#"{{"ok":true,"factor_id":{},"factor_group_id":{},"rarity":{},"grade":{},"factor_lv":{},"factor_type":{},"factor_type_name":"{}","factor_category":"{}","effect_group_id":{},"name":null,"name_status":"requires_game_factor_Name_or_factor_effects_resource","inherited_this_time":null,"theoretical_effects_available_without_inheritance":true,"selected_effect_count":{},"selected_effects":[{}],"all_effect_count":{},"all_effects":[{}]}}"#,
        factor_id,group_id,rarity,grade,lv_json,factor_type,succession_factor_type_name(factor_type as i32),
        succession_factor_category(factor_type as i32),effect_group_id,selected.len(),selected.join(","),all.len(),all.join(","))
}

fn succession_factor_batch_lookup(uri: &str) -> String {
    let pairs = match parse_query_pairs(uri) { Ok(value) => value, Err(error) => return k_json_error(&error) };
    let ids = query_pair(&pairs, "factor_ids");
    if ids.is_empty() { return k_json_error("missing_factor_ids"); }
    let parsed: Vec<i64> = ids.split(',').filter_map(|value| value.trim().parse::<i64>().ok()).filter(|value| *value > 0).take(512).collect();
    if parsed.is_empty() { return k_json_error("no_valid_factor_ids"); }
    let factor_lv = query_pair(&pairs, "factor_lv");
    let items = parsed.iter().map(|factor_id| {
        let synthetic = if factor_lv.is_empty() { format!("/inherit/factor_static?factor_id={}",factor_id) }
            else { format!("/inherit/factor_static?factor_id={}&factor_lv={}",factor_id,factor_lv) };
        succession_factor_static_lookup(&synthetic)
    }).collect::<Vec<_>>();
    format!(r#"{{"ok":true,"count":{},"factors":[{}]}}"#,items.len(),items.join(","))
}

extern "C" fn succession_get_bonus_hook_handler(this: *mut c_void, target_type: i32) -> i32 {
    unsafe {
        let trampoline = interceptor_get_trampoline(succession_get_bonus_hook_handler as usize);
        if trampoline == 0 { return 0; }
        type FnType = unsafe extern "C" fn(*mut c_void, i32) -> i32;
        let original: FnType = std::mem::transmute(trampoline);
        let bonus = original(this, target_type);
        SUCCESSION_ACTIVE_FACTOR.with(|slot| {
            if let Some(context) = slot.borrow().as_ref() {
                succession_push_audit_event(format!(r#"{{"sequence":{},"timestamp_ms":{},"method_hit":"SuccessionBonusParams.GetBonusValueByType","arguments":{{"this":"0x{:x}","FactorTargetType":{},"FactorTargetTypeName":"{}"}},"return_bonus":{},"factor_id":{},"factor_group_id":{},"factor_type":{},"factor_category":"{}","factorLv":{},"source_position":{},"source_trained_chara_id":{},"target_trained_chara_id":{},"attribute_before":null,"attribute_after":null,"attribute_delta":null,"attribute_snapshot_status":"event_level_protocol_snapshot_required"}}"#,
                    context.sequence,sniff_timestamp_ms(),this as usize,target_type,succession_target_type_name(target_type),bonus,
                    context.factor_id,context.factor_group_id,context.factor_type,succession_factor_category(context.factor_type),context.factor_lv,
                    context.source_position,context.source_trained_chara_id,context.target_trained_chara_id));
            }
        });
        bonus
    }
}

extern "C" fn succession_apply_factor_hook_handler(this: *mut c_void, factor: *mut c_void, skill: *mut c_void, factor_lv: i32) {
    unsafe {
        let trampoline = interceptor_get_trampoline(succession_apply_factor_hook_handler as usize);
        if trampoline == 0 { return; }
        type FnType = unsafe extern "C" fn(*mut c_void,*mut c_void,*mut c_void,i32);
        let original: FnType = std::mem::transmute(trampoline);
        let sequence = SUCCESSION_AUDIT_SEQUENCE.fetch_add(1, Ordering::Relaxed).saturating_add(1);
        let factor_id = l_named_i32(factor,&["FactorId","factorId","factor_id","Id","id"]).unwrap_or(0);
        let factor_group_id = l_named_i32(factor,&["FactorGroupId","factorGroupId","factor_group_id"]).unwrap_or(0);
        let factor_type = l_named_i32(factor,&["FactorType","factorType","factor_type"]).unwrap_or(0);
        let source_position = l_named_i32(factor,&["Position","position","SourcePosition","sourcePosition"]).unwrap_or(-1);
        let context = SuccessionAuditContext { sequence,factor_id,factor_group_id,factor_type,factor_lv,source_position,
            source_trained_chara_id:0,target_trained_chara_id:0,apply_this:this as usize,factor_argument:factor as usize,
            skill_argument:skill as usize,started_at_ms:sniff_timestamp_ms() };
        succession_push_audit_event(format!(r#"{{"sequence":{},"timestamp_ms":{},"phase":"enter","method_hit":"SuccessionBonusParams.ApplyFactor","arguments":{{"this":"0x{:x}","factor":"0x{:x}","uniqueSkill":"0x{:x}","factorLv":{}}},"factor_id":{},"factor_group_id":{},"factor_type":{},"factor_category":"{}","source_position":{},"source_trained_chara_id":{},"target_trained_chara_id":{},"parent_binding_status":"pending_runtime_context","attribute_before":null,"attribute_snapshot_status":"event_level_protocol_snapshot_required"}}"#,
            sequence,context.started_at_ms,this as usize,factor as usize,skill as usize,factor_lv,factor_id,factor_group_id,factor_type,
            succession_factor_category(factor_type),source_position,context.source_trained_chara_id,context.target_trained_chara_id));
        SUCCESSION_ACTIVE_FACTOR.with(|slot| *slot.borrow_mut() = Some(context.clone()));
        original(this,factor,skill,factor_lv);
        SUCCESSION_ACTIVE_FACTOR.with(|slot| *slot.borrow_mut() = None);
        succession_push_audit_event(format!(r#"{{"sequence":{},"timestamp_ms":{},"phase":"return","method_hit":"SuccessionBonusParams.ApplyFactor","factor_id":{},"factor_group_id":{},"factor_type":{},"factor_category":"{}","factorLv":{},"source_position":{},"attribute_after":null,"attribute_delta":null,"attribute_snapshot_status":"event_level_protocol_snapshot_required","result":"apply_factor_returned"}}"#,
            sequence,sniff_timestamp_ms(),factor_id,factor_group_id,factor_type,succession_factor_category(factor_type),factor_lv,source_position));
    }
}

unsafe fn install_succession_factor_audit_hooks() {
    if API.is_null() { set_hook_status("succession.factor_audit", "failed:api_null"); return; }
    let api=&*API;
    if api.interceptor==0 { set_hook_status("succession.factor_audit", "failed:interceptor_unavailable"); return; }
    let image=get_image(); if image.is_null(){set_hook_status("succession.factor_audit","failed:image_null");return;}
    let class=find_class_by_short_name(image,"SuccessionBonusParams");
    if class.is_null(){set_hook_status("succession.factor_audit","failed:class_not_found");return;}
    let get_method=match api.il2cpp_get_method_addr_fn{Some(value)=>value,None=>{set_hook_status("succession.factor_audit","failed:method_api_unavailable");return;}};
    if SUCCESSION_APPLY_FACTOR_ADDR==0 {
        let address=get_method(class as usize,to_cstr("ApplyFactor").as_ptr(),3);
        if address!=0 && interceptor_hook(address,succession_apply_factor_hook_handler as usize){SUCCESSION_APPLY_FACTOR_ADDR=address;set_hook_status("succession.apply_factor",&format!("hooked@0x{:x}",address));}
        else {set_hook_status("succession.apply_factor","failed:resolve_or_hook");}
    }
    if SUCCESSION_GET_BONUS_ADDR==0 {
        let address=get_method(class as usize,to_cstr("GetBonusValueByType").as_ptr(),1);
        if address!=0 && interceptor_hook(address,succession_get_bonus_hook_handler as usize){SUCCESSION_GET_BONUS_ADDR=address;set_hook_status("succession.get_bonus",&format!("hooked@0x{:x}",address));}
        else {set_hook_status("succession.get_bonus","failed:resolve_or_hook");}
    }
}

fn succession_audit_events_endpoint(uri:&str)->String{
    let pairs=match parse_query_pairs(uri){Ok(value)=>value,Err(error)=>return k_json_error(&error)};
    let after=query_pair(&pairs,"after_sequence").parse::<u64>().unwrap_or(0);
    let events=SUCCESSION_AUDIT_EVENTS.lock().unwrap_or_else(|error|error.into_inner());
    let selected=events.iter().filter(|item|{
        item.find("\"sequence\":").and_then(|start|item[start+11..].split(|c:char|!c.is_ascii_digit()).next()).and_then(|v|v.parse::<u64>().ok()).unwrap_or(0)>after
    }).cloned().collect::<Vec<_>>();
    format!(r#"{{"ok":true,"after_sequence":{},"count":{},"events":[{}],"hooks":{{"ApplyFactor":"0x{:x}","GetBonusValueByType":"0x{:x}"}}}}"#,after,selected.len(),selected.join(","),unsafe{SUCCESSION_APPLY_FACTOR_ADDR},unsafe{SUCCESSION_GET_BONUS_ADDR})
}

fn succession_audit_clear_endpoint()->String{
    if let Ok(mut events)=SUCCESSION_AUDIT_EVENTS.lock(){events.clear();}
    r#"{"ok":true,"cleared":true}"#.to_string()
}

'''
replace_once(anchor, rust + anchor, "audit_code")

install_anchor = '''unsafe fn install_api_sniff_hooks() {
    install_text_common_observer_hook();
'''
install_new = '''unsafe fn install_api_sniff_hooks() {
    install_text_common_observer_hook();
    install_succession_factor_audit_hooks();
'''
replace_once(install_anchor, install_new, "audit_install")

route_anchor = '    } else if path == "/storage/files" {\n'
routes = '''    } else if path == "/inherit/factor_static" {
        succession_factor_static_lookup(&full_uri)
    } else if path == "/inherit/factors_static" {
        succession_factor_batch_lookup(&full_uri)
    } else if path == "/inherit/audit/events" {
        succession_audit_events_endpoint(&full_uri)
    } else if path == "/inherit/audit/clear" {
        succession_audit_clear_endpoint()
'''
replace_once(route_anchor, routes + route_anchor, "audit_routes")

boot = re.search(r'((?:static|const)\s+BOOT_SAFE_EXACT\b[^=]*=\s*&\[)', s, re.M)
assert boot is not None
end=s.find('];',boot.end())
for endpoint in ["/inherit/factor_static","/inherit/factors_static","/inherit/audit/events","/inherit/audit/clear"]:
    if f'"{endpoint}"' not in s[boot.start():end]:
        s=s[:boot.end()]+f'\n    "{endpoint}",'+s[boot.end():]

replace_once(anchor, MARKER + "\n" + anchor, "audit_marker")
SOURCE.write_text(s,encoding="utf-8")
print("succession_white_factor_audit=applied")
