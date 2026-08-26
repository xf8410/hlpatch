from pathlib import Path

p = Path('hachimi_ura_plugin/src/lib.rs')
s = p.read_text(encoding='utf-8')
marker = '// ===== v3.27.15 HomeInfo runtime command-array resolution ====='
if marker in s:
    print('home_training_gains_runtime_fix=already_applied')
    raise SystemExit(0)

old = '''    // ===== v3.27.15 verified direct home gain path =====
    // Runtime validation on the target build: HomeInfo +0x10 is
    // SingleModeCommandInfoData[10]. This is the same object chain used by
    // /debug/training_partners and avoids guessing unavailable property names.
    let commands = read_ptr_at(home, 0x10);
    let commands_getter = "verified_field_offset_0x10";
    if commands.is_null() {
        return format!(r#"{{\\"error\\":\\"home_command_array_null\\",\\"home\\":\\"{:p}\\",\\"home_getter\\":\\"{}\\"}}"#, home, home_getter);
    }
'''
new = '''    // ===== v3.27.15 HomeInfo runtime command-array resolution =====
    // Do not assume a single field offset: the target build exposed a bogus
    // List length when HomeInfo +0x10 was treated as CommandInfoArray.
    let home_class = get_class_from_object(home);
    let mut commands: *mut c_void = ptr::null_mut();
    let mut commands_getter = "";

    for getter in ["get_CommandInfoArray", "get_CommandArray", "get_Commands"] {
        let candidate = call_getter_on_instance(home_class, home, getter);
        if !candidate.is_null() {
            let base = candidate as *const u8;
            let len = std::ptr::read_unaligned::<usize>(base.add(IL2CPP_LIST_COUNT_OFF) as *const usize);
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
            if candidate.is_null() { continue; }
            let base = candidate as *const u8;
            let len = std::ptr::read_unaligned::<usize>(base.add(IL2CPP_LIST_COUNT_OFF) as *const usize);
            if len > 0 && len <= 32 {
                commands = candidate;
                commands_getter = field;
                break;
            }
        }
    }

    if commands.is_null() {
        return format!(r#"{{\\"error\\":\\"home_command_array_null\\",\\"home\\":\\"{:p}\\",\\"home_getter\\":\\"{}\\",\\"reason\\":\\"no_valid_runtime_array\\"}}"#, home, home_getter);
    }
'''
assert s.count(old) == 1, f'home resolution anchor count={s.count(old)}'
s = s.replace(old, new, 1)

old2 = '''        // Verified SingleModeCommandInfoData layout on this build:
        // CommandId ObscuredInt @ +0x24, Params array @ +0x60,
        // FailureRate ObscuredInt @ +0x68.
        let command_id = read_obscured_int_at(command, 0x24);
        if !matches!(command_id, 101 | 102 | 103 | 105 | 106) { continue; }
        let failure_rate = read_obscured_int_at(command, 0x68);
        let params = read_ptr_at(command, 0x60);
'''
new2 = '''        // Prefer the verified object properties; retain direct offsets only as
        // a fallback for builds where the properties are unavailable.
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

old3 = '''                    // SingleModeParamsIncDecInfoData: TargetType ObscuredInt
                    // @ +0x10 and Value ObscuredInt @ +0x24.
                    let target = read_obscured_int_at(item, 0x10);
                    let value = read_obscured_int_at(item, 0x24);
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
print('home_training_gains_runtime_fix=applied')
