//! **The follow lane** (REMOTE §5.5, DESIGN §14.1): a conversation that is
//! writing has its tail held open beside the pass, every frame folded onto
//! what stands, and the fold goes when the turn does.
//!
//! What the lane produces is not a field of its own but the transcript's own
//! streaming entry, freshened (bl-e3d1) — so these read the tail where the
//! frame reads it, which is the one place it exists. The lane's connection
//! is scripted positionally as a fed turn: its dial is the pass's, right
//! after the transcript read, and the test is the engine writing frames.

use std::sync::mpsc;

use super::{
    Turn, conv_flying, conv_reply, model_turns, nothing_set, ops, settle, tr_reply, ws_reply,
};
use crate::codec::EntryKind;
use serde_json::json;

/// The transcript's tail, if it carries one — where the lane's frames land.
fn tail(snap: &crate::seat::Snapshot) -> Option<(String, String)> {
    snap.transcript.iter().find_map(|entry| match &entry.kind {
        EntryKind::Streaming { thinking, text } => Some((thinking.clone(), text.clone())),
        _ => None,
    })
}

/// One follow frame: what landed since the frame before it (§5.5).
fn frame(thinking: &str, text: &str) -> Vec<u8> {
    json!({ "ok": true, "kind": "follow",
            "stream": { "delta": "text", "thinking": thinking, "text": text } })
    .to_string()
    .into_bytes()
}

/// The first pass, the preload, and the pass under a WRITING conversation —
/// then the lane's dial, which the test feeds.
fn writing() -> (Vec<Turn>, mpsc::Sender<Vec<u8>>) {
    let (feed, frames) = mpsc::channel();
    let turns = vec![
        Turn::Answer(vec![ws_reply()]),
        Turn::Answer(vec![nothing_set()]),
        Turn::Answer(vec![ws_reply()]),
        Turn::Answer(vec![conv_flying()]),
        Turn::Answer(vec![tr_reply()]),
        Turn::Feed(frames),
    ];
    (turns, feed)
}

/// The tail is held open, and **a frame is an append**: the second frame
/// carries only what landed since the first, and the glass shows the fold.
#[test]
fn a_writing_conversation_streams_its_tail_and_the_frames_fold() {
    let (turns, feed) = writing();
    let (mut model, served) = model_turns(turns);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.focus_conversation("home".into(), "a1".into());
    feed.send(frame("first I", "then this")).unwrap();
    let snap = settle(&mut model, &|s| {
        tail(s).is_some_and(|(_, text)| text == "then this")
    });
    assert_eq!(tail(&snap).unwrap_or_default().0, "first I");
    // One tail, never two: the lane replaced the read's own rather than
    // painting beside it (bl-e3d1).
    assert_eq!(
        snap.transcript
            .iter()
            .filter(|e| matches!(e.kind, EntryKind::Streaming { .. }))
            .count(),
        1
    );
    feed.send(frame("", ", and more")).unwrap();
    let snap = settle(&mut model, &|s| {
        tail(s).is_some_and(|(_, text)| text.ends_with("and more"))
    });
    assert_eq!(
        tail(&snap).unwrap_or_default(),
        ("first I".to_owned(), "then this, and more".to_owned())
    );
    // The lane is the pass's own dial, right after the transcript.
    drop(model);
    assert_eq!(
        ops(&served.join().unwrap()),
        [
            "workspaces",
            "roles",
            "workspaces",
            "conversations",
            "transcript",
            "follow"
        ]
    );
}

