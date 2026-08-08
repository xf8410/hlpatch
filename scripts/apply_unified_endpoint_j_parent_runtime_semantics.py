from pathlib import Path

SOURCE = Path("hachimi_ura_plugin/src/lib.rs")
s = SOURCE.read_text(encoding="utf-8")
MARKER = "// ===== Selected-parent runtime semantics J-stage ====="
if MARKER in s:
    print("unified_endpoint_j_parent_runtime_semantics=already_applied")
    raise SystemExit(0)

old = '''    let id = call_getter_obscured_int(record_class, record, "get_Id");
    let card_id = call_getter_int(record_class, record, "get_CardId");
    let chara_id = call_getter_obscured_int(record_class, record, "get_CharaId");
    let is_player = call_getter_bool(record_class, record, "get_IsPlayer");
    let is_rental = call_getter_bool(record_class, record, "get_IsRental");
    let is_others = call_getter_bool(record_class, record, "get_IsOthers");
    let is_succession_only = call_getter_bool(record_class, record, "get_IsSuccessionOnly");
'''
new = '''    // Runtime MethodInfo says Id/CardId return System.Int32, while CharaId returns ObscuredInt.
    let id = call_getter_int(record_class, record, "get_Id");
    let card_id = call_getter_int(record_class, record, "get_CardId");
    let chara_id = call_getter_obscured_int(record_class, record, "get_CharaId");
    // Boolean return values are boxed with one payload byte at object + 0x10.
    let boxed_bool = |method_name: &str| -> bool {
        let boxed = call_getter_ref(record_class, record, method_name);
        !boxed.is_null() && std::ptr::read_unaligned::<u8>((boxed as *const u8).add(16)) != 0
    };
    let is_player = boxed_bool("get_IsPlayer");
    let is_rental = boxed_bool("get_IsRental");
    let is_others = boxed_bool("get_IsOthers");
    let is_succession_only = boxed_bool("get_IsSuccessionOnly");
'''
assert s.count(old) == 1, f"record getter semantics anchor count={s.count(old)}"
s = s.replace(old, new, 1)
old_contract = '"selected_temp_coverage":"included_by_succession_only_all_lookup"'
new_contract = '"selected_temp_lookup":"via_succession_only_get_all_contract","selected_temp_runtime_hit":"pending_device_execution"'
assert s.count(old_contract) == 1, f"selected temp contract anchor count={s.count(old_contract)}"
s = s.replace(old_contract, new_contract, 1)
anchor = "/// 辅助函数：IL2CPP类型枚举转可读名称\n"
assert s.count(anchor) == 1
s = s.replace(anchor, MARKER + "\n" + anchor, 1)
SOURCE.write_text(s, encoding="utf-8")
print("unified_endpoint_j_parent_runtime_semantics=applied")
