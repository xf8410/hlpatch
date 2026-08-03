use std::{env, fs, path::PathBuf};

fn replace_once(source: &mut String, old: &str, new: &str, label: &str) {
    let count = source.matches(old).count();
    assert_eq!(count, 1, "expected exactly one {label} anchor, found {count}");
    *source = source.replacen(old, new, 1);
}

fn main() {
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=src/ramen_observation.rs");

    let mut source = fs::read_to_string("src/lib.rs").expect("read legacy runtime");

    while source.starts_with("//!") {
        let end = source.find('\n').expect("leading doc line newline");
        source.replace_range(..=end, "");
    }
    replace_once(
        &mut source,
        "#![allow(dead_code)]",
        "mod ramen_observation;",
        "crate attribute",
    );

    replace_once(
        &mut source,
        "let turn = std::cmp::min((mon - 1) * 2 + (half - 1), 71);",
        "let turn = ramen_observation::authoritative_turn_for_ai().unwrap_or(-1);",
        "calendar-derived turn",
    );

    replace_once(
        &mut source,
        "} else if path == \"/summary\" {\n        read_summary()",
        "} else if path == \"/single_mode/timeline\" {\n        ramen_observation::read_timeline_json()\n    } else if path == \"/ramen/state\" {\n        ramen_observation::read_ramen_state_json()\n    } else if path == \"/ramen/transitions\" {\n        ramen_observation::read_ramen_transitions_json()\n    } else if path == \"/summary\" {\n        read_summary()",
        "summary route",
    );

    let out = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR"));
    fs::write(out.join("hlpatch_runtime.rs"), source).expect("write generated runtime");
    fs::copy("src/ramen_observation.rs", out.join("ramen_observation.rs"))
        .expect("copy Ramen observation module");
}
