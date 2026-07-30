from pathlib import Path
import re

src = Path("hachimi_ura_plugin/src/lib.rs")
s = src.read_text()

if "mod protocol_sniff;" not in s:
    s = s.replace("#![allow(dead_code)]\n", "#![allow(dead_code)]\nmod protocol_sniff;\n", 1)

marker = '    } else if path == "/api/sniff/status" {'
if 'path == "/api/sniff/decoded"' not in s:
    routes = '''    } else if path == "/api/sniff/decoded" {
        let id = parse_query(&full_uri, "id").parse::<u64>().unwrap_or(0);
        let _lock = SNIFF_MUTEX.lock();
        unsafe { protocol_sniff::render_decoded(id, &SNIFF_REQUESTS, &SNIFF_RESPONSES) }
    } else if path == "/api/sniff/schema" {
        let id = parse_query(&full_uri, "id").parse::<u64>().unwrap_or(0);
        let _lock = SNIFF_MUTEX.lock();
        unsafe { protocol_sniff::render_schema(id, &SNIFF_REQUESTS, &SNIFF_RESPONSES) }
    } else if path == "/api/sniff/routes" {
        let _lock = SNIFF_MUTEX.lock();
        unsafe { protocol_sniff::render_routes(&SNIFF_REQUESTS) }
'''
    if s.count(marker) != 1:
        raise SystemExit(f"route marker count={s.count(marker)}")
    s = s.replace(marker, routes + marker, 1)

# Advertise endpoints wherever the sniff endpoint list already occurs.
s = s.replace(
    '"/api/sniff", "/api/sniff/metadata", "/api/sniff/status",',
    '"/api/sniff", "/api/sniff/metadata", "/api/sniff/decoded", "/api/sniff/schema", "/api/sniff/routes", "/api/sniff/status",',
)
s = s.replace(
    '\"/api/sniff\",\"/api/sniff/metadata\",\"/api/sniff/status\",',
    '\"/api/sniff\",\"/api/sniff/metadata\",\"/api/sniff/decoded\",\"/api/sniff/schema\",\"/api/sniff/routes\",\"/api/sniff/status\",',
)
src.write_text(s)

cargo = Path("hachimi_ura_plugin/Cargo.toml")
c = cargo.read_text()
c, n = re.subn(r'(?m)^version = "[^"]+"', 'version = "3.25.1"', c, count=1)
if n != 1:
    raise SystemExit("version marker missing")
if 'rmpv = ' not in c:
    c = c.replace('libc = "0.2"\n', 'libc = "0.2"\nrmpv = "1"\nserde_json = "1"\n', 1)
cargo.write_text(c)
