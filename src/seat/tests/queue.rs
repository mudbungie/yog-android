//! **The attention lane** (DESIGN §14.1, REMOTE §14.1): the decision queue
//! held standing from the first pass, its frames replacing the rows, its
//! end answered by the next pass, and a frame this build cannot read named
//! for what it is.
//!
//! The lane is served aside from the script by the harness — a `Feed` the
//! test writes frames into — because it stands for the seat's whole life
//! and its redial's timing against a gesture is nobody's to script.

use std::sync::mpsc;

use serde_json::json;

use super::{QUICK, REST, Turn, model_lanes, ops, queue_quiet, settle, ws_reply};

/// The queue with one conversation waiting, addressed at no conversation this
/// seat has focused: the whole-queue read names no place.
fn queue() -> Vec<u8> {
    json!({ "ok": true, "kind": "attention",
            "rows": [{ "workspace": "home", "agent": "a9", "display": "d",
                       "state": "stopped", "uncertain": false,
                       "signals": ["mail"], "says": "has mail queued", "preview": "p", "age_secs": 5,
                       "pending": 2, "held": null, "failure": null,
                       "flag": null }] })
    .to_string()
    .into_bytes()
}

/// **The queue reaches the snapshot with nothing focused and nothing asked**
/// (§14.1): the lane is dialled by the first pass, its frame is the rows, and
/// the roster's own requests are exactly what they were — the lane is not a
/// turn of the script.
#[test]
fn the_queue_is_held_standing_from_the_first_pass() {
    let (feed, frames) = mpsc::channel();
    let dir = super::pki();
    let at = super::cache_in(&dir);
    let (address, served) = super::serve_lanes(
        &dir,
        "ca",
        "server",
        vec![
            Turn::Answer(vec![ws_reply()]),
            Turn::Answer(vec![super::ws_named("away")]),
        ],
        vec![Turn::Feed(frames)],
    );
    let seat =
        crate::transport::Seat::open(&super::material(&dir, "ca", "client", &address)).unwrap();
    let mut model = super::Model::start(seat, REST, at.clone());
    settle(&mut model, &|s| !s.workspaces.is_empty());
    feed.send(queue()).unwrap();
    let snap = settle(&mut model, &|s| !s.queue.is_empty());
    assert_eq!(snap.queue[0].agent, "a9");
    assert_eq!(snap.queue[0].says, "has mail queued");
    // **The lane's frame is what the §14 cache stores for the queue**: the
    // next pass the engine answers writes it, the rows having changed.
    model.focus_workspace(None);
    settle(&mut model, &|s| s.workspaces[0].workspace == "away");
    let (_, kept, stored) = crate::cache::read(&at).unwrap();
    assert_eq!(kept.queue[0].agent, "a9");
    assert!(stored.attention.is_some());
    // **Frames replace; they never append** (REMOTE §14.1).
    feed.send(queue_quiet()).unwrap();
    settle(&mut model, &|s| s.queue.is_empty());
    drop(model);
    assert_eq!(ops(&served.join().unwrap()), ["workspaces", "workspaces"]);
}

/// **The hold's end is the next pass's to answer**: the engine ends the lane
/// at its bound, and the pass after that dials it again. Nothing reopens it
/// sooner, which is what bounds the redial rate to the cadence.
#[test]
fn a_lane_the_engine_ended_is_reopened_by_the_next_pass() {
    let (feed, frames) = mpsc::channel();
    let (mut model, _served) = model_lanes(
        vec![vec![ws_reply()]; 12],
        vec![Turn::Answer(vec![queue()]), Turn::Feed(frames)],
        QUICK,
    );
    settle(&mut model, &|s| !s.queue.is_empty());
    // The second dial is a cadence pass's; the rows it held stand meanwhile.
    feed.send(queue_quiet()).unwrap();
    let snap = settle(&mut model, &|s| s.queue.is_empty());
    assert_eq!(snap.error, None);
}

/// A lane frame this build cannot read as a queue is a sentence naming the
/// read, and the rows already held are not dropped for it.
#[test]
fn a_queue_frame_of_the_wrong_kind_names_the_read_and_keeps_what_was_there() {
    let (feed, frames) = mpsc::channel();
    let (mut model, _served) = model_lanes(vec![vec![ws_reply()]], vec![Turn::Feed(frames)], REST);
    feed.send(queue()).unwrap();
    settle(&mut model, &|s| !s.queue.is_empty());
    feed.send(ws_reply()).unwrap();
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(
        snap.error.as_deref(),
        Some("attention: the engine answered workspaces instead")
    );
    assert_eq!(snap.queue.len(), 1);
    // The engine's own no on the lane is its sentence, verbatim.
    feed.send(
        json!({ "ok": false, "error": "registered nowhere" })
            .to_string()
            .into_bytes(),
    )
    .unwrap();
    let snap = settle(&mut model, &|s| {
        s.error.as_deref() == Some("registered nowhere")
    });
    assert_eq!(snap.queue.len(), 1);
}
