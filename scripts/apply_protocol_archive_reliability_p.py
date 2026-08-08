from pathlib import Path
import re

SOURCE = Path("hachimi_ura_plugin/src/lib.rs")
s = SOURCE.read_text(encoding="utf-8")
MARKER = "// ===== Protocol archive reliability P-stage ====="
if MARKER in s:
    print("protocol_archive_reliability=already_applied")
    raise SystemExit(0)


def replace_once(old: str, new: str, label: str) -> None:
    global s
    count = s.count(old)
    assert count == 1, f"{label} anchor count={count}"
    s = s.replace(old, new, 1)

# 持久存储和历史协议导出只访问插件自身磁盘，不依赖游戏IL2CPP初始化。
def add_boot_safe(name: str, values: list[str]) -> None:
    global s
    pattern = re.compile(rf"((?:static|const)\s+{name}\b[^=]*=\s*&\[)", re.M)
    match = pattern.search(s)
    assert match is not None, f"{name} declaration missing"
    insertion = "".join(f'\n    "{value}",' for value in values if f'"{value}"' not in s[match.start():s.find("];", match.end())])
    if insertion:
        s = s[:match.end()] + insertion + s[match.end():]

add_boot_safe("BOOT_SAFE_EXACT", [
    "/storage/status", "/storage/sessions", "/storage/session", "/storage/files",
    "/storage/download", "/storage/flush", "/storage/recover", "/storage/audit",
    "/api/sniff/exchange", "/api/sniff/exchanges",
])

# 启动日志版本必须来自唯一编译版本常量，不能继续嵌入历史版本字符串。
startup_pattern = re.compile(
    r'ura_log\(([^,\n]+),\s*"URA plugin v3\.24\.9 loaded \(Interceptor API hooks\)"\);'
)
startup_matches = list(startup_pattern.finditer(s))
assert len(startup_matches) == 1, f"startup version log matches={len(startup_matches)}"
s = startup_pattern.sub(
    r'ura_log(\1, &format!("URA plugin {} loaded (Interceptor API hooks)", PLUGIN_VERSION));',
    s,
    count=1,
)
assert re.search(r'"URA plugin v\d+\.\d+\.\d+ loaded', s) is None

