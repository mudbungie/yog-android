//! The conversation row: the server's full spelling reads back, every token
//! table is live, the optionals are facts, and malformed shapes refuse.

use super::{AgentState, ConvBall, Flight, Tone, row};
use serde_json::{Value, json};

fn base() -> Value {
    json!({
        "root_id": "a1", "display": "quotes", "display_only": false,
        "state": "live", "uncertain": false, "preview": "hi", "age_secs": 12,
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
    let r = row(&v).unwrap();
    assert_eq!(r.name, Some("quotes".to_owned()));
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
