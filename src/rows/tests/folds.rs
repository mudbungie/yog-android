//! The expansion rule: class knob XOR explicit override, with in-flight
//! overriding the knob in one direction only.

use std::collections::BTreeSet;

use super::{THOUGHT, call, delivered, go, model, result, text, thought};
use crate::codec::Entry;
use crate::rows::{AutoExpand, Row, rows};

fn under(entries: &[Entry], auto: AutoExpand, folds: &[&str]) -> Vec<Row> {
    let folds: BTreeSet<String> = folds.iter().map(|key| (*key).to_string()).collect();
    rows(entries, None, auto, &folds)
}

fn shut_others() -> AutoExpand {
    AutoExpand {
        responses: false,
        others: false,
        fold: true,
    }
}

#[test]
fn the_defaults_open_the_conversation_and_fold_the_machinery() {
    let out = go(&[
        delivered("001", "user", "hi"),
        model("002", vec![thought("t")]),
    ]);
    assert_eq!(out[0].prefix, "user");
    assert!(out[0].expanded);
    assert_eq!(out[1].prefix, THOUGHT);
    assert!(!out[1].expanded);
}

#[test]
fn an_override_flips_whichever_way_the_knob_points() {
    let entries = [
        delivered("001", "user", "hi"),
        model("002", vec![thought("t")]),
    ];
    let out = under(&entries, AutoExpand::default(), &["tx/001#0", "tx/002#0"]);
    assert!(!out[0].expanded);
    assert!(out[1].expanded);
}

#[test]
fn a_knob_turned_off_folds_the_conversation_too() {
    let out = under(&[delivered("001", "user", "hi")], shut_others(), &[]);
    assert!(!out[0].expanded);
}

#[test]
fn a_step_in_flight_opens_against_a_shut_knob() {
    let entries = [model("001", vec![call("t1", "Read", "{}")])];
    let out = under(&entries, shut_others(), &[]);
    assert!(out[0].expanded);

    // The override still flips it: in-flight sets the auto-state, not the answer.
    let out = under(&entries, shut_others(), &["tx/001#0"]);
    assert!(!out[0].expanded);
}

#[test]
fn a_working_that_does_not_fold_itself_stands_open_to_its_steps() {
    let turn = [
        delivered("001", "user", "go"),
        model("002", vec![thought("hmm"), call("t1", "Read", "{}")]),
        result("003", "t1", "contents", false),
        model("004", vec![text("done")]),
    ];
    let unfolded = AutoExpand {
        fold: false,
        ..AutoExpand::default()
    };
    let out = under(&turn, unfolded, &[]);
    assert!(out[1].expanded, "{}", out[1].prefix);
    assert!(!under(&turn, AutoExpand::default(), &[])[1].expanded);
    // The operator's own flip still wins: an override shuts it again.
    assert!(!under(&turn, unfolded, &["tx/002#turn"])[1].expanded);
}
