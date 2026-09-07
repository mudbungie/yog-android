//! The capability gestures' spellings and their receipt, both directions.

use serde_json::{Value, json};

use super::{Scope, Verdict, answered_of, decode, encode};

fn body(v: &Value) -> serde_json::Map<String, Value> {
    v.as_object().unwrap().clone()
}

/// The three verdicts at each of the three scopes, spelled the engine's way
/// and read back to themselves — nine frames, which is the corpus's own count.
#[test]
fn every_verdict_round_trips_through_its_own_word_at_every_scope() {
    for verdict in Verdict::ALL {
        for scope in Scope::ALL {
            let frame = encode("ws", "c-1", verdict, scope);
            assert_eq!(
                frame,
                json!({ "op": "answer", "workspace": "ws", "agent": "c-1",
                        "verdict": verdict.word(), "scope": scope.word() })
            );
            assert_eq!(
                decode(&body(&frame)).unwrap(),
                ("ws".to_owned(), "c-1".to_owned(), verdict, scope)
            );
        }
    }
}

/// **The default is the narrow one** (PROTOCOL 18): a scope nobody chose must
/// settle the held call and nothing else, because *wider* is the reading
/// nobody may arrive at by accident.
#[test]
fn an_unchosen_scope_is_the_held_call_alone() {
    assert_eq!(Scope::default(), Scope::Call);
    assert_eq!(Scope::ALL.first().copied(), Some(Scope::Call));
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
                          "tool_use": "toolu_1", "verdict": "hold",
                          "scope": "conversation", "advanced": false });
    let read = answered_of(&body(&receipt)).unwrap();
    assert_eq!(read.tool_use, "toolu_1");
    assert_eq!(read.tool, "Bash");
    assert_eq!(read.verdict, Verdict::Hold);
    // **How wide it landed is read back**, never assumed from what was sent.
    assert_eq!(read.scope, Scope::Conversation);
    assert!(!read.advanced);
}

/// A stray token refuses naming who was reading it — the gesture and the
/// receipt say different words for the same miss, which is what tells an
/// author which side of the wire drifted.
#[test]
fn an_unknown_verdict_refuses_by_name() {
    let gesture = json!({ "op": "answer", "workspace": "ws", "agent": "c-1",
                          "verdict": "maybe", "scope": "call" });
    assert_eq!(
        decode(&body(&gesture)).unwrap_err(),
        "answer: unknown verdict \"maybe\""
    );
    let receipt = json!({ "ok": true, "kind": "answered", "tool": "Bash",
                          "tool_use": "toolu_1", "verdict": "maybe",
                          "scope": "call", "advanced": true });
    assert_eq!(
        answered_of(&body(&receipt)).unwrap_err(),
        "answered: unknown verdict \"maybe\""
    );
}

/// **A scope this build has not heard of refuses by name**, on both sides, for
/// the verdict's reason exactly: reading an unknown reach as the nearest one
/// this seat knows is the silent misread REMOTE §3's third rule forbids, and
/// here it would authorize an action wider than anybody said.
#[test]
fn an_unknown_scope_refuses_by_name() {
    let gesture = json!({ "op": "answer", "workspace": "ws", "agent": "c-1",
                          "verdict": "pass", "scope": "world" });
    assert_eq!(
        decode(&body(&gesture)).unwrap_err(),
        "answer: unknown scope \"world\""
    );
    let receipt = json!({ "ok": true, "kind": "answered", "tool": "Bash",
                          "tool_use": "toolu_1", "verdict": "pass",
                          "scope": "world", "advanced": true });
    assert_eq!(
        answered_of(&body(&receipt)).unwrap_err(),
        "answered: unknown scope \"world\""
    );
}
