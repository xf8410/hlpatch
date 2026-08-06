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
// UnityWebRequest objects remain owned by the live async operation until response
// decompression. Queue only game API POSTs, in the same bounded FIFO model used
// for request-id correlation, so DecompressResponse can call GetResponseHeaders.
static UNITY_RESPONSE_REQUESTS: Mutex<Vec<usize>> = Mutex::new(Vec::new());
// Pending request body parking (CompressRequest → Post matching)
''', "unity_response_queue")

replace_once(
'''    let item = UnityRequestObservation {
        id: UNITY_OBSERVATION_ID.fetch_add(1, Ordering::Relaxed),
''',
'''    if method.eq_ignore_ascii_case("POST") && url.contains("/umamusume/") {
        if let Ok(mut pending) = UNITY_RESPONSE_REQUESTS.lock() {
            pending.push(request as usize);
            if pending.len() > SNIFF_RAW_MAX {
                pending.remove(0);
            }
        }
    }
    let item = UnityRequestObservation {
        id: UNITY_OBSERVATION_ID.fetch_add(1, Ordering::Relaxed),
''', "queue_unity_request")

replace_once(
'''                push_sniff_metadata(rid, "response", &response_url, bytes.len(), &bytes, Vec::new());
''',
'''                let response_headers = UNITY_RESPONSE_REQUESTS.lock().ok()
                    .and_then(|mut pending| if pending.is_empty() { None } else { Some(pending.remove(0)) })
                    .map(|request| {
                        let request = request as *mut c_void;
                        if request.is_null() { return Vec::new(); }
                        let class = get_class_from_object(request);
                        if class.is_null() { return Vec::new(); }
                        let dictionary = call_getter_on_instance(class, request, "GetResponseHeaders");
                        if dictionary.is_null() { Vec::new() } else { read_string_dict(dictionary) }
                    }).unwrap_or_default();
                push_sniff_metadata(rid, "response", &response_url, bytes.len(), &bytes, response_headers);
''', "response_headers_capture")

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
                pending.clear();
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
