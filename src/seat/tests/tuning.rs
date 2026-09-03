//! **The tuning pair's gestures** (REMOTE §9.4, bl-dfbb): the two envelopes
//! the engine reads back, the role this seat spends, and the two sentences a
//! misdirected one earns.

use super::pick::applied;
use super::{conv_reply, nothing_set, settle, ws_reply};
use serde_json::{Value, json};

/// **The two tuning gestures** (REMOTE §9.4, bl-dfbb): the envelopes the
/// engine reads back, the role this seat spends, and the receipt each earns.
/// `off` is a real null and not a fourth word.
#[test]
fn the_tuning_gestures_state_the_role_and_the_level_the_wire_spells() {
    let (mut model, served) = super::model_against(vec![
        vec![ws_reply()],
        vec![nothing_set()],
        vec![ws_reply()],
        vec![conv_reply()],
        vec![applied()], // effort high
        vec![nothing_set()],
        vec![ws_reply()],
        vec![conv_reply()],
        vec![applied()], // effort off
        vec![nothing_set()],
        vec![ws_reply()],
        vec![conv_reply()],
        vec![applied()], // priority on
        vec![nothing_set()],
        vec![ws_reply()],
        vec![conv_reply()],
    ]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.focus_workspace(Some("home".into()));
    settle(&mut model, &|s| !s.conversations.is_empty());

    model.set_effort(Some(crate::codec::Effort::High));
    settle(&mut model, &|s| {
        s.error.is_none() && !s.conversations.is_empty()
    });
    model.set_effort(None);
    settle(&mut model, &|s| {
        s.error.is_none() && !s.conversations.is_empty()
    });
    model.set_priority(true);
    settle(&mut model, &|s| {
        s.error.is_none() && !s.conversations.is_empty()
    });

    drop(model);
    let requests = served.join().unwrap();
    assert_eq!(
        serde_json::from_slice::<Value>(&requests[4]).unwrap(),
        json!({ "op": "effort", "workspace": "home", "role": "worker", "level": "high" })
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&requests[8]).unwrap(),
        json!({ "op": "effort", "workspace": "home", "role": "worker", "level": null })
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&requests[12]).unwrap(),
        json!({ "op": "priority", "workspace": "home", "role": "worker", "on": true })
    );
}

/// A tuning gesture with no workspace focused, and one answered with the
/// wrong kind — the same two sentences every other act on this row earns.
#[test]
fn tuning_with_nothing_focused_and_a_wrong_kind_both_name_themselves() {
    let (mut model, _s) = super::model_against(vec![vec![ws_reply()]]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.set_effort(Some(crate::codec::Effort::Low));
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(snap.error.as_deref(), Some("no workspace is focused"));
    model.set_priority(false);
    settle(&mut model, &|s| s.error.is_some());
    drop(model);

    let (mut model, _s) = super::model_against(vec![
        vec![ws_reply()],
        vec![nothing_set()],
        vec![ws_reply()],
        vec![conv_reply()],
        vec![ws_reply()], // the tuning act, answered with a roster
    ]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.focus_workspace(Some("home".into()));
    settle(&mut model, &|s| !s.conversations.is_empty());
    model.set_priority(true);
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(
        snap.error.as_deref(),
        Some("tune: the engine answered workspaces instead")
    );
}
