from pathlib import Path

SOURCE = Path("hachimi_ura_plugin/src/lib.rs")
s = SOURCE.read_text(encoding="utf-8")

MARKER = "// ===== Unified runtime correction E-stage ====="
if MARKER in s:
    print("unified_endpoint_e_runtime_correction=already_applied")
    raise SystemExit(0)

old_root = '''fn observation_storage_root() -> std::path::PathBuf {
    if let Some(so_path) = find_own_so_path() {
        if let Some(parent) = std::path::Path::new(&so_path).parent() {
            return parent.join("hlpatch-observations");
        }
    }
    std::path::PathBuf::from("/data/data/jp.pokemon.pokeuma/files/hlpatch-observations")
}
'''
new_root = '''fn observation_storage_root() -> std::path::PathBuf {
    if let Ok(command_line) = std::fs::read("/proc/self/cmdline") {
        let package_bytes = command_line.split(|byte| *byte == 0).next().unwrap_or(&[]);
        if let Ok(package_name) = std::str::from_utf8(package_bytes) {
            if !package_name.is_empty() {
                return std::path::PathBuf::from("/data/user/0")
                    .join(package_name)
                    .join("files")
                    .join("hlpatch-observations");
            }
        }
    }
    std::path::PathBuf::from("/data/user/0/jp.co.cygames.umamusume/files/hlpatch-observations")
}
'''
assert s.count(old_root) == 1, f"storage root correction anchor count={s.count(old_root)}"
s = s.replace(old_root, new_root, 1)

old_parent = '''        let trained_chara_id = call_getter_int(
            succession_info_class,
            info,
            "get_TrainedCharaId",
        );
'''
new_parent = '''        let trained_chara_id = call_getter_obscured_int(
            succession_info_class,
            info,
            "get_TrainedCharaId",
        );
'''
assert s.count(old_parent) == 1, f"trained chara decode correction anchor count={s.count(old_parent)}"
s = s.replace(old_parent, new_parent, 1)

anchor = "/// 辅助函数：IL2CPP类型枚举转可读名称\n"
assert s.count(anchor) == 1
s = s.replace(anchor, MARKER + "\n" + anchor, 1)

SOURCE.write_text(s, encoding="utf-8")
print("unified_endpoint_e_runtime_correction=applied")
