//! The ledger's two readings: the rule over a stated table, and the table this
//! build actually vendors.

use super::{EDITION, FLOOR, STAMPS, spells, spells_at};

/// The synthetic table is the point: `STAMPS` is empty at a major's cut, so
/// the RULE has to be asserted against a corpus that has grown rather than
/// against the one in the tree today.
const GROWN: [(&str, &str, u32); 2] = [
    ("reply/conversations", "/rows/[]/mood", 20),
    ("reply/steps", "/rows/[]/cadence", 22),
];

#[test]
fn a_path_the_engine_is_too_old_for_is_one_it_cannot_spell() {
    assert!(spells_at(
        &GROWN,
        "reply/conversations",
        "/rows/[]/mood",
        20
    ));
    assert!(spells_at(
        &GROWN,
        "reply/conversations",
        "/rows/[]/mood",
        99
    ));
    assert!(!spells_at(
        &GROWN,
        "reply/conversations",
        "/rows/[]/mood",
        19
    ));
    assert!(!spells_at(&GROWN, "reply/steps", "/rows/[]/cadence", 21));
}

/// A path the table does not name is at or below the floor, and every engine
/// of this major has it — which is why the table carries the post-floor paths
/// only and why its absence is an answer rather than a gap.
#[test]
fn a_path_at_or_below_the_floor_is_spelled_by_every_engine() {
    assert!(spells_at(&GROWN, "reply/conversations", "/rows/[]/tone", 0));
    assert!(spells_at(&[], "reply/steps", "/rows/[]/framing", 0));
}

/// The vendored corpus's own three facts, held to each other rather than to a
/// number written here: the edition is the newest stamp, so it cannot be below
/// the floor, and nothing post-floor exists at the cut.
#[test]
fn this_builds_ledger_is_its_own_corpus() {
    const { assert!(EDITION >= FLOOR) };
    for (shape, path, stamp) in STAMPS {
        assert!(*stamp > FLOOR, "{shape}{path} is not post-floor");
    }
    // Nothing is post-floor at the cut, so every path answers *yes* — for the
    // oldest engine of this major as much as for the newest.
    assert!(spells("reply/conversations", "/rows/[]/tone", FLOOR));
}
