# Response header hook source contexts

## `unity_send_hook_handler` (3 matches)

### match 1

```rust
w()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis().min(u128::from(u64::MAX)) as u64)
        .unwrap_or(0)
}

fn unity_observer_path(url: &str) -> String {
    let no_query = url.split('?').next().unwrap_or(url);
    if let Some(scheme) = no_query.find("://") {
        let rest = &no_query[scheme + 3..];
        return rest
            .find('/')
            .map(|i| rest[i..].to_string())
            .unwrap_or_else(|| "/".to_string());
    }
    no_query.to_string()
}

unsafe fn unity_get_string(obj: *const c_void, getter: &str) -> String {
    if obj.is_null() {
        return String::new();
    }
    let class = get_class_from_object(obj);
    if class.is_null() {
        return String::new();
    }
    read_il2cpp_string(call_getter_on_instance(class, obj, getter))
}

unsafe fn observe_unity_web_request(request: *mut c_void) {
    if request.is_null() || !SNIFF_ENABLED.load(Ordering::Relaxed) {
        return;
    }
    let method = unity_get_string(request, "get_method");
    let url = unity_get_string(request, "get_url");
    let request_class = get_class_from_object(request);
    let upload = if request_class.is_null() {
        ptr::null_mut()
    } else {
        call_getter_ref(request_class, request, "get_uploadHandler")
    };
    let (body_size, body_hex, content_type) = if upload.is_null() {
        (0, String::new(), String::new())
    } else {
        let upload_class = get_class_from_object(upload);
        let data = if upload_class.is_null() {
            ptr::null_mut()
        } else {
            call_getter_on_instance(upload_class, upload, "get_data")
        };
        let body_bytes = read_il2cpp_byte_array(data);
        let body_hex = body_bytes.iter().map(|b| format!("{:02x}", b)).collect::<String>();
        (
            body_bytes.len(),
            body_hex,
            unity_get_string(upload, "get_contentType"),
        )
    };
    let item = UnityRequestObservation {
        id: UNITY_OBSERVATION_ID.fetch_add(1, Ordering::Relaxed),
        timestamp_ms: unity_observer_timestamp_ms(),
        method,
        path: unity_observer_path(&url),
        body_size,
        body_hex,
        content_type,
    };
    if let Ok(mut entries) = UNITY_OBSERVATIONS.lock() {
        if entries.len() >= UNITY_OBSERVATIONS_MAX {
            entries.remove(0);
        }
        entries.push(item);
    }
}

// UnityWebRequest.SendWebRequest() is asynchronous; this observes request entry only.
extern "C" fn unity_send_hook_handler(this: *mut c_void) -> *mut c_void {
    unsafe {
        let trampoline = interceptor_get_trampoline(unity_send_hook_handler as usize);
        if trampoline == 0 {
            return ptr::null_mut();
        }
        type FnType = unsafe extern "C" fn(*mut c_void) -> *mut c_void;
        let original: FnType = std::mem::transmute(trampoline);
        // Observation failures must never block or replace the game's request.
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            observe_unity_web_request(this);
        }));
        original(this)
    }
}

// MakeMd5(string input) -> string
// Hook to capture MD5 input (contains salt) and output
extern "C" fn makemd5_hook_handler(input: *mut c_void) -> *mut c_void {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
        // Read input string before calling original
        let input_str = if !input.is_null() {
            read_il2cpp_string(input)
        } else {
            String::new()
        };
        
        let trampoline = interceptor_get_trampoline(makemd5_hook_handler as usize);
        if trampoline == 0 {
            return std::ptr::null_mut();
        }
        type FnType = unsafe extern "C" fn(*mut c_void) -> *mut c_void;
        let original: FnType = std::mem::transmute(trampoline);
        let ret = original(input);
        
        // Read output string
        let output_str = if !ret.is_null() {
            read_il2cpp_string(ret)
        } else {
            String::new()
        };
        
        // Log input + output
        if !input_str.is_empty() {
            if let Ok(mut log) = MD5_LOG.lock() {
                if log.len() >= 100 {
                    log.remove(0);
                }
                log.push((input_str, output_str));
            }
        }
        
        ret
    }));
    result.unwrap_or(std::ptr::null_mut())
}

// ComputeHash(string input) -> string
// Hook to capture intermediate data — if MakeMd5 calls ComputeHash internally,
// the input here will be the salted string (input + salt)
extern "C" fn computehash_hook_handler(input: *mut c_void) -> *mut c_void {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
        let input_str = if !input.is_null() {
            read_il2cpp_string(input)
        } else {
            String::new()
        };

        let trampoline = interceptor_get_trampoline(computehash_hook_handler as usize);
        if trampoline == 0 {
            return std::ptr::null_mut();
        }
        type FnType = unsafe extern "C" fn(*mut c_void) -> *mut c_void;
        let original: FnType = std::mem::transmute(trampoline);
        let ret = original(input);

        let output_str = if !ret.is_null() {
            read_il2cpp_string(ret)
        } else {
            String::new()
        };

        // Log with "CH:" prefix to distinguish from MakeMd5 entries
        if !input_str.is_empty() {
            if let Ok(mut log) = MD5_LOG.lock() {
                if log.len() >= 100 {
                    log.remove(0);
                }
                log.push((format!("CH:{}", input_str), output_str));
            }
        }

        ret
    }));
    result.unwrap_or(std::ptr::null_mut())
}

// ★ v3.23.3: Hook handler for CompressRequest(byte[] data) -> byte[]
// Parks the uncompressed request body, keyed by the compressed byte array returned by the original.
// WWWRequest.Post will match it later.
extern "C" fn compress_request_hook_handler(data: *mut c_void) -> *mut c_void {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
        let body = read_il2cpp_byte_array(data);
        let trampoline = interceptor_get_trampoline(compress_request_hook_handler as usize);
        if trampoline == 0 {
            return std::ptr::null_mut();
        }
        type FnType = unsafe extern "C" fn(*mut c_void) -> *mut c_void;
        let original: FnType = std::mem::transmute(trampoline);
        let compressed = original(data);
        if !body.is_empty() && POST_ADDR != 0 {
            PENDING_REQ_BODY = Some(body);
            PENDING_COMPRESSED = compressed as usize;
        }
        compressed
    }));
    result.unwrap_or_else(|e| {
        unsafe {
            ura_log(1, &format!("compress_hook panic: {:?}", e));
        }
        std::ptr::null_mut()
    })
}

// ★ v3.23.3: Hook handler for DecompressResponse(byte[] data) -> byte[]
// Forwards the decompressed response body with the matching request's URL + headers.
extern "C" fn decompress_response_hook_handler(data: *mut c_void) -> *mut c_void {
    unsafe {
        let trampoline = interceptor_get_trampoline(decompress_response_hook_handler as usize);
        if trampoline == 0 {
            return std::ptr::null_mut();
        }
        type FnType = unsafe extern "C" fn(*mut c_void) -> *mut c_void;
        let original: FnType = std::mem::transmute(trampoline);
        let decomp
```

### match 2

```rust
unwrap_or(0)
}

fn unity_observer_path(url: &str) -> String {
    let no_query = url.split('?').next().unwrap_or(url);
    if let Some(scheme) = no_query.find("://") {
        let rest = &no_query[scheme + 3..];
        return rest
            .find('/')
            .map(|i| rest[i..].to_string())
            .unwrap_or_else(|| "/".to_string());
    }
    no_query.to_string()
}

unsafe fn unity_get_string(obj: *const c_void, getter: &str) -> String {
    if obj.is_null() {
        return String::new();
    }
    let class = get_class_from_object(obj);
    if class.is_null() {
        return String::new();
    }
    read_il2cpp_string(call_getter_on_instance(class, obj, getter))
}

unsafe fn observe_unity_web_request(request: *mut c_void) {
    if request.is_null() || !SNIFF_ENABLED.load(Ordering::Relaxed) {
        return;
    }
    let method = unity_get_string(request, "get_method");
    let url = unity_get_string(request, "get_url");
    let request_class = get_class_from_object(request);
    let upload = if request_class.is_null() {
        ptr::null_mut()
    } else {
        call_getter_ref(request_class, request, "get_uploadHandler")
    };
    let (body_size, body_hex, content_type) = if upload.is_null() {
        (0, String::new(), String::new())
    } else {
        let upload_class = get_class_from_object(upload);
        let data = if upload_class.is_null() {
            ptr::null_mut()
        } else {
            call_getter_on_instance(upload_class, upload, "get_data")
        };
        let body_bytes = read_il2cpp_byte_array(data);
        let body_hex = body_bytes.iter().map(|b| format!("{:02x}", b)).collect::<String>();
        (
            body_bytes.len(),
            body_hex,
            unity_get_string(upload, "get_contentType"),
        )
    };
    let item = UnityRequestObservation {
        id: UNITY_OBSERVATION_ID.fetch_add(1, Ordering::Relaxed),
        timestamp_ms: unity_observer_timestamp_ms(),
        method,
        path: unity_observer_path(&url),
        body_size,
        body_hex,
        content_type,
    };
    if let Ok(mut entries) = UNITY_OBSERVATIONS.lock() {
        if entries.len() >= UNITY_OBSERVATIONS_MAX {
            entries.remove(0);
        }
        entries.push(item);
    }
}

// UnityWebRequest.SendWebRequest() is asynchronous; this observes request entry only.
extern "C" fn unity_send_hook_handler(this: *mut c_void) -> *mut c_void {
    unsafe {
        let trampoline = interceptor_get_trampoline(unity_send_hook_handler as usize);
        if trampoline == 0 {
            return ptr::null_mut();
        }
        type FnType = unsafe extern "C" fn(*mut c_void) -> *mut c_void;
        let original: FnType = std::mem::transmute(trampoline);
        // Observation failures must never block or replace the game's request.
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            observe_unity_web_request(this);
        }));
        original(this)
    }
}

// MakeMd5(string input) -> string
// Hook to capture MD5 input (contains salt) and output
extern "C" fn makemd5_hook_handler(input: *mut c_void) -> *mut c_void {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
        // Read input string before calling original
        let input_str = if !input.is_null() {
            read_il2cpp_string(input)
        } else {
            String::new()
        };
        
        let trampoline = interceptor_get_trampoline(makemd5_hook_handler as usize);
        if trampoline == 0 {
            return std::ptr::null_mut();
        }
        type FnType = unsafe extern "C" fn(*mut c_void) -> *mut c_void;
        let original: FnType = std::mem::transmute(trampoline);
        let ret = original(input);
        
        // Read output string
        let output_str = if !ret.is_null() {
            read_il2cpp_string(ret)
        } else {
            String::new()
        };
        
        // Log input + output
        if !input_str.is_empty() {
            if let Ok(mut log) = MD5_LOG.lock() {
                if log.len() >= 100 {
                    log.remove(0);
                }
                log.push((input_str, output_str));
            }
        }
        
        ret
    }));
    result.unwrap_or(std::ptr::null_mut())
}

// ComputeHash(string input) -> string
// Hook to capture intermediate data — if MakeMd5 calls ComputeHash internally,
// the input here will be the salted string (input + salt)
extern "C" fn computehash_hook_handler(input: *mut c_void) -> *mut c_void {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
        let input_str = if !input.is_null() {
            read_il2cpp_string(input)
        } else {
            String::new()
        };

        let trampoline = interceptor_get_trampoline(computehash_hook_handler as usize);
        if trampoline == 0 {
            return std::ptr::null_mut();
        }
        type FnType = unsafe extern "C" fn(*mut c_void) -> *mut c_void;
        let original: FnType = std::mem::transmute(trampoline);
        let ret = original(input);

        let output_str = if !ret.is_null() {
            read_il2cpp_string(ret)
        } else {
            String::new()
        };

        // Log with "CH:" prefix to distinguish from MakeMd5 entries
        if !input_str.is_empty() {
            if let Ok(mut log) = MD5_LOG.lock() {
                if log.len() >= 100 {
                    log.remove(0);
                }
                log.push((format!("CH:{}", input_str), output_str));
            }
        }

        ret
    }));
    result.unwrap_or(std::ptr::null_mut())
}

// ★ v3.23.3: Hook handler for CompressRequest(byte[] data) -> byte[]
// Parks the uncompressed request body, keyed by the compressed byte array returned by the original.
// WWWRequest.Post will match it later.
extern "C" fn compress_request_hook_handler(data: *mut c_void) -> *mut c_void {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
        let body = read_il2cpp_byte_array(data);
        let trampoline = interceptor_get_trampoline(compress_request_hook_handler as usize);
        if trampoline == 0 {
            return std::ptr::null_mut();
        }
        type FnType = unsafe extern "C" fn(*mut c_void) -> *mut c_void;
        let original: FnType = std::mem::transmute(trampoline);
        let compressed = original(data);
        if !body.is_empty() && POST_ADDR != 0 {
            PENDING_REQ_BODY = Some(body);
            PENDING_COMPRESSED = compressed as usize;
        }
        compressed
    }));
    result.unwrap_or_else(|e| {
        unsafe {
            ura_log(1, &format!("compress_hook panic: {:?}", e));
        }
        std::ptr::null_mut()
    })
}

// ★ v3.23.3: Hook handler for DecompressResponse(byte[] data) -> byte[]
// Forwards the decompressed response body with the matching request's URL + headers.
extern "C" fn decompress_response_hook_handler(data: *mut c_void) -> *mut c_void {
    unsafe {
        let trampoline = interceptor_get_trampoline(decompress_response_hook_handler as usize);
        if trampoline == 0 {
            return std::ptr::null_mut();
        }
        type FnType = unsafe extern "C" fn(*mut c_void) -> *mut c_void;
        let original: FnType = std::mem::transmute(trampoline);
        let decompressed = original(data);
        let bytes = read_il2cpp_byte_array(decompressed);
        if !bytes.is_empty() {
           
```

### match 3

```rust
       let name = CStr::from_ptr(np).to_string_lossy();
        if name.contains(substr) {
            ura_log(3, &format!("find_class_fuzzy: {}~{}", substr, name));
            return cls as *mut c_void;
        }
    }
    ptr::null_mut()
}

unsafe fn install_api_sniff_hooks() {
    let all_hooked = COMPRESS_REQUEST_ADDR != 0
        && DECOMPRESS_RESPONSE_ADDR != 0
        && POST_ADDR != 0
        && UNITY_SEND_ADDR != 0;
    if all_hooked {
        return;
    }
    if API.is_null() {
        ura_log(3, "API sniff: API is null");
        set_hook_status("sniff", "failed: api_null");
        return;
    }
    let api = &*API;
    if api.interceptor == 0 {
        ura_log(3, "API sniff: interceptor not available");
        set_hook_status("sniff", "failed: interceptor_unavailable");
        return;
    }

    // Get umamusume.dll assembly image
    let get_asm = match api.il2cpp_get_assembly_image_fn {
        Some(f) => f,
        None => {
            ura_log(3, "API sniff: get_assembly_image not available");
            return;
        }
    };
    let get_class = match api.il2cpp_get_class_fn {
        Some(f) => f,
        None => {
            ura_log(3, "API sniff: get_class not available");
            return;
        }
    };
    let get_method_addr = match api.il2cpp_get_method_addr_fn {
        Some(f) => f,
        None => {
            ura_log(3, "API sniff: get_method_addr not available");
            return;
        }
    };

    // Observe the lower UnityWebRequest request-entry path used by boot/auth traffic.
    if UNITY_SEND_ADDR == 0 {
        let unity_image = get_asm(to_cstr("UnityEngine.UnityWebRequestModule.dll").as_ptr());
        if unity_image.is_null() {
            set_hook_status("sniff.unity_send", "failed: image_not_found");
        } else {
            let unity_request = get_class(
                unity_image,
                to_cstr("UnityEngine.Networking").as_ptr(),
                to_cstr("UnityWebRequest").as_ptr(),
            );
            if unity_request.is_null() {
                set_hook_status("sniff.unity_send", "failed: class_not_found");
            } else {
                let addr = get_method_addr(
                    unity_request as usize,
                    to_cstr("SendWebRequest").as_ptr(),
                    0,
                );
                if addr == 0 {
                    set_hook_status("sniff.unity_send", "failed: method_not_found");
                } else if interceptor_hook(addr, unity_send_hook_handler as usize) {
                    UNITY_SEND_ADDR = addr;
                    set_hook_status("sniff.unity_send", &format!("hooked@0x{:x}", addr));
                    ura_log(
                        3,
                        &format!(
                            "API sniff: UnityWebRequest.SendWebRequest hooked at 0x{:x}",
                            addr
                        ),
                    );
                } else {
                    set_hook_status("sniff.unity_send", "failed: interceptor_hook");
                }
            }
        }
    }

    // Hook Cryptographer.MakeMd5 to capture salt
    if MAKEMD5_ADDR == 0 {
        let umamusume_img = get_asm(to_cstr("umamusume.dll").as_ptr());
        if !umamusume_img.is_null() {
            let crypto_class = get_class(
                umamusume_img,
                to_cstr("Gallop").as_ptr(),
                to_cstr("Cryptographer").as_ptr(),
            );
            if !crypto_class.is_null() {
                let addr = get_method_addr(
                    crypto_class as usize,
                    to_cstr("MakeMd5").as_ptr(),
                    1,
                );
                if addr != 0 {
                    if interceptor_hook(addr, makemd5_hook_handler as usize) {
                        MAKEMD5_ADDR = addr;
                        set_hook_status("sniff.makemd5", &format!("hooked@0x{:x}", addr));
                        ura_log(3, &format!("API sniff: Cryptographer.MakeMd5 hooked at 0x{:x}", addr));
                    } else {
                        set_hook_status("sniff.makemd5", "failed: interceptor_hook");
                    }
                }
                // Also hook ComputeHash to capture intermediate data (salted input)
                let ch_addr = get_method_addr(
                    crypto_class as usize,
                    to_cstr("ComputeHash").as_ptr(),
                    1,
                );
                if ch_addr != 0 {
                    if interceptor_hook(ch_addr, computehash_hook_handler as usize) {
                        COMPUTEHASH_ADDR = ch_addr;
                        set_hook_status("sniff.computehash", &format!("hooked@0x{:x}", ch_addr));
                        ura_log(3, &format!("API sniff: Cryptographer.ComputeHash hooked at 0x{:x}", ch_addr));
                    }
                }
            }
        }
    }

    let umamusume = get_asm(to_cstr("umamusume.dll").as_ptr());
    if umamusume.is_null() {
        ura_log(3, "API sniff: umamusume.dll image not found");
        set_hook_status("sniff", "failed: image_not_found");
        return;
    }

    // HttpHelper class (exact, then fuzzy fallback — v3.24.40)
    let mut http_helper = get_class(
        umamusume,
        to_cstr("Gallop").as_ptr(),
        to_cstr("HttpHelper").as_ptr(),
    );
    if http_helper.is_null() {
        http_helper = find_class_fuzzy(umamusume, "HttpHelper");
    }
    if http_helper.is_null() {
        ura_log(3, "API sniff: HttpHelper class not found");
        set_hook_status("sniff", "failed: httphelper_class_not_found");
        return;
    }
    ura_log(3, "API sniff: HttpHelper class found");

    // Hook CompressRequest
    if COMPRESS_REQUEST_ADDR == 0 {
        let mut addr =
            get_method_addr(http_helper as usize, to_cstr("CompressRequest").as_ptr(), 1);
        if addr == 0 {
            addr = find_method_fuzzy(http_helper, "CompressRequest");
        }
        if addr != 0 {
            if interceptor_hook(addr, compress_request_hook_handler as usize) {
                COMPRESS_REQUEST_ADDR = addr;
                ura_log(
                    3,
                    &format!("API sniff: CompressRequest hooked at 0x{:x}", addr),
                );
                set_hook_status("sniff.compress", &format!("hooked@0x{:x}", addr));
            } else {
                ura_log(
                    3,
                    &format!("API sniff: CompressRequest hook FAILED at 0x{:x}", addr),
                );
                set_hook_status("sniff.compress", "failed: interceptor_hook");
            }
        } else {
            ura_log(3, "API sniff: CompressRequest NOT FOUND");
            set_hook_status("sniff.compress", "failed: method_not_found");
        }
    }

    // Hook DecompressResponse
    if DECOMPRESS_RESPONSE_ADDR == 0 {
        let addr = get_method_addr(
            http_helper as usize,
            to_cstr("DecompressResponse").as_ptr(),
            1,
        );
        if addr != 0 {
            if interceptor_hook(addr, decompress_response_hook_handler as usize) {
                DECOMPRESS_RESPONSE_ADDR = addr;
                ura_log(
                    3,
                    &format!("API sniff: DecompressResponse hooked at 0x{:x}", addr),
                );
                set_hook_status("sniff.decompress", &format!("hooked@0x{:x}", addr));
            } else {
                ura_log(
                    3,
                 
```

## `decompress_response_hook_handler` (3 matches)

### match 1

