import subprocess
from pathlib import Path
import hashlib
import re

# 本候选在累计生成源码前固定唯一发布版本；Cargo.toml与lock必须同步。
cargo_toml = Path('hachimi_ura_plugin/Cargo.toml')
toml_text = cargo_toml.read_text(encoding='utf-8')
toml_text = toml_text.replace('version = "3.27.12-slim"', 'version = "3.27.12-slim"', 1)
cargo_toml.write_text(toml_text, encoding='utf-8')
cargo_lock = Path('hachimi_ura_plugin/Cargo.lock')
lock_text = cargo_lock.read_text(encoding='utf-8')
package_anchor = 'name = "hachimi_ura"\nversion = "3.27.12-slim"'
if package_anchor in lock_text:
    lock_text = lock_text.replace(package_anchor, 'name = "hachimi_ura"\nversion = "3.27.11"', 1)
elif 'name = "hachimi_ura"\nversion = "3.27.12-slim"' not in lock_text:
    raise RuntimeError('hachimi_ura Cargo.lock package version anchor missing')
cargo_lock.write_text(lock_text, encoding='utf-8')


def apply_next_generation_foundation() -> None:
    source = Path('hachimi_ura_plugin/src/lib.rs')
    text = source.read_text(encoding='utf-8')
    marker = '// ===== Next-generation passive init and HookRegistry foundation ====='
    if marker in text:
        print('next_generation_foundation=already_applied')
        return

    boot_match = re.search(r'((?:static|const)\s+BOOT_SAFE_EXACT\b[^=]*=\s*&\[)', text, re.M)
    if boot_match is None:
        raise RuntimeError('BOOT_SAFE_EXACT declaration missing')
    boot_end = text.find('];', boot_match.end())
    if boot_end < 0:
        raise RuntimeError('BOOT_SAFE_EXACT terminator missing')
    boot_values = [
        '/runtime/init_status', '/hooks/registry', '/hooks/diagnostics', '/capture/status'
    ]
    insertion = ''.join(
        f'\n    "{value}",'
        for value in boot_values
        if f'"{value}"' not in text[boot_match.start():boot_end]
    )
    text = text[:boot_match.end()] + insertion + text[boot_match.end():]

    anchor = '/// 辅助函数：IL2CPP类型枚举转可读名称\n'
    if text.count(anchor) != 1:
        raise RuntimeError(f'foundation insertion anchor count={text.count(anchor)}')

    rust = r'''// ===== Next-generation passive init and HookRegistry foundation =====
#[derive(Clone)]
struct FoundationHookRecord {
    hook_id: &'static str,
    role: &'static str,
    assembly: &'static str,
    namespace: &'static str,
    declaring_type: &'static str,
    method: &'static str,
    parameter_types: &'static [&'static str],
    return_type: &'static str,
    module_name: &'static str,
    target_address: usize,
}

struct FoundationInitStateData {
    phase: String,
    generation: u64,
    attempts: u64,
    first_observed_wall_clock_ms: u64,
    last_transition_wall_clock_ms: u64,
    last_transition_monotonic_ns: u64,
}

static FOUNDATION_INIT_STATE: Mutex<Option<FoundationInitStateData>> = Mutex::new(None);

fn foundation_monotonic_ns() -> u64 {
    unsafe {
        let mut value: libc::timespec = std::mem::zeroed();
        if libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut value) != 0 {
            return 0;
        }
        (value.tv_sec.max(0) as u64)
            .saturating_mul(1_000_000_000)
            .saturating_add(value.tv_nsec.max(0) as u64)
    }
}

unsafe fn foundation_hook_records() -> Vec<FoundationHookRecord> {
    vec![
        FoundationHookRecord {
            hook_id: "protocol.compress_request", role: "core", assembly: "umamusume.dll",
            namespace: "Gallop", declaring_type: "HttpHelper", method: "CompressRequest",
            parameter_types: &["System.Byte[]"], return_type: "System.Byte[]",
            module_name: "libil2cpp.so", target_address: COMPRESS_REQUEST_ADDR,
        },
        FoundationHookRecord {
            hook_id: "protocol.decompress_response", role: "core", assembly: "umamusume.dll",
            namespace: "Gallop", declaring_type: "HttpHelper", method: "DecompressResponse",
            parameter_types: &["System.Byte[]"], return_type: "System.Byte[]",
            module_name: "libil2cpp.so", target_address: DECOMPRESS_RESPONSE_ADDR,
        },
        FoundationHookRecord {
            hook_id: "protocol.www_post", role: "core", assembly: "Cute.Http.Assembly.dll",
            namespace: "Cute.Http", declaring_type: "WWWRequest", method: "Post",
            parameter_types: &["System.String", "System.Byte[]", "System.Collections.Generic.Dictionary<System.String,System.String>"],
            return_type: "UnityEngine.Networking.UnityWebRequestAsyncOperation",
            module_name: "libil2cpp.so", target_address: POST_ADDR,
        },
        FoundationHookRecord {
            hook_id: "protocol.unity_send", role: "core", assembly: "UnityEngine.UnityWebRequestModule.dll",
            namespace: "UnityEngine.Networking", declaring_type: "UnityWebRequest", method: "SendWebRequest",
            parameter_types: &[], return_type: "UnityEngine.Networking.UnityWebRequestAsyncOperation",
            module_name: "libil2cpp.so", target_address: UNITY_SEND_ADDR,
        },
        FoundationHookRecord {
            hook_id: "protocol.unity_completion", role: "core", assembly: "UnityEngine.CoreModule.dll",
            namespace: "UnityEngine", declaring_type: "AsyncOperation", method: "InvokeCompletionEvent",
            parameter_types: &[], return_type: "System.Void",
            module_name: "libil2cpp.so", target_address: UNITY_COMPLETE_ADDR,
        },
        FoundationHookRecord {
            hook_id: "ui.text_common_set_text", role: "optional", assembly: "umamusume.dll",
            namespace: "Gallop", declaring_type: "TextCommon", method: "set_text",
            parameter_types: &["System.String"], return_type: "System.Void",
            module_name: "libil2cpp.so", target_address: TEXT_COMMON_SET_TEXT_ADDR,
        },
    ]
}

fn foundation_parameter_types_json(values: &[&str]) -> String {
    values.iter().map(|value| format!("\"{}\"", json_escape(value)))
        .collect::<Vec<_>>().join(",")
}

fn foundation_hook_record_json(record: &FoundationHookRecord) -> String {
    let address = if record.target_address == 0 {
        "null".to_string()
    } else {
        format!("\"0x{:x}\"", record.target_address)
    };
    let owner = if record.target_address == 0 { "null" } else { "\"hlpatch_legacy_chain\"" };
    let state = if record.target_address == 0 { "not_observed" } else { "legacy_installed_unverified" };
    format!(
        r#"{{"hook_id":"{}","role":"{}","key":{{"assembly":"{}","namespace":"{}","declaring_type":"{}","method":"{}","parameter_types":[{}],"return_type":"{}"}},"method_info":null,"target_address":{},"module_name":"{}","module_base":null,"module_generation":null,"mapping_permissions":null,"alignment_valid":{},"original_prologue_bytes":null,"current_prologue_bytes":null,"prologue_fingerprint":null,"external_hook_present":null,"owner":{},"install_generation":null,"install_state":"{}","trampoline_address":null,"call_count":null,"last_call_monotonic_ns":null,"last_error_stage":null,"last_error_raw":null}}"#,
        json_escape(record.hook_id), json_escape(record.role), json_escape(record.assembly),
        json_escape(record.namespace), json_escape(record.declaring_type), json_escape(record.method),
        foundation_parameter_types_json(record.parameter_types), json_escape(record.return_type), address,
        json_escape(record.module_name), record.target_address != 0 && record.target_address % 4 == 0,
        owner, state
    )
}

fn foundation_legacy_hook_errors() -> Vec<(String, String)> {
    HOOK_STATUS.lock().map(|values| {
        values.iter().filter(|(_, status)| status.starts_with("failed:"))
            .cloned().collect::<Vec<_>>()
    }).unwrap_or_default()
}

unsafe fn foundation_observed_phase(records: &[FoundationHookRecord]) -> (&'static str, &'static str) {
    if API.is_null() {
        return ("waiting_domain", "hachimi_api_null");
    }
    if (*API).il2cpp_get_assembly_image_fn.is_none() {
        return ("waiting_assemblies", "assembly_image_api_unavailable");
    }
    if !GAME_INITIALIZED.load(Ordering::Acquire) {
        return ("probing_core_types", "game_initialization_not_observed");
    }
    let core_total = records.iter().filter(|record| record.role == "core").count();
    let core_observed = records.iter().filter(|record| record.role == "core" && record.target_address != 0).count();
    if core_observed < core_total {
        if !foundation_legacy_hook_errors().is_empty() {
            return ("degraded", "legacy_core_hook_failure_observed");
        }
        return ("installing_core_hooks", "core_hook_address_not_observed");
    }
    ("installing_core_hooks", "core_hooks_not_registry_validated")
}

fn foundation_refresh_state(phase: &str) -> (u64, u64, u64, u64, u64) {
    let now_wall = sniff_timestamp_ms();
    let now_mono = foundation_monotonic_ns();
    let mut state = match FOUNDATION_INIT_STATE.lock() {
        Ok(value) => value,
        Err(poisoned) => poisoned.into_inner(),
    };
    match state.as_mut() {
        Some(value) => {
            value.attempts = value.attempts.saturating_add(1);
            if value.phase != phase {
                value.phase = phase.to_string();
                value.generation = value.generation.saturating_add(1);
                value.last_transition_wall_clock_ms = now_wall;
                value.last_transition_monotonic_ns = now_mono;
            }
        }
        None => {
            *state = Some(FoundationInitStateData {
                phase: phase.to_string(), generation: 1, attempts: 1,
                first_observed_wall_clock_ms: now_wall,
                last_transition_wall_clock_ms: now_wall,
                last_transition_monotonic_ns: now_mono,
            });
        }
    }
    let value = state.as_ref().unwrap();
    (value.generation, value.attempts, value.first_observed_wall_clock_ms,
     value.last_transition_wall_clock_ms, value.last_transition_monotonic_ns)
}

unsafe fn foundation_init_status_endpoint() -> String {
    let records = foundation_hook_records();
    let (phase, blocker) = foundation_observed_phase(&records);
    let (generation, attempts, first_wall, transition_wall, transition_mono) = foundation_refresh_state(phase);
    let core_total = records.iter().filter(|record| record.role == "core").count();
    let core_observed = records.iter().filter(|record| record.role == "core" && record.target_address != 0).count();
    let optional_total = records.iter().filter(|record| record.role == "optional").count();
    let optional_observed = records.iter().filter(|record| record.role == "optional" && record.target_address != 0).count();
    let source_commit = option_env!("HLPATCH_SOURCE_COMMIT")
        .map(|value| format!("\"{}\"", json_escape(value))).unwrap_or_else(|| "null".to_string());
    format!(
        r#"{{"ok":true,"foundation_mode":"passive_observation_only","phase":"{}","ready":false,"readiness_blocker":"{}","generation":{},"attempts":{},"first_observed_wall_clock_ms":{},"last_transition_wall_clock_ms":{},"last_transition_monotonic_ns":{},"hooks":{{"core_total":{},"core_legacy_address_observed":{},"core_registry_validated":0,"optional_total":{},"optional_legacy_address_observed":{}}},"capture_enabled":{},"fingerprints":{{"plugin_version":"{}","source_commit":{},"source_commit_error":{},"game_version":null,"game_version_error":"not_collected_in_foundation","resource_version":null,"resource_version_error":"not_collected_in_foundation","assembly_fingerprint":null,"assembly_fingerprint_error":"not_collected_in_foundation","mdb_sha256":null,"mdb_sha256_error":"not_collected_in_foundation","hook_registry_schema_version":1,"observation_schema_version":null}}}}"#,
        phase, blocker, generation, attempts, first_wall, transition_wall, transition_mono,
        core_total, core_observed, optional_total, optional_observed,
        SNIFF_ENABLED.load(Ordering::Acquire), json_escape(PLUGIN_VERSION), source_commit,
        if option_env!("HLPATCH_SOURCE_COMMIT").is_some() { "null" } else { "\"compile_time_source_commit_unavailable\"" }
    )
}

unsafe fn foundation_hook_registry_endpoint() -> String {
    let records = foundation_hook_records();
    let items = records.iter().map(foundation_hook_record_json).collect::<Vec<_>>().join(",");
    format!(
        r#"{{"ok":true,"schema_version":1,"ownership_model":"legacy_snapshot_not_yet_managed","resolve_validate_commit_active":false,"count":{},"hooks":[{}]}}"#,
        records.len(), items
    )
}

fn foundation_hook_diagnostics_endpoint() -> String {
    let errors = foundation_legacy_hook_errors();
    let items = errors.iter().map(|(hook, error)| format!(
        r#"{{"hook_id":"{}","stage":"legacy_install","raw_error":"{}"}}"#,
        json_escape(hook), json_escape(error)
    )).collect::<Vec<_>>().join(",");
    format!(
        r#"{{"ok":true,"schema_version":1,"diagnostic_source":"legacy_hook_status","failure_count":{},"failures":[{}]}}"#,
        errors.len(), items
    )
}

fn foundation_capture_status_endpoint() -> String {
    format!(
        r#"{{"ok":true,"hooks_installed":"reported_separately_by_registry","capture_enabled":{},"active_mode":"legacy_protocol_capture","capture_generation":null,"change_sequence":null,"control_status":"read_only_foundation"}}"#,
        SNIFF_ENABLED.load(Ordering::Acquire)
    )
}

'''
    text = text.replace(anchor, rust + anchor, 1)

    route_anchor = '    } else if path == "/storage/files" {\n'
    if text.count(route_anchor) != 1:
        raise RuntimeError(f'foundation route anchor count={text.count(route_anchor)}')
    routes = '''    } else if path == "/runtime/init_status" {
        unsafe { foundation_init_status_endpoint() }
    } else if path == "/hooks/registry" {
        unsafe { foundation_hook_registry_endpoint() }
    } else if path == "/hooks/diagnostics" {
        foundation_hook_diagnostics_endpoint()
    } else if path == "/capture/status" {
        foundation_capture_status_endpoint()
'''
    text = text.replace(route_anchor, routes + route_anchor, 1)
    source.write_text(text, encoding='utf-8')
    print('next_generation_foundation=applied')


