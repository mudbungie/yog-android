//! The three lines a row can have, and the fold that keeps each of them to
//! one line.

use super::{ATTENTION_MARK, lines};
use crate::codec::Tone;
use crate::roster::tests::under;

#[test]
fn a_quiet_row_is_who_and_when_and_what_it_said() {
    let mut row = under("c-1", 1_700_000_000, 0);
    row.display = "brave-fox".to_owned();
    row.preview = "first line".to_owned();
    assert_eq!(
        lines(&row, 1_700_003_600),
        ["brave-fox \u{b7} 1h", "first line"]
    );
}

/// An empty preview is a row with nothing to quote, not a blank line: the
/// list is shorter by exactly the rows that have said nothing.
#[test]
fn a_row_that_has_said_nothing_is_one_line() {
    let row = under("c-2", 0, 0);
    assert_eq!(lines(&row, 0), ["c-2 \u{b7} now"]);
}

#[test]
fn a_waiting_row_carries_the_mark_before_the_stamp() {
    let mut row = under("c-3", 0, 0);
    row.attention = 2;
    let listed = lines(&row, 0);
    assert_eq!(listed, [format!("c-3{ATTENTION_MARK} \u{b7} now")]);
}

/// The defect this fold exists for: a provider clause with hard breaks in it
/// took four lines of the list. One line in, one line out, whatever the
/// engine's words did.
#[test]
fn a_failure_is_one_line_however_the_provider_wrote_it() {
    let mut row = under("c-4", 0, 0);
    row.tone = Tone::Bad;
    row.preview = "asked\nfor  a  branch".to_owned();
    row.failure = Some("no tool call found\nfor call_id z1\n".to_owned());
    assert_eq!(
        lines(&row, 0),
        [
            "c-4 \u{b7} now",
            "asked for a branch",
            "failed \u{b7} no tool call found for call_id z1"
        ]
    );
}

/// A reddened row with no clause says nothing extra — the third thing it is.
#[test]
fn a_bad_row_with_no_clause_gets_no_status_line() {
    let mut row = under("c-5", 0, 0);
    row.tone = Tone::Bad;
    assert_eq!(lines(&row, 0).len(), 1);
}
