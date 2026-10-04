//! Immutable versioned observations. Unknown state is never simulated or defaulted.
use serde_json::{Map, Value, json};
use std::{sync::{Arc, atomic::{AtomicU64, Ordering}}, time::{SystemTime, UNIX_EPOCH}};

/// Required PC base-state fields, including explicitly nullable story.
pub const BASE_FIELDS: &[&str] = &[
    "scenarioId", "umaId", "umaStar", "turn", "vital", "maxVital", "motivation",
    "fiveStatus", "fiveStatusLimit", "skillPt", "skillScore", "totalHints",
    "trainLevelCount", "ptScoreRate", "failureRateBias", "isIll", "isQieZhe",
    "isAiJiao", "isPositiveThinking", "isRefreshMind", "isLucky", "zhongMaBlueCount",
    "isRacing", "cardId", "persons", "personDistribution", "lockedTrainingId",
    "friendship_noncard_yayoi", "friendship_noncard_reporter", "friend_stage",
    "friend_outgoingUsed", "playing_state", "raceHistory", "story", "source",
    "single_mode_chara_id",
];
/// Required ramen state, using the PC wire names.
pub const RAMEN_FIELDS: &[&str] = &[
    "feeling_gauge_gains", "feeling_gauge", "feeling_stock", "special_feeling",
    "train_feeling_type", "active_effect_array", "super_ramen", "selected_regions",
    "feeling_gauge_gain_base", "last_ramen", "scenario_pt", "next_scenario_pt",
];
/// State needed to continue simulation without inventing earlier actions.
pub const CONTINUATION_FIELDS: &[&str] = &[
    "eat_count", "rmj_results", "train_level_bonus", "pending_ramen",
    "pending_special_targets", "inherit_extra_count",
];

/// One sampler-owned publication cache. Readers share only immutable published values.
pub struct Publisher {
    instance: String,
    serial: u64,
    issued: u64,
    completed: u64,
    captured_at: u64,
    material: Option<Value>,
    current: Option<Arc<Value>>,
}

/// A single-use capture permit. Only the sampler issues permits; readers use `current`.
pub struct CaptureTicket { instance: String, ordinal: u64 }

/// Safe transport/path identifier. The legacy sentinel is reserved for V1 readers.
pub fn valid_instance_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= 128 && id != "legacy-v1"
        && id != "." && id != ".."
        && id.bytes().all(|c| c.is_ascii_alphanumeric() || b"._-".contains(&c))
}

impl Default for Publisher {
    fn default() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos();
        let instance = format!("collector-{}-{stamp}-{}", std::process::id(), COUNTER.fetch_add(1, Ordering::Relaxed));
        Self { instance, serial: 0, issued: 0, completed: 0, captured_at: 0, material: None, current: None }
    }
}

