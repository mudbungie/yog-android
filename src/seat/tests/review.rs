//! **The work-review reads** (DESIGN §13.15): what each gesture asks, what
//! reaches the snapshot, and the two ways each can be asked for and not
//! answered.
//!
//! What is load-bearing here is that **the deeper ask carries the parameter
//! into the value**. Neither answer echoes what it was asked for — a `files`
//! reply has a preview and no path, a `work-diff` reply a patch and no address
//! — so the paint could not know which row the bytes belong under unless the
//! fold named it, and a preview under the wrong row is the defect §13.11's
//! echoed `seq` exists to prevent.

use serde_json::{Value, json};

use super::{conv_reply, nothing_set, ops, settle, tr_reply, ws_reply};

fn files(preview: bool) -> Vec<u8> {
    let mut body = json!({ "ok": true, "kind": "files", "worktree": true,
                           "truncated": false,
                           "rows": [{ "path": "src/a.rs", "size": 12, "dir": false }] });
    if preview && let Some(map) = body.as_object_mut() {
        map.insert(
            "preview".to_owned(),
            json!({ "kind": "text", "text": "fn main() {}" }),
        );
    }
    body.to_string().into_bytes()
}

fn churn(patch: bool) -> Vec<u8> {
    let mut body = json!({ "ok": true, "kind": "work-diff",
                           "rows": [{ "ball_id": "bl-1", "project": "p", "state": "diff",
                                      "handle": "at-1", "target": "main",
                                      "source": "attempt/at-1", "target_oid": "aaa",
                                      "source_oid": "bbb", "truncated": false,
                                      "files": [{ "path": "src/a.rs", "added": 3,
                                                  "removed": 1 }] }] });
    if patch && let Some(map) = body.as_object_mut() {
        map.insert(
            "patch".to_owned(),
            json!({ "kind": "text", "text": "@@ -1 +1 @@" }),
        );
    }
    body.to_string().into_bytes()
}

/// A workspace focused, its conversation opened, and a pass after each.
fn opened() -> Vec<Vec<Vec<u8>>> {
    vec![
        vec![ws_reply()],
        vec![nothing_set()],
        vec![ws_reply()],
        vec![conv_reply()],
        vec![tr_reply()],
    ]
}

fn after() -> Vec<Vec<Vec<u8>>> {
    vec![vec![ws_reply()], vec![conv_reply()], vec![tr_reply()]]
}

fn focused(model: &mut super::Model) {
    settle(model, &|s| !s.workspaces.is_empty());
    model.focus_conversation("home".into(), "a1".into());
    settle(model, &|s| {
        s.focus.agent.is_some() && !s.transcript.is_empty()
    });
}

fn frame(requests: &[Vec<u8>], at: usize) -> Value {
    serde_json::from_slice(requests.get(at).unwrap_or_else(|| unreachable!())).unwrap()
}

/// **Opening asks the bare listing**, addressed at the conversation the
/// operator is standing in, and the answer says which conversation it is
/// about.
#[test]
fn opening_the_files_asks_the_bare_listing_for_the_focused_conversation() {
    let mut scripts = opened();
    scripts.push(vec![files(false)]);
    scripts.extend(after());
    let (mut model, served) = super::model_against(scripts);
    focused(&mut model);
    model.open_files(None);
    let snap = settle(&mut model, &|s| s.files.is_some());
    let held = snap.files.unwrap_or_else(|| unreachable!());
    assert!(held.about("home", "a1"));
    assert_eq!(held.opened, "", "the bare listing names no path");
    assert_eq!(held.listing.rows.len(), 1);
    drop(model);
    let requests = served.join().unwrap();
    assert_eq!(
        frame(&requests, 5),
        json!({ "op": "files", "workspace": "home", "agent": "a1" })
    );
}

/// **A file's bytes are the same read one depth down**, and the path the ask
/// carried is what the value comes back with — the answer states none.
#[test]
fn a_files_ask_carries_its_path_into_the_value_because_the_answer_does_not() {
    let mut scripts = opened();
    scripts.push(vec![files(true)]);
    scripts.extend(after());
    let (mut model, served) = super::model_against(scripts);
    focused(&mut model);
    model.open_files(Some("src/a.rs".into()));
    let snap = settle(&mut model, &|s| s.files.is_some());
    let held = snap.files.unwrap_or_else(|| unreachable!());
    assert_eq!(held.opened, "src/a.rs");
    assert_eq!(
        held.listing.preview,
        Some(crate::codec::Preview::Text("fn main() {}".to_owned()))
    );
    drop(model);
    let requests = served.join().unwrap();
    assert_eq!(
        frame(&requests, 5),
        json!({ "op": "files", "workspace": "home", "agent": "a1", "path": "src/a.rs" })
    );
}

