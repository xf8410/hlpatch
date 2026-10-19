#!/usr/bin/env python3
"""Fix url_decode UTF-8 mojibake in generated lib.rs (3.28.2 lineage).

Root cause (diag runs 39287529934 / 39288973508, lib.rs:13991):
old url_decode pushes decoded BYTES as chars (`result.push(n as char)`,
`result.push(bytes[i] as char)`) — Latin-1 semantics. Any multi-byte UTF-8
in a query param (e.g. Japanese LIKE patterns to /mdb/raw?sql=) arrives as
mojibake. Affects every parse_query consumer, not just /mdb/raw.

Fix: collect bytes into Vec<u8>, decode once with String::from_utf8_lossy.
ASCII behavior is unchanged; invalid sequences degrade gracefully.

Anchor-asserted, idempotent.
"""
from pathlib import Path
import sys

SRC = Path("hachimi_ura_plugin/src/lib.rs")
text = SRC.read_text(encoding="utf-8")

if "v3.28.11: URL解码 UTF-8 修复" in text:
    print("url_decode_utf8=already_applied")
    sys.exit(0)

OLD = '''/// v3.22.89: 简易URL解码（处理+和%XX）
fn url_decode(s: &str) -> String {
    let mut result = String::new();
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'+' {
            result.push(' ');
            i += 1;
        } else if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hex = &s[i + 1..i + 3];
            if let Ok(n) = u8::from_str_radix(hex, 16) {
                result.push(n as char);
                i += 3;
            } else {
                result.push(bytes[i] as char);
                i += 1;
            }
        } else {
            result.push(bytes[i] as char);
            i += 1;
        }
    }
    result
}'''

NEW = '''/// v3.28.11: URL解码 UTF-8 修复 — 先按字节收集再整体解码
/// （旧版逐字节 as char 是 Latin-1 语义，多字节字符（日文等）全变乱码；
/// 影响所有经 parse_query 的查询参数，例如 /mdb/raw?sql= 的 LIKE 模式）
fn url_decode(s: &str) -> String {
    let mut out: Vec<u8> = Vec::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'+' {
            out.push(b' ');
            i += 1;
        } else if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hex = &s[i + 1..i + 3];
            if let Ok(n) = u8::from_str_radix(hex, 16) {
                out.push(n);
                i += 3;
            } else {
                out.push(bytes[i]);
                i += 1;
            }
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}'''

count = text.count(OLD)
assert count == 1, f"url_decode anchor count={count}"
text = text.replace(OLD, NEW, 1)

assert "result.push(n as char)" not in text
assert "result.push(bytes[i] as char)" not in text

SRC.write_text(text, encoding="utf-8")
print("url_decode_utf8=applied")
