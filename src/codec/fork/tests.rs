//! The attempt's envelope, and the two narrowings that refuse inside it.
//!
//! The corpus replay (`tests/conformance`) drives both real frames — the one
//! that pins skills, refused, and the one that pins none, round-tripped — so
//! what is asserted here is the malformed shapes no fixture carries.

use serde_json::json;

fn object(v: &serde_json::Value) -> serde_json::Map<String, serde_json::Value> {
    v.as_object().cloned().unwrap_or_default()
}

#[test]
fn a_frame_that_states_no_skills_at_all_refuses_naming_the_field() {
    let bare = json!({ "op": "fork", "workspace": "ws", "parent": "c-1",
                       "from": "aaaa1111", "role": "worker", "goal": "g" });
    assert_eq!(
        super::decode(&object(&bare)).unwrap_err(),
        "fork: missing field \"skills\""
    );
}

#[test]
fn a_skills_field_that_is_not_an_array_refuses_naming_its_shape() {
    let wrong = json!({ "op": "fork", "workspace": "ws", "parent": "c-1",
                        "from": "aaaa1111", "role": "worker", "goal": "g",
                        "skills": "bash" });
    assert_eq!(
        super::decode(&object(&wrong)).unwrap_err(),
        "fork: field \"skills\" is not an array"
    );
}
