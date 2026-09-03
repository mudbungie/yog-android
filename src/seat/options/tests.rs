//! What the selectors offer, and the two rules that bound it: a workspace
//! owns its options, and bytes that will not decode are simply absent.

use super::Options;
use crate::seat::{Focus, Snapshot};
use serde_json::json;

fn providers() -> serde_json::Value {
    json!({ "kind": "providers", "ok": true,
            "rows": [{ "name": "acme", "fact": "credential present", "effort": true, "priority": true, "blocked": null }] })
}

fn models() -> serde_json::Value {
    json!({ "kind": "models", "ok": true, "rows": ["opus", "sonnet"] })
}

fn focus(workspace: &str) -> Focus {
    Focus {
        workspace: Some(workspace.to_owned()),
        agent: None,
    }
}

#[test]
fn what_was_learned_under_a_focus_paints_under_it() {
    let mut options = Options::default();
    options.learned("home", None, providers());
    options.learned("home", Some("acme"), models());
    let mut snap = Snapshot::default();
    options.paint(&focus("home"), &mut snap);
    assert_eq!(snap.providers[0].name, "acme");
    assert_eq!(snap.models["acme"], ["opus", "sonnet"]);
    assert_eq!(options.workspace().as_deref(), Some("home"));
    let (stored, listed, _) = options.envelopes();
    assert_eq!(stored, Some(providers()));
    assert_eq!(listed["acme"], models());
}

/// The pairing law: another workspace's focus paints nothing, and learning
/// under another workspace drops what the first one held.
#[test]
fn options_belong_to_the_workspace_they_were_read_for() {
    let mut options = Options::default();
    options.learned("home", None, providers());
    options.learned("home", Some("acme"), models());
    let mut snap = Snapshot::default();
    options.paint(&focus("away"), &mut snap);
    assert!(snap.providers.is_empty() && snap.models.is_empty());
    options.paint(&Focus::default(), &mut snap);
    assert!(snap.providers.is_empty());

    options.learned("away", Some("acme"), models());
    let (stored, _, _) = options.envelopes();
    assert_eq!(
        stored, None,
        "the first workspace's list went with its focus"
    );
    let mut snap = Snapshot::default();
    options.paint(&focus("away"), &mut snap);
    assert!(snap.providers.is_empty());
    assert_eq!(snap.models["acme"], ["opus", "sonnet"]);
}

/// Nothing learned paints nothing, whatever the focus — and an envelope that
/// will not decode is absent rather than an error: these bytes decoded once
/// when they arrived (they can only be a tampered cache), and an empty
/// selector is the honest answer.
#[test]
fn nothing_learned_and_nothing_readable_both_paint_nothing() {
    let mut snap = Snapshot::default();
    Options::default().paint(&focus("home"), &mut snap);
    assert!(snap.providers.is_empty() && snap.models.is_empty());

    let junk = Options::resumed(
        Some("home".to_owned()),
        Some(json!({ "kind": "workspaces", "ok": true, "rows": [] })),
        [("acme".to_owned(), json!(7))].into_iter().collect(),
        Some(json!({ "kind": "models", "ok": true, "rows": [] })),
    );
    junk.paint(&focus("home"), &mut snap);
    assert!(snap.providers.is_empty() && snap.models.is_empty());
}

/// The assignments are learned and painted under the same workspace rule as
/// the options beside them (bl-e9f9), and go with the focus.
#[test]
fn the_assignments_belong_to_their_workspace_too() {
    let assigned = json!({ "kind": "roles", "ok": true,
                           "rows": [{ "role": "worker", "provider": "acme",
                                      "model": "opus", "effort": null,
                                      "priority": false }] });
    let mut options = Options::default();
    options.assigned("home", assigned.clone());
    let mut snap = Snapshot::default();
    options.paint(&focus("home"), &mut snap);
    assert_eq!(snap.roles[0].provider, "acme");
    let (_, _, stored) = options.envelopes();
    assert_eq!(stored, Some(assigned.clone()));

    let mut snap = Snapshot::default();
    options.paint(&focus("away"), &mut snap);
    assert!(
        snap.roles.is_empty(),
        "another workspace's focus paints none"
    );

    // Learning under another workspace drops what the first one held.
    options.learned("away", None, providers());
    let (_, _, stored) = options.envelopes();
    assert_eq!(stored, None);
}
