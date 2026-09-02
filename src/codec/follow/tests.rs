//! The follow frame, every shape the corpus carries: the empty stream an
//! answer opens with, the two content kinds, and the refusals a malformed
//! one earns.

use super::{Stream, stream_of};
use serde_json::{Value, json};

fn read(v: &Value) -> Result<Stream, String> {
    stream_of(v.as_object().unwrap())
}

#[test]
fn an_opened_answer_carries_an_empty_stream() {
    let empty = read(&json!({ "kind": "follow", "ok": true, "stream": {} })).unwrap();
    assert_eq!(empty, Stream::default());
    assert!(empty.is_empty(), "nothing has landed yet");
}

#[test]
fn both_content_kinds_read_back_with_the_token_that_named_them() {
    let thinking =
        read(&json!({ "stream": { "delta": "thinking", "thinking": "first I" } })).unwrap();
    assert_eq!(thinking.delta.as_deref(), Some("thinking"));
    assert_eq!(thinking.thinking.as_deref(), Some("first I"));
    assert!(!thinking.is_empty());
    let text = read(&json!({ "stream": { "delta": "text", "text": "then this",
                                         "thinking": "first I" } }))
    .unwrap();
    assert_eq!(text.text.as_deref(), Some("then this"));
    assert!(!text.is_empty());
}

#[test]
fn a_malformed_frame_refuses_naming_the_field() {
    assert!(
        read(&json!({ "kind": "follow" }))
            .unwrap_err()
            .contains("\"stream\"")
    );
    assert!(
        read(&json!({ "stream": [] }))
            .unwrap_err()
            .contains("non-object")
    );
    assert!(
        read(&json!({ "stream": { "text": 7 } }))
            .unwrap_err()
            .contains("text")
    );
}

/// The ask names the conversation and nothing else — one shot, so there is
/// no cursor, no since, and nothing for a client to get wrong about where a
/// read resumes (REMOTE §5.5: every read starts holding nothing).
#[test]
fn the_ask_names_the_conversation_and_nothing_else() {
    use crate::codec::{Ask, Gesture, encode};
    assert_eq!(
        encode(&Gesture::Ask(Ask::Follow {
            workspace: "home".into(),
            agent: "a1".into(),
        })),
        json!({ "op": "follow", "workspace": "home", "agent": "a1" })
    );
}