```rust
lted string (input + salt)
extern "C" fn computehash_hook_handler(input: *mut c_void) -> *mut c_void {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
        let input_str = if !input.is_null() {
            read_il2cpp_string(input)
        } else {
            String::new()
        };

        let trampoline = interceptor_get_trampoline(computehash_hook_handler as usize);
        if trampoline == 0 {
            return std::ptr::null_mut();
        }
        type FnType = unsafe extern "C" fn(*mut c_void) -> *mut c_void;
        let original: FnType = std::mem::transmute(trampoline);
        let ret = original(input);

        let output_str = if !ret.is_null() {
            read_il2cpp_string(ret)
        } else {
            String::new()
        };

        // Log with "CH:" prefix to distinguish from MakeMd5 entries
        if !input_str.is_empty() {
            if let Ok(mut log) = MD5_LOG.lock() {
                if log.len() >= 100 {
                    log.remove(0);
                }
                log.push((format!("CH:{}", input_str), output_str));
            }
        }

        ret
    }));
    result.unwrap_or(std::ptr::null_mut())
}

// ★ v3.23.3: Hook handler for CompressRequest(byte[] data) -> byte[]
// Parks the uncompressed request body, keyed by the compressed byte array returned by the original.
// WWWRequest.Post will match it later.
extern "C" fn compress_request_hook_handler(data: *mut c_void) -> *mut c_void {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
        let body = read_il2cpp_byte_array(data);
        let trampoline = interceptor_get_trampoline(compress_request_hook_handler as usize);
        if trampoline == 0 {
            return std::ptr::null_mut();
        }
        type FnType = unsafe extern "C" fn(*mut c_void) -> *mut c_void;
        let original: FnType = std::mem::transmute(trampoline);
        let compressed = original(data);
        if !body.is_empty() && POST_ADDR != 0 {
            PENDING_REQ_BODY = Some(body);
            PENDING_COMPRESSED = compressed as usize;
        }
        compressed
    }));
    result.unwrap_or_else(|e| {
        unsafe {
            ura_log(1, &format!("compress_hook panic: {:?}", e));
        }
        std::ptr::null_mut()
    })
}

// ★ v3.23.3: Hook handler for DecompressResponse(byte[] data) -> byte[]
// Forwards the decompressed response body with the matching request's URL + headers.
extern "C" fn decompress_response_hook_handler(data: *mut c_void) -> *mut c_void {
    unsafe {
        let trampoline = interceptor_get_trampoline(decompress_response_hook_handler as usize);
        if trampoline == 0 {
            return std::ptr::null_mut();
        }
        type FnType = unsafe extern "C" fn(*mut c_void) -> *mut c_void;
        let original: FnType = std::mem::transmute(trampoline);
        let decompressed = original(data);
        let bytes = read_il2cpp_byte_array(decompressed);
        if !bytes.is_empty() {
            if let Ok(mut pending) = EVENT_PENDING_RESULT.lock() {
                if let Some(sel) = pending.take() {
                    let preview_len = bytes.len().min(EVENT_RESPONSE_PREVIEW_MAX);
                    let preview = String::from_utf8_lossy(&bytes[..preview_len]);
                    let (label, gain_id, next_block_idx, loop_exit_gain_id) = match sel.choice {
                        Some(c) => (c.label, c.gain_id, c.next_block_idx, c.loop_exit_gain_id),
                        None => (String::new(), -1, -1, -1),
                    };
                    let observation_id = EVENT_OBSERVATION_ID.fetch_add(1, Ordering::Relaxed);
                    let record = format!(
                        r#"{{"schema_version":2,"observation_id":{},"source":"runtime_observation","causality":"unknown","result_label":"unknown","captured_at":{},"generation":{},"story_id":{},"chara_id":{},"selected_idx_raw":{},"choice":{{"label":"{}","gain_id":{},"next_block_idx":{},"loop_exit_gain_id":{}}},"response":{{"request_id":{},"url":"{}","size_captured":{},"preview_truncated":{},"hex_prefix":"{}","text_preview":"{}"}}}}"#,
                        observation_id,
                        sel.captured_at,
                        sel.generation,
                        sel.story_id,
                        sel.chara_id,
                        sel.selected_idx_raw,
                        json_escape(&label),
                        gain_id,
                        next_block_idx,
                        loop_exit_gain_id,
                        PENDING_REQ_ID,
                        json_escape(&PENDING_URL),
                        bytes.len(),
                        bytes.len() > preview_len,
                        hex_encode(&bytes[..bytes.len().min(64)]),
                        json_escape(&preview)
                    );
                    if let Ok(mut obs) = EVENT_OBSERVATIONS.lock() {
                        if obs.len() >= EVENT_OBSERVATIONS_MAX {
                            obs.remove(0);
                        }
                        obs.push(record);
                    }
                }
            }
        }
        if SNIFF_ENABLED.load(Ordering::Relaxed) {
            if !bytes.is_empty() {
                let _lock = SNIFF_MUTEX.lock();
                let (rid, response_url) = if SNIFF_RESPONSE_QUEUE.is_empty() {
                    (0, String::new())
                } else {
                    SNIFF_RESPONSE_QUEUE.remove(0)
                };
                push_sniff_metadata(rid, "response", &response_url, bytes.len(), &bytes, Vec::new());
                SNIFF_RESPONSES.push((rid, bytes));
                if SNIFF_RESPONSES.len() > SNIFF_RAW_MAX {
                    SNIFF_RESPONSES.remove(0);
                }
            }
        }
        decompressed
    }
}

// ★ v3.23.3: Hook handler for WWWRequest.Post(this, url, postData, headers)
// Captures URL + headers directly, and matches the parked request body from CompressRequest.
// This replaces the old _Send + SetHeader approach.
extern "C" fn post_hook_handler(
    this: *mut c_void,
    url: *const c_void,
    post_data: *mut c_void,
    headers: *mut c_void,
) -> *mut c_void {
    unsafe {
        let trampoline = interceptor_get_trampoline(post_hook_handler as usize);
        if trampoline == 0 {
            return std::ptr::null_mut();
        }
        type FnType = unsafe extern "C" fn(
            *mut c_void,
            *const c_void,
            *mut c_void,
            *mut c_void,
        ) -> *mut c_void;
        let original: FnType = std::mem::transmute(trampoline);

        // Capture URL
        let game_url = if !url.is_null() {
            read_il2cpp_string(url)
        } else {
            String::new()
        };
        let game_url = if game_url.is_empty() {
            None
        } else {
            Some(game_url)
        };

        // Capture headers from Dictionary<string,string>
        let req_headers = read_string_dict(headers);

        if SNIFF_ENABLED.load(Ordering::Relaxed) {
            let rid = SNIFF_REQ_ID.fetch_add(1, Ordering::Relaxed);
            PENDING_REQ_ID = rid;
            let body = PENDING_REQ_BODY.take().unwrap_or_default();
            let headers_json = format_headers_json(&req_headers);
            let url_str = game_url.clone().unwrap_or_default();
            {
                let _lock = SNIFF_MUTEX.lock();
                push_sniff_metadata(rid, 
```

### match 2

```rust
tch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
        let input_str = if !input.is_null() {
            read_il2cpp_string(input)
        } else {
            String::new()
        };

        let trampoline = interceptor_get_trampoline(computehash_hook_handler as usize);
        if trampoline == 0 {
            return std::ptr::null_mut();
        }
        type FnType = unsafe extern "C" fn(*mut c_void) -> *mut c_void;
        let original: FnType = std::mem::transmute(trampoline);
        let ret = original(input);

        let output_str = if !ret.is_null() {
            read_il2cpp_string(ret)
        } else {
            String::new()
        };

        // Log with "CH:" prefix to distinguish from MakeMd5 entries
        if !input_str.is_empty() {
            if let Ok(mut log) = MD5_LOG.lock() {
                if log.len() >= 100 {
                    log.remove(0);
                }
                log.push((format!("CH:{}", input_str), output_str));
            }
        }

        ret
    }));
    result.unwrap_or(std::ptr::null_mut())
}

// ★ v3.23.3: Hook handler for CompressRequest(byte[] data) -> byte[]
// Parks the uncompressed request body, keyed by the compressed byte array returned by the original.
// WWWRequest.Post will match it later.
extern "C" fn compress_request_hook_handler(data: *mut c_void) -> *mut c_void {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
        let body = read_il2cpp_byte_array(data);
        let trampoline = interceptor_get_trampoline(compress_request_hook_handler as usize);
        if trampoline == 0 {
            return std::ptr::null_mut();
        }
        type FnType = unsafe extern "C" fn(*mut c_void) -> *mut c_void;
        let original: FnType = std::mem::transmute(trampoline);
        let compressed = original(data);
        if !body.is_empty() && POST_ADDR != 0 {
            PENDING_REQ_BODY = Some(body);
            PENDING_COMPRESSED = compressed as usize;
        }
        compressed
    }));
    result.unwrap_or_else(|e| {
        unsafe {
            ura_log(1, &format!("compress_hook panic: {:?}", e));
        }
        std::ptr::null_mut()
    })
}

// ★ v3.23.3: Hook handler for DecompressResponse(byte[] data) -> byte[]
// Forwards the decompressed response body with the matching request's URL + headers.
extern "C" fn decompress_response_hook_handler(data: *mut c_void) -> *mut c_void {
    unsafe {
        let trampoline = interceptor_get_trampoline(decompress_response_hook_handler as usize);
        if trampoline == 0 {
            return std::ptr::null_mut();
        }
        type FnType = unsafe extern "C" fn(*mut c_void) -> *mut c_void;
        let original: FnType = std::mem::transmute(trampoline);
        let decompressed = original(data);
        let bytes = read_il2cpp_byte_array(decompressed);
        if !bytes.is_empty() {
            if let Ok(mut pending) = EVENT_PENDING_RESULT.lock() {
                if let Some(sel) = pending.take() {
                    let preview_len = bytes.len().min(EVENT_RESPONSE_PREVIEW_MAX);
                    let preview = String::from_utf8_lossy(&bytes[..preview_len]);
                    let (label, gain_id, next_block_idx, loop_exit_gain_id) = match sel.choice {
                        Some(c) => (c.label, c.gain_id, c.next_block_idx, c.loop_exit_gain_id),
                        None => (String::new(), -1, -1, -1),
                    };
                    let observation_id = EVENT_OBSERVATION_ID.fetch_add(1, Ordering::Relaxed);
                    let record = format!(
                        r#"{{"schema_version":2,"observation_id":{},"source":"runtime_observation","causality":"unknown","result_label":"unknown","captured_at":{},"generation":{},"story_id":{},"chara_id":{},"selected_idx_raw":{},"choice":{{"label":"{}","gain_id":{},"next_block_idx":{},"loop_exit_gain_id":{}}},"response":{{"request_id":{},"url":"{}","size_captured":{},"preview_truncated":{},"hex_prefix":"{}","text_preview":"{}"}}}}"#,
                        observation_id,
                        sel.captured_at,
                        sel.generation,
                        sel.story_id,
                        sel.chara_id,
                        sel.selected_idx_raw,
                        json_escape(&label),
                        gain_id,
                        next_block_idx,
                        loop_exit_gain_id,
                        PENDING_REQ_ID,
                        json_escape(&PENDING_URL),
                        bytes.len(),
                        bytes.len() > preview_len,
                        hex_encode(&bytes[..bytes.len().min(64)]),
                        json_escape(&preview)
                    );
                    if let Ok(mut obs) = EVENT_OBSERVATIONS.lock() {
                        if obs.len() >= EVENT_OBSERVATIONS_MAX {
                            obs.remove(0);
                        }
                        obs.push(record);
                    }
                }
            }
        }
        if SNIFF_ENABLED.load(Ordering::Relaxed) {
            if !bytes.is_empty() {
                let _lock = SNIFF_MUTEX.lock();
                let (rid, response_url) = if SNIFF_RESPONSE_QUEUE.is_empty() {
                    (0, String::new())
                } else {
                    SNIFF_RESPONSE_QUEUE.remove(0)
                };
                push_sniff_metadata(rid, "response", &response_url, bytes.len(), &bytes, Vec::new());
                SNIFF_RESPONSES.push((rid, bytes));
                if SNIFF_RESPONSES.len() > SNIFF_RAW_MAX {
                    SNIFF_RESPONSES.remove(0);
                }
            }
        }
        decompressed
    }
}

// ★ v3.23.3: Hook handler for WWWRequest.Post(this, url, postData, headers)
// Captures URL + headers directly, and matches the parked request body from CompressRequest.
// This replaces the old _Send + SetHeader approach.
extern "C" fn post_hook_handler(
    this: *mut c_void,
    url: *const c_void,
    post_data: *mut c_void,
    headers: *mut c_void,
) -> *mut c_void {
    unsafe {
        let trampoline = interceptor_get_trampoline(post_hook_handler as usize);
        if trampoline == 0 {
            return std::ptr::null_mut();
        }
        type FnType = unsafe extern "C" fn(
            *mut c_void,
            *const c_void,
            *mut c_void,
            *mut c_void,
        ) -> *mut c_void;
        let original: FnType = std::mem::transmute(trampoline);

        // Capture URL
        let game_url = if !url.is_null() {
            read_il2cpp_string(url)
        } else {
            String::new()
        };
        let game_url = if game_url.is_empty() {
            None
        } else {
            Some(game_url)
        };

        // Capture headers from Dictionary<string,string>
        let req_headers = read_string_dict(headers);

        if SNIFF_ENABLED.load(Ordering::Relaxed) {
            let rid = SNIFF_REQ_ID.fetch_add(1, Ordering::Relaxed);
            PENDING_REQ_ID = rid;
            let body = PENDING_REQ_BODY.take().unwrap_or_default();
            let headers_json = format_headers_json(&req_headers);
            let url_str = game_url.clone().unwrap_or_default();
            {
                let _lock = SNIFF_MUTEX.lock();
                push_sniff_metadata(rid, "request", &url_str, body.len(), &body, req_headers.clone());
                SNIFF_RESPONSE_QUEUE.push((rid, url_str.clone()));
     
```

### match 3

```rust
                    COMPUTEHASH_ADDR = ch_addr;
                        set_hook_status("sniff.computehash", &format!("hooked@0x{:x}", ch_addr));
                        ura_log(3, &format!("API sniff: Cryptographer.ComputeHash hooked at 0x{:x}", ch_addr));
                    }
                }
            }
        }
    }

    let umamusume = get_asm(to_cstr("umamusume.dll").as_ptr());
    if umamusume.is_null() {
        ura_log(3, "API sniff: umamusume.dll image not found");
        set_hook_status("sniff", "failed: image_not_found");
        return;
    }

    // HttpHelper class (exact, then fuzzy fallback — v3.24.40)
    let mut http_helper = get_class(
        umamusume,
        to_cstr("Gallop").as_ptr(),
        to_cstr("HttpHelper").as_ptr(),
    );
    if http_helper.is_null() {
        http_helper = find_class_fuzzy(umamusume, "HttpHelper");
    }
    if http_helper.is_null() {
        ura_log(3, "API sniff: HttpHelper class not found");
        set_hook_status("sniff", "failed: httphelper_class_not_found");
        return;
    }
    ura_log(3, "API sniff: HttpHelper class found");

    // Hook CompressRequest
    if COMPRESS_REQUEST_ADDR == 0 {
        let mut addr =
            get_method_addr(http_helper as usize, to_cstr("CompressRequest").as_ptr(), 1);
        if addr == 0 {
            addr = find_method_fuzzy(http_helper, "CompressRequest");
        }
        if addr != 0 {
            if interceptor_hook(addr, compress_request_hook_handler as usize) {
                COMPRESS_REQUEST_ADDR = addr;
                ura_log(
                    3,
                    &format!("API sniff: CompressRequest hooked at 0x{:x}", addr),
                );
                set_hook_status("sniff.compress", &format!("hooked@0x{:x}", addr));
            } else {
                ura_log(
                    3,
                    &format!("API sniff: CompressRequest hook FAILED at 0x{:x}", addr),
                );
                set_hook_status("sniff.compress", "failed: interceptor_hook");
            }
        } else {
            ura_log(3, "API sniff: CompressRequest NOT FOUND");
            set_hook_status("sniff.compress", "failed: method_not_found");
        }
    }

    // Hook DecompressResponse
    if DECOMPRESS_RESPONSE_ADDR == 0 {
        let addr = get_method_addr(
            http_helper as usize,
            to_cstr("DecompressResponse").as_ptr(),
            1,
        );
        if addr != 0 {
            if interceptor_hook(addr, decompress_response_hook_handler as usize) {
                DECOMPRESS_RESPONSE_ADDR = addr;
                ura_log(
                    3,
                    &format!("API sniff: DecompressResponse hooked at 0x{:x}", addr),
                );
                set_hook_status("sniff.decompress", &format!("hooked@0x{:x}", addr));
            } else {
                ura_log(
                    3,
                    &format!("API sniff: DecompressResponse hook FAILED at 0x{:x}", addr),
                );
                set_hook_status("sniff.decompress", "failed: interceptor_hook");
            }
        } else {
            ura_log(3, "API sniff: DecompressResponse NOT FOUND");
            set_hook_status("sniff.decompress", "failed: method_not_found");
        }
    }

    // Hook WWWRequest.Post (from Cute.Http.Assembly.dll)
    if POST_ADDR == 0 {
        let cute_http = get_asm(to_cstr("Cute.Http.Assembly.dll").as_ptr());
        if !cute_http.is_null() {
            let mut www_request = get_class(
                cute_http,
                to_cstr("Cute.Http").as_ptr(),
                to_cstr("WWWRequest").as_ptr(),
            );
            if www_request.is_null() {
                www_request = find_class_fuzzy(cute_http, "WWWRequest");
            }
            if !www_request.is_null() {
                let mut addr = get_method_addr(www_request as usize, to_cstr("Post").as_ptr(), 3);
                if addr == 0 {
                    addr = find_method_fuzzy(www_request, "Post");
                }
                if addr != 0 {
                    if interceptor_hook(addr, post_hook_handler as usize) {
                        POST_ADDR = addr;
                        ura_log(
                            3,
                            &format!("API sniff: WWWRequest.Post hooked at 0x{:x}", addr),
                        );
                        set_hook_status("sniff.post", &format!("hooked@0x{:x}", addr));
                    } else {
                        ura_log(
                            3,
                            &format!("API sniff: WWWRequest.Post hook FAILED at 0x{:x}", addr),
                        );
                        set_hook_status("sniff.post", "failed: interceptor_hook");
                    }
                } else {
                    ura_log(3, "API sniff: WWWRequest.Post NOT FOUND");
                    set_hook_status("sniff.post", "failed: method_not_found");
                }
            } else {
                ura_log(3, "API sniff: Cute.Http.WWWRequest class not found");
                set_hook_status("sniff.post", "failed: class_not_found");
            }
        } else {
            ura_log(3, "API sniff: Cute.Http.Assembly.dll image not found");
        }
    }
}

// ★ v3.24.2: Story event choice hook — capture career event choices
// StoryChoiceController.Choice(int choiceIndex, ???)
// ARM64: X0=this, W1=choiceIndex, X2=???
extern "C" fn event_choice_hook_handler(
    this: *mut c_void,
    choice_index: i32,
    _param2: *mut c_void,
) {
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
        let choices_count = {
            let _lock = EVENT_STATE_MUTEX.lock();
            EVENT_SELECTED_IDX = choice_index;
            let choice = if choice_index >= 0 {
                EVENT_CHOICES.get(choice_index as usize).cloned()
            } else {
                None
            };
            if let Ok(mut pending) = EVENT_PENDING_RESULT.lock() {
                *pending = Some(PendingEventSelection {
                    captured_at: sniff_timestamp(),
                    generation: EVENT_GENERATION,
                    story_id: EVENT_STORY_ID,
                    chara_id: EVENT_CHARA_ID,
                    selected_idx_raw: choice_index,
                    choice,
                });
            }
            EVENT_CHOICES.len()
        };

        ura_log(
            3,
            &format!(
                "Event choice: index={} choices_count={}",
                choice_index, choices_count
            ),
        );

        let trampoline = interceptor_get_trampoline(event_choice_hook_handler as usize);
        if trampoline == 0 {
            ura_log(1, "event_choice_hook: trampoline not found");
            return;
        }
        type FnChoice = unsafe extern "C" fn(*mut c_void, i32, *mut c_void);
        let original: FnChoice = std::mem::transmute(trampoline);
        original(this, choice_index, _param2);
    }));
}

// ★ v3.24.41: runtime class of an object via il2cpp_object_get_class
unsafe fn obj_class(obj: *const c_void) -> *mut c_void {
    if obj.is_null() {
        return ptr::null_mut();
    }
    let f: Option<unsafe extern "C" fn(*const c_void) -> *mut c_void> = {
        let p = resolve_il2cpp_symbol("il2cpp_object_get_class");
        if p.is_null() {
            None
        } else {
            Some(std::mem::transmute(p))
        }
    };
    match f {
        Some(g) => g(obj),
    
```

## `install_api_sniff_hooks` (4 matches)

### match 1

```rust
-juece floating window app receives and displays the data
// ============================================================

fn simple_hash(s: &str) -> u64 {
    let mut h: u64 = 5381;
    for b in s.bytes() {
        h = h.wrapping_mul(33).wrapping_add(b as u64);
    }
    h
}

fn push_to_app(json: &str) {
    use std::io::{Read, Write};
    let cfg = unsafe { get_config() };
    if !cfg.push_enabled {
        return;
    }
    let addr_str = cfg.push_addr();
    let addr: std::net::SocketAddr = match addr_str.parse() {
        Ok(a) => a,
        Err(_) => return,
    };
    let mut stream =
        match std::net::TcpStream::connect_timeout(&addr, std::time::Duration::from_secs(2)) {
            Ok(s) => s,
            Err(_) => return, // App not running, that's fine
        };
    let body = json.as_bytes();
    let req = format!(
        "POST /data HTTP/1.1\r\nHost: {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        addr_str, body.len()
    );
    let _ = stream.write_all(req.as_bytes());
    let _ = stream.write_all(body);
    let _ = stream.flush();
    let mut buf = [0u8; 256];
    let _ = stream.read(&mut buf);
}

fn push_loop() {
    let interval =
        std::time::Duration::from_secs(unsafe { get_config() }.push_interval_secs.max(2));
    let mut consecutive_errors: u32 = 0;

    // ★ Initial push: try pushing current data on startup
    // Don't rely solely on GAME_INITIALIZED callback — it may never fire
    // if the game was already initialized before the plugin loaded.
    // Instead, try reading data; if it succeeds, the game is ready.
    for wait_round in 0..60 {
        if GAME_INITIALIZED.load(Ordering::Relaxed) {
            break;
        }
        boot_trace("push_probe_begin");
        // Try a probe read — if it doesn't error, game is ready
        let probe = read_summary();
        if !probe.contains("\"error\"") {
            GAME_INITIALIZED.store(true, Ordering::Relaxed);
            unsafe {
                ura_log(3, "Push: game detected via probe (no callback)");
                // v3.22.98: Install hooks in fallback (on_game_initialized may never fire)
                install_training_hook();
                install_exec_training_hook();
                install_failure_rate_hook();
                install_event_choice_hook();
                // ★ v3.24.40: sniff hooks were missing here — fallback mode
                // left /api/sniff permanently unhooked.
                install_api_sniff_hooks();
            }
            break;
        }
        if wait_round % 10 == 0 {
            unsafe {
                ura_log(
                    3,
                    &format!("Push: waiting for game... round={}", wait_round),
                );
            }
        }
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
    let init_summary = read_summary();
    if !init_summary.contains("\"error\"") {
        unsafe {
            LAST_PUSH_HASH = simple_hash(&init_summary);
        }
        push_to_app(&init_summary);
        unsafe {
            ura_log(3, "Push: initial data pushed");
        }
    }

    loop {
        std::thread::sleep(interval);
        // Don't gate on GAME_INITIALIZED — just try reading;
        // if the game isn't ready, read_summary returns error and we skip.
        let summary = read_summary();
        if summary.contains("\"error\"") {
            consecutive_errors += 1;
            // ★ v3.22.89: Extra cooldown for SIGSEGV recovery — game state transition
            if summary.contains("sigsegv") {
                let cool = std::time::Duration::from_secs(60);
                unsafe {
                    ura_log(
                        2,
                        "Push: SIGSEGV recovered, cooling 60s for game state transition",
                    );
                }
                std::thread::sleep(cool);
            }
            // ★ v3.14.2: backoff on consecutive errors to avoid crash loop
            if consecutive_errors >= 1 {
                let backoff =
                    std::time::Duration::from_secs((consecutive_errors as u64 * 5).min(60));
                unsafe {
                    ura_log(
                        3,
                        &format!(
                            "Push: {} consecutive errors, backing off {}s",
                            consecutive_errors,
                            backoff.as_secs()
                        ),
                    );
                }
                std::thread::sleep(backoff);
            }
            continue;
        }
        consecutive_errors = 0;
        // If we got here, game is definitely ready
        if !GAME_INITIALIZED.load(Ordering::Relaxed) {
            GAME_INITIALIZED.store(true, Ordering::Relaxed);
        }
        let hash = simple_hash(&summary);
        let should_push = unsafe {
            if hash != LAST_PUSH_HASH {
                LAST_PUSH_HASH = hash;
                true
            } else {
                false
            }
        };
        if should_push {
            unsafe {
                ura_log(3, "Push: data changed, pushing to app");
            }
            push_to_app(&summary);
        }
    }
}

fn start_http_server() {
    if HTTP_RUNNING.load(Ordering::Relaxed) {
        return;
    }
    HTTP_RUNNING.store(true, Ordering::Relaxed);
    std::thread::spawn(|| {
        unsafe {
            // ★ v3.24.32: bind loopback only. The floating-window App talks to
            // the plugin on the same device, and desktop/LAN debugging works
            // via `adb forward tcp:18765 tcp:18765`. Binding 0.0.0.0 exposed
            // /il2cpp/call, /il2cpp/read_mem, /update etc. to the whole LAN
            // without authentication.
            ura_log(3, "HTTP starting on 127.0.0.1:18765");
        }
        let listener = match std::net::TcpListener::bind("127.0.0.1:18765") {
            Ok(l) => l,
            Err(e) => {
                unsafe {
                    ura_log(1, &format!("HTTP bind failed: {}", e));
                }
                HTTP_RUNNING.store(false, Ordering::Relaxed);
                return;
            }
        };
        unsafe {
            ura_log(3, "HTTP listening on :18765");
        }
        unsafe {
            ura_notify("URA HTTP :18765 ON");
        }

        // ★ Start push-to-app loop (v3.10.0)
        std::thread::spawn(|| {
            push_loop();
        });

        for stream in listener.incoming() {
            if !HTTP_RUNNING.load(Ordering::Relaxed) {
                break;
            }
            match stream {
                Ok(stream) => {
                    // ★ v3.18.8: spawn thread per request — prevents slow endpoint from blocking others
                    let _ = stream.set_read_timeout(Some(std::time::Duration::from_secs(5)));
                    let _ = stream.set_write_timeout(Some(std::time::Duration::from_secs(10)));
                    std::thread::spawn(move || handle_http(stream));
                }
                Err(_) => continue,
            }
        }
        HTTP_RUNNING.store(false, Ordering::Relaxed);
    });
}

fn parse_path(req: &str) -> String {
    let first_line = req.lines().next().unwrap_or("");
    let uri = first_line.split(' ').nth(1).unwrap_or("/");
    let path = uri.split('?').next().unwrap_or(uri);
    if path.starts_with("http://") || path.starts_with("https://") {
        if let Some(after_host) = path.splitn(4, '/').n
```

