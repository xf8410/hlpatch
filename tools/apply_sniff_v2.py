from pathlib import Path
import re

p = Path("hachimi_ura_plugin/src/lib.rs")
s = p.read_text()


def replace(old: str, new: str) -> None:
    global s
    count = s.count(old)
    if count != 1:
        raise SystemExit(f"expected one literal match, got {count}: {old[:100]!r}")
    s = s.replace(old, new, 1)


replace(
    """// SniffEntry: (id, url, headers_json, body)
static mut SNIFF_REQUESTS: Vec<(u64, String, String, Vec<u8>)> = Vec::new();
static mut SNIFF_RESPONSES: Vec<(u64, Vec<u8>)> = Vec::new();
static SNIFF_MAX: usize = 20;
static SNIFF_REQ_ID: AtomicU64 = AtomicU64::new(0);""",
    """// Raw payloads and safe metadata use separate bounded rings.
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
}
static mut SNIFF_METADATA: Vec<SniffMetadata> = Vec::new();
// Bounded temporal FIFO; unmatched responses are reported with request_id=0.
static mut SNIFF_RESPONSE_QUEUE: Vec<(u64, String)> = Vec::new();""",
)

replace(
    """static mut PENDING_COMPRESSED: usize = 0;
// ★ Mutex to prevent concurrent read_summary_inner""",
    """static mut PENDING_COMPRESSED: usize = 0;

fn sniff_timestamp_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis().min(u128::from(u64::MAX)) as u64)
        .unwrap_or(0)
}

fn sniff_path(url: &str) -> String {
    let no_query = url.split('?').next().unwrap_or(url);
    if let Some(i) = no_query.find(\"://\") {
        let rest = &no_query[i + 3..];
        return rest
            .find('/')
            .map(|j| rest[j..].to_string())
            .unwrap_or_else(|| \"/\".to_string());
    }
    no_query.to_string()
}

unsafe fn push_sniff_metadata(request_id: u64, direction: &'static str, url: &str, size: usize) {
    let id = SNIFF_METADATA_ID.fetch_add(1, Ordering::Relaxed);
    SNIFF_METADATA.push(SniffMetadata {
        id,
        request_id,
        timestamp_ms: sniff_timestamp_ms(),
        direction,
        path: sniff_path(url),
        size,
    });
    if SNIFF_METADATA.len() > SNIFF_METADATA_MAX {
        SNIFF_METADATA.remove(0);
    }
}
// ★ Mutex to prevent concurrent read_summary_inner""",
)

s = s.replace(
    '"/api/sniff", "/api/sniff/diag", "/api/sniff/toggle", "/api/sniff/clear",',
    '"/api/sniff", "/api/sniff/metadata", "/api/sniff/status", "/api/sniff/diag", "/api/sniff/toggle", "/api/sniff/clear",',
)
s = s.replace(
    '"/api/sniff", "/api/sniff/diag", "/api/event/choices"',
    '"/api/sniff", "/api/sniff/metadata", "/api/sniff/diag", "/api/event/choices"',
)
s = s.replace(
    '"/api/sniff","/api/sniff/toggle","/api/sniff/clear","/api/sniff/diag"',
    '"/api/sniff","/api/sniff/metadata","/api/sniff/status","/api/sniff/toggle","/api/sniff/clear","/api/sniff/diag"',
)

marker = '    } else if path == "/api/sniff/toggle" {'
routes = '''    } else if path == "/api/sniff/status" {
        let _lock = SNIFF_MUTEX.lock();
        unsafe {
            let last_id = SNIFF_METADATA.last().map(|m| m.id).unwrap_or(0);
            let request_count = SNIFF_METADATA.iter().filter(|m| m.direction == "request").count();
            let response_count = SNIFF_METADATA.iter().filter(|m| m.direction == "response").count();
            format!(r#"{{"enabled":{},"raw_request_count":{},"raw_response_count":{},"metadata_count":{},"request_count":{},"response_count":{},"last_id":{},"raw_limit":{},"metadata_limit":{}}}"#,
                SNIFF_ENABLED.load(Ordering::Relaxed), SNIFF_REQUESTS.len(), SNIFF_RESPONSES.len(),
                SNIFF_METADATA.len(), request_count, response_count, last_id, SNIFF_RAW_MAX, SNIFF_METADATA_MAX)
        }
    } else if path == "/api/sniff/metadata" {
        let after_id = parse_query(&full_uri, "after_id").parse::<u64>().unwrap_or(0);
        let _lock = SNIFF_MUTEX.lock();
        unsafe {
            let entries: Vec<String> = SNIFF_METADATA.iter()
                .filter(|m| m.id > after_id)
                .map(|m| format!(r#"{{"id":{},"request_id":{},"timestamp_ms":{},"direction":"{}","path":"{}","size":{}}}"#,
                    m.id, m.request_id, m.timestamp_ms, m.direction, json_escape(&m.path), m.size))
                .collect();
            let last_id = SNIFF_METADATA.last().map(|m| m.id).unwrap_or(after_id);
            format!(r#"{{"enabled":{},"after_id":{},"last_id":{},"count":{},"entries":[{}]}}"#,
                SNIFF_ENABLED.load(Ordering::Relaxed), after_id, last_id, entries.len(), entries.join(","))
        }
'''
replace(marker, routes + marker)

