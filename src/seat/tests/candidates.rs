//! **The candidate surfaces** (DESIGN §13.12): what an opening asks, what the
//! row's handle decides, and the two receipts that are worth a sentence on
//! success.
//!
//! This file is the READ and the scaffolding the acts beside it share; the
//! three acts have a file of their own (`candidates/acts.rs`), on the seam the
//! screen itself is cut along — what the listing SAYS, against what is done to
//! a row of it.

use serde_json::{Value, json};

mod acts;

use super::{conv_reply, nothing_set, ops, settle, tr_reply, ws_reply};

pub(super) fn science() -> Vec<u8> {
    json!({ "ok": true, "kind": "science",
            "rows": [{ "diff": { "ball_id": "bl-1", "project": "p", "state": "unreadable" },
                       "outcome": { "state": "pending" }, "steps": 1, "wall_secs": 2,
                       "verdicts": [] },
                     { "diff": { "ball_id": "bl-1", "project": "p", "state": "diff",
                                 "handle": "at-1", "target": "work/bl-1",
                                 "source": "attempt/at-1", "target_oid": "aaa",
                                 "source_oid": "bbb", "truncated": false,
                                 "files": [{ "path": "src/a.rs", "added": 3, "removed": 1 }] },
                       "outcome": { "state": "accepted", "commit": "ccc" }, "steps": 1,
                       "wall_secs": 2, "verdicts": [{ "sender": "judge", "body": "cleaner" }] }] })
    .to_string()
    .into_bytes()
}

pub(super) fn delivered(commit: bool) -> Vec<u8> {
    let mut body = json!({ "ok": true, "kind": "delivered", "base": "aaa", "target": "main" });
    if commit && let Some(map) = body.as_object_mut() {
        map.insert("commit".to_owned(), json!("ccc"));
    }
    body.to_string().into_bytes()
}

pub(super) fn retired(discarded: bool) -> Vec<u8> {
    json!({ "ok": true, "kind": "retired", "discarded": discarded })
        .to_string()
        .into_bytes()
}

pub(super) fn fanned(n: usize) -> Vec<u8> {
    let rows: Vec<Value> = (0..n)
        .map(|_| {
            json!({ "workspace": "home", "binding": "/candidate", "lineage": Value::Null,
                    "goal": "g", "origin": "balls" })
        })
        .collect();
    json!({ "ok": true, "kind": "fanned", "rows": rows })
        .to_string()
        .into_bytes()
}

pub(super) fn started() -> Vec<u8> {
    json!({ "ok": true, "kind": "started", "conversation": "c-1" })
        .to_string()
        .into_bytes()
}

/// The scripts a focused workspace costs before any gesture: the first pass,
/// the preload, and the pass the focus woke.
pub(super) fn focused_scripts() -> Vec<Vec<Vec<u8>>> {
    vec![
        vec![ws_reply()],
        vec![nothing_set()],
        vec![ws_reply()],
        vec![conv_reply()],
    ]
}

/// The pass every gesture wakes after it.
pub(super) fn after() -> Vec<Vec<Vec<u8>>> {
    vec![vec![ws_reply()], vec![conv_reply()]]
}

pub(super) fn focused(model: &mut super::Model) {
    settle(model, &|s| !s.workspaces.is_empty());
    model.focus_workspace(Some("home".into()));
    settle(model, &|s| !s.conversations.is_empty());
}

/// **The read is aimed at the workspace the operator is standing in**, and the
/// row's handle is what says which acts it earns.
#[test]
fn opening_the_candidates_asks_science_for_the_focused_workspace() {
    let mut scripts = focused_scripts();
    scripts.push(vec![science()]);
    scripts.extend(after());
    let (mut model, served) = super::model_against(scripts);
    focused(&mut model);
    model.list_candidates();
    let snap = settle(&mut model, &|s| s.candidates.is_some());
    let spread = snap.candidates.unwrap_or_else(|| unreachable!());
    assert!(spread.about("home"));
    assert_eq!(
        spread
            .rows
            .iter()
            .map(|row| row.diff.handle.clone())
            .collect::<Vec<String>>(),
        [String::new(), "at-1".to_owned()]
    );
    drop(model);
    let requests = served.join().unwrap();
    assert_eq!(
        serde_json::from_slice::<Value>(&requests[4]).unwrap(),
        json!({ "op": "science", "workspace": "home" })
    );
}

/// **Asked with nothing focused, it crosses nothing** and the sentence is the
/// one every focused read gives.
#[test]
fn the_listing_with_no_workspace_focused_crosses_nothing() {
    let (mut model, served) = super::model_against(vec![vec![ws_reply()], vec![ws_reply()]]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.list_candidates();
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(snap.error.as_deref(), Some("no workspace is focused"));
    assert!(snap.candidates.is_none());
    drop(model);
    assert!(
        ops(&served.join().unwrap())
            .iter()
            .all(|op| op == "workspaces"),
        "nothing but the pass crossed"
    );
}

/// An answer of another kind names the read and keeps what was there.
#[test]
fn an_answer_of_the_wrong_kind_names_science_and_keeps_what_was_there() {
    let mut scripts = focused_scripts();
    scripts.push(vec![science()]);
    scripts.extend(after());
    scripts.push(vec![tr_reply()]);
    scripts.extend(after());
    let (mut model, served) = super::model_against(scripts);
    focused(&mut model);
    model.list_candidates();
    settle(&mut model, &|s| s.candidates.is_some());
    model.list_candidates();
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(
        snap.error.as_deref(),
        Some("science: the engine answered transcript instead")
    );
    assert!(snap.candidates.is_some_and(|spread| spread.rows.len() == 2));
    drop(model);
    served.join().unwrap();
}
