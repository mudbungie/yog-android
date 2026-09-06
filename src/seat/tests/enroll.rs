//! **The mint** (REMOTE §8.4, DESIGN §13.18): what the act sends, what it
//! answers with, and the forgetting that is a gesture of its own.
//!
//! What is load-bearing here is that **the material rides back with the
//! outcome and lives in exactly one place**. No read can fetch it again — the
//! engine shredded the key as it answered — so a seat that dropped it would
//! have lost it, and a seat that wrote it down would have leaked it.

use serde_json::{Value, json};

use super::{conv_reply, nothing_set, ops, settle, tr_reply, ws_reply};

fn enrolled(grade: &str) -> Vec<u8> {
    json!({ "ok": true, "kind": "enrolled", "grade": grade, "name": "phone-2",
            "address": "engine.invalid:7737",
            "ca": "-----BEGIN CERTIFICATE-----\nnotreal\n-----END CERTIFICATE-----\n",
            "cert": "-----BEGIN CERTIFICATE-----\nnotreal\n-----END CERTIFICATE-----\n",
            "key": "-----BEGIN notreal KEY-----\nnotreal\n-----END notreal KEY-----\n" })
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

/// **The gesture names the workspace, the device and the grade**, and the
/// material comes back on the snapshot.
#[test]
fn a_mint_names_the_pair_and_the_material_rides_back_with_it() {
    let mut scripts = focused_scripts();
    scripts.push(vec![enrolled("foot")]);
    scripts.extend(after());
    let (mut model, served) = super::model_against(scripts);
    focused(&mut model);
    model.enroll("phone-2".into(), crate::leaf::Grade::Foot);
    let snap = settle(&mut model, &|s| s.minted.is_some());
    let held = snap.minted.unwrap_or_else(|| unreachable!());
    assert_eq!(
        (held.name.as_str(), held.grade),
        ("phone-2", crate::leaf::Grade::Foot)
    );
    // The envelope it displays is the one the NEXT device reads: one shape,
    // proved by the round trip rather than by two spellings agreeing.
    let written = crate::envelope::write(&held);
    assert!(written.contains("\"yog-enroll\":1"), "{written}");
    drop(model);
    let requests = served.join().unwrap();
    assert_eq!(
        frame(&requests, 4),
        json!({ "op": "enroll", "workspace": "home", "name": "phone-2", "grade": "foot" })
    );
}

/// **Forgetting is a gesture and it crosses no wire.** The material lives in
/// the worker's own memory and nowhere else, so dropping it there is the whole
/// of the act.
#[test]
fn forgetting_drops_the_material_and_sends_nothing() {
    let mut scripts = focused_scripts();
    scripts.push(vec![enrolled("operator")]);
    scripts.extend(after());
    scripts.extend(after());
    let (mut model, served) = super::model_against(scripts);
    focused(&mut model);
    model.enroll("phone-2".into(), crate::leaf::Grade::Operator);
    settle(&mut model, &|s| s.minted.is_some());
    model.forget();
    let snap = settle(&mut model, &|s| s.minted.is_none());
    assert_eq!(snap.error, None);
    drop(model);
    let wire = ops(&served.join().unwrap());
    assert_eq!(
        wire.iter().filter(|op| *op == "enroll").count(),
        1,
        "{wire:?}"
    );
}

/// **With no workspace focused it crosses nothing**: a registration is the
/// pair, and there is no pair without one. An answer of another kind names
/// the act.
#[test]
fn a_mint_with_no_workspace_crosses_nothing_and_a_wrong_kind_names_it() {
    let (mut model, served) = super::model_against(vec![vec![ws_reply()], vec![ws_reply()]]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.enroll("phone-2".into(), crate::leaf::Grade::Foot);
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(
        snap.error.as_deref(),
        Some("enroll: no workspace is focused")
    );
    assert!(snap.minted.is_none());
    drop(model);
    assert!(
        ops(&served.join().unwrap())
            .iter()
            .all(|op| op == "workspaces")
    );

    let mut scripts = focused_scripts();
    scripts.push(vec![tr_reply()]);
    scripts.extend(after());
    let (mut model, served) = super::model_against(scripts);
    focused(&mut model);
    model.enroll("phone-2".into(), crate::leaf::Grade::Foot);
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(
        snap.error.as_deref(),
        Some("enroll: the engine answered transcript instead")
    );
    assert!(snap.minted.is_none());
    drop(model);
    served.join().unwrap();
}
