from pathlib import Path
import re

p = Path("hachimi_ura_plugin/src/lib.rs")
s = p.read_text()

# Nested classes must be selected by their declaring type, not a repeated short name.
pattern = re.compile(r"unsafe fn find_class_by_short_name\(.*?\n\}\n(?=\n/// Slow fallback:)", re.S)
replacement = r'''unsafe fn find_class_by_short_name(image: *const c_void, class_name: &str) -> *mut c_void {
    // Accept "Outer/Nested" or "Namespace.Outer/Nested" for compiler-generated classes.
    if let Some((declaring_requested, nested_requested)) = class_name.rsplit_once('/') {
        let count_ptr = resolve_il2cpp_symbol("il2cpp_image_get_class_count");
        let class_ptr = resolve_il2cpp_symbol("il2cpp_image_get_class");
        let declaring_ptr = resolve_il2cpp_symbol("il2cpp_class_get_declaring_type");
        if count_ptr.is_null() || class_ptr.is_null() || declaring_ptr.is_null() {
            return ptr::null_mut();
        }
        let get_count: FnImageGetClassCount = std::mem::transmute(count_ptr);
        let get_class: FnImageGetClass = std::mem::transmute(class_ptr);
        let get_declaring: unsafe extern "C" fn(*mut c_void) -> *mut c_void = std::mem::transmute(declaring_ptr);
        let requested_short = declaring_requested.rsplit('.').next().unwrap_or(declaring_requested);
        for index in 0..get_count(image) {
            let candidate = get_class(image, index);
            if candidate.is_null() || get_class_name_from_pointer(candidate) != nested_requested { continue; }
            let declaring = get_declaring(candidate);
            if declaring.is_null() || get_class_name_from_pointer(declaring) != requested_short { continue; }
            if declaring_requested.contains('.') {
                let namespace_ptr = resolve_il2cpp_symbol("il2cpp_class_get_namespace");
                if namespace_ptr.is_null() { return ptr::null_mut(); }
                let get_namespace: FnClassGetName = std::mem::transmute(namespace_ptr);
                let raw = get_namespace(declaring);
                let namespace = if raw.is_null() { String::new() } else { CStr::from_ptr(raw).to_string_lossy().into_owned() };
                let full = if namespace.is_empty() { requested_short.to_string() } else { format!("{}.{}", namespace, requested_short) };
                if full != declaring_requested { continue; }
            }
            return candidate;
        }
        return ptr::null_mut();
    }
    let name_c = to_cstr(class_name);
    let ns_gallop = to_cstr("Gallop");
    let ns_empty = to_cstr("");
    for ns in [ns_gallop.as_ptr(), ns_empty.as_ptr()] {
        let cls = find_class(image, ns, name_c.as_ptr());
        if !cls.is_null() { return cls; }
    }
    find_class_by_iteration(image, class_name)
}'''
s, count = pattern.subn(replacement, s, count=1)
assert count == 1, f"find_class replacement count={count}"