replace(
    """        let new_val = !SNIFF_ENABLED.load(Ordering::Relaxed);
        SNIFF_ENABLED.store(new_val, Ordering::Relaxed);""",
    """        let requested = parse_query(&full_uri, "enabled");
        let new_val = match requested.as_str() {
            "1" | "true" => true,
            "0" | "false" => false,
            _ => !SNIFF_ENABLED.load(Ordering::Relaxed),
        };
        SNIFF_ENABLED.store(new_val, Ordering::Relaxed);""",
)

replace(
    """            SNIFF_REQUESTS.clear();
            SNIFF_RESPONSES.clear();""",
    """            SNIFF_REQUESTS.clear();
            SNIFF_RESPONSES.clear();
            SNIFF_METADATA.clear();
            SNIFF_RESPONSE_QUEUE.clear();
            PENDING_REQ_BODY = None;""",
)

replace(
    """                let rid = PENDING_REQ_ID;
                SNIFF_RESPONSES.push((rid, bytes));
                if SNIFF_RESPONSES.len() > SNIFF_MAX {
                    SNIFF_RESPONSES.remove(0);
                }""",
    """                let (rid, response_url) = if SNIFF_RESPONSE_QUEUE.is_empty() {
                    (0, String::new())
                } else {
                    SNIFF_RESPONSE_QUEUE.remove(0)
                };
                push_sniff_metadata(rid, "response", &response_url, bytes.len());
                SNIFF_RESPONSES.push((rid, bytes));
                if SNIFF_RESPONSES.len() > SNIFF_RAW_MAX {
                    SNIFF_RESPONSES.remove(0);
                }""",
)

replace(
    """            // Try to match parked request body
            if let Some(body) = PENDING_REQ_BODY.take() {
                let headers_json = format_headers_json(&req_headers);
                let url_str = game_url.clone().unwrap_or_default();
                let _lock = SNIFF_MUTEX.lock();
                SNIFF_REQUESTS.push((rid, url_str, headers_json, body));
                if SNIFF_REQUESTS.len() > SNIFF_MAX {
                    SNIFF_REQUESTS.remove(0);
                }
            }
            PENDING_URL = game_url.clone().unwrap_or_default();""",
    """            let body = PENDING_REQ_BODY.take().unwrap_or_default();
            let headers_json = format_headers_json(&req_headers);
            let url_str = game_url.clone().unwrap_or_default();
            {
                let _lock = SNIFF_MUTEX.lock();
                push_sniff_metadata(rid, "request", &url_str, body.len());
                SNIFF_RESPONSE_QUEUE.push((rid, url_str.clone()));
                if SNIFF_RESPONSE_QUEUE.len() > SNIFF_METADATA_MAX {
                    SNIFF_RESPONSE_QUEUE.remove(0);
                }
                SNIFF_REQUESTS.push((rid, url_str, headers_json, body));
                if SNIFF_REQUESTS.len() > SNIFF_RAW_MAX {
                    SNIFF_REQUESTS.remove(0);
                }
            }
            PENDING_URL = game_url.clone().unwrap_or_default();""",
)

if "SNIFF_MAX" in s:
    raise SystemExit("obsolete SNIFF_MAX remains")
p.write_text(s)

cargo = Path("hachimi_ura_plugin/Cargo.toml")
c = cargo.read_text()
c, count = re.subn(r'(?m)^version = "[^"]+"', 'version = "3.25.0"', c, count=1)
if count != 1:
    raise SystemExit("Cargo version marker mismatch")
cargo.write_text(c)
