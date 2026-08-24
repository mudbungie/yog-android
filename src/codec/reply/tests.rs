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
