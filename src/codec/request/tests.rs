//! The decode side's refusals. The *agreements* are proved by the conformance
//! corpus (`tests/conformance/`) against frames the server's own codec wrote —
//! a fixture written here would only prove this file agrees with itself.

use super::decode;
use crate::codec::{Act, Ask, Gesture, encode};
use serde_json::json;

/// The envelope's own shape, before any op is looked for.
#[test]
fn a_request_that_is_not_an_object_refuses() {
    assert_eq!(
        decode(&json!([1, 2])).unwrap_err(),
        "request: not a JSON object"
    );
}

#[test]
fn an_envelope_with_no_op_refuses_naming_the_field() {
    assert_eq!(
        decode(&json!({ "workspace": "ws" })).unwrap_err(),
        "missing or non-string field \"op\""
    );
}

/// REMOTE §3: an unknown verb *"already refuses in band, naming it, which is
/// the boundary correcting itself rather than two protocols meeting."* Naming
/// it is the whole of the contract — a shape this codec skips must still be
/// locatable in the sentence that skipped it.
#[test]
fn an_op_outside_this_slice_refuses_naming_itself() {
    assert_eq!(
        decode(&json!({ "op": "fleet" })).unwrap_err(),
        "unknown op \"fleet\""
    );
}

#[test]
fn a_staging_with_no_payload_refuses() {
    assert_eq!(
        decode(&json!({ "op": "prepare", "workspace": "ws" })).unwrap_err(),
        "prepare: missing field \"payload\""
    );
}

#[test]
fn a_staging_whose_payload_is_not_an_object_refuses() {
    let frame = json!({ "op": "prepare", "workspace": "ws", "payload": "bare" });
    assert_eq!(
        decode(&frame).unwrap_err(),
        "prepare: payload is not an object"
    );
}

#[test]
fn a_firing_with_no_prepared_body_refuses() {
    let frame = json!({ "op": "prompt", "goal": "g", "seed": null });
    assert_eq!(
        decode(&frame).unwrap_err(),
        "prompt: missing field \"prepared\""
    );
}

/// The seed is written as a real null, so its **absence** is a different
/// envelope from its null — and a codec that read the two the same way would
/// re-encode one as the other, which is the field-dropped-on-the-way-out miss
/// the corpus exists to catch.
#[test]
fn a_firing_that_states_no_seed_at_all_refuses() {
    let frame = json!({ "op": "prompt", "goal": "g",
                        "prepared": { "workspace": "ws", "binding": null,
                                      "lineage": null, "goal": "g",
                                      "origin": "world" } });
    assert_eq!(
        decode(&frame).unwrap_err(),
        "prompt: missing field \"seed\""
    );
}

/// The round trip in the other direction: what this codec *emits* decodes back
/// to the gesture that emitted it. The corpus proves the server's frames; this
/// proves nothing was lost on a frame this crate built itself.
#[test]
fn what_this_codec_emits_decodes_back_to_itself() {
    let gestures = [
        Gesture::Ask(Ask::Workspaces),
        Gesture::Ask(Ask::Invocations),
        Gesture::Ask(Ask::Conversations {
            workspace: "ws".to_owned(),
        }),
        Gesture::Ask(Ask::Transcript {
            workspace: "ws".to_owned(),
            agent: "c-1".to_owned(),
        }),
        Gesture::Act(Act::Message {
            workspace: "ws".to_owned(),
            agent: "c-1".to_owned(),
            content: "hi".to_owned(),
        }),
        Gesture::Act(Act::Prepare {
            workspace: "ws".to_owned(),
        }),
    ];
    for gesture in gestures {
        assert_eq!(decode(&encode(&gesture)).unwrap(), gesture);
    }
}