### match 2

```rust
lse if path == "/api/sniff/status" {
        let _lock = SNIFF_MUTEX.lock();
        unsafe {
            let last_id = SNIFF_METADATA.last().map(|m| m.id).unwrap_or(0);
            let request_count = SNIFF_METADATA
                .iter()
                .filter(|m| m.direction == "request")
                .count();
            let response_count = SNIFF_METADATA
                .iter()
                .filter(|m| m.direction == "response")
                .count();
            format!(
                r#"{{"enabled":{},"raw_request_count":{},"raw_response_count":{},"metadata_count":{},"request_count":{},"response_count":{},"last_id":{},"raw_limit":{},"metadata_limit":{}}}"#,
                SNIFF_ENABLED.load(Ordering::Relaxed),
                SNIFF_REQUESTS.len(),
                SNIFF_RESPONSES.len(),
                SNIFF_METADATA.len(),
                request_count,
                response_count,
                last_id,
                SNIFF_RAW_MAX,
                SNIFF_METADATA_MAX
            )
        }
    } else if path == "/api/sniff/metadata" {
        let after_id = parse_query(&full_uri, "after_id")
            .parse::<u64>()
            .unwrap_or(0);
        let _lock = SNIFF_MUTEX.lock();
        unsafe {
            let entries: Vec<String> = SNIFF_METADATA.iter()
                .filter(|m| m.id > after_id)
                .map(|m| {
                    let headers_json: String = m.headers.iter()
                        .map(|(k, v)| format!(r#"{{"key":"{}","value":"{}"}}"#, json_escape(k), json_escape(v)))
                        .collect::<Vec<String>>()
                        .join(",");
                    format!(r#"{{"id":{},"request_id":{},"timestamp_ms":{},"direction":"{}","path":"{}","size":{},"body_hex":"{}","headers":[{}]}}"#,
                        m.id, m.request_id, m.timestamp_ms, m.direction, json_escape(&m.path), m.size, m.body_hex, headers_json)
                })
                .collect();
            let last_id = SNIFF_METADATA.last().map(|m| m.id).unwrap_or(after_id);
            format!(
                r#"{{"enabled":{},"after_id":{},"last_id":{},"count":{},"entries":[{}]}}"#,
                SNIFF_ENABLED.load(Ordering::Relaxed),
                after_id,
                last_id,
                entries.len(),
                entries.join(",")
            )
        }
    } else if path == "/api/sniff/toggle" {
        // ★ v3.24.40: lazy retry for fallback-mode installs.
        unsafe {
            install_api_sniff_hooks();
        }
        // ★ If hooks installed successfully, game is ready — set GAME_INITIALIZED
        let any_hooked = unsafe {
            COMPRESS_REQUEST_ADDR != 0
                || DECOMPRESS_RESPONSE_ADDR != 0
                || POST_ADDR != 0
                || MAKEMD5_ADDR != 0
                || COMPUTEHASH_ADDR != 0
        };
        if any_hooked && !GAME_INITIALIZED.load(Ordering::Relaxed) {
            GAME_INITIALIZED.store(true, Ordering::Relaxed);
            unsafe {
                ura_log(3, "sniff/toggle: GAME_INITIALIZED set (hooks installed via toggle)");
            }
        }
        let requested = parse_query(&full_uri, "enabled");
        let new_val = match requested.as_str() {
            "1" | "true" => true,
            "0" | "false" => false,
            _ => !SNIFF_ENABLED.load(Ordering::Relaxed),
        };
        SNIFF_ENABLED.store(new_val, Ordering::Relaxed);
        let req_hooked = unsafe { COMPRESS_REQUEST_ADDR != 0 };
        let resp_hooked = unsafe { DECOMPRESS_RESPONSE_ADDR != 0 };
        let post_hooked = unsafe { POST_ADDR != 0 };
        format!(
            r#"{{"sniff_enabled":{},"compress_hooked":{},"decompress_hooked":{},"post_hooked":{}}}"#,
            new_val, req_hooked, resp_hooked, post_hooked
        )
    } else if path == "/api/sniff/clear" {
        let _lock = SNIFF_MUTEX.lock();
        unsafe {
            SNIFF_REQUESTS.clear();
            SNIFF_RESPONSES.clear();
            if let Ok(mut entries) = UNITY_OBSERVATIONS.lock() {
                entries.clear();
            }
            SNIFF_METADATA.clear();
            SNIFF_RESPONSE_QUEUE.clear();
            PENDING_REQ_BODY = None;
        }
        r#"{"ok":true}"#.to_string()
    } else if path.starts_with("/debug/hooklog") {
        // ★ v3.24.40/42: last HOOK_LOG_MAX lines, optional ?filter=substr
        let filter = parse_query(&full_uri, "filter");
        let entries: Vec<String> = match HOOK_LOG.lock() {
            Ok(g) => g
                .iter()
                .filter(|l| filter.is_empty() || l.contains(&filter))
                .map(|l| json_escape(l))
                .collect(),
            Err(_) => Vec::new(),
        };
        format!(
            r#"{{"count":{},"entries":[{}]}}"#,
            entries.len(),
            entries.join(",")
        )
    } else if path == "/debug/resource_reads" {
        // ★ v3.24.58: meta/dat file-read trace. Lazy-starts the /proc watcher
        // on first request (never at init — thread spawn in init context).
        start_res_fd_watcher();
        let entries: Vec<String> = match RES_READ_LOG.lock() {
            Ok(g) => g
                .iter()
                .map(|l| format!("\"{}\"", json_escape(l)))
                .collect(),
            Err(_) => Vec::new(),
        };
        format!(
            r#"{{"count":{},"entries":[{}]}}"#,
            entries.len(),
            entries.join(",")
        )
    } else if path.starts_with("/debug/mem_scan_sqlite") {
        // ★ v3.24.58: hunt plaintext "SQLite format 3" pages in process memory
        // — any custom decryption MUST materialize this in RAM.
        let max_hits: usize = parse_query(&full_uri, "max").parse().unwrap_or(8);
        let mut hits: Vec<String> = Vec::new();
        let needle = b"SQLite format 3 ";
        if let Ok(maps) = std::fs::read_to_string("/proc/self/maps") {
            let mem = std::fs::File::open("/proc/self/mem");
            use std::os::unix::fs::FileExt;
            if let Ok(mem) = mem {
                'outer: for line in maps.lines() {
                    let cols: Vec<&str> = line.split_whitespace().collect();
                    if cols.len() < 6 {
                        continue;
                    }
                    if !cols[1].contains("rw") {
                        continue;
                    }
                    let range: Vec<&str> = cols[0].split('-').collect();
                    if range.len() != 2 {
                        continue;
                    }
                    let (Ok(sa), Ok(ea)) = (
                        usize::from_str_radix(range[0], 16),
                        usize::from_str_radix(range[1], 16),
                    ) else {
                        continue;
                    };
                    let len = ea - sa;
                    if len < 4096 || len > 512 * 1024 * 1024 {
                        continue;
                    }
                    let mut off = 0usize;
                    while off < len {
                        let chunk = (4 * 1024 * 1024usize).min(len - off);
                        let mut buf = vec![0u8; chunk];
                        if mem.read_at(&mut buf, (sa + off) as u64).is_err() {
                            break;
                        }
                        for (i, w) in buf.windows(needle.len()).enumerate() {
                            if w == needle {
                                let abs = sa + off + i;
  
```

### match 3

```rust
_get_ptr_fn: Option<unsafe extern "C" fn(*const c_void) -> *const c_void> = {
        let p = resolve_il2cpp_symbol("il2cpp_method_get_pointer");
        if p.is_null() {
            None
        } else {
            Some(std::mem::transmute(p))
        }
    };
    if get_methods_fn.is_none() || method_get_name_fn.is_none() {
        return 0;
    }
    let mut iter: *mut c_void = std::ptr::null_mut();
    loop {
        let mi = get_methods_fn.unwrap()(class, &mut iter);
        if mi.is_null() {
            break;
        }
        let name_ptr = method_get_name_fn.unwrap()(mi);
        if name_ptr.is_null() {
            continue;
        }
        let name = CStr::from_ptr(name_ptr).to_string_lossy();
        if name.contains(substr) {
            if let Some(get_ptr) = method_get_ptr_fn {
                let ptr = get_ptr(mi);
                if !ptr.is_null() {
                    ura_log(
                        3,
                        &format!(
                            "find_method_fuzzy: {}~{} -> 0x{:x}",
                            substr, name, ptr as usize
                        ),
                    );
                    return ptr as usize;
                }
            }
        }
    }
    0
}

/// ★ v3.24.40: fuzzy variant — first class whose name CONTAINS `substr`.
unsafe fn find_class_fuzzy(image: *const c_void, substr: &str) -> *mut c_void {
    let get_count_fn = resolve_il2cpp_symbol("il2cpp_image_get_class_count");
    let get_class_fn = resolve_il2cpp_symbol("il2cpp_image_get_class");
    let get_name_fn = resolve_il2cpp_symbol("il2cpp_class_get_name");
    if get_count_fn.is_null() || get_class_fn.is_null() || get_name_fn.is_null() {
        return ptr::null_mut();
    }
    let get_count: FnImageGetClassCount = std::mem::transmute(get_count_fn);
    let get_class: FnImageGetClass = std::mem::transmute(get_class_fn);
    let get_name: unsafe extern "C" fn(*const c_void) -> *const c_char =
        std::mem::transmute(get_name_fn);
    let count = get_count(image);
    for i in 0..count {
        let cls = get_class(image, i);
        if cls.is_null() {
            continue;
        }
        let np = get_name(cls);
        if np.is_null() {
            continue;
        }
        let name = CStr::from_ptr(np).to_string_lossy();
        if name.contains(substr) {
            ura_log(3, &format!("find_class_fuzzy: {}~{}", substr, name));
            return cls as *mut c_void;
        }
    }
    ptr::null_mut()
}

unsafe fn install_api_sniff_hooks() {
    let all_hooked = COMPRESS_REQUEST_ADDR != 0
        && DECOMPRESS_RESPONSE_ADDR != 0
        && POST_ADDR != 0
        && UNITY_SEND_ADDR != 0;
    if all_hooked {
        return;
    }
    if API.is_null() {
        ura_log(3, "API sniff: API is null");
        set_hook_status("sniff", "failed: api_null");
        return;
    }
    let api = &*API;
    if api.interceptor == 0 {
        ura_log(3, "API sniff: interceptor not available");
        set_hook_status("sniff", "failed: interceptor_unavailable");
        return;
    }

    // Get umamusume.dll assembly image
    let get_asm = match api.il2cpp_get_assembly_image_fn {
        Some(f) => f,
        None => {
            ura_log(3, "API sniff: get_assembly_image not available");
            return;
        }
    };
    let get_class = match api.il2cpp_get_class_fn {
        Some(f) => f,
        None => {
            ura_log(3, "API sniff: get_class not available");
            return;
        }
    };
    let get_method_addr = match api.il2cpp_get_method_addr_fn {
        Some(f) => f,
        None => {
            ura_log(3, "API sniff: get_method_addr not available");
            return;
        }
    };

    // Observe the lower UnityWebRequest request-entry path used by boot/auth traffic.
    if UNITY_SEND_ADDR == 0 {
        let unity_image = get_asm(to_cstr("UnityEngine.UnityWebRequestModule.dll").as_ptr());
        if unity_image.is_null() {
            set_hook_status("sniff.unity_send", "failed: image_not_found");
        } else {
            let unity_request = get_class(
                unity_image,
                to_cstr("UnityEngine.Networking").as_ptr(),
                to_cstr("UnityWebRequest").as_ptr(),
            );
            if unity_request.is_null() {
                set_hook_status("sniff.unity_send", "failed: class_not_found");
            } else {
                let addr = get_method_addr(
                    unity_request as usize,
                    to_cstr("SendWebRequest").as_ptr(),
                    0,
                );
                if addr == 0 {
                    set_hook_status("sniff.unity_send", "failed: method_not_found");
                } else if interceptor_hook(addr, unity_send_hook_handler as usize) {
                    UNITY_SEND_ADDR = addr;
                    set_hook_status("sniff.unity_send", &format!("hooked@0x{:x}", addr));
                    ura_log(
                        3,
                        &format!(
                            "API sniff: UnityWebRequest.SendWebRequest hooked at 0x{:x}",
                            addr
                        ),
                    );
                } else {
                    set_hook_status("sniff.unity_send", "failed: interceptor_hook");
                }
            }
        }
    }

    // Hook Cryptographer.MakeMd5 to capture salt
    if MAKEMD5_ADDR == 0 {
        let umamusume_img = get_asm(to_cstr("umamusume.dll").as_ptr());
        if !umamusume_img.is_null() {
            let crypto_class = get_class(
                umamusume_img,
                to_cstr("Gallop").as_ptr(),
                to_cstr("Cryptographer").as_ptr(),
            );
            if !crypto_class.is_null() {
                let addr = get_method_addr(
                    crypto_class as usize,
                    to_cstr("MakeMd5").as_ptr(),
                    1,
                );
                if addr != 0 {
                    if interceptor_hook(addr, makemd5_hook_handler as usize) {
                        MAKEMD5_ADDR = addr;
                        set_hook_status("sniff.makemd5", &format!("hooked@0x{:x}", addr));
                        ura_log(3, &format!("API sniff: Cryptographer.MakeMd5 hooked at 0x{:x}", addr));
                    } else {
                        set_hook_status("sniff.makemd5", "failed: interceptor_hook");
                    }
                }
                // Also hook ComputeHash to capture intermediate data (salted input)
                let ch_addr = get_method_addr(
                    crypto_class as usize,
                    to_cstr("ComputeHash").as_ptr(),
                    1,
                );
                if ch_addr != 0 {
                    if interceptor_hook(ch_addr, computehash_hook_handler as usize) {
                        COMPUTEHASH_ADDR = ch_addr;
                        set_hook_status("sniff.computehash", &format!("hooked@0x{:x}", ch_addr));
                        ura_log(3, &format!("API sniff: Cryptographer.ComputeHash hooked at 0x{:x}", ch_addr));
                    }
                }
            }
        }
    }

    let umamusume = get_asm(to_cstr("umamusume.dll").as_ptr());
    if umamusume.is_null() {
        ura_log(3, "API sniff: umamusume.dll image not found");
        set_hook_status("sniff", "failed: image_not_found");
        return;
    }

    // HttpHelper class (exact, then fuzzy fallback — v3.24.40)
    let mut http_helper = get_class(
  
```

### match 4

```rust
   choice_addr,
            event_choice_hook_handler as usize,
            &mut ORIG_EVENT_CHOICE_PROLOGUE,
        );
        set_hook_status("event.choice", &format!("resolved@0x{:x}", choice_addr));
    } else {
        ura_log(3, "Event hook: Choice NOT FOUND");
        set_hook_status("event.choice", "failed: method_not_found");
    }

    // ★ v3.24.2: Hook StoryManager.SetStory to capture story_id and chara_id
    let mut story_mgr_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("StoryManager").as_ptr(),
    );
    if story_mgr_class.is_null() {
        story_mgr_class = find_class_fuzzy(image, "StoryManager");
    }
    if !story_mgr_class.is_null() {
        let set_story_addr = find_method_addr(story_mgr_class, "SetStory", 4);
        if set_story_addr != 0 {
            STORY_SET_ADDR = set_story_addr;
            STORY_SET_HOOK_INSTALLED = true;
            install_hook_safe(
                "StorySet",
                set_story_addr,
                story_set_hook_handler as usize,
                &mut ORIG_STORY_SET_PROLOGUE,
            );
            set_hook_status(
                "event.story_set",
                &format!("resolved@0x{:x}", set_story_addr),
            );
            ura_log(
                3,
                &format!(
                    "Event hook: StoryManager.SetStory hooked at 0x{:x}",
                    set_story_addr
                ),
            );
        } else {
            ura_log(3, "Event hook: StoryManager.SetStory NOT FOUND");
            set_hook_status("event.story_set", "failed: method_not_found");
        }
    } else {
        ura_log(3, "Event hook: StoryManager class NOT FOUND");
        set_hook_status("event.story_set", "failed: class_not_found");
    }

    // ★ v3.24.40: only mark installed when at least one hook landed, so the
    // lazy retry in /api/event/choices can re-attempt after early-boot misses.
    EVENT_CHOICE_HOOK_INSTALLED =
        EVENT_ADD_BTN_ADDR != 0 || EVENT_CHOICE_ADDR != 0 || STORY_SET_HOOK_INSTALLED;
}

extern "C" fn on_game_initialized(_userdata: *mut c_void) {
    GAME_INITIALIZED.store(true, Ordering::Relaxed);
    boot_trace("game_init_cb");
    unsafe {
        ura_log(3, "Game initialized");
        ura_notify("URA: Game ready!");
        // v3.22.98: Install hooks FIRST (before precache, which may panic)
        install_training_hook();
        install_exec_training_hook();
        install_failure_rate_hook();
        install_api_sniff_hooks();
        install_event_choice_hook();
        // v3.22.51: Pre-cache all IL2CPP metadata on game thread
        precache_metadata();
        boot_trace("game_init_done");
    }
}

extern "C" fn on_menu_section(ui: *mut c_void, _userdata: *mut c_void) {
    unsafe {
        if API.is_null() || ui.is_null() {
            return;
        }
        let api = &*API;

        if let Some(f) = api.gui_ui_heading_fn {
            f(
                ui,
                to_cstr(&format!("URA Assistant v{}", PLUGIN_VERSION)).as_ptr(),
            );
        }
        if let Some(f) = api.gui_ui_separator_fn {
            f(ui);
        }

        if let Some(f) = api.gui_ui_colored_label_fn {
            if GAME_INITIALIZED.load(Ordering::Relaxed) {
                f(ui, 0, 255, 136, 255, to_cstr("Game: Connected").as_ptr());
            } else {
                f(ui, 255, 200, 0, 255, to_cstr("Game: Waiting...").as_ptr());
            }
        }

        if let Some(f) = api.gui_ui_colored_label_fn {
            if HTTP_RUNNING.load(Ordering::Relaxed) {
                f(
                    ui,
                    0,
                    255,
                    136,
                    255,
                    to_cstr(&format!(
                        "HTTP: Running :{}",
                        unsafe { get_config() }.http_port
                    ))
                    .as_ptr(),
                );
            } else {
                f(ui, 255, 80, 80, 255, to_cstr("HTTP: Failed").as_ptr());
            }
        }

        if let Some(f) = api.gui_ui_label_fn {
            f(
                ui,
                to_cstr("Data: WDM->SingleMode->Chara (getters)").as_ptr(),
            );
        }

        let c = CHARA;
        if c.valid {
            if let Some(f) = api.gui_ui_separator_fn {
                f(ui);
            }

            if let Some(f) = api.gui_ui_colored_label_fn {
                f(
                    ui,
                    0,
                    200,
                    255,
                    255,
                    to_cstr(&format!(
                        "Month {} | Half {} | PS:{}",
                        c.month, c.half, c.playing_state
                    ))
                    .as_ptr(),
                );
            }

            if let Some(f) = api.gui_ui_colored_label_fn {
                f(
                    ui,
                    255,
                    100,
                    100,
                    255,
                    to_cstr(&format!("SPD: {}", c.speed)).as_ptr(),
                );
            }
            if let Some(f) = api.gui_ui_colored_label_fn {
                f(
                    ui,
                    100,
                    255,
                    100,
                    255,
                    to_cstr(&format!("STA: {}", c.stamina)).as_ptr(),
                );
            }
            if let Some(f) = api.gui_ui_colored_label_fn {
                f(
                    ui,
                    255,
                    200,
                    50,
                    255,
                    to_cstr(&format!("POW: {}", c.power)).as_ptr(),
                );
            }
            if let Some(f) = api.gui_ui_colored_label_fn {
                f(
                    ui,
                    255,
                    130,
                    50,
                    255,
                    to_cstr(&format!("GUT: {}", c.guts)).as_ptr(),
                );
            }
            if let Some(f) = api.gui_ui_colored_label_fn {
                f(
                    ui,
                    100,
                    180,
                    255,
                    255,
                    to_cstr(&format!("WIZ: {}", c.wiz)).as_ptr(),
                );
            }

            if let Some(f) = api.gui_ui_label_fn {
                f(
                    ui,
                    to_cstr(&format!("Vital: {}/{}", c.vital, c.max_vital)).as_ptr(),
                );
            }
            if let Some(f) = api.gui_ui_colored_label_fn {
                let mot_text = match c.motivation {
                    5 => "Motivation: Best!!!",
                    4 => "Motivation: Good",
                    3 => "Motivation: Normal",
                    2 => "Motivation: Bad",
                    1 => "Motivation: Worst",
                    _ => "Motivation: ???",
                };
                let color = match c.motivation {
                    5 => (0, 255, 136),
                    4 => (100, 255, 100),
                    3 => (255, 255, 100),
                    2 => (255, 150, 50),
                    1 => (255, 50, 50),
                    _ => (200, 200, 200),
                };
                f(
                    ui,
                    color.0,
                    color.1,
                    color.2,
                    255,
                    to_cstr(mot_text).as_ptr(),
                );
            
```

