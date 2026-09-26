//! **Every answer names its own kind** — the one table that has to list every
//! variant of `Reply`, in its own file so the decode contract beside it stays
//! readable as the vocabulary grows (§13.7's four entries were the fourth
//! wave through it).
//!
//! What the word is for: two readers already need it — the seat model and the
//! tool host both say *"the engine answered X instead"* — and a second table
//! of these words anywhere would drift from the decoder's own.

use super::super::Reply;
use serde_json::json;

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
        (Reply::Advertised { wrote: false }, "advertised"),
        (
            Reply::Invocations(vec![Invocation {
                id: "i".into(),
                tool: "t".into(),
                input: json!({}),
                cwd: None,
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
                role: None,
            }),
            "prepared",
        ),
        (Reply::Providers(Vec::new()), "providers"),
        (Reply::Models(Vec::new()), "models"),
        (Reply::Applied, "applied"),
        (Reply::Nudged, "nudged"),
        (Reply::Follow(crate::codec::Stream::default()), "follow"),
        (Reply::Login(crate::codec::LoginView::default()), "login"),
        (Reply::Roles(Vec::new()), "roles"),
        (Reply::Search(crate::codec::Found::default()), "search"),
        (Reply::Attention(Vec::new()), "attention"),
        (
            Reply::Answered(crate::codec::Answered {
                tool_use: "toolu_1".into(),
                tool: "Bash".into(),
                verdict: crate::codec::Verdict::Hold,
                scope: crate::codec::Scope::Call,
                advanced: false,
            }),
            "answered",
        ),
        (Reply::Floored { standing: true }, "floored"),
        (Reply::Ops(Vec::new()), "ops"),
        (Reply::Acked, "acked"),
        (Reply::TrailCleared, "trail-cleared"),
    ];
    for (reply, kind) in named {
        assert_eq!(reply.kind(), kind);
    }
}

/// **The machinery, the candidates and the work review name themselves out of
/// what they decoded from** (§13.11, §13.12, §13.15). Built by the decoder rather than by hand: six literal values
/// here would be a second spelling of the shapes `codec::records` already
/// reads, and two spellings of one thing is what this file exists to prevent.
#[test]
fn the_machinery_answers_name_the_kind_they_were_read_from() {
    for body in [
        json!({ "ok": true, "kind": "agent", "display": "d", "root": "a1",
                "state": "stopped", "present": true, "refused": false, "tip": "" }),
        json!({ "ok": true, "kind": "steps", "orphan": "none", "rows": [] }),
        json!({ "ok": true, "kind": "step", "seq": "001", "meta": { "kind": "absent" },
                "request": { "kind": "absent" }, "staging": { "kind": "absent" },
                "response": [], "tools": [] }),
        json!({ "ok": true, "kind": "rail", "rows": [], "cards": [] }),
        json!({ "ok": true, "kind": "governing", "oid": "b", "short_oid": "b",
                "follows": "default", "diverged_lineages": 0, "files": [] }),
        json!({ "ok": true, "kind": "inbox", "rows": [] }),
        json!({ "ok": true, "kind": "science", "rows": [] }),
        json!({ "ok": true, "kind": "files", "worktree": false }),
        json!({ "ok": true, "kind": "config", "settings": [], "text": "" }),
        json!({ "ok": true, "kind": "marks", "branch": "balls/tasks" }),
        json!({ "ok": true, "kind": "deleted" }),
        json!({ "ok": true, "kind": "enrolled", "grade": "foot", "name": "phone-2",
                "address": "engine.invalid:7737", "ca": "notreal", "cert": "notreal",
                "key": "notreal" }),
        json!({ "ok": true, "kind": "work-diff", "rows": [] }),
        json!({ "ok": true, "kind": "fanned", "rows": [] }),
        json!({ "ok": true, "kind": "delivered", "base": "a", "target": "main" }),
        json!({ "ok": true, "kind": "retired", "discarded": false }),
        json!({ "ok": true, "kind": "armed", "armed": true }),
        json!({ "ok": true, "kind": "clients", "rows": [] }),
        json!({ "ok": true, "kind": "lineages", "rows": [] }),
        json!({ "ok": true, "kind": "proposals", "rows": [] }),
    ] {
        let named = body["kind"].as_str().unwrap_or_default().to_owned();
        let read = super::super::decode(&body).unwrap().unwrap();
        assert_eq!(read.kind(), named);
    }
}

/// **The engine's own `enrolled` frame, byte for byte** (yog
/// `corpus/reply/enrolled.json`, edition 20, yog bl-9043): the roving pair
/// rides under `rendezvous_pub` and `pairing_salt` and decodes into the
/// envelope's pair; the same frame without them is an entry that does not
/// rove, and one without the other refuses rather than landing half.
#[test]
fn the_engines_enrolled_frame_carries_the_rendezvous_material() {
    let frame = br#"{"address":"engine.invalid:7737","ca":"-----BEGIN CERTIFICATE-----\nnotreal\n-----END CERTIFICATE-----\n","cert":"-----BEGIN CERTIFICATE-----\nnotreal\n-----END CERTIFICATE-----\n","grade":"foot","key":"-----BEGIN notreal KEY-----\nnotreal\n-----END notreal KEY-----\n","kind":"enrolled","name":"builder","ok":true,"pairing_salt":"1111111111111111111111111111111111111111111111111111111111111111","rendezvous_pub":"0000000000000000000000000000000000000000000000000000000000000000"}"#;
    let mut body: serde_json::Value = serde_json::from_slice(frame).unwrap();
    let Reply::Enrolled(envelope) = super::super::decode(&body).unwrap().unwrap() else {
        panic!("not enrolled");
    };
    assert_eq!(
        envelope.roving,
        Some(("0".repeat(64), "1".repeat(64))),
        "the engine's pair, in the engine's order"
    );
    let o = body.as_object_mut().unwrap();
    o.remove("pairing_salt");
    let half = super::super::decode(&body);
    assert!(half.is_err(), "half a pair is malformed: {half:?}");
    body.as_object_mut().unwrap().remove("rendezvous_pub");
    let Reply::Enrolled(envelope) = super::super::decode(&body).unwrap().unwrap() else {
        panic!("not enrolled");
    };
    assert_eq!(
        envelope.roving, None,
        "a loopback-only engine sends neither"
    );
}
