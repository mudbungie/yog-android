//! **The live lane** (REMOTE §5.5, bl-4822): a conversation that is writing
//! is asked for its tail between passes, and the tail goes when the turn
//! does. The gate is the row's own `flight`, so a test that wants the lane
//! scripts a row that is flying.
//!
//! What the lane produces is not a field of its own but the transcript's own
//! streaming entry, freshened (bl-e3d1) — so these read the tail where the
//! frame reads it, which is the one place it exists.

use super::{
    Model, cache_in, conv_flying, conv_reply, material, nothing_set, pki, serve_many, settle,
    tr_reply, ws_reply,
};
use crate::codec::EntryKind;
use crate::transport::Seat;
use serde_json::json;
use std::time::Duration;

/// The transcript's tail, if it carries one — where the lane's reads land.
fn tail(snap: &crate::seat::Snapshot) -> Option<(String, String)> {
    snap.transcript.iter().find_map(|entry| match &entry.kind {
        EntryKind::Streaming { thinking, text } => Some((thinking.clone(), text.clone())),
        _ => None,
    })
}

/// A rest short enough that a pass and a live tick both fit inside a test.
const QUICK: Duration = Duration::from_millis(300);

fn follow_reply(thinking: &str, text: &str) -> Vec<u8> {
    json!({ "ok": true, "kind": "follow",
            "stream": { "delta": "text", "thinking": thinking, "text": text } })
    .to_string()
    .into_bytes()
}

fn model_at(scripts: Vec<Vec<Vec<u8>>>) -> Model {
    let dir = pki();
    let (address, _served) = serve_many(&dir, "ca", "server", scripts);
    let seat = Seat::open(&material(&dir, "ca", "client", &address)).unwrap();
    Model::start(seat, QUICK, cache_in(&dir))
}

/// A writing conversation streams: the tail arrives between passes, and it
/// arrives as the whole answer so far — every read of this seat's is a first
/// frame, so the fold is assignment and a later read simply replaces.
#[test]
fn a_writing_conversation_streams_its_tail_between_passes() {
    let mut model = model_at(vec![
        vec![ws_reply()],
        vec![nothing_set()],
        vec![ws_reply()],
        vec![conv_flying()],
        vec![tr_reply()],
        vec![follow_reply("first I", "then this")],
        vec![ws_reply()],
        vec![conv_flying()],
        vec![tr_reply()],
        vec![follow_reply("first I", "then this, and more")],
        vec![ws_reply()],
        vec![conv_flying()],
        vec![tr_reply()],
    ]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.focus_conversation("home".into(), "a1".into());
    let snap = settle(&mut model, &|s| {
        tail(s).is_some_and(|(_, text)| text == "then this")
    });
    let (thinking, text) = tail(&snap).unwrap_or_default();
    assert_eq!(text, "then this");
    assert_eq!(thinking, "first I");
    // One tail, never two: the lane replaced the read's own rather than
    // painting beside it (bl-e3d1).
    assert_eq!(
        snap.transcript
            .iter()
            .filter(|e| matches!(e.kind, EntryKind::Streaming { .. }))
            .count(),
        1
    );
    // The next read replaces rather than appends.
    let snap = settle(&mut model, &|s| {
        tail(s).is_some_and(|(_, text)| text.ends_with("and more"))
    });
    assert_eq!(tail(&snap).unwrap_or_default().1, "then this, and more");
}

/// **The tail goes when the turn does.** A pass that sees the row at rest
/// clears the fold, because the finished answer arrives as a transcript row
/// and a fold left standing under it would be the same words twice.
#[test]
fn a_finished_turn_drops_the_fold_it_was_writing() {
    let mut model = model_at(vec![
        vec![ws_reply()],
        vec![nothing_set()],
        vec![ws_reply()],
        vec![conv_flying()],
        vec![tr_reply()],
        vec![follow_reply("", "half an answer")],
        // The next pass finds it at rest: no flight, no lane, no fold.
        vec![ws_reply()],
        vec![conv_reply()],
        vec![tr_reply()],
        vec![ws_reply()],
        vec![conv_reply()],
        vec![tr_reply()],
    ]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.focus_conversation("home".into(), "a1".into());
    settle(&mut model, &|s| tail(s).is_some());
    // **The flight-end path** (bl-e3d1): the row states no flight, so no tail
    // paints — whatever the read still carries.
    let snap = settle(&mut model, &|s| {
        tail(s).is_none() && !s.transcript.is_empty()
    });
    assert!(tail(&snap).is_none());
}

/// A live read that fails is a sentence, not a stop: the lane is one read at
/// a time and the next tick re-asks whole.
#[test]
fn a_live_read_that_fails_reaches_the_banner_and_the_lane_goes_on() {
    let mut model = model_at(vec![
        vec![ws_reply()],
        vec![nothing_set()],
        vec![ws_reply()],
        vec![conv_flying()],
        vec![tr_reply()],
        vec![ws_reply()], // the follow read, answered with a roster
        vec![ws_reply()],
        vec![conv_flying()],
        vec![tr_reply()],
        vec![follow_reply("", "recovered")],
        vec![ws_reply()],
        vec![conv_flying()],
        vec![tr_reply()],
    ]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.focus_conversation("home".into(), "a1".into());
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(
        snap.error.as_deref(),
        Some("follow: the engine answered workspaces instead")
    );
    let snap = settle(&mut model, &|s| tail(s).is_some());
    assert_eq!(tail(&snap).unwrap_or_default().1, "recovered");
}

/// The lane is the focused conversation's: with none focused there is
/// nothing to follow, and the read says so rather than dialling with a hole
/// in the envelope.
#[test]
fn a_follow_with_nothing_focused_names_itself() {
    let dir = pki();
    let (address, _served) = serve_many(&dir, "ca", "server", vec![vec![ws_reply()]]);
    let seat = Seat::open(&material(&dir, "ca", "client", &address)).unwrap();
    assert_eq!(
        crate::seat::acts::follow(&seat, &crate::seat::Focus::default()).unwrap_err(),
        "follow: no conversation is focused"
    );
}
