//! Starting a conversation: the §8.1 pair run as one act, the prepared body
//! carried back whole, and every way either half can answer wrongly.

use super::{conv_reply, model_against, nothing_set, ops, outcome, prepared, settle, ws_reply};
use serde_json::{Value, json};

#[test]
fn starting_a_conversation_stages_then_fires_carrying_the_body_whole() {
    let (mut model, served) = model_against(vec![
        vec![ws_reply()],
        vec![nothing_set()],
        vec![ws_reply()],        // focus_workspace refreshes…
        vec![conv_reply()],      // …two deep
        vec![prepared()],        // the staging
        vec![outcome(true, "")], // the firing
        vec![ws_reply()],
        vec![conv_reply()],
    ]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.focus_workspace(Some("home".into()));
    settle(&mut model, &|s| !s.conversations.is_empty());
    model.start_conversation("look".into());
    settle(&mut model, &|s| {
        !s.conversations.is_empty() && s.error.is_none()
    });
    let requests = served.join().unwrap();
    assert_eq!(
        ops(&requests),
        [
            "workspaces",
            "roles",
            "workspaces",
            "conversations",
            "prepare",
            "prompt",
            "workspaces",
            "conversations"
        ]
    );
    let staging: Value = serde_json::from_slice(&requests[4]).unwrap();
    assert_eq!(
        staging,
        json!({ "op": "prepare", "workspace": "home",
                "payload": { "rung": "bare" } })
    );
    // The body the engine stated goes back to it unchanged.
    let firing: Value = serde_json::from_slice(&requests[5]).unwrap();
    assert_eq!(
        firing,
        json!({ "op": "prompt",
                "prepared": { "workspace": "home", "binding": null, "lineage": null,
                              "goal": "look", "origin": "conversation" },
                "goal": "look", "seed": null })
    );
}

#[test]
fn a_start_with_no_workspace_focused_reaches_the_banner() {
    let (mut model, _served) = model_against(vec![vec![ws_reply()], vec![ws_reply()]]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.start_conversation("look".into());
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(
        snap.error.as_deref(),
        Some("start: no workspace is focused")
    );
}

#[test]
fn a_staging_answered_with_the_wrong_kind_names_it() {
    let (mut model, _served) = model_against(vec![
        vec![ws_reply()],
        vec![nothing_set()],
        vec![ws_reply()],
        vec![conv_reply()],
        vec![ws_reply()], // wrong: the staging slot
        vec![ws_reply()],
        vec![conv_reply()],
    ]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.focus_workspace(Some("home".into()));
    settle(&mut model, &|s| !s.conversations.is_empty());
    model.start_conversation("look".into());
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(
        snap.error.as_deref(),
        Some("start: the engine answered workspaces instead")
    );
}

#[test]
fn a_refused_firing_reaches_the_banner_and_a_re_staging_is_accepted() {
    let (mut model, _served) = model_against(vec![
        vec![ws_reply()],
        vec![nothing_set()],
        vec![ws_reply()],
        vec![conv_reply()],
        vec![prepared()],
        vec![outcome(false, "no lineage")],
        vec![ws_reply()],
        vec![conv_reply()],
    ]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.focus_workspace(Some("home".into()));
    settle(&mut model, &|s| !s.conversations.is_empty());
    model.start_conversation("look".into());
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(snap.error.as_deref(), Some("start refused: no lineage"));
}

#[test]
fn a_firing_answered_with_a_prepared_body_is_accepted() {
    // Some engine paths answer the fire with the staging it fired, which is
    // a receipt and not a refusal — a client that called it wrong would
    // redden a banner over a conversation that had in fact started.
    let (mut model, _served) = model_against(vec![
        vec![ws_reply()],
        vec![nothing_set()],
        vec![ws_reply()],
        vec![conv_reply()],
        vec![prepared()],
        vec![prepared()],
        vec![ws_reply()],
        vec![conv_reply()],
    ]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.focus_workspace(Some("home".into()));
    settle(&mut model, &|s| !s.conversations.is_empty());
    model.start_conversation("look".into());
    settle(&mut model, &|s| {
        !s.conversations.is_empty() && s.error.is_none()
    });
}

#[test]
fn a_firing_answered_with_the_wrong_kind_names_it() {
    let (mut model, _served) = model_against(vec![
        vec![ws_reply()],
        vec![nothing_set()],
        vec![ws_reply()],
        vec![conv_reply()],
        vec![prepared()],
        vec![conv_reply()], // wrong: the firing slot
        vec![ws_reply()],
        vec![conv_reply()],
    ]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.focus_workspace(Some("home".into()));
    settle(&mut model, &|s| !s.conversations.is_empty());
    model.start_conversation("look".into());
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(
        snap.error.as_deref(),
        Some("start: the engine answered conversations instead")
    );
}
