//! The provider/model family, both directions of the strictness contract:
//! the engine's spelling reads back, a real null is a fact, every malformed
//! shape refuses by name, and the pick's envelope is pinned byte for byte.

use super::{encode_pick, names, row};
use crate::codec::{Act, Ask, Gesture, encode};
use serde_json::json;

#[test]
fn a_provider_row_reads_its_name_and_both_facts() {
    let r = row(&json!({ "name": "anthropic", "fact": "credential present",
                         "blocked": null }))
    .unwrap();
    assert_eq!(r.name, "anthropic");
    assert_eq!(r.fact, "credential present");
    assert_eq!(r.blocked, None);
    let r = row(&json!({ "name": "openai", "fact": "no credential",
                         "blocked": "no login flow" }))
    .unwrap();
    assert_eq!(r.blocked.as_deref(), Some("no login flow"));
}

/// The three ways a row is not one, each naming what it is not.
#[test]
fn a_malformed_provider_row_refuses_by_name() {
    assert!(row(&json!("anthropic")).unwrap_err().contains("not a JSON"));
    let missing = row(&json!({ "fact": "credential present" })).unwrap_err();
    assert!(missing.contains("name"), "{missing}");
    let mistyped = row(&json!({ "name": "a", "fact": 7 })).unwrap_err();
    assert!(mistyped.contains("fact"), "{mistyped}");
    let blocked = row(&json!({ "name": "a", "fact": "b", "blocked": 7 })).unwrap_err();
    assert!(blocked.contains("blocked"), "{blocked}");
}

/// The models listing is bare names, and a row that is not one refuses —
/// this codec does not flatten a shape it was not told.
#[test]
fn models_read_as_names_and_refuse_anything_else() {
    let o = json!({ "kind": "models", "ok": true, "rows": ["opus", "sonnet"] });
    let listed = names(o.as_object().unwrap()).unwrap();
    assert_eq!(listed, ["opus", "sonnet"]);
    let o = json!({ "rows": [{ "name": "opus" }] });
    let why = names(o.as_object().unwrap()).unwrap_err();
    assert!(why.contains("non-string row"), "{why}");
    let o = json!({ "kind": "models", "ok": true });
    assert!(names(o.as_object().unwrap()).is_err());
}

/// The pick states all four facts, in the server's own spelling.
#[test]
fn the_pick_envelope_is_the_servers_spelling() {
    assert_eq!(
        encode_pick("ws", "worker", "codex", "gpt-5.4"),
        json!({ "op": "model", "workspace": "ws", "role": "worker",
                "provider": "codex", "model": "gpt-5.4" })
    );
    assert_eq!(
        encode(&Gesture::Act(Act::PickModel {
            workspace: "ws".into(),
            role: "worker".into(),
            provider: "codex".into(),
            model: "gpt-5.4".into(),
        })),
        encode_pick("ws", "worker", "codex", "gpt-5.4")
    );
}

/// Both reads name their workspace: sign-ins live per workspace, so neither
/// is a global fact (bl-0267).
#[test]
fn both_reads_name_their_workspace() {
    assert_eq!(
        encode(&Gesture::Ask(Ask::Providers {
            workspace: "home".into()
        })),
        json!({ "op": "providers", "workspace": "home" })
    );
    assert_eq!(
        encode(&Gesture::Ask(Ask::Models {
            workspace: "home".into(),
            provider: "acme".into(),
        })),
        json!({ "op": "models", "workspace": "home", "provider": "acme" })
    );
}
