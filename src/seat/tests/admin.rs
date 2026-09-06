//! **The admin surface** (DESIGN §13.17): what each read asks, what each act
//! sends, and the one receipt of the five that carries a fact.
//!
//! What is load-bearing here is that **neither read's answer echoes what it
//! was asked about** — a config reply carries bytes and no destination, a
//! marks reply a branch and no workspace — so the ask names it, and that
//! pairing is what lets a screen paint a file under the destination it asked
//! for.

use serde_json::{Value, json};

use super::{conv_reply, nothing_set, ops, settle, tr_reply, ws_reply};

fn config(text: &str) -> Vec<u8> {
    json!({ "ok": true, "kind": "config", "settings": [], "text": text })
        .to_string()
        .into_bytes()
}

fn marks(branch: &str) -> Vec<u8> {
    json!({ "ok": true, "kind": "marks", "branch": branch })
        .to_string()
        .into_bytes()
}

fn deleted() -> Vec<u8> {
    json!({ "ok": true, "kind": "deleted" })
        .to_string()
        .into_bytes()
}

fn applied() -> Vec<u8> {
    json!({ "ok": true, "kind": "applied" })
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

fn frame(requests: &[Vec<u8>], at: usize) -> Value {
    serde_json::from_slice(requests.get(at).unwrap_or_else(|| unreachable!())).unwrap()
}

/// **The read names its destination and the value comes back carrying it**,
/// because the answer states none.
#[test]
fn a_config_read_carries_its_destination_into_the_value() {
    let mut scripts = focused_scripts();
    scripts.push(vec![config("roles: []")]);
    scripts.extend(after());
    let (mut model, served) = super::model_against(scripts);
    focused(&mut model);
    model.read_config(crate::codec::Destination::Cadence);
    let snap = settle(&mut model, &|s| s.config.is_some());
    let held = snap.config.unwrap_or_else(|| unreachable!());
    assert_eq!(
        (held.at.file(), held.text.as_str()),
        ("cadence", "roles: []")
    );
    drop(model);
    let requests = served.join().unwrap();
    assert_eq!(
        frame(&requests, 4),
        json!({ "op": "config", "target": { "file": "cadence" } })
    );
}

/// **The mark is aimed**, and the value says which workspace it is about.
#[test]
fn a_marks_read_is_aimed_and_refuses_without_a_workspace() {
    let mut scripts = focused_scripts();
    scripts.push(vec![marks("balls/tasks")]);
    scripts.extend(after());
    let (mut model, served) = super::model_against(scripts);
    focused(&mut model);
    model.read_marks();
    let snap = settle(&mut model, &|s| s.marks.is_some());
    let held = snap.marks.unwrap_or_else(|| unreachable!());
    assert!(held.about("home"));
    assert_eq!(held.branch, "balls/tasks");
    drop(model);
    let requests = served.join().unwrap();
    assert_eq!(
        frame(&requests, 4),
        json!({ "op": "marks", "workspace": "home" })
    );

    let (mut model, served) = super::model_against(vec![vec![ws_reply()], vec![ws_reply()]]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.read_marks();
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(snap.error.as_deref(), Some("no workspace is focused"));
    drop(model);
    assert!(
        ops(&served.join().unwrap())
            .iter()
            .all(|op| op == "workspaces")
    );
}

/// An answer of another kind at the marks read names it and keeps what was
/// there — `configured`'s rule on the other admin read.
#[test]
fn a_wrong_kind_at_a_marks_read_names_it_and_keeps_what_was_there() {
    let mut scripts = focused_scripts();
    scripts.push(vec![marks("balls/tasks")]);
    scripts.extend(after());
    scripts.push(vec![tr_reply()]);
    scripts.extend(after());
    let (mut model, served) = super::model_against(scripts);
    focused(&mut model);
    model.read_marks();
    settle(&mut model, &|s| s.marks.is_some());
    model.read_marks();
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(
        snap.error.as_deref(),
        Some("marks: the engine answered transcript instead")
    );
    assert!(snap.marks.is_some_and(|held| held.branch == "balls/tasks"));
    drop(model);
    served.join().unwrap();
}

/// **A `marks` write answers with the branch it landed on**, and that is the
/// engine's own re-read — so nothing here asks again.
#[test]
fn a_marks_write_folds_the_branch_the_engine_re_read() {
    let mut scripts = focused_scripts();
    scripts.push(vec![marks("balls/agents/corp")]);
    scripts.extend(after());
    let (mut model, served) = super::model_against(scripts);
    focused(&mut model);
    model.admin(crate::codec::AdminAct::Marks {
        workspace: "home".into(),
        branch: "balls/agents/corp".into(),
    });
    let snap = settle(&mut model, &|s| s.marks.is_some());
    assert_eq!(
        snap.marks.map(|held| held.branch),
        Some("balls/agents/corp".to_owned())
    );
    drop(model);
    let requests = served.join().unwrap();
    assert_eq!(
        frame(&requests, 4),
        json!({ "op": "marks", "workspace": "home", "branch": "balls/agents/corp" })
    );
}

/// **The other four acts take and say nothing**, each in the receipt shape its
/// own op earns.
#[test]
fn the_other_four_acts_are_taken_in_their_own_receipt_shapes() {
    use crate::codec::{AdminAct, Destination};
    let fired: Vec<(AdminAct, Vec<u8>)> = vec![
        (
            AdminAct::Config {
                at: Destination::LitanyModels,
                text: "models: {}".into(),
            },
            applied(),
        ),
        (
            AdminAct::Scan {
                workspace: "home".into(),
            },
            super::outcome(true, ""),
        ),
        (
            AdminAct::DeleteAgent {
                workspace: "home".into(),
                agent: "a1".into(),
                typed: String::new(),
            },
            deleted(),
        ),
        (
            AdminAct::DeleteWorkspace {
                workspace: "home".into(),
                typed: "home".into(),
            },
            deleted(),
        ),
    ];
    for (act, receipt) in fired {
        let mut scripts = focused_scripts();
        scripts.push(vec![receipt]);
        scripts.extend(after());
        let (mut model, served) = super::model_against(scripts);
        focused(&mut model);
        model.admin(act);
        let snap = settle(&mut model, &|s| !s.conversations.is_empty());
        assert_eq!(snap.error, None);
        drop(model);
        served.join().unwrap();
    }
}

/// **A refusal is the engine's own words**, and an answer of another kind
/// names the act.
#[test]
fn an_admin_act_refused_says_why_and_a_wrong_kind_names_it() {
    let mut scripts = focused_scripts();
    scripts.push(vec![super::outcome(false, "no such workspace")]);
    scripts.extend(after());
    scripts.push(vec![tr_reply()]);
    scripts.extend(after());
    let (mut model, served) = super::model_against(scripts);
    focused(&mut model);
    model.admin(crate::codec::AdminAct::Scan {
        workspace: "home".into(),
    });
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(
        snap.error.as_deref(),
        Some("scan refused: no such workspace")
    );
    model.admin(crate::codec::AdminAct::Scan {
        workspace: "home".into(),
    });
    let snap = settle(&mut model, &|s| {
        s.error.as_deref() != Some("scan refused: no such workspace")
    });
    assert_eq!(
        snap.error.as_deref(),
        Some("scan: the engine answered transcript instead")
    );
    drop(model);
    served.join().unwrap();
}

/// A config read of another kind names the read and keeps what was there.
#[test]
fn a_wrong_kind_at_a_config_read_names_it_and_keeps_what_was_there() {
    let mut scripts = focused_scripts();
    scripts.push(vec![config("roles: []")]);
    scripts.extend(after());
    scripts.push(vec![tr_reply()]);
    scripts.extend(after());
    let (mut model, served) = super::model_against(scripts);
    focused(&mut model);
    model.read_config(crate::codec::Destination::Cadence);
    settle(&mut model, &|s| s.config.is_some());
    model.read_config(crate::codec::Destination::Cadence);
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(
        snap.error.as_deref(),
        Some("config: the engine answered transcript instead")
    );
    assert!(snap.config.is_some_and(|held| held.text == "roles: []"));
    drop(model);
    served.join().unwrap();
}
