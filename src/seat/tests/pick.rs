//! **The controls row's selectors** (bl-0267): the two reads that populate
//! them, the one act that spends them, and the workspace that owns all
//! three. The row's other half — the acts on a turn — is `turn`.

use super::{Model, REST, cache_in, conv_reply, material, ops, pki, serve_many, settle, ws_reply};
use crate::transport::Seat;
use serde_json::{Value, json};

fn providers_reply() -> Vec<u8> {
    json!({ "ok": true, "kind": "providers",
            "rows": [{ "name": "acme", "fact": "credential present", "effort": true, "priority": true, "blocked": null },
                     { "name": "rival", "fact": "no credential", "effort": false, "priority": false, "blocked": "no login flow" }] })
    .to_string()
    .into_bytes()
}

fn models_reply() -> Vec<u8> {
    json!({ "ok": true, "kind": "models", "rows": ["opus", "sonnet"] })
        .to_string()
        .into_bytes()
}

fn applied() -> Vec<u8> {
    json!({ "ok": true, "kind": "applied" })
        .to_string()
        .into_bytes()
}

/// The whole walk: list the workspace's providers, list one provider's
/// models, assign the worker's model — and the envelopes the engine read
/// back, because this device's side of the wire is pinned rather than
/// assumed.
#[test]
fn the_selectors_read_their_options_and_the_pick_states_the_assignment_whole() {
    let (mut model, served) = super::model_against(vec![
        vec![ws_reply()],        // boot
        vec![ws_reply()],        // focus: refresh…
        vec![conv_reply()],      // …two deep
        vec![providers_reply()], // the providers gesture
        vec![ws_reply()],
        vec![conv_reply()],
        vec![models_reply()], // the models gesture
        vec![ws_reply()],
        vec![conv_reply()],
        vec![applied()], // the pick
        vec![ws_reply()],
        vec![conv_reply()],
    ]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.focus_workspace(Some("home".into()));
    settle(&mut model, &|s| !s.conversations.is_empty());

    model.list_providers();
    let snap = settle(&mut model, &|s| !s.providers.is_empty());
    assert_eq!(snap.providers[0].name, "acme");
    assert_eq!(snap.providers[0].fact, "credential present");
    assert_eq!(snap.providers[1].blocked.as_deref(), Some("no login flow"));

    model.list_models("acme".into());
    let snap = settle(&mut model, &|s| !s.models.is_empty());
    assert_eq!(snap.models["acme"], ["opus", "sonnet"]);
    // And the options ride every later snapshot, not just the one that
    // learned them.
    assert_eq!(snap.providers.len(), 2);

    model.pick_model("acme".into(), "opus".into());
    let snap = settle(&mut model, &|s| s.error.is_none() && !s.models.is_empty());
    assert_eq!(snap.error, None);

    drop(model);
    let requests = served.join().unwrap();
    assert_eq!(
        ops(&requests),
        [
            "workspaces",
            "workspaces",
            "conversations",
            "providers",
            "workspaces",
            "conversations",
            "models",
            "workspaces",
            "conversations",
            "model",
            "workspaces",
            "conversations"
        ]
    );
    let asked: Value = serde_json::from_slice(&requests[3]).unwrap();
    assert_eq!(asked, json!({ "op": "providers", "workspace": "home" }));
    let asked: Value = serde_json::from_slice(&requests[6]).unwrap();
    assert_eq!(
        asked,
        json!({ "op": "models", "workspace": "home", "provider": "acme" })
    );
    // The pick names all four facts, and the role is this seat's one.
    let picked: Value = serde_json::from_slice(&requests[9]).unwrap();
    assert_eq!(
        picked,
        json!({ "op": "model", "workspace": "home", "role": "worker",
                "provider": "acme", "model": "opus" })
    );
}

/// Every selector gesture needs a workspace under it, and says so in one
/// sentence rather than dialling with a hole in the envelope.
#[test]
fn a_selector_gesture_with_no_workspace_focused_reaches_the_banner() {
    let (mut model, _served) = super::model_against(vec![vec![ws_reply()]]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.list_providers();
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(snap.error.as_deref(), Some("no workspace is focused"));
    model.list_models("acme".into());
    settle(&mut model, &|s| s.error.is_some());
    model.pick_model("acme".into(), "opus".into());
    settle(&mut model, &|s| s.error.is_some());
}

/// A wrong kind under a selector's own gesture names it, exactly as the
/// standing reads do — the options are not a lane with its own manners.
#[test]
fn wrong_kinds_under_the_selectors_name_the_kind() {
    let (mut model, _s) = super::model_against(vec![
        vec![ws_reply()],
        vec![ws_reply()],
        vec![conv_reply()],
        vec![ws_reply()], // providers answered with a roster
    ]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.focus_workspace(Some("home".into()));
    settle(&mut model, &|s| !s.conversations.is_empty());
    model.list_providers();
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(
        snap.error.as_deref(),
        Some("providers: the engine answered workspaces instead")
    );
    drop(model);

    let (mut model, _s) = super::model_against(vec![
        vec![ws_reply()],
        vec![ws_reply()],
        vec![conv_reply()],
        vec![ws_reply()], // models answered with a roster
    ]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.focus_workspace(Some("home".into()));
    settle(&mut model, &|s| !s.conversations.is_empty());
    model.list_models("acme".into());
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(
        snap.error.as_deref(),
        Some("models: the engine answered workspaces instead")
    );
    drop(model);

    let (mut model, _s) = super::model_against(vec![
        vec![ws_reply()],
        vec![ws_reply()],
        vec![conv_reply()],
        vec![ws_reply()], // the pick answered with a roster
    ]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.focus_workspace(Some("home".into()));
    settle(&mut model, &|s| !s.conversations.is_empty());
    model.pick_model("acme".into(), "opus".into());
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(
        snap.error.as_deref(),
        Some("model: the engine answered workspaces instead")
    );
}

/// A resumed seat opens its selectors on what the cache holds, before the
/// wire answers anything — the §14 mechanism carrying the options too.
#[test]
fn a_second_boot_offers_the_options_it_had() {
    let dir = pki();
    let at = cache_in(&dir);
    let (address, served) = serve_many(
        &dir,
        "ca",
        "server",
        vec![
            vec![ws_reply()],
            vec![ws_reply()],
            vec![conv_reply()],
            vec![providers_reply()],
            vec![ws_reply()],
            vec![conv_reply()],
        ],
    );
    let seat = Seat::open(&material(&dir, "ca", "client", &address)).unwrap();
    let mut model = Model::start(seat, REST, at.clone());
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.focus_workspace(Some("home".into()));
    settle(&mut model, &|s| !s.conversations.is_empty());
    model.list_providers();
    settle(&mut model, &|s| !s.providers.is_empty());
    drop(model);
    served.join().unwrap();

    let seat = Seat::open(&material(&dir, "ca", "client", "127.0.0.1:1")).unwrap();
    let mut model = Model::start(seat, REST, at);
    let snap = model.snapshot();
    assert_eq!(snap.focus.workspace.as_deref(), Some("home"));
    assert_eq!(snap.providers[0].name, "acme");
}

/// **The two tuning gestures** (REMOTE §9.4, bl-dfbb): the envelopes the
/// engine reads back, the role this seat spends, and the receipt each earns.
/// `off` is a real null and not a fourth word.
#[test]
fn the_tuning_gestures_state_the_role_and_the_level_the_wire_spells() {
    let (mut model, served) = super::model_against(vec![
        vec![ws_reply()],
        vec![ws_reply()],
        vec![conv_reply()],
        vec![applied()], // effort high
        vec![ws_reply()],
        vec![conv_reply()],
        vec![applied()], // effort off
        vec![ws_reply()],
        vec![conv_reply()],
        vec![applied()], // priority on
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
        serde_json::from_slice::<Value>(&requests[3]).unwrap(),
        json!({ "op": "effort", "workspace": "home", "role": "worker", "level": "high" })
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&requests[6]).unwrap(),
        json!({ "op": "effort", "workspace": "home", "role": "worker", "level": null })
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&requests[9]).unwrap(),
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
