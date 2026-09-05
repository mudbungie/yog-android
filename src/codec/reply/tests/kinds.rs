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
            }),
            "prepared",
        ),
        (Reply::Providers(Vec::new()), "providers"),
        (Reply::Models(Vec::new()), "models"),
        (Reply::Applied, "applied"),
        (Reply::Nudged, "nudged"),
        (Reply::Follow(crate::codec::Stream::default()), "follow"),
        (Reply::Roles(Vec::new()), "roles"),
        (Reply::Search(crate::codec::Found::default()), "search"),
        (Reply::Attention(Vec::new()), "attention"),
        (
            Reply::Answered(crate::codec::Answered {
                tool_use: "toolu_1".into(),
                tool: "Bash".into(),
                verdict: crate::codec::Verdict::Hold,
                advanced: false,
            }),
            "answered",
        ),
        (Reply::Floored { standing: true }, "floored"),
    ];
    for (reply, kind) in named {
        assert_eq!(reply.kind(), kind);
    }
}