## `UNITY_SEND_ADDR` (5 matches)

### match 1

```rust
;
static CRASH_SIG: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(0);
static CRASH_STEP: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
static mut LAST_STEP_BUF: [u8; 128] = [0; 128];
static LAST_STEP_LEN: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
static AUTO_UPDATE_STATUS: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);
// ★ Training result/action state is shared by the game hook and HTTP/summary threads.
// Keep correlated fields under one mutex to avoid data races and torn records.
struct ActionState {
    training_result: i32,
    training_sub_id: i32,
    command_id: i32,
    sequence: u64,
}
static ACTION_STATE: Mutex<ActionState> = Mutex::new(ActionState {
    training_result: -1,
    training_sub_id: -1,
    command_id: -1,
    sequence: 0,
});
static mut TRAINING_HOOK_INSTALLED: bool = false;
static mut ORIG_ON_SUCCESS_PROLOGUE: [u8; 16] = [0; 16];
static mut ON_SUCCESS_ADDR: usize = 0;
// ★ v3.23.3: API sniffing — use Hachimi Interceptor API (hook+trampoline) + WWWRequest.Post for URL (replaces _Send+SetHeader)
static SNIFF_ENABLED: AtomicBool = AtomicBool::new(true);
static SNIFF_MUTEX: Mutex<()> = Mutex::new(());
// Raw payloads and protocol observations use separate rings.
static mut SNIFF_REQUESTS: Vec<(u64, String, String, Vec<u8>)> = Vec::new();
static mut SNIFF_RESPONSES: Vec<(u64, Vec<u8>)> = Vec::new();
const SNIFF_RAW_MAX: usize = 50;
const SNIFF_METADATA_MAX: usize = 1000;
static SNIFF_REQ_ID: AtomicU64 = AtomicU64::new(1);
static SNIFF_METADATA_ID: AtomicU64 = AtomicU64::new(1);
#[derive(Clone)]
struct SniffMetadata {
    id: u64,
    request_id: u64,
    timestamp_ms: u64,
    direction: &'static str,
    path: String,
    size: usize,
    body_hex: String,
    headers: Vec<(String, String)>,
}
static mut SNIFF_METADATA: Vec<SniffMetadata> = Vec::new();
// Bounded temporal FIFO; unmatched responses are reported with request_id=0.
static mut SNIFF_RESPONSE_QUEUE: Vec<(u64, String)> = Vec::new();
static mut PENDING_URL: String = String::new();
static mut PENDING_HEADERS: Vec<(String, String)> = Vec::new();
static mut PENDING_REQ_ID: u64 = 0;
// CompressRequest/DecompressResponse/Post hook addresses (via Interceptor API)
static mut COMPRESS_REQUEST_ADDR: usize = 0;
static mut DECOMPRESS_RESPONSE_ADDR: usize = 0;
static mut POST_ADDR: usize = 0;
// UnityWebRequest request-entry observer. Full capture: headers, bodies, tokens and query strings.
static mut UNITY_SEND_ADDR: usize = 0;
// MakeMd5 hook
static mut MAKEMD5_ADDR: usize = 0;
static mut COMPUTEHASH_ADDR: usize = 0;
static MD5_LOG: Mutex<Vec<(String, String)>> = Mutex::new(Vec::new()); // (input, output)
static UNITY_OBSERVATION_ID: AtomicU64 = AtomicU64::new(1);
const UNITY_OBSERVATIONS_MAX: usize = 256;
#[derive(Clone)]
struct UnityRequestObservation {
    id: u64,
    timestamp_ms: u64,
    method: String,
    path: String,
    body_size: usize,
    body_hex: String,
    content_type: String,
}
static UNITY_OBSERVATIONS: Mutex<Vec<UnityRequestObservation>> = Mutex::new(Vec::new());
// Pending request body parking (CompressRequest → Post matching)
static mut PENDING_REQ_BODY: Option<Vec<u8>> = None;
static mut PENDING_COMPRESSED: usize = 0;

fn sniff_timestamp_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis().min(u128::from(u64::MAX)) as u64)
        .unwrap_or(0)
}

fn sniff_path(url: &str) -> String {
    let no_query = url.split('?').next().unwrap_or(url);
    if let Some(i) = no_query.find("://") {
        let rest = &no_query[i + 3..];
        return rest
            .find('/')
            .map(|j| rest[j..].to_string())
            .unwrap_or_else(|| "/".to_string());
    }
    no_query.to_string()
}

unsafe fn push_sniff_metadata(
    request_id: u64,
    direction: &'static str,
    url: &str,
    size: usize,
    body: &[u8],
    headers: Vec<(String, String)>,
) {
    let id = SNIFF_METADATA_ID.fetch_add(1, Ordering::Relaxed);
    let body_hex = body.iter().map(|b| format!("{:02x}", b)).collect::<String>();
    SNIFF_METADATA.push(SniffMetadata {
        id,
        request_id,
        timestamp_ms: sniff_timestamp_ms(),
        direction,
        path: sniff_path(url),
        size,
        body_hex,
        headers,
    });
    if SNIFF_METADATA.len() > SNIFF_METADATA_MAX {
        SNIFF_METADATA.remove(0);
    }
}
// ★ Mutex to prevent concurrent read_summary_inner calls from HTTP + push threads
static READ_MUTEX: Mutex<()> = Mutex::new(());

// ★ v3.24.2: Story event choice hook — capture career event choices (options, effects, branches)
static mut EVENT_CHOICE_HOOK_INSTALLED: bool = false;
static mut EVENT_CHOICE_ADDR: usize = 0; // StoryChoiceController.Choice
static mut EVENT_ADD_BTN_ADDR: usize = 0; // StoryChoiceController.AddChoiceButton
static mut ORIG_EVENT_CHOICE_PROLOGUE: [u8; 16] = [0; 16];
static mut ORIG_EVENT_ADD_BTN_PROLOGUE: [u8; 16] = [0; 16];
// ★ v3.24.2: StoryManager.SetStory hook — capture story_id and chara_id for event type identification
static mut STORY_SET_HOOK_INSTALLED: bool = false;
static mut STORY_SET_ADDR: usize = 0;
static mut ORIG_STORY_SET_PROLOGUE: [u8; 16] = [0; 16];
// Event state: accumulated choices for current event
static EVENT_STATE_MUTEX: Mutex<()> = Mutex::new(());

// ★ v3.24.40: mirror every ura_log line into a queryable ring buffer
// (Hachimi logcat was the only outlet before; /debug/hooklog exposes it).
static HOOK_LOG: Mutex<Vec<String>> = Mutex::new(Vec::new());
const HOOK_LOG_MAX: usize = 256;

// ★ v3.24.42: high-frequency read_summary/push spam is excluded from the
// ring (still goes to logcat) so event/sniff diagnostics survive.
const HOOK_LOG_NOISE: &[&str] = &[
    "★ read_summary",
    "ramen scalar",
    "ramen arrays",
    "evaluation_list",
    "sc: ",
    "skill_eval=",
    "v3.22.51 ramen",
    "★ Scenario 14",
    "Push:",
    "call_getter: 'get_Skill",
    "call_getter: 'get_PossessSkill",
    "find_field_offset: 'RemainTurn'",
];
fn hook_log(msg: &str) {
    if HOOK_LOG_NOISE.iter().any(|n| msg.contains(n)) {
        return;
    }
    if let Ok(mut g) = HOOK_LOG.lock() {
        if g.len() >= HOOK_LOG_MAX {
            g.remove(0);
        }
        g.push(msg.to_string());
    }
}

// ★ v3.24.40: per-hook install status for /debug/hookdiag
static HOOK_STATUS: Mutex<Vec<(String, String)>> = Mutex::new(Vec::new());
fn set_hook_status(name: &str, status: &str) {
    hook_log(&format!("hook[{}] = {}", name, status));
    if let Ok(mut g) = HOOK_STATUS.lock() {
        if let Some(e) = g.iter_mut().find(|(n, _)| n == name) {
            e.1 = status.to_string();
        } else {
            g.push((name.to_string(), status.to_string()));
        }
    }
}
static mut EVENT_CHOICES: Vec<EventChoice> = Vec::new();
static mut EVENT_SELECTED_IDX: i32 = -1;
static mut EVENT_STORY_ID: i32 = 0;
static mut EVENT_CHARA_ID: i32 = 0;

// Incremented whenever a new story_id takes over (or state is cleared).
// Guarded by EVENT_STATE_MUTEX; never read/write outside the lock.
static mut EVENT_GENERATION: u64 = 0;

// Cap against runaway AddChoiceButton repeats in abnormal UI rebuilds.
const EVENT_CHOICES_MAX: usize = 32;

#[derive(Clone)]
struct EventChoice {
    label: String,
    gain_id: i32,
    next_block_idx: i32,
    loop_exit_gain_id: i32,
}

// v3.24.73: bounded cache-only pairing. This is temporal co-occurrence,
// never a success/failure classification 
```

### match 2

```rust
" {
        // ★ v3.24.61: in-process meta dump (libnative sqlite + captured key)
        meta_dump_endpoint()
    } else if path == "/debug/resource_db_keys" {
        // ★ v3.24.45: full db open/key/mc_config pairing log
        let entries: Vec<String> = match DB_KEY_LOG.lock() {
            Ok(g) => g
                .iter()
                .map(|l| format!("\"{}\"", json_escape(l)))
                .collect(),
            Err(_) => Vec::new(),
        };
        format!(
            r#"{{"count":{},"entries":[{}]}}"#,
            entries.len(),
            entries.join(",")
        )
    } else if path == "/debug/resource_meta_key" {
        // ★ v3.24.44: captured SQLCipher key for the resource `meta` DB
        let key = META_KEY_HEX.lock().map(|g| g.clone()).unwrap_or_default();
        format!(
            r#"{{"captured":{},"key_len":{},"key_hex":"{}","persisted_file":"files/ura_meta_key.txt"}}"#,
            if key.is_empty() { "false" } else { "true" },
            key.len() / 2,
            key
        )
    } else if path == "/debug/hookdiag" {
        // ★ v3.24.40: per-hook install status
        let items: Vec<String> = match HOOK_STATUS.lock() {
            Ok(g) => g
                .iter()
                .map(|(n, st)| {
                    format!(
                        r#"{{"hook":"{}","status":"{}"}}"#,
                        json_escape(n),
                        json_escape(st)
                    )
                })
                .collect(),
            Err(_) => Vec::new(),
        };
        format!(
            r#"{{"game_initialized":{},"hooks":[{}]}}"#,
            GAME_INITIALIZED.load(Ordering::Relaxed),
            items.join(",")
        )
    } else if path.starts_with("/api/sniff/unity") {
        let after_id = parse_query(&full_uri, "after_id")
            .parse::<u64>()
            .unwrap_or(0);
        let entries = UNITY_OBSERVATIONS.lock().map(|g| {
        g.iter().filter(|x| x.id > after_id).map(|x| format!(
            r#"{{"id":{},"timestamp_ms":{},"method":"{}","path":"{}","body_size":{},"body_hex":"{}","content_type":"{}"}}"#,
            x.id, x.timestamp_ms, json_escape(&x.method), json_escape(&x.path),
            x.body_size, x.body_hex, json_escape(&x.content_type)
        )).collect::<Vec<_>>()
    }).unwrap_or_default();
        format!(
            r#"{{"enabled":{},"unity_send_hooked":{},"count":{},"entries":[{}]}}"#,
            SNIFF_ENABLED.load(Ordering::Relaxed),
            unsafe { UNITY_SEND_ADDR != 0 },
            entries.len(),
            entries.join(",")
        )
    } else if path == "/api/sniff/diag" {
        // v3.23.3: Diagnostic endpoint for hook installation (Interceptor API)
        let req_hooked = unsafe { COMPRESS_REQUEST_ADDR != 0 };
        let resp_hooked = unsafe { DECOMPRESS_RESPONSE_ADDR != 0 };
        let post_hooked = unsafe { POST_ADDR != 0 };
        let req_addr = unsafe { COMPRESS_REQUEST_ADDR };
        let resp_addr = unsafe { DECOMPRESS_RESPONSE_ADDR };
        let post_addr = unsafe { POST_ADDR };
        let makemd5_hooked = unsafe { MAKEMD5_ADDR != 0 };
        let makemd5_addr = unsafe { MAKEMD5_ADDR };
        let computehash_hooked = unsafe { COMPUTEHASH_ADDR != 0 };
        let computehash_addr = unsafe { COMPUTEHASH_ADDR };
        let interceptor_available = unsafe { !API.is_null() && (*API).interceptor != 0 };
        let has_get_method_addr =
            unsafe { !API.is_null() && (*API).il2cpp_get_method_addr_fn.is_some() };
        format!(
            r#"{{"sniff_enabled":{},"compress_hooked":{},"decompress_hooked":{},"post_hooked":{},"makemd5_hooked":{},"computehash_hooked":{},"compress_addr":"0x{:x}","decompress_addr":"0x{:x}","post_addr":"0x{:x}","makemd5_addr":"0x{:x}","computehash_addr":"0x{:x}","interceptor_available":{},"get_method_addr_available":{}}}"#,
            SNIFF_ENABLED.load(Ordering::Relaxed),
            req_hooked,
            resp_hooked,
            post_hooked,
            makemd5_hooked,
            computehash_hooked,
            req_addr,
            resp_addr,
            post_addr,
            makemd5_addr,
            computehash_addr,
            interceptor_available,
            has_get_method_addr
        )
    } else if path == "/api/md5log" {
        let log = MD5_LOG.lock().unwrap();
        let entries: Vec<String> = log.iter()
            .enumerate()
            .map(|(i, (input, output))| {
                format!(
                    r#"{{"id":{},"input":"{}","output":"{}"}}"#,
                    i,
                    input.replace('\\', "\\\\").replace('"', "\\\""),
                    output.replace('\\', "\\\\").replace('"', "\\\"")
                )
            })
            .collect();
        format!(r#"{{"count":{},"entries":[{}]}}"#, entries.len(), entries.join(","))
    } else if path == "/api/md5log/clear" {
        MD5_LOG.lock().unwrap().clear();
        r#"{"ok":true,"cleared":true}"#.to_string()
    } else if path == "/api/md5log/install" {
        // Scope early String returns to this route, not handle_http() -> ().
        (|| -> String {
        unsafe {
            if MAKEMD5_ADDR != 0 {
                format!(r#"{{"ok":true,"already_hooked":true,"addr":"0x{:x}"}}"#, MAKEMD5_ADDR)
            } else if API.is_null() || (*API).interceptor == 0 {
                r#"{"ok":false,"error":"interceptor_unavailable"}"#.to_string()
            } else {
                let get_asm = match (*API).il2cpp_get_assembly_image_fn {
                    Some(f) => f,
                    None => return r#"{"ok":false,"error":"no_get_asm"}"#.to_string(),
                };
                let get_class = match (*API).il2cpp_get_class_fn {
                    Some(f) => f,
                    None => return r#"{"ok":false,"error":"no_get_class"}"#.to_string(),
                };
                let get_method_addr = match (*API).il2cpp_get_method_addr_fn {
                    Some(f) => f,
                    None => return r#"{"ok":false,"error":"no_get_method_addr"}"#.to_string(),
                };

                let img = get_asm(to_cstr("umamusume.dll").as_ptr());
                if img.is_null() {
                    return r#"{"ok":false,"error":"umamusume_dll_not_found"}"#.to_string();
                }
                let cls = get_class(
                    img,
                    to_cstr("Gallop").as_ptr(),
                    to_cstr("Cryptographer").as_ptr(),
                );
                if cls.is_null() {
                    return r#"{"ok":false,"error":"cryptographer_class_not_found"}"#.to_string();
                }
                let addr = get_method_addr(
                    cls as usize,
                    to_cstr("MakeMd5").as_ptr(),
                    1,
                );
                if addr == 0 {
                    return r#"{"ok":false,"error":"makemd5_method_not_found"}"#.to_string();
                }
                if interceptor_hook(addr, makemd5_hook_handler as usize) {
                    MAKEMD5_ADDR = addr;
                    set_hook_status("sniff.makemd5", &format!("hooked@0x{:x}", addr));
                    format!(r#"{{"ok":true,"hooked":true,"addr":"0x{:x}"}}"#, addr)
                } else {
                    r#"{"ok":false,"error":"interceptor_hook_failed"}"#.to_string()
                }
            }
        }
        })()
    } else if path == "/api/sniff" {
        let _lock = SNIFF_MUTEX.lock();
        unsafe {
            l
```

### match 3

```rust
if p.is_null() {
            None
        } else {
            Some(std::mem::transmute(p))
        }
    };
    if get_methods_fn.is_none() || method_get_name_fn.is_none() {
        return 0;
    }
    let mut iter: *mut c_void = std::ptr::null_mut();
    loop {
        let mi = get_methods_fn.unwrap()(class, &mut iter);
        if mi.is_null() {
            break;
        }
        let name_ptr = method_get_name_fn.unwrap()(mi);
        if name_ptr.is_null() {
            continue;
        }
        let name = CStr::from_ptr(name_ptr).to_string_lossy();
        if name.contains(substr) {
            if let Some(get_ptr) = method_get_ptr_fn {
                let ptr = get_ptr(mi);
                if !ptr.is_null() {
                    ura_log(
                        3,
                        &format!(
                            "find_method_fuzzy: {}~{} -> 0x{:x}",
                            substr, name, ptr as usize
                        ),
                    );
                    return ptr as usize;
                }
            }
        }
    }
    0
}

/// ★ v3.24.40: fuzzy variant — first class whose name CONTAINS `substr`.
unsafe fn find_class_fuzzy(image: *const c_void, substr: &str) -> *mut c_void {
    let get_count_fn = resolve_il2cpp_symbol("il2cpp_image_get_class_count");
    let get_class_fn = resolve_il2cpp_symbol("il2cpp_image_get_class");
    let get_name_fn = resolve_il2cpp_symbol("il2cpp_class_get_name");
    if get_count_fn.is_null() || get_class_fn.is_null() || get_name_fn.is_null() {
        return ptr::null_mut();
    }
    let get_count: FnImageGetClassCount = std::mem::transmute(get_count_fn);
    let get_class: FnImageGetClass = std::mem::transmute(get_class_fn);
    let get_name: unsafe extern "C" fn(*const c_void) -> *const c_char =
        std::mem::transmute(get_name_fn);
    let count = get_count(image);
    for i in 0..count {
        let cls = get_class(image, i);
        if cls.is_null() {
            continue;
        }
        let np = get_name(cls);
        if np.is_null() {
            continue;
        }
        let name = CStr::from_ptr(np).to_string_lossy();
        if name.contains(substr) {
            ura_log(3, &format!("find_class_fuzzy: {}~{}", substr, name));
            return cls as *mut c_void;
        }
    }
    ptr::null_mut()
}

unsafe fn install_api_sniff_hooks() {
    let all_hooked = COMPRESS_REQUEST_ADDR != 0
        && DECOMPRESS_RESPONSE_ADDR != 0
        && POST_ADDR != 0
        && UNITY_SEND_ADDR != 0;
    if all_hooked {
        return;
    }
    if API.is_null() {
        ura_log(3, "API sniff: API is null");
        set_hook_status("sniff", "failed: api_null");
        return;
    }
    let api = &*API;
    if api.interceptor == 0 {
        ura_log(3, "API sniff: interceptor not available");
        set_hook_status("sniff", "failed: interceptor_unavailable");
        return;
    }

    // Get umamusume.dll assembly image
    let get_asm = match api.il2cpp_get_assembly_image_fn {
        Some(f) => f,
        None => {
            ura_log(3, "API sniff: get_assembly_image not available");
            return;
        }
    };
    let get_class = match api.il2cpp_get_class_fn {
        Some(f) => f,
        None => {
            ura_log(3, "API sniff: get_class not available");
            return;
        }
    };
    let get_method_addr = match api.il2cpp_get_method_addr_fn {
        Some(f) => f,
        None => {
            ura_log(3, "API sniff: get_method_addr not available");
            return;
        }
    };

    // Observe the lower UnityWebRequest request-entry path used by boot/auth traffic.
    if UNITY_SEND_ADDR == 0 {
        let unity_image = get_asm(to_cstr("UnityEngine.UnityWebRequestModule.dll").as_ptr());
        if unity_image.is_null() {
            set_hook_status("sniff.unity_send", "failed: image_not_found");
        } else {
            let unity_request = get_class(
                unity_image,
                to_cstr("UnityEngine.Networking").as_ptr(),
                to_cstr("UnityWebRequest").as_ptr(),
            );
            if unity_request.is_null() {
                set_hook_status("sniff.unity_send", "failed: class_not_found");
            } else {
                let addr = get_method_addr(
                    unity_request as usize,
                    to_cstr("SendWebRequest").as_ptr(),
                    0,
                );
                if addr == 0 {
                    set_hook_status("sniff.unity_send", "failed: method_not_found");
                } else if interceptor_hook(addr, unity_send_hook_handler as usize) {
                    UNITY_SEND_ADDR = addr;
                    set_hook_status("sniff.unity_send", &format!("hooked@0x{:x}", addr));
                    ura_log(
                        3,
                        &format!(
                            "API sniff: UnityWebRequest.SendWebRequest hooked at 0x{:x}",
                            addr
                        ),
                    );
                } else {
                    set_hook_status("sniff.unity_send", "failed: interceptor_hook");
                }
            }
        }
    }

    // Hook Cryptographer.MakeMd5 to capture salt
    if MAKEMD5_ADDR == 0 {
        let umamusume_img = get_asm(to_cstr("umamusume.dll").as_ptr());
        if !umamusume_img.is_null() {
            let crypto_class = get_class(
                umamusume_img,
                to_cstr("Gallop").as_ptr(),
                to_cstr("Cryptographer").as_ptr(),
            );
            if !crypto_class.is_null() {
                let addr = get_method_addr(
                    crypto_class as usize,
                    to_cstr("MakeMd5").as_ptr(),
                    1,
                );
                if addr != 0 {
                    if interceptor_hook(addr, makemd5_hook_handler as usize) {
                        MAKEMD5_ADDR = addr;
                        set_hook_status("sniff.makemd5", &format!("hooked@0x{:x}", addr));
                        ura_log(3, &format!("API sniff: Cryptographer.MakeMd5 hooked at 0x{:x}", addr));
                    } else {
                        set_hook_status("sniff.makemd5", "failed: interceptor_hook");
                    }
                }
                // Also hook ComputeHash to capture intermediate data (salted input)
                let ch_addr = get_method_addr(
                    crypto_class as usize,
                    to_cstr("ComputeHash").as_ptr(),
                    1,
                );
                if ch_addr != 0 {
                    if interceptor_hook(ch_addr, computehash_hook_handler as usize) {
                        COMPUTEHASH_ADDR = ch_addr;
                        set_hook_status("sniff.computehash", &format!("hooked@0x{:x}", ch_addr));
                        ura_log(3, &format!("API sniff: Cryptographer.ComputeHash hooked at 0x{:x}", ch_addr));
                    }
                }
            }
        }
    }

    let umamusume = get_asm(to_cstr("umamusume.dll").as_ptr());
    if umamusume.is_null() {
        ura_log(3, "API sniff: umamusume.dll image not found");
        set_hook_status("sniff", "failed: image_not_found");
        return;
    }

    // HttpHelper class (exact, then fuzzy fallback — v3.24.40)
    let mut http_helper = get_class(
        umamusume,
        to_cstr("Gallop").as_ptr(),
        to_cstr("HttpHelper").as_ptr(),
    );
    if http_helper.is_null() {
        http_helper = f
```

