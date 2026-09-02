//! The conversation row: the server's full spelling reads back, every token
//! table is live, the optionals are facts, and malformed shapes refuse.

use super::{AgentState, ConvBall, Flight, Tone, row};
use serde_json::{Value, json};

fn base() -> Value {
    json!({
        "root_id": "a1", "display": "quotes", "display_only": false,
        "state": "live", "uncertain": false, "preview": "hi", "age_secs": 12,
        "last_active_unix": 1_700_000_000_i64,
        "attention": 1, "members": 3, "direct": 2, "stoppable": true,
        "stop_children": false, "depth": 0, "tone": "plain",
    })
}

fn with(mut v: Value, key: &str, value: Value) -> Value {
    v.as_object_mut().unwrap().insert(key.into(), value);
    v
}

#[test]
fn minimal_row_reads_back() {
    let r = row(&base()).unwrap();
    assert_eq!(r.root_id, "a1");
    assert_eq!(r.display, "quotes");
    assert_eq!(r.state, AgentState::Live);
    assert_eq!(r.tone, Tone::Plain);
    assert_eq!(r.age_secs, 12);
    // The stamp rides beside the age and is not a second copy of it: the age
    // is the distance from the ENGINE's clock at answer time, the stamp is
    // absolute (REMOTE §9.9).
    assert_eq!(r.last_active_unix, 1_700_000_000);
    assert_eq!(r.failure, None, "absent is no failure");
    assert_eq!((r.members, r.direct, r.depth), (3, 2, 0));
    assert_eq!(
        (r.name, r.flight, r.alignment, r.ball),
        (None, None, None, None)
    );
}

#[test]
fn full_row_reads_back() {
    let v = with(base(), "name", json!("quotes"));
    let v = with(v, "flight", json!("inference"));
    let v = with(v, "alignment", json!({ "verdict": "notreal" }));
    let v = with(
        v,
        "ball",
        json!({ "id": "bl-1234", "state": "claimed", "title": "t", "badge": "p1" }),
    );
    let v = with(
        v,
        "failure",
        json!("no credential for provider row \"work\""),
    );
    let r = row(&v).unwrap();
    assert_eq!(r.name, Some("quotes".to_owned()));
    assert_eq!(
        r.failure.as_deref(),
        Some("no credential for provider row \"work\"")
    );
    assert_eq!(r.flight, Some(Flight::Inference));
    assert_eq!(r.alignment, Some(json!({ "verdict": "notreal" })));
    assert_eq!(
        r.ball,
        Some(ConvBall {
            id: "bl-1234".into(),
            state: Some("claimed".into()),
            title: Some("t".into()),
            badge: Some("p1".into()),
        })
    );
}

#[test]
fn every_state_flight_and_tone_token_is_live() {
    for (token, state) in [
        ("live", AgentState::Live),
        ("in-flight", AgentState::InFlight),
        ("quiescent", AgentState::Quiescent),
        ("stopped", AgentState::Stopped),
    ] {
        let r = row(&with(base(), "state", json!(token))).unwrap();
        assert_eq!(r.state, state);
    }
    for (token, flight) in [
        ("inference", Flight::Inference),
        ("tools", Flight::Tools),
        ("subagents", Flight::Subagents),
    ] {
        let r = row(&with(base(), "flight", json!(token))).unwrap();
        assert_eq!(r.flight, Some(flight));
    }
    for (token, tone) in [
        ("plain", Tone::Plain),
        ("weak", Tone::Weak),
        ("good", Tone::Good),
        ("bad", Tone::Bad),
        ("live", Tone::Live),
        ("in-flight", Tone::InFlight),
    ] {
        let r = row(&with(base(), "tone", json!(token))).unwrap();
        assert_eq!(r.tone, tone);
    }
}

#[test]
fn null_flight_and_null_ball_are_at_rest() {
    let v = with(base(), "flight", Value::Null);
    let v = with(v, "ball", Value::Null);
    let r = row(&v).unwrap();
    assert_eq!(r.flight, None);
    assert_eq!(r.ball, None);
}

#[test]
fn refusals_name_the_offender() {
    assert_eq!(
        row(&json!([])).unwrap_err(),
        "conversation row: not an object"
    );
    assert_eq!(
        row(&with(base(), "tone", json!("shiny"))).unwrap_err(),
        "field \"tone\": unknown token \"shiny\""
    );
    assert_eq!(
        row(&with(base(), "ball", json!("bl-1"))).unwrap_err(),
        "ball chip: not an object"
    );
    assert!(row(&with(base(), "flight", json!("walking"))).is_err());
}

/// **The compat fact this whole re-vendor turned on** (bl-e837): a field this
/// build has never heard of does not break the row. Strict decode is strict
/// about the fields it SPELLS — a missing one, a mistyped one, an unknown
/// token — and says nothing about extra ones, so an engine that grows a
/// column does not break an installed seat. What DOES break one is the §3
/// version preface, which is fail-closed on purpose (`crate::hello`), and
/// that is why the answer to a protocol bump is a new build rather than a
/// tolerant decoder.
#[test]
fn a_field_this_build_has_never_heard_of_is_ignored() {
    let v = with(base(), "some_future_column", json!({ "nested": [1, 2, 3] }));
    let v = with(v, "another", json!("whatever the engine grew"));
    let r = row(&v).unwrap();
    assert_eq!(r.root_id, "a1");
    assert_eq!(r.last_active_unix, 1_700_000_000);
}

/// The stamp is required and the clause is not, which is the engine's own
/// spelling: `last_active_unix` is written for every row, `failure` only for
/// a row that has one (REMOTE §9.10's absent-not-null).
#[test]
fn the_stamp_is_required_and_the_clause_is_not() {
    let mut bare = base();
    bare.as_object_mut().unwrap().remove("last_active_unix");
    let why = row(&bare).unwrap_err();
    assert!(why.contains("last_active_unix"), "{why}");
    let nulled = with(base(), "failure", json!(null));
    assert_eq!(row(&nulled).unwrap().failure, None);
    let mistyped = with(base(), "failure", json!(7));
    assert!(row(&mistyped).unwrap_err().contains("failure"));
}