anchor = "fn k_observation_files(domain: &str, uri: &str) -> String {\n"
assert s.count(anchor) == 1
rust = r'''// ===== Protocol archive reliability P-stage =====
#[derive(Default)]
struct ProtocolArchiveAudit {
    request_ids: std::collections::BTreeSet<u64>,
    response_ids: std::collections::BTreeSet<u64>,
    request_files: usize,
    response_files: usize,
    request_bytes: u64,
    response_bytes: u64,
    zero_length_files: Vec<String>,
    indexed_length_mismatches: Vec<String>,
}

fn protocol_path_request_id(relative_path: &str, direction: &str) -> Option<u64> {
    let prefix = format!("protocol/{}/", direction);
    let remainder = relative_path.strip_prefix(&prefix)?;
    let component = remainder.split('/').next()?;
    let numeric = if direction == "response" {
        component.split('-').next().unwrap_or(component)
    } else {
        component
    };
    numeric.parse::<u64>().ok()
}

fn protocol_archive_rows(session_id: &str) -> Result<Vec<(i64, String, String, i64, Option<String>, i64)>, String> {
    let connection = open_observation_storage()?;
    let mut statement = connection.prepare(
        "SELECT file_id,relative_path,content_type,byte_length,sha256,created_at_ms \
         FROM observation_files WHERE session_id=?1 AND relative_path LIKE 'protocol/%' \
         ORDER BY file_id"
    ).map_err(|error| format!("prepare_protocol_archive:{}", error))?;
    let mapped = statement.query_map(rusqlite::params![session_id], |row| Ok((
        row.get::<_, i64>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?,
        row.get::<_, i64>(3)?, row.get::<_, Option<String>>(4)?, row.get::<_, i64>(5)?,
    ))).map_err(|error| format!("query_protocol_archive:{}", error))?;
    let mut rows = Vec::new();
    for row in mapped {
        rows.push(row.map_err(|error| format!("decode_protocol_archive:{}", error))?);
    }
    Ok(rows)
}

fn protocol_file_json(row: &(i64, String, String, i64, Option<String>, i64)) -> String {
    let sha = row.4.as_ref().map(|value| format!("\"{}\"", json_escape(value)))
        .unwrap_or_else(|| "null".to_string());
    format!(r#"{{"file_id":{},"relative_path":"{}","content_type":"{}","byte_length":{},"sha256":{},"created_at_ms":{},"download":"/storage/download?file_id={}"}}"#,
        row.0, json_escape(&row.1), json_escape(&row.2), row.3, sha, row.5, row.0)
}

fn protocol_exchange_export_endpoint(uri: &str) -> String {
    let pairs = match parse_query_pairs(uri) { Ok(value) => value, Err(error) => return k_json_error(&error) };
    let session_id = query_pair(&pairs, "session_id");
    if session_id.is_empty() { return k_json_error("missing_session_id"); }
    let request_id = match query_pair(&pairs, "request_id").parse::<u64>() {
        Ok(value) if value > 0 => value,
        _ => return k_json_error("invalid_or_missing_request_id"),
    };
    let rows = match protocol_archive_rows(&session_id) { Ok(value) => value, Err(error) => return k_json_error(&error) };
    let request: Vec<_> = rows.iter().filter(|row| protocol_path_request_id(&row.1, "request") == Some(request_id)).collect();
    let response: Vec<_> = rows.iter().filter(|row| protocol_path_request_id(&row.1, "response") == Some(request_id)).collect();
    let request_json = request.iter().map(|row| protocol_file_json(row)).collect::<Vec<_>>().join(",");
    let response_json = response.iter().map(|row| protocol_file_json(row)).collect::<Vec<_>>().join(",");
    format!(r#"{{"ok":true,"session_id":"{}","request_id":{},"paired":{},"request_file_count":{},"response_file_count":{},"request_files":[{}],"response_files":[{}]}}"#,
        json_escape(&session_id), request_id, !request.is_empty() && !response.is_empty(),
        request.len(), response.len(), request_json, response_json)
}

fn protocol_exchanges_export_endpoint(uri: &str) -> String {
    let pairs = match parse_query_pairs(uri) { Ok(value) => value, Err(error) => return k_json_error(&error) };
    let session_id = query_pair(&pairs, "session_id");
    if session_id.is_empty() { return k_json_error("missing_session_id"); }
    let after = query_pair(&pairs, "after_request_id").parse::<u64>().unwrap_or(0);
    let limit = query_pair(&pairs, "limit").parse::<usize>().unwrap_or(200).clamp(1, 1000);
    let rows = match protocol_archive_rows(&session_id) { Ok(value) => value, Err(error) => return k_json_error(&error) };
    let mut request_ids = std::collections::BTreeSet::new();
    let mut response_ids = std::collections::BTreeSet::new();
    for row in &rows {
        if let Some(value) = protocol_path_request_id(&row.1, "request") { request_ids.insert(value); }
        if let Some(value) = protocol_path_request_id(&row.1, "response") { response_ids.insert(value); }
    }
    let selected: Vec<u64> = request_ids.union(&response_ids).copied().filter(|value| *value > after).take(limit).collect();
    let items = selected.iter().map(|request_id| format!(
        r#"{{"request_id":{},"has_request":{},"has_response":{},"paired":{},"export":"/api/sniff/exchange?session_id={}&request_id={}"}}"#,
        request_id, request_ids.contains(request_id), response_ids.contains(request_id),
        request_ids.contains(request_id) && response_ids.contains(request_id),
        json_escape(&session_id), request_id
    )).collect::<Vec<_>>();
    let next = selected.last().copied().unwrap_or(after);
    format!(r#"{{"ok":true,"session_id":"{}","after_request_id":{},"next_request_id":{},"count":{},"exchanges":[{}]}}"#,
        json_escape(&session_id), after, next, items.len(), items.join(","))
}

fn protocol_archive_audit_endpoint(uri: &str) -> String {
    let pairs = match parse_query_pairs(uri) { Ok(value) => value, Err(error) => return k_json_error(&error) };
    let session_id = query_pair(&pairs, "session_id");
    if session_id.is_empty() { return k_json_error("missing_session_id"); }
    let rows = match protocol_archive_rows(&session_id) { Ok(value) => value, Err(error) => return k_json_error(&error) };
    let mut audit = ProtocolArchiveAudit::default();
    let session_root = observation_storage_root().join("sessions").join(&session_id);
    for row in &rows {
        let direction = if row.1.starts_with("protocol/request/") { "request" } else if row.1.starts_with("protocol/response/") { "response" } else { continue };
        if let Some(request_id) = protocol_path_request_id(&row.1, direction) {
            if direction == "request" { audit.request_ids.insert(request_id); audit.request_files += 1; audit.request_bytes = audit.request_bytes.saturating_add(row.3.max(0) as u64); }
            else { audit.response_ids.insert(request_id); audit.response_files += 1; audit.response_bytes = audit.response_bytes.saturating_add(row.3.max(0) as u64); }
        }
        if row.3 == 0 { audit.zero_length_files.push(row.1.clone()); }
        match std::fs::metadata(session_root.join(&row.1)) {
            Ok(metadata) if metadata.len() != row.3.max(0) as u64 => audit.indexed_length_mismatches.push(row.1.clone()),
            Err(_) => audit.indexed_length_mismatches.push(row.1.clone()),
            _ => {}
        }
    }
    let missing_response: Vec<u64> = audit.request_ids.difference(&audit.response_ids).copied().collect();
    let orphan_response: Vec<u64> = audit.response_ids.difference(&audit.request_ids).copied().collect();
    let u64_json = |values: &[u64]| values.iter().map(|value| value.to_string()).collect::<Vec<_>>().join(",");
    let path_json = |values: &[String]| values.iter().map(|value| format!("\"{}\"", json_escape(value))).collect::<Vec<_>>().join(",");
    format!(r#"{{"ok":true,"session_id":"{}","request_count":{},"response_count":{},"paired_count":{},"request_file_count":{},"response_file_count":{},"request_bytes":{},"response_bytes":{},"missing_response_ids":[{}],"orphan_response_ids":[{}],"zero_length_files":[{}],"indexed_length_mismatches":[{}]}}"#,
        json_escape(&session_id), audit.request_ids.len(), audit.response_ids.len(),
        audit.request_ids.intersection(&audit.response_ids).count(), audit.request_files, audit.response_files,
        audit.request_bytes, audit.response_bytes, u64_json(&missing_response), u64_json(&orphan_response),
        path_json(&audit.zero_length_files), path_json(&audit.indexed_length_mismatches))
}

'''
s = s.replace(anchor, rust + anchor, 1)

replace_once(
    '        "/api/sniff/exchanges"|"/api/sniff/exchange"=>k_observation_files("protocol",uri),\n',
    '        "/api/sniff/exchanges"=>protocol_exchanges_export_endpoint(uri),\n'
    '        "/api/sniff/exchange"=>protocol_exchange_export_endpoint(uri),\n',
    "precise_exchange_routes",
)

route_anchor = '    } else if path == "/storage/status" {\n'
replace_once(
    route_anchor,
    '    } else if path == "/storage/audit" {\n        protocol_archive_audit_endpoint(&full_uri)\n' + route_anchor,
    "storage_audit_route",
)

# timeline仅保存对raw文件的引用和派生section，不复制payload正文；raw仍逐文件sync+rename。
assert '"relative_base":"{}"' in s
assert 'std::io::Write::write_all(&mut file, bytes)' in s
assert 'file.sync_data()' in s

base_anchor = "/// 辅助函数：IL2CPP类型枚举转可读名称\n"
replace_once(base_anchor, MARKER + "\n" + base_anchor, "p_marker")
SOURCE.write_text(s, encoding="utf-8")
print("protocol_archive_reliability=applied")
