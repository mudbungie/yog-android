//! The order and the stamp, and the two clock disagreements each has to
//! survive — plus the tree the order is taken over.

use super::{indent, now_unix, ordered, stamp};
use crate::codec::{AgentState, ConvRow, Tone};

fn under(root_id: &str, last_active_unix: i64, depth: usize) -> ConvRow {
    ConvRow {
        depth,
        ..row(root_id, last_active_unix)
    }
}

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

/// The order is taken over SUBTREES, so a child never leaves its parent — the
/// defect this replaced put a root's two dispatched children among the
/// operator's own conversations, indistinguishable from them.
#[test]
fn a_subtree_moves_whole_and_keeps_the_engines_descent() {
    let listed = ordered(vec![
        row("quiet", 500),
        row("root", 100),
        under("first-child", 900, 1),
        under("second-child", 400, 1),
        row("busy", 700),
    ]);
    let names: Vec<&str> = listed.iter().map(|r| r.root_id.as_str()).collect();
    assert_eq!(
        names,
        ["busy", "quiet", "root", "first-child", "second-child"]
    );
}

/// A child's own stamp never lifts its subtree: the root row already carries
/// when the SUBTREE last acted, so ordering by the child would count one fact
/// twice and put a busy tree in two places at once.
#[test]
fn a_child_does_not_order_the_tree_it_hangs_under() {
    let listed = ordered(vec![
        row("root", 100),
        under("child", 9_000, 1),
        row("later", 200),
    ]);
    let names: Vec<&str> = listed.iter().map(|r| r.root_id.as_str()).collect();
    assert_eq!(names, ["later", "root", "child"]);
}

/// An answer whose first row is already deep — a read that did not carry the
/// root — is a subtree of its own rather than a member of nothing.
#[test]
fn a_row_with_no_root_above_it_opens_its_own_subtree() {
    let listed = ordered(vec![under("orphan", 100, 2), row("root", 300)]);
    let names: Vec<&str> = listed.iter().map(|r| r.root_id.as_str()).collect();
    assert_eq!(names, ["root", "orphan"]);
}

/// The indent is bounded and monotone: a step per rung until the cap, and the
/// cap holds for anything deeper, because past it a phone's list is unreadable
/// whatever the number says.
#[test]
fn the_indent_steps_per_rung_and_stops_at_the_cap() {
    // Bit patterns rather than `==`: every value here is a sum of one exactly
    // representable constant, so equality is the right question, and
    // `float_cmp` is right about every case that is not this one.
    let points = |depth: usize| indent(depth).to_bits();
    assert_eq!(points(0), 0.0_f32.to_bits());
    assert_eq!(points(1), 16.0_f32.to_bits());
    assert_eq!(points(3), 48.0_f32.to_bits());
    assert_eq!(points(8), 128.0_f32.to_bits());
    assert_eq!(points(9), points(8));
    assert_eq!(points(usize::MAX), points(8));
}

/// **The machines roster's own spelling** (REMOTE §5): an age in the units
/// every other surface speaks, and a row that never dialled saying so in
/// words rather than as a missing line.
#[test]
fn a_machine_says_when_it_last_spoke_or_that_it_never_has() {
    assert_eq!(
        super::spoke(Some(1_700_000_000), 1_700_000_180),
        "last spoke 3m"
    );
    assert_eq!(
        super::spoke(Some(1_700_000_000), 1_700_000_000),
        "last spoke now"
    );
    assert_eq!(
        super::spoke(None, 1_700_000_000),
        "never dialled — minted and never used"
    );
}
