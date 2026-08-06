# `build_method_index`

source_commit: `a340a147acf13672b2fbc64925bfa321d08091fd`
source_line: `22470`

```rust
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
```