### match 4

```rust
str`.
unsafe fn find_class_fuzzy(image: *const c_void, substr: &str) -> *mut c_void {
    let get_count_fn = resolve_il2cpp_symbol("il2cpp_image_get_class_count");
    let get_class_fn = resolve_il2cpp_symbol("il2cpp_image_get_class");
    let get_name_fn = resolve_il2cpp_symbol("il2cpp_class_get_name");
    if get_count_fn.is_null() || get_class_fn.is_null() || get_name_fn.is_null() {
        return ptr::null_mut();
    }
    let get_count: FnImageGetClassCount = std::mem::transmute(get_count_fn);
    let get_class: FnImageGetClass = std::mem::transmute(get_class_fn);
    let get_name: unsafe extern "C" fn(*const c_void) -> *const c_char =
        std::mem::transmute(get_name_fn);
    let count = get_count(image);
    for i in 0..count {
        let cls = get_class(image, i);
        if cls.is_null() {
            continue;
        }
        let np = get_name(cls);
        if np.is_null() {
            continue;
        }
        let name = CStr::from_ptr(np).to_string_lossy();
        if name.contains(substr) {
            ura_log(3, &format!("find_class_fuzzy: {}~{}", substr, name));
            return cls as *mut c_void;
        }
    }
    ptr::null_mut()
}

unsafe fn install_api_sniff_hooks() {
    let all_hooked = COMPRESS_REQUEST_ADDR != 0
        && DECOMPRESS_RESPONSE_ADDR != 0
        && POST_ADDR != 0
        && UNITY_SEND_ADDR != 0;
    if all_hooked {
        return;
    }
    if API.is_null() {
        ura_log(3, "API sniff: API is null");
        set_hook_status("sniff", "failed: api_null");
        return;
    }
    let api = &*API;
    if api.interceptor == 0 {
        ura_log(3, "API sniff: interceptor not available");
        set_hook_status("sniff", "failed: interceptor_unavailable");
        return;
    }

    // Get umamusume.dll assembly image
    let get_asm = match api.il2cpp_get_assembly_image_fn {
        Some(f) => f,
        None => {
            ura_log(3, "API sniff: get_assembly_image not available");
            return;
        }
    };
    let get_class = match api.il2cpp_get_class_fn {
        Some(f) => f,
        None => {
            ura_log(3, "API sniff: get_class not available");
            return;
        }
    };
    let get_method_addr = match api.il2cpp_get_method_addr_fn {
        Some(f) => f,
        None => {
            ura_log(3, "API sniff: get_method_addr not available");
            return;
        }
    };

    // Observe the lower UnityWebRequest request-entry path used by boot/auth traffic.
    if UNITY_SEND_ADDR == 0 {
        let unity_image = get_asm(to_cstr("UnityEngine.UnityWebRequestModule.dll").as_ptr());
        if unity_image.is_null() {
            set_hook_status("sniff.unity_send", "failed: image_not_found");
        } else {
            let unity_request = get_class(
                unity_image,
                to_cstr("UnityEngine.Networking").as_ptr(),
                to_cstr("UnityWebRequest").as_ptr(),
            );
            if unity_request.is_null() {
                set_hook_status("sniff.unity_send", "failed: class_not_found");
            } else {
                let addr = get_method_addr(
                    unity_request as usize,
                    to_cstr("SendWebRequest").as_ptr(),
                    0,
                );
                if addr == 0 {
                    set_hook_status("sniff.unity_send", "failed: method_not_found");
                } else if interceptor_hook(addr, unity_send_hook_handler as usize) {
                    UNITY_SEND_ADDR = addr;
                    set_hook_status("sniff.unity_send", &format!("hooked@0x{:x}", addr));
                    ura_log(
                        3,
                        &format!(
                            "API sniff: UnityWebRequest.SendWebRequest hooked at 0x{:x}",
                            addr
                        ),
                    );
                } else {
                    set_hook_status("sniff.unity_send", "failed: interceptor_hook");
                }
            }
        }
    }

    // Hook Cryptographer.MakeMd5 to capture salt
    if MAKEMD5_ADDR == 0 {
        let umamusume_img = get_asm(to_cstr("umamusume.dll").as_ptr());
        if !umamusume_img.is_null() {
            let crypto_class = get_class(
                umamusume_img,
                to_cstr("Gallop").as_ptr(),
                to_cstr("Cryptographer").as_ptr(),
            );
            if !crypto_class.is_null() {
                let addr = get_method_addr(
                    crypto_class as usize,
                    to_cstr("MakeMd5").as_ptr(),
                    1,
                );
                if addr != 0 {
                    if interceptor_hook(addr, makemd5_hook_handler as usize) {
                        MAKEMD5_ADDR = addr;
                        set_hook_status("sniff.makemd5", &format!("hooked@0x{:x}", addr));
                        ura_log(3, &format!("API sniff: Cryptographer.MakeMd5 hooked at 0x{:x}", addr));
                    } else {
                        set_hook_status("sniff.makemd5", "failed: interceptor_hook");
                    }
                }
                // Also hook ComputeHash to capture intermediate data (salted input)
                let ch_addr = get_method_addr(
                    crypto_class as usize,
                    to_cstr("ComputeHash").as_ptr(),
                    1,
                );
                if ch_addr != 0 {
                    if interceptor_hook(ch_addr, computehash_hook_handler as usize) {
                        COMPUTEHASH_ADDR = ch_addr;
                        set_hook_status("sniff.computehash", &format!("hooked@0x{:x}", ch_addr));
                        ura_log(3, &format!("API sniff: Cryptographer.ComputeHash hooked at 0x{:x}", ch_addr));
                    }
                }
            }
        }
    }

    let umamusume = get_asm(to_cstr("umamusume.dll").as_ptr());
    if umamusume.is_null() {
        ura_log(3, "API sniff: umamusume.dll image not found");
        set_hook_status("sniff", "failed: image_not_found");
        return;
    }

    // HttpHelper class (exact, then fuzzy fallback — v3.24.40)
    let mut http_helper = get_class(
        umamusume,
        to_cstr("Gallop").as_ptr(),
        to_cstr("HttpHelper").as_ptr(),
    );
    if http_helper.is_null() {
        http_helper = find_class_fuzzy(umamusume, "HttpHelper");
    }
    if http_helper.is_null() {
        ura_log(3, "API sniff: HttpHelper class not found");
        set_hook_status("sniff", "failed: httphelper_class_not_found");
        return;
    }
    ura_log(3, "API sniff: HttpHelper class found");

    // Hook CompressRequest
    if COMPRESS_REQUEST_ADDR == 0 {
        let mut addr =
            get_method_addr(http_helper as usize, to_cstr("CompressRequest").as_ptr(), 1);
        if addr == 0 {
            addr = find_method_fuzzy(http_helper, "CompressRequest");
        }
        if addr != 0 {
            if interceptor_hook(addr, compress_request_hook_handler as usize) {
                COMPRESS_REQUEST_ADDR = addr;
                ura_log(
                    3,
                    &format!("API sniff: CompressRequest hooked at 0x{:x}", addr),
                );
                set_hook_status("sniff.compress", &format!("hooked@0x{:x}", addr));
            } else {
                ura_log(
                    3,
                    &format!("API sniff: CompressRequest hook FAILED at 0x{:x}", addr),
                );
                set_hook_s
```

### match 5

```rust
        if name.contains(substr) {
            ura_log(3, &format!("find_class_fuzzy: {}~{}", substr, name));
            return cls as *mut c_void;
        }
    }
    ptr::null_mut()
}

unsafe fn install_api_sniff_hooks() {
    let all_hooked = COMPRESS_REQUEST_ADDR != 0
        && DECOMPRESS_RESPONSE_ADDR != 0
        && POST_ADDR != 0
        && UNITY_SEND_ADDR != 0;
    if all_hooked {
        return;
    }
    if API.is_null() {
        ura_log(3, "API sniff: API is null");
        set_hook_status("sniff", "failed: api_null");
        return;
    }
    let api = &*API;
    if api.interceptor == 0 {
        ura_log(3, "API sniff: interceptor not available");
        set_hook_status("sniff", "failed: interceptor_unavailable");
        return;
    }

    // Get umamusume.dll assembly image
    let get_asm = match api.il2cpp_get_assembly_image_fn {
        Some(f) => f,
        None => {
            ura_log(3, "API sniff: get_assembly_image not available");
            return;
        }
    };
    let get_class = match api.il2cpp_get_class_fn {
        Some(f) => f,
        None => {
            ura_log(3, "API sniff: get_class not available");
            return;
        }
    };
    let get_method_addr = match api.il2cpp_get_method_addr_fn {
        Some(f) => f,
        None => {
            ura_log(3, "API sniff: get_method_addr not available");
            return;
        }
    };

    // Observe the lower UnityWebRequest request-entry path used by boot/auth traffic.
    if UNITY_SEND_ADDR == 0 {
        let unity_image = get_asm(to_cstr("UnityEngine.UnityWebRequestModule.dll").as_ptr());
        if unity_image.is_null() {
            set_hook_status("sniff.unity_send", "failed: image_not_found");
        } else {
            let unity_request = get_class(
                unity_image,
                to_cstr("UnityEngine.Networking").as_ptr(),
                to_cstr("UnityWebRequest").as_ptr(),
            );
            if unity_request.is_null() {
                set_hook_status("sniff.unity_send", "failed: class_not_found");
            } else {
                let addr = get_method_addr(
                    unity_request as usize,
                    to_cstr("SendWebRequest").as_ptr(),
                    0,
                );
                if addr == 0 {
                    set_hook_status("sniff.unity_send", "failed: method_not_found");
                } else if interceptor_hook(addr, unity_send_hook_handler as usize) {
                    UNITY_SEND_ADDR = addr;
                    set_hook_status("sniff.unity_send", &format!("hooked@0x{:x}", addr));
                    ura_log(
                        3,
                        &format!(
                            "API sniff: UnityWebRequest.SendWebRequest hooked at 0x{:x}",
                            addr
                        ),
                    );
                } else {
                    set_hook_status("sniff.unity_send", "failed: interceptor_hook");
                }
            }
        }
    }

    // Hook Cryptographer.MakeMd5 to capture salt
    if MAKEMD5_ADDR == 0 {
        let umamusume_img = get_asm(to_cstr("umamusume.dll").as_ptr());
        if !umamusume_img.is_null() {
            let crypto_class = get_class(
                umamusume_img,
                to_cstr("Gallop").as_ptr(),
                to_cstr("Cryptographer").as_ptr(),
            );
            if !crypto_class.is_null() {
                let addr = get_method_addr(
                    crypto_class as usize,
                    to_cstr("MakeMd5").as_ptr(),
                    1,
                );
                if addr != 0 {
                    if interceptor_hook(addr, makemd5_hook_handler as usize) {
                        MAKEMD5_ADDR = addr;
                        set_hook_status("sniff.makemd5", &format!("hooked@0x{:x}", addr));
                        ura_log(3, &format!("API sniff: Cryptographer.MakeMd5 hooked at 0x{:x}", addr));
                    } else {
                        set_hook_status("sniff.makemd5", "failed: interceptor_hook");
                    }
                }
                // Also hook ComputeHash to capture intermediate data (salted input)
                let ch_addr = get_method_addr(
                    crypto_class as usize,
                    to_cstr("ComputeHash").as_ptr(),
                    1,
                );
                if ch_addr != 0 {
                    if interceptor_hook(ch_addr, computehash_hook_handler as usize) {
                        COMPUTEHASH_ADDR = ch_addr;
                        set_hook_status("sniff.computehash", &format!("hooked@0x{:x}", ch_addr));
                        ura_log(3, &format!("API sniff: Cryptographer.ComputeHash hooked at 0x{:x}", ch_addr));
                    }
                }
            }
        }
    }

    let umamusume = get_asm(to_cstr("umamusume.dll").as_ptr());
    if umamusume.is_null() {
        ura_log(3, "API sniff: umamusume.dll image not found");
        set_hook_status("sniff", "failed: image_not_found");
        return;
    }

    // HttpHelper class (exact, then fuzzy fallback — v3.24.40)
    let mut http_helper = get_class(
        umamusume,
        to_cstr("Gallop").as_ptr(),
        to_cstr("HttpHelper").as_ptr(),
    );
    if http_helper.is_null() {
        http_helper = find_class_fuzzy(umamusume, "HttpHelper");
    }
    if http_helper.is_null() {
        ura_log(3, "API sniff: HttpHelper class not found");
        set_hook_status("sniff", "failed: httphelper_class_not_found");
        return;
    }
    ura_log(3, "API sniff: HttpHelper class found");

    // Hook CompressRequest
    if COMPRESS_REQUEST_ADDR == 0 {
        let mut addr =
            get_method_addr(http_helper as usize, to_cstr("CompressRequest").as_ptr(), 1);
        if addr == 0 {
            addr = find_method_fuzzy(http_helper, "CompressRequest");
        }
        if addr != 0 {
            if interceptor_hook(addr, compress_request_hook_handler as usize) {
                COMPRESS_REQUEST_ADDR = addr;
                ura_log(
                    3,
                    &format!("API sniff: CompressRequest hooked at 0x{:x}", addr),
                );
                set_hook_status("sniff.compress", &format!("hooked@0x{:x}", addr));
            } else {
                ura_log(
                    3,
                    &format!("API sniff: CompressRequest hook FAILED at 0x{:x}", addr),
                );
                set_hook_status("sniff.compress", "failed: interceptor_hook");
            }
        } else {
            ura_log(3, "API sniff: CompressRequest NOT FOUND");
            set_hook_status("sniff.compress", "failed: method_not_found");
        }
    }

    // Hook DecompressResponse
    if DECOMPRESS_RESPONSE_ADDR == 0 {
        let addr = get_method_addr(
            http_helper as usize,
            to_cstr("DecompressResponse").as_ptr(),
            1,
        );
        if addr != 0 {
            if interceptor_hook(addr, decompress_response_hook_handler as usize) {
                DECOMPRESS_RESPONSE_ADDR = addr;
                ura_log(
                    3,
                    &format!("API sniff: DecompressResponse hooked at 0x{:x}", addr),
                );
                set_hook_status("sniff.decompress", &format!("hooked@0x{:x}", addr));
            } else {
                ura_log(
                    3,
                    &format!("API sniff: DecompressResponse hook FAILED a
```

## `push_sniff_metadata(` (3 matches)

### match 1

```rust
(u64, String, String, Vec<u8>)> = Vec::new();
static mut SNIFF_RESPONSES: Vec<(u64, Vec<u8>)> = Vec::new();
const SNIFF_RAW_MAX: usize = 50;
const SNIFF_METADATA_MAX: usize = 1000;
static SNIFF_REQ_ID: AtomicU64 = AtomicU64::new(1);
static SNIFF_METADATA_ID: AtomicU64 = AtomicU64::new(1);
#[derive(Clone)]
struct SniffMetadata {
    id: u64,
    request_id: u64,
    timestamp_ms: u64,
    direction: &'static str,
    path: String,
    size: usize,
    body_hex: String,
    headers: Vec<(String, String)>,
}
static mut SNIFF_METADATA: Vec<SniffMetadata> = Vec::new();
// Bounded temporal FIFO; unmatched responses are reported with request_id=0.
static mut SNIFF_RESPONSE_QUEUE: Vec<(u64, String)> = Vec::new();
static mut PENDING_URL: String = String::new();
static mut PENDING_HEADERS: Vec<(String, String)> = Vec::new();
static mut PENDING_REQ_ID: u64 = 0;
// CompressRequest/DecompressResponse/Post hook addresses (via Interceptor API)
static mut COMPRESS_REQUEST_ADDR: usize = 0;
static mut DECOMPRESS_RESPONSE_ADDR: usize = 0;
static mut POST_ADDR: usize = 0;
// UnityWebRequest request-entry observer. Full capture: headers, bodies, tokens and query strings.
static mut UNITY_SEND_ADDR: usize = 0;
// MakeMd5 hook
static mut MAKEMD5_ADDR: usize = 0;
static mut COMPUTEHASH_ADDR: usize = 0;
static MD5_LOG: Mutex<Vec<(String, String)>> = Mutex::new(Vec::new()); // (input, output)
static UNITY_OBSERVATION_ID: AtomicU64 = AtomicU64::new(1);
const UNITY_OBSERVATIONS_MAX: usize = 256;
#[derive(Clone)]
struct UnityRequestObservation {
    id: u64,
    timestamp_ms: u64,
    method: String,
    path: String,
    body_size: usize,
    body_hex: String,
    content_type: String,
}
static UNITY_OBSERVATIONS: Mutex<Vec<UnityRequestObservation>> = Mutex::new(Vec::new());
// Pending request body parking (CompressRequest → Post matching)
static mut PENDING_REQ_BODY: Option<Vec<u8>> = None;
static mut PENDING_COMPRESSED: usize = 0;

fn sniff_timestamp_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis().min(u128::from(u64::MAX)) as u64)
        .unwrap_or(0)
}

fn sniff_path(url: &str) -> String {
    let no_query = url.split('?').next().unwrap_or(url);
    if let Some(i) = no_query.find("://") {
        let rest = &no_query[i + 3..];
        return rest
            .find('/')
            .map(|j| rest[j..].to_string())
            .unwrap_or_else(|| "/".to_string());
    }
    no_query.to_string()
}

unsafe fn push_sniff_metadata(
    request_id: u64,
    direction: &'static str,
    url: &str,
    size: usize,
    body: &[u8],
    headers: Vec<(String, String)>,
) {
    let id = SNIFF_METADATA_ID.fetch_add(1, Ordering::Relaxed);
    let body_hex = body.iter().map(|b| format!("{:02x}", b)).collect::<String>();
    SNIFF_METADATA.push(SniffMetadata {
        id,
        request_id,
        timestamp_ms: sniff_timestamp_ms(),
        direction,
        path: sniff_path(url),
        size,
        body_hex,
        headers,
    });
    if SNIFF_METADATA.len() > SNIFF_METADATA_MAX {
        SNIFF_METADATA.remove(0);
    }
}
// ★ Mutex to prevent concurrent read_summary_inner calls from HTTP + push threads
static READ_MUTEX: Mutex<()> = Mutex::new(());

// ★ v3.24.2: Story event choice hook — capture career event choices (options, effects, branches)
static mut EVENT_CHOICE_HOOK_INSTALLED: bool = false;
static mut EVENT_CHOICE_ADDR: usize = 0; // StoryChoiceController.Choice
static mut EVENT_ADD_BTN_ADDR: usize = 0; // StoryChoiceController.AddChoiceButton
static mut ORIG_EVENT_CHOICE_PROLOGUE: [u8; 16] = [0; 16];
static mut ORIG_EVENT_ADD_BTN_PROLOGUE: [u8; 16] = [0; 16];
// ★ v3.24.2: StoryManager.SetStory hook — capture story_id and chara_id for event type identification
static mut STORY_SET_HOOK_INSTALLED: bool = false;
static mut STORY_SET_ADDR: usize = 0;
static mut ORIG_STORY_SET_PROLOGUE: [u8; 16] = [0; 16];
// Event state: accumulated choices for current event
static EVENT_STATE_MUTEX: Mutex<()> = Mutex::new(());

// ★ v3.24.40: mirror every ura_log line into a queryable ring buffer
// (Hachimi logcat was the only outlet before; /debug/hooklog exposes it).
static HOOK_LOG: Mutex<Vec<String>> = Mutex::new(Vec::new());
const HOOK_LOG_MAX: usize = 256;

// ★ v3.24.42: high-frequency read_summary/push spam is excluded from the
// ring (still goes to logcat) so event/sniff diagnostics survive.
const HOOK_LOG_NOISE: &[&str] = &[
    "★ read_summary",
    "ramen scalar",
    "ramen arrays",
    "evaluation_list",
    "sc: ",
    "skill_eval=",
    "v3.22.51 ramen",
    "★ Scenario 14",
    "Push:",
    "call_getter: 'get_Skill",
    "call_getter: 'get_PossessSkill",
    "find_field_offset: 'RemainTurn'",
];
fn hook_log(msg: &str) {
    if HOOK_LOG_NOISE.iter().any(|n| msg.contains(n)) {
        return;
    }
    if let Ok(mut g) = HOOK_LOG.lock() {
        if g.len() >= HOOK_LOG_MAX {
            g.remove(0);
        }
        g.push(msg.to_string());
    }
}

// ★ v3.24.40: per-hook install status for /debug/hookdiag
static HOOK_STATUS: Mutex<Vec<(String, String)>> = Mutex::new(Vec::new());
fn set_hook_status(name: &str, status: &str) {
    hook_log(&format!("hook[{}] = {}", name, status));
    if let Ok(mut g) = HOOK_STATUS.lock() {
        if let Some(e) = g.iter_mut().find(|(n, _)| n == name) {
            e.1 = status.to_string();
        } else {
            g.push((name.to_string(), status.to_string()));
        }
    }
}
static mut EVENT_CHOICES: Vec<EventChoice> = Vec::new();
static mut EVENT_SELECTED_IDX: i32 = -1;
static mut EVENT_STORY_ID: i32 = 0;
static mut EVENT_CHARA_ID: i32 = 0;

// Incremented whenever a new story_id takes over (or state is cleared).
// Guarded by EVENT_STATE_MUTEX; never read/write outside the lock.
static mut EVENT_GENERATION: u64 = 0;

// Cap against runaway AddChoiceButton repeats in abnormal UI rebuilds.
const EVENT_CHOICES_MAX: usize = 32;

#[derive(Clone)]
struct EventChoice {
    label: String,
    gain_id: i32,
    next_block_idx: i32,
    loop_exit_gain_id: i32,
}

// v3.24.73: bounded cache-only pairing. This is temporal co-occurrence,
// never a success/failure classification or a causality claim.
#[derive(Clone)]
struct PendingEventSelection {
    captured_at: u64,
    generation: u64,
    story_id: i32,
    chara_id: i32,
    selected_idx_raw: i32,
    choice: Option<EventChoice>,
}
static EVENT_PENDING_RESULT: Mutex<Option<PendingEventSelection>> = Mutex::new(None);
static EVENT_OBSERVATIONS: Mutex<Vec<String>> = Mutex::new(Vec::new());
static EVENT_OBSERVATION_ID: AtomicU64 = AtomicU64::new(1);
const EVENT_OBSERVATIONS_MAX: usize = 16;
const EVENT_RESPONSE_PREVIEW_MAX: usize = 16 * 1024;

// ★ v3.24.2: Read C# string from IL2CPP String object
unsafe fn read_il2cpp_string(s: *const c_void) -> String {
    if s.is_null() {
        return String::new();
    }
    let len = std::ptr::read::<i32>((s as *const u8).offset(16) as *const i32);
    if len <= 0 || len > 4096 {
        return String::new();
    }
    let chars_ptr = (s as *const u8).offset(20);
    let chars_slice = std::slice::from_raw_parts(chars_ptr as *const u16, len as usize);
    String::from_utf16_lossy(chars_slice)
}

// ★ Push-to-app state (v3.10.0): auto-push /summary to uma-juece when data changes
static mut LAST_PUSH_HASH: u64 = 0;
static PUSH_INTERVAL_SECS: u64 = 1;

// ★ Config (v3.11.0): runtime config updated via POST /config from App
// No file editing needed — App settings page sends config to 
```

### match 2

```rust
ending) = EVENT_PENDING_RESULT.lock() {
                if let Some(sel) = pending.take() {
                    let preview_len = bytes.len().min(EVENT_RESPONSE_PREVIEW_MAX);
                    let preview = String::from_utf8_lossy(&bytes[..preview_len]);
                    let (label, gain_id, next_block_idx, loop_exit_gain_id) = match sel.choice {
                        Some(c) => (c.label, c.gain_id, c.next_block_idx, c.loop_exit_gain_id),
                        None => (String::new(), -1, -1, -1),
                    };
                    let observation_id = EVENT_OBSERVATION_ID.fetch_add(1, Ordering::Relaxed);
                    let record = format!(
                        r#"{{"schema_version":2,"observation_id":{},"source":"runtime_observation","causality":"unknown","result_label":"unknown","captured_at":{},"generation":{},"story_id":{},"chara_id":{},"selected_idx_raw":{},"choice":{{"label":"{}","gain_id":{},"next_block_idx":{},"loop_exit_gain_id":{}}},"response":{{"request_id":{},"url":"{}","size_captured":{},"preview_truncated":{},"hex_prefix":"{}","text_preview":"{}"}}}}"#,
                        observation_id,
                        sel.captured_at,
                        sel.generation,
                        sel.story_id,
                        sel.chara_id,
                        sel.selected_idx_raw,
                        json_escape(&label),
                        gain_id,
                        next_block_idx,
                        loop_exit_gain_id,
                        PENDING_REQ_ID,
                        json_escape(&PENDING_URL),
                        bytes.len(),
                        bytes.len() > preview_len,
                        hex_encode(&bytes[..bytes.len().min(64)]),
                        json_escape(&preview)
                    );
                    if let Ok(mut obs) = EVENT_OBSERVATIONS.lock() {
                        if obs.len() >= EVENT_OBSERVATIONS_MAX {
                            obs.remove(0);
                        }
                        obs.push(record);
                    }
                }
            }
        }
        if SNIFF_ENABLED.load(Ordering::Relaxed) {
            if !bytes.is_empty() {
                let _lock = SNIFF_MUTEX.lock();
                let (rid, response_url) = if SNIFF_RESPONSE_QUEUE.is_empty() {
                    (0, String::new())
                } else {
                    SNIFF_RESPONSE_QUEUE.remove(0)
                };
                push_sniff_metadata(rid, "response", &response_url, bytes.len(), &bytes, Vec::new());
                SNIFF_RESPONSES.push((rid, bytes));
                if SNIFF_RESPONSES.len() > SNIFF_RAW_MAX {
                    SNIFF_RESPONSES.remove(0);
                }
            }
        }
        decompressed
    }
}

// ★ v3.23.3: Hook handler for WWWRequest.Post(this, url, postData, headers)
// Captures URL + headers directly, and matches the parked request body from CompressRequest.
// This replaces the old _Send + SetHeader approach.
extern "C" fn post_hook_handler(
    this: *mut c_void,
    url: *const c_void,
    post_data: *mut c_void,
    headers: *mut c_void,
) -> *mut c_void {
    unsafe {
        let trampoline = interceptor_get_trampoline(post_hook_handler as usize);
        if trampoline == 0 {
            return std::ptr::null_mut();
        }
        type FnType = unsafe extern "C" fn(
            *mut c_void,
            *const c_void,
            *mut c_void,
            *mut c_void,
        ) -> *mut c_void;
        let original: FnType = std::mem::transmute(trampoline);

        // Capture URL
        let game_url = if !url.is_null() {
            read_il2cpp_string(url)
        } else {
            String::new()
        };
        let game_url = if game_url.is_empty() {
            None
        } else {
            Some(game_url)
        };

        // Capture headers from Dictionary<string,string>
        let req_headers = read_string_dict(headers);

        if SNIFF_ENABLED.load(Ordering::Relaxed) {
            let rid = SNIFF_REQ_ID.fetch_add(1, Ordering::Relaxed);
            PENDING_REQ_ID = rid;
            let body = PENDING_REQ_BODY.take().unwrap_or_default();
            let headers_json = format_headers_json(&req_headers);
            let url_str = game_url.clone().unwrap_or_default();
            {
                let _lock = SNIFF_MUTEX.lock();
                push_sniff_metadata(rid, "request", &url_str, body.len(), &body, req_headers.clone());
                SNIFF_RESPONSE_QUEUE.push((rid, url_str.clone()));
                if SNIFF_RESPONSE_QUEUE.len() > SNIFF_METADATA_MAX {
                    SNIFF_RESPONSE_QUEUE.remove(0);
                }
                SNIFF_REQUESTS.push((rid, url_str, headers_json, body));
                if SNIFF_REQUESTS.len() > SNIFF_RAW_MAX {
                    SNIFF_REQUESTS.remove(0);
                }
            }
            PENDING_URL = game_url.clone().unwrap_or_default();
            PENDING_HEADERS = req_headers.clone();
        }

        let _ = this;
        original(this, url, post_data, headers)
    }
}

// ★ v3.23.3: Read IL2CPP Dictionary<string,string> into Vec<(String,String)>
// Layout: [hdr 0x10][fields...]; _entries @+0x18, _count @+0x20
// Entry: [hashCode:i32][next:i32][key:ptr][value:ptr] = 24B per entry
unsafe fn read_string_dict(dict: *mut c_void) -> Vec<(String, String)> {
    let mut out = Vec::new();
    if dict.is_null() {
        return out;
    }
    let count = std::ptr::read_unaligned::<i32>((dict as *const u8).add(0x20) as *const i32);
    if count <= 0 {
        return out;
    }
    let entries = std::ptr::read_unaligned::<usize>((dict as *const u8).add(0x18) as *const usize);
    if entries == 0 {
        return out;
    }
    // Il2CppArray header: 0x20 bytes, then entries
    let capacity =
        std::ptr::read_unaligned::<usize>((entries as *const u8).add(0x18) as *const usize);
    let entries_base = entries + 0x20;
    for i in 0..capacity {
        let entry_addr = entries_base + i * 24;
        let hash_code = std::ptr::read_unaligned::<i32>((entry_addr as *const u8) as *const i32);
        if hash_code < 0 {
            continue;
        } // free entry
        let key =
            std::ptr::read_unaligned::<usize>((entry_addr as *const u8).add(8) as *const usize);
        let value =
            std::ptr::read_unaligned::<usize>((entry_addr as *const u8).add(16) as *const usize);
        let key_str = read_il2cpp_string(key as *const c_void);
        let val_str = read_il2cpp_string(value as *const c_void);
        out.push((key_str, val_str));
        if out.len() >= count as usize {
            break;
        }
    }
    out
}

// Format headers Vec to JSON string: {"key1":"val1","key2":"val2"}
unsafe fn format_headers_json(headers: &[(String, String)]) -> String {
    if headers.is_empty() {
        return "{}".to_string();
    }
    let mut s = String::from("{");
    for (i, (k, v)) in headers.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        let v_escaped = v.replace('\\', "\\\\").replace('"', "\\\"");
        s.push_str(&format!("\"{}\":\"{}\"", k, v_escaped));
    }
    s.push('}');
    s
}

// ============================================================
// ★ v3.24.44: SQLCipher key capture (route B: offline meta decryption)
// The game's resource index `meta` is a SQLCipher-encrypted SQLite DB
// (no plain header; libnative.so exports sqlite3_key/sqlite3
```

### match 3

```rust
f obs.len() >= EVENT_OBSERVATIONS_MAX {
                            obs.remove(0);
                        }
                        obs.push(record);
                    }
                }
            }
        }
        if SNIFF_ENABLED.load(Ordering::Relaxed) {
            if !bytes.is_empty() {
                let _lock = SNIFF_MUTEX.lock();
                let (rid, response_url) = if SNIFF_RESPONSE_QUEUE.is_empty() {
                    (0, String::new())
                } else {
                    SNIFF_RESPONSE_QUEUE.remove(0)
                };
                push_sniff_metadata(rid, "response", &response_url, bytes.len(), &bytes, Vec::new());
                SNIFF_RESPONSES.push((rid, bytes));
                if SNIFF_RESPONSES.len() > SNIFF_RAW_MAX {
                    SNIFF_RESPONSES.remove(0);
                }
            }
        }
        decompressed
    }
}

