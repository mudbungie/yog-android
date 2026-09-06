//! **Sign-in from the glass** (REMOTE §8.3, DESIGN §13.19): the act that
//! starts a run inside the focused workspace's wall, the third held lane that
//! follows it, and the two ends of that lane's life — a watch the glass sets,
//! and a run that settles.
//!
//! This file is the ACT and the scaffolding both halves share; the held
//! tail's own cases are `signin/lane.rs`, split on the seam the wire draws —
//! `login` is what the operator says, `login-tail` is what the seat holds
//! open to hear the answer.

use super::{Model, Turn, conv_reply, model_turns, nothing_set, ops, settle, ws_reply};
use serde_json::json;

/// A `login` frame: the lines that landed since the last one, and — where
/// the run has ended — its exit and the command to run by hand.
pub(super) fn frame(lines: &[(bool, &str)], outcome: Option<i64>) -> Vec<u8> {
    let lines: Vec<_> = lines
        .iter()
        .map(|(err, text)| json!({ "err": err, "text": text }))
        .collect();
    let mut body = json!({ "ok": true, "kind": "login", "lines": lines });
    if let Some(outcome) = outcome {
        body["outcome"] = json!(outcome);
        body["fallback"] = json!("yog seat login acme");
    }
    body.to_string().into_bytes()
}

/// Boot, focus a workspace, and stop with the model settled two deep — the
/// state every case here starts from.
pub(super) fn focused(turns: Vec<Turn>) -> (Model, super::JoinHandle) {
    let mut script = vec![
        Turn::Answer(vec![ws_reply()]),
        Turn::Answer(vec![nothing_set()]),
        Turn::Answer(vec![ws_reply()]),
        Turn::Answer(vec![conv_reply()]),
    ];
    script.extend(turns);
    let (mut model, served) = model_turns(script);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.focus_workspace(Some("home".into()));
    settle(&mut model, &|s| !s.conversations.is_empty());
    (model, served)
}

/// The lines one snapshot's tail holds, under the provider it is about.
pub(super) fn tail(snap: &crate::seat::Snapshot, provider: &str) -> Option<Vec<String>> {
    let held = snap.login.as_ref().filter(|held| held.about(provider))?;
    Some(
        held.view
            .lines
            .iter()
            .map(|line| line.text.clone())
            .collect(),
    )
}

/// **The act's receipt is on the glass before any lane opens.** A sign-in
/// answers the run's standing, so the flow paints the moment the act lands
/// rather than a cadence later — here the lane is held open saying nothing,
/// and what stands is what the act came back with.
#[test]
fn the_act_seeds_the_tail_before_the_lane_says_anything() {
    let (mut model, served) = focused(vec![
        Turn::Answer(vec![frame(&[(true, "open https://auth.invalid")], None)]),
        Turn::Answer(vec![ws_reply()]),
        Turn::Answer(vec![conv_reply()]),
        Turn::Hold(Vec::new()),
    ]);
    model.sign_in("acme".into());
    let snap = settle(&mut model, &|s| tail(s, "acme").is_some());
    assert_eq!(
        tail(&snap, "acme").unwrap_or_default(),
        ["open https://auth.invalid"]
    );
    // The act is fired at the focused workspace, and the lane is the pass's
    // own dial after it — both spelled the way the engine reads them.
    drop(model);
    let requests = served.join().unwrap();
    assert_eq!(
        ops(&requests),
        [
            "workspaces",
            "roles",
            "workspaces",
            "conversations",
            "login",
            "workspaces",
            "conversations",
            "login-tail"
        ]
    );
    let asked: serde_json::Value = serde_json::from_slice(&requests[4]).unwrap();
    assert_eq!(
        asked,
        json!({ "op": "login", "workspace": "home", "provider": "acme" })
    );
    let held: serde_json::Value = serde_json::from_slice(&requests[7]).unwrap();
    assert_eq!(
        held,
        json!({ "op": "login-tail", "workspace": "home", "provider": "acme" })
    );
}

/// **A refused sign-in is the banner's sentence and no tail at all**: a run
/// that never started has said nothing. An unsigned wall crosses here, as
/// every refusal does — the act's own `ok: false`.
#[test]
fn a_refused_sign_in_paints_the_reason_and_no_tail() {
    let refusal = json!({ "ok": false, "error": "this wall has no signature" })
        .to_string()
        .into_bytes();
    let (mut model, _served) = focused(vec![
        Turn::Answer(vec![refusal]),
        Turn::Answer(vec![ws_reply()]),
        Turn::Answer(vec![conv_reply()]),
    ]);
    model.sign_in("acme".into());
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(snap.error.as_deref(), Some("this wall has no signature"));
    assert!(snap.login.is_none());
}

/// **An answer of the wrong kind is the wrong-kind sentence**, whichever end
/// of the pair it arrives at — the act's receipt here, and the lane's frame
/// in the case below it.
#[test]
fn an_answer_of_the_wrong_kind_says_which_kind_came_instead() {
    let (mut model, _served) = focused(vec![
        Turn::Answer(vec![super::pick::applied()]),
        Turn::Answer(vec![ws_reply()]),
        Turn::Answer(vec![conv_reply()]),
    ]);
    model.sign_in("acme".into());
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(
        snap.error.as_deref(),
        Some("login: the engine answered applied instead")
    );
}

/// **No workspace, no sign-in.** The act names one and the focus is where it
/// comes from, so a seat at the roster refuses here rather than sending a
/// frame the engine would refuse.
#[test]
fn a_sign_in_with_no_workspace_focused_never_leaves_this_device() {
    let (mut model, served) = super::model_turns(vec![Turn::Answer(vec![ws_reply()])]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.sign_in("acme".into());
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(
        snap.error.as_deref(),
        Some("login: no workspace is focused")
    );
    drop(model);
    assert_eq!(ops(&served.join().unwrap()), ["workspaces"]);
}

mod lane;
