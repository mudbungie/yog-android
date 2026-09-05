//! **The two world surfaces' reads and acts** (DESIGN §13.8, bl-35bd): the
//! trail, the queue asked in its own right, and the acknowledgement and
//! truncation over the trail.
//!
//! Two things are load-bearing here and neither is the happy path. The queue
//! read reaches the snapshot at the TOP depth — the pass asks it only under an
//! open conversation, so a whole-queue surface that painted the pass's answer
//! would be empty exactly when nobody had opened a conversation first. And
//! both acts are followed by a trail read, because a watermark and a
//! truncation are invisible until the trail is read again: the screen would
//! otherwise stand on the rows it had before the act.

use serde_json::{Value, json};

use super::{Turn, model_turns, ops, settle, ws_reply};

/// The trail, as `corpus/reply/ops.json` spells one.
fn trail(argv: &str) -> Vec<u8> {
    json!({ "ok": true, "kind": "ops",
            "rows": [{ "argv": argv, "cwd": "/p", "exit": 1, "origin": "balls",
                       "stderr": "gate", "stdout": "", "ts": "1700" }] })
    .to_string()
    .into_bytes()
}

/// An empty trail — what a cleared one answers.
fn cleared() -> Vec<u8> {
    json!({ "ok": true, "kind": "ops", "rows": [] })
        .to_string()
        .into_bytes()
}

/// The queue with one conversation waiting, addressed at no conversation this
/// seat has focused: the whole-queue read names no place.
fn queue() -> Vec<u8> {
    json!({ "ok": true, "kind": "attention",
            "rows": [{ "workspace": "home", "agent": "a9", "display": "d",
                       "state": "stopped", "uncertain": false,
                       "signals": ["mail"], "preview": "p", "age_secs": 5,
                       "pending": 2, "held": null, "failure": null,
                       "flag": null }] })
    .to_string()
    .into_bytes()
}

fn receipt(kind: &str) -> Vec<u8> {
    json!({ "ok": true, "kind": kind }).to_string().into_bytes()
}

