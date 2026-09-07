// ===== HTTP proxy endpoint (workbench/http-proxy-endpoint-20260907) =====
// GET /proxy?url=<percent-encoded https URL>
//   Fetches the URL via ureq (rustls TLS with bundled Mozilla roots — no
//   dependency on the Android system trust store) and returns the body
//   verbatim for text content types. Non-text bodies are wrapped as a
//   base64 JSON envelope. Never touches game memory or il2cpp state.
// GET /proxy/status
//   Small JSON: active fetches, last fetch result, install marker.
//
// Safety model (do not widen without review):
// - https:// only, default port 443 only, userinfo rejected, redirects <= 3
// - loopback / RFC1918 / link-local / 0.0.0.0 / .local / .internal rejected
// - response capped at 16 MiB; connect timeout 8 s; total timeout 20 s
// - at most 2 concurrent fetches (AtomicU64 gate)
// - route registered in BOOT_SAFE_EXACT: usable before game init

use std::io::Read;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

const CONNECT_TIMEOUT_SECS: u64 = 8;
const TOTAL_TIMEOUT_SECS: u64 = 20;
const MAX_BODY_BYTES: usize = 16 * 1024 * 1024;
const MAX_URL_BYTES: usize = 2048;
const MAX_ACTIVE_FETCHES: u64 = 2;

static ACTIVE_FETCHES: AtomicU64 = AtomicU64::new(0);

struct LastFetch {
    ok: bool,
    url: String,
    status: u16,
    bytes: u64,
    duration_ms: u64,
    detail: String,
}

static LAST_FETCH: Mutex<Option<LastFetch>> = Mutex::new(None);

fn record_last(ok: bool, url: &str, status: u16, bytes: u64, duration_ms: u64, detail: &str) {
    let entry = LastFetch {
        ok,
        url: url.to_string(),
        status,
        bytes,
        duration_ms,
        detail: detail.to_string(),
    };
    if let Ok(mut guard) = LAST_FETCH.lock() {
        *guard = Some(entry);
    }
}

fn validate_url(url: &str) -> Result<String, String> {
    if !url.to_ascii_lowercase().starts_with("https://") {
        return Err("https_only".to_string());
    }
    if url.len() > MAX_URL_BYTES {
        return Err("url_too_long".to_string());
    }
    let rest = &url["https://".len()..];
    let authority_end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let authority = &rest[..authority_end];
    if authority.is_empty() {
        return Err("missing_host".to_string());
    }
    if authority.contains('@') {
        return Err("userinfo_blocked".to_string());
    }
    let (host, port) = match authority.rsplit_once(':') {
        Some((host, port)) => {
            let port_num: u16 = port.parse().map_err(|_| "bad_port".to_string())?;
            (host, port_num)
        }
        None => (authority, 443u16),
    };
    if port != 443 {
        return Err("port_443_only".to_string());
    }
    let host = host
        .trim_start_matches('[')
        .trim_end_matches(']')
        .to_ascii_lowercase();
    if host.is_empty() {
        return Err("missing_host".to_string());
    }
    let blocked = matches!(host.as_str(), "localhost" | "::1" | "0.0.0.0")
        || host.starts_with("127.")
        || host.starts_with("10.")
        || host.starts_with("192.168.")
        || host.starts_with("169.254.")
        || host.starts_with("0.")
        || host.ends_with(".local")
        || host.ends_with(".internal")
        || host.ends_with(".localhost");
    if blocked {
        return Err("private_address_blocked".to_string());
    }
    if let Some(second_octet) = host
        .strip_prefix("172.")
        .and_then(|tail| tail.split('.').next())
    {
        if let Ok(value) = second_octet.parse::<u32>() {
            if (16..=31).contains(&value) {
                return Err("private_address_blocked".to_string());
            }
        }
    }
    Ok(host)
}

struct FetchOutcome {
    status: u16,
    content_type: String,
    body: Vec<u8>,
}

