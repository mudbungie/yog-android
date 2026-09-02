//! The order and the stamp, and the two clock disagreements each has to
//! survive.

use super::{now_unix, ordered, stamp};
use crate::codec::{AgentState, ConvRow, Tone};

fn row(root_id: &str, last_active_unix: i64) -> ConvRow {
    ConvRow {
        root_id: root_id.to_owned(),
        display: root_id.to_owned(),
        name: None,
        display_only: false,
        state: AgentState::Quiescent,
        uncertain: false,
        preview: String::new(),
        age_secs: 0,
        last_active_unix,
        flight: None,
        attention: 0,
        members: 1,
        direct: 0,
        stoppable: false,
        stop_children: false,
        depth: 0,
        tone: Tone::Plain,
        failure: None,
        alignment: None,
        ball: None,
    }
}

#[test]
fn the_list_is_newest_first() {
    let listed = ordered(vec![row("old", 100), row("new", 300), row("mid", 200)]);
    let names: Vec<&str> = listed.iter().map(|r| r.root_id.as_str()).collect();
    assert_eq!(names, ["new", "mid", "old"]);
}

/// The sort is stable, so rows sharing a stamp keep the engine's own descent
/// order underneath rather than being shuffled by this end.
#[test]
fn rows_sharing_a_stamp_keep_the_order_they_arrived_in() {
    let listed = ordered(vec![row("a", 100), row("b", 100), row("c", 100)]);
    let names: Vec<&str> = listed.iter().map(|r| r.root_id.as_str()).collect();
    assert_eq!(names, ["a", "b", "c"]);
    assert!(ordered(Vec::new()).is_empty());
}

#[test]
fn the_stamp_says_the_shortest_true_thing() {
    let at = 1_700_000_000;
    assert_eq!(stamp(at, at), "now");
    assert_eq!(stamp(at, at + 59), "now");
    assert_eq!(stamp(at, at + 60), "1m");
    assert_eq!(stamp(at, at + 59 * 60), "59m");
    assert_eq!(stamp(at, at + 3600), "1h");
    assert_eq!(stamp(at, at + 23 * 3600), "23h");
    assert_eq!(stamp(at, at + 24 * 3600), "1d");
    assert_eq!(stamp(at, at + 6 * 24 * 3600), "6d");
    assert_eq!(stamp(at, at + 7 * 24 * 3600), "1w");
    assert_eq!(stamp(at, at + 60 * 24 * 3600), "8w");
}

/// A device whose clock is behind the engine's does not paint a negative
/// age: the two clocks disagreeing is not a fact about the conversation.
#[test]
fn a_clock_behind_the_engines_reads_now() {
    assert_eq!(stamp(1_700_000_000, 1_699_999_000), "now");
    assert_eq!(stamp(1_700_000_000, 0), "now");
}

/// The device clock answers something an epoch second later than nothing —
/// the only claim worth making about a real clock in a test.
#[test]
fn this_devices_clock_is_after_the_epoch() {
    assert!(now_unix() > 1_600_000_000);
}