/// The trail crosses as the engine's own ask — a bounded tail — and its rows
/// reach the snapshot with the row's own facts on them.
#[test]
fn the_trail_is_asked_for_and_its_rows_reach_the_snapshot() {
    let (mut model, served) = super::model_against(vec![
        vec![ws_reply()],
        vec![trail("bl close x")],
        vec![ws_reply()],
    ]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.list_trail();
    let snap = settle(&mut model, &|s| !s.trail.is_empty());
    let row = &snap.trail[0];
    assert_eq!(
        (row.argv.as_str(), row.origin.as_str(), row.exit),
        ("bl close x", "balls", 1)
    );
    drop(model);
    let requests = served.join().unwrap();
    let asked: Value = serde_json::from_slice(&requests[1]).unwrap();
    assert_eq!(asked["op"], json!("ops"));
    assert!(
        asked["max"].as_u64().unwrap() > 0,
        "the ask carries its own bound"
    );
}

/// **The queue read reaches the snapshot with nothing focused.** The pass asks
/// it only under an open conversation (§13.7); this is the surface asking, and
/// the rows are the same answer in the same holder.
#[test]
fn the_queue_can_be_asked_for_where_no_conversation_is_open() {
    let (mut model, served) =
        super::model_against(vec![vec![ws_reply()], vec![queue()], vec![ws_reply()]]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.list_queue();
    let snap = settle(&mut model, &|s| !s.queue.is_empty());
    assert_eq!(snap.queue[0].agent, "a9");
    assert!(snap.focus.workspace.is_none(), "asked at the top depth");
    drop(model);
    let requests = served.join().unwrap();
    assert_eq!(ops(&requests), ["workspaces", "attention", "workspaces"]);
}

/// **The acknowledgement is followed by the read that shows what it did.**
/// Nothing else can: the watermark is invisible until the trail says so.
#[test]
fn the_ack_is_followed_by_the_read_that_says_what_it_did() {
    let (mut model, served) = super::model_against(vec![
        vec![ws_reply()],
        vec![trail("bl close x")],
        vec![ws_reply()],
        vec![receipt("acked")],
        vec![trail("bz login")],
        vec![ws_reply()],
    ]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.list_trail();
    settle(&mut model, &|s| !s.trail.is_empty());
    model.ack_trail();
    let snap = settle(&mut model, &|s| s.trail[0].argv == "bz login");
    assert!(
        snap.error.is_none(),
        "a receipt says nothing to the operator"
    );
    drop(model);
    let requests = served.join().unwrap();
    assert_eq!(
        serde_json::from_slice::<Value>(&requests[3]).unwrap(),
        json!({ "op": "ack" })
    );
    assert_eq!(ops(&requests)[4], "ops");
}

/// The truncation, and the read that shows the trail is gone.
#[test]
fn the_truncation_crosses_and_the_trail_reads_back_empty() {
    let (mut model, served) = super::model_against(vec![
        vec![ws_reply()],
        vec![trail("bl close x")],
        vec![ws_reply()],
        vec![receipt("trail-cleared")],
        vec![cleared()],
        vec![ws_reply()],
    ]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.list_trail();
    settle(&mut model, &|s| !s.trail.is_empty());
    model.clear_trail();
    settle(&mut model, &|s| s.trail.is_empty() && s.error.is_none());
    drop(model);
    let requests = served.join().unwrap();
    assert_eq!(
        serde_json::from_slice::<Value>(&requests[3]).unwrap(),
        json!({ "op": "clear-trail" })
    );
}

/// **A read answered with the wrong kind is a sentence naming what was asked**
/// — the same shape every other read here refuses with, and the reason the
/// rows already held are not dropped for it.
#[test]
fn an_answer_of_the_wrong_kind_is_named_and_keeps_what_was_there() {
    let (mut model, served) = super::model_against(vec![
        vec![ws_reply()],
        vec![trail("bl close x")],
        vec![ws_reply()],
        vec![receipt("acked")],
        vec![ws_reply()],
        vec![receipt("nudged")],
        vec![ws_reply()],
    ]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.list_trail();
    settle(&mut model, &|s| !s.trail.is_empty());
    model.list_trail();
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(
        snap.error.as_deref(),
        Some("ops: the engine answered acked instead")
    );
    assert_eq!(
        snap.trail.len(),
        1,
        "an answer this seat could not read drops none"
    );
    model.list_queue();
    let snap = settle(&mut model, &|s| {
        s.error.as_deref() == Some("attention: the engine answered nudged instead")
    });
    assert!(snap.queue.is_empty());
    drop(model);
    served.join().unwrap();
}

/// **An act answered with the wrong kind is refused in the operator's own
/// banner**, and — for the pair over the trail — the read after it still runs:
/// what the act did is unknowable from the receipt either way.
#[test]
fn an_act_answered_with_the_wrong_kind_is_refused_by_name() {
    let (mut model, served) = super::model_against(vec![
        vec![ws_reply()],
        vec![receipt("nudged")],
        vec![trail("bl close x")],
        vec![ws_reply()],
        vec![receipt("applied")],
        vec![trail("bl close x")],
        vec![ws_reply()],
    ]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.ack_trail();
    settle(&mut model, &|s| {
        s.error.as_deref() == Some("ack: the engine answered nudged instead")
    });
    model.clear_trail();
    settle(&mut model, &|s| {
        s.error.as_deref() == Some("clear-trail: the engine answered applied instead")
    });
    drop(model);
    served.join().unwrap();
}

/// **A lost reply leaves either act in doubt, and neither is ever re-sent.**
/// The sentence names the read that settles it, which is the trail — and the
/// re-read that follows the act is that read, made straight away.
#[test]
fn a_lost_receipt_leaves_the_act_in_doubt_and_names_the_trail() {
    let (mut model, served) = model_turns(vec![
        Turn::Answer(vec![ws_reply()]),
        Turn::Hangup,
        Turn::Answer(vec![trail("bl close x")]),
        Turn::Answer(vec![ws_reply()]),
        Turn::Hangup,
        Turn::Answer(vec![trail("bl close x")]),
        Turn::Answer(vec![ws_reply()]),
    ]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.ack_trail();
    let snap = settle(&mut model, &|s| s.error.is_some());
    let said = snap.error.unwrap_or_default();
    assert!(
        said.starts_with("ack may have run: the reply was lost ("),
        "said: {said}"
    );
    assert!(
        said.ends_with("The trail says what it stands at when it is read again."),
        "said: {said}"
    );
    model.clear_trail();
    let snap = settle(&mut model, &|s| {
        s.error
            .as_deref()
            .is_some_and(|e| e.starts_with("clear-trail may have run"))
    });
    assert!(
        !snap.trail.is_empty(),
        "the re-read after the act still ran"
    );
    drop(model);
    served.join().unwrap();
}

/// **A read that could not be made at all is one sentence and no lost rows.**
#[test]
fn a_read_that_failed_keeps_the_rows_it_had() {
    let (mut model, served) = model_turns(vec![
        Turn::Answer(vec![ws_reply()]),
        Turn::Answer(vec![trail("bl close x")]),
        Turn::Answer(vec![ws_reply()]),
        Turn::Hangup,
        Turn::Answer(vec![ws_reply()]),
    ]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.list_trail();
    settle(&mut model, &|s| !s.trail.is_empty());
    model.list_trail();
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(snap.trail.len(), 1, "the answer the engine gave stands");
    drop(model);
    served.join().unwrap();
}
