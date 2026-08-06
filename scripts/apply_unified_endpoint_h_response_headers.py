from pathlib import Path

SOURCE = Path("hachimi_ura_plugin/src/lib.rs")
s = SOURCE.read_text(encoding="utf-8")
MARKER = "// ===== Unified response-header capture H-stage ====="
if MARKER in s:
    print("unified_endpoint_h_response_headers=already_applied")
    raise SystemExit(0)


def replace_once(old: str, new: str, label: str) -> None:
    global s
    count = s.count(old)
    assert count == 1, f"{label} anchor count={count}"
    s = s.replace(old, new, 1)


replace_once(
'''static UNITY_OBSERVATIONS: Mutex<Vec<UnityRequestObservation>> = Mutex::new(Vec::new());
// Pending request body parking (CompressRequest → Post matching)
''',
'''static UNITY_OBSERVATIONS: Mutex<Vec<UnityRequestObservation>> = Mutex::new(Vec::new());
// Keep a strong IL2CPP GC handle until the matching game API response is
// decompressed. URL matching prevents unrelated/concurrent Unity requests from
// shifting response headers onto the wrong request id.
static UNITY_RESPONSE_REQUESTS: Mutex<Vec<(u32, String)>> = Mutex::new(Vec::new());

unsafe fn unity_response_retain(request: *mut c_void) -> u32 {
    let symbol = resolve_il2cpp_symbol("il2cpp_gchandle_new");
    if symbol.is_null() || request.is_null() { return 0; }
    let create: unsafe extern "C" fn(*mut c_void, bool) -> u32 = std::mem::transmute(symbol);
    create(request, false)
}

unsafe fn unity_response_target(handle: u32) -> *mut c_void {
    let symbol = resolve_il2cpp_symbol("il2cpp_gchandle_get_target");
    if symbol.is_null() || handle == 0 { return ptr::null_mut(); }
    let get: unsafe extern "C" fn(u32) -> *mut c_void = std::mem::transmute(symbol);
    get(handle)
}

unsafe fn unity_response_release(handle: u32) {
    let symbol = resolve_il2cpp_symbol("il2cpp_gchandle_free");
    if symbol.is_null() || handle == 0 { return; }
    let free: unsafe extern "C" fn(u32) = std::mem::transmute(symbol);
    free(handle);
}

unsafe fn take_unity_response_headers(url: &str) -> Vec<(String, String)> {
    let selected = UNITY_RESPONSE_REQUESTS.lock().ok().and_then(|mut pending| {
        let wanted = sniff_path(url);
        let index = pending.iter().position(|(_, candidate)| sniff_path(candidate) == wanted)?;
        Some(pending.remove(index).0)
    });
    let Some(handle) = selected else { return Vec::new(); };
    let request = unity_response_target(handle);
    let headers = if request.is_null() {
        Vec::new()
    } else {
        let class = get_class_from_object(request);
        if class.is_null() {
            Vec::new()
        } else {
            let dictionary = call_getter_on_instance(class, request, "GetResponseHeaders");
            if dictionary.is_null() { Vec::new() } else { read_string_dict(dictionary) }
        }
    };
    unity_response_release(handle);
    headers
}
// Pending request body parking (CompressRequest → Post matching)
''', "unity_response_queue")

replace_once(
'''    let item = UnityRequestObservation {
        id: UNITY_OBSERVATION_ID.fetch_add(1, Ordering::Relaxed),
''',
'''    if method.eq_ignore_ascii_case("POST") && url.contains("/umamusume/") {
        let handle = unity_response_retain(request);
        if handle != 0 {
            if let Ok(mut pending) = UNITY_RESPONSE_REQUESTS.lock() {
                pending.push((handle, url.clone()));
                if pending.len() > SNIFF_RAW_MAX {
                    let (expired, _) = pending.remove(0);
                    unity_response_release(expired);
                }
            } else {
                unity_response_release(handle);
            }
        }
    }
    let item = UnityRequestObservation {
        id: UNITY_OBSERVATION_ID.fetch_add(1, Ordering::Relaxed),
''', "queue_unity_request")

replace_once(
'''                push_sniff_metadata(rid, "response", &response_url, bytes.len(), &bytes, Vec::new());
                if let Err(error) = persist_protocol_capture("response", rid, &response_url, &[], &bytes) { storage_set_error(&error); }
''',
'''                let response_headers = take_unity_response_headers(&response_url);
                let response_headers_json = format_headers_json(&response_headers);
                push_sniff_metadata(rid, "response", &response_url, bytes.len(), &bytes, response_headers);
                if let Err(error) = persist_protocol_capture("response", rid, &response_url, response_headers_json.as_bytes(), &bytes) { storage_set_error(&error); }
''', "response_headers_capture_and_persist")

replace_once(
'''            if let Ok(mut entries) = UNITY_OBSERVATIONS.lock() {
                entries.clear();
            }
            SNIFF_METADATA.clear();
''',
'''            if let Ok(mut entries) = UNITY_OBSERVATIONS.lock() {
                entries.clear();
            }
            if let Ok(mut pending) = UNITY_RESPONSE_REQUESTS.lock() {
                for (handle, _) in pending.drain(..) { unity_response_release(handle); }
            }
            SNIFF_METADATA.clear();
''', "clear_response_queue")

replace_once(
'''response_headers_capture=unavailable_in_current_hook''',
'''response_headers_capture=unity_web_request_get_response_headers_at_decompress''',
"manifest_capability")

anchor = "/// 辅助函数：IL2CPP类型枚举转可读名称\n"
replace_once(anchor, MARKER + "\n" + anchor, "h_marker")
SOURCE.write_text(s, encoding="utf-8")
print("unified_endpoint_h_response_headers=applied")
