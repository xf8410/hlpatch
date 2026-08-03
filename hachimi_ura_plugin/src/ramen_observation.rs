use super::*;

const TRANSITION_CAPACITY: usize = 128;

struct CommonState {
    image: *const c_void,
    sm_class: *mut c_void,
    sm_obj: *mut c_void,
    chara_class: *mut c_void,
    chara_obj: *mut c_void,
    scenario_id: i32,
    month: i32,
    half: i32,
    turn_raw: Option<i32>,
}

struct TransitionRecord {
    id: u64,
    captured_at_unix_ms: u128,
    kind: &'static str,
    before: Option<String>,
    after: String,
}

struct TransitionBuffer {
    run_id: String,
    next_id: u64,
    last_snapshot: Option<String>,
    records: std::collections::VecDeque<TransitionRecord>,
}

static TRANSITIONS: std::sync::OnceLock<std::sync::Mutex<TransitionBuffer>> =
    std::sync::OnceLock::new();

fn unix_ms() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

fn transition_buffer() -> &'static std::sync::Mutex<TransitionBuffer> {
    TRANSITIONS.get_or_init(|| {
        let now = unix_ms();
        std::sync::Mutex::new(TransitionBuffer {
            run_id: format!("ramen-local-{}", now),
            next_id: 1,
            last_snapshot: None,
            records: std::collections::VecDeque::with_capacity(TRANSITION_CAPACITY),
        })
    })
}

fn observe_snapshot(snapshot: &str) {
    let mut buffer = match transition_buffer().lock() {
        Ok(v) => v,
        Err(_) => return,
    };
    if buffer.last_snapshot.as_deref() == Some(snapshot) {
        return;
    }
    let before = buffer.last_snapshot.clone();
    let kind = if before.is_none() { "initial" } else { "state_change" };
    let id = buffer.next_id;
    buffer.next_id = buffer.next_id.saturating_add(1);
    if buffer.records.len() == TRANSITION_CAPACITY {
        buffer.records.pop_front();
    }
    buffer.records.push_back(TransitionRecord {
        id,
        captured_at_unix_ms: unix_ms(),
        kind,
        before,
        after: snapshot.to_string(),
    });
    buffer.last_snapshot = Some(snapshot.to_string());
}

unsafe fn read_common_state() -> Result<CommonState, &'static str> {
    if API.is_null() {
        return Err("api_null");
    }
    let image = get_image();
    if image.is_null() {
        return Err("image_null");
    }
    let wdm_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkDataManager").as_ptr(),
    );
    if wdm_class.is_null() {
        return Err("work_data_manager_class_null");
    }
    let wdm = get_singleton(wdm_class);
    if wdm.is_null() {
        return Err("work_data_manager_instance_null");
    }
    let sm_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkSingleModeData").as_ptr(),
    );
    if sm_class.is_null() {
        return Err("single_mode_class_null");
    }
    let sm_obj = call_getter_ref(wdm_class, wdm, "get_SingleMode");
    if sm_obj.is_null() {
        return Err("single_mode_instance_null");
    }
    let chara_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkSingleModeCharaData").as_ptr(),
    );
    if chara_class.is_null() {
        return Err("chara_class_null");
    }
    let chara_obj = call_getter_ref(sm_class, sm_obj, "get_Character");
    if chara_obj.is_null() {
        return Err("chara_instance_null");
    }

    let raw = call_getter_int(sm_class, sm_obj, "GetCurrentTurn");
    let turn_raw = if raw >= 0 { Some(raw) } else { None };

    Ok(CommonState {
        image,
        sm_class,
        sm_obj,
        chara_class,
        chara_obj,
        scenario_id: call_getter_int(chara_class, chara_obj, "get_ScenarioId"),
        month: call_getter_int(chara_class, chara_obj, "get_Month"),
        half: call_getter_int(chara_class, chara_obj, "get_Half"),
        turn_raw,
    })
}

pub(super) fn authoritative_turn_for_ai() -> Option<i32> {
    let _guard = READ_MUTEX.lock().ok()?;
    let common = unsafe { read_common_state().ok()? };
    let _ = common.turn_raw;
    None
}

