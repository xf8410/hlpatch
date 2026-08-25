from pathlib import Path

p = Path('hachimi_ura_plugin/src/lib.rs')
s = p.read_text(encoding='utf-8')
marker = '// ===== v3.27.15 home training gains endpoint ====='
if marker in s:
    print('home_training_gains=already_applied')
    raise SystemExit(0)

# Add a separate endpoint; do not change the old scenario-side paramsincdec endpoint.
route_candidates = [
    '    } else if path == "/debug/ramengains" {\n',
    '    } else if path == "/debug/paramsincdec" {\n',
    '    } else if path == "/debug/cmdinfo" {\n',
]
route = next((x for x in route_candidates if s.count(x) == 1), None)
assert route is not None, 'no diagnostic route anchor found'
s = s.replace(route, '''    } else if path == "/debug/home_training_gains" {
        unsafe { read_home_training_gains_v32715() }
''' + route, 1)

anchor = '// ============================================================\n// ★ Enumerate ALL classes in assembly (runtime dump)\n'
assert s.count(anchor) == 1, f'function anchor count={s.count(anchor)}'
fn = r'''
// ===== v3.27.15 home training gains endpoint =====
// Reads the final values shown by the training home screen. This is deliberately
// separate from WorkSingleModeScenarioRamenDataSet.CommandInfoArray, whose params
// are scenario deltas and can legitimately be empty before a ramen is active.
unsafe fn read_home_training_gains_v32715() -> String {
    let image = get_image();
    if image.is_null() { return r#"{"error":"image_null"}"#.to_string(); }
    let wdm_class = find_class_by_short_name(image, "WorkDataManager");
    let wdm = get_singleton(wdm_class);
    if wdm.is_null() { return r#"{"error":"wdm_null"}"#.to_string(); }
    let sm = call_getter_ref(wdm_class, wdm, "get_SingleMode");
    if sm.is_null() { return r#"{"error":"single_mode_null"}"#.to_string(); }
    let sm_class = get_class_from_object(sm);

    let mut home: *mut c_void = ptr::null_mut();
    let mut home_getter = "";
    for getter in ["get_HomeInfo", "get_HomeInfoData", "get_Home", "get_SingleModeHomeInfo"] {
        let candidate = call_getter_ref(sm_class, sm, getter);
        if !candidate.is_null() {
            home = candidate;
            home_getter = getter;
            break;
        }
    }
    if home.is_null() {
        for class_name in ["WorkSingleModeHomeInfoData", "WorkSingleModeHomeInfo"] {
            let cls = find_class_by_short_name(image, class_name);
            let candidate = get_singleton(cls) as *mut c_void;
            if !candidate.is_null() {
                home = candidate;
                home_getter = "singleton_fallback";
                break;
            }
        }
    }
    if home.is_null() {
        return r#"{"error":"home_info_null","tried":["get_HomeInfo","get_HomeInfoData","get_Home","get_SingleModeHomeInfo","singleton_fallback"]}"#.to_string();
    }
    let home_class = get_class_from_object(home);
    let mut commands: *mut c_void = ptr::null_mut();
    let mut commands_getter = "";
    for getter in ["get_CommandInfoArray", "get_CommandArray", "get_Commands"] {
        let candidate = call_getter_on_instance(home_class, home, getter);
        if !candidate.is_null() {
            commands = candidate;
            commands_getter = getter;
            break;
        }
    }
    if commands.is_null() {
        for field in ["CommandInfoArray", "<CommandInfoArray>k__BackingField"] {
            let candidate = read_field_value(home_class, home, field);
            if !candidate.is_null() {
                commands = candidate;
                commands_getter = field;
                break;
            }
        }
    }
    if commands.is_null() { return r#"{"error":"home_command_array_null"}"#.to_string(); }

    let base = commands as *const u8;
    let len = std::ptr::read_unaligned::<usize>(base.add(IL2CPP_LIST_COUNT_OFF) as *const usize);
    if len == 0 || len > 32 { return format!(r#"{{"error":"bad_command_len","len":{}}}"#, len); }
    let mut rows = Vec::new();
    for i in 0..len {
        let command = std::ptr::read_unaligned::<*mut c_void>(
            base.add(IL2CPP_LIST_ITEMS_OFF + i * IL2CPP_LIST_ITEM_SIZE) as *const *mut c_void
        );
        if command.is_null() { continue; }
        let cls = get_class_from_object(command);
        let command_id = call_getter_obscured_int(cls, command, "get_CommandId");
        if !matches!(command_id, 101 | 102 | 103 | 105 | 106) { continue; }
        let failure_rate = call_getter_obscured_int(cls, command, "get_FailureRate");
        let params = call_getter_on_instance(cls, command, "get_ParamsIncDecInfoArray");
        let mut values = Vec::new();
        if !params.is_null() {
            let pb = params as *const u8;
            let plen = std::ptr::read_unaligned::<usize>(pb.add(IL2CPP_LIST_COUNT_OFF) as *const usize);
            if plen <= 32 {
                for j in 0..plen {
                    let item = std::ptr::read_unaligned::<*mut c_void>(
                        pb.add(IL2CPP_LIST_ITEMS_OFF + j * IL2CPP_LIST_ITEM_SIZE) as *const *mut c_void
                    );
                    if item.is_null() { continue; }
                    let icls = get_class_from_object(item);
                    let target = call_getter_obscured_int(icls, item, "get_TargetType");
                    let value = call_getter_obscured_int(icls, item, "get_Value");
                    let name = match target {
                        1 => "speed", 2 => "stamina", 3 => "power", 4 => "guts", 5 => "wisdom",
                        10 => "vital", 20 => "motivation", 30 => "skill_pt", _ => "unknown"
                    };
                    values.push(format!(r#"{{"target_type":{},"name":"{}","value":{}}}"#, target, name, value));
                }
            }
        }
        rows.push(format!(r#"{{"command_id":{},"failure_rate":{},"params":[{}]}}"#,
            command_id, failure_rate, values.join(",")));
    }
    format!(r#"{{"ok":true,"source":"home_final_display","home_getter":"{}","commands_getter":"{}","command_count":{},"trainings":[{}]}}"#,
        home_getter, commands_getter, len, rows.join(","))
}

'''
s = s.replace(anchor, fn + anchor, 1)
p.write_text(s, encoding='utf-8')
print('home_training_gains=applied')