// ★ v3.23.3: Hook handler for WWWRequest.Post(this, url, postData, headers)
// Captures URL + headers directly, and matches the parked request body from CompressRequest.
// This replaces the old _Send + SetHeader approach.
extern "C" fn post_hook_handler(
    this: *mut c_void,
    url: *const c_void,
    post_data: *mut c_void,
    headers: *mut c_void,
) -> *mut c_void {
    unsafe {
        let trampoline = interceptor_get_trampoline(post_hook_handler as usize);
        if trampoline == 0 {
            return std::ptr::null_mut();
        }
        type FnType = unsafe extern "C" fn(
            *mut c_void,
            *const c_void,
            *mut c_void,
            *mut c_void,
        ) -> *mut c_void;
        let original: FnType = std::mem::transmute(trampoline);

        // Capture URL
        let game_url = if !url.is_null() {
            read_il2cpp_string(url)
        } else {
            String::new()
        };
        let game_url = if game_url.is_empty() {
            None
        } else {
            Some(game_url)
        };

        // Capture headers from Dictionary<string,string>
        let req_headers = read_string_dict(headers);

        if SNIFF_ENABLED.load(Ordering::Relaxed) {
            let rid = SNIFF_REQ_ID.fetch_add(1, Ordering::Relaxed);
            PENDING_REQ_ID = rid;
            let body = PENDING_REQ_BODY.take().unwrap_or_default();
            let headers_json = format_headers_json(&req_headers);
            let url_str = game_url.clone().unwrap_or_default();
            {
                let _lock = SNIFF_MUTEX.lock();
                push_sniff_metadata(rid, "request", &url_str, body.len(), &body, req_headers.clone());
                SNIFF_RESPONSE_QUEUE.push((rid, url_str.clone()));
                if SNIFF_RESPONSE_QUEUE.len() > SNIFF_METADATA_MAX {
                    SNIFF_RESPONSE_QUEUE.remove(0);
                }
                SNIFF_REQUESTS.push((rid, url_str, headers_json, body));
                if SNIFF_REQUESTS.len() > SNIFF_RAW_MAX {
                    SNIFF_REQUESTS.remove(0);
                }
            }
            PENDING_URL = game_url.clone().unwrap_or_default();
            PENDING_HEADERS = req_headers.clone();
        }

        let _ = this;
        original(this, url, post_data, headers)
    }
}

// ★ v3.23.3: Read IL2CPP Dictionary<string,string> into Vec<(String,String)>
// Layout: [hdr 0x10][fields...]; _entries @+0x18, _count @+0x20
// Entry: [hashCode:i32][next:i32][key:ptr][value:ptr] = 24B per entry
unsafe fn read_string_dict(dict: *mut c_void) -> Vec<(String, String)> {
    let mut out = Vec::new();
    if dict.is_null() {
        return out;
    }
    let count = std::ptr::read_unaligned::<i32>((dict as *const u8).add(0x20) as *const i32);
    if count <= 0 {
        return out;
    }
    let entries = std::ptr::read_unaligned::<usize>((dict as *const u8).add(0x18) as *const usize);
    if entries == 0 {
        return out;
    }
    // Il2CppArray header: 0x20 bytes, then entries
    let capacity =
        std::ptr::read_unaligned::<usize>((entries as *const u8).add(0x18) as *const usize);
    let entries_base = entries + 0x20;
    for i in 0..capacity {
        let entry_addr = entries_base + i * 24;
        let hash_code = std::ptr::read_unaligned::<i32>((entry_addr as *const u8) as *const i32);
        if hash_code < 0 {
            continue;
        } // free entry
        let key =
            std::ptr::read_unaligned::<usize>((entry_addr as *const u8).add(8) as *const usize);
        let value =
            std::ptr::read_unaligned::<usize>((entry_addr as *const u8).add(16) as *const usize);
        let key_str = read_il2cpp_string(key as *const c_void);
        let val_str = read_il2cpp_string(value as *const c_void);
        out.push((key_str, val_str));
        if out.len() >= count as usize {
            break;
        }
    }
    out
}

// Format headers Vec to JSON string: {"key1":"val1","key2":"val2"}
unsafe fn format_headers_json(headers: &[(String, String)]) -> String {
    if headers.is_empty() {
        return "{}".to_string();
    }
    let mut s = String::from("{");
    for (i, (k, v)) in headers.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        let v_escaped = v.replace('\\', "\\\\").replace('"', "\\\"");
        s.push_str(&format!("\"{}\":\"{}\"", k, v_escaped));
    }
    s.push('}');
    s
}

// ============================================================
// ★ v3.24.44: SQLCipher key capture (route B: offline meta decryption)
// The game's resource index `meta` is a SQLCipher-encrypted SQLite DB
// (no plain header; libnative.so exports sqlite3_key/sqlite3_key_v2).
// Hook the keying functions at plugin init (before the game opens meta),
// capture the key bytes, persist to the private files dir.
static META_KEY_HEX: Mutex<String> = Mutex::new(String::new());
static mut SQLCIPHER_KEY_HOOK_DONE: bool = false;

// ★ v3.24.45: pair db handle -> filename -> key + cipher config.
// (v3.24.44's "first key wins" caught the WRONG database's key.)
static DB_HANDLES: Mutex<Vec<(usize, String)>> = Mutex::new(Vec::new());
static DB_KEY_LOG: Mutex<Vec<String>> = Mutex::new(Vec::new());
fn db_track(entry: String) {
    hook_log(&entry);
    if let Ok(mut g) = DB_KEY_LOG.lock() {
        if g.len() >= 96 {
            g.remove(0);
        }
        g.push(entry);
    }
}
fn db_file_of(handle: usize) -> String {
    DB_HANDLES
        .lock()
        .ok()
        .and_then(|g| g.iter().find(|(h, _)| *h == handle).map(|(_, f)| f.clone()))
        .unwrap_or_else(|| "?".to_string())
}

/// ★ v3.24.46: read a C string at a raw address ONLY if it lies inside a
/// readable mapped region (mc_config varargs may or may not be pointers).
unsafe fn safe_read_cstr(addr: usize, max: usize) -> String {
    if addr < 0x10000 {
        return String::new();
    }
    if let Ok(maps) = std::fs::read_to_string("/proc/self/maps") {
        for line in maps.lines() {
            let mut parts = line.split_whitespace();
            let range = match parts.next() {
                Some(r) => r,
                None => continue,
            };
            let (a, b) = match range.split_once('-') {
                Some(x) => x,
                None => continue,
            };
            let sa = match usize::from_str_radix(a, 16) {
                Ok(v) => v,
                Err(_) => continue,
            };
            let ea = match usize::from_str_radix(b, 16) {
                Ok(v) => v,
                Err(_) => continue,
            };
            if addr >= sa && add
```

## `SNIFF_RESPONSE_QUEUE` (7 matches)

### match 1

```rust
" fn(usize, *mut c_void, *mut c_void) -> *mut c_void>,
    interceptor_get_trampoline_addr_fn:
        Option<unsafe extern "C" fn(usize, *mut c_void) -> *mut c_void>,
    il2cpp_get_method_addr_fn: Option<unsafe extern "C" fn(usize, *const c_char, i32) -> usize>,
}

static mut API: *mut Api = ptr::null_mut();
static GAME_INITIALIZED: AtomicBool = AtomicBool::new(false);
static HTTP_RUNNING: AtomicBool = AtomicBool::new(false);
static PREDICT_STEP: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
static CRASH_SIG: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(0);
static CRASH_STEP: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
static mut LAST_STEP_BUF: [u8; 128] = [0; 128];
static LAST_STEP_LEN: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
static AUTO_UPDATE_STATUS: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);
// ★ Training result/action state is shared by the game hook and HTTP/summary threads.
// Keep correlated fields under one mutex to avoid data races and torn records.
struct ActionState {
    training_result: i32,
    training_sub_id: i32,
    command_id: i32,
    sequence: u64,
}
static ACTION_STATE: Mutex<ActionState> = Mutex::new(ActionState {
    training_result: -1,
    training_sub_id: -1,
    command_id: -1,
    sequence: 0,
});
static mut TRAINING_HOOK_INSTALLED: bool = false;
static mut ORIG_ON_SUCCESS_PROLOGUE: [u8; 16] = [0; 16];
static mut ON_SUCCESS_ADDR: usize = 0;
// ★ v3.23.3: API sniffing — use Hachimi Interceptor API (hook+trampoline) + WWWRequest.Post for URL (replaces _Send+SetHeader)
static SNIFF_ENABLED: AtomicBool = AtomicBool::new(true);
static SNIFF_MUTEX: Mutex<()> = Mutex::new(());
// Raw payloads and protocol observations use separate rings.
static mut SNIFF_REQUESTS: Vec<(u64, String, String, Vec<u8>)> = Vec::new();
static mut SNIFF_RESPONSES: Vec<(u64, Vec<u8>)> = Vec::new();
const SNIFF_RAW_MAX: usize = 50;
const SNIFF_METADATA_MAX: usize = 1000;
static SNIFF_REQ_ID: AtomicU64 = AtomicU64::new(1);
static SNIFF_METADATA_ID: AtomicU64 = AtomicU64::new(1);
#[derive(Clone)]
struct SniffMetadata {
    id: u64,
    request_id: u64,
    timestamp_ms: u64,
    direction: &'static str,
    path: String,
    size: usize,
    body_hex: String,
    headers: Vec<(String, String)>,
}
static mut SNIFF_METADATA: Vec<SniffMetadata> = Vec::new();
// Bounded temporal FIFO; unmatched responses are reported with request_id=0.
static mut SNIFF_RESPONSE_QUEUE: Vec<(u64, String)> = Vec::new();
static mut PENDING_URL: String = String::new();
static mut PENDING_HEADERS: Vec<(String, String)> = Vec::new();
static mut PENDING_REQ_ID: u64 = 0;
// CompressRequest/DecompressResponse/Post hook addresses (via Interceptor API)
static mut COMPRESS_REQUEST_ADDR: usize = 0;
static mut DECOMPRESS_RESPONSE_ADDR: usize = 0;
static mut POST_ADDR: usize = 0;
// UnityWebRequest request-entry observer. Full capture: headers, bodies, tokens and query strings.
static mut UNITY_SEND_ADDR: usize = 0;
// MakeMd5 hook
static mut MAKEMD5_ADDR: usize = 0;
static mut COMPUTEHASH_ADDR: usize = 0;
static MD5_LOG: Mutex<Vec<(String, String)>> = Mutex::new(Vec::new()); // (input, output)
static UNITY_OBSERVATION_ID: AtomicU64 = AtomicU64::new(1);
const UNITY_OBSERVATIONS_MAX: usize = 256;
#[derive(Clone)]
struct UnityRequestObservation {
    id: u64,
    timestamp_ms: u64,
    method: String,
    path: String,
    body_size: usize,
    body_hex: String,
    content_type: String,
}
static UNITY_OBSERVATIONS: Mutex<Vec<UnityRequestObservation>> = Mutex::new(Vec::new());
// Pending request body parking (CompressRequest → Post matching)
static mut PENDING_REQ_BODY: Option<Vec<u8>> = None;
static mut PENDING_COMPRESSED: usize = 0;

fn sniff_timestamp_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis().min(u128::from(u64::MAX)) as u64)
        .unwrap_or(0)
}

fn sniff_path(url: &str) -> String {
    let no_query = url.split('?').next().unwrap_or(url);
    if let Some(i) = no_query.find("://") {
        let rest = &no_query[i + 3..];
        return rest
            .find('/')
            .map(|j| rest[j..].to_string())
            .unwrap_or_else(|| "/".to_string());
    }
    no_query.to_string()
}

unsafe fn push_sniff_metadata(
    request_id: u64,
    direction: &'static str,
    url: &str,
    size: usize,
    body: &[u8],
    headers: Vec<(String, String)>,
) {
    let id = SNIFF_METADATA_ID.fetch_add(1, Ordering::Relaxed);
    let body_hex = body.iter().map(|b| format!("{:02x}", b)).collect::<String>();
    SNIFF_METADATA.push(SniffMetadata {
        id,
        request_id,
        timestamp_ms: sniff_timestamp_ms(),
        direction,
        path: sniff_path(url),
        size,
        body_hex,
        headers,
    });
    if SNIFF_METADATA.len() > SNIFF_METADATA_MAX {
        SNIFF_METADATA.remove(0);
    }
}
// ★ Mutex to prevent concurrent read_summary_inner calls from HTTP + push threads
static READ_MUTEX: Mutex<()> = Mutex::new(());

// ★ v3.24.2: Story event choice hook — capture career event choices (options, effects, branches)
static mut EVENT_CHOICE_HOOK_INSTALLED: bool = false;
static mut EVENT_CHOICE_ADDR: usize = 0; // StoryChoiceController.Choice
static mut EVENT_ADD_BTN_ADDR: usize = 0; // StoryChoiceController.AddChoiceButton
static mut ORIG_EVENT_CHOICE_PROLOGUE: [u8; 16] = [0; 16];
static mut ORIG_EVENT_ADD_BTN_PROLOGUE: [u8; 16] = [0; 16];
// ★ v3.24.2: StoryManager.SetStory hook — capture story_id and chara_id for event type identification
static mut STORY_SET_HOOK_INSTALLED: bool = false;
static mut STORY_SET_ADDR: usize = 0;
static mut ORIG_STORY_SET_PROLOGUE: [u8; 16] = [0; 16];
// Event state: accumulated choices for current event
static EVENT_STATE_MUTEX: Mutex<()> = Mutex::new(());

// ★ v3.24.40: mirror every ura_log line into a queryable ring buffer
// (Hachimi logcat was the only outlet before; /debug/hooklog exposes it).
static HOOK_LOG: Mutex<Vec<String>> = Mutex::new(Vec::new());
const HOOK_LOG_MAX: usize = 256;