/// **The tail goes when the turn does.** A pass that sees the row at rest
/// hangs the lane up and clears the fold, because the finished answer
/// arrives as a transcript row and a fold left standing under it would be the
/// same words twice.
#[test]
fn a_finished_turn_drops_the_fold_it_was_writing() {
    let (mut turns, feed) = writing();
    // The next pass finds it at rest: no flight, no lane, no fold.
    turns.extend([
        Turn::Answer(vec![ws_reply()]),
        Turn::Answer(vec![conv_reply()]),
        Turn::Answer(vec![tr_reply()]),
    ]);
    let (mut model, _served) = model_turns(turns);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.focus_conversation("home".into(), "a1".into());
    feed.send(frame("", "half an answer")).unwrap();
    settle(&mut model, &|s| tail(s).is_some());
    model.focus_conversation("home".into(), "a1".into());
    // **The flight-end path** (bl-e3d1): the row states no flight, so no tail
    // paints — whatever the lane still carried.
    let snap = settle(&mut model, &|s| {
        tail(s).is_none() && !s.transcript.is_empty()
    });
    assert!(tail(&snap).is_none());
}

/// **The stream's end empties the fold, and the next pass reopens the lane
/// onto a first frame that is whole** (§5.5: a read starts holding nothing).
#[test]
fn the_streams_end_empties_the_fold_and_the_next_pass_reopens_the_lane() {
    let (mut turns, feed) = writing();
    let (again, frames) = mpsc::channel();
    turns.extend([
        Turn::Answer(vec![ws_reply()]),
        Turn::Answer(vec![conv_flying()]),
        Turn::Answer(vec![tr_reply()]),
        Turn::Feed(frames),
    ]);
    let (mut model, _served) = model_turns(turns);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.focus_conversation("home".into(), "a1".into());
    feed.send(frame("", "the first step")).unwrap();
    settle(&mut model, &|s| tail(s).is_some());
    // The step committed: the engine ends the stream.
    drop(feed);
    settle(&mut model, &|s| tail(s).is_none());
    // The next pass reopens it, and the new stream's first frame is whole.
    model.focus_conversation("home".into(), "a1".into());
    again.send(frame("", "the second step")).unwrap();
    let snap = settle(&mut model, &|s| tail(s).is_some());
    assert_eq!(tail(&snap).unwrap_or_default().1, "the second step");
}

/// A frame this build cannot read as a tail is a sentence, not a stop: the
/// lane stays held and the next frame folds as if nothing had happened. The
/// sentence stands for one pass — the next pass is what clears it.
#[test]
fn a_lane_frame_of_the_wrong_kind_reaches_the_banner_and_the_lane_goes_on() {
    let (mut turns, feed) = writing();
    turns.extend([
        Turn::Answer(vec![ws_reply()]),
        Turn::Answer(vec![conv_flying()]),
        Turn::Answer(vec![tr_reply()]),
    ]);
    let (mut model, _served) = model_turns(turns);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.focus_conversation("home".into(), "a1".into());
    feed.send(ws_reply()).unwrap();
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(
        snap.error.as_deref(),
        Some("follow: the engine answered workspaces instead")
    );
    feed.send(frame("", "recovered")).unwrap();
    let snap = settle(&mut model, &|s| tail(s).is_some());
    assert_eq!(tail(&snap).unwrap_or_default().1, "recovered");
    assert!(
        snap.error.is_some(),
        "the sentence stands until the next pass"
    );
    model.focus_conversation("home".into(), "a1".into());
    settle(&mut model, &|s| s.error.is_none() && tail(s).is_some());
}

/// **A frame from a lane no longer held is nothing** — by its id, never its
/// subject, so a new lane on the same conversation cannot absorb the old
/// stream's deltas; and the end of a lane never held is nothing too.
#[test]
fn a_frame_from_a_dropped_lane_is_ignored_by_its_id() {
    let mut standing = crate::seat::pass::Standing::default();
    let stale =
        crate::seat::lane::Framed::Frame(7, serde_json::from_slice(&frame("", "x")).unwrap());
    standing.adopted(stale);
    standing.adopted(crate::seat::lane::Framed::Over(7));
    let snap = standing.publish(&crate::seat::Focus::default());
    assert!(snap.transcript.is_empty() && snap.error.is_none());
}
