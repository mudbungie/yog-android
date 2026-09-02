//! The provider/model family, both directions of the strictness contract:
//! the engine's spelling reads back, a real null is a fact, every malformed
//! shape refuses by name, and the pick's envelope is pinned byte for byte.

use super::{
    Effort, LEVELS, encode_effort, encode_pick, encode_priority, level_of, names, row, tunable,
};
use crate::codec::{Act, Ask, Gesture, encode};
use serde_json::{Value, json};

#[test]
fn a_provider_row_reads_its_name_and_both_facts() {
    let r = row(&json!({ "name": "anthropic", "fact": "credential present", "effort": true, "priority": true,
                         "blocked": null }))
    .unwrap();
    assert_eq!(r.name, "anthropic");
    assert_eq!(r.fact, "credential present");
    assert_eq!(r.blocked, None);
    let r = row(
        &json!({ "name": "openai", "fact": "no credential", "effort": false, "priority": false,
                         "blocked": "no login flow" }),
    )
    .unwrap();
    assert_eq!(r.blocked.as_deref(), Some("no login flow"));
    // The capability booleans the row states about itself (bl-dfbb): what a
    // seat may ask this provider for, carried and never derived.
    assert!(!r.effort && !r.priority);
}

/// The three ways a row is not one, each naming what it is not.
#[test]
fn a_malformed_provider_row_refuses_by_name() {
    assert!(row(&json!("anthropic")).unwrap_err().contains("not a JSON"));
    let missing = row(&json!({ "fact": "credential present", "effort": true, "priority": true }))
        .unwrap_err();
    assert!(missing.contains("name"), "{missing}");
    let mistyped =
        row(&json!({ "name": "a", "fact": 7, "effort": true, "priority": true })).unwrap_err();
    assert!(mistyped.contains("fact"), "{mistyped}");
    let blocked =
        row(&json!({ "name": "a", "fact": "b", "effort": true, "priority": true, "blocked": 7 }))
            .unwrap_err();
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

/// The level vocabulary, closed and round-tripping — and `off`, which is the
/// absence of a level rather than a fourth word for one.
#[test]
fn the_level_vocabulary_is_three_words_and_an_absence() {
    for level in [Effort::Low, Effort::Medium, Effort::High] {
        assert_eq!(Effort::parse(&level.as_str()), Some(level));
        assert_eq!(Effort::label(Some(level)), level.as_str());
    }
    assert_eq!(Effort::label(None), "off");
    assert_eq!(Effort::parse("off"), None, "off is not a level");
    assert_eq!(Effort::parse("HIGH"), None, "the vocabulary is exact");
    assert_eq!(Effort::parse(""), None);
    // What a chooser offers, in the order it offers it.
    assert_eq!(
        LEVELS.map(Effort::label).to_vec(),
        ["low", "medium", "high", "off"]
    );
}

/// Both tuning envelopes, in the server's own spelling. The level key is
/// written always — a real null for off, which is what the engine reads.
#[test]
fn the_tuning_envelopes_are_the_servers_spelling() {
    assert_eq!(
        encode_effort("ws", "worker", Some(Effort::Low)),
        json!({ "op": "effort", "workspace": "ws", "role": "worker", "level": "low" })
    );
    assert_eq!(
        encode_effort("ws", "worker", None),
        json!({ "op": "effort", "workspace": "ws", "role": "worker", "level": null })
    );
    assert_eq!(
        encode_priority("ws", "compactor", true),
        json!({ "op": "priority", "workspace": "ws", "role": "compactor", "on": true })
    );
    assert_eq!(
        encode(&Gesture::Act(Act::Effort {
            workspace: "ws".into(),
            role: "worker".into(),
            level: Some(Effort::High),
        })),
        encode_effort("ws", "worker", Some(Effort::High))
    );
    assert_eq!(
        encode(&Gesture::Act(Act::Priority {
            workspace: "ws".into(),
            role: "worker".into(),
            on: false,
        })),
        encode_priority("ws", "worker", false)
    );
}

/// A level read back strictly: the vocabulary is closed, so a word outside it
/// is a codec that has drifted rather than an operator's typo, and it refuses
/// naming what it got.
#[test]
fn a_level_outside_the_vocabulary_refuses_naming_it() {
    let read = |v: Value| level_of(v.as_object().unwrap());
    assert_eq!(
        read(json!({ "level": "medium" })).unwrap(),
        Some(Effort::Medium)
    );
    assert_eq!(read(json!({ "level": null })).unwrap(), None);
    assert_eq!(read(json!({})).unwrap(), None, "absent reads as off");
    let why = read(json!({ "level": "extreme" })).unwrap_err();
    assert!(
        why.contains("extreme") && why.contains("low|medium|high|off"),
        "{why}"
    );
    assert!(read(json!({ "level": 7 })).is_err());
}

/// **What the controls may offer** (bl-dfbb): the capability of the selected
/// provider's own row, and nothing at all for a provider this seat has not
/// picked or the engine did not list.
#[test]
fn the_gate_is_the_selected_providers_own_row() {
    let rows = vec![
        row(
            &json!({ "name": "acme", "fact": "credential present", "blocked": null,
                     "effort": true, "priority": false }),
        )
        .unwrap(),
        row(
            &json!({ "name": "rival", "fact": "no credential", "blocked": null,
                     "effort": false, "priority": true }),
        )
        .unwrap(),
    ];
    assert_eq!(tunable(&rows, Some("acme")), (true, false));
    assert_eq!(tunable(&rows, Some("rival")), (false, true));
    assert_eq!(tunable(&rows, Some("nobody")), (false, false));
    assert_eq!(tunable(&rows, None), (false, false));
    assert_eq!(tunable(&[], Some("acme")), (false, false));
}
