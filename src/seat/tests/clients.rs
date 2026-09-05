//! **The machines roster** (DESIGN §13.14): what an opening asks, and the two
//! lifetimes one row carries.
//!
//! What is load-bearing here is that the read is aimed and re-askable: a
//! registration is per workspace, and `present` is true only at the instant it
//! was answered, so the same gesture a minute later is a statement about the
//! world a minute later.

use serde_json::{Value, json};

use super::{conv_reply, nothing_set, ops, settle, tr_reply, ws_reply};

fn clients() -> Vec<u8> {
    json!({ "ok": true, "kind": "clients", "rows": [
        { "client": "laptop", "present": false, "tools": [
            { "name": "Bash", "description": "run a command",
              "input_schema": { "type": "object" }, "subject_cwd": true }] }] })
    .to_string()
    .into_bytes()
}

fn focused_scripts() -> Vec<Vec<Vec<u8>>> {
    vec![
        vec![ws_reply()],
        vec![nothing_set()],
        vec![ws_reply()],
        vec![conv_reply()],
    ]
}

fn after() -> Vec<Vec<Vec<u8>>> {
    vec![vec![ws_reply()], vec![conv_reply()]]
}

fn focused(model: &mut super::Model) {
    settle(model, &|s| !s.workspaces.is_empty());
    model.focus_workspace(Some("home".into()));
    settle(model, &|s| !s.conversations.is_empty());
}

/// **The read is aimed at the workspace the operator is standing in**, and a
/// machine that is not connected still says what it offers.
#[test]
fn opening_the_roster_asks_clients_for_the_focused_workspace() {
    let mut scripts = focused_scripts();
    scripts.push(vec![clients()]);
    scripts.extend(after());
    let (mut model, served) = super::model_against(scripts);
    focused(&mut model);
    model.list_clients();
    let snap = settle(&mut model, &|s| s.clients.is_some());
    let machines = snap.clients.unwrap_or_else(|| unreachable!());
    assert!(machines.about("home"));
    let row = machines
        .rows
        .first()
        .cloned()
        .unwrap_or_else(|| unreachable!());
    assert!(!row.present, "an observation, not a state");
    assert!(
        row.tools.first().is_some_and(|tool| tool.subject_cwd),
        "the statement stands whether or not it is connected"
    );
    drop(model);
    let requests = served.join().unwrap();
    assert_eq!(
        serde_json::from_slice::<Value>(&requests[4]).unwrap(),
        json!({ "op": "clients", "workspace": "home" })
    );
}

/// **Asked with nothing focused, it crosses nothing**: a registration is per
/// workspace, and there is nothing to ask without one.
#[test]
fn the_roster_with_no_workspace_focused_crosses_nothing() {
    let (mut model, served) = super::model_against(vec![vec![ws_reply()], vec![ws_reply()]]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.list_clients();
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(snap.error.as_deref(), Some("no workspace is focused"));
    assert!(snap.clients.is_none());
    drop(model);
    assert!(
        ops(&served.join().unwrap())
            .iter()
            .all(|op| op == "workspaces")
    );
}

/// An answer of another kind names the read and keeps what was there.
#[test]
fn an_answer_of_the_wrong_kind_names_clients_and_keeps_what_was_there() {
    let mut scripts = focused_scripts();
    scripts.push(vec![clients()]);
    scripts.extend(after());
    scripts.push(vec![tr_reply()]);
    scripts.extend(after());
    let (mut model, served) = super::model_against(scripts);
    focused(&mut model);
    model.list_clients();
    settle(&mut model, &|s| s.clients.is_some());
    model.list_clients();
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(
        snap.error.as_deref(),
        Some("clients: the engine answered transcript instead")
    );
    assert!(
        snap.clients
            .is_some_and(|machines| machines.rows.len() == 1)
    );
    drop(model);
    served.join().unwrap();
}
