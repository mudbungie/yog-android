//! Which lines stand on one bubble, and whose (DESIGN §23 item 7).

use super::{call, delivered, go, go_open, model, result, streaming, text, thought, wounded};
use crate::rows::{Block, blocks, lines};
use crate::theme::bubble::Bubble;
use std::collections::BTreeSet;

fn shape(entries: &[crate::codec::Entry], open: bool) -> Vec<(Bubble, usize)> {
    let rows = if open { go_open(entries) } else { go(entries) };
    blocks(&lines(&rows, &BTreeSet::new()))
        .into_iter()
        .map(|Block { bubble, lines }| (bubble, lines))
        .collect()
}

#[test]
fn a_finished_turn_is_the_operators_the_trail_and_the_answer() {
    let turn = [
        delivered("001", "user", "go"),
        model("002", vec![thought("hmm"), call("t1", "Read", "{}")]),
        result("003", "t1", "contents", false),
        model("004", vec![text("done")]),
    ];
    assert_eq!(
        shape(&turn, false),
        [
            (Bubble::Operator, 1),
            (Bubble::Trail, 1),
            (Bubble::Final, 1)
        ]
    );
    // Open, the head and every step under it stand on one trail.
    assert_eq!(shape(&turn, true)[1], (Bubble::Trail, 3));
}

#[test]
fn a_remark_is_a_step_of_its_working_and_the_live_tail_is_intermediate() {
    let turn = [
        delivered("001", "user", "go"),
        model("002", vec![call("t1", "Read", "{}")]),
        result("003", "t1", "contents", false),
        model("004", vec![text("checking")]),
        model("005", vec![call("t2", "Bash", "{}")]),
        result("006", "t2", "ok", false),
        model("007", vec![text("done")]),
        delivered("008", "user", "more"),
        streaming("009", "", "so far"),
    ];
    let said = shape(&turn, true);
    let bubbles: Vec<Bubble> = said.iter().map(|(bubble, _)| *bubble).collect();
    // The turn's own remark between two calls is a step of its working
    // (§22 item 4) and a met call is one row, so the open turn is one trail
    // of three steps; the live
    // tail is prose whose answer is not known yet.
    assert_eq!(
        bubbles,
        [
            Bubble::Operator,
            Bubble::Trail,
            Bubble::Final,
            Bubble::Operator,
            Bubble::Intermediate,
        ]
    );
    assert_eq!(said[1].1, 4);
    assert_eq!(said.iter().map(|(_, n)| n).sum::<usize>(), 8);
}

#[test]
fn what_nobody_in_particular_said_stands_on_the_surface() {
    let out = shape(&[wounded("001", "refused", None, None)], false);
    assert_eq!(out, [(Bubble::Intermediate, 1)]);
}