impl Publisher {
    /// Start one publisher for one externally supplied process epoch.
    pub fn new(instance: &str) -> Result<Self, String> {
        if !valid_instance_id(instance) { return Err("invalid_collector_instance_id".into()); }
        Ok(Self { instance: instance.into(), serial: 0, issued: 0, completed: 0, captured_at: 0, material: None, current: None })
    }
    /// Identity confirmed by the HTTP capabilities handshake.
    pub fn collector_instance_id(&self) -> &str { &self.instance }
    /// This producer emits V2 only; it never resets a V1 run counter on restart.
    pub fn capabilities(&self, version: &str) -> Value {
        json!({"capability_schema_version":1,"snapshot_schema_versions":[2],
            "collector_instance_id":self.instance,"collector_version":version})
    }
    /// Issue before sampling. A completed newer ticket makes older tickets permanently stale.
    pub fn begin_capture(&mut self) -> Result<CaptureTicket, String> {
        self.issued = self.issued.checked_add(1).ok_or("capture sequence exhausted")?;
        Ok(CaptureTicket { instance: self.instance.clone(), ordinal: self.issued })
    }
    /// Publish a freshly captured value once. Cached data cannot obtain a new identity.
    pub fn finish_capture(&mut self, ticket: CaptureTicket, mut observation: Value, at: u64) -> Result<&Value, String> {
        if ticket.instance != self.instance || ticket.ordinal <= self.completed || ticket.ordinal > self.issued {
            return Err("stale_capture_ticket".into());
        }
        // Timestamp is additional freshness evidence, never an epoch identifier.
        if at < self.captured_at { return Err("stale_capture_timestamp".into()); }
        let object = observation.as_object_mut().ok_or("observation must be an object")?;
        if object.contains_key("snapshot_id") || object.contains_key("captured_at_ms") {
            return Err("cached_envelope_cannot_be_republished".into());
        }
        for key in ["snapshot_id", "captured_at_ms"] { object.remove(key); }
        object.insert("schema_version".into(), json!(2));
        object.insert("collector_instance_id".into(), json!(self.instance));
        self.completed = ticket.ordinal;
        self.captured_at = at;
        if self.material.as_ref() == Some(&observation) {
            return self.current.as_deref().ok_or_else(|| "publication invariant".into());
        }
        self.serial = self.serial.checked_add(1).ok_or("sequence exhausted")?;
        self.material = Some(observation.clone());
        observation["snapshot_id"] = json!(self.serial);
        observation["captured_at_ms"] = json!(at);
        let missing = missing_fields(&observation);
        observation["ready"] = json!(missing.is_empty());
        observation["missing_fields"] = json!(missing);
        self.current = Some(Arc::new(observation));
        self.current.as_deref().ok_or_else(|| "publication invariant".into())
    }
    /// Read the cached immutable observation.
    pub fn current(&self) -> Option<&Value> { self.current.as_deref() }
    /// Shallow publication handle; capture deduplication preserves this allocation too.
    pub fn shared_current(&self) -> Option<Arc<Value>> { self.current.clone() }
    /// Convert known legacy measurements from the sampler's ticket; never elevate inferred fields.
    pub fn finish_summary(&mut self, ticket: CaptureTicket, summary: &str, version: &str, at: u64) -> Result<&Value, String> {
        let raw: Value = serde_json::from_str(summary).map_err(|e| e.to_string())?;
        self.finish_capture(ticket, from_legacy_summary(&raw, version), at)
    }
    /// Test helper for the historical synchronous sampling API; unavailable to production hosts.
    #[cfg(test)]
    fn observe_summary(&mut self, summary: &str, version: &str, at: u64) -> Result<&Value, String> {
        let ticket = self.begin_capture()?;
        self.finish_summary(ticket, summary, version, at)
    }
    /// Test helper that keeps the original validation regressions exercising the public path.
    #[cfg(test)]
    fn publish(&mut self, observation: Value, at: u64) -> Result<&Value, String> {
        let ticket = self.begin_capture()?;
        self.finish_capture(ticket, observation, at)
    }
}

/// Presence and producer validation. Runtime performs full semantic validation too.
pub fn missing_fields(value: &Value) -> Vec<String> {
    let mut missing = Vec::new();
    // Producer vetoes carry semantic information that field presence cannot prove.
    if value.get("ready").and_then(Value::as_bool) == Some(false) { missing.push("producer_not_ready".into()); }
    if let Some(declared) = value.get("missing_fields") {
        match declared.as_array() {
            Some(fields) => for field in fields {
                missing.push(field.as_str().unwrap_or("invalid_missing_fields_entry").into());
            },
            None => missing.push("invalid_missing_fields".into()),
        }
    }
    match value["schema_version"].as_u64() {
        Some(1) => {},
        Some(2) => if !value["collector_instance_id"].as_str().is_some_and(valid_instance_id) { missing.push("collector_instance_id".into()); },
        _ => missing.push("schema_version".into()),
    }
    if value["run_id"].as_u64().filter(|n| *n > 0).is_none() { missing.push("run_id".into()); }
    for field in ["collector_version", "game_version"] {
        if value[field].as_str().filter(|s| !s.is_empty()).is_none() { missing.push(field.into()); }
    }
    if value["capture_coherence"].as_str() != Some("verified") { missing.push("capture_coherence".into()); }
    if !matches!(value["stage"].as_str(), Some("train" | "ramen_select" | "special_select" | "region_select" | "super_ramen_select" | "event" | "settlement")) { missing.push("stage".into()); }
    for (path, object, fields) in [
        ("state.baseGame", &value["state"]["baseGame"], BASE_FIELDS),
        ("state.ramen", &value["state"]["ramen"], RAMEN_FIELDS),
        ("continuation", &value["continuation"], CONTINUATION_FIELDS),
    ] {
        for field in fields {
            if !object.get(*field).is_some_and(|v| !v.is_null() || matches!(*field, "story" | "pending_ramen")) {
                missing.push(format!("{path}.{field}"));
            }
        }
    }
    if value.get("event").is_none() || (value["stage"] == "event" && !value["event"].is_object()) { missing.push("event".into()); }
    if value["state"]["baseGame"]["scenarioId"].as_u64() != Some(14) { missing.push("scenarioId=14".into()); }
    missing.sort(); missing.dedup(); missing
}

