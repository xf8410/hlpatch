from pathlib import Path

SOURCE = Path("hachimi_ura_plugin/src/lib.rs")
s = SOURCE.read_text(encoding="utf-8")
MARKER = "// ===== Hachimi TextCommon final display observer N-stage ====="
if MARKER in s:
    print("hachimi_textcommon_observer=already_applied")
    raise SystemExit(0)


def replace_once(old: str, new: str, label: str) -> None:
    global s
    count = s.count(old)
    assert count == 1, f"{label} anchor count={count}"
    s = s.replace(old, new, 1)

anchor = "/// 辅助函数：IL2CPP类型枚举转可读名称\n"
assert s.count(anchor) == 1
rust = r'''// ===== Hachimi TextCommon final display observer N-stage =====
static mut TEXT_COMMON_SET_TEXT_ADDR: usize = 0;

// 先执行原始set_text，再读取同一TextCommon对象的get_text。
// 这样同时保存调用输入与该调用返回时组件实际持有的显示文本，不把输入值冒充最终汉化结果。
extern "C" fn text_common_set_text_hook_handler(this: *mut c_void, input: *const c_void) {
    unsafe {
        let trampoline = interceptor_get_trampoline(text_common_set_text_hook_handler as usize);
        if trampoline == 0 { return; }
        type FnType = unsafe extern "C" fn(*mut c_void, *const c_void);
        let original: FnType = std::mem::transmute(trampoline);
        original(this, input);

        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let input_text = read_il2cpp_string(input);
            let displayed_text = if this.is_null() {
                String::new()
            } else {
                read_il2cpp_string_from_obj(this, "get_text")
            };
            let payload = format!(
                r#"{{"hook":"Gallop.TextCommon.set_text(System.String)","object":"0x{:x}","input_text":"{}","displayed_text_after_original":"{}","displayed_text_status":"read_after_original_get_text"}}"#,
                this as usize, json_escape(&input_text), json_escape(&displayed_text)
            );
            if let Err(error) = append_global_observation("ui_text", "complete", &payload, false) {
                storage_set_error(&format!("persist_ui_text:{}", error));
            }
        }));
    }
}

unsafe fn install_text_common_observer_hook() {
    if TEXT_COMMON_SET_TEXT_ADDR != 0 { return; }
    if API.is_null() {
        set_hook_status("ui_text.text_common_set_text", "failed: api_null");
        return;
    }
    let api = &*API;
    if api.interceptor == 0 {
        set_hook_status("ui_text.text_common_set_text", "failed: interceptor_unavailable");
        return;
    }
    let get_assembly_image = match api.il2cpp_get_assembly_image_fn {
        Some(value) => value,
        None => {
            set_hook_status("ui_text.text_common_set_text", "failed: assembly_api_unavailable");
            return;
        }
    };
    let get_class = match api.il2cpp_get_class_fn {
        Some(value) => value,
        None => {
            set_hook_status("ui_text.text_common_set_text", "failed: class_api_unavailable");
            return;
        }
    };
    let get_method_addr = match api.il2cpp_get_method_addr_fn {
        Some(value) => value,
        None => {
            set_hook_status("ui_text.text_common_set_text", "failed: method_api_unavailable");
            return;
        }
    };
    let image = get_assembly_image(to_cstr("umamusume.dll").as_ptr());
    if image.is_null() {
        set_hook_status("ui_text.text_common_set_text", "failed: image_not_found");
        return;
    }
    let class = get_class(image, to_cstr("Gallop").as_ptr(), to_cstr("TextCommon").as_ptr());
    if class.is_null() {
        set_hook_status("ui_text.text_common_set_text", "failed: class_not_found");
        return;
    }
    let address = get_method_addr(class as usize, to_cstr("set_text").as_ptr(), 1);
    if address == 0 {
        set_hook_status("ui_text.text_common_set_text", "failed: method_not_found");
        return;
    }
    if interceptor_hook(address, text_common_set_text_hook_handler as usize) {
        TEXT_COMMON_SET_TEXT_ADDR = address;
        set_hook_status("ui_text.text_common_set_text", &format!("hooked@0x{:x}", address));
    } else {
        set_hook_status("ui_text.text_common_set_text", "failed: interceptor_hook");
    }
}

'''
replace_once(anchor, rust + anchor, "text_observer_code")

# install_api_sniff_hooks已在游戏初始化、fallback探测及手动toggle路径调用；
# 在其入口重试TextCommon安装可覆盖早期类尚未就绪的情况。
install_anchor = '''unsafe fn install_api_sniff_hooks() {
    let all_hooked = COMPRESS_REQUEST_ADDR != 0
'''
install_new = '''unsafe fn install_api_sniff_hooks() {
    install_text_common_observer_hook();
    let all_hooked = COMPRESS_REQUEST_ADDR != 0
'''
replace_once(install_anchor, install_new, "install_text_hook")

SOURCE.write_text(s, encoding="utf-8")
print("hachimi_textcommon_observer=applied")
