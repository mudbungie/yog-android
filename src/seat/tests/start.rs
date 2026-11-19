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
    // The body the server stated goes back to it unchanged.
    let firing: Value = serde_json::from_slice(&requests[5]).unwrap();
    assert_eq!(
        firing,
        json!({ "op": "prompt",
                "prepared": { "workspace": "home", "binding": null, "lineage": null,
                              "goal": "look", "origin": "conversation",
                              "role": null },
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
        Some("start: the server answered workspaces instead")
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
    // Some server paths answer the fire with the staging it fired, which is
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
        Some("start: the server answered conversations instead")
    );
}

/// The firing's receipt, as the server answers it: the name it minted.
fn started() -> Vec<u8> {
    json!({ "ok": true, "kind": "started", "conversation": "brave-fox" })
        .to_string()
        .into_bytes()
}

/// The list once the founded conversation answers to its minted name.
fn conv_founded() -> Vec<u8> {
    json!({ "ok": true, "kind": "conversations",
            "rows": [{ "root_id": "b7", "display": "brave-fox", "name": "brave-fox",
                       "display_only": false, "state": "in-flight", "uncertain": false,
                       "preview": "", "age_secs": 0,
                       "last_active_unix": 1_700_000_042_i64, "attention": 0,
                       "members": 1, "direct": 0, "stoppable": true,
                       "stop_children": false, "depth": 0, "tone": "live" }] })
    .to_string()
    .into_bytes()
}

/// **A start shows the conversation it founded** (bl-82d7): the receipt names
/// it, the list is where the name gains an id, and the focus follows — so the
/// chat is the new conversation's and not the empty start page.
#[test]
fn a_start_is_followed_into_focus_once_the_list_carries_its_name() {
    let (mut model, served) = model_against(vec![
        vec![ws_reply()],
        vec![nothing_set()],
        vec![ws_reply()],
        vec![conv_reply()],
        vec![prepared()],
        vec![started()],
        vec![ws_reply()],     // the pass the gesture woke…
        vec![conv_founded()], // …finds the name, and is taken again under it
        vec![ws_reply()],
        vec![conv_founded()],
        vec![super::tr_reply()],
        vec![super::ag_reply()],
    ]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.focus_workspace(Some("home".into()));
    settle(&mut model, &|s| !s.conversations.is_empty());
    model.start_conversation("look".into());
    let snap = settle(&mut model, &|s| s.focus.agent.is_some());
    assert_eq!(snap.focus.agent.as_deref(), Some("b7"));
    assert_eq!(snap.focus.workspace.as_deref(), Some("home"));
    drop(model);
    let requests = served.join().unwrap();
    assert_eq!(
        ops(&requests)[6..],
        [
            "workspaces",
            "conversations",
            "workspaces",
            "conversations",
            "transcript",
            "agent"
        ]
    );
}

/// **A focus gesture ends the claim**: the operator went somewhere else
/// before the name appeared, and a list that carries it later moves nothing.
#[test]
fn a_focus_gesture_after_a_start_keeps_the_focus_where_it_was_put() {
    let (mut model, served) = model_against(vec![
        vec![ws_reply()],
        vec![nothing_set()],
        vec![ws_reply()],
        vec![conv_reply()],
        vec![prepared()],
        vec![started()],
        vec![ws_reply()],
        vec![conv_reply()], // the name is not on the list yet
        vec![ws_reply()],   // the operator opens another conversation
        vec![conv_founded()],
        vec![super::tr_reply()],
        vec![super::ag_reply()],
    ]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.focus_workspace(Some("home".into()));
    let before = settle(&mut model, &|s| !s.conversations.is_empty()).passed;
    model.start_conversation("look".into());
    settle(&mut model, &|s| s.passed > before);
    model.focus_conversation("home".into(), "a1".into());
    let snap = settle(&mut model, &|s| !s.transcript.is_empty());
    assert_eq!(snap.focus.agent.as_deref(), Some("a1"));
    drop(model);
    assert_eq!(served.join().unwrap().len(), 12);
}