/// Explicitly partial conversion. Legacy turn numbering, partner identities,
/// gauge remaining/value semantics, and capture coherence are unverified.
pub fn from_legacy_summary(raw: &Value, version: &str) -> Value {
    let mut base = Map::new(); let mut ramen = Map::new(); let mut provenance = Map::new();
    if raw["scenario"].as_str() == Some("Ramen") { base.insert("scenarioId".into(), json!(14)); }
    copy_nonnegative(raw.get("chara_id"), "umaId", &mut base);
    for (source, target) in [("vital", "vital"), ("max_vital", "maxVital"), ("skill_point", "skillPt")] {
        copy_nonnegative(raw["stats"].get(source), target, &mut base);
    }
    let motivation = raw["stats"]["motivation"].as_i64().filter(|v| (1..=5).contains(v)).or_else(|| match raw["stats"]["motivation"].as_str() {
        Some("Best") => Some(5), Some("Good") => Some(4), Some("Normal") => Some(3),
        Some("Bad") => Some(2), Some("Worst") => Some(1), _ => None,
    });
    if let Some(v) = motivation { base.insert("motivation".into(), json!(v)); }
    for (source, target) in [("stats", "fiveStatus"), ("max_stats", "fiveStatusLimit")] {
        let values: Option<Vec<i64>> = ["speed", "stamina", "power", "guts", "wiz"].iter()
            .map(|key| raw[source][*key].as_i64().filter(|v| *v >= 0)).collect();
        if let Some(v) = values { base.insert(target.into(), json!(v)); }
    }
    if let Some(cards) = raw["support_cards"].as_array().filter(|v| v.len() == 6) {
        let mut ordered = cards.iter().collect::<Vec<_>>(); ordered.sort_by_key(|v| v["position"].as_u64());
        let ids: Option<Vec<u64>> = ordered.iter().enumerate().map(|(i, v)| {
            if v["position"].as_u64() != Some(i as u64 + 1) { return None; }
            let id = v["support_card_id"].as_u64().filter(|id| *id > 0)?;
            let rank = v["limit_break_count"].as_u64().filter(|r| *r <= 4)?;
            id.checked_mul(10)?.checked_add(rank)
        }).collect();
        if let Some(ids) = ids { base.insert("cardId".into(), json!(ids)); }
    }
    copy_nonnegative(raw["ramen"].get("checkpoint_pt"), "scenario_pt", &mut ramen);
    copy_nonnegative(raw["ramen"].get("special_feeling_num"), "special_feeling", &mut ramen);
    for field in base.keys() { provenance.insert(format!("state.baseGame.{field}"), json!("legacy_runtime_observation")); }
    for field in ramen.keys() { provenance.insert(format!("state.ramen.{field}"), json!("legacy_runtime_observation")); }
    json!({"schema_version":1,"run_id":null,"stage":"unknown","collector_version":version,
        "game_version":null,"capture_coherence":"unverified","state":{"baseGame":base,"ramen":ramen},
        "continuation":{},"event":null,"provenance":provenance,"capture_error":raw.get("error"),
        "display_summary":{"scenario":raw.get("scenario"),"stats":raw.get("stats"),
            "max_stats":raw.get("max_stats"),"trainings":raw.get("trainings"),"ramen":raw.get("ramen"),
            "support_cards":raw.get("support_cards"),"last_action":raw.get("last_action"),
            "turn_observation":{"raw_total_turn_num":raw.get("raw_total_turn_num"),
                "raw_total_turn_num_source":raw.get("raw_total_turn_num_source"),
                "raw_field_mapping":raw.get("raw_field_mapping"),"ui_turn_semantics":raw.get("ui_turn_semantics"),
                "turn":raw.get("turn"),"year":raw.get("year"),
                "turn_source":raw.get("turn_source"),"year_source":raw.get("year_source")}},
        "limitations":["Legacy summary lacks verified run identity and atomic stage snapshot; decision disabled."]})
}

