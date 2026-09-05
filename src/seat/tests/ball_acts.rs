//! **The ball pane's five acts** (DESIGN §13.10): what crosses, what the
//! banner says, and the read that follows every one of them.
//!
//! What is load-bearing here is not the happy path. It is that the act is
//! stamped with the FOCUSED WORKSPACE and never with a name this seat
//! invented, and that a pane which is not a standing read is re-asked after
//! the act — a filing, a claim or a close is invisible until the view it
//! happened in is read again.

use serde_json::{Value, json};

use super::{conv_reply, nothing_set, ops, outcome, settle, ws_reply};
use crate::codec::{BallAct, Pane, View};

/// `workspace-balls`, one row, with the id the acts below address.
fn held(id: &str) -> Vec<u8> {
    json!({ "ok": true, "kind": "workspace-balls",
            "rows": [{ "id": id, "project": "p", "state": "ready", "owner": "home" }] })
    .to_string()
    .into_bytes()
}

/// A model standing on the aimed pane, with `serve` answering after it.
fn on_the_pane(after: Vec<Vec<Vec<u8>>>) -> (super::Model, super::JoinHandle) {
    let mut script = vec![
        vec![ws_reply()],
        vec![nothing_set()],
        vec![ws_reply()],
        vec![conv_reply()],
        vec![held("bl-1")],
        // The pass the pane's own gesture woke, which is what the act below
        // lands after.
        vec![ws_reply()],
        vec![conv_reply()],
    ];
    script.extend(after);
    let (mut model, served) = super::model_against(script);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.focus_workspace(Some("home".into()));
    settle(&mut model, &|s| !s.conversations.is_empty());
    model.list_balls(View::Here);
    settle(&mut model, &|s| s.pane.is_some());
    (model, served)
}

/// **The act carries the row's project and the FOCUS's workspace name**, and
/// the pane is read again straight after it — the act's own effect is
/// invisible until it is.
#[test]
fn an_act_is_stamped_with_the_focused_workspace_and_the_pane_is_read_again() {
    let (model, served) = on_the_pane(vec![
        vec![outcome(true, "")],
        vec![held("bl-2")],
        vec![ws_reply()],
        vec![conv_reply()],
    ]);
    model.ball_act("p".into(), BallAct::Assign { id: "bl-1".into() });
    let mut model = model;
    let snap = settle(
        &mut model,
        &|s| matches!(&s.pane, Some(Pane::Here(rows)) if rows.first().is_some_and(|r| r.id == "bl-2")),
    );
    assert!(snap.error.is_none(), "a taken act says nothing");
    drop(model);
    let requests = served.join().unwrap();
    assert_eq!(
        serde_json::from_slice::<Value>(&requests[7]).unwrap(),
        json!({ "op": "assign", "project": "p", "id": "bl-1", "name": "home" })
    );
    assert_eq!(ops(&requests)[8], "workspace-balls");
}

/// **A refusal arrives in the engine's own words**, named by the op that
/// earned it — nothing here re-words what the child said.
#[test]
fn a_refused_act_says_what_the_engine_said_and_names_the_op() {
    let (model, served) = on_the_pane(vec![
        vec![outcome(false, "bl: no such ball")],
        vec![held("bl-1")],
        vec![ws_reply()],
        vec![conv_reply()],
    ]);
    model.ball_act(
        "p".into(),
        BallAct::Close {
            id: "bl-404".into(),
        },
    );
    let mut model = model;
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(
        snap.error.as_deref(),
        Some("close refused: bl: no such ball")
    );
    drop(model);
    served.join().unwrap();
}

/// An answer of another kind is a sentence naming the op, the same shape
/// every act in this crate gives.
#[test]
fn an_answer_of_the_wrong_kind_names_the_op() {
    let (model, served) = on_the_pane(vec![
        vec![ws_reply()],
        vec![held("bl-1")],
        vec![ws_reply()],
        vec![conv_reply()],
    ]);
    model.ball_act(
        "p".into(),
        BallAct::Create {
            title: "a title".into(),
            body: None,
        },
    );
    let mut model = model;
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(
        snap.error.as_deref(),
        Some("create: the engine answered workspaces instead")
    );
    drop(model);
    served.join().unwrap();
}

/// **An act fired with no pane held asks for none afterwards.** The re-read is
/// the view's own, so there is nothing to re-ask when nothing was opened — and
/// this is also the arm that proves the stamp survives a pane that never was.
#[test]
fn an_act_with_no_pane_open_re_reads_nothing() {
    let (mut model, served) = super::model_against(vec![
        vec![ws_reply()],
        vec![nothing_set()],
        vec![ws_reply()],
        vec![conv_reply()],
        vec![outcome(true, "")],
        vec![ws_reply()],
        vec![conv_reply()],
    ]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.focus_workspace(Some("home".into()));
    settle(&mut model, &|s| !s.conversations.is_empty());
    model.ball_act(
        "p".into(),
        BallAct::Update {
            id: "bl-1".into(),
            title: Some("t".into()),
            body: None,
            note: None,
        },
    );
    settle(&mut model, &|s| s.pane.is_none());
    drop(model);
    let requests = served.join().unwrap();
    let asked = ops(&requests);
    assert!(asked.contains(&"update".to_owned()));
    assert!(
        !asked.contains(&"workspace-balls".to_owned()),
        "nothing was open, so nothing was re-read"
    );
}
