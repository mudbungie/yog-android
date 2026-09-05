//! The queue row, both directions of the strictness contract: the engine's
//! full spelling reads back, the two absent sub-objects are facts, and every
//! malformed shape refuses by name.

use serde_json::{Value, json};

use super::{Held, held_at, row};
use crate::codec::AgentState;

/// The engine's own row, as `corpus/reply/attention.json` carries one.
fn full() -> Value {
    json!({
        "workspace": "ws", "agent": "c-1", "display": "Cobalt",
        "state": "stopped", "uncertain": false,
        "signals": ["held", "mail", "flagged"],
        "preview": "p", "age_secs": 5, "pending": 2,
        "held": { "tool": "Bash", "tool_use": "toolu_1", "reason": "writes" },
        "failure": "Unauthorized",
        "flag": { "at": "2026-01-01T00:00:00Z", "reason": "please look at this one" },
    })
}

/// The same row with nothing waiting in it — the second frame's shape, where
/// all three optionals are real nulls rather than absences.
fn quiet() -> Value {
    json!({
        "workspace": "ws", "agent": "c-2", "display": "Dun",
        "state": "live", "uncertain": true, "signals": [],
        "preview": "", "age_secs": 0, "pending": 0,
        "held": null, "failure": null, "flag": null,
    })
}

#[test]
fn the_full_row_reads_back() {
    let r = row(&full()).unwrap();
    assert_eq!((r.workspace.as_str(), r.agent.as_str()), ("ws", "c-1"));
    assert_eq!(r.state, AgentState::Stopped);
    assert_eq!(r.signals, ["held", "mail", "flagged"]);
    assert_eq!((r.preview.as_str(), r.age_secs, r.pending), ("p", 5, 2));
    assert_eq!(
        r.held,
        Some(Held {
            tool_use: "toolu_1".into(),
            tool: "Bash".into(),
            reason: "writes".into(),
        })
    );
    assert_eq!(r.failure.as_deref(), Some("Unauthorized"));
    let flag = r.flag.unwrap();
    assert_eq!(flag.at, "2026-01-01T00:00:00Z");
    assert_eq!(flag.reason, "please look at this one");
    assert_eq!(r.display, "Cobalt");
    assert!(!r.uncertain);
}

#[test]
fn a_null_optional_is_a_fact() {
    let r = row(&quiet()).unwrap();
    assert_eq!(r.held, None);
    assert_eq!(r.failure, None);
    assert_eq!(r.flag, None);
    assert!(r.signals.is_empty());
}

/// **The pairing is both keys.** An agent id is unique inside a workspace and
/// this queue spans every workspace, so a lookup by agent alone would answer
/// one workspace's parked call under another's conversation.
#[test]
fn the_held_call_is_found_by_workspace_and_agent_together() {
    let rows = vec![row(&full()).unwrap(), row(&quiet()).unwrap()];
    assert_eq!(held_at(&rows, "ws", "c-1").unwrap().tool, "Bash");
    assert_eq!(held_at(&rows, "other", "c-1"), None);
    assert_eq!(held_at(&rows, "ws", "c-2"), None);
    assert_eq!(held_at(&rows, "ws", "c-9"), None);
}

#[test]
fn refusals_name_the_offender() {
    assert_eq!(row(&json!(3)).unwrap_err(), "attention row: not an object");
    let bad = |key: &str, value: Value| {
        let mut v = full();
        v[key] = value;
        row(&v).unwrap_err()
    };
    assert_eq!(
        bad("state", json!("wandering")),
        "field \"state\": unknown token \"wandering\""
    );
    assert_eq!(
        bad("signals", json!([7])),
        "attention row: non-string signal"
    );
    assert_eq!(bad("held", json!(7)), "held: not an object");
    assert_eq!(bad("flag", json!(7)), "flag: not an object");
    assert_eq!(
        bad("held", json!({ "tool": "Bash", "reason": "writes" })),
        "missing or non-string field \"tool_use\""
    );
    assert_eq!(
        bad("flag", json!({ "reason": "look" })),
        "missing or non-string field \"at\""
    );
    assert_eq!(
        bad("pending", json!(-1)),
        "missing or non-integer field \"pending\""
    );
}
