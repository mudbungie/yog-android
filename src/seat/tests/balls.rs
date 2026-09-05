//! **The ball pane's three reads** (DESIGN §13.9): what crosses, what reaches
//! the snapshot, and the two ways a view can be asked for and not answered.
//!
//! What is load-bearing here is the PAIRING: the pane holds one answer and
//! says which read produced it, so a screen paints its own view's rows or none
//! — the same law `cache::read` keeps over rows and the focus they were asked
//! at, one surface along.

use serde_json::{Value, json};

use super::{conv_reply, nothing_set, ops, settle, tr_reply, ws_reply};
use crate::codec::{Pane, View};

/// `balls`, as the engine spells one.
fn balls() -> Vec<u8> {
    json!({ "ok": true, "kind": "balls",
            "rows": [{ "ball_id": "bl-1", "project": "p", "state": "ready",
                       "title": "t", "claimant": "alba", "workspace": "home" }] })
    .to_string()
    .into_bytes()
}

/// `workspace-balls`, with the spend the engine rendered.
fn held() -> Vec<u8> {
    json!({ "ok": true, "kind": "workspace-balls",
            "rows": [{ "id": "bl-2", "project": "p", "state": "bound", "owner": "home",
                       "spend": { "usd": "$2.50" } }] })
    .to_string()
    .into_bytes()
}

/// `board`, with one armed loop's own sentence beside its rows.
fn board() -> Vec<u8> {
    json!({ "ok": true, "kind": "board",
            "rows": [{ "id": "bl-3", "project": "p", "column": "ready", "priority": 0 }],
            "fleet": [{ "label": "1/4 drones", "workspace": "home", "project": "p" }] })
    .to_string()
    .into_bytes()
}

/// **The read that names no place** crosses with nothing but its op, and its
/// rows reach the snapshot under the view that asked for them.
#[test]
fn the_whole_worlds_balls_are_read_by_the_gesture_and_reach_the_snapshot() {
    let (mut model, served) = super::model_against(vec![
        vec![ws_reply()],
        vec![balls()],
        vec![ws_reply()],
        vec![board()],
        vec![ws_reply()],
    ]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.list_balls(View::Everywhere);
    let snap = settle(&mut model, &|s| s.pane.is_some());
    let Some(Pane::Everywhere(rows)) = snap.pane else {
        panic!("the pane is not the view that was asked for")
    };
    assert_eq!(rows.first().map(|row| row.id.clone()), Some("bl-1".into()));

    // **The board replaces it**, and the pane says which read answered.
    model.list_balls(View::Board);
    let snap = settle(&mut model, &|s| {
        s.pane.as_ref().is_some_and(|p| p.view() == View::Board)
    });
    let Some(Pane::Board(board)) = snap.pane else {
        panic!("the board did not land")
    };
    assert_eq!(board.fleet, ["1/4 drones"]);
    drop(model);
    let requests = served.join().unwrap();
    assert_eq!(
        serde_json::from_slice::<Value>(&requests[1]).unwrap(),
        json!({ "op": "balls" })
    );
    assert_eq!(ops(&requests)[3], "board");
}

/// **The read that names a place takes the focus**, and names the workspace
/// the operator is standing in rather than one it invented.
#[test]
fn a_workspaces_own_balls_are_asked_of_the_focused_workspace() {
    let (mut model, served) = super::model_against(vec![
        vec![ws_reply()],
        vec![nothing_set()],
        vec![ws_reply()],
        vec![conv_reply()],
        vec![held()],
        vec![ws_reply()],
        vec![conv_reply()],
    ]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.focus_workspace(Some("home".into()));
    settle(&mut model, &|s| !s.conversations.is_empty());
    model.list_balls(View::Here);
    let snap = settle(&mut model, &|s| s.pane.is_some());
    let Some(Pane::Here(rows)) = snap.pane else {
        panic!("the pane is not the view that was asked for")
    };
    assert_eq!(
        rows.first().map(|row| row.usd.clone()),
        Some("$2.50".into())
    );
    drop(model);
    let requests = served.join().unwrap();
    assert_eq!(
        serde_json::from_slice::<Value>(&requests[4]).unwrap(),
        json!({ "op": "workspace-balls", "workspace": "home" })
    );
}

/// **Asked with nothing focused, it is refused here and crosses nothing**: a
/// workspace's balls under no workspace's name is the wrong claim, and the
/// sentence is the same one every focused read gives.
#[test]
fn the_aimed_read_with_no_workspace_focused_crosses_nothing() {
    let (mut model, served) = super::model_against(vec![vec![ws_reply()], vec![ws_reply()]]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.list_balls(View::Here);
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(snap.error.as_deref(), Some("no workspace is focused"));
    assert!(snap.pane.is_none());
    drop(model);
    assert!(
        ops(&served.join().unwrap())
            .iter()
            .all(|op| op == "workspaces"),
        "nothing but the pass crossed"
    );
}

/// An answer of another kind is a sentence naming the view that was opened,
/// and what the pane already held is not dropped for it — `searched`'s rule,
/// which every gesture-driven read here keeps.
#[test]
fn an_answer_of_the_wrong_kind_names_the_view_and_keeps_what_was_there() {
    let (mut model, served) = super::model_against(vec![
        vec![ws_reply()],
        vec![balls()],
        vec![ws_reply()],
        vec![tr_reply()],
        vec![ws_reply()],
    ]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.list_balls(View::Everywhere);
    settle(&mut model, &|s| s.pane.is_some());
    model.list_balls(View::Board);
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(
        snap.error.as_deref(),
        Some("board: the engine answered transcript instead")
    );
    assert_eq!(
        snap.pane.map(|pane| pane.view()),
        Some(View::Everywhere),
        "the answer it had is still the answer it has"
    );
    drop(model);
    served.join().unwrap();
}
