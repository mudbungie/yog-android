//! **What the controls load** (REMOTE §9.4's read, bl-e9f9): the assignments
//! a workspace actually has, the count that tells an optimistic control it
//! has been overtaken, and the older engine that cannot answer at all.

use super::pick::applied;
use super::{conv_reply, model_against, ops, settle, ws_reply};
use serde_json::{Value, json};

fn roles_reply(effort: Value, priority: bool) -> Vec<u8> {
    json!({ "ok": true, "kind": "roles",
            "rows": [{ "role": "worker", "provider": "acme", "model": "opus",
                       "effort": effort, "priority": priority }] })
    .to_string()
    .into_bytes()
}

/// **The controls load what the workspace actually has** (bl-e9f9): focusing
/// a workspace reads the assignments, and the read rides every snapshot after
/// it. The file's own word crosses whole — a level outside the gesture
/// vocabulary is shown, never flattened to nothing-set.
#[test]
fn focusing_a_workspace_reads_what_its_roles_are_set_to() {
    let (mut model, served) = super::model_against(vec![
        vec![ws_reply()],
        vec![roles_reply(json!("extreme"), true)], // the focus preload
        vec![ws_reply()],
        vec![conv_reply()],
    ]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.focus_workspace(Some("home".into()));
    let snap = settle(&mut model, &|s| !s.roles.is_empty());
    let worker = crate::codec::pick::worker(&snap.roles).unwrap_or_else(|| unreachable!());
    assert_eq!(
        (worker.provider.as_str(), worker.model.as_str()),
        ("acme", "opus")
    );
    assert_eq!(worker.effort.as_deref(), Some("extreme"));
    assert!(worker.priority);
    assert_eq!(
        snap.roles_read, 1,
        "the count is what tells a control it was overtaken"
    );
    drop(model);
    let requests = served.join().unwrap();
    assert_eq!(ops(&requests)[1], "roles");
    assert_eq!(
        serde_json::from_slice::<Value>(&requests[1]).unwrap(),
        json!({ "op": "roles", "workspace": "home" })
    );
}

/// **An engine that predates the read says nothing** (bl-e9f9): the deployed
/// build refuses the op in band by name, and that means *no preload* — never
/// a banner. An operator running the engine they have is not being told off
/// by this app.
#[test]
fn an_engine_without_the_read_is_silent_not_an_error() {
    let refusal = json!({ "ok": false, "error": "unknown op \"roles\"" })
        .to_string()
        .into_bytes();
    let (mut model, _served) = super::model_against(vec![
        vec![ws_reply()],
        vec![refusal], // the focus preload, refused by an older engine
        vec![ws_reply()],
        vec![conv_reply()],
    ]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.focus_workspace(Some("home".into()));
    let snap = settle(&mut model, &|s| !s.conversations.is_empty());
    assert_eq!(snap.error, None, "a missing preload is not an error");
    assert!(snap.roles.is_empty());
    assert_eq!(snap.roles_read, 0, "nothing was read, so nothing overtakes");
}

/// A tuning act is followed by the read that makes it true, so the optimistic
/// value on the glass is overtaken within one gesture rather than one cadence.
#[test]
fn a_tuning_act_is_followed_by_the_read_that_settles_it() {
    let (mut model, served) = super::model_against(vec![
        vec![ws_reply()],
        vec![roles_reply(json!(null), false)],
        vec![ws_reply()],
        vec![conv_reply()],
        vec![applied()],                         // the effort act
        vec![roles_reply(json!("high"), false)], // …and the read after it
        vec![ws_reply()],
        vec![conv_reply()],
    ]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.focus_workspace(Some("home".into()));
    settle(&mut model, &|s| s.roles_read == 1);
    model.set_effort(Some(crate::codec::Effort::High));
    let snap = settle(&mut model, &|s| s.roles_read == 2);
    let worker = crate::codec::pick::worker(&snap.roles).unwrap_or_else(|| unreachable!());
    assert_eq!(worker.effort.as_deref(), Some("high"));
    drop(model);
    assert_eq!(
        ops(&served.join().unwrap()),
        [
            "workspaces",
            "roles",
            "workspaces",
            "conversations",
            "effort",
            "roles",
            "workspaces",
            "conversations"
        ]
    );
}

/// The other way the preload can come back useless: an answer of the wrong
/// kind. It is the same silence — a preload is not a gesture the operator
/// made, so nothing it can do earns a banner — and the kind check is what
/// stops a roster being read as an assignment.
#[test]
fn a_preload_answered_with_the_wrong_kind_is_silent_too() {
    let (mut model, _served) = model_against(vec![
        vec![ws_reply()],
        vec![ws_reply()], // the preload, answered with a roster
        vec![ws_reply()],
        vec![conv_reply()],
    ]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.focus_workspace(Some("home".into()));
    let snap = settle(&mut model, &|s| !s.conversations.is_empty());
    assert_eq!(snap.error, None);
    assert!(snap.roles.is_empty());
    assert_eq!(snap.roles_read, 0);
}
