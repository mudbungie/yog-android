//! The message deposit: what a composer's submit does, and what a refusal or
//! an unreachable engine leaves in the banner.

use super::{conv_reply, model_against, outcome, settle, tr_reply, ws_reply};
use serde_json::{Value, json};

#[test]
fn a_deposit_posts_the_composer_and_refreshes() {
    let (mut model, served) = model_against(vec![
        vec![ws_reply()],
        vec![ws_reply()],
        vec![conv_reply()],
        vec![tr_reply()],
        vec![outcome(true, "")], // the deposit's receipt
        vec![ws_reply()],
        vec![conv_reply()],
        vec![tr_reply()],
    ]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.focus_conversation("home".into(), "a1".into());
    settle(&mut model, &|s| !s.transcript.is_empty());
    model.deposit("hello".into());
    // The post-deposit refresh publishes with no error: the receipt was ok.
    settle(&mut model, &|s| {
        !s.transcript.is_empty() && s.error.is_none()
    });
    drop(model);
    let requests = served.join().unwrap();
    let message: Value = serde_json::from_slice(&requests[4]).unwrap();
    assert_eq!(
        message,
        json!({ "op": "message", "workspace": "home", "agent": "a1", "content": "hello" })
    );
}

#[test]
fn a_refused_deposit_reaches_the_banner() {
    let (mut model, _served) = model_against(vec![
        vec![ws_reply()],
        vec![ws_reply()],
        vec![conv_reply()],
        vec![tr_reply()],
        vec![outcome(false, "gate red")],
        vec![ws_reply()],
        vec![conv_reply()],
        vec![tr_reply()],
    ]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.focus_conversation("home".into(), "a1".into());
    settle(&mut model, &|s| !s.transcript.is_empty());
    model.deposit("hello".into());
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(snap.error.as_deref(), Some("deposit refused: gate red"));
}

/// **A gesture's own answer never waits; the refresh behind it does**
/// (bl-3202). One scripted connection, so once it is served the listener is
/// gone and the post-deposit refresh fails too — and the banner carries the
/// deposit's sentence ALONE, because a first failed pass is inside the grace.
/// The roster the engine already gave is still painted under it.
#[test]
fn an_unfocused_deposits_sentence_paints_while_the_refresh_behind_it_waits() {
    let (mut model, served) = model_against(vec![vec![ws_reply()]]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    served.join().unwrap();
    model.deposit("hello".into());
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(
        snap.error.as_deref(),
        Some("deposit: no conversation is focused")
    );
    assert!(!snap.workspaces.is_empty());
}

#[test]
fn wrong_reply_kinds_name_the_kind() {
    // workspaces answered with conversations. Twice, and a gesture between
    // them: a first failed pass is inside the §13.2 grace (bl-3202), so the
    // sentence is earned by the second — which is what a test asks for by
    // forcing another pass.
    let (mut model, _s) = model_against(vec![vec![conv_reply()], vec![conv_reply()]]);
    model.focus_workspace(None);
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(
        snap.error.as_deref(),
        Some("workspaces: the engine answered conversations instead")
    );
    drop(model);

    // conversations answered with an outcome.
    let (mut model, _s) = model_against(vec![
        vec![ws_reply()],
        vec![ws_reply()],
        vec![outcome(true, "")],
        vec![ws_reply()],
        vec![outcome(true, "")],
    ]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.focus_workspace(Some("home".into()));
    model.focus_workspace(Some("home".into()));
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(
        snap.error.as_deref(),
        Some("conversations: the engine answered outcome instead")
    );
    drop(model);

    // transcript answered with workspaces.
    let (mut model, _s) = model_against(vec![
        vec![ws_reply()],
        vec![ws_reply()],
        vec![conv_reply()],
        vec![ws_reply()],
        vec![ws_reply()],
        vec![conv_reply()],
        vec![ws_reply()],
    ]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.focus_conversation("home".into(), "a1".into());
    model.focus_conversation("home".into(), "a1".into());
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(
        snap.error.as_deref(),
        Some("transcript: the engine answered workspaces instead")
    );
    drop(model);

    // the deposit's receipt answered with a transcript.
    let (mut model, _s) = model_against(vec![
        vec![ws_reply()],
        vec![ws_reply()],
        vec![conv_reply()],
        vec![tr_reply()],
        vec![tr_reply()], // wrong: the receipt slot
        vec![ws_reply()],
        vec![conv_reply()],
        vec![tr_reply()],
    ]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.focus_conversation("home".into(), "a1".into());
    settle(&mut model, &|s| !s.transcript.is_empty());
    model.deposit("hello".into());
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(
        snap.error.as_deref(),
        Some("deposit: the engine answered transcript instead")
    );
}
