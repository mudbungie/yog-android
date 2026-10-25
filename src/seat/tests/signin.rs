//! **Sign-in from the glass** (REMOTE §8.3, DESIGN §13.19): the act that
//! starts a run inside the focused workspace's directory, the third held lane that
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
    // own dial after it — both spelled the way the server reads them.
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

/// **Opening the sign-in screen asks for the CATALOG, not the table**
/// (bl-892e): what paints is every row the workspace can sign in to, signed
/// in or not, and the routing table beside it stays exactly as it was — here
/// empty, which is a fresh workspace's whole point. The ask names the focused
/// workspace, because *signed in* is a fact of the workspace asked about.
#[test]
fn opening_the_sign_in_screen_lists_the_catalog_and_leaves_the_table_alone() {
    let catalog = json!({ "ok": true, "kind": "catalog",
                          "rows": [{ "name": "acme", "fact": "auth oauth2 · not signed in",
                                     "blocked": null, "effort": true, "priority": false }] })
    .to_string()
    .into_bytes();
    let (mut model, served) = focused(vec![
        Turn::Answer(vec![catalog]),
        Turn::Answer(vec![ws_reply()]),
        Turn::Answer(vec![conv_reply()]),
    ]);
    model.list_catalog();
    let snap = settle(&mut model, &|s| !s.catalog.is_empty());
    assert_eq!(snap.catalog[0].name, "acme");
    assert!(
        snap.providers.is_empty(),
        "the table was not asked and not painted"
    );
    drop(model);
    let requests = served.join().unwrap();
    assert_eq!(ops(&requests)[4], "catalog");
    let asked: serde_json::Value = serde_json::from_slice(&requests[4]).unwrap();
    assert_eq!(asked, json!({ "op": "catalog", "workspace": "home" }));
}

/// A catalog answered in the wrong kind is the wrong-kind sentence, like
/// every read here — and nothing paints.
#[test]
fn a_catalog_of_the_wrong_kind_says_which_kind_came_instead() {
    let (mut model, _served) = focused(vec![
        Turn::Answer(vec![super::pick::applied()]),
        Turn::Answer(vec![ws_reply()]),
        Turn::Answer(vec![conv_reply()]),
    ]);
    model.list_catalog();
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(
        snap.error.as_deref(),
        Some("catalog: the server answered applied instead")
    );
    assert!(snap.catalog.is_empty());
}

/// **A refused sign-in is the banner's sentence and no tail at all**: a run
/// that never started has said nothing. An unsigned workspace crosses here, as
/// every refusal does — the act's own `ok: false`.
#[test]
fn a_refused_sign_in_paints_the_reason_and_no_tail() {
    let refusal = json!({ "ok": false, "error": "this workspace has no signature" })
        .to_string()
        .into_bytes();
    let (mut model, _served) = focused(vec![
        Turn::Answer(vec![refusal]),
        Turn::Answer(vec![ws_reply()]),
        Turn::Answer(vec![conv_reply()]),
    ]);
    model.sign_in("acme".into());
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(
        snap.error.as_deref(),
        Some("this workspace has no signature")
    );
    assert!(snap.login.is_none());
}

/// **A sign-in in doubt holds its tail open** (§24 item 5): the act was
/// written and nothing answered it, so the doubt sentence names the tail as
/// the read that settles it — and the tail is what then stands under the
/// row, saying whether a run started. Nothing is sent again.
#[test]
fn a_sign_in_whose_reply_was_lost_watches_the_tail_that_settles_it() {
    let (mut model, served) = focused(vec![
        Turn::Hangup, // the act: read, then a FIN where the answer belongs
        Turn::Answer(vec![ws_reply()]),
        Turn::Answer(vec![conv_reply()]),
        Turn::Hold(vec![frame(&[(true, "open https://auth.invalid")], None)]),
    ]);
    model.sign_in("acme".into());
    let snap = settle(&mut model, &|s| s.error.is_some());
    let said = snap.error.unwrap_or_default();
    assert!(
        said.starts_with("login may have run: the reply was lost ("),
        "{said}"
    );
    assert!(said.contains("held open on this screen"), "{said}");
    let snap = settle(&mut model, &|s| {
        tail(s, "acme").is_some_and(|lines| !lines.is_empty())
    });
    assert_eq!(
        tail(&snap, "acme").unwrap_or_default(),
        ["open https://auth.invalid"]
    );
    drop(model);
    let wire = ops(&served.join().unwrap());
    assert_eq!(wire.iter().filter(|op| *op == "login").count(), 1);
    assert!(wire.iter().any(|op| op == "login-tail"), "{wire:?}");
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
        Some("login: the server answered applied instead")
    );
}

/// **No workspace, no sign-in.** The act names one and the focus is where it
/// comes from, so a seat at the roster refuses here rather than sending a
/// frame the server would refuse.
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