fn copy_nonnegative(value: Option<&Value>, key: &str, out: &mut Map<String, Value>) {
    if let Some(v) = value.and_then(Value::as_i64).filter(|v| *v >= 0) { out.insert(key.into(), json!(v)); }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stale_ticket_and_cached_envelope_cannot_gain_a_new_sequence() {
        let mut p = Publisher::new("boot-a").unwrap();
        let old = p.begin_capture().unwrap();
        let new = p.begin_capture().unwrap();
        let current = p.finish_summary(new, "{}", "test", 100).unwrap().clone();
        assert!(p.finish_summary(old, "{}", "test", 200).is_err());
        let retry = p.begin_capture().unwrap();
        assert!(p.finish_capture(retry, current.clone(), 300).is_err());
        assert_eq!(p.current(), Some(&current));
        assert_eq!(p.capabilities("test")["snapshot_schema_versions"], json!([2]));
    }
    #[test]
    fn instance_ids_are_path_safe_and_tickets_cannot_cross_publishers() {
        for invalid in ["", ".", "..", "legacy-v1", "a/b", "a b", "中文"] {
            assert!(Publisher::new(invalid).is_err());
        }
        assert!(Publisher::new(&"a".repeat(129)).is_err());
        let mut a = Publisher::new("a").unwrap();
        let mut b = Publisher::new("b").unwrap();
        let ticket = a.begin_capture().unwrap();
        assert!(b.finish_summary(ticket, "{}", "test", 100).is_err());
    }
    #[test]
    fn restart_has_distinct_instance_identity() {
        let mut a = Publisher::default();
        let mut b = Publisher::default();
        let av = a.observe_summary("{}", "test", 100).unwrap().clone();
        let bv = b.observe_summary("{}", "test", 101).unwrap().clone();
        assert!(av["collector_instance_id"].is_string());
        assert_ne!(av["collector_instance_id"], bv["collector_instance_id"]);
    }
    #[test]
    fn late_capture_cannot_overwrite_new_snapshot() {
        let mut p = Publisher::default();
        p.observe_summary(r#"{"stats":{"vital":70}}"#, "test", 100).unwrap();
        p.observe_summary(r#"{"stats":{"vital":80}}"#, "test", 300).unwrap();
        let latest = p.current().cloned();
        assert!(p.observe_summary(r#"{"stats":{"vital":70}}"#, "test", 100).is_err());
        assert_eq!(p.current(), latest.as_ref());
    }
    #[test]
    fn legacy_never_invents_identity_turn_or_inventory_order() {
        let v = from_legacy_summary(&json!({"scenario":"Ramen","turn":31,"ramen":{"sozai":[1,2,3]}}), "test");
        assert!(v["run_id"].is_null()); assert!(v["state"]["baseGame"].get("turn").is_none());
        assert!(v["state"]["ramen"].get("feeling_stock").is_none());
        assert!(missing_fields(&v).contains(&"capture_coherence".to_string()));
    }
    #[test]
    fn v2_preserves_raw_turn_evidence_without_promoting_readiness() {
        let raw = json!({"scenario":"Ramen","turn":null,"year":null,"raw_total_turn_num":48,
            "raw_total_turn_num_source":"WorkSingleModeData._totalTurnNum:obscured_int_offset_68",
            "raw_field_mapping":"unverified","ui_turn_semantics":"countdown",
            "turn_source":"unknown","year_source":"unknown"});
        let mut publisher = Publisher::new("turn-evidence-test").unwrap();
        let value = publisher.observe_summary(&raw.to_string(), "test", 100).unwrap();
        assert_eq!(value["schema_version"], 2);
        let evidence = &value["display_summary"]["turn_observation"];
        for key in ["raw_total_turn_num","raw_total_turn_num_source","raw_field_mapping",
                    "ui_turn_semantics","turn","year","turn_source","year_source"] {
            assert_eq!(evidence[key], raw[key], "missing or altered raw evidence: {key}");
        }
        assert_eq!(value["ready"], false);
        assert_eq!(value["capture_coherence"], "unverified");
        assert!(value["state"]["baseGame"].get("turn").is_none());
        assert!(missing_fields(value).contains(&"state.baseGame.turn".to_string()));
        let absent = from_legacy_summary(&json!({"scenario":"Ramen"}), "test");
        assert!(absent["display_summary"]["turn_observation"]["raw_total_turn_num"].is_null());
    }
    #[test]
    fn duplicate_polling_preserves_sequence_and_time() {
        let mut p = Publisher::default();
        let first = p.observe_summary(r#"{"scenario":"Ramen","stats":{"vital":70}}"#, "test", 100).unwrap().clone();
        let duplicate = p.observe_summary(r#"{"stats":{"vital":70},"scenario":"Ramen"}"#, "test", 200).unwrap().clone();
        assert_eq!(first, duplicate);
        let updated = p.observe_summary(r#"{"scenario":"Ramen","stats":{"vital":71}}"#, "test", 300).unwrap();
        assert_eq!(updated["snapshot_id"], 2); assert_eq!(updated["captured_at_ms"], 300);
    }
    #[test]
    fn shared_publication_deduplicates_without_replacing_reader_allocation() {
        let mut publisher = Publisher::new("shared-reader-test").unwrap();
        publisher.observe_summary(r#"{"stats":{"vital":70}}"#, "test", 100).unwrap();
        let first = publisher.shared_current().unwrap();
        publisher.observe_summary(r#"{"stats":{"vital":70}}"#, "test", 200).unwrap();
        assert!(Arc::ptr_eq(&first, &publisher.shared_current().unwrap()));
        assert_eq!(first["captured_at_ms"], 100);
        let stale = publisher.begin_capture().unwrap();
        let newest = publisher.begin_capture().unwrap();
        publisher.finish_summary(newest, r#"{"stats":{"vital":80}}"#, "test", 300).unwrap();
        let second = publisher.shared_current().unwrap();
        assert!(!Arc::ptr_eq(&first, &second));
        assert_eq!(first["snapshot_id"], 1);
        assert_eq!(first["state"]["baseGame"]["vital"], 70);
        assert_eq!(second["snapshot_id"], 2);
        assert!(publisher.finish_summary(stale, "{}", "test", 400).is_err());
        assert!(Arc::ptr_eq(&second, &publisher.shared_current().unwrap()));
    }
    #[test]
    fn unknown_break_count_is_not_max_rank() {
        let cards = (1..=6).map(|i| json!({"position":i,"support_card_id":30242,"limit_break_count":-1})).collect::<Vec<_>>();
        let v = from_legacy_summary(&json!({"support_cards":cards}), "test");
        assert!(v["state"]["baseGame"].get("cardId").is_none());
    }
    #[test]
    fn ready_flag_does_not_bypass_validation() {
        let mut p = Publisher::default();
        let v = p.publish(json!({"ready":true,"missing_fields":[],"schema_version":1}), 10).unwrap();
        assert_eq!(v["ready"], false); assert!(!v["missing_fields"].as_array().unwrap().is_empty());
    }
    #[test]
    fn malformed_observation_preserves_previous_snapshot() {
        let mut p = Publisher::default(); p.observe_summary("{}", "test", 10).unwrap();
        let previous = p.current().cloned();
        assert!(p.observe_summary("not-json", "test", 11).is_err()); assert_eq!(p.current(), previous.as_ref());
    }
    #[test]
    fn producer_semantic_veto_survives_publication() {
        let mut p = Publisher::default();
        let value = p.publish(json!({"ready":false,"missing_fields":["partner_id_mapping_unverified"]}), 1).unwrap();
        assert_eq!(value["ready"], false);
        let fields = value["missing_fields"].as_array().unwrap();
        assert!(fields.contains(&json!("partner_id_mapping_unverified")));
        assert!(fields.contains(&json!("producer_not_ready")));
    }
    #[test]
    fn non_event_requires_explicit_null_event() {
        let without = json!({"stage":"train"});
        assert!(missing_fields(&without).contains(&"event".into()));
        let with = json!({"stage":"train","event":null});
        assert!(!missing_fields(&with).contains(&"event".into()));
    }
}
