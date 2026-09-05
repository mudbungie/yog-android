//! The five spellings, both directions. The corpus is what proves them
//! against the server's own codec (`tests/conformance`); these are the arms
//! that file cannot reach — the refusals, and the absent-is-a-value rule at
//! the one site that writes it.

use super::{BallAct, decode, encode};
use serde_json::{Map, Value, json};

fn object(v: &Value) -> Map<String, Value> {
    v.as_object().cloned().unwrap()
}

#[test]
fn the_three_that_name_only_a_ball_spell_one_shape() {
    for (act, op) in [
        (BallAct::Assign { id: "bl-1".into() }, "assign"),
        (BallAct::Release { id: "bl-1".into() }, "release"),
        (BallAct::Close { id: "bl-1".into() }, "close"),
    ] {
        assert_eq!(act.op(), op);
        assert_eq!(
            encode("proj", "alba", &act),
            json!({ "op": op, "project": "proj", "id": "bl-1", "name": "alba" })
        );
    }
}

#[test]
fn an_absent_optional_key_is_omitted_and_never_nulled() {
    let bare = encode(
        "proj",
        "alba",
        &BallAct::Create {
            title: "a title".into(),
            body: None,
        },
    );
    assert_eq!(
        bare,
        json!({ "op": "create", "project": "proj", "name": "alba", "title": "a title" })
    );
    let bodied = encode(
        "proj",
        "alba",
        &BallAct::Create {
            title: "a title".into(),
            body: Some(String::new()),
        },
    );
    // An empty string is a VALUE — it asks the engine to blank the field — so
    // it is written, where an absent one is not.
    assert_eq!(bodied["body"], json!(""));
}

#[test]
fn an_update_writes_only_the_fields_it_was_given() {
    let act = BallAct::Update {
        id: "bl-1".into(),
        title: Some("t".into()),
        body: None,
        note: Some("n".into()),
    };
    assert_eq!(
        encode("proj", "alba", &act),
        json!({ "op": "update", "project": "proj", "id": "bl-1",
                "name": "alba", "title": "t", "note": "n" })
    );
}

#[test]
fn the_scheduling_fields_are_refused_by_name() {
    for op in ["create", "update"] {
        let frame = object(&json!({ "op": op, "project": "proj", "id": "bl-1",
                                    "name": "alba", "title": "t",
                                    "fields": [{ "field": "priority", "value": 2 }] }));
        let refusal = decode(op, &frame).unwrap_err();
        assert!(refusal.contains(op), "{refusal}");
        assert!(refusal.contains("fields"), "{refusal}");
    }
}

#[test]
fn an_op_this_family_does_not_hold_is_refused_by_name() {
    let frame = object(&json!({ "op": "drop", "project": "proj", "name": "alba" }));
    assert!(decode("drop", &frame).unwrap_err().contains("drop"));
}

#[test]
fn a_missing_field_is_refused_rather_than_defaulted() {
    let frame = object(&json!({ "op": "assign", "project": "proj", "name": "alba" }));
    assert!(decode("assign", &frame).is_err());
}

#[test]
fn the_sentence_a_control_states_is_the_field_it_wants() {
    assert_eq!(BallAct::Assign { id: String::new() }.wants(), None);
    assert!(
        BallAct::Create {
            title: String::new(),
            body: None,
        }
        .wants()
        .is_some()
    );
    assert!(
        BallAct::Update {
            id: String::new(),
            title: None,
            body: None,
            note: None,
        }
        .wants()
        .is_some()
    );
}

#[test]
fn the_text_lands_in_whichever_field_the_act_takes() {
    let empty = || String::new();
    assert_eq!(
        BallAct::Assign { id: empty() }.on("bl-9".into(), "typed".into()),
        BallAct::Assign { id: "bl-9".into() }
    );
    assert_eq!(
        BallAct::Release { id: empty() }.on("bl-9".into(), "typed".into()),
        BallAct::Release { id: "bl-9".into() }
    );
    assert_eq!(
        BallAct::Close { id: empty() }.on("bl-9".into(), "typed".into()),
        BallAct::Close { id: "bl-9".into() }
    );
    assert_eq!(
        BallAct::Create {
            title: empty(),
            body: None
        }
        .on("bl-9".into(), "typed".into()),
        BallAct::Create {
            title: "typed".into(),
            body: None
        }
    );
    assert_eq!(
        BallAct::Update {
            id: empty(),
            title: None,
            body: None,
            note: None,
        }
        .on("bl-9".into(), "typed".into()),
        BallAct::Update {
            id: "bl-9".into(),
            title: Some("typed".into()),
            body: None,
            note: None,
        }
    );
}
