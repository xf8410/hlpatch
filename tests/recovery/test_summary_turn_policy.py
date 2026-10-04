"""Run the real summary formatter and AI call site with fake observed scalar values."""
from pathlib import Path
import os
import re
import subprocess
import unittest

ROOT = Path(__file__).resolve().parents[2]


class SummaryTurnPolicy(unittest.TestCase):
    def test_production_summary_preserves_raw_and_refuses_unverified_ramen_decisions(self):
        source = (ROOT / "hachimi_ura_plugin/src/lib.rs").read_text(encoding="utf-8")
        summary = source.split("unsafe fn read_summary_inner_impl() -> String {", 1)[1].split(
            "\n}\n", 1)[0]
        formatter = summary.split('log_predict_step("S:json");', 1)[1].strip()
        ai_call = summary[summary.index("let ai_json = "):summary.index("// ★ Breeders team member data")]
        # Compile the actual output expression, not a parallel test serializer. Only unrelated
        # observed inputs and the legacy evaluator are fakes; the evaluator spy catches entry.
        args = formatter.split('"#,', 1)[1].rsplit(")", 1)[0]
        names = [line.strip().rstrip(",") for line in args.splitlines() if line.strip()]
        special = {"PLUGIN_VERSION", "year", "cumulative_turn", "raw_total_turn_num", "sid",
                   "mon", "half", "scn_s", "ai_json", "summary_turn_fields(sid, raw_total_turn_num, year, cumulative_turn)",
                   'effect_ids_str.join(",")', "team_json", "ramen_json", "last_action_json"}
        declarations = "\n".join(f"let {name} = 0;" for name in names if name not in special)
        helpers = ""
        for name in ("summary_turn_fields", "summary_ai_json", "unavailable_ai_json"):
            match = re.search(r"(?m)^fn " + name + r"\(", source)
            if match:
                helpers += source[match.start():source.index("\n}", match.start()) + 2] + "\n"
        actual_evaluator = source[source.index("const STAT_EVAL_SCORE:"):source.index("unsafe fn read_chara_skills(")]
        code = r'''
#![allow(dead_code,unused_variables)]
use std::sync::atomic::{AtomicUsize,Ordering};
static CALLS:AtomicUsize=AtomicUsize::new(0);
fn evaluate_ai(_:i32,_:[i32;5],_:i32,_:i32,_:i32,_:i32,_:&Vec<()>,_:bool,_:bool,
               _:i32,_:i32,_:&Vec<()>,_:i32,_:bool)->Result<i32,&'static str> {
    CALLS.fetch_add(1,Ordering::SeqCst); Ok(1)
}
fn ai_result_to_json(_: &i32)->String { r#"{"available":true}"#.into() }
''' + helpers + "\nmod actual {\n" + actual_evaluator + r'''
#[test]fn direct_ramen_evaluator_cannot_emit_a_recommendation() {
    let gains=std::collections::HashMap::new();
    for turn in [-1,0,24,48,77] {
        let result=evaluate_ai(turn,[100;5],50,100,3,14,&[],false,false,20,1,&gains,2,false);
        assert!(matches!(result,Err("turn_mapping_unverified")));
    }
    for scenario in [1,13] {
        let result=evaluate_ai(24,[100;5],50,100,3,scenario,&[],false,false,20,1,&gains,2,false);
        assert!(result.is_ok());
    }
}
#[test]fn non_ramen_scores_match_prechange_production_results() {
    // Captured by running the prechange production evaluator, including its serializer.
    let gains=std::collections::HashMap::new();
    for (scenario,turn,best,rest) in [(1,24,299.6,274.6),(1,76,25.0,0.0),
                                     (13,24,305.0,280.0),(13,76,25.0,0.0)] {
        let result=evaluate_ai(turn,[100;5],50,100,3,scenario,&[],false,false,20,1,&gains,2,false).unwrap();
        let value:serde_json::Value=serde_json::from_str(&ai_result_to_json(&result)).unwrap();
        assert_eq!(value["score"],350); assert_eq!(value["total_stats"],500);
        assert_eq!(value["skill_eval"],20); assert_eq!(value["skill_count"],1);
        assert_eq!(value["best"],"Outgoing");
        assert_eq!(value["best_v"].as_f64(),Some(best));
        assert_eq!(value["rest"].as_f64(),Some(rest));
        assert_eq!(value["outgoing"].as_f64(),Some(best));
        assert_eq!(value["train"],serde_json::json!({}));
    }
}
}
''' + r'''
fn production_summary(sid:i32,raw_total_turn_num:i32)->serde_json::Value {
    const PLUGIN_VERSION:&str="test";
    let year=2; let cumulative_turn=raw_total_turn_num; let mon=5; let half=1;
    let scn_s=if sid==14 {"Ramen"} else {"URA"};
    let effect_ids_str:Vec<String>=vec![];
    let (team_json,ramen_json,last_action_json)=("","","");
    let eval_trainings=vec![]; let ramen_gauge_gains_map=vec![];
    let chara_effect_ids:Vec<i32>=vec![]; let mot=3; let ramen_special_feeling_num=2;
''' + declarations + "\n" + ai_call + "\nlet output = " + formatter + r''';
    serde_json::from_str(&output).expect("actual summary format must remain valid JSON")
}
#[test]fn ramen_export_policy() {
    for raw in [-1,0,1,23,24,47,48,73,77,78] {
        CALLS.store(0,Ordering::SeqCst);
        let value=production_summary(14,raw);
        assert!(value["turn"].is_null(),"unverified raw={raw} became observed turn");
        assert!(value["year"].is_null());
        assert_eq!(value["raw_total_turn_num"],raw);
        assert_eq!(value["raw_total_turn_num_source"],"WorkSingleModeData._totalTurnNum:obscured_int_offset_68");
        assert_eq!(value["raw_field_mapping"],"unverified");
        assert_eq!(value["turn_source"],"unknown");
        assert_eq!(value["year_source"],"unknown");
        assert_eq!(value["ai"]["available"],false);
        assert_eq!(value["ai"]["status"],"unavailable");
        assert_eq!(value["ai"]["reason"],"turn_mapping_unverified");
        assert_eq!(value["ai"]["collector_role"],"observation_only");
        assert_eq!(CALLS.load(Ordering::SeqCst),0,"Ramen invoked the legacy heuristic");
        let envelope=ramen_observation::from_legacy_summary(&value,"test");
        assert!(envelope["state"]["baseGame"].get("turn").is_none());
        assert!(!ramen_observation::missing_fields(&envelope).is_empty());
    }
}
#[test]fn other_scenarios_keep_legacy_evaluation() {
    for scenario in [1,13] {
        CALLS.store(0,Ordering::SeqCst);
        let value=production_summary(scenario,32);
        assert_eq!(value["turn"],32); assert_eq!(value["year"],2);
        assert_eq!(value["raw_field_mapping"],"unverified");
        assert_eq!(value["turn_source"],"legacy_unverified");
        assert_eq!(value["ai"]["available"],true);
        assert_eq!(CALLS.load(Ordering::SeqCst),1);
    }
}
'''
        cache = ROOT / ".recovery-cache/summary-turn-policy"
        cache.mkdir(parents=True, exist_ok=True)
        manifest = cache / "Cargo.toml"
        if not manifest.exists():
            manifest.write_text('[package]\nname="summary-turn-policy-tests"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n', encoding="utf-8")
            subprocess.run(["cargo", "add", "--offline", "--manifest-path", str(manifest),
                            "serde_json@1.0.151"], check=True, timeout=60)
            subprocess.run(["cargo", "add", "--offline", "--manifest-path", str(manifest),
                            "--path", str(ROOT / "ramen_observation"), "ramen_observation"], check=True, timeout=60)
        (cache / "lib.rs").write_text(code, encoding="utf-8")
        environment = dict(os.environ)
        environment["CARGO_TARGET_DIR"] = str(ROOT / ".recovery-cache/summary-turn-policy-target")
        subprocess.run(["cargo", "test", "--release", "--offline", "--locked", "--manifest-path", str(manifest),
                        "--", "--test-threads=1"], check=True, timeout=180, env=environment)


if __name__ == "__main__":
    unittest.main()
