//! **Answering the parked tool call** (DESIGN §13.7, bl-b39d): the envelope
//! the verdict puts on the wire, the parked call reaching the snapshot, and
//! the one success that still owes the operator a sentence.
//!
//! The load-bearing assertion is that the gesture names the CONVERSATION and
//! not the call: the engine reads the held mark itself at fire time, so a
//! `tool_use` in this envelope would be a client answering a call that may
//! already have moved on.

use std::sync::mpsc;

use serde_json::{Value, json};

use super::{
    REST, Turn, conv_reply, model_lanes, nothing_set, queue_quiet, settle, tr_reply, ws_reply,
};
use crate::codec::Verdict;

/// A model whose attention lane the test FEEDS (§14.1): the queue arrives as
/// a frame the test sends, when the test sends it.
fn fed(scripts: Vec<Vec<Vec<u8>>>) -> (super::Model, super::JoinHandle, mpsc::Sender<Vec<u8>>) {
    let (feed, frames) = mpsc::channel();
    let (model, served) = model_lanes(scripts, vec![Turn::Feed(frames)], REST);
    (model, served, feed)
}

/// A queue with one conversation parked on a tool call — the corpus row's own
/// spelling, addressed at the conversation these tests focus.
fn queue_held() -> Vec<u8> {
    json!({ "ok": true, "kind": "attention",
            "rows": [{ "workspace": "home", "agent": "a1", "display": "d",
                       "state": "stopped", "uncertain": false,
                       "signals": ["held"], "says": "parked a tool invocation for your answer", "preview": "", "age_secs": 3,
                       "pending": 0,
                       "held": { "tool": "Bash", "tool_use": "toolu_1",
                                 "reason": "writes" },
                       "failure": null, "flag": null }] })
    .to_string()
    .into_bytes()
}

/// The receipt, with `advanced` as the caller wants it.
fn answered(verdict: &str, advanced: bool) -> Vec<u8> {
    json!({ "ok": true, "kind": "answered", "tool": "Bash",
            "tool_use": "toolu_1", "verdict": verdict, "advanced": advanced })
    .to_string()
    .into_bytes()
}

/// The queue reaches the snapshot, and the verdict crosses naming the
/// conversation and nothing else.
#[test]
fn the_parked_call_reaches_the_snapshot_and_the_verdict_names_the_conversation() {
    let (mut model, served, feed) = fed(vec![
        vec![ws_reply()],
        vec![nothing_set()],
        vec![ws_reply()],
        vec![conv_reply()],
        vec![tr_reply()],
        vec![answered("pass", true)],
        vec![ws_reply()],
        vec![conv_reply()],
        vec![tr_reply()],
    ]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.focus_conversation("home".into(), "a1".into());
    feed.send(queue_held()).unwrap();
    let snap = settle(&mut model, &|s| !s.queue.is_empty());
    let held = crate::codec::queue::held_at(&snap.queue, "home", "a1").unwrap();
    assert_eq!(
        (held.tool.as_str(), held.reason.as_str()),
        ("Bash", "writes")
    );
    model.answer(Verdict::Pass);
    // The answered call is gone from the lane's next frame, which is the
    // engine saying the answer changed (§14.1) — and the read that settles
    // this act.
    feed.send(queue_quiet()).unwrap();
    settle(&mut model, &|s| s.queue.is_empty() && s.error.is_none());
    drop(model);
    let requests = served.join().unwrap();
    assert_eq!(
        serde_json::from_slice::<Value>(&requests[5]).unwrap(),
        json!({ "op": "answer", "workspace": "home", "agent": "a1", "verdict": "pass" })
    );
}

/// **A release that did not advance is a sentence**, because it is the one
/// outcome the screen cannot show: the answer is recorded and the
/// conversation is exactly where it was.
#[test]
fn a_release_that_drove_nothing_says_so() {
    let (mut model, _s, feed) = fed(vec![
        vec![ws_reply()],
        vec![nothing_set()],
        vec![ws_reply()],
        vec![conv_reply()],
        vec![tr_reply()],
        vec![answered("pass", false)],
        vec![ws_reply()],
        vec![conv_reply()],
        vec![tr_reply()],
    ]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.focus_conversation("home".into(), "a1".into());
    feed.send(queue_held()).unwrap();
    settle(&mut model, &|s| !s.queue.is_empty());
    model.answer(Verdict::Pass);
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(
        snap.error.as_deref(),
        Some(
            "answer pass: recorded on Bash, but the conversation was not driven on — \
             nothing moves until it is"
        )
    );
}

/// **A hold drives nothing and owes nothing**: `advanced: false` under the one
/// verdict that never releases is the state the operator asked for.
#[test]
fn keeping_it_parked_is_silent() {
    let (mut model, _s, feed) = fed(vec![
        vec![ws_reply()],
        vec![nothing_set()],
        vec![ws_reply()],
        vec![conv_reply()],
        vec![tr_reply()],
        vec![answered("hold", false)],
        vec![ws_reply()],
        vec![conv_reply()],
        vec![tr_reply()],
    ]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.focus_conversation("home".into(), "a1".into());
    feed.send(queue_held()).unwrap();
    settle(&mut model, &|s| !s.queue.is_empty());
    model.answer(Verdict::Hold);
    settle(&mut model, &|s| s.roles_read > 0 || s.error.is_none());
    assert_eq!(model.snapshot().error, None);
}

/// No conversation focused, and a receipt of the wrong shape: both name
/// themselves, the same way every other act's guard does.
#[test]
fn an_answer_with_nothing_focused_and_a_wrong_kind_both_name_themselves() {
    let (mut model, served) = super::model_against(vec![vec![ws_reply()]]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.answer(Verdict::Refuse);
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(
        snap.error.as_deref(),
        Some("answer: no conversation is focused")
    );
    drop(model);
    assert_eq!(super::ops(&served.join().unwrap()), ["workspaces"]);

    let (mut model, _s, feed) = fed(vec![
        vec![ws_reply()],
        vec![nothing_set()],
        vec![ws_reply()],
        vec![conv_reply()],
        vec![tr_reply()],
        vec![ws_reply()], // the answer, answered with a roster
    ]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.focus_conversation("home".into(), "a1".into());
    feed.send(queue_held()).unwrap();
    settle(&mut model, &|s| !s.queue.is_empty());
    model.answer(Verdict::Refuse);
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(
        snap.error.as_deref(),
        Some("answer: the engine answered workspaces instead")
    );
}
