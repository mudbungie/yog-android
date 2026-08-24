//! The gesture encoder's pins: exact envelope bytes, because the server's
//! decoder is strict and a drifted spelling is a refused gesture.

use super::{Act, Ask, Gesture, encode};
use serde_json::json;

#[test]
fn message_deposit_spelling() {
    let g = Gesture::Act(Act::Message {
        workspace: "home".into(),
        agent: "a1".into(),
        content: "hello".into(),
    });
    assert_eq!(
        encode(&g),
        json!({ "op": "message", "workspace": "home", "agent": "a1", "content": "hello" })
    );
}

#[test]
fn workspaces_spelling() {
    assert_eq!(
        encode(&Gesture::Ask(Ask::Workspaces)),
        json!({ "op": "workspaces" })
    );
}

#[test]
fn conversations_spelling() {
    assert_eq!(
        encode(&Gesture::Ask(Ask::Conversations {
            workspace: "home".into()
        })),
        json!({ "op": "conversations", "workspace": "home" })
    );
}

#[test]
fn transcript_spelling() {
    assert_eq!(
        encode(&Gesture::Ask(Ask::Transcript {
            workspace: "home".into(),
            agent: "a1".into()
        })),
        json!({ "op": "transcript", "workspace": "home", "agent": "a1" })
    );
}
