from pathlib import Path

SOURCE = Path("hachimi_ura_plugin/src/lib.rs")
s = SOURCE.read_text(encoding="utf-8")

MARKER = "// ===== Unified observation endpoint A-stage ====="
if MARKER in s:
    print("unified_endpoint_a_patch=already_applied")
    raise SystemExit(0)

anchor = "/// 辅助函数：IL2CPP类型枚举转可读名称\n"
assert s.count(anchor) == 1, f"IL2CPP insertion anchor count={s.count(anchor)}"

rust = r'''// ===== Unified observation endpoint A-stage =====
// The index is built once per game process. Addresses are stored as usize so the
// synchronized state never contains raw pointers shared between threads.
#[derive(Clone)]
struct MethodIndexEntry {
    method_info: usize,
    method_pointer: usize,
    namespace: String,
    declaring_type: String,
    method_name: String,
    return_type: String,
    parameter_names: Vec<Option<String>>,
    parameter_types: Vec<String>,
    flags: u32,
}

struct MethodIndexState {
    status: &'static str,
    error: String,
    entries: Vec<MethodIndexEntry>,
    image_class_count: u32,
    indexed_class_count: u32,
    indexed_method_count: usize,
    null_method_pointer_count: usize,
    duplicate_method_pointer_count: usize,
}

static METHOD_INDEX: Mutex<MethodIndexState> = Mutex::new(MethodIndexState {
    status: "empty",
    error: String::new(),
    entries: Vec::new(),
    image_class_count: 0,
    indexed_class_count: 0,
    indexed_method_count: 0,
    null_method_pointer_count: 0,
    duplicate_method_pointer_count: 0,
});

fn percent_decode_component(input: &str) -> Result<String, String> {
    let bytes = input.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0usize;
    while index < bytes.len() {
        match bytes[index] {
            b'%' => {
                if index + 2 >= bytes.len() {
                    return Err("incomplete_percent_escape".to_string());
                }
                let hex = &input[index + 1..index + 3];
                let value = u8::from_str_radix(hex, 16)
                    .map_err(|_| "invalid_percent_escape".to_string())?;
                output.push(value);
                index += 3;
            }
            b'+' => {
                output.push(b' ');
                index += 1;
            }
            value => {
                output.push(value);
                index += 1;
            }
        }
    }
    String::from_utf8(output).map_err(|_| "query_not_utf8".to_string())
}

fn parse_request_uri(request: &str) -> Result<String, String> {
    let line = request.lines().next().ok_or_else(|| "missing_request_line".to_string())?;
    let mut parts = line.split_whitespace();
    let method = parts.next().ok_or_else(|| "missing_http_method".to_string())?;
    let uri = parts.next().ok_or_else(|| "missing_request_uri".to_string())?;
    let version = parts.next().ok_or_else(|| "missing_http_version".to_string())?;
    if method.is_empty() || !version.starts_with("HTTP/") || parts.next().is_some() {
        return Err("invalid_request_line".to_string());
    }
    Ok(uri.to_string())
}

fn parse_query_pairs(uri: &str) -> Result<Vec<(String, String)>, String> {
    let query = match uri.split_once('?') {
        Some((_, value)) => value.split('#').next().unwrap_or(""),
        None => return Ok(Vec::new()),
    };
    let mut pairs = Vec::new();
    for item in query.split('&') {
        if item.is_empty() { continue; }
        let (raw_key, raw_value) = item.split_once('=').unwrap_or((item, ""));
        pairs.push((percent_decode_component(raw_key)?, percent_decode_component(raw_value)?));
    }
    Ok(pairs)
}

fn query_pair(pairs: &[(String, String)], name: &str) -> String {
    pairs.iter().find(|(key, _)| key == name).map(|(_, value)| value.clone()).unwrap_or_default()
}

fn parse_address(value: &str) -> Option<usize> {
    let trimmed = value.trim();
    if let Some(hex) = trimmed.strip_prefix("0x").or_else(|| trimmed.strip_prefix("0X")) {
        usize::from_str_radix(hex, 16).ok()
    } else {
        trimmed.parse::<usize>().ok()
    }
}

unsafe fn il2cpp_c_string(pointer: *const c_char) -> String {
    if pointer.is_null() { String::new() } else { CStr::from_ptr(pointer).to_string_lossy().into_owned() }
}

unsafe fn class_full_declaring_name(class: *mut c_void) -> String {
    if class.is_null() { return String::new(); }
    let get_name_ptr = resolve_il2cpp_symbol("il2cpp_class_get_name");
    let get_namespace_ptr = resolve_il2cpp_symbol("il2cpp_class_get_namespace");
    let get_declaring_ptr = resolve_il2cpp_symbol("il2cpp_class_get_declaring_type");
    if get_name_ptr.is_null() || get_namespace_ptr.is_null() || get_declaring_ptr.is_null() {
        return String::new();
    }
    let get_name: unsafe extern "C" fn(*mut c_void) -> *const c_char = std::mem::transmute(get_name_ptr);
    let get_namespace: unsafe extern "C" fn(*mut c_void) -> *const c_char = std::mem::transmute(get_namespace_ptr);
    let get_declaring: unsafe extern "C" fn(*mut c_void) -> *mut c_void = std::mem::transmute(get_declaring_ptr);
    let mut names = Vec::new();
    let mut current = class;
    let mut namespace = String::new();
    for _ in 0..64 {
        if current.is_null() { break; }
        names.push(il2cpp_c_string(get_name(current)));
        namespace = il2cpp_c_string(get_namespace(current));
        current = get_declaring(current);
    }
    names.reverse();
    let chain = names.join("/");
    if namespace.is_empty() { chain } else { format!("{}.{}", namespace, chain) }
}

unsafe fn find_class_by_full_declaring_name(requested: &str) -> *mut c_void {
    let image = get_image();
    if image.is_null() || requested.is_empty() { return ptr::null_mut(); }
    let (namespace, type_chain) = match requested.split_once('.') {
        Some((namespace, rest)) => (namespace, rest),
        None => ("", requested),
    };
    let mut names = type_chain.split('/');
    let outer = match names.next() { Some(value) if !value.is_empty() => value, _ => return ptr::null_mut() };
    let mut class = find_class(image, to_cstr(namespace).as_ptr(), to_cstr(outer).as_ptr());
    if class.is_null() { return ptr::null_mut(); }
    let nested_ptr = resolve_il2cpp_symbol("il2cpp_class_get_nested_types");
    let name_ptr = resolve_il2cpp_symbol("il2cpp_class_get_name");
    if names.clone().next().is_some() && (nested_ptr.is_null() || name_ptr.is_null()) { return ptr::null_mut(); }
    let get_nested: unsafe extern "C" fn(*mut c_void, *mut *mut c_void) -> *mut c_void = std::mem::transmute(nested_ptr);
    let get_name: unsafe extern "C" fn(*mut c_void) -> *const c_char = std::mem::transmute(name_ptr);
    for requested_nested in names {
        let mut iterator = ptr::null_mut();
        let mut found = ptr::null_mut();
        loop {
            let candidate = get_nested(class, &mut iterator);
            if candidate.is_null() { break; }
            if il2cpp_c_string(get_name(candidate)) == requested_nested {
                if !found.is_null() { return ptr::null_mut(); }
                found = candidate;
            }
        }
        if found.is_null() { return ptr::null_mut(); }
        class = found;
    }
    class
}

unsafe fn build_method_index() -> Result<Vec<MethodIndexEntry>, String> {
    let image = get_image();
    if image.is_null() { return Err("image_null".to_string()); }
    let symbols = [
        "il2cpp_image_get_class_count", "il2cpp_image_get_class", "il2cpp_class_get_methods",
        "il2cpp_method_get_name", "il2cpp_method_get_param_count", "il2cpp_method_get_param",
        "il2cpp_method_get_param_name", "il2cpp_method_get_return_type", "il2cpp_type_get_name",
        "il2cpp_method_get_flags", "il2cpp_method_get_class",
    ];
    let resolved: Vec<*mut c_void> = symbols.iter().map(|name| resolve_il2cpp_symbol(name)).collect();
    if let Some(index) = resolved.iter().position(|value| value.is_null()) {
        return Err(format!("missing_symbol:{}", symbols[index]));
    }
    let get_class_count: FnImageGetClassCount = std::mem::transmute(resolved[0]);
    let get_class: FnImageGetClass = std::mem::transmute(resolved[1]);
    let get_methods: FnClassGetMethods = std::mem::transmute(resolved[2]);
    let get_method_name: FnMethodGetName = std::mem::transmute(resolved[3]);
    let get_param_count: unsafe extern "C" fn(*const c_void) -> u32 = std::mem::transmute(resolved[4]);
    let get_param: unsafe extern "C" fn(*const c_void, u32) -> *const c_void = std::mem::transmute(resolved[5]);
    let get_param_name: unsafe extern "C" fn(*const c_void, u32) -> *const c_char = std::mem::transmute(resolved[6]);
    let get_return_type: unsafe extern "C" fn(*const c_void) -> *const c_void = std::mem::transmute(resolved[7]);
    let get_type_name: unsafe extern "C" fn(*const c_void) -> *const c_char = std::mem::transmute(resolved[8]);
    let get_flags: unsafe extern "C" fn(*const c_void, *mut u32) -> u32 = std::mem::transmute(resolved[9]);
    let get_method_class: unsafe extern "C" fn(*const c_void) -> *mut c_void = std::mem::transmute(resolved[10]);
    let mut entries = Vec::new();
    let class_count = get_class_count(image);
    for class_index in 0..class_count {
        let class = get_class(image, class_index);
        if class.is_null() { continue; }
        let mut iterator = ptr::null_mut();
        loop {
            let method_info = get_methods(class, &mut iterator);
            if method_info.is_null() { break; }
            let declaring_class = get_method_class(method_info);
            let declaring_type = class_full_declaring_name(declaring_class);
            let namespace = declaring_type.split_once('.').map(|(value, _)| value.to_string()).unwrap_or_default();
            let parameter_count = get_param_count(method_info);
            let mut parameter_names = Vec::with_capacity(parameter_count as usize);
            let mut parameter_types = Vec::with_capacity(parameter_count as usize);
            for parameter_index in 0..parameter_count {
                let parameter_type = get_param(method_info, parameter_index);
                parameter_types.push(if parameter_type.is_null() { "unresolved".to_string() } else { il2cpp_c_string(get_type_name(parameter_type)) });
                let parameter_name = il2cpp_c_string(get_param_name(method_info, parameter_index));
                parameter_names.push(if parameter_name.is_empty() { None } else { Some(parameter_name) });
            }
            let return_type_pointer = get_return_type(method_info);
            let return_type = if return_type_pointer.is_null() { "unresolved".to_string() } else { il2cpp_c_string(get_type_name(return_type_pointer)) };
            let mut iflags = 0u32;
            let flags = get_flags(method_info, &mut iflags);
            let method_pointer = if is_readable_range(method_info as usize, std::mem::size_of::<usize>()) {
                std::ptr::read_unaligned::<usize>(method_info as *const usize)
            } else { 0 };
            entries.push(MethodIndexEntry {
                method_info: method_info as usize,
                method_pointer,
                namespace,
                declaring_type,
                method_name: il2cpp_c_string(get_method_name(method_info)),
                return_type,
                parameter_names,
                parameter_types,
                flags,
            });
        }
    }
    entries.sort_by(|left, right| left.method_pointer.cmp(&right.method_pointer).then(left.method_info.cmp(&right.method_info)));
    Ok(entries)
}

unsafe fn ensure_method_index() -> Result<(), String> {
    {
        let mut state = METHOD_INDEX.lock().map_err(|_| "method_index_lock_poisoned".to_string())?;
        match state.status {
            "ready" => return Ok(()),
            "building" => return Err("method_index_building".to_string()),
            "failed" => return Err(state.error.clone()),
            _ => state.status = "building",
        }
    }
    let result = build_method_index();
    let mut state = METHOD_INDEX.lock().map_err(|_| "method_index_lock_poisoned".to_string())?;
    match result {
        Ok(entries) => {
            let class_count = {
                let image = get_image();
                let pointer = resolve_il2cpp_symbol("il2cpp_image_get_class_count");
                if image.is_null() || pointer.is_null() { 0 } else {
                    let function: FnImageGetClassCount = std::mem::transmute(pointer);
                    function(image)
                }
            };
            let null_count = entries.iter().filter(|entry| entry.method_pointer == 0).count();
            let mut duplicate_count = 0usize;
            let mut previous = 0usize;
            for entry in entries.iter().filter(|entry| entry.method_pointer != 0) {
                if entry.method_pointer == previous { duplicate_count += 1; }
                previous = entry.method_pointer;
            }
            state.image_class_count = class_count;
            state.indexed_class_count = class_count;
            state.indexed_method_count = entries.len();
            state.null_method_pointer_count = null_count;
            state.duplicate_method_pointer_count = duplicate_count;
            state.entries = entries;
            state.error.clear();
            state.status = "ready";
            Ok(())
        }
        Err(error) => {
            state.status = "failed";
            state.error = error.clone();
            Err(error)
        }
    }
}

fn method_entry_json(entry: &MethodIndexEntry, upper_bound: Option<usize>) -> String {
    let parameters = entry.parameter_types.iter().enumerate().map(|(index, parameter_type)| {
        let name = entry.parameter_names.get(index).and_then(|value| value.as_ref())
            .map(|value| format!("\"{}\"", json_escape(value))).unwrap_or_else(|| "null".to_string());
        format!(r#"{{"index":{},"name":{},"type":"{}"}}"#, index, name, json_escape(parameter_type))
    }).collect::<Vec<_>>().join(",");
    let upper = upper_bound.map(|value| format!("\"0x{:x}\"", value)).unwrap_or_else(|| "null".to_string());
    format!(r#"{{"method_info":"0x{:x}","method_pointer":"0x{:x}","namespace":"{}","declaring_type":"{}","method":"{}","return_type":"{}","parameters":[{}],"flags":{},"static":{},"next_distinct_pointer_upper_bound":{},"boundary_kind":"upper_bound_estimate"}}"#,
        entry.method_info, entry.method_pointer, json_escape(&entry.namespace), json_escape(&entry.declaring_type),
        json_escape(&entry.method_name), json_escape(&entry.return_type), parameters, entry.flags,
        (entry.flags & 0x0010) != 0, upper)
}

unsafe fn il2cpp_method_by_addr(uri: &str) -> String {
    let pairs = match parse_query_pairs(uri) { Ok(value) => value, Err(error) => return format!(r#"{{"ok":false,"error":"{}"}}"#, json_escape(&error)) };
    let raw = query_pair(&pairs, "addr");
    let address = match parse_address(&raw) { Some(value) if value != 0 => value, _ => return r#"{"ok":false,"error":"invalid_or_missing_addr"}"#.to_string() };
    if let Err(error) = ensure_method_index() { return format!(r#"{{"ok":false,"error":"{}"}}"#, json_escape(&error)); }
    let state = match METHOD_INDEX.lock() { Ok(value) => value, Err(_) => return r#"{"ok":false,"error":"method_index_lock_poisoned"}"#.to_string() };
    let method_info_matches: Vec<&MethodIndexEntry> = state.entries.iter().filter(|entry| entry.method_info == address).collect();
    let exact_pointer_matches: Vec<&MethodIndexEntry> = state.entries.iter().filter(|entry| entry.method_pointer == address && entry.method_pointer != 0).collect();
    let (kind, matches): (&str, Vec<&MethodIndexEntry>) = if !method_info_matches.is_empty() {
        (if method_info_matches.len() == 1 { "exact_method_info" } else { "ambiguous" }, method_info_matches)
    } else if !exact_pointer_matches.is_empty() {
        (if exact_pointer_matches.len() == 1 { "exact_method_pointer" } else { "ambiguous" }, exact_pointer_matches)
    } else {
        let mut distinct: Vec<usize> = state.entries.iter().map(|entry| entry.method_pointer).filter(|value| *value != 0).collect();
        distinct.dedup();
        match distinct.binary_search(&address) {
            Ok(_) => ("none", Vec::new()),
            Err(position) if position > 0 && position < distinct.len() => {
                let start = distinct[position - 1];
                let candidates: Vec<&MethodIndexEntry> = state.entries.iter().filter(|entry| entry.method_pointer == start).collect();
                (if candidates.len() == 1 { "upper_bound_candidate" } else { "ambiguous" }, candidates)
            }
            _ => ("none", Vec::new()),
        }
    };
    let items = matches.iter().map(|entry| {
        let upper = state.entries.iter().map(|candidate| candidate.method_pointer).filter(|pointer| *pointer > entry.method_pointer).min();
        method_entry_json(entry, upper)
    }).collect::<Vec<_>>().join(",");
    format!(r#"{{"ok":true,"query":"0x{:x}","status":"{}","ambiguous":{},"matches":[{}],"index":{{"status":"{}","classes":{},"methods":{},"null_method_pointers":{},"duplicate_method_pointers":{}}}}}"#,
        address, kind, kind == "ambiguous", items, state.status, state.indexed_class_count,
        state.indexed_method_count, state.null_method_pointer_count, state.duplicate_method_pointer_count)
}

unsafe fn il2cpp_method_detail(uri: &str) -> String {
    let pairs = match parse_query_pairs(uri) { Ok(value) => value, Err(error) => return format!(r#"{{"ok":false,"error":"{}"}}"#, json_escape(&error)) };
    let namespace = query_pair(&pairs, "namespace");
    let declaring_type = query_pair(&pairs, "declaring_type");
    let method = query_pair(&pairs, "method");
    let parameter_text = query_pair(&pairs, "parameter_types");
    if declaring_type.is_empty() || method.is_empty() { return r#"{"ok":false,"error":"missing_declaring_type_or_method"}"#.to_string(); }
    let parameter_types: Vec<String> = if parameter_text.is_empty() { Vec::new() } else { parameter_text.split(',').map(|value| value.trim().to_string()).collect() };
    if let Err(error) = ensure_method_index() { return format!(r#"{{"ok":false,"error":"{}"}}"#, json_escape(&error)); }
    let state = match METHOD_INDEX.lock() { Ok(value) => value, Err(_) => return r#"{"ok":false,"error":"method_index_lock_poisoned"}"#.to_string() };
    let matches: Vec<&MethodIndexEntry> = state.entries.iter().filter(|entry| {
        (namespace.is_empty() || entry.namespace == namespace) && entry.declaring_type == declaring_type &&
        entry.method_name == method && entry.parameter_types == parameter_types
    }).collect();
    let status = if matches.is_empty() { "none" } else if matches.len() == 1 { "exact" } else { "ambiguous" };
    let items = matches.iter().map(|entry| {
        let upper = state.entries.iter().map(|candidate| candidate.method_pointer).filter(|pointer| *pointer > entry.method_pointer).min();
        method_entry_json(entry, upper)
    }).collect::<Vec<_>>().join(",");
    format!(r#"{{"ok":true,"status":"{}","ambiguous":{},"matches":[{}]}}"#, status, status == "ambiguous", items)
}

unsafe fn il2cpp_nested_types(uri: &str) -> String {
    let pairs = match parse_query_pairs(uri) { Ok(value) => value, Err(error) => return format!(r#"{{"ok":false,"error":"{}"}}"#, json_escape(&error)) };
    let requested = query_pair(&pairs, "type");
    if requested.is_empty() { return r#"{"ok":false,"error":"missing_type"}"#.to_string(); }
    let class = find_class_by_full_declaring_name(&requested);
    if class.is_null() { return format!(r#"{{"ok":false,"error":"class_not_found_or_ambiguous","type":"{}"}}"#, json_escape(&requested)); }
    let nested_ptr = resolve_il2cpp_symbol("il2cpp_class_get_nested_types");
    if nested_ptr.is_null() { return r#"{"ok":false,"error":"il2cpp_class_get_nested_types_unavailable"}"#.to_string(); }
    let get_nested: unsafe extern "C" fn(*mut c_void, *mut *mut c_void) -> *mut c_void = std::mem::transmute(nested_ptr);
    let mut iterator = ptr::null_mut();
    let mut items = Vec::new();
    loop {
        let nested = get_nested(class, &mut iterator);
        if nested.is_null() { break; }
        items.push(format!(r#"{{"type":"{}","class_pointer":"0x{:x}"}}"#, json_escape(&class_full_declaring_name(nested)), nested as usize));
    }
    format!(r#"{{"ok":true,"requested":"{}","direct_only":true,"count":{},"nested_types":[{}]}}"#, json_escape(&requested), items.len(), items.join(","))
}

unsafe fn il2cpp_enum_values_capability(uri: &str) -> String {
    let pairs = match parse_query_pairs(uri) { Ok(value) => value, Err(error) => return format!(r#"{{"ok":false,"error":"{}"}}"#, json_escape(&error)) };
    let requested = query_pair(&pairs, "type");
    let required = ["il2cpp_class_get_fields", "il2cpp_field_get_flags", "il2cpp_field_static_get_value"];
    let available: Vec<bool> = required.iter().map(|name| !resolve_il2cpp_symbol(name).is_null()).collect();
    format!(r#"{{"ok":true,"requested":"{}","value_status":"unresolved","integer_values":null,"declaration_order_inference":false,"runtime_api":{{"il2cpp_class_get_fields":{},"il2cpp_field_get_flags":{},"il2cpp_field_static_get_value":{}}}}}"#,
        json_escape(&requested), available[0], available[1], available[2])
}

'''
s = s.replace(anchor, rust + anchor, 1)

