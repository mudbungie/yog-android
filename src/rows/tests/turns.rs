//! The rollup: where a turn is, when it folds, and what happens to the step
//! rows when it does.

use std::collections::BTreeSet;

use super::{
    SPEAKER, call, compacted, delivered, go, go_open, model, prefixes, raw, result, text, thought,
};
use crate::codec::Entry;
use crate::rows::{AutoExpand, Fold, RowClass, Tone, rows};

/// One finished turn: a message in, machinery, then the agent talking.
fn a_turn() -> Vec<Entry> {
    vec![
        delivered("001", "user", "go"),
        model("002", vec![thought("hmm"), call("t1", "Read", "{}")]),
        result("003", "t1", "contents", false),
        model("004", vec![text("done")]),
    ]
}

const AGGREGATE: &str = "⚙ 1 inference call · 1 tool call · 1 thinking block";

#[test]
fn a_finished_turn_folds_its_machinery_to_one_line() {
    assert_eq!(prefixes(&go(&a_turn())), ["user:", AGGREGATE, "yog:"]);
}

#[test]
fn a_shut_aggregate_omits_its_steps_rather_than_hiding_them() {
    let out = go(&a_turn());
    assert_eq!(out.len(), 3);
    assert!(!out[1].expanded);
    assert!(!out.iter().any(|row| row.prefix == "thinking:"));
}

#[test]
fn an_open_aggregate_re_emits_every_step_after_it() {
    assert_eq!(
        prefixes(&go_open(&a_turn())),
        [
            "user:",
            AGGREGATE,
            "thinking:",
            "⚙ Read",
            "✔ tool result — ok",
            "yog:",
        ]
    );
}

#[test]
fn an_override_opens_one_turn_without_touching_the_knob() {
    let folds: BTreeSet<String> = ["tx/002#turn".to_string()].into_iter().collect();
    let out = rows(&a_turn(), SPEAKER, AutoExpand::default(), &folds);
    assert_eq!(prefixes(&out).len(), 6);
    assert!(out[1].expanded);
}

#[test]
fn the_aggregate_is_machinery_with_nothing_of_its_own_to_show() {
    let out = go(&a_turn());
    assert_eq!(out[1].key, "tx/002#turn");
    assert_eq!(out[1].preview, "");
    assert_eq!(out[1].body, "");
    assert_eq!(out[1].class, RowClass::Other);
    assert_eq!(out[1].tone, Tone::Weak);
    assert_eq!(out[1].role, None);
    assert_eq!(out[1].fold, Fold::Steps);
    assert_eq!(
        out[1].hover,
        "what the agent did before answering — open it for each step"
    );
}

#[test]
fn a_delivered_message_bounds_the_turns_on_either_side() {
    let entries = [
        delivered("001", "user", "go"),
        model("002", vec![thought("a")]),
        model("003", vec![text("first")]),
        delivered("004", "user", "again"),
        model("005", vec![thought("b")]),
        model("006", vec![text("second")]),
    ];
    let one = "⚙ 1 inference call · 1 thinking block";
    assert_eq!(
        prefixes(&go(&entries)),
        ["user:", one, "yog:", "user:", one, "yog:"]
    );
}

#[test]
fn a_compaction_mark_is_a_boundary_and_is_never_swallowed() {
    let entries = [
        model("001", vec![thought("a")]),
        compacted("002", 1, 1, ""),
        model("003", vec![text("hi")]),
    ];
    assert_eq!(
        prefixes(&go(&entries)),
        ["thinking:", "✂ 1 entry compacted away — 001", "yog:"]
    );
}

#[test]
fn a_turn_that_ended_on_machinery_keeps_its_steps_on_screen() {
    let entries = [
        model("001", vec![thought("a")]),
        model("002", vec![thought("b")]),
    ];
    assert_eq!(prefixes(&go(&entries)), ["thinking:", "thinking:"]);
}

#[test]
fn anything_in_flight_makes_the_whole_turn_the_show() {
    let entries = [
        model("001", vec![thought("a"), call("t1", "Read", "{}")]),
        model("002", vec![text("done")]),
    ];
    assert_eq!(
        prefixes(&go(&entries)),
        ["thinking:", "⚙ Read — running", "yog:"]
    );
}

#[test]
fn a_run_with_no_inference_call_is_not_a_turn() {
    let entries = [raw("001", "noise"), model("002", vec![text("hi")])];
    assert_eq!(prefixes(&go(&entries)), ["001", "yog:"]);
}