/// **Asked with no conversation focused, it crosses nothing**: the read is
/// about a conversation, and there is nothing to ask without one.
#[test]
fn the_worktree_with_no_conversation_focused_crosses_nothing() {
    let (mut model, served) = super::model_against(vec![vec![ws_reply()], vec![ws_reply()]]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.open_files(None);
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(
        snap.error.as_deref(),
        Some("files: no conversation is focused")
    );
    assert!(snap.files.is_none());
    drop(model);
    assert!(
        ops(&served.join().unwrap())
            .iter()
            .all(|op| op == "workspaces")
    );
}

/// An answer of another kind names the read and keeps what was there.
#[test]
fn an_answer_of_the_wrong_kind_names_files_and_keeps_what_was_there() {
    let mut scripts = opened();
    scripts.push(vec![files(false)]);
    scripts.extend(after());
    scripts.push(vec![tr_reply()]);
    scripts.extend(after());
    let (mut model, served) = super::model_against(scripts);
    focused(&mut model);
    model.open_files(None);
    settle(&mut model, &|s| s.files.is_some());
    model.open_files(None);
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(
        snap.error.as_deref(),
        Some("files: the engine answered transcript instead")
    );
    assert!(snap.files.is_some_and(|held| held.listing.rows.len() == 1));
    drop(model);
    served.join().unwrap();
}

/// **The work diff is aimed at the workspace**, and its deeper ask names the
/// file whose patch it wants — the address carried into the value for the
/// listing's reason exactly.
#[test]
fn the_work_diff_is_aimed_and_its_patch_ask_names_the_file_it_is_for() {
    let mut scripts = opened();
    scripts.push(vec![churn(false)]);
    scripts.extend(after());
    scripts.push(vec![churn(true)]);
    scripts.extend(after());
    let (mut model, served) = super::model_against(scripts);
    focused(&mut model);
    model.open_work(None);
    let snap = settle(&mut model, &|s| s.work.is_some());
    let held = snap.work.unwrap_or_else(|| unreachable!());
    assert!(held.about("home"));
    assert_eq!((held.opened, held.patch), (None, None));
    let file = crate::codec::WorkFile {
        ball: "bl-1".to_owned(),
        path: "src/a.rs".to_owned(),
        handle: "at-1".to_owned(),
    };
    model.open_work(Some(file.clone()));
    let snap = settle(&mut model, &|s| {
        s.work.as_ref().is_some_and(|work| work.patch.is_some())
    });
    let held = snap.work.unwrap_or_else(|| unreachable!());
    assert_eq!(held.opened, Some(file));
    drop(model);
    let requests = served.join().unwrap();
    assert_eq!(
        frame(&requests, 5),
        json!({ "op": "work-diff", "workspace": "home" })
    );
    assert_eq!(
        frame(&requests, 9),
        json!({ "op": "work-diff", "workspace": "home",
                "file": { "ball": "bl-1", "path": "src/a.rs", "handle": "at-1" } })
    );
}

/// **Asked with no workspace focused, it crosses nothing**, and an answer of
/// another kind names the read.
#[test]
fn the_work_diff_refuses_without_a_workspace_and_names_a_wrong_kind() {
    let (mut model, served) = super::model_against(vec![vec![ws_reply()], vec![ws_reply()]]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.open_work(None);
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(snap.error.as_deref(), Some("no workspace is focused"));
    assert!(snap.work.is_none());
    drop(model);
    served.join().unwrap();

    let mut scripts = opened();
    scripts.push(vec![tr_reply()]);
    scripts.extend(after());
    let (mut model, served) = super::model_against(scripts);
    focused(&mut model);
    model.open_work(None);
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(
        snap.error.as_deref(),
        Some("work-diff: the engine answered transcript instead")
    );
    drop(model);
    served.join().unwrap();
}
