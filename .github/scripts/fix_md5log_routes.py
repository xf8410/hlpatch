from pathlib import Path

p = Path("hachimi_ura_plugin/src/lib.rs")
s = p.read_text()

old = '} else if path.starts_with("/api/md5log") {'
assert s.count(old) == 1, s.count(old)
s = s.replace(old, '} else if path == "/api/md5log" {', 1)

old = '''    } else if path == "/api/md5log/install" {
        // Manually trigger MakeMd5 hook installation (useful if auto-install failed at boot)
        unsafe {'''
new = '''    } else if path == "/api/md5log/install" {
        // Scope early String returns to this route, not handle_http() -> ().
        (|| -> String {
        unsafe {'''
assert s.count(old) == 1, s.count(old)
s = s.replace(old, new, 1)

old = '''            }
        }
    } else if path == "/api/sniff" {'''
new = '''            }
        }
        })()
    } else if path == "/api/sniff" {'''
assert s.count(old) == 1, s.count(old)
s = s.replace(old, new, 1)

p.write_text(s)
