//! The reply envelope: every kind in the slice decodes, the kind-less shape
//! is a refusal and only a refusal, and malformed bytes earn the outer error.

use super::{Reply, decode};
use serde_json::json;

#[test]
fn outcome_reads_back_and_ok_is_carried_not_derived() {
    let v = json!({ "ok": false, "kind": "outcome", "exit": 1,
                    "stdout": "", "stderr": "gate red" });
    let r = decode(&v).unwrap().unwrap();
    assert_eq!(
        r,
        Reply::Outcome {
            ok: false,
            exit: 1,
            stdout: String::new(),
            stderr: "gate red".into()
        }
    );
}

#[test]
fn workspaces_reads_rows_and_notes() {
    let v = json!({
        "ok": true, "kind": "workspaces",
        "rows": [{ "workspace": "home", "kind": "named", "attention": 0,
                   "agents": 1, "running": false }],
        "stale": "rebuilding",
    });
    let Reply::Workspaces {
        rows,
        stale,
        growth,
    } = decode(&v).unwrap().unwrap()
    else {
        panic!("wrong reply");
    };
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].workspace, "home");
    assert_eq!(stale, Some("rebuilding".to_owned()));
    assert_eq!(growth, None);
}

#[test]
fn conversations_and_transcript_read_rows() {
    let conv = json!({
        "ok": true, "kind": "conversations",
        "rows": [{
            "root_id": "a1", "display": "d", "display_only": false,
            "state": "quiescent", "uncertain": false, "preview": "",
            "age_secs": 0, "attention": 0, "members": 1, "direct": 0,
            "stoppable": false, "stop_children": false, "depth": 0,
            "tone": "plain",
        }],
    });
    let Reply::Conversations(rows) = decode(&conv).unwrap().unwrap() else {
        panic!("wrong reply");
    };
    assert_eq!(rows[0].root_id, "a1");
    let tr = json!({
        "ok": true, "kind": "transcript",
        "rows": [{ "name": "001", "raw": "", "kind": "raw" }],
    });
    let Reply::Transcript(rows) = decode(&tr).unwrap().unwrap() else {
        panic!("wrong reply");
    };
    assert_eq!(rows[0].name, "001");
}

#[test]
fn the_kindless_envelope_is_a_refusal() {
    let v = json!({ "ok": false, "error": "no such workspace" });
    assert_eq!(decode(&v).unwrap().unwrap_err(), "no such workspace");
}

#[test]
fn malformed_envelopes_earn_the_outer_error() {
    assert_eq!(decode(&json!("x")).unwrap_err(), "reply: not a JSON object");
    assert_eq!(
        decode(&json!({ "ok": true, "kind": 3 })).unwrap_err(),
        "reply: non-string field \"kind\""
    );
    assert_eq!(
        decode(&json!({ "ok": true, "kind": "riddle" })).unwrap_err(),
        "unknown reply kind \"riddle\""
    );
    // ok:true with no kind is a spelling neither end writes.
    assert!(
        decode(&json!({ "ok": true }))
            .unwrap_err()
            .contains("no kind")
    );
    // a refusal missing its error text is malformed, not an empty refusal.
    assert!(
        decode(&json!({ "ok": false }))
            .unwrap_err()
            .contains("error")
    );
    // a bad row inside a listing propagates as the outer error, named.
    let bad_rows = json!({ "ok": true, "kind": "transcript", "rows": [7] });
    assert_eq!(
        decode(&bad_rows).unwrap_err(),
        "transcript entry: not an object"
    );
    let no_rows = json!({ "ok": true, "kind": "conversations" });
    assert_eq!(
        decode(&no_rows).unwrap_err(),
        "missing or non-array field \"rows\""
    );
}

#[test]
fn every_answer_names_its_own_kind() {
    use super::super::{Capture, Invocation};
    let named = [
        (
            Reply::Outcome {
                ok: true,
                exit: 0,
                stdout: String::new(),
                stderr: String::new(),
            },
            "outcome",
        ),
        (
            Reply::Workspaces {
                rows: vec![],
                stale: None,
                growth: None,
            },
            "workspaces",
        ),
        (Reply::Conversations(vec![]), "conversations"),
        (Reply::Transcript(vec![]), "transcript"),
        (Reply::Advertised, "advertised"),
        (
            Reply::Invocations(vec![Invocation {
                id: "i".into(),
                tool: "t".into(),
                input: json!({}),
            }]),
            "invocations",
        ),
        (
            Reply::Routed {
                invocation: "i".into(),
                capture: Some(Capture::default()),
            },
            "routed",
        ),
        (
            Reply::Prepared(crate::codec::Prepared {
                workspace: "home".into(),
                binding: None,
                lineage: None,
                goal: "g".into(),
                origin: "world".into(),
            }),
            "prepared",
        ),
    ];
    for (reply, kind) in named {
        assert_eq!(reply.kind(), kind);
    }
}

#[test]
fn the_routing_legs_replies_read_back() {
    assert_eq!(
        decode(&json!({ "ok": true, "kind": "advertised" }))
            .unwrap()
            .unwrap(),
        Reply::Advertised
    );
    let work = json!({ "ok": true, "kind": "invocations",
                       "rows": [{ "invocation": "i1", "tool": "shell",
                                  "input": { "command": "id" } }] });
    let Reply::Invocations(rows) = decode(&work).unwrap().unwrap() else {
        panic!("wrong reply");
    };
    assert_eq!(rows[0].id, "i1");
    // A capture is ABSENT while the far machine still runs it — a reader must
    // not have to tell "not finished" from "finished saying nothing".
    let waiting = json!({ "ok": true, "kind": "routed", "invocation": "i1" });
    assert_eq!(
        decode(&waiting).unwrap().unwrap(),
        Reply::Routed {
            invocation: "i1".into(),
            capture: None
        }
    );
    let done = json!({ "ok": true, "kind": "routed", "invocation": "i1",
                       "capture": { "stdout": "o", "stderr": "", "exit_code": 0 } });
    let Reply::Routed { capture, .. } = decode(&done).unwrap().unwrap() else {
        panic!("wrong reply");
    };
    assert_eq!(capture.unwrap_or_default().stdout, "o");
}

/// **The answer to a firing** (§8.1), and the last frame of the two-gesture
/// start. It was missing until the conformance corpus said so: this client
/// could stage a conversation and then read the engine's `started` as an
/// unknown kind, so the one gesture that makes a conversation reported a
/// failure over a conversation that was in fact running.
#[test]
fn a_fired_conversation_reads_back_with_the_name_the_engine_gave_it() {
    let frame = json!({ "ok": true, "kind": "started", "conversation": "brave-fox" });
    let started = decode(&frame).unwrap().unwrap();
    assert_eq!(
        started,
        Reply::Started {
            conversation: "brave-fox".into()
        }
    );
    assert_eq!(started.kind(), "started");
    // Strict, like every other kind: the name is the reply's whole content.
    let e = decode(&json!({ "ok": true, "kind": "started" })).unwrap_err();
    assert_eq!(e, "missing or non-string field \"conversation\"");
}