route_anchor = '''    } else if path.starts_with("/il2cpp/methods") {
        // v3.22.89: 列出类的所有方法名和参数数量
'''
assert s.count(route_anchor) == 1, f"route anchor count={s.count(route_anchor)}"
route = '''    } else if path == "/il2cpp/method_by_addr" {
        unsafe { il2cpp_method_by_addr(&full_uri) }
    } else if path == "/il2cpp/method_detail" {
        unsafe { il2cpp_method_detail(&full_uri) }
    } else if path == "/il2cpp/nested_types" {
        unsafe { il2cpp_nested_types(&full_uri) }
    } else if path == "/il2cpp/enum_values" {
        unsafe { il2cpp_enum_values_capability(&full_uri) }
'''
s = s.replace(route_anchor, route + route_anchor, 1)

# Integrate the strict request-line parser without depending on parse_path for query data.
# The existing full_uri remains the routing source; new handlers decode it themselves.
request_anchor = '    let body = if path == "/" || path == "/health" {\n'
assert s.count(request_anchor) == 1, f"request parser integration anchor count={s.count(request_anchor)}"
s = s.replace(request_anchor, '''    let _parsed_request_uri = parse_request_uri(req).unwrap_or_else(|_| full_uri.clone());
    let body = if path == "/debug/global_metadata_probe" {
''', 1)

SOURCE.write_text(s, encoding="utf-8")
print("unified_endpoint_a_patch=applied")
