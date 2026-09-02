//! The one rule, and the weakness it is honest about.

use super::taken;
use crate::codec::{Entry, EntryKind};

fn delivered(body: &str) -> Entry {
    Entry {
        name: "001".to_owned(),
        raw: String::new(),
        kind: EntryKind::Delivered {
            sender: "op".to_owned(),
            epitaph: None,
            body: body.to_owned(),
        },
    }
}

fn other() -> Entry {
    Entry {
        name: "002".to_owned(),
        raw: String::new(),
        kind: EntryKind::Raw,
    }
}

#[test]
fn a_delivered_row_at_the_tail_takes_the_echo() {
    assert!(taken(&[delivered("hello")], "hello"));
    assert!(taken(&[other(), delivered("hello")], "hello"));
    // Whitespace is not the difference between a message and its echo: the
    // composer's text and the engine's row agree once trimmed.
    assert!(taken(&[delivered("hello\n")], "  hello "));
}

#[test]
fn nothing_matching_leaves_the_echo_standing() {
    assert!(!taken(&[], "hello"));
    assert!(!taken(&[other()], "hello"));
    assert!(!taken(&[delivered("goodbye")], "hello"));
    // A model turn saying the same words is not this message coming back.
    assert!(!taken(&[other(), other()], "hello"));
}

/// The tail is the window, and it is what keeps an identical message far up
/// the conversation from dissolving a fresh echo.
#[test]
fn an_identical_message_far_up_the_transcript_is_not_this_one() {
    let mut long = vec![delivered("ok")];
    long.extend((0..8).map(|_| other()));
    assert!(!taken(&long, "ok"));
    // …and once it is back inside the window, it is.
    let short = vec![delivered("ok"), other(), other()];
    assert!(taken(&short, "ok"));
}
