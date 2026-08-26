from pathlib import Path

p = Path('hachimi_ura_plugin/src/lib.rs')
s = p.read_text(encoding='utf-8')
marker = '// ===== v3.27.15 verified direct home gain path ====='
if marker in s:
    print('home_training_gains_direct_fix=already_applied')
    raise SystemExit(0)

# Replace the naive command-array discovery from apply_v32715_home_training_gains.py
# with runtime-validated discovery. Anchors below mirror the base script verbatim.
old = '''    let home_class = get_class_from_object(home);
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
    if commands.is_null() { return r#"{\\"error\\":\\"home_command_array_null\\"}"#.to_string(); }
'''
new = '''    // ===== v3.27.15 verified direct home gain path =====
    // Validate every candidate array at runtime; do not trust a single fixed
    // field offset (a wrong offset yielded a bogus List length on-device).
    let home_class = get_class_from_object(home);
    let mut commands: *mut c_void = ptr::null_mut();
    let mut commands_getter = "";
    for getter in ["get_CommandInfoArray", "get_CommandArray", "get_Commands"] {
        let candidate = call_getter_on_instance(home_class, home, getter);
        if !candidate.is_null() {
            let len = std::ptr::read_unaligned::<usize>(
                (candidate as *const u8).add(IL2CPP_LIST_COUNT_OFF) as *const usize,
            );
            if len > 0 && len <= 32 {
                commands = candidate;
                commands_getter = getter;
                break;
            }
        }
    }
    if commands.is_null() {
        for field in ["CommandInfoArray", "<CommandInfoArray>k__BackingField", "CommandArray", "<CommandArray>k__BackingField"] {
            let candidate = read_field_value(home_class, home, field);
            if !candidate.is_null() {
                let len = std::ptr::read_unaligned::<usize>(
                    (candidate as *const u8).add(IL2CPP_LIST_COUNT_OFF) as *const usize,
                );
                if len > 0 && len <= 32 {
                    commands = candidate;
                    commands_getter = field;
                    break;
                }
            }
        }
    }
    if commands.is_null() { return r#"{\\"error\\":\\"home_command_array_null\\",\\"reason\\":\\"no_valid_runtime_array\\"}"#.to_string(); }
'''
assert s.count(old) == 1, f'command discovery anchor count={s.count(old)}'
s = s.replace(old, new, 1)

old2 = '''        let cls = get_class_from_object(command);
        let command_id = call_getter_obscured_int(cls, command, "get_CommandId");
        if !matches!(command_id, 101 | 102 | 103 | 105 | 106) { continue; }
        let failure_rate = call_getter_obscured_int(cls, command, "get_FailureRate");
        let params = call_getter_on_instance(cls, command, "get_ParamsIncDecInfoArray");
'''
new2 = '''        // Prefer property getters; fall back to previously observed offsets
        // (CommandId @0x24, Params @0x60, FailureRate @0x68) only when needed.
        let cls = get_class_from_object(command);
        let mut command_id = call_getter_obscured_int(cls, command, "get_CommandId");
        if !matches!(command_id, 101 | 102 | 103 | 105 | 106) {
            command_id = read_obscured_int_at(command, 0x24);
        }
        if !matches!(command_id, 101 | 102 | 103 | 105 | 106) { continue; }
        let mut failure_rate = call_getter_obscured_int(cls, command, "get_FailureRate");
        if failure_rate < 0 { failure_rate = read_obscured_int_at(command, 0x68); }
        let mut params = call_getter_on_instance(cls, command, "get_ParamsIncDecInfoArray");
        if params.is_null() { params = read_ptr_at(command, 0x60); }
'''
assert s.count(old2) == 1, f'command parse anchor count={s.count(old2)}'
s = s.replace(old2, new2, 1)

old3 = '''                    let icls = get_class_from_object(item);
                    let target = call_getter_obscured_int(icls, item, "get_TargetType");
                    let value = call_getter_obscured_int(icls, item, "get_Value");
'''
new3 = '''                    let icls = get_class_from_object(item);
                    let mut target = call_getter_obscured_int(icls, item, "get_TargetType");
                    let mut value = call_getter_obscured_int(icls, item, "get_Value");
                    if target < 0 { target = read_obscured_int_at(item, 0x10); }
                    if value < 0 { value = read_obscured_int_at(item, 0x24); }
'''
assert s.count(old3) == 1, f'param parse anchor count={s.count(old3)}'
s = s.replace(old3, new3, 1)

p.write_text(s, encoding='utf-8')
print('home_training_gains_direct_fix=applied')
