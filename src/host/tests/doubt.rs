//! **A completion whose reply was lost** (yog REMOTE §3, bl-d1f1, consumed in
//! bl-07b1). The tool RAN — its effects are on this device — and the channel
//! died before the engine's receipt came back, so this end cannot know whether
//! the answer reached the invocation it belongs to.
//!
//! Two halves, and this file exists because only one of them was fenced. The
//! redial ladder (bl-8bd0) is about the channel and was proven on the
//! `invocations` leg; what nothing asserted is what the ladder does with the
//! answer it was carrying. It must drop it: **the presentation is the gesture
//! a redial may repeat — idempotent by design, and the engine's own `wrote`
//! reports it — and the completion is the one it may not.**
//!
//! The recovery belongs to the engine and is a read: §5.3's invocation leg is
//! at-least-once, a claim its taker dropped is requeued (yog bl-e658), and the
//! work comes back on the next `invocations`. This device answers the new
//! delivery and remembers nothing, which is thrall's own no-dedupe ruling
//! (its DESIGN §3.8) from the other side.

use super::super::Host;
use super::{advertised, dispatch, ops, pki, routed, settle, table, unslept, work};
use crate::foot::Foot;
use crate::test_support::{Turn, material, serve_turns};
use serde_json::{Value, json};

#[test]
fn a_lost_completion_is_not_re_posted_and_the_redial_presents_instead() {
    let dir = pki();
    let one = json!([{ "invocation": "i1", "tool": "echo", "input": {} }]);
    let (address, served) = serve_turns(
        &dir,
        "ca",
        "server",
        vec![
            Turn::Answer(vec![advertised()]),
            Turn::Answer(vec![work(one.clone())]),
            // The capture is written and the engine hangs up where the
            // receipt belongs: the act is in doubt from here on.
            Turn::Hangup,
            // The redial. What must come first is the presentation, and the
            // completion must NOT come at all — the capture went with the
            // channel that could not carry it.
            Turn::Answer(vec![advertised()]),
            // The engine's own recovery: the dropped claim is requeued, so
            // the same work is offered again.
            Turn::Answer(vec![work(one)]),
            Turn::Answer(vec![routed("i1")]),
            Turn::Answer(vec![advertised()]),
            Turn::Answer(vec![]),
        ],
    );
    let foot = Foot::open(&material(&dir, "ca", "client", &address)).unwrap();
    let mut host = Host::start(foot, table(), Box::new(dispatch), unslept());
    // Twice served: the tool ran again for the second delivery rather than
    // being suppressed by a remembered id. At-least-once is the contract, and
    // answering every delivery is this device's whole half of it.
    settle(&mut host, &|s| s.served == 2);
    // The join is the wait, and the host is left running for it: `served`
    // counts a tool that RAN, so it moves before the completion is written —
    // and a handle dropped here would stop the worker at its next publish,
    // with the script's last turns unserved and this thread parked on an
    // accept that never comes.
    let requests = served.join().unwrap();
    assert_eq!(
        ops(&requests),
        [
            "advertise",
            "invocations",
            "complete",
            "advertise",
            "invocations",
            "complete",
            "advertise",
            "invocations"
        ]
    );
    // The second completion is an answer to the second DELIVERY, not the
    // first capture posted twice: it is written after a read that handed the
    // work over again, which the order above is exactly the proof of.
    let posted: Vec<Value> = requests
        .iter()
        .map(|r| serde_json::from_slice(r).unwrap())
        .filter(|v: &Value| v["op"] == "complete")
        .collect();
    assert_eq!(posted.len(), 2);
    assert!(posted.iter().all(|v| v["invocation"] == "i1"));
}
