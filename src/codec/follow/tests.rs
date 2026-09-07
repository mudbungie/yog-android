//! The follow frame, every shape the corpus carries: the empty stream an
//! answer opens with, the two content kinds, and the refusals a malformed
//! one earns.

use super::{Stream, stream_of};
use serde_json::{Value, json};

/// Every frame carries a window, empty included (REMOTE §5.5), so these
/// fixtures state one exactly as the corpus does and the required field is
/// pinned by every case rather than by one.
fn read(v: &Value) -> Result<Stream, String> {
    let mut frame = v.as_object().unwrap().clone();
    frame.entry("tools").or_insert_with(|| json!([]));
    stream_of(&frame)
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

/// **The fold is the engine's own** (REMOTE §5.5): `fold(a).absorb(fold(b))
/// == fold(a ++ b)`. Text accretes, the newer delta kind wins, and a part
/// that never spoke stays absent rather than becoming an empty string.
#[test]
fn absorbing_a_later_frame_is_the_fold_of_both() {
    let mut held =
        read(&json!({ "stream": { "delta": "thinking", "thinking": "first I" } })).unwrap();
    held.absorb(read(&json!({ "stream": { "delta": "text", "text": "then" } })).unwrap());
    held.absorb(read(&json!({ "stream": { "text": " this" } })).unwrap());
    assert_eq!(
        held,
        Stream {
            delta: Some("text".into()),
            text: Some("then this".into()),
            thinking: Some("first I".into()),
            tools: Vec::new(),
        }
    );
    let mut silent = Stream::default();
    silent.absorb(Stream::default());
    assert!(
        silent.is_empty(),
        "nothing absorbed onto nothing is nothing"
    );
}

/// **The window absorbs by concatenation** (REMOTE §5.5): the lists of a
/// read's frames append in order, exactly as the prose accretes, and what the
/// concatenation means is `window`'s merge.
#[test]
fn absorbing_a_later_frame_appends_its_window() {
    let mut held = read(&json!({ "stream": {},
        "tools": [{ "tool_use": "toolu_01", "tool": "box2_Bash", "input": "{}" }] }))
    .unwrap();
    held.absorb(
        read(&json!({ "stream": { "text": "done" },
        "tools": [{ "tool_use": "toolu_01", "exit_code": 0 }] }))
        .unwrap(),
    );
    assert_eq!(held.tools.len(), 2, "both transitions are held");
    let calls = held.window();
    assert_eq!(calls.len(), 1, "and they are one call");
    assert_eq!(calls[0].tool.as_deref(), Some("box2_Bash"));
    assert_eq!(calls[0].exit_code, Some(0));
}

/// A frame in the spelling from before the window refuses by name rather than
/// reading as one with nothing running.
#[test]
fn a_frame_with_no_window_refuses_naming_the_field() {
    let refusal = stream_of(json!({ "stream": {} }).as_object().unwrap()).unwrap_err();
    assert!(refusal.contains("\"tools\""), "{refusal}");
}

/// The ask names the conversation and nothing else — no cursor and no since,
/// because a read starts holding nothing and its first frame is whole
/// (REMOTE §5.5).
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
