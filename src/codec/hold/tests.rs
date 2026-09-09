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
            let frame = encode("ws", "c-1", verdict.clone(), scope.clone());
            assert_eq!(
                frame,
                json!({ "op": "answer", "workspace": "ws", "agent": "c-1",
                        "verdict": verdict.word(), "scope": scope.word() })
            );
            assert_eq!(
                decode(&body(&frame)).unwrap(),
                (
                    "ws".to_owned(),
                    "c-1".to_owned(),
                    verdict.clone(),
                    scope.clone()
                )
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
    assert_eq!(Scope::ALL.first(), Some(&Scope::Call));
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

/// **A stray token is carried, not refused** (REMOTE §3.2): a fourth verdict
/// word a newer engine of this major spells arrives on the receipt as the
/// catch-all, and the receipt still says which call it landed on and whether
/// the branch advanced. Refusing it would throw those away to say nothing.
#[test]
fn an_unknown_verdict_rides_as_the_catch_all() {
    let gesture = json!({ "op": "answer", "workspace": "ws", "agent": "c-1",
                          "verdict": "maybe", "scope": "call" });
    let (_, _, verdict, _) = decode(&body(&gesture)).unwrap();
    assert_eq!(verdict, Verdict::Unknown("maybe".to_owned()));
    let receipt = json!({ "ok": true, "kind": "answered", "tool": "Bash",
                          "tool_use": "toolu_1", "verdict": "maybe",
                          "scope": "call", "advanced": true });
    let answered = answered_of(&body(&receipt)).unwrap();
    assert_eq!(answered.verdict, Verdict::Unknown("maybe".to_owned()));
    assert_eq!(answered.tool_use, "toolu_1");
    // Said as unknown, never as a word an operator could read as a decision.
    assert_eq!(answered.verdict.word(), "unknown verdict: maybe");
}

/// **A scope this build has not heard of is said, never widened.** It rides as
/// the catch-all for the verdict's reason exactly, and the danger the old
/// refusal was guarding against is not reachable from here: this seat only
/// ever SENDS a scope out of [`Scope::ALL`], so an unknown word is the engine
/// naming a standing in a spelling this build cannot render — and rendering it
/// as any known reach, narrow or wide, would be the lie.
#[test]
fn an_unknown_scope_is_said_rather_than_read_as_a_reach() {
    let gesture = json!({ "op": "answer", "workspace": "ws", "agent": "c-1",
                          "verdict": "pass", "scope": "world" });
    let (_, _, _, scope) = decode(&body(&gesture)).unwrap();
    assert_eq!(scope, Scope::Unknown("world".to_owned()));
    let receipt = json!({ "ok": true, "kind": "answered", "tool": "Bash",
                          "tool_use": "toolu_1", "verdict": "pass",
                          "scope": "world", "advanced": true });
    let answered = answered_of(&body(&receipt)).unwrap();
    assert_eq!(answered.scope, Scope::Unknown("world".to_owned()));
    assert_eq!(answered.scope.word(), "unknown scope: world");
    for known in Scope::ALL {
        assert_ne!(answered.scope, known);
    }
}