// ★ v3.24.42: high-frequency read_summary/push spam is excluded from the
// ring (still goes to logcat) so event/sniff diagnostics survive.
const HOOK_LOG_NOISE: &[&str] = &[
    "★ read_summary",
    "ramen scalar",
    "ramen arrays",
    "evaluation_list",
    "sc: ",
    "skill_eval=",
    "v3.22.51 ramen",
    "★ Scenario 14",
    "Push:",
    "call_getter: 'get_Skill",
    "call_getter: 'get_PossessSkill",
    "find_field_offset: 'RemainTurn'",
];
fn hook_log(msg: &str) {
    if HOOK_LOG_NOISE.iter().any(|n| msg.contains(n)) {
        return;
    }
    if let Ok(mut g) = HOOK_LOG.lock() {
        if g.len() >= HOOK_LOG_MAX {
            g.remove(0);
        }
        g.push(msg.to_string());
    }
}

// ★ v3.24.40: per-hook install status for /debug/hookdiag
static HOOK_STATUS: Mutex<Vec<(String, String)>> = Mutex::new(Vec::new());
fn set_hook_status(name: &str, status: &str) {
    hook_log(&format!("hook[{}] = {}", name, status));
    if let Ok(mut g) = HOOK_STATUS.lock() {
        if let Some(e) = g.iter_mut().find(|(n, _)| n == name) {
            e.1 = status.to_string();
        } else {
            g.push((name.to_string(), status.to_string()));
        }
    }
}
static mut EVENT_CHOICES: Vec<EventChoice> = Vec::new();
static mut EVENT_SELECTED_IDX: i32 = -1;
static mut EVENT_STORY_ID: i32 = 0;
static mut EVENT_CHARA_ID: i32 = 0;

// Incremented w
```

### match 2

```rust
,");
                    format!(r#"{{"id":{},"request_id":{},"timestamp_ms":{},"direction":"{}","path":"{}","size":{},"body_hex":"{}","headers":[{}]}}"#,
                        m.id, m.request_id, m.timestamp_ms, m.direction, json_escape(&m.path), m.size, m.body_hex, headers_json)
                })
                .collect();
            let last_id = SNIFF_METADATA.last().map(|m| m.id).unwrap_or(after_id);
            format!(
                r#"{{"enabled":{},"after_id":{},"last_id":{},"count":{},"entries":[{}]}}"#,
                SNIFF_ENABLED.load(Ordering::Relaxed),
                after_id,
                last_id,
                entries.len(),
                entries.join(",")
            )
        }
    } else if path == "/api/sniff/toggle" {
        // ★ v3.24.40: lazy retry for fallback-mode installs.
        unsafe {
            install_api_sniff_hooks();
        }
        // ★ If hooks installed successfully, game is ready — set GAME_INITIALIZED
        let any_hooked = unsafe {
            COMPRESS_REQUEST_ADDR != 0
                || DECOMPRESS_RESPONSE_ADDR != 0
                || POST_ADDR != 0
                || MAKEMD5_ADDR != 0
                || COMPUTEHASH_ADDR != 0
        };
        if any_hooked && !GAME_INITIALIZED.load(Ordering::Relaxed) {
            GAME_INITIALIZED.store(true, Ordering::Relaxed);
            unsafe {
                ura_log(3, "sniff/toggle: GAME_INITIALIZED set (hooks installed via toggle)");
            }
        }
        let requested = parse_query(&full_uri, "enabled");
        let new_val = match requested.as_str() {
            "1" | "true" => true,
            "0" | "false" => false,
            _ => !SNIFF_ENABLED.load(Ordering::Relaxed),
        };
        SNIFF_ENABLED.store(new_val, Ordering::Relaxed);
        let req_hooked = unsafe { COMPRESS_REQUEST_ADDR != 0 };
        let resp_hooked = unsafe { DECOMPRESS_RESPONSE_ADDR != 0 };
        let post_hooked = unsafe { POST_ADDR != 0 };
        format!(
            r#"{{"sniff_enabled":{},"compress_hooked":{},"decompress_hooked":{},"post_hooked":{}}}"#,
            new_val, req_hooked, resp_hooked, post_hooked
        )
    } else if path == "/api/sniff/clear" {
        let _lock = SNIFF_MUTEX.lock();
        unsafe {
            SNIFF_REQUESTS.clear();
            SNIFF_RESPONSES.clear();
            if let Ok(mut entries) = UNITY_OBSERVATIONS.lock() {
                entries.clear();
            }
            SNIFF_METADATA.clear();
            SNIFF_RESPONSE_QUEUE.clear();
            PENDING_REQ_BODY = None;
        }
        r#"{"ok":true}"#.to_string()
    } else if path.starts_with("/debug/hooklog") {
        // ★ v3.24.40/42: last HOOK_LOG_MAX lines, optional ?filter=substr
        let filter = parse_query(&full_uri, "filter");
        let entries: Vec<String> = match HOOK_LOG.lock() {
            Ok(g) => g
                .iter()
                .filter(|l| filter.is_empty() || l.contains(&filter))
                .map(|l| json_escape(l))
                .collect(),
            Err(_) => Vec::new(),
        };
        format!(
            r#"{{"count":{},"entries":[{}]}}"#,
            entries.len(),
            entries.join(",")
        )
    } else if path == "/debug/resource_reads" {
        // ★ v3.24.58: meta/dat file-read trace. Lazy-starts the /proc watcher
        // on first request (never at init — thread spawn in init context).
        start_res_fd_watcher();
        let entries: Vec<String> = match RES_READ_LOG.lock() {
            Ok(g) => g
                .iter()
                .map(|l| format!("\"{}\"", json_escape(l)))
                .collect(),
            Err(_) => Vec::new(),
        };
        format!(
            r#"{{"count":{},"entries":[{}]}}"#,
            entries.len(),
            entries.join(",")
        )
    } else if path.starts_with("/debug/mem_scan_sqlite") {
        // ★ v3.24.58: hunt plaintext "SQLite format 3" pages in process memory
        // — any custom decryption MUST materialize this in RAM.
        let max_hits: usize = parse_query(&full_uri, "max").parse().unwrap_or(8);
        let mut hits: Vec<String> = Vec::new();
        let needle = b"SQLite format 3 ";
        if let Ok(maps) = std::fs::read_to_string("/proc/self/maps") {
            let mem = std::fs::File::open("/proc/self/mem");
            use std::os::unix::fs::FileExt;
            if let Ok(mem) = mem {
                'outer: for line in maps.lines() {
                    let cols: Vec<&str> = line.split_whitespace().collect();
                    if cols.len() < 6 {
                        continue;
                    }
                    if !cols[1].contains("rw") {
                        continue;
                    }
                    let range: Vec<&str> = cols[0].split('-').collect();
                    if range.len() != 2 {
                        continue;
                    }
                    let (Ok(sa), Ok(ea)) = (
                        usize::from_str_radix(range[0], 16),
                        usize::from_str_radix(range[1], 16),
                    ) else {
                        continue;
                    };
                    let len = ea - sa;
                    if len < 4096 || len > 512 * 1024 * 1024 {
                        continue;
                    }
                    let mut off = 0usize;
                    while off < len {
                        let chunk = (4 * 1024 * 1024usize).min(len - off);
                        let mut buf = vec![0u8; chunk];
                        if mem.read_at(&mut buf, (sa + off) as u64).is_err() {
                            break;
                        }
                        for (i, w) in buf.windows(needle.len()).enumerate() {
                            if w == needle {
                                let abs = sa + off + i;
                                let after =
                                    &buf[i + needle.len()..(i + needle.len() + 16).min(buf.len())];
                                hits.push(format!("0x{:x} {}", abs, hex_encode(after)));
                                if hits.len() >= max_hits {
                                    break 'outer;
                                }
                            }
                        }
                        off += chunk;
                    }
                }
            }
        }
        format!(
            r#"{{"needle":"SQLite format 3","hits":{},"locations":[{}]}}"#,
            hits.len(),
            hits.iter()
                .map(|h| format!("\"{}\"", h))
                .collect::<Vec<_>>()
                .join(",")
        )
    } else if path == "/debug/mem_scan_zdict" {
        // ★ v3.24.63: hunt zstd dictionary magic (37 A4 30 EC) in ALL readable
        // memory regions (incl. r-- rodata of .so files). Each hit dumps 256KB
        // of context to the media dir for offline inspection.
        let needle = [0x37u8, 0xa4, 0x30, 0xec];
        let max_hits: usize = parse_query(&full_uri, "max").parse().unwrap_or(4);
        let mut hits: Vec<String> = Vec::new();
        if let Ok(maps) = std::fs::read_to_string("/proc/self/maps") {
            if let Ok(mem) = std::fs::File::open("/proc/self/mem") {
                use std::os::unix::fs::FileExt;
                'outer: for line in maps.lines() {
                    let cols: Vec<&str> = line.split_whitespace().collect();
                    if cols.len() < 2 {
                        continue;
        
```

### match 3

```rust
::transmute(trampoline);
        let decompressed = original(data);
        let bytes = read_il2cpp_byte_array(decompressed);
        if !bytes.is_empty() {
            if let Ok(mut pending) = EVENT_PENDING_RESULT.lock() {
                if let Some(sel) = pending.take() {
                    let preview_len = bytes.len().min(EVENT_RESPONSE_PREVIEW_MAX);
                    let preview = String::from_utf8_lossy(&bytes[..preview_len]);
                    let (label, gain_id, next_block_idx, loop_exit_gain_id) = match sel.choice {
                        Some(c) => (c.label, c.gain_id, c.next_block_idx, c.loop_exit_gain_id),
                        None => (String::new(), -1, -1, -1),
                    };
                    let observation_id = EVENT_OBSERVATION_ID.fetch_add(1, Ordering::Relaxed);
                    let record = format!(
                        r#"{{"schema_version":2,"observation_id":{},"source":"runtime_observation","causality":"unknown","result_label":"unknown","captured_at":{},"generation":{},"story_id":{},"chara_id":{},"selected_idx_raw":{},"choice":{{"label":"{}","gain_id":{},"next_block_idx":{},"loop_exit_gain_id":{}}},"response":{{"request_id":{},"url":"{}","size_captured":{},"preview_truncated":{},"hex_prefix":"{}","text_preview":"{}"}}}}"#,
                        observation_id,
                        sel.captured_at,
                        sel.generation,
                        sel.story_id,
                        sel.chara_id,
                        sel.selected_idx_raw,
                        json_escape(&label),
                        gain_id,
                        next_block_idx,
                        loop_exit_gain_id,
                        PENDING_REQ_ID,
                        json_escape(&PENDING_URL),
                        bytes.len(),
                        bytes.len() > preview_len,
                        hex_encode(&bytes[..bytes.len().min(64)]),
                        json_escape(&preview)
                    );
                    if let Ok(mut obs) = EVENT_OBSERVATIONS.lock() {
                        if obs.len() >= EVENT_OBSERVATIONS_MAX {
                            obs.remove(0);
                        }
                        obs.push(record);
                    }
                }
            }
        }
        if SNIFF_ENABLED.load(Ordering::Relaxed) {
            if !bytes.is_empty() {
                let _lock = SNIFF_MUTEX.lock();
                let (rid, response_url) = if SNIFF_RESPONSE_QUEUE.is_empty() {
                    (0, String::new())
                } else {
                    SNIFF_RESPONSE_QUEUE.remove(0)
                };
                push_sniff_metadata(rid, "response", &response_url, bytes.len(), &bytes, Vec::new());
                SNIFF_RESPONSES.push((rid, bytes));
                if SNIFF_RESPONSES.len() > SNIFF_RAW_MAX {
                    SNIFF_RESPONSES.remove(0);
                }
            }
        }
        decompressed
    }
}

// ★ v3.23.3: Hook handler for WWWRequest.Post(this, url, postData, headers)
// Captures URL + headers directly, and matches the parked request body from CompressRequest.
// This replaces the old _Send + SetHeader approach.
extern "C" fn post_hook_handler(
    this: *mut c_void,
    url: *const c_void,
    post_data: *mut c_void,
    headers: *mut c_void,
) -> *mut c_void {
    unsafe {
        let trampoline = interceptor_get_trampoline(post_hook_handler as usize);
        if trampoline == 0 {
            return std::ptr::null_mut();
        }
        type FnType = unsafe extern "C" fn(
            *mut c_void,
            *const c_void,
            *mut c_void,
            *mut c_void,
        ) -> *mut c_void;
        let original: FnType = std::mem::transmute(trampoline);

        // Capture URL
        let game_url = if !url.is_null() {
            read_il2cpp_string(url)
        } else {
            String::new()
        };
        let game_url = if game_url.is_empty() {
            None
        } else {
            Some(game_url)
        };

        // Capture headers from Dictionary<string,string>
        let req_headers = read_string_dict(headers);

        if SNIFF_ENABLED.load(Ordering::Relaxed) {
            let rid = SNIFF_REQ_ID.fetch_add(1, Ordering::Relaxed);
            PENDING_REQ_ID = rid;
            let body = PENDING_REQ_BODY.take().unwrap_or_default();
            let headers_json = format_headers_json(&req_headers);
            let url_str = game_url.clone().unwrap_or_default();
            {
                let _lock = SNIFF_MUTEX.lock();
                push_sniff_metadata(rid, "request", &url_str, body.len(), &body, req_headers.clone());
                SNIFF_RESPONSE_QUEUE.push((rid, url_str.clone()));
                if SNIFF_RESPONSE_QUEUE.len() > SNIFF_METADATA_MAX {
                    SNIFF_RESPONSE_QUEUE.remove(0);
                }
                SNIFF_REQUESTS.push((rid, url_str, headers_json, body));
                if SNIFF_REQUESTS.len() > SNIFF_RAW_MAX {
                    SNIFF_REQUESTS.remove(0);
                }
            }
            PENDING_URL = game_url.clone().unwrap_or_default();
            PENDING_HEADERS = req_headers.clone();
        }

        let _ = this;
        original(this, url, post_data, headers)
    }
}

// ★ v3.23.3: Read IL2CPP Dictionary<string,string> into Vec<(String,String)>
// Layout: [hdr 0x10][fields...]; _entries @+0x18, _count @+0x20
// Entry: [hashCode:i32][next:i32][key:ptr][value:ptr] = 24B per entry
unsafe fn read_string_dict(dict: *mut c_void) -> Vec<(String, String)> {
    let mut out = Vec::new();
    if dict.is_null() {
        return out;
    }
    let count = std::ptr::read_unaligned::<i32>((dict as *const u8).add(0x20) as *const i32);
    if count <= 0 {
        return out;
    }
    let entries = std::ptr::read_unaligned::<usize>((dict as *const u8).add(0x18) as *const usize);
    if entries == 0 {
        return out;
    }
    // Il2CppArray header: 0x20 bytes, then entries
    let capacity =
        std::ptr::read_unaligned::<usize>((entries as *const u8).add(0x18) as *const usize);
    let entries_base = entries + 0x20;
    for i in 0..capacity {
        let entry_addr = entries_base + i * 24;
        let hash_code = std::ptr::read_unaligned::<i32>((entry_addr as *const u8) as *const i32);
        if hash_code < 0 {
            continue;
        } // free entry
        let key =
            std::ptr::read_unaligned::<usize>((entry_addr as *const u8).add(8) as *const usize);
        let value =
            std::ptr::read_unaligned::<usize>((entry_addr as *const u8).add(16) as *const usize);
        let key_str = read_il2cpp_string(key as *const c_void);
        let val_str = read_il2cpp_string(value as *const c_void);
        out.push((key_str, val_str));
        if out.len() >= count as usize {
            break;
        }
    }
    out
}

// Format headers Vec to JSON string: {"key1":"val1","key2":"val2"}
unsafe fn format_headers_json(headers: &[(String, String)]) -> String {
    if headers.is_empty() {
        return "{}".to_string();
    }
    let mut s = String::from("{");
    for (i, (k, v)) in headers.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        let v_escaped = v.replace('\\', "\\\\").replace('"', "\\\"");
        s.push_str(&format!("\"{}\":\"{}\"", k, v_escaped));
    }
    s.push('}');
    s
}

// ============================================================
// ★ v3.24.44: SQLCi
```

### match 4

```rust
essed);
        if !bytes.is_empty() {
            if let Ok(mut pending) = EVENT_PENDING_RESULT.lock() {
                if let Some(sel) = pending.take() {
                    let preview_len = bytes.len().min(EVENT_RESPONSE_PREVIEW_MAX);
                    let preview = String::from_utf8_lossy(&bytes[..preview_len]);
                    let (label, gain_id, next_block_idx, loop_exit_gain_id) = match sel.choice {
                        Some(c) => (c.label, c.gain_id, c.next_block_idx, c.loop_exit_gain_id),
                        None => (String::new(), -1, -1, -1),
                    };
                    let observation_id = EVENT_OBSERVATION_ID.fetch_add(1, Ordering::Relaxed);
                    let record = format!(
                        r#"{{"schema_version":2,"observation_id":{},"source":"runtime_observation","causality":"unknown","result_label":"unknown","captured_at":{},"generation":{},"story_id":{},"chara_id":{},"selected_idx_raw":{},"choice":{{"label":"{}","gain_id":{},"next_block_idx":{},"loop_exit_gain_id":{}}},"response":{{"request_id":{},"url":"{}","size_captured":{},"preview_truncated":{},"hex_prefix":"{}","text_preview":"{}"}}}}"#,
                        observation_id,
                        sel.captured_at,
                        sel.generation,
                        sel.story_id,
                        sel.chara_id,
                        sel.selected_idx_raw,
                        json_escape(&label),
                        gain_id,
                        next_block_idx,
                        loop_exit_gain_id,
                        PENDING_REQ_ID,
                        json_escape(&PENDING_URL),
                        bytes.len(),
                        bytes.len() > preview_len,
                        hex_encode(&bytes[..bytes.len().min(64)]),
                        json_escape(&preview)
                    );
                    if let Ok(mut obs) = EVENT_OBSERVATIONS.lock() {
                        if obs.len() >= EVENT_OBSERVATIONS_MAX {
                            obs.remove(0);
                        }
                        obs.push(record);
                    }
                }
            }
        }
        if SNIFF_ENABLED.load(Ordering::Relaxed) {
            if !bytes.is_empty() {
                let _lock = SNIFF_MUTEX.lock();
                let (rid, response_url) = if SNIFF_RESPONSE_QUEUE.is_empty() {
                    (0, String::new())
                } else {
                    SNIFF_RESPONSE_QUEUE.remove(0)
                };
                push_sniff_metadata(rid, "response", &response_url, bytes.len(), &bytes, Vec::new());
                SNIFF_RESPONSES.push((rid, bytes));
                if SNIFF_RESPONSES.len() > SNIFF_RAW_MAX {
                    SNIFF_RESPONSES.remove(0);
                }
            }
        }
        decompressed
    }
}

// ★ v3.23.3: Hook handler for WWWRequest.Post(this, url, postData, headers)
// Captures URL + headers directly, and matches the parked request body from CompressRequest.
// This replaces the old _Send + SetHeader approach.
extern "C" fn post_hook_handler(
    this: *mut c_void,
    url: *const c_void,
    post_data: *mut c_void,
    headers: *mut c_void,
) -> *mut c_void {
    unsafe {
        let trampoline = interceptor_get_trampoline(post_hook_handler as usize);
        if trampoline == 0 {
            return std::ptr::null_mut();
        }
        type FnType = unsafe extern "C" fn(
            *mut c_void,
            *const c_void,
            *mut c_void,
            *mut c_void,
        ) -> *mut c_void;
        let original: FnType = std::mem::transmute(trampoline);

        // Capture URL
        let game_url = if !url.is_null() {
            read_il2cpp_string(url)
        } else {
            String::new()
        };
        let game_url = if game_url.is_empty() {
            None
        } else {
            Some(game_url)
        };

        // Capture headers from Dictionary<string,string>
        let req_headers = read_string_dict(headers);

        if SNIFF_ENABLED.load(Ordering::Relaxed) {
            let rid = SNIFF_REQ_ID.fetch_add(1, Ordering::Relaxed);
            PENDING_REQ_ID = rid;
            let body = PENDING_REQ_BODY.take().unwrap_or_default();
            let headers_json = format_headers_json(&req_headers);
            let url_str = game_url.clone().unwrap_or_default();
            {
                let _lock = SNIFF_MUTEX.lock();
                push_sniff_metadata(rid, "request", &url_str, body.len(), &body, req_headers.clone());
                SNIFF_RESPONSE_QUEUE.push((rid, url_str.clone()));
                if SNIFF_RESPONSE_QUEUE.len() > SNIFF_METADATA_MAX {
                    SNIFF_RESPONSE_QUEUE.remove(0);
                }
                SNIFF_REQUESTS.push((rid, url_str, headers_json, body));
                if SNIFF_REQUESTS.len() > SNIFF_RAW_MAX {
                    SNIFF_REQUESTS.remove(0);
                }
            }
            PENDING_URL = game_url.clone().unwrap_or_default();
            PENDING_HEADERS = req_headers.clone();
        }

        let _ = this;
        original(this, url, post_data, headers)
    }
}

// ★ v3.23.3: Read IL2CPP Dictionary<string,string> into Vec<(String,String)>
// Layout: [hdr 0x10][fields...]; _entries @+0x18, _count @+0x20
// Entry: [hashCode:i32][next:i32][key:ptr][value:ptr] = 24B per entry
unsafe fn read_string_dict(dict: *mut c_void) -> Vec<(String, String)> {
    let mut out = Vec::new();
    if dict.is_null() {
        return out;
    }
    let count = std::ptr::read_unaligned::<i32>((dict as *const u8).add(0x20) as *const i32);
    if count <= 0 {
        return out;
    }
    let entries = std::ptr::read_unaligned::<usize>((dict as *const u8).add(0x18) as *const usize);
    if entries == 0 {
        return out;
    }
    // Il2CppArray header: 0x20 bytes, then entries
    let capacity =
        std::ptr::read_unaligned::<usize>((entries as *const u8).add(0x18) as *const usize);
    let entries_base = entries + 0x20;
    for i in 0..capacity {
        let entry_addr = entries_base + i * 24;
        let hash_code = std::ptr::read_unaligned::<i32>((entry_addr as *const u8) as *const i32);
        if hash_code < 0 {
            continue;
        } // free entry
        let key =
            std::ptr::read_unaligned::<usize>((entry_addr as *const u8).add(8) as *const usize);
        let value =
            std::ptr::read_unaligned::<usize>((entry_addr as *const u8).add(16) as *const usize);
        let key_str = read_il2cpp_string(key as *const c_void);
        let val_str = read_il2cpp_string(value as *const c_void);
        out.push((key_str, val_str));
        if out.len() >= count as usize {
            break;
        }
    }
    out
}