pub(super) fn read_timeline_json() -> String {
    let _guard = match READ_MUTEX.lock() {
        Ok(g) => g,
        Err(_) => return r#"{"schema_version":1,"error":"read_lock_poisoned"}"#.to_string(),
    };
    let common = match unsafe { read_common_state() } {
        Ok(v) => v,
        Err(e) => return format!(r#"{{"schema_version":1,"error":"{}"}}"#, e),
    };
    let raw = common
        .turn_raw
        .map(|v| v.to_string())
        .unwrap_or_else(|| "null".to_string());
    format!(
        r#"{{"schema_version":1,"scenario_id":{},"authoritative_turn":{{"raw":{},"zero_based":null,"one_based":null,"source":"WorkSingleModeData.GetCurrentTurn","source_available":{},"base_verified":false}},"display_calendar":{{"month":{},"half":{},"unique":false}},"phase":"unknown","recommendation_allowed":false,"fallback_used":false}}"#,
        common.scenario_id,
        raw,
        common.turn_raw.is_some(),
        common.month,
        common.half,
    )
}

unsafe fn int_array_json(class: *mut c_void, obj: *mut c_void, getter: &str) -> String {
    let values = read_obscured_int_array(class, obj, getter);
    format!(
        "[{}]",
        values.iter().map(i32::to_string).collect::<Vec<_>>().join(",")
    )
}

unsafe fn build_ramen_state_json(common: &CommonState) -> String {
    if common.scenario_id != 14 {
        return format!(
            r#"{{"schema_version":1,"error":"not_ramen","scenario_id":{}}}"#,
            common.scenario_id
        );
    }
    let scenario_obj = try_get_scenario_obj(common.chara_class, common.chara_obj, 14);
    if scenario_obj.is_null() {
        return r#"{"schema_version":1,"error":"ramen_scenario_instance_null"}"#.to_string();
    }
    let scenario_class = find_class_by_short_name(common.image, "WorkSingleModeScenarioRamen");
    if scenario_class.is_null() {
        return r#"{"schema_version":1,"error":"ramen_scenario_class_null"}"#.to_string();
    }
    let dataset = call_getter_ref(scenario_class, scenario_obj, "get_DataSet");
    if dataset.is_null() {
        return r#"{"schema_version":1,"error":"ramen_dataset_null"}"#.to_string();
    }
    let dataset_class = find_class_by_short_name(common.image, "WorkSingleModeScenarioRamenDataSet");
    if dataset_class.is_null() {
        return r#"{"schema_version":1,"error":"ramen_dataset_class_null"}"#.to_string();
    }

    let checkpoint = call_getter_obscured_int(dataset_class, dataset, "get_CheckPointPt");
    let expected = call_getter_obscured_int(dataset_class, dataset, "get_ExpectedCheckPointPt");
    let special = call_getter_obscured_int(dataset_class, dataset, "get_SpecialFeelingNum");
    let selected = int_array_json(dataset_class, dataset, "get_SelectedRegionIdArray");
    let all_selected = int_array_json(dataset_class, dataset, "get_AllSelectedRegionIdArray");

    let last = call_getter_ref(dataset_class, dataset, "get_LastTastingInfo");
    let last_json = if last.is_null() {
        "null".to_string()
    } else {
        let f1 = read_obscured_int_from_obj(last, "get_FeelingId1Num");
        let f2 = read_obscured_int_from_obj(last, "get_FeelingId2Num");
        let f3 = read_obscured_int_from_obj(last, "get_FeelingId3Num");
        let su = read_obscured_int_from_obj(last, "get_SpecialFeelingNum");
        let region = read_obscured_int_from_obj(last, "get_RegionId");
        format!(
            r#"{{"region_id":{},"ordinary_consumption":[{},{},{}],"special_used":{},"special_substitution":null}}"#,
            region, f1, f2, f3, su
        )
    };

    let raw_turn = common
        .turn_raw
        .map(|v| v.to_string())
        .unwrap_or_else(|| "null".to_string());
    format!(
        r#"{{"schema_version":1,"scenario_id":14,"timeline":{{"authoritative_turn_raw":{},"source":"WorkSingleModeData.GetCurrentTurn","base_verified":false,"phase":"unknown","display_month":{},"display_half":{},"fallback_used":false}},"checkpoint_pt":{{"current":{},"expected":{},"special_bonus_formula":null,"formula_status":"runtime_validation_required"}},"ordinary_materials":{{"capacity":10,"fifo":null,"counts":null,"status":"not_yet_exposed"}},"material_gauges":{{"values":null,"status":"not_yet_exposed"}},"special_feeling":{{"count":{},"capacity":4,"shares_ordinary_slots":false,"max_use_per_tasting":2}},"selected_region_ids":{},"all_selected_region_ids":{},"last_tasting":{},"recommendation_allowed":false}}"#,
        raw_turn,
        common.month,
        common.half,
        checkpoint,
        expected,
        special,
        selected,
        all_selected,
        last_json,
    )
}

pub(super) fn read_ramen_state_json() -> String {
    let snapshot = {
        let _guard = match READ_MUTEX.lock() {
            Ok(g) => g,
            Err(_) => return r#"{"schema_version":1,"error":"read_lock_poisoned"}"#.to_string(),
        };
        let common = match unsafe { read_common_state() } {
            Ok(v) => v,
            Err(e) => return format!(r#"{{"schema_version":1,"error":"{}"}}"#, e),
        };
        unsafe { build_ramen_state_json(&common) }
    };
    observe_snapshot(&snapshot);
    snapshot
}

pub(super) fn read_ramen_transitions_json() -> String {
    let snapshot = {
        let _guard = match READ_MUTEX.lock() {
            Ok(g) => g,
            Err(_) => return r#"{"schema_version":1,"error":"read_lock_poisoned"}"#.to_string(),
        };
        let common = match unsafe { read_common_state() } {
            Ok(v) => v,
            Err(e) => return format!(r#"{{"schema_version":1,"error":"{}"}}"#, e),
        };
        unsafe { build_ramen_state_json(&common) }
    };
    observe_snapshot(&snapshot);

    let buffer = match transition_buffer().lock() {
        Ok(v) => v,
        Err(_) => return r#"{"schema_version":1,"error":"transition_lock_poisoned"}"#.to_string(),
    };
    let records = buffer
        .records
        .iter()
        .map(|record| {
            let before = record.before.as_deref().unwrap_or("null");
            format!(
                r#"{{"transition_id":{},"captured_at_unix_ms":{},"kind":"{}","before":{},"after":{}}}"#,
                record.id, record.captured_at_unix_ms, record.kind, before, record.after
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        r#"{{"schema_version":1,"run_id":"{}","capacity":{},"dropped_before_id":{},"latest_transition_id":{},"records":[{}]}}"#,
        buffer.run_id,
        TRANSITION_CAPACITY,
        buffer.records.front().map(|v| v.id.saturating_sub(1)).unwrap_or(0),
        buffer.records.back().map(|v| v.id).unwrap_or(0),
        records,
    )
}
