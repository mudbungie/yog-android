//! The capability gestures' spellings and their receipt, both directions.

use serde_json::{Value, json};

use super::{Verdict, answered_of, decode, encode};

fn body(v: &Value) -> serde_json::Map<String, Value> {
    v.as_object().unwrap().clone()
}

/// The three verdicts, spelled the engine's way and read back to themselves.
#[test]
fn every_verdict_round_trips_through_its_own_word() {
    for verdict in Verdict::ALL {
        let frame = encode("ws", "c-1", verdict);
        assert_eq!(
            frame,
            json!({ "op": "answer", "workspace": "ws", "agent": "c-1",
                    "verdict": verdict.word() })
        );
        assert_eq!(
            decode(&body(&frame)).unwrap(),
            ("ws".to_owned(), "c-1".to_owned(), verdict)
        );
    }
}

/// **Which verdicts release the branch** — the reading that decides whether an
/// unadvanced receipt is worth a sentence.
#[test]
fn the_two_releasing_verdicts_are_pass_and_refuse() {
    assert!(Verdict::Pass.releases());
    assert!(Verdict::Refuse.releases());
    assert!(!Verdict::Hold.releases());
}

#[test]
fn the_receipt_reads_back_whole() {
    let receipt = json!({ "ok": true, "kind": "answered", "tool": "Bash",
                          "tool_use": "toolu_1", "verdict": "hold", "advanced": false });
    let read = answered_of(&body(&receipt)).unwrap();
    assert_eq!(read.tool_use, "toolu_1");
    assert_eq!(read.tool, "Bash");
    assert_eq!(read.verdict, Verdict::Hold);
    assert!(!read.advanced);
}

/// A stray token refuses naming who was reading it — the gesture and the
/// receipt say different words for the same miss, which is what tells an
/// author which side of the wire drifted.
#[test]
fn an_unknown_verdict_refuses_by_name() {
    let gesture = json!({ "op": "answer", "workspace": "ws", "agent": "c-1",
                          "verdict": "maybe" });
    assert_eq!(
        decode(&body(&gesture)).unwrap_err(),
        "answer: unknown verdict \"maybe\""
    );
    let receipt = json!({ "ok": true, "kind": "answered", "tool": "Bash",
                          "tool_use": "toolu_1", "verdict": "maybe", "advanced": true });
    assert_eq!(
        answered_of(&body(&receipt)).unwrap_err(),
        "answered: unknown verdict \"maybe\""
    );
}
