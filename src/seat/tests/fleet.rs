//! **The two armings a workspace carries** (DESIGN §13.13): what crosses, and
//! the receipt that says a STATE without saying which SETTING.
//!
//! What is load-bearing here is that last part. All four ops answer one shape,
//! so the sentence an operator reads is composed from the answer and the
//! gesture together — and a seat that read the reply alone would be guessing
//! between the loop and the monitor.

use serde_json::{Value, json};

use super::{conv_reply, nothing_set, ops, settle, tr_reply, ws_reply};
use crate::codec::FleetAct;

fn armed(armed: bool) -> Vec<u8> {
    json!({ "ok": true, "kind": "armed", "armed": armed })
        .to_string()
        .into_bytes()
}

/// The scripts a focused workspace costs before any gesture.
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

/// **The loop's two acts cross with the workspace the screen is painted
/// under**, and each receipt is read back under the op that earned it.
#[test]
fn the_loop_is_armed_and_disbanded_and_each_receipt_names_its_own_family() {
    let mut scripts = focused_scripts();
    scripts.push(vec![armed(true)]);
    scripts.extend(after());
    scripts.push(vec![armed(false)]);
    scripts.extend(after());
    let (mut model, served) = super::model_against(scripts);
    focused(&mut model);
    model.fleet_act(FleetAct::Fleet {
        project: "p".into(),
        cap: 4,
    });
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(snap.error.as_deref(), Some("fleet: the loop is armed"));
    model.fleet_act(FleetAct::Disband);
    let snap = settle(&mut model, &|s| {
        s.error.as_deref() == Some("disband: the loop is not armed")
    });
    assert!(snap.error.is_some());
    drop(model);
    let requests = served.join().unwrap();
    assert_eq!(
        serde_json::from_slice::<Value>(&requests[4]).unwrap(),
        json!({ "op": "fleet", "workspace": "home", "project": "p", "cap": 4 })
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&requests[7]).unwrap(),
        json!({ "op": "disband", "workspace": "home" })
    );
}

/// **The monitor's pair reads the SAME reply and says something else**, which
/// is the whole point of reading the op back.
#[test]
fn the_monitors_pair_reads_the_same_reply_and_says_the_other_thing() {
    let mut scripts = focused_scripts();
    scripts.push(vec![armed(true)]);
    scripts.extend(after());
    scripts.push(vec![armed(false)]);
    scripts.extend(after());
    let (mut model, served) = super::model_against(scripts);
    focused(&mut model);
    model.fleet_act(FleetAct::Arm {
        model: "haiku".into(),
    });
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(snap.error.as_deref(), Some("arm: the monitor is armed"));
    model.fleet_act(FleetAct::Disarm);
    settle(&mut model, &|s| {
        s.error.as_deref() == Some("disarm: the monitor is not armed")
    });
    drop(model);
    let requests = served.join().unwrap();
    assert_eq!(
        serde_json::from_slice::<Value>(&requests[4]).unwrap(),
        json!({ "op": "arm", "workspace": "home", "model": "haiku" })
    );
    assert_eq!(ops(&requests)[7], "disarm");
}

/// **Fired with no workspace focused, it crosses nothing**: these acts run
/// drones in ONE workspace, and there is nothing to arm without one.
#[test]
fn an_arming_with_no_workspace_focused_crosses_nothing() {
    let (mut model, served) = super::model_against(vec![vec![ws_reply()], vec![ws_reply()]]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.fleet_act(FleetAct::Disband);
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(snap.error.as_deref(), Some("no workspace is focused"));
    drop(model);
    assert!(
        ops(&served.join().unwrap())
            .iter()
            .all(|op| op == "workspaces")
    );
}

/// An answer of another kind names the op that was sent, never the one that
/// answered — and a lost reply leaves the arming in doubt, naming the board as
/// the read that settles it.
#[test]
fn a_wrong_kind_names_the_op_and_a_lost_reply_names_the_board() {
    let mut scripts = focused_scripts();
    scripts.push(vec![tr_reply()]);
    scripts.extend(after());
    let (mut model, served) = super::model_against(scripts);
    focused(&mut model);
    model.fleet_act(FleetAct::Disarm);
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(
        snap.error.as_deref(),
        Some("disarm: the engine answered transcript instead")
    );
    drop(model);
    served.join().unwrap();

    let mut turns = vec![
        super::Turn::Answer(vec![ws_reply()]),
        super::Turn::Answer(vec![nothing_set()]),
        super::Turn::Answer(vec![ws_reply()]),
        super::Turn::Answer(vec![conv_reply()]),
        super::Turn::Hangup,
    ];
    turns.extend([
        super::Turn::Answer(vec![ws_reply()]),
        super::Turn::Answer(vec![conv_reply()]),
    ]);
    let (mut model, _served) = super::model_turns(turns);
    focused(&mut model);
    model.fleet_act(FleetAct::Disband);
    let said = settle(&mut model, &|s| s.error.is_some())
        .error
        .unwrap_or_default();
    assert!(said.starts_with("disband may have run:"), "{said}");
    assert!(
        said.contains("The board says which loops are armed"),
        "{said}"
    );
}
