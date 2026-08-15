use std::collections::{BTreeMap, BTreeSet};

const MAX_CANDIDATES: usize = 200;

fn lower_path(url: &str) -> String {
    super::sniff_path(url).to_ascii_lowercase()
}

fn risk_for_path(path: &str) -> (&'static str, &'static str, bool) {
    let p = path.to_ascii_lowercase();
    let destructive = [
        "signup", "purchase", "buy", "gacha", "draw", "exec_command", "exec-command",
        "training", "race/entry", "race_entry", "finish", "start", "claim", "receive",
        "present/receive", "friend/request", "delete", "remove", "update", "change",
    ];
    if destructive.iter().any(|needle| p.contains(needle)) {
        return ("state_changing_or_unknown", "deny_replay_candidate", false);
    }
    let likely_read_only = [
        "status", "version", "information", "notice", "load", "list", "index",
        "profile", "configuration", "config", "master", "check", "get_", "search",
    ];
    if likely_read_only.iter().any(|needle| p.contains(needle)) {
        return ("likely_read_only_unverified", "manual_review_required", false);
    }
    ("unknown", "manual_review_required", false)
}

fn address_json(address: usize) -> String {
    if address == 0 { "null".to_string() } else { format!("\"0x{:x}\"", address) }
}

fn hook_json(
    hook_id: &str,
    assembly: &str,
    namespace: &str,
    class_name: &str,
    method: &str,
    params: &[&str],
    return_type: &str,
    address: usize,
    boundary: &str,
) -> String {
    let params = params.iter().map(|value| format!("\"{}\"", super::json_escape(value)))
        .collect::<Vec<_>>().join(",");
    format!(r#"{{"hook_id":"{}","assembly":"{}","namespace":"{}","class":"{}","method":"{}","parameter_types":[{}],"return_type":"{}","target_address":{},"observed":{},"boundary":"{}"}}"#,
        super::json_escape(hook_id), super::json_escape(assembly), super::json_escape(namespace),
        super::json_escape(class_name), super::json_escape(method), params,
        super::json_escape(return_type), address_json(address), address != 0,
        super::json_escape(boundary))
}

pub unsafe fn discovery_endpoint() -> String {
    let hooks = vec![
        hook_json(
            "protocol.compress_request", "umamusume.dll", "Gallop", "HttpHelper", "CompressRequest",
            &["System.Byte[]"], "System.Byte[]", super::COMPRESS_REQUEST_ADDR, "request_plain_to_encoded",
        ),
        hook_json(
            "protocol.www_post", "Cute.Http.Assembly.dll", "Cute.Http", "WWWRequest", "Post",
            &["System.String", "System.Byte[]", "System.Collections.Generic.Dictionary<System.String,System.String>"],
            "UnityEngine.Networking.UnityWebRequestAsyncOperation", super::POST_ADDR, "game_http_dispatch",
        ),
        hook_json(
            "protocol.unity_send", "UnityEngine.UnityWebRequestModule.dll", "UnityEngine.Networking",
            "UnityWebRequest", "SendWebRequest", &[], "UnityEngine.Networking.UnityWebRequestAsyncOperation",
            super::UNITY_SEND_ADDR, "unity_transport_dispatch",
        ),
        hook_json(
            "protocol.unity_completion", "UnityEngine.CoreModule.dll", "UnityEngine", "AsyncOperation",
            "InvokeCompletionEvent", &[], "System.Void", super::UNITY_COMPLETE_ADDR, "unity_transport_completion",
        ),
        hook_json(
            "protocol.decompress_response", "umamusume.dll", "Gallop", "HttpHelper", "DecompressResponse",
            &["System.Byte[]"], "System.Byte[]", super::DECOMPRESS_RESPONSE_ADDR, "encoded_to_response_plain",
        ),
    ];
    let hook_ready = super::COMPRESS_REQUEST_ADDR != 0 && super::POST_ADDR != 0
        && super::UNITY_SEND_ADDR != 0 && super::UNITY_COMPLETE_ADDR != 0
        && super::DECOMPRESS_RESPONSE_ADDR != 0;
    let captured = super::SNIFF_REQUESTS.len();
    format!(r#"{{"ok":true,"schema_version":1,"mode":"passive_in_game_send_chain_discovery","mutates_game_or_network":false,"adds_new_hooks":false,"capture_enabled":{},"known_chain_ready":{},"captured_request_count":{},"chain":[{}],"unresolved":[{{"role":"business_api_entry","examples":["Gallop.SingleModeAPI.SendExecCommand","Gallop.SingleModeRamenAPI.SendExecCommand","SendCheckEvent","SendLoad"],"resolution":"observe_real_call_then_exact_method_probe"}},{{"role":"request_formatter","resolution":"correlate formatter/serializer call before CompressRequest"}},{{"role":"response_formatter","resolution":"correlate formatter/deserializer call after DecompressResponse"}},{{"role":"main_thread_dispatch","resolution":"record thread identity on real calls before commit implementation"}}],"next":"capture normal game traffic, inspect /api/protocol/send/candidates, then exact-probe one reviewed route"}}"#,
        super::SNIFF_ENABLED.load(std::sync::atomic::Ordering::Acquire), hook_ready, captured, hooks.join(","))
}

