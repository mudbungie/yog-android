//! One tail or none, and never two.

use super::settled;
use crate::codec::{Entry, EntryKind, Stream};

fn delivered(name: &str) -> Entry {
    Entry {
        name: name.to_owned(),
        raw: String::new(),
        kind: EntryKind::Delivered {
            sender: "op".to_owned(),
            epitaph: None,
            body: "go".to_owned(),
        },
    }
}

fn tail(thinking: &str, text: &str) -> Entry {
    Entry {
        name: "streaming".to_owned(),
        raw: String::new(),
        kind: EntryKind::Streaming {
            thinking: thinking.to_owned(),
            text: text.to_owned(),
        },
    }
}

fn stream(thinking: &str, text: &str) -> Stream {
    Stream {
        delta: Some("text".to_owned()),
        thinking: (!thinking.is_empty()).then(|| thinking.to_owned()),
        text: (!text.is_empty()).then(|| text.to_owned()),
    }
}

fn kinds(entries: &[Entry]) -> Vec<String> {
    entries
        .iter()
        .map(|entry| match &entry.kind {
            EntryKind::Streaming { text, .. } => format!("streaming:{text}"),
            EntryKind::Delivered { .. } => "delivered".to_owned(),
            _ => "other".to_owned(),
        })
        .collect()
}

/// The lane's read replaces the read's own tail — one row, the fresher text.
#[test]
fn the_lane_replaces_the_transcripts_own_tail() {
    let read = vec![delivered("001"), tail("mulling", "half")];
    let out = settled(read, Some(&stream("mulling", "half and more")), true);
    assert_eq!(kinds(&out), ["delivered", "streaming:half and more"]);
}

/// With no lane read yet, the transcript's own tail is what there is — the
/// cadence copy, untouched.
#[test]
fn with_nothing_read_the_transcripts_tail_stands() {
    let read = vec![delivered("001"), tail("", "half")];
    let out = settled(read, None, true);
    assert_eq!(kinds(&out), ["delivered", "streaming:half"]);
}

/// **The flight-end path**: at rest there is no tail, whatever the read still
/// carries — that is the settled reply painting twice.
#[test]
fn a_conversation_at_rest_has_no_tail_however_the_read_reads() {
    let read = vec![delivered("001"), tail("", "the whole answer")];
    assert_eq!(kinds(&settled(read.clone(), None, false)), ["delivered"]);
    let held = settled(read, Some(&stream("", "the whole answer")), false);
    assert_eq!(kinds(&held), ["delivered"], "a held fold is not a tail");
}

/// A lane read that has landed nothing yet paints no row: an answer that has
/// begun and said nothing is not something to make a row for.
#[test]
fn an_empty_fold_is_no_row() {
    let out = settled(vec![delivered("001")], Some(&Stream::default()), true);
    assert_eq!(kinds(&out), ["delivered"]);
}

/// Two tails in one read — a shape the engine does not write. With a fold to
/// replace them the rule leaves exactly one; with none it hands back what the
/// engine said, because inventing a reading of a shape the engine does not
/// write is worse than carrying it.
#[test]
fn a_fold_leaves_exactly_one_tail_whatever_the_read_carried() {
    let read = vec![tail("", "one"), delivered("001"), tail("", "two")];
    let out = settled(read.clone(), Some(&stream("", "fresh")), true);
    assert_eq!(kinds(&out), ["delivered", "streaming:fresh"]);
    assert_eq!(
        kinds(&settled(read, None, true)),
        ["streaming:one", "delivered", "streaming:two"]
    );
}