fn fetch(url: &str) -> Result<FetchOutcome, String> {
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(CONNECT_TIMEOUT_SECS))
        .timeout(Duration::from_secs(TOTAL_TIMEOUT_SECS))
        .redirects(3)
        .user_agent("hlpatch-http-proxy/1.0 (local diagnostic endpoint)")
        .build();
    let response = match agent.get(url).call() {
        Ok(response) => response,
        // Non-2xx responses still carry a readable body: pass them through.
        Err(ureq::Error::Status(_status, response)) => response,
        Err(ureq::Error::Transport(transport)) => {
            return Err(format!("transport: {}", transport));
        }
    };
    let status = response.status();
    let content_type = response
        .header("content-type")
        .unwrap_or("")
        .split(';')
        .next()
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase();
    let mut reader = response.into_reader();
    let mut body: Vec<u8> = Vec::new();
    let mut chunk = [0u8; 16 * 1024];
    loop {
        match reader.read(&mut chunk) {
            Ok(0) => break,
            Ok(n) => {
                if body.len() + n > MAX_BODY_BYTES {
                    return Err("body_too_large".to_string());
                }
                body.extend_from_slice(&chunk[..n]);
            }
            Err(error) => return Err(format!("read_failed: {}", error)),
        }
    }
    Ok(FetchOutcome {
        status,
        content_type,
        body,
    })
}

pub fn endpoint(full_uri: &str) -> String {
    if ACTIVE_FETCHES.load(Ordering::Acquire) >= MAX_ACTIVE_FETCHES {
        return super::k_json_error("proxy_busy");
    }
    let pairs = match super::parse_query_pairs(full_uri) {
        Ok(value) => value,
        Err(error) => return super::k_json_error(&error),
    };
    let url = super::query_pair(&pairs, "url");
    if url.is_empty() {
        return super::k_json_error("missing_url");
    }
    let host = match validate_url(&url) {
        Ok(value) => value,
        Err(error) => {
            super::hook_log(&format!("proxy: rejected {} ({})", url, error));
            return super::k_json_error(&error);
        }
    };
    ACTIVE_FETCHES.fetch_add(1, Ordering::AcqRel);
    let started = Instant::now();
    let outcome = fetch(&url);
    let duration_ms = started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64;
    ACTIVE_FETCHES.fetch_sub(1, Ordering::AcqRel);
    match outcome {
        Ok(data) => {
            let bytes = data.body.len() as u64;
            let is_text = data.content_type.starts_with("text/")
                || data.content_type.contains("json")
                || data.content_type.contains("xml")
                || data.content_type.contains("html")
                || data.content_type.contains("javascript");
            record_last(true, &url, data.status, bytes, duration_ms, &data.content_type);
            super::hook_log(&format!(
                "proxy: {} status={} bytes={} type={} in {}ms",
                host, data.status, bytes, data.content_type, duration_ms
            ));
            if is_text {
                String::from_utf8_lossy(&data.body).into_owned()
            } else {
                format!(
                    r#"{{"proxy_binary":true,"url":"{}","status":{},"content_type":"{}","bytes":{},"data_base64":"{}"}}"#,
                    super::json_escape(&url),
                    data.status,
                    super::json_escape(&data.content_type),
                    bytes,
                    super::base64_encode(&data.body)
                )
            }
        }
        Err(error) => {
            record_last(false, &url, 0, 0, duration_ms, &error);
            super::hook_log(&format!(
                "proxy: failed {} in {}ms ({})",
                url, duration_ms, error
            ));
            super::k_json_error(&format!("proxy_fetch_failed: {}", error))
        }
    }
}

pub fn status_endpoint() -> String {
    let active = ACTIVE_FETCHES.load(Ordering::Acquire);
    let last = match LAST_FETCH.lock() {
        Ok(guard) => guard
            .as_ref()
            .map(|entry| {
                format!(
                    r#"{{"ok":{},"url":"{}","status":{},"bytes":{},"duration_ms":{},"detail":"{}"}}"#,
                    entry.ok,
                    super::json_escape(&entry.url),
                    entry.status,
                    entry.bytes,
                    entry.duration_ms,
                    super::json_escape(&entry.detail)
                )
            })
            .unwrap_or_else(|| "null".to_string()),
        Err(_) => "null".to_string(),
    };
    format!(
        r#"{{"ok":true,"feature":"http_proxy","active_fetches":{},"max_active_fetches":{},"last":{}}}"#,
        active, MAX_ACTIVE_FETCHES, last
    )
}
