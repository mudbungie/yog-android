//! The three gestures over a real handshake, and the one refusal shared by
//! all three. The host loop's own tests (`src/host/tests.rs`) drive them in
//! sequence; these pin each gesture's envelope and its wrong-kind answer.

use super::Foot;
use crate::codec::{Capture, Tool};
use crate::test_support::{material, mint_ca, mint_foot, mint_leaf, scratch, serve_once};
use serde_json::{Value, json};

fn pki() -> std::path::PathBuf {
    let dir = scratch();
    mint_ca(&dir, "ca");
    mint_leaf(&dir, "ca", "server", true);
    // A foot-grade leaf, because that is the certificate this surface exists
    // for — the same mint an operator runs with the foot flag set.
    mint_foot(&dir, "ca", "phone");
    dir
}

/// A foot over a one-shot engine that answers `reply`, and the envelope it
/// read back.
fn dialled(reply: Value) -> (Foot, std::thread::JoinHandle<Vec<u8>>) {
    let dir = pki();
    let (address, served) = serve_once(&dir, "ca", "server", vec![reply.to_string().into_bytes()]);
    let foot = Foot::open(&material(&dir, "ca", "phone", &address)).unwrap();
    (foot, served)
}

fn sent(served: std::thread::JoinHandle<Vec<u8>>) -> Value {
    serde_json::from_slice(&served.join().unwrap()).unwrap()
}

/// REMOTE §5.1: the advertisement is three facts per tool and **names no
/// client** — the identity a set lands under is the connection's.
#[test]
fn advertising_sends_the_table_and_names_no_client() {
    let (foot, served) = dialled(json!({ "ok": true, "kind": "advertised", "wrote": false }));
    let tools = vec![Tool {
        name: "echo".into(),
        description: "say it back".into(),
        input_schema: json!({ "type": "object" }),
        subject_cwd: false,
    }];
    assert_eq!(foot.advertise(tools), Ok(false));
    assert_eq!(
        sent(served),
        json!({ "op": "advertise", "tools": [
            { "name": "echo", "description": "say it back",
              "input_schema": { "type": "object" } }] })
    );
}

/// **The receipt's reading is handed back, not swallowed** (REMOTE §5.1,
/// PROTOCOL 8). This surface reports what the engine said about its own
/// document and judges none of it: what a `true` MEANS depends on which
/// presentation earned it, and only the host loop knows that.
#[test]
fn advertising_answers_whether_the_engine_wrote() {
    for wrote in [false, true] {
        let (foot, _served) = dialled(json!({ "ok": true, "kind": "advertised", "wrote": wrote }));
        assert_eq!(foot.advertise(vec![]), Ok(wrote));
    }
}

/// The follow-class read. An empty answer is ordinary — a hold that ended
/// quietly — and the gesture addresses no workspace, because it addresses a
/// *machine* (REMOTE §5).
#[test]
fn waiting_asks_for_this_machines_work_and_names_no_workspace() {
    let (foot, served) = dialled(json!({ "ok": true, "kind": "invocations", "rows": [] }));
    assert_eq!(foot.invocations(), Ok(vec![]));
    assert_eq!(sent(served), json!({ "op": "invocations" }));
}

#[test]
fn work_comes_back_with_the_models_own_arguments_verbatim() {
    let rows = json!([{ "invocation": "inv-1", "tool": "echo",
                        "input": { "say": "hi", "times": 2 } }]);
    let (foot, _served) = dialled(json!({ "ok": true, "kind": "invocations", "rows": rows }));
    let work = foot.invocations().unwrap();
    assert_eq!(work.len(), 1);
    assert_eq!(work[0].id, "inv-1");
    assert_eq!(work[0].input, json!({ "say": "hi", "times": 2 }));
}

/// The completion quotes the handle and carries the capture's three facts —
/// stdout, stderr, an exit code (REMOTE §5.3) — and names no client either.
#[test]
fn completing_quotes_the_handle_and_the_captures_three_facts() {
    let (foot, served) = dialled(json!({ "ok": true, "kind": "routed", "invocation": "inv-1" }));
    let capture = Capture {
        stdout: "out".into(),
        stderr: "warned".into(),
        exit_code: 3,
    };
    assert_eq!(foot.complete("inv-1".to_owned(), capture), Ok(()));
    assert_eq!(
        sent(served),
        json!({ "op": "complete", "invocation": "inv-1",
                "capture": { "stdout": "out", "stderr": "warned", "exit_code": 3 } })
    );
}

/// **One sentence for all three**, because it is the same question three
/// times: an engine that answered a kind this gesture does not earn is a
/// channel this device cannot go on using, not content to paint.
#[test]
fn an_answer_of_the_wrong_kind_stops_each_gesture_the_same_way() {
    let wrong = json!({ "ok": true, "kind": "workspaces", "rows": [] });
    let (foot, _s) = dialled(wrong.clone());
    let refused = foot.advertise(vec![]).unwrap_err();
    assert_eq!(
        refused.sentence(),
        "the engine answered workspaces, not this machine's work"
    );
    // And it is a REFUSAL, not a channel failure: the connection worked
    // perfectly and carried an answer this gesture does not earn, so the host
    // that redials a broken socket stops dead on this one (bl-8641).
    assert!(!refused.transport());
    let (foot, _s) = dialled(wrong.clone());
    assert!(foot.invocations().is_err());
    let (foot, _s) = dialled(wrong);
    assert!(
        foot.complete("inv-1".to_owned(), Capture::default())
            .is_err()
    );
}

/// A refusal the engine wrote crosses as the engine's own sentence — which is
/// what a foot refused for its grade would read as (REMOTE §4.2: *"in band and
/// naming the grade"*).
#[test]
fn a_carried_refusal_is_the_engines_sentence() {
    let refusal = json!({ "ok": false, "error": "client \"phone\" is foot grade" });
    let (foot, _served) = dialled(refusal);
    let refused = foot.invocations().unwrap_err();
    assert_eq!(refused.sentence(), "client \"phone\" is foot grade");
    assert!(!refused.transport());
}

/// The address is carried for the sentence a stopped host publishes; opening
/// dials nothing, so a foot exists before its engine does.
#[test]
fn a_foot_is_opened_without_dialling() {
    let dir = pki();
    let foot = Foot::open(&material(&dir, "ca", "phone", "engine.example.com:7737")).unwrap();
    assert_eq!(foot.address(), "engine.example.com:7737");
}
