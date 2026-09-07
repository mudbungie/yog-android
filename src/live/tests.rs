//! One tail or none, and never two — and the window beside it.
//!
//! The fixtures and the tail's own rules are here; [`window`] holds the half
//! REMOTE §5.5 added, on the seam the module itself draws: what the lane says
//! about the PROSE, and what it says about the machine.

mod window;

use super::settled;
use crate::codec::{Block, Call, Entry, EntryKind, Flight, Stream};

/// The flight the engine states while a model call is streaming, which is the
/// only one that opens the prose half (REMOTE §5.5).
const SPEAKING: Option<Flight> = Some(Flight::Inference);
/// A step live with its tools running: the window's flight, and not the
/// prose's.
const RUNNING: Option<Flight> = Some(Flight::Tools);

fn delivered(name: &str) -> Entry {
    Entry {
        name: name.to_owned(),
        raw: String::new(),
        kind: EntryKind::Delivered {
            sender: "op".to_owned(),
            sender_name: None,
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
        tools: Vec::new(),
    }
}

/// A stream holding one window transition apiece, in the order they landed.
fn windowing(events: Vec<Call>) -> Stream {
    Stream {
        tools: events,
        ..Stream::default()
    }
}

/// The opening transition of one call.
fn opened(tool_use: &str, tool: &str) -> Call {
    Call {
        tool_use: tool_use.to_owned(),
        tool: Some(tool.to_owned()),
        input: Some("{\"command\":\"uptime\"}".to_owned()),
        exit_code: None,
    }
}

/// Its closing transition, which restates nothing.
fn closed(tool_use: &str, exit_code: i64) -> Call {
    Call {
        tool_use: tool_use.to_owned(),
        tool: None,
        input: None,
        exit_code: Some(exit_code),
    }
}

/// A committed model turn that called `tool_use` — the record catching up.
fn calling(name: &str, tool_use: &str) -> Entry {
    Entry {
        name: name.to_owned(),
        raw: String::new(),
        kind: EntryKind::Model {
            model_id: "m".to_owned(),
            blocks: vec![Block::ToolUse {
                id: tool_use.to_owned(),
                name: "box2_Bash".to_owned(),
                input: "{}".to_owned(),
            }],
            usage: serde_json::json!({}),
        },
    }
}

fn kinds(entries: &[Entry]) -> Vec<String> {
    entries
        .iter()
        .map(|entry| match &entry.kind {
            EntryKind::Streaming { text, .. } => format!("streaming:{text}"),
            EntryKind::Windowed {
                tool, exit_code, ..
            } => format!("window:{tool}:{exit_code:?}"),
            EntryKind::Model { .. } => "model".to_owned(),
            // The only other kind these fixtures build is a delivered
            // message, so the catch-all IS that arm rather than an arm
            // nothing reaches.
            _ => "delivered".to_owned(),
        })
        .collect()
}

/// The lane's read replaces the read's own tail — one row, the fresher text.
#[test]
fn the_lane_replaces_the_transcripts_own_tail() {
    let read = vec![delivered("001"), tail("mulling", "half")];
    let out = settled(read, Some(&stream("mulling", "half and more")), SPEAKING);
    assert_eq!(kinds(&out), ["delivered", "streaming:half and more"]);
}

/// With no lane read yet, the transcript's own tail is what there is — the
/// cadence copy, untouched.
#[test]
fn with_nothing_read_the_transcripts_tail_stands() {
    let read = vec![delivered("001"), tail("", "half")];
    let out = settled(read, None, SPEAKING);
    assert_eq!(kinds(&out), ["delivered", "streaming:half"]);
}

/// **The flight-end path**: at rest there is no tail, whatever the read still
/// carries — that is the settled reply painting twice.
#[test]
fn a_conversation_at_rest_has_no_tail_however_the_read_reads() {
    let read = vec![delivered("001"), tail("", "the whole answer")];
    assert_eq!(kinds(&settled(read.clone(), None, None)), ["delivered"]);
    let held = settled(read, Some(&stream("", "the whole answer")), None);
    assert_eq!(kinds(&held), ["delivered"], "a held fold is not a tail");
}

/// A lane read that has landed nothing yet paints no row: an answer that has
/// begun and said nothing is not something to make a row for.
#[test]
fn an_empty_fold_is_no_row() {
    let out = settled(vec![delivered("001")], Some(&Stream::default()), SPEAKING);
    assert_eq!(kinds(&out), ["delivered"]);
}

/// Two tails in one read — a shape the engine does not write. With a fold to
/// replace them the rule leaves exactly one; with none it hands back what the
/// engine said, because inventing a reading of a shape the engine does not
/// write is worse than carrying it.
#[test]
fn a_fold_leaves_exactly_one_tail_whatever_the_read_carried() {
    let read = vec![tail("", "one"), delivered("001"), tail("", "two")];
    let out = settled(read.clone(), Some(&stream("", "fresh")), SPEAKING);
    assert_eq!(kinds(&out), ["delivered", "streaming:fresh"]);
    assert_eq!(
        kinds(&settled(read, None, SPEAKING)),
        ["streaming:one", "delivered", "streaming:two"]
    );
}
