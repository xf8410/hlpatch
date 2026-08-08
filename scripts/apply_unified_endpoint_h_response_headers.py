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
'''// UnityWebRequest request-entry observer. Full capture: headers, bodies, tokens and query strings.
static mut UNITY_SEND_ADDR: usize = 0;
''',
'''// UnityWebRequest request entry and AsyncOperation completion observers.
static mut UNITY_SEND_ADDR: usize = 0;
static mut UNITY_COMPLETE_ADDR: usize = 0;
''', "completion_address")

replace_once(
'''static UNITY_OBSERVATIONS: Mutex<Vec<UnityRequestObservation>> = Mutex::new(Vec::new());
// Pending request body parking (CompressRequest → Post matching)
''',
'''static UNITY_OBSERVATIONS: Mutex<Vec<UnityRequestObservation>> = Mutex::new(Vec::new());
// Completed response headers are keyed by the full request URL. Capture occurs
// immediately before Unity dispatches the completion callback that reaches the
// game's DecompressResponse path.
static UNITY_COMPLETED_RESPONSE_HEADERS: Mutex<Vec<(String, Vec<(String, String)>)>> = Mutex::new(Vec::new());

unsafe fn observe_unity_response_completion(operation: *mut c_void) {
    if operation.is_null() || !SNIFF_ENABLED.load(Ordering::Relaxed) { return; }
    let operation_class = get_class_from_object(operation);
    if operation_class.is_null() { return; }
    let request = call_getter_ref(operation_class, operation, "get_webRequest");
    if request.is_null() { return; }
    let url = unity_get_string(request, "get_url");
    if url.is_empty() || !url.contains("/umamusume/") { return; }
    let request_class = get_class_from_object(request);
    if request_class.is_null() { return; }
    let dictionary = call_getter_on_instance(request_class, request, "GetResponseHeaders");
    if dictionary.is_null() { return; }
    let headers = read_string_dict(dictionary);
    if let Ok(mut completed) = UNITY_COMPLETED_RESPONSE_HEADERS.lock() {
        completed.push((url, headers));
    }
}

unsafe fn take_unity_response_headers(url: &str) -> Option<Vec<(String, String)>> {
    UNITY_COMPLETED_RESPONSE_HEADERS.lock().ok().and_then(|mut completed| {
        let wanted = sniff_path(url);
        let index = completed.iter().position(|(candidate, _)| sniff_path(candidate) == wanted)?;
        Some(completed.remove(index).1)
    })
}
// Pending request body parking (CompressRequest → Post matching)
''', "completed_headers_queue")

replace_once(
'''// MakeMd5(string input) -> string
''',
'''// AsyncOperation.InvokeCompletionEvent runs after response headers are available
// and immediately before Unity invokes the request completion callbacks.
extern "C" fn unity_complete_hook_handler(this: *mut c_void) {
    unsafe {
        let trampoline = interceptor_get_trampoline(unity_complete_hook_handler as usize);
        if trampoline == 0 { return; }
        type FnType = unsafe extern "C" fn(*mut c_void);
        let original: FnType = std::mem::transmute(trampoline);
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            observe_unity_response_completion(this);
        }));
        original(this);
    }
}

// MakeMd5(string input) -> string
''', "completion_handler")

replace_once(
'''                push_sniff_metadata(rid, "response", &response_url, bytes.len(), &bytes, Vec::new());
                if let Err(error) = persist_protocol_capture("response", rid, &response_url, &[], &bytes) { storage_set_error(&error); }
''',
'''                let response_headers = take_unity_response_headers(&response_url);
                let response_headers_json = response_headers.as_ref().map(|headers| format_headers_json(headers));
                match (response_headers, response_headers_json) {
                    (Some(headers), Some(headers_json)) => {
                        push_sniff_metadata(rid, "response", &response_url, bytes.len(), &bytes, headers);
                        if let Err(error) = persist_protocol_capture("response", rid, &response_url, headers_json.as_bytes(), &bytes) { storage_set_error(&error); }
                    }
                    _ => {
                        storage_set_error("response_headers_not_correlated");
                        push_sniff_metadata(rid, "response", &response_url, bytes.len(), &bytes, Vec::new());
                        if let Err(error) = persist_protocol_capture("response", rid, &response_url, &[], &bytes) { storage_set_error(&error); }
                    }
                }
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
            if let Ok(mut completed) = UNITY_COMPLETED_RESPONSE_HEADERS.lock() {
                completed.clear();
            }
            SNIFF_METADATA.clear();
''', "clear_completed_headers")

replace_once(
'''    let all_hooked = COMPRESS_REQUEST_ADDR != 0
        && DECOMPRESS_RESPONSE_ADDR != 0
        && POST_ADDR != 0
        && UNITY_SEND_ADDR != 0;
''',
'''    let all_hooked = COMPRESS_REQUEST_ADDR != 0
        && DECOMPRESS_RESPONSE_ADDR != 0
        && POST_ADDR != 0
        && UNITY_SEND_ADDR != 0
        && UNITY_COMPLETE_ADDR != 0;
''', "all_hooks_gate")

replace_once(
'''    // Hook Cryptographer.MakeMd5 to capture salt
''',
'''    if UNITY_COMPLETE_ADDR == 0 {
        let core_image = get_asm(to_cstr("UnityEngine.CoreModule.dll").as_ptr());
        if core_image.is_null() {
            set_hook_status("sniff.unity_complete", "failed: image_not_found");
        } else {
            let async_operation = get_class(
                core_image,
                to_cstr("UnityEngine").as_ptr(),
                to_cstr("AsyncOperation").as_ptr(),
            );
            if async_operation.is_null() {
                set_hook_status("sniff.unity_complete", "failed: class_not_found");
            } else {
                let addr = get_method_addr(async_operation as usize, to_cstr("InvokeCompletionEvent").as_ptr(), 0);
                if addr == 0 {
                    set_hook_status("sniff.unity_complete", "failed: method_not_found");
                } else if interceptor_hook(addr, unity_complete_hook_handler as usize) {
                    UNITY_COMPLETE_ADDR = addr;
                    set_hook_status("sniff.unity_complete", &format!("hooked@0x{:x}", addr));
                } else {
                    set_hook_status("sniff.unity_complete", "failed: interceptor_hook");
                }
            }
        }
    }

    // Hook Cryptographer.MakeMd5 to capture salt
''', "install_completion_hook")

anchor = "/// 辅助函数：IL2CPP类型枚举转可读名称\n"
replace_once(anchor, MARKER + "\n" + anchor, "h_marker")
SOURCE.write_text(s, encoding="utf-8")
print("unified_endpoint_h_response_headers=applied")
