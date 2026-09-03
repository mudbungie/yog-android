//! **The controls row's acts on a turn**: the stop the row offers while one
//! is in flight (REMOTE §3.1, bl-48fa) and the nudge it offers when none is
//! (§8.2, bl-d09e). Split from `pick` on the row's own seam — what answers
//! here, and what to do with what is answering.

use super::{conv_reply, nothing_set, ops, settle, ws_reply};
use serde_json::{Value, json};

/// **The stop gesture is the op** (REMOTE §3.1, bl-48fa): the envelope names
/// the conversation and whether the subtree goes with it, and nothing about
/// this path deposits content. The receipt is an `outcome`, and its `ok` is
/// litany's own verdict — a stop that landed on nothing reaches the banner
/// rather than reading as success.
#[test]
fn stopping_sends_the_op_and_carries_litanys_verdict() {
    let (mut model, served) = super::model_against(vec![
        vec![ws_reply()],
        vec![nothing_set()],
        vec![ws_reply()],
        vec![conv_reply()],
        vec![super::tr_reply()],
        vec![super::outcome(true, "")], // the stop
        vec![ws_reply()],
        vec![conv_reply()],
        vec![super::tr_reply()],
        vec![super::outcome(false, "nothing was running")], // stop all
        vec![ws_reply()],
        vec![conv_reply()],
        vec![super::tr_reply()],
    ]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.focus_conversation("home".into(), "a1".into());
    settle(&mut model, &|s| !s.transcript.is_empty());
    model.stop_turn(false);
    settle(&mut model, &|s| {
        s.error.is_none() && !s.transcript.is_empty()
    });
    model.stop_turn(true);
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(
        snap.error.as_deref(),
        Some("stop refused: nothing was running")
    );
    drop(model);
    let requests = served.join().unwrap();
    let stopped: Value = serde_json::from_slice(&requests[5]).unwrap();
    assert_eq!(
        stopped,
        json!({ "op": "stop", "workspace": "home", "agent": "a1", "children": false })
    );
    let all: Value = serde_json::from_slice(&requests[9]).unwrap();
    assert_eq!(all["children"], json!(true));
    // Nothing was deposited: a `/stop` line is content, and content wakes the
    // driver it meant to kill.
    assert!(!ops(&requests).contains(&"message".to_owned()));
}

/// A stop with no conversation focused is one sentence, not a hole in an
/// envelope — and a wrong kind under it names the kind.
#[test]
fn a_stop_with_nothing_focused_and_a_wrong_kind_both_name_themselves() {
    let (mut model, _s) = super::model_against(vec![vec![ws_reply()]]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.stop_turn(false);
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(
        snap.error.as_deref(),
        Some("stop: no conversation is focused")
    );
    drop(model);

    let (mut model, _s) = super::model_against(vec![
        vec![ws_reply()],
        vec![nothing_set()],
        vec![ws_reply()],
        vec![conv_reply()],
        vec![super::tr_reply()],
        vec![ws_reply()], // the stop, answered with a roster
    ]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.focus_conversation("home".into(), "a1".into());
    settle(&mut model, &|s| !s.transcript.is_empty());
    model.stop_turn(false);
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(
        snap.error.as_deref(),
        Some("stop: the engine answered workspaces instead")
    );
}

/// **The nudge deposits nothing** (§8.2, bl-d09e): its envelope names the
/// conversation and says nothing else, its receipt carries nothing, and no
/// message crosses — a branch that stopped advancing goes on without a line
/// in its transcript saying an operator poked it.
#[test]
fn nudging_re_prompts_without_depositing_anything() {
    let nudged = json!({ "ok": true, "kind": "nudged" })
        .to_string()
        .into_bytes();
    let (mut model, served) = super::model_against(vec![
        vec![ws_reply()],
        vec![nothing_set()],
        vec![ws_reply()],
        vec![conv_reply()],
        vec![super::tr_reply()],
        vec![nudged],
        vec![ws_reply()],
        vec![conv_reply()],
        vec![super::tr_reply()],
    ]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.focus_conversation("home".into(), "a1".into());
    settle(&mut model, &|s| !s.transcript.is_empty());
    model.nudge();
    settle(&mut model, &|s| {
        s.error.is_none() && !s.transcript.is_empty()
    });
    drop(model);
    let requests = served.join().unwrap();
    let asked: Value = serde_json::from_slice(&requests[5]).unwrap();
    assert_eq!(
        asked,
        json!({ "op": "nudge", "workspace": "home", "agent": "a1" })
    );
    assert!(!ops(&requests).contains(&"message".to_owned()));
}

/// A nudge with nothing focused is one sentence, and a wrong kind names it.
#[test]
fn a_nudge_with_nothing_focused_and_a_wrong_kind_both_name_themselves() {
    let (mut model, _s) = super::model_against(vec![vec![ws_reply()]]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.nudge();
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(
        snap.error.as_deref(),
        Some("nudge: no conversation is focused")
    );
    drop(model);

    let (mut model, _s) = super::model_against(vec![
        vec![ws_reply()],
        vec![nothing_set()],
        vec![ws_reply()],
        vec![conv_reply()],
        vec![super::tr_reply()],
        vec![ws_reply()], // the nudge, answered with a roster
    ]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.focus_conversation("home".into(), "a1".into());
    settle(&mut model, &|s| !s.transcript.is_empty());
    model.nudge();
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(
        snap.error.as_deref(),
        Some("nudge: the engine answered workspaces instead")
    );
}
