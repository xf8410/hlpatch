// ===== HTTP proxy endpoint v2 — curl subprocess transport =====
// CHANGE FROM v1: v1 referenced ureq::AgentBuilder, which caused rustls+ring
// to be linked into the cdylib for the first time (they were compiled but
// dead-stripped before). That is the prime suspect for the boot regression.
// v2 shells out to /system/bin/curl via the crate-root sys_system extern —
// the exact transport the base plugin already uses for GitHub uploads — so
// ZERO new native code is linked and load-time behavior matches the old SO.
//
// GET /proxy?url=<percent-encoded https URL>
//   curl fetches the URL; text bodies are returned verbatim, non-text bodies
//   are wrapped as a base64 JSON envelope. Never touches game memory.
// GET /proxy/status
//   Small JSON: active fetches, last fetch result, install marker.
//
// Safety model (do not widen without review):
// - https:// only (also enforced for redirects via --proto =https), port 443
//   only, userinfo rejected
// - loopback / RFC1918 / link-local / 0.0.0.0 / .local / .internal rejected
// - URL shell-safety: quotes, backticks, $, ;, |, \ and control chars rejected
//   (URL is single-quoted into the shell command)
// - body capped at 16 MiB via curl --max-filesize; connect 8 s; total 20 s
// - at most 1 concurrent fetch (fixed temp paths, AtomicU64 gate)
// - temp files live in the app private files dir (same dir the base plugin
//   already uses); removed after every request
// - route registered in BOOT_SAFE_EXACT: usable before game init

use std::ffi::CString;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

const FILES_DIR: &str = "/data/data/jp.pokemon.pokeuma/files";
const BODY_PATH: &str = "/data/data/jp.pokemon.pokeuma/files/uma_proxy_body.bin";
const META_PATH: &str = "/data/data/jp.pokemon.pokeuma/files/uma_proxy_meta.txt";
const MAX_BODY_BYTES: usize = 16 * 1024 * 1024;
const MAX_URL_BYTES: usize = 2048;
const MAX_ACTIVE_FETCHES: u64 = 1;

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
    // Shell-safety: the URL is embedded in a single-quoted curl argument.
    if url
        .chars()
        .any(|c| matches!(c, '\'' | '`' | '$' | ';' | '|' | '\\') || (c as u32) < 0x20)
    {
        return Err("url_shell_unsafe".to_string());
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

fn run_curl(url: &str) -> (i32, String) {
    let command = format!(
        "curl -sL --connect-timeout 8 --max-time 20 --max-redirs 3 --max-filesize {} \
         --proto =https -A 'hlpatch-http-proxy/1.0' -o '{}' -w '%{{http_code}} %{{content_type}}' \
         '{}' > '{}' 2>/dev/null",
        MAX_BODY_BYTES, BODY_PATH, url, META_PATH
    );
    let exit_code = match CString::new(command) {
        Ok(command_c) => unsafe { super::sys_system(command_c.as_ptr() as *const i8) },
        Err(_) => -1,
    };
    let meta = std::fs::read_to_string(META_PATH).unwrap_or_default();
    (exit_code, meta)
}

fn fetch(url: &str) -> Result<FetchOutcome, String> {
    let (exit_code, meta) = run_curl(url);
    let mut parts = meta.split_whitespace();
    let status: u16 = parts
        .next()
        .and_then(|token| token.parse().ok())
        .unwrap_or(0);
    let content_type = parts.next().unwrap_or("").to_ascii_lowercase();
    if status == 0 {
        // curl absent (127), DNS/connect failure, timeout (28), or blocked write.
        return Err(format!(
            "curl_unavailable_or_failed (exit {})",
            exit_code
        ));
    }
    let body = match std::fs::read(BODY_PATH) {
        Ok(body) => body,
        Err(_) => Vec::new(),
    };
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
    // Always clean the temp files, success or failure.
    let _ = std::fs::remove_file(BODY_PATH);
    let _ = std::fs::remove_file(META_PATH);
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
        r#"{{"ok":true,"feature":"http_proxy","transport":"curl_subprocess","active_fetches":{},"max_active_fetches":{},"last":{}}}"#,
        active, MAX_ACTIVE_FETCHES, last
    )
}
