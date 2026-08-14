from pathlib import Path
import re

exec(
    Path("tooling/apply_turn_event_json_export.py").read_text(encoding="utf-8"),
    {"__name__": "__main__"},
)

SOURCE = Path("hachimi_ura_plugin/src/lib.rs")
s = SOURCE.read_text(encoding="utf-8")
MARKER = "// ===== Exact single-method IL2CPP probe B1 ====="
if MARKER in s:
    print("exact_method_probe_b1=already_applied")
    raise SystemExit(0)

anchor = "/// 辅助函数：IL2CPP类型枚举转可读名称\n"
assert s.count(anchor) == 1, f"exact method insertion anchor count={s.count(anchor)}"

rust = r'''// ===== Exact single-method IL2CPP probe B1 =====
unsafe fn exact_method_probe(uri: &str) -> String {
    let pairs = match parse_query_pairs(uri) {
        Ok(value) => value,
        Err(error) => return format!(r#"{{"ok":false,"error":"{}"}}"#, json_escape(&error)),
    };
    let declaring_type = query_pair(&pairs, "declaring_type");
    let method_name = query_pair(&pairs, "method");
    if declaring_type.is_empty() || method_name.is_empty() {
        return r#"{"ok":false,"error":"missing_declaring_type_or_method"}"#.to_string();
    }
    let requested_parameter_count = if query_pair(&pairs, "parameter_count").is_empty() {
        None
    } else {
        match query_pair(&pairs, "parameter_count").parse::<u32>() {
            Ok(value) if value <= 64 => Some(value),
            _ => return r#"{"ok":false,"error":"invalid_parameter_count"}"#.to_string(),
        }
    };
    let requested_bytes = if query_pair(&pairs, "max_bytes").is_empty() {
        256usize
    } else {
        match query_pair(&pairs, "max_bytes").parse::<usize>() {
            Ok(value) if value >= 4 && value <= 512 => value,
            _ => return r#"{"ok":false,"error":"invalid_max_bytes"}"#.to_string(),
        }
    };
    let image = get_image();
    if image.is_null() {
        return r#"{"ok":false,"error":"image_null"}"#.to_string();
    }
    let class = find_class_by_short_name(image, &declaring_type);
    if class.is_null() {
        return r#"{"ok":false,"error":"declaring_type_not_found"}"#.to_string();
    }
    let get_methods_ptr = resolve_il2cpp_symbol("il2cpp_class_get_methods");
    let get_name_ptr = resolve_il2cpp_symbol("il2cpp_method_get_name");
    let get_param_count_ptr = resolve_il2cpp_symbol("il2cpp_method_get_param_count");
    let get_param_ptr = resolve_il2cpp_symbol("il2cpp_method_get_param");
    let get_return_ptr = resolve_il2cpp_symbol("il2cpp_method_get_return_type");
    let get_type_name_ptr = resolve_il2cpp_symbol("il2cpp_type_get_name");
    if get_methods_ptr.is_null() || get_name_ptr.is_null() || get_param_count_ptr.is_null()
        || get_param_ptr.is_null() || get_return_ptr.is_null() || get_type_name_ptr.is_null()
    {
        return r#"{"ok":false,"error":"required_il2cpp_symbol_missing"}"#.to_string();
    }
    let get_methods: FnClassGetMethods = std::mem::transmute(get_methods_ptr);
    let get_name: FnMethodGetName = std::mem::transmute(get_name_ptr);
    let get_param_count: unsafe extern "C" fn(*const c_void) -> u32 = std::mem::transmute(get_param_count_ptr);
    let get_param: unsafe extern "C" fn(*const c_void, u32) -> *const c_void = std::mem::transmute(get_param_ptr);
    let get_return: unsafe extern "C" fn(*const c_void) -> *const c_void = std::mem::transmute(get_return_ptr);
    let get_type_name: unsafe extern "C" fn(*const c_void) -> *const c_char = std::mem::transmute(get_type_name_ptr);

    let mut iterator = ptr::null_mut();
    let mut matches: Vec<*const c_void> = Vec::new();
    loop {
        let method_info = get_methods(class, &mut iterator);
        if method_info.is_null() {
            break;
        }
        if il2cpp_c_string(get_name(method_info)) != method_name {
            continue;
        }
        if let Some(expected) = requested_parameter_count {
            if get_param_count(method_info) != expected {
                continue;
            }
        }
        matches.push(method_info);
    }
    if matches.is_empty() {
        return r#"{"ok":false,"error":"method_not_found"}"#.to_string();
    }
    if matches.len() != 1 {
        let counts = matches.iter().map(|method_info| get_param_count(*method_info).to_string())
            .collect::<Vec<_>>().join(",");
        return format!(r#"{{"ok":false,"error":"ambiguous_method","match_count":{},"parameter_counts":[{}]}}"#, matches.len(), counts);
    }

    let method_info = matches[0];
    let method_pointer = if is_readable_range(method_info as usize, std::mem::size_of::<usize>()) {
        std::ptr::read_unaligned::<usize>(method_info as *const usize)
    } else {
        0
    };
    if method_pointer == 0 {
        return r#"{"ok":false,"error":"method_pointer_null"}"#.to_string();
    }
    let readable_bytes = (requested_bytes / 4) * 4;
    if readable_bytes == 0 || !is_readable_range(method_pointer, readable_bytes) {
        return r#"{"ok":false,"error":"method_bytes_not_readable"}"#.to_string();
    }
    let bytes = std::slice::from_raw_parts(method_pointer as *const u8, readable_bytes);
    let mut direct_calls = Vec::new();
    for offset in (0..readable_bytes).step_by(4) {
        let instruction = u32::from_le_bytes([bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3]]);
        if instruction & 0xfc00_0000 == 0x9400_0000 {
            let immediate = ((instruction & 0x03ff_ffff) as i32) << 6 >> 4;
            let target = (method_pointer as i64 + offset as i64 + immediate as i64) as usize;
            direct_calls.push(format!(r#"{{"offset":{},"target":"0x{:x}"}}"#, offset, target));
        }
    }
    let parameter_count = get_param_count(method_info);
    let mut parameter_types = Vec::with_capacity(parameter_count as usize);
    for index in 0..parameter_count {
        let parameter = get_param(method_info, index);
        parameter_types.push(if parameter.is_null() {
            "unresolved".to_string()
        } else {
            il2cpp_c_string(get_type_name(parameter))
        });
    }
    let return_pointer = get_return(method_info);
    let return_type = if return_pointer.is_null() {
        "unresolved".to_string()
    } else {
        il2cpp_c_string(get_type_name(return_pointer))
    };
    let parameters_json = parameter_types.iter()
        .map(|value| format!(r#""{}""#, json_escape(value)))
        .collect::<Vec<_>>().join(",");
    format!(
        r#"{{"ok":true,"scope":"exact_declaring_type_method","declaring_type":"{}","method":"{}","parameter_count":{},"parameter_types":[{}],"return_type":"{}","method_info":"0x{:x}","method_pointer":"0x{:x}","bounded_bytes_length":{},"bounded_bytes_complete":false,"bounded_bytes_hex":"{}","direct_bl_targets":[{}]}}"#,
        json_escape(&declaring_type), json_escape(&method_name), parameter_count, parameters_json,
        json_escape(&return_type), method_info as usize, method_pointer, readable_bytes,
        hex_encode(bytes), direct_calls.join(",")
    )
}

'''
s = s.replace(anchor, rust + anchor, 1)

boot = re.search(r'((?:static|const)\s+BOOT_SAFE_EXACT\b[^=]*=\s*&\[)', s, re.M)
assert boot is not None, "BOOT_SAFE_EXACT declaration missing"
boot_end = s.find('];', boot.end())
assert boot_end >= 0, "BOOT_SAFE_EXACT terminator missing"
if '"/il2cpp/exact_method"' not in s[boot.start():boot_end]:
    s = s[:boot.end()] + '\n    "/il2cpp/exact_method",' + s[boot.end():]

route_anchor = '    } else if path == "/il2cpp/method_detail" {\n'
assert s.count(route_anchor) == 1, f"exact method route anchor count={s.count(route_anchor)}"
s = s.replace(
    route_anchor,
    '''    } else if path == "/il2cpp/exact_method" {
        unsafe { exact_method_probe(&full_uri) }
''' + route_anchor,
    1,
)

s = s.replace(anchor, MARKER + "\n" + anchor, 1)
SOURCE.write_text(s, encoding="utf-8")
print("exact_method_probe_b1=applied")
