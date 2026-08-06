# Parent multi-source I exact runtime helpers

## `unsafe fn call_getter_ref`

match_count=1

```rust
unsafe fn call_getter_ref(
    class: *mut c_void,
    instance: *const c_void,
    method_name: &str,
) -> *mut c_void {
    call_getter_on_instance(class, instance, method_name)
}
```

## `unsafe fn call_getter_int`

match_count=3

```rust
unsafe fn call_getter_int(class: *mut c_void, instance: *const c_void, method_name: &str) -> i32 {
    if class.is_null() || instance.is_null() {
        return -1;
    }

    let result = call_getter_on_instance(class, instance, method_name);
    if result.is_null() {
        return -1;
    }

    // Value type (int/enum) is boxed: real value at offset +16
    let val_ptr = result as *const u8;
    let int_val = std::ptr::read_unaligned::<i32>(val_ptr.add(16) as *const i32);
    int_val
}
```

```rust
unsafe fn call_getter_int_with_arg(
    class: *mut c_void,
    instance: *const c_void,
    method_name: &str,
    int_arg: i32,
) -> i32 {
    if class.is_null() || instance.is_null() {
        return -1;
    }
    let get_method_ptr = resolve_il2cpp_symbol("il2cpp_class_get_method_from_name");
    let invoke_ptr = resolve_il2cpp_symbol("il2cpp_runtime_invoke");
    if get_method_ptr.is_null() || invoke_ptr.is_null() {
        return -1;
    }
    let get_method: FnClassGetMethodFromName = std::mem::transmute(get_method_ptr);
    let invoke: FnRuntimeInvoke = std::mem::transmute(invoke_ptr);
    let method_info = get_method(class, to_cstr(method_name).as_ptr(), 1);
    if method_info.is_null() {
        ura_log(4, &format!("call_int_with_arg: '{}' not found", method_name));
        return -1;
    }

    // il2cpp_runtime_invoke expects argv entries to point to unboxed value data.
    let mut arg = int_arg;
    let mut args = [&mut arg as *mut i32 as *mut c_void];
    let mut exc: *mut c_void = ptr::null_mut();
    let result = invoke(
        method_info,
        instance as *mut c_void,
        args.as_mut_ptr(),
        &mut exc,
    );
    if !exc.is_null() || result.is_null() {
        return -1;
    }
    std::ptr::read_unaligned::<i32>((result as *const u8).add(16) as *const i32)
}
```

```rust
unsafe fn call_getter_int_raw(obj: *const c_void, method_name: &str) -> i32 {
    if obj.is_null() || API.is_null() {
        return 0;
    }
    // We need to find the class and method. For simplicity, we use the method pointer approach.
    // Since we don't know the class, we try to read directly from known offsets.
    // StoryChoiceParam is a simple struct, we can try field offsets.
    // Actually, let's use the proper IL2CPP approach.
    let api = &*API;
    let get_class_fn: Option<unsafe extern "C" fn(*const c_void) -> *mut c_void> = {
        let p = resolve_il2cpp_symbol("il2cpp_object_get_class");
        if p.is_null() {
            None
        } else {
            Some(std::mem::transmute(p))
        }
    };
    if get_class_fn.is_none() {
        return 0;
    }
    let class = get_class_fn.unwrap()(obj);
    if class.is_null() {
        return 0;
    }
    call_getter_int(class, obj, method_name)
}
```

## `unsafe fn call_getter_obscured_int`

match_count=1

```rust
unsafe fn call_getter_obscured_int(
    class: *mut c_void,
    instance: *const c_void,
    method_name: &str,
) -> i32 {
    if class.is_null() || instance.is_null() {
        return -1;
    }

    let result = call_getter_on_instance(class, instance, method_name);
    if result.is_null() {
        return -1;
    }

    // Boxed ObscuredInt struct layout (from dump.cs Anti-Cheat Toolkit):
    // offset 0x10: currentCryptoKey (Int32) — the decryption key
    // offset 0x14: hiddenValue (Int32) — the encrypted value
    // offset 0x18: inited (Boolean)
    // offset 0x1C: fakeValue (Int32)
    // offset 0x20: fakeValueActive (Boolean)
    let base = result as *const u8;

    let current_crypto_key =
        std::ptr::read_unaligned::<i32>(base.add(IL2CPP_OBSCURED_INT_KEY_OFF) as *const i32);
    let hidden_value =
        std::ptr::read_unaligned::<i32>(base.add(IL2CPP_OBSCURED_INT_HIDDEN_OFF) as *const i32);

    // Decrypt: hiddenValue ^ currentCryptoKey
    let decrypted = hidden_value ^ current_crypto_key;

    ura_log(
        4,
        &format!(
            "ObscuredInt {}: hidden={} key={} decrypted={}",
            method_name, hidden_value, current_crypto_key, decrypted
        ),
    );

    decrypted
}
```

## `unsafe fn call_getter_bool`

match_count=1

```rust
unsafe fn call_getter_bool(class: *mut c_void, instance: *const c_void, method_name: &str) -> bool {
    call_getter_int(class, instance, method_name) != 0
}
```

## `unsafe fn find_exact_instance_method`

match_count=0

## `unsafe fn invoke_parent_store_get`

match_count=0

## `unsafe fn selected_parent_record_json`

match_count=0

## `unsafe fn inherit_selected_parent_records_endpoint`

match_count=0