common=[
 'scripts/fix_unified_endpoint_f_patch_anchors.py',
 'scripts/apply_unified_endpoint_a_patch.py','scripts/apply_unified_endpoint_a_compile_fix.py',
 'scripts/apply_unified_endpoint_b_storage_patch.py','scripts/apply_unified_endpoint_b_storage_compile_fix.py',
 'scripts/apply_unified_endpoint_c_inherit_pair_patch.py','scripts/apply_unified_endpoint_d_selected_parent_patch.py',
 'scripts/apply_unified_endpoint_e_runtime_correction.py','scripts/apply_unified_endpoint_f_pre_release_fix.py',
 'scripts/apply_unified_endpoint_g_release_gate_fix.py','scripts/apply_unified_endpoint_h_response_headers.py',
 'scripts/apply_unified_endpoint_i_parent_multisource.py','scripts/apply_unified_endpoint_j_parent_runtime_semantics.py',
 'scripts/apply_unified_endpoint_k_complete.py']
# 两项修正修改的是K补丁生成器，必须在K第一次生成Rust源码之前执行。
subprocess.run(['python3','scripts/fix_unified_endpoint_k_complete_anchors.py'],check=True)
subprocess.run(['python3','scripts/fix_unified_endpoint_k_rust_types.py'],check=True)
for pass_no in (1,2):
    for script in common: subprocess.run(['python3',script],check=True)
    subprocess.run(['python3','scripts/apply_generated_succession_runtime_l.py'],check=True)
    subprocess.run(['python3','scripts/apply_generated_succession_runtime_l_support_fix.py'],check=True)
    subprocess.run(['python3','scripts/apply_ramen_global_durable_observation.py'],check=True)
    subprocess.run(['python3','scripts/apply_hachimi_textcommon_observer.py'],check=True)
    subprocess.run(['python3','scripts/apply_protocol_multisection_event_timeline.py'],check=True)
    subprocess.run(['python3','scripts/apply_protocol_archive_reliability_p.py'],check=True)
    apply_next_generation_foundation()
    subprocess.run(['python3','scripts/apply_storage_read_range_a1.py'],check=True)
    subprocess.run(['python3','scripts/apply_exact_method_probe_b1.py'],check=True)
    source=Path('hachimi_ura_plugin/src/lib.rs').read_bytes()
    Path(f'source-{pass_no}.sha').write_text(hashlib.sha256(source).hexdigest()+'\n')
assert Path('source-1.sha').read_text()==Path('source-2.sha').read_text()
assert 'version = "3.27.12-slim"' in cargo_toml.read_text(encoding='utf-8')
assert 'name = "hachimi_ura"\nversion = "3.27.12-slim"' in cargo_lock.read_text(encoding='utf-8')
assert 'Next-generation passive init and HookRegistry foundation' in Path('hachimi_ura_plugin/src/lib.rs').read_text(encoding='utf-8')
assert 'Exact single-method IL2CPP probe B1' in Path('hachimi_ura_plugin/src/lib.rs').read_text(encoding='utf-8')
print('generated_succession_l_cumulative=idempotent_v3.27.11_next_generation_foundation_exact_method_probe')
