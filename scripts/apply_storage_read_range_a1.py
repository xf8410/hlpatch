from pathlib import Path

SOURCE = Path("hachimi_ura_plugin/src/lib.rs")
s = SOURCE.read_text(encoding="utf-8")
MARKER = "// ===== Observation storage raw range export A1 ====="
if MARKER in s:
    print("storage_read_range_a1=already_applied")
    raise SystemExit(0)

anchor = "/// 辅助函数：IL2CPP类型枚举转可读名称\n"
assert s.count(anchor) == 1, f"read_range insertion anchor count={s.count(anchor)}"

rust = r'''// ===== Observation storage raw range export A1 =====
fn storage_range_error(stream: &mut std::net::TcpStream, status: &str, error: &str) {
    use std::io::Write;
    let body = format!(r#"{{"ok":false,"error":"{}"}}"#, json_escape(error));
    let response = format!(
        "HTTP/1.1 {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        status, body.len(), body
    );
    let _ = stream.write_all(response.as_bytes());
    let _ = stream.flush();
}

fn storage_read_range(stream: &mut std::net::TcpStream, uri: &str) {
    use std::io::{Read, Seek, SeekFrom, Write};
    let pairs = match parse_query_pairs(uri) {
        Ok(value) => value,
        Err(error) => {
            storage_range_error(stream, "400 Bad Request", &error);
            return;
        }
    };
    if pairs.iter().filter(|(key, _)| key == "file_id").count() != 1
        || pairs.iter().filter(|(key, _)| key == "offset").count() != 1
        || pairs.iter().filter(|(key, _)| key == "length").count() != 1
    {
        storage_range_error(stream, "400 Bad Request", "missing_or_duplicate_range_parameter");
        return;
    }
    let file_id = match query_pair(&pairs, "file_id").parse::<i64>() {
        Ok(value) if value > 0 => value,
        _ => {
            storage_range_error(stream, "400 Bad Request", "invalid_file_id");
            return;
        }
    };
    let offset = match query_pair(&pairs, "offset").parse::<u64>() {
        Ok(value) => value,
        _ => {
            storage_range_error(stream, "400 Bad Request", "invalid_offset");
            return;
        }
    };
    let requested_length = match query_pair(&pairs, "length").parse::<u64>() {
        Ok(value) if value > 0 => value,
        _ => {
            storage_range_error(stream, "400 Bad Request", "invalid_length");
            return;
        }
    };
    let connection = match open_observation_storage() {
        Ok(value) => value,
        Err(error) => {
            storage_range_error(stream, "500 Internal Server Error", &error);
            return;
        }
    };
    let record = connection.query_row(
        "SELECT session_id,relative_path,byte_length FROM observation_files WHERE file_id=?1",
        rusqlite::params![file_id],
        |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, i64>(2)?)),
    );
    let (session_id, relative_path, indexed_length) = match record {
        Ok(value) => value,
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            storage_range_error(stream, "404 Not Found", "file_not_found");
            return;
        }
        Err(error) => {
            storage_range_error(stream, "500 Internal Server Error", &format!("query_file:{}", error));
            return;
        }
    };
    if indexed_length < 0 {
        storage_range_error(stream, "500 Internal Server Error", "negative_indexed_length");
        return;
    }
    let session_root = observation_storage_root().join("sessions").join(&session_id);
    let target = session_root.join(&relative_path);
    let canonical_root = match session_root.canonicalize() {
        Ok(value) => value,
        Err(error) => {
            storage_range_error(stream, "500 Internal Server Error", &format!("canonical_session_root:{}", error));
            return;
        }
    };
    let canonical_target = match target.canonicalize() {
        Ok(value) => value,
        Err(error) => {
            storage_range_error(stream, "404 Not Found", &format!("canonical_file:{}", error));
            return;
        }
    };
    if !canonical_target.starts_with(&canonical_root) {
        storage_range_error(stream, "409 Conflict", "file_outside_session_root");
        return;
    }
    let mut file = match std::fs::File::open(&canonical_target) {
        Ok(value) => value,
        Err(error) => {
            storage_range_error(stream, "404 Not Found", &format!("open_file:{}", error));
            return;
        }
    };
    let actual_total = match file.metadata() {
        Ok(metadata) if metadata.is_file() => metadata.len(),
        Ok(_) => {
            storage_range_error(stream, "409 Conflict", "indexed_target_not_file");
            return;
        }
        Err(error) => {
            storage_range_error(stream, "500 Internal Server Error", &format!("file_metadata:{}", error));
            return;
        }
    };
    if actual_total != indexed_length as u64 {
        storage_range_error(stream, "409 Conflict", "indexed_length_mismatch");
        return;
    }
    if actual_total == 0 {
        if offset != 0 {
            storage_range_error(stream, "416 Range Not Satisfiable", "offset_out_of_range");
            return;
        }
        let header = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: 0\r\nAccept-Ranges: bytes\r\nX-HLPATCH-File-Id: {}\r\nX-HLPATCH-File-Length: 0\r\nX-HLPATCH-Range-Start: 0\r\nX-HLPATCH-Range-End-Exclusive: 0\r\nConnection: close\r\n\r\n",
            file_id
        );
        let _ = stream.write_all(header.as_bytes());
        let _ = stream.flush();
        return;
    }
    if offset >= actual_total {
        storage_range_error(stream, "416 Range Not Satisfiable", "offset_out_of_range");
        return;
    }
    let actual_length = requested_length.min(actual_total - offset);
    if let Err(error) = file.seek(SeekFrom::Start(offset)) {
        storage_range_error(stream, "500 Internal Server Error", &format!("seek_file:{}", error));
        return;
    }
    let header = format!(
        "HTTP/1.1 206 Partial Content\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\nAccept-Ranges: bytes\r\nContent-Range: bytes {}-{}/{}\r\nX-HLPATCH-File-Id: {}\r\nX-HLPATCH-File-Length: {}\r\nX-HLPATCH-Range-Start: {}\r\nX-HLPATCH-Range-End-Exclusive: {}\r\nConnection: close\r\n\r\n",
        actual_length, offset, offset + actual_length - 1, actual_total, file_id,
        actual_total, offset, offset + actual_length
    );
    if stream.write_all(header.as_bytes()).is_err() {
        return;
    }
    let mut remaining = actual_length;
    let mut buffer = [0u8; 65536];
    while remaining > 0 {
        let want = remaining.min(buffer.len() as u64) as usize;
        let count = match file.read(&mut buffer[..want]) {
            Ok(0) => return,
            Ok(value) => value,
            Err(_) => return,
        };
        if stream.write_all(&buffer[..count]).is_err() {
            return;
        }
        remaining -= count as u64;
    }
    let _ = stream.flush();
}

'''
s = s.replace(anchor, rust + anchor, 1)

boot_anchor = '    "/storage/download",\n'
assert s.count(boot_anchor) == 1, f"read_range boot anchor count={s.count(boot_anchor)}"
s = s.replace(boot_anchor, boot_anchor + '    "/storage/read_range",\n', 1)

route_anchor = '    let _parsed_request_uri = parse_request_uri(req).unwrap_or_else(|_| full_uri.to_string());\n'
assert s.count(route_anchor) == 1, f"read_range route anchor count={s.count(route_anchor)}"
s = s.replace(
    route_anchor,
    route_anchor
    + '''    if path == "/storage/read_range" {
        storage_read_range(&mut stream, &full_uri);
        return;
    }
''',
    1,
)

s = s.replace(anchor, MARKER + "\n" + anchor, 1)
SOURCE.write_text(s, encoding="utf-8")
print("storage_read_range_a1=applied")