pub unsafe fn candidates_endpoint() -> String {
    let _guard = match super::SNIFF_MUTEX.lock() {
        Ok(value) => value,
        Err(poisoned) => poisoned.into_inner(),
    };
    let mut grouped: BTreeMap<String, (String, String, usize, usize, BTreeSet<u64>)> = BTreeMap::new();
    for (request_id, url, method, body) in super::SNIFF_REQUESTS.iter() {
        let path = lower_path(url);
        let item = grouped.entry(path).or_insert_with(|| (
            url.clone(), method.clone(), 0, body.len(), BTreeSet::new()
        ));
        item.2 += 1;
        item.3 = item.3.max(body.len());
        item.4.insert(*request_id);
    }
    let mut rows = Vec::new();
    for (path, (url, method, observations, max_body, ids)) in grouped.into_iter().take(MAX_CANDIDATES) {
        let (risk, policy, approved) = risk_for_path(&path);
        let id_json = ids.iter().rev().take(5).copied().collect::<Vec<_>>().into_iter().rev()
            .map(|value| value.to_string()).collect::<Vec<_>>().join(",");
        rows.push(format!(r#"{{"path":"{}","example_url":"{}","method":"{}","observation_count":{},"max_request_plain_size":{},"recent_request_ids":[{}],"risk":"{}","replay_policy":"{}","approved_for_commit":{},"review":{{"required":true,"reason":"path-name classification is evidence, not proof of server-side idempotence"}}}}"#,
            super::json_escape(&path), super::json_escape(&url), super::json_escape(&method),
            observations, max_body, id_json, risk, policy, approved));
    }
    format!(r#"{{"ok":true,"schema_version":1,"mode":"captured_real_game_routes","mutates_game_or_network":false,"default_deny":true,"automatic_commit_enabled":false,"count":{},"candidates":[{}]}}"#,
        rows.len(), rows.join(","))
}

pub unsafe fn evidence_endpoint(uri: &str) -> String {
    let pairs = match super::parse_query_pairs(uri) {
        Ok(value) => value,
        Err(error) => return super::k_json_error(&error),
    };
    let request_id = match super::query_pair(&pairs, "request_id").parse::<u64>() {
        Ok(value) if value > 0 => value,
        _ => return super::k_json_error("invalid_or_missing_request_id"),
    };
    let _guard = match super::SNIFF_MUTEX.lock() {
        Ok(value) => value,
        Err(poisoned) => poisoned.into_inner(),
    };
    let request = match super::SNIFF_REQUESTS.iter().rev().find(|item| item.0 == request_id) {
        Some(value) => value,
        None => return super::k_json_error("request_id_not_found_or_evicted"),
    };
    let response = super::SNIFF_RESPONSES.iter().rev().find(|item| item.0 == request_id);
    let path = lower_path(&request.1);
    let (risk, policy, approved) = risk_for_path(&path);
    format!(r#"{{"ok":true,"schema_version":1,"request_id":{},"url":"{}","path":"{}","method":"{}","request_plain":{{"captured":true,"size":{},"hook":"HttpHelper.CompressRequest:input"}},"request_dispatch":{{"observed":{},"hook":"WWWRequest.Post"}},"response_plain":{{"captured":{},"size":{},"hook":"HttpHelper.DecompressResponse:output"}},"risk":"{}","replay_policy":"{}","approved_for_commit":{},"prepare_readiness":{{"ready":false,"blockers":["exact business API or reviewed generic send entry not selected","main-thread dispatch contract not verified","server-side idempotence not proven","one-time commit storage not implemented"]}},"mutation":{{"performed":false,"network_request_sent":false}}}}"#,
        request_id, super::json_escape(&request.1), super::json_escape(&path), super::json_escape(&request.2),
        request.3.len(), super::POST_ADDR != 0, response.is_some(), response.map(|item| item.1.len()).unwrap_or(0),
        risk, policy, approved)
}
