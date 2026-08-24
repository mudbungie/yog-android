//! The transcript entry: one fixture per kind and per block, and every
//! refusal named — the §4.4 vocabulary held to the server's spelling.

use super::{Block, EntryKind, entry};
use serde_json::json;

#[test]
fn delivered_reads_back() {
    let v = json!({
        "name": "001", "raw": "hello", "kind": "delivered",
        "sender": "operator", "body": "hello",
    });
    let e = entry(&v).unwrap();
    assert_eq!(e.name, "001");
    assert_eq!(e.raw, "hello");
    assert_eq!(
        e.kind,
        EntryKind::Delivered {
            sender: "operator".into(),
            epitaph: None,
            body: "hello".into()
        }
    );
}

#[test]
fn delivered_epitaph_is_a_fact_when_stated() {
    let v = json!({
        "name": "001", "raw": "x", "kind": "delivered",
        "sender": "op", "epitaph": "interrupted", "body": "x",
    });
    let EntryKind::Delivered { epitaph, .. } = entry(&v).unwrap().kind else {
        panic!("wrong kind");
    };
    assert_eq!(epitaph, Some("interrupted".to_owned()));
}

#[test]
fn model_reads_back_with_every_block_kind() {
    let v = json!({
        "name": "002", "raw": "…", "kind": "model", "model_id": "m-notreal",
        "blocks": [
            { "kind": "text", "text": "hi" },
            { "kind": "thinking", "text": "hmm" },
            { "kind": "tool-use", "id": "t1", "name": "bash", "input": "ls" },
        ],
        "usage": { "input_tokens": 3 },
    });
    let EntryKind::Model {
        model_id,
        blocks,
        usage,
    } = entry(&v).unwrap().kind
    else {
        panic!("wrong kind");
    };
    assert_eq!(model_id, "m-notreal");
    assert_eq!(
        blocks,
        vec![
            Block::Text("hi".into()),
            Block::Thinking("hmm".into()),
            Block::ToolUse {
                id: "t1".into(),
                name: "bash".into(),
                input: "ls".into()
            },
        ]
    );
    assert_eq!(usage, json!({ "input_tokens": 3 }));
}

#[test]
fn tool_result_streaming_compacted_and_raw_read_back() {
    let tr = json!({
        "name": "003", "raw": "r", "kind": "tool-result",
        "tool_use_id": "t1", "content": "ok", "is_error": false,
    });
    assert_eq!(
        entry(&tr).unwrap().kind,
        EntryKind::ToolResult {
            tool_use_id: "t1".into(),
            content: "ok".into(),
            is_error: false
        }
    );
    let st = json!({
        "name": "004", "raw": "", "kind": "streaming",
        "thinking": "…", "text": "so",
    });
    assert_eq!(
        entry(&st).unwrap().kind,
        EntryKind::Streaming {
            thinking: "…".into(),
            text: "so".into()
        }
    );
    let co = json!({
        "name": "005", "raw": "", "kind": "compacted",
        "first": 1, "last": 9, "summary": "s",
    });
    assert_eq!(
        entry(&co).unwrap().kind,
        EntryKind::Compacted {
            first: 1,
            last: 9,
            summary: "s".into()
        }
    );
    let raw = json!({ "name": "006", "raw": "??", "kind": "raw" });
    assert_eq!(entry(&raw).unwrap().kind, EntryKind::Raw);
}

#[test]
fn refusals_name_the_offender() {
    assert_eq!(
        entry(&json!(0)).unwrap_err(),
        "transcript entry: not an object"
    );
    assert_eq!(
        entry(&json!({ "name": "x", "raw": "", "kind": "poem" })).unwrap_err(),
        "transcript entry: unknown kind \"poem\""
    );
    let no_usage = json!({
        "name": "x", "raw": "", "kind": "model", "model_id": "m",
        "blocks": [],
    });
    assert_eq!(
        entry(&no_usage).unwrap_err(),
        "model entry: missing field \"usage\""
    );
    let bad_block = json!({
        "name": "x", "raw": "", "kind": "model", "model_id": "m",
        "blocks": [3], "usage": {},
    });
    assert_eq!(
        entry(&bad_block).unwrap_err(),
        "content block: not an object"
    );
    let stray_block = json!({
        "name": "x", "raw": "", "kind": "model", "model_id": "m",
        "blocks": [{ "kind": "song", "text": "la" }], "usage": {},
    });
    assert_eq!(
        entry(&stray_block).unwrap_err(),
        "content block: unknown kind \"song\""
    );
}