// Format headers Vec to JSON string: {"key1":"val1","key2":"val2"}
unsafe fn format_headers_json(headers: &[(String, String)]) -> String {
    if headers.is_empty() {
        return "{}".to_string();
    }
    let mut s = String::from("{");
    for (i, (k, v)) in headers.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        let v_escaped = v.replace('\\', "\\\\").replace('"', "\\\"");
        s.push_str(&format!("\"{}\":\"{}\"", k, v_escaped));
    }
    s.push('}');
    s
}

// ============================================================
// ★ v3.24.44: SQLCipher key capture (route B: offline meta decryption)
// The game's resource index `meta` is a SQLCipher-encrypted SQLit
```

### match 5

```rust
    }
                        obs.push(record);
                    }
                }
            }
        }
        if SNIFF_ENABLED.load(Ordering::Relaxed) {
            if !bytes.is_empty() {
                let _lock = SNIFF_MUTEX.lock();
                let (rid, response_url) = if SNIFF_RESPONSE_QUEUE.is_empty() {
                    (0, String::new())
                } else {
                    SNIFF_RESPONSE_QUEUE.remove(0)
                };
                push_sniff_metadata(rid, "response", &response_url, bytes.len(), &bytes, Vec::new());
                SNIFF_RESPONSES.push((rid, bytes));
                if SNIFF_RESPONSES.len() > SNIFF_RAW_MAX {
                    SNIFF_RESPONSES.remove(0);
                }
            }
        }
        decompressed
    }
}

// ★ v3.23.3: Hook handler for WWWRequest.Post(this, url, postData, headers)
// Captures URL + headers directly, and matches the parked request body from CompressRequest.
// This replaces the old _Send + SetHeader approach.
extern "C" fn post_hook_handler(
    this: *mut c_void,
    url: *const c_void,
    post_data: *mut c_void,
    headers: *mut c_void,
) -> *mut c_void {
    unsafe {
        let trampoline = interceptor_get_trampoline(post_hook_handler as usize);
        if trampoline == 0 {
            return std::ptr::null_mut();
        }
        type FnType = unsafe extern "C" fn(
            *mut c_void,
            *const c_void,
            *mut c_void,
            *mut c_void,
        ) -> *mut c_void;
        let original: FnType = std::mem::transmute(trampoline);

        // Capture URL
        let game_url = if !url.is_null() {
            read_il2cpp_string(url)
        } else {
            String::new()
        };
        let game_url = if game_url.is_empty() {
            None
        } else {
            Some(game_url)
        };

        // Capture headers from Dictionary<string,string>
        let req_headers = read_string_dict(headers);

        if SNIFF_ENABLED.load(Ordering::Relaxed) {
            let rid = SNIFF_REQ_ID.fetch_add(1, Ordering::Relaxed);
            PENDING_REQ_ID = rid;
            let body = PENDING_REQ_BODY.take().unwrap_or_default();
            let headers_json = format_headers_json(&req_headers);
            let url_str = game_url.clone().unwrap_or_default();
            {
                let _lock = SNIFF_MUTEX.lock();
                push_sniff_metadata(rid, "request", &url_str, body.len(), &body, req_headers.clone());
                SNIFF_RESPONSE_QUEUE.push((rid, url_str.clone()));
                if SNIFF_RESPONSE_QUEUE.len() > SNIFF_METADATA_MAX {
                    SNIFF_RESPONSE_QUEUE.remove(0);
                }
                SNIFF_REQUESTS.push((rid, url_str, headers_json, body));
                if SNIFF_REQUESTS.len() > SNIFF_RAW_MAX {
                    SNIFF_REQUESTS.remove(0);
                }
            }
            PENDING_URL = game_url.clone().unwrap_or_default();
            PENDING_HEADERS = req_headers.clone();
        }

        let _ = this;
        original(this, url, post_data, headers)
    }
}

// ★ v3.23.3: Read IL2CPP Dictionary<string,string> into Vec<(String,String)>
// Layout: [hdr 0x10][fields...]; _entries @+0x18, _count @+0x20
// Entry: [hashCode:i32][next:i32][key:ptr][value:ptr] = 24B per entry
unsafe fn read_string_dict(dict: *mut c_void) -> Vec<(String, String)> {
    let mut out = Vec::new();
    if dict.is_null() {
        return out;
    }
    let count = std::ptr::read_unaligned::<i32>((dict as *const u8).add(0x20) as *const i32);
    if count <= 0 {
        return out;
    }
    let entries = std::ptr::read_unaligned::<usize>((dict as *const u8).add(0x18) as *const usize);
    if entries == 0 {
        return out;
    }
    // Il2CppArray header: 0x20 bytes, then entries
    let capacity =
        std::ptr::read_unaligned::<usize>((entries as *const u8).add(0x18) as *const usize);
    let entries_base = entries + 0x20;
    for i in 0..capacity {
        let entry_addr = entries_base + i * 24;
        let hash_code = std::ptr::read_unaligned::<i32>((entry_addr as *const u8) as *const i32);
        if hash_code < 0 {
            continue;
        } // free entry
        let key =
            std::ptr::read_unaligned::<usize>((entry_addr as *const u8).add(8) as *const usize);
        let value =
            std::ptr::read_unaligned::<usize>((entry_addr as *const u8).add(16) as *const usize);
        let key_str = read_il2cpp_string(key as *const c_void);
        let val_str = read_il2cpp_string(value as *const c_void);
        out.push((key_str, val_str));
        if out.len() >= count as usize {
            break;
        }
    }
    out
}

// Format headers Vec to JSON string: {"key1":"val1","key2":"val2"}
unsafe fn format_headers_json(headers: &[(String, String)]) -> String {
    if headers.is_empty() {
        return "{}".to_string();
    }
    let mut s = String::from("{");
    for (i, (k, v)) in headers.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        let v_escaped = v.replace('\\', "\\\\").replace('"', "\\\"");
        s.push_str(&format!("\"{}\":\"{}\"", k, v_escaped));
    }
    s.push('}');
    s
}

// ============================================================
// ★ v3.24.44: SQLCipher key capture (route B: offline meta decryption)
// The game's resource index `meta` is a SQLCipher-encrypted SQLite DB
// (no plain header; libnative.so exports sqlite3_key/sqlite3_key_v2).
// Hook the keying functions at plugin init (before the game opens meta),
// capture the key bytes, persist to the private files dir.
static META_KEY_HEX: Mutex<String> = Mutex::new(String::new());
static mut SQLCIPHER_KEY_HOOK_DONE: bool = false;

// ★ v3.24.45: pair db handle -> filename -> key + cipher config.
// (v3.24.44's "first key wins" caught the WRONG database's key.)
static DB_HANDLES: Mutex<Vec<(usize, String)>> = Mutex::new(Vec::new());
static DB_KEY_LOG: Mutex<Vec<String>> = Mutex::new(Vec::new());
fn db_track(entry: String) {
    hook_log(&entry);
    if let Ok(mut g) = DB_KEY_LOG.lock() {
        if g.len() >= 96 {
            g.remove(0);
        }
        g.push(entry);
    }
}
fn db_file_of(handle: usize) -> String {
    DB_HANDLES
        .lock()
        .ok()
        .and_then(|g| g.iter().find(|(h, _)| *h == handle).map(|(_, f)| f.clone()))
        .unwrap_or_else(|| "?".to_string())
}

/// ★ v3.24.46: read a C string at a raw address ONLY if it lies inside a
/// readable mapped region (mc_config varargs may or may not be pointers).
unsafe fn safe_read_cstr(addr: usize, max: usize) -> String {
    if addr < 0x10000 {
        return String::new();
    }
    if let Ok(maps) = std::fs::read_to_string("/proc/self/maps") {
        for line in maps.lines() {
            let mut parts = line.split_whitespace();
            let range = match parts.next() {
                Some(r) => r,
                None => continue,
            };
            let (a, b) = match range.split_once('-') {
                Some(x) => x,
                None => continue,
            };
            let sa = match usize::from_str_radix(a, 16) {
                Ok(v) => v,
                Err(_) => continue,
            };
            let ea = match usize::from_str_radix(b, 16) {
                Ok(v) => v,
                Err(_) => continue,
            };
            if addr >= sa && addr + max <= ea && line.contains('r') {
                let s = std::slice::from_raw_parts(addr as *const
```

### match 6

```rust
                }
            }
        }
        if SNIFF_ENABLED.load(Ordering::Relaxed) {
            if !bytes.is_empty() {
                let _lock = SNIFF_MUTEX.lock();
                let (rid, response_url) = if SNIFF_RESPONSE_QUEUE.is_empty() {
                    (0, String::new())
                } else {
                    SNIFF_RESPONSE_QUEUE.remove(0)
                };
                push_sniff_metadata(rid, "response", &response_url, bytes.len(), &bytes, Vec::new());
                SNIFF_RESPONSES.push((rid, bytes));
                if SNIFF_RESPONSES.len() > SNIFF_RAW_MAX {
                    SNIFF_RESPONSES.remove(0);
                }
            }
        }
        decompressed
    }
}

// ★ v3.23.3: Hook handler for WWWRequest.Post(this, url, postData, headers)
// Captures URL + headers directly, and matches the parked request body from CompressRequest.
// This replaces the old _Send + SetHeader approach.
extern "C" fn post_hook_handler(
    this: *mut c_void,
    url: *const c_void,
    post_data: *mut c_void,
    headers: *mut c_void,
) -> *mut c_void {
    unsafe {
        let trampoline = interceptor_get_trampoline(post_hook_handler as usize);
        if trampoline == 0 {
            return std::ptr::null_mut();
        }
        type FnType = unsafe extern "C" fn(
            *mut c_void,
            *const c_void,
            *mut c_void,
            *mut c_void,
        ) -> *mut c_void;
        let original: FnType = std::mem::transmute(trampoline);

        // Capture URL
        let game_url = if !url.is_null() {
            read_il2cpp_string(url)
        } else {
            String::new()
        };
        let game_url = if game_url.is_empty() {
            None
        } else {
            Some(game_url)
        };

        // Capture headers from Dictionary<string,string>
        let req_headers = read_string_dict(headers);

        if SNIFF_ENABLED.load(Ordering::Relaxed) {
            let rid = SNIFF_REQ_ID.fetch_add(1, Ordering::Relaxed);
            PENDING_REQ_ID = rid;
            let body = PENDING_REQ_BODY.take().unwrap_or_default();
            let headers_json = format_headers_json(&req_headers);
            let url_str = game_url.clone().unwrap_or_default();
            {
                let _lock = SNIFF_MUTEX.lock();
                push_sniff_metadata(rid, "request", &url_str, body.len(), &body, req_headers.clone());
                SNIFF_RESPONSE_QUEUE.push((rid, url_str.clone()));
                if SNIFF_RESPONSE_QUEUE.len() > SNIFF_METADATA_MAX {
                    SNIFF_RESPONSE_QUEUE.remove(0);
                }
                SNIFF_REQUESTS.push((rid, url_str, headers_json, body));
                if SNIFF_REQUESTS.len() > SNIFF_RAW_MAX {
                    SNIFF_REQUESTS.remove(0);
                }
            }
            PENDING_URL = game_url.clone().unwrap_or_default();
            PENDING_HEADERS = req_headers.clone();
        }

        let _ = this;
        original(this, url, post_data, headers)
    }
}

// ★ v3.23.3: Read IL2CPP Dictionary<string,string> into Vec<(String,String)>
// Layout: [hdr 0x10][fields...]; _entries @+0x18, _count @+0x20
// Entry: [hashCode:i32][next:i32][key:ptr][value:ptr] = 24B per entry
unsafe fn read_string_dict(dict: *mut c_void) -> Vec<(String, String)> {
    let mut out = Vec::new();
    if dict.is_null() {
        return out;
    }
    let count = std::ptr::read_unaligned::<i32>((dict as *const u8).add(0x20) as *const i32);
    if count <= 0 {
        return out;
    }
    let entries = std::ptr::read_unaligned::<usize>((dict as *const u8).add(0x18) as *const usize);
    if entries == 0 {
        return out;
    }
    // Il2CppArray header: 0x20 bytes, then entries
    let capacity =
        std::ptr::read_unaligned::<usize>((entries as *const u8).add(0x18) as *const usize);
    let entries_base = entries + 0x20;
    for i in 0..capacity {
        let entry_addr = entries_base + i * 24;
        let hash_code = std::ptr::read_unaligned::<i32>((entry_addr as *const u8) as *const i32);
        if hash_code < 0 {
            continue;
        } // free entry
        let key =
            std::ptr::read_unaligned::<usize>((entry_addr as *const u8).add(8) as *const usize);
        let value =
            std::ptr::read_unaligned::<usize>((entry_addr as *const u8).add(16) as *const usize);
        let key_str = read_il2cpp_string(key as *const c_void);
        let val_str = read_il2cpp_string(value as *const c_void);
        out.push((key_str, val_str));
        if out.len() >= count as usize {
            break;
        }
    }
    out
}

// Format headers Vec to JSON string: {"key1":"val1","key2":"val2"}
unsafe fn format_headers_json(headers: &[(String, String)]) -> String {
    if headers.is_empty() {
        return "{}".to_string();
    }
    let mut s = String::from("{");
    for (i, (k, v)) in headers.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        let v_escaped = v.replace('\\', "\\\\").replace('"', "\\\"");
        s.push_str(&format!("\"{}\":\"{}\"", k, v_escaped));
    }
    s.push('}');
    s
}

// ============================================================
// ★ v3.24.44: SQLCipher key capture (route B: offline meta decryption)
// The game's resource index `meta` is a SQLCipher-encrypted SQLite DB
// (no plain header; libnative.so exports sqlite3_key/sqlite3_key_v2).
// Hook the keying functions at plugin init (before the game opens meta),
// capture the key bytes, persist to the private files dir.
static META_KEY_HEX: Mutex<String> = Mutex::new(String::new());
static mut SQLCIPHER_KEY_HOOK_DONE: bool = false;

// ★ v3.24.45: pair db handle -> filename -> key + cipher config.
// (v3.24.44's "first key wins" caught the WRONG database's key.)
static DB_HANDLES: Mutex<Vec<(usize, String)>> = Mutex::new(Vec::new());
static DB_KEY_LOG: Mutex<Vec<String>> = Mutex::new(Vec::new());
fn db_track(entry: String) {
    hook_log(&entry);
    if let Ok(mut g) = DB_KEY_LOG.lock() {
        if g.len() >= 96 {
            g.remove(0);
        }
        g.push(entry);
    }
}
fn db_file_of(handle: usize) -> String {
    DB_HANDLES
        .lock()
        .ok()
        .and_then(|g| g.iter().find(|(h, _)| *h == handle).map(|(_, f)| f.clone()))
        .unwrap_or_else(|| "?".to_string())
}

/// ★ v3.24.46: read a C string at a raw address ONLY if it lies inside a
/// readable mapped region (mc_config varargs may or may not be pointers).
unsafe fn safe_read_cstr(addr: usize, max: usize) -> String {
    if addr < 0x10000 {
        return String::new();
    }
    if let Ok(maps) = std::fs::read_to_string("/proc/self/maps") {
        for line in maps.lines() {
            let mut parts = line.split_whitespace();
            let range = match parts.next() {
                Some(r) => r,
                None => continue,
            };
            let (a, b) = match range.split_once('-') {
                Some(x) => x,
                None => continue,
            };
            let sa = match usize::from_str_radix(a, 16) {
                Ok(v) => v,
                Err(_) => continue,
            };
            let ea = match usize::from_str_radix(b, 16) {
                Ok(v) => v,
                Err(_) => continue,
            };
            if addr >= sa && addr + max <= ea && line.contains('r') {
                let s = std::slice::from_raw_parts(addr as *const u8, max);
                let end = s.iter().position(|&c| c == 0).un
```

### match 7

```rust
d(Ordering::Relaxed) {
            if !bytes.is_empty() {
                let _lock = SNIFF_MUTEX.lock();
                let (rid, response_url) = if SNIFF_RESPONSE_QUEUE.is_empty() {
                    (0, String::new())
                } else {
                    SNIFF_RESPONSE_QUEUE.remove(0)
                };
                push_sniff_metadata(rid, "response", &response_url, bytes.len(), &bytes, Vec::new());
                SNIFF_RESPONSES.push((rid, bytes));
                if SNIFF_RESPONSES.len() > SNIFF_RAW_MAX {
                    SNIFF_RESPONSES.remove(0);
                }
            }
        }
        decompressed
    }
}

// ★ v3.23.3: Hook handler for WWWRequest.Post(this, url, postData, headers)
// Captures URL + headers directly, and matches the parked request body from CompressRequest.
// This replaces the old _Send + SetHeader approach.
extern "C" fn post_hook_handler(
    this: *mut c_void,
    url: *const c_void,
    post_data: *mut c_void,
    headers: *mut c_void,
) -> *mut c_void {
    unsafe {
        let trampoline = interceptor_get_trampoline(post_hook_handler as usize);
        if trampoline == 0 {
            return std::ptr::null_mut();
        }
        type FnType = unsafe extern "C" fn(
            *mut c_void,
            *const c_void,
            *mut c_void,
            *mut c_void,
        ) -> *mut c_void;
        let original: FnType = std::mem::transmute(trampoline);

        // Capture URL
        let game_url = if !url.is_null() {
            read_il2cpp_string(url)
        } else {
            String::new()
        };
        let game_url = if game_url.is_empty() {
            None
        } else {
            Some(game_url)
        };

        // Capture headers from Dictionary<string,string>
        let req_headers = read_string_dict(headers);

        if SNIFF_ENABLED.load(Ordering::Relaxed) {
            let rid = SNIFF_REQ_ID.fetch_add(1, Ordering::Relaxed);
            PENDING_REQ_ID = rid;
            let body = PENDING_REQ_BODY.take().unwrap_or_default();
            let headers_json = format_headers_json(&req_headers);
            let url_str = game_url.clone().unwrap_or_default();
            {
                let _lock = SNIFF_MUTEX.lock();
                push_sniff_metadata(rid, "request", &url_str, body.len(), &body, req_headers.clone());
                SNIFF_RESPONSE_QUEUE.push((rid, url_str.clone()));
                if SNIFF_RESPONSE_QUEUE.len() > SNIFF_METADATA_MAX {
                    SNIFF_RESPONSE_QUEUE.remove(0);
                }
                SNIFF_REQUESTS.push((rid, url_str, headers_json, body));
                if SNIFF_REQUESTS.len() > SNIFF_RAW_MAX {
                    SNIFF_REQUESTS.remove(0);
                }
            }
            PENDING_URL = game_url.clone().unwrap_or_default();
            PENDING_HEADERS = req_headers.clone();
        }

        let _ = this;
        original(this, url, post_data, headers)
    }
}

// ★ v3.23.3: Read IL2CPP Dictionary<string,string> into Vec<(String,String)>
// Layout: [hdr 0x10][fields...]; _entries @+0x18, _count @+0x20
// Entry: [hashCode:i32][next:i32][key:ptr][value:ptr] = 24B per entry
unsafe fn read_string_dict(dict: *mut c_void) -> Vec<(String, String)> {
    let mut out = Vec::new();
    if dict.is_null() {
        return out;
    }
    let count = std::ptr::read_unaligned::<i32>((dict as *const u8).add(0x20) as *const i32);
    if count <= 0 {
        return out;
    }
    let entries = std::ptr::read_unaligned::<usize>((dict as *const u8).add(0x18) as *const usize);
    if entries == 0 {
        return out;
    }
    // Il2CppArray header: 0x20 bytes, then entries
    let capacity =
        std::ptr::read_unaligned::<usize>((entries as *const u8).add(0x18) as *const usize);
    let entries_base = entries + 0x20;
    for i in 0..capacity {
        let entry_addr = entries_base + i * 24;
        let hash_code = std::ptr::read_unaligned::<i32>((entry_addr as *const u8) as *const i32);
        if hash_code < 0 {
            continue;
        } // free entry
        let key =
            std::ptr::read_unaligned::<usize>((entry_addr as *const u8).add(8) as *const usize);
        let value =
            std::ptr::read_unaligned::<usize>((entry_addr as *const u8).add(16) as *const usize);
        let key_str = read_il2cpp_string(key as *const c_void);
        let val_str = read_il2cpp_string(value as *const c_void);
        out.push((key_str, val_str));
        if out.len() >= count as usize {
            break;
        }
    }
    out
}

// Format headers Vec to JSON string: {"key1":"val1","key2":"val2"}
unsafe fn format_headers_json(headers: &[(String, String)]) -> String {
    if headers.is_empty() {
        return "{}".to_string();
    }
    let mut s = String::from("{");
    for (i, (k, v)) in headers.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        let v_escaped = v.replace('\\', "\\\\").replace('"', "\\\"");
        s.push_str(&format!("\"{}\":\"{}\"", k, v_escaped));
    }
    s.push('}');
    s
}

// ============================================================
// ★ v3.24.44: SQLCipher key capture (route B: offline meta decryption)
// The game's resource index `meta` is a SQLCipher-encrypted SQLite DB
// (no plain header; libnative.so exports sqlite3_key/sqlite3_key_v2).
// Hook the keying functions at plugin init (before the game opens meta),
// capture the key bytes, persist to the private files dir.
static META_KEY_HEX: Mutex<String> = Mutex::new(String::new());
static mut SQLCIPHER_KEY_HOOK_DONE: bool = false;

// ★ v3.24.45: pair db handle -> filename -> key + cipher config.
// (v3.24.44's "first key wins" caught the WRONG database's key.)
static DB_HANDLES: Mutex<Vec<(usize, String)>> = Mutex::new(Vec::new());
static DB_KEY_LOG: Mutex<Vec<String>> = Mutex::new(Vec::new());
fn db_track(entry: String) {
    hook_log(&entry);
    if let Ok(mut g) = DB_KEY_LOG.lock() {
        if g.len() >= 96 {
            g.remove(0);
        }
        g.push(entry);
    }
}
fn db_file_of(handle: usize) -> String {
    DB_HANDLES
        .lock()
        .ok()
        .and_then(|g| g.iter().find(|(h, _)| *h == handle).map(|(_, f)| f.clone()))
        .unwrap_or_else(|| "?".to_string())
}

/// ★ v3.24.46: read a C string at a raw address ONLY if it lies inside a
/// readable mapped region (mc_config varargs may or may not be pointers).
unsafe fn safe_read_cstr(addr: usize, max: usize) -> String {
    if addr < 0x10000 {
        return String::new();
    }
    if let Ok(maps) = std::fs::read_to_string("/proc/self/maps") {
        for line in maps.lines() {
            let mut parts = line.split_whitespace();
            let range = match parts.next() {
                Some(r) => r,
                None => continue,
            };
            let (a, b) = match range.split_once('-') {
                Some(x) => x,
                None => continue,
            };
            let sa = match usize::from_str_radix(a, 16) {
                Ok(v) => v,
                Err(_) => continue,
            };
            let ea = match usize::from_str_radix(b, 16) {
                Ok(v) => v,
                Err(_) => continue,
            };
            if addr >= sa && addr + max <= ea && line.contains('r') {
                let s = std::slice::from_raw_parts(addr as *const u8, max);
                let end = s.iter().position(|&c| c == 0).unwrap_or(max);
                if end == 0 {
                    return
```
