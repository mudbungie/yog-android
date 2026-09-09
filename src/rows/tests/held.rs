//! **The parked call's row** (bl-8c94): the transcript's own account of the
//! call the composer's band is asking about.
//!
//! The complaint this answers had the two surfaces disagreeing — the band
//! under the composer said *held* and printed the whole sentence, while the
//! transcript row for the same call still said *running* in the working
//! accent. One call, two readings, and the one an operator was looking at was
//! the wrong one.

use std::collections::BTreeSet;

use super::{SPEAKER, call, delivered, model, text};
use crate::codec::{Entry, Held};
use crate::rows::{AutoExpand, Row, Tone, rows};

/// The engine's own sentence shape: the tool, a clip of the call's JSON input,
/// the class clause, and the evidence.
const REASON: &str = "Bash {\"command\":\"rm -rf build\"} classified destructive (rm on a path \
                      inside the writable root)";

fn held(tool_use: &str) -> Held {
    Held {
        tool_use: tool_use.to_owned(),
        tool: "Bash".to_owned(),
        reason: REASON.to_owned(),
    }
}

fn go(entries: &[Entry], parked: Option<&Held>) -> Vec<Row> {
    rows(
        entries,
        SPEAKER,
        parked,
        AutoExpand::default(),
        &BTreeSet::new(),
    )
}

fn a_call() -> Vec<Entry> {
    vec![
        delivered("001", "user", "clean the tree"),
        model(
            "002",
            vec![call("toolu_1", "Bash", "{\"command\":\"rm -rf build\"}")],
        ),
    ]
}

/// **The row the queue names is the held row**, and it wears the one hue that
/// means *asking for you* (STYLE.md §3). Unmarked, the very same transcript
/// reads as work in flight — which is what an operator saw while the band was
/// asking them to answer it.
#[test]
fn the_parked_call_becomes_the_held_row() {
    let plain = go(&a_call(), None);
    assert_eq!(plain[1].prefix, "⚙ Bash — running");
    assert_eq!(plain[1].tone, Tone::InFlight);

    let marked = go(&a_call(), Some(&held("toolu_1")));
    assert_eq!(marked.len(), plain.len(), "the mark adds no row");
    assert_eq!(marked[1].key, plain[1].key, "and moves none");
    assert_eq!(marked[1].prefix, "⚙ Bash — held for you");
    assert_eq!(marked[1].tone, Tone::Held);
}

/// **The decision leads and the command follows it under the fold** — the
/// preview/body split doing what it is for. The clip of the input the engine
/// wrote into its own sentence is gone: the row already carries the input
/// whole, and two truncations of one string are two answers to one question.
#[test]
fn the_held_row_leads_with_the_decision_and_folds_onto_the_command() {
    let marked = go(&a_call(), Some(&held("toolu_1")));
    assert_eq!(
        marked[1].preview,
        "classified destructive (rm on a path inside the writable root)"
    );
    assert!(marked[1].body.ends_with("{\"command\":\"rm -rf build\"}"));
    assert_eq!(
        marked[1].body.matches("rm -rf build").count(),
        1,
        "the sentence's own clip of the input is not a second copy of it"
    );
    assert!(marked[1].expanded, "a row asking for you is the show");
}

/// **A call the transcript has not committed yet marks nothing.** The queue
/// read and the transcript read arrive on their own cadences, so the pairing
/// can be one frame apart in either direction; an unmatched id is the honest
/// answer and never a row invented to hold it.
#[test]
fn a_hold_naming_an_uncommitted_call_marks_nothing() {
    let marked = go(&a_call(), Some(&held("toolu_other")));
    assert_eq!(marked[1].prefix, "⚙ Bash — running");
    assert_eq!(marked[1].tone, Tone::InFlight);
}

/// **A turn holding a parked call keeps its steps on screen.** The rollup's
/// second condition is that nothing in the turn is the show
/// (`rows::showing`), and a held call is — so the mark has to land before the
/// grouping or the one row asking for an answer would be folded inside an
/// aggregate line.
#[test]
fn a_turn_with_a_parked_call_does_not_roll_up() {
    let turn = vec![
        delivered("001", "user", "clean the tree"),
        model("002", vec![call("toolu_1", "Bash", "{}")]),
        model("003", vec![text("done")]),
    ];
    let marked = go(&turn, Some(&held("toolu_1")));
    let prefixes: Vec<&str> = marked.iter().map(|row| row.prefix.as_str()).collect();
    assert!(
        prefixes.contains(&"⚙ Bash — held for you"),
        "the parked call is on the glass: {prefixes:?}"
    );
}
