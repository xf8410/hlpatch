from pathlib import Path

p = Path("hachimi_ura_plugin/src/lib.rs")
s = p.read_text()

old = '''    } else if path.starts_with("/il2cpp/read_string") {
        // ★ Read IL2CPP string at address (or via pointer indirection)'''
new = '''    } else if path.starts_with("/il2cpp/read_string") {
        (|| -> String {
        // ★ Read IL2CPP string at address (or via pointer indirection)'''
assert s.count(old) == 1, s.count(old)
s = s.replace(old, new, 1)

old = '''            raw_len
        )
    } else if path == "/il2cpp/search_methods_page" {'''
new = '''            raw_len
        )
        })()
    } else if path == "/il2cpp/search_methods_page" {'''
assert s.count(old) == 1, s.count(old)
s = s.replace(old, new, 1)

p.write_text(s)