start = s.index("unsafe fn il2cpp_list_methods(")
end = s.index("/// 辅助函数：IL2CPP类型枚举转可读名称", start)
prefix, body, suffix = s[:start], s[start:end], s[end:]
anchor = "    // il2cpp_method_get_return_type 获取返回类型\n"
assert body.count(anchor) == 1, f"parameter API anchor count={body.count(anchor)}"
body = body.replace(anchor, '''    // Read exact parameter metadata from the live MethodInfo.
    let method_get_param_fn: Option<unsafe extern "C" fn(*const c_void, u32) -> *const c_void> = {
        let p = resolve_il2cpp_symbol("il2cpp_method_get_param");
        if p.is_null() { None } else { Some(std::mem::transmute(p)) }
    };
    let method_get_param_name_fn: Option<unsafe extern "C" fn(*const c_void, u32) -> *const c_char> = {
        let p = resolve_il2cpp_symbol("il2cpp_method_get_param_name");
        if p.is_null() { None } else { Some(std::mem::transmute(p)) }
    };

''' + anchor, 1)
old_push = '''        methods.push(format!(
            r#"{{"name":"{}","params":{},"return_type":"{}","return_type_name":"{}","static":{},"own":{}}}"#,
            json_escape(&method_name),
            param_count,
            return_type_str,
            json_escape(&return_type_name),
            is_static,
            is_own_method
        ));
'''
new_push = '''        let mut parameter_items = Vec::new();
        for index in 0..param_count {
            let parameter_type = method_get_param_fn.map(|f| f(method_info, index)).unwrap_or(ptr::null());
            let parameter_name = method_get_param_name_fn.and_then(|f| {
                let value = f(method_info, index);
                if value.is_null() { None } else { Some(CStr::from_ptr(value).to_string_lossy().into_owned()) }
            });
            let parameter_type_enum = if parameter_type.is_null() { 0 } else { il2cpp_type_get_type_enum(parameter_type) };
            let parameter_type_name = if parameter_type.is_null() { "unknown".to_string() } else {
                type_get_name_fn.and_then(|f| {
                    let value = f(parameter_type);
                    if value.is_null() { None } else { Some(CStr::from_ptr(value).to_string_lossy().into_owned()) }
                }).unwrap_or_else(|| type_enum_to_name(parameter_type_enum))
            };
            parameter_items.push(format!(
                r#"{{"index":{},"name":{},"type":"{}","type_name":"{}","resolved":{}}}"#,
                index,
                parameter_name.map(|value| format!("\\\"{}\\\"", json_escape(&value))).unwrap_or_else(|| "null".to_string()),
                type_enum_to_name(parameter_type_enum), json_escape(&parameter_type_name), !parameter_type.is_null()
            ));
        }
        methods.push(format!(
            r#"{{"name":"{}","params":{},"parameters":[{}],"return_type":"{}","return_type_name":"{}","static":{},"own":{}}}"#,
            json_escape(&method_name), param_count, parameter_items.join(","), return_type_str,
            json_escape(&return_type_name), is_static, is_own_method
        ));
'''
assert body.count(old_push) == 1, f"method output block count={body.count(old_push)}"
body = body.replace(old_push, new_push, 1)
s = prefix + body + suffix

start = s.index("unsafe fn il2cpp_invoke_static_method(")
end = s.index("unsafe fn il2cpp_invoke_instance_method(", start)
prefix, body, suffix = s[:start], s[start:end], s[end:]
anchor = "    // This endpoint accepts i32-compatible value parameters only.\n"
assert body.count(anchor) == 1, f"invoke guard anchor count={body.count(anchor)}"
guard = '''    // The query parameter count does not define the ABI. Resolve every parameter first.
    let get_param_ptr = resolve_il2cpp_symbol("il2cpp_method_get_param");
    let type_name_ptr = resolve_il2cpp_symbol("il2cpp_type_get_name");
    if get_param_ptr.is_null() || type_name_ptr.is_null() {
        return r#"{"ok":false,"error":"parameter_metadata_api_unavailable"}"#.to_string();
    }
    let get_param: unsafe extern "C" fn(*const c_void, u32) -> *const c_void = std::mem::transmute(get_param_ptr);
    let get_type_name: unsafe extern "C" fn(*const c_void) -> *const c_char = std::mem::transmute(type_name_ptr);
    let mut signature = Vec::new();
    for index in 0..param_count as u32 {
        let parameter_type = get_param(method_info, index);
        if parameter_type.is_null() {
            return format!(r#"{{"ok":false,"error":"parameter_type_unresolved","index":{}}}"#, index);
        }
        let raw_name = get_type_name(parameter_type);
        let type_name = if raw_name.is_null() { "unknown".to_string() } else { CStr::from_ptr(raw_name).to_string_lossy().into_owned() };
        signature.push(type_name.clone());
        if type_name != "System.Int32" {
            return format!(r#"{{"ok":false,"error":"unsupported_parameter_type","index":{},"type_name":"{}","supported":["System.Int32"]}}"#, index, json_escape(&type_name));
        }
    }
    if class_name == "SingleModeRamenAPI" && method_name == "SendUrafEffectApply" {
        return format!(r#"{{"ok":false,"error":"runtime_state_gate_required","class":"{}","method":"{}","signature":[{}]}}"#,
            json_escape(class_name), json_escape(method_name), signature.iter().map(|value| format!("\\\"{}\\\"", json_escape(value))).collect::<Vec<_>>().join(","));
    }

'''
body = body.replace(anchor, guard + "    // This endpoint accepts verified System.Int32 value parameters only.\n", 1)
s = prefix + body + suffix
p.write_text(s)

cargo = Path("hachimi_ura_plugin/Cargo.toml")
cargo.write_text(cargo.read_text().replace('version = "3.27.3"', 'version = "3.27.4"', 1))
lock = Path("hachimi_ura_plugin/Cargo.lock")
lock.write_text(lock.read_text().replace('version = "3.27.3"', 'version = "3.27.4"', 1))
for temp in Path(".").glob("tmp_*"):
    temp.unlink()
print("patch_applied=true")
