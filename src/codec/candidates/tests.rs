//! The candidate shapes, read strictly — and the absences that are facts: an
//! attempt with no handle is the claim, a delivery that landed nothing states
//! no commit, and a rejection with no winner names none.
//!
//! The corpus replay (`tests/conformance`) drives every shape's real frames;
//! what is asserted here is the readings those frames do not reach and the
//! refusals nobody would otherwise see.

use serde_json::{Value, json};

use super::super::{Act, CandidateAct, Gesture, encode};

fn object(v: &Value) -> serde_json::Map<String, Value> {
    v.as_object().cloned().unwrap_or_default()
}

/// The least a science row can say and still be one.
fn bare() -> Value {
    json!({ "rows": [{ "diff": { "ball_id": "bl-1", "project": "p", "state": "unreadable" },
                       "outcome": { "state": "pending" }, "steps": 2, "wall_secs": 9,
                       "verdicts": [] }] })
}

#[test]
fn a_row_with_no_handle_is_the_claim_and_says_so_by_saying_nothing() {
    let rows = super::science(&object(&bare())).unwrap();
    let row = rows.first().cloned().unwrap_or_else(|| unreachable!());
    assert_eq!((row.diff.handle.as_str(), row.by.as_str()), ("", ""));
    assert_eq!((row.goal.as_str(), row.response.as_str()), ("", ""));
    assert_eq!((row.outcome.as_str(), row.steps), ("pending", 2));
}

#[test]
fn a_row_missing_its_diff_or_its_outcome_refuses_naming_which() {
    let no_diff = json!({ "rows": [{ "outcome": { "state": "pending" } }] });
    assert_eq!(
        super::science(&object(&no_diff)).unwrap_err(),
        "science: a row states no diff"
    );
    let no_outcome = json!({ "rows": [{ "diff": { "ball_id": "b", "project": "p",
                                                  "state": "unreadable" } }] });
    assert_eq!(
        super::science(&object(&no_outcome)).unwrap_err(),
        "science: a row states no outcome"
    );
    assert_eq!(
        super::science(&object(&json!({ "rows": ["bl-1"] }))).unwrap_err(),
        "science: row is not an object"
    );
}

#[test]
fn a_delivery_that_landed_nothing_states_no_commit() {
    let landed = super::delivered(&object(
        &json!({ "base": "aaa", "target": "main", "commit": "ccc", "source": "bbb" }),
    ))
    .unwrap();
    assert_eq!(
        (landed.commit.as_str(), landed.source.as_str()),
        ("ccc", "bbb")
    );
    let nothing = super::delivered(&object(&json!({ "base": "aaa", "target": "main" }))).unwrap();
    assert_eq!((nothing.commit.as_str(), nothing.source.as_str()), ("", ""));
}

/// **The listing is paintable only under the workspace it was read for** —
/// `science` names one, so a listing under another is the wrong claim.
#[test]
fn a_listing_is_paintable_only_under_the_workspace_it_was_read_for() {
    let spread = super::Spread {
        workspace: "home".to_owned(),
        rows: Vec::new(),
    };
    assert!(spread.about("home"));
    assert!(!spread.about("other"));
}

/// **The handle and the text are put on at the control**, one site knowing
/// which field the composer's words are.
#[test]
fn an_act_carries_the_picked_handle_and_the_field_its_text_belongs_in() {
    let deliver = CandidateAct::Deliver {
        handle: String::new(),
        summary: String::new(),
    }
    .on("at-1".to_owned(), "the winner".to_owned());
    assert_eq!(deliver.op(), "deliver");
    assert_eq!(deliver.wants(), Some("say what this delivery is"));
    assert_eq!(
        encode(&Gesture::Act(Act::Candidate {
            project: "p".to_owned(),
            ball: "bl-1".to_owned(),
            act: deliver,
        })),
        json!({ "op": "deliver", "project": "p", "ball": "bl-1",
                "handle": "at-1", "summary": "the winner" })
    );
    let retire = CandidateAct::Retire {
        handle: String::new(),
    }
    .on("at-1".to_owned(), "ignored".to_owned());
    assert_eq!(retire.wants(), None);
    assert_eq!(
        encode(&Gesture::Act(Act::Candidate {
            project: "p".to_owned(),
            ball: "bl-1".to_owned(),
            act: retire,
        })),
        json!({ "op": "retire", "project": "p", "ball": "bl-1", "handle": "at-1" })
    );
}

/// **A gesture naming no ball is refused by name**, which is what keeps the
/// bare project-repo obligation from being read as the one a row named.
#[test]
fn a_frame_naming_no_ball_refuses_by_name() {
    for op in ["fan", "deliver", "retire"] {
        let said = json!({ "op": op, "project": "p", "handle": "at-1", "n": 2 });
        assert_eq!(
            super::super::decode(&said).unwrap_err(),
            format!("{op}: unimplemented without a ball")
        );
    }
}
