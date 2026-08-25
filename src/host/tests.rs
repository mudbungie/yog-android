//! The host loop end to end, over the real transport against the scripted
//! server: the three gestures in order, the capture that goes back, and every
//! way a channel can stop it. The requests the server read back are asserted,
//! so this device's side of the wire is pinned rather than assumed.

use super::{Host, Standing};
use crate::codec::{Capture, Tool};
use crate::test_support::{material, mint_ca, mint_leaf, scratch, serve_many};
use crate::transport::Seat;
use serde_json::{Value, json};
use std::time::Duration;

fn pki() -> std::path::PathBuf {
    let dir = scratch();
    mint_ca(&dir, "ca");
    mint_leaf(&dir, "ca", "server", true);
    mint_leaf(&dir, "ca", "client", false);
    dir
}

fn table() -> Vec<Tool> {
    vec![Tool {
        name: "echo".into(),
        description: "say it back".into(),
        input_schema: json!({ "type": "object" }),
    }]
}

/// A dispatch that takes long enough for the frame to go away while it runs —
/// the window the mid-loop publish has to fail in.
fn slow_dispatch(tool: &str, input: &Value) -> Capture {
    std::thread::sleep(Duration::from_millis(300));
    dispatch(tool, input)
}

/// The test's whole dispatch: it echoes its arguments, so a capture the server
/// reads back proves the input reached the tool verbatim.
fn dispatch(tool: &str, input: &Value) -> Capture {
    Capture {
        stdout: format!("{tool}:{input}"),
        stderr: String::new(),
        exit_code: 0,
    }
}

fn advertised() -> Vec<u8> {
    json!({ "ok": true, "kind": "advertised" })
        .to_string()
        .into_bytes()
}

fn work(rows: Value) -> Vec<u8> {
    json!({ "ok": true, "kind": "invocations", "rows": rows })
        .to_string()
        .into_bytes()
}

fn routed(id: &str) -> Vec<u8> {
    json!({ "ok": true, "kind": "routed", "invocation": id })
        .to_string()
        .into_bytes()
}

fn host_against(scripts: Vec<Vec<Vec<u8>>>) -> (Host, std::thread::JoinHandle<Vec<Vec<u8>>>) {
    let dir = pki();
    let (address, served) = serve_many(&dir, "ca", "server", scripts);
    let seat = Seat::open(&material(&dir, "ca", "client", &address)).unwrap();
    (Host::start(seat, table(), dispatch), served)
}

/// Poll until a standing satisfies `pass` — the host publishes on its own
/// thread, so the test waits the way a frame would, just faster.
fn settle<F: Fn(&Standing) -> bool>(host: &mut Host, pass: F) -> Standing {
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    loop {
        let standing = host.standing();
        if pass(&standing) {
            return standing;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "no matching standing; last: {standing:?}"
        );
        std::thread::sleep(Duration::from_millis(2));
    }
}

fn ops(requests: &[Vec<u8>]) -> Vec<String> {
    requests
        .iter()
        .map(|r| {
            let v: Value = serde_json::from_slice(r).unwrap();
            v["op"].as_str().unwrap().to_owned()
        })
        .collect()
}

#[test]
fn it_advertises_then_runs_what_it_is_handed_and_completes_it() {
    let (mut host, served) = host_against(vec![
        vec![advertised()],
        vec![work(json!([{ "invocation": "i1", "tool": "echo",
                           "input": { "say": "hi" } }]))],
        vec![routed("i1")],
        // The loop asks again; the script ends there, which stops the host.
        vec![],
    ]);
    let standing = settle(&mut host, |s| s.served == 1);
    assert_eq!(standing.tools, ["echo"]);
    assert!(standing.advertised);
    assert_eq!(standing.last.as_deref(), Some("echo → 0"));
    let requests = served.join().unwrap();
    assert_eq!(
        ops(&requests),
        ["advertise", "invocations", "complete", "invocations"]
    );
    // The presentation names no client: the identity is the intake's.
    let presented: Value = serde_json::from_slice(&requests[0]).unwrap();
    assert_eq!(
        presented,
        json!({ "op": "advertise",
                "tools": [{ "name": "echo", "description": "say it back",
                            "input_schema": { "type": "object" } }] })
    );
    // The completion quotes the handle and carries the capture's three facts,
    // with the arguments having reached the tool verbatim.
    let completion: Value = serde_json::from_slice(&requests[2]).unwrap();
    assert_eq!(
        completion,
        json!({ "op": "complete", "invocation": "i1",
                "capture": { "stdout": "echo:{\"say\":\"hi\"}",
                             "stderr": "", "exit_code": 0 } })
    );
}

#[test]
fn an_empty_answer_is_ordinary_and_the_host_asks_again() {
    let (mut host, served) = host_against(vec![
        vec![advertised()],
        vec![work(json!([]))],
        vec![work(
            json!([{ "invocation": "i2", "tool": "echo", "input": {} }]),
        )],
        vec![routed("i2")],
        vec![],
    ]);
    settle(&mut host, |s| s.served == 1);
    assert_eq!(
        ops(&served.join().unwrap()),
        [
            "advertise",
            "invocations",
            "invocations",
            "complete",
            "invocations"
        ]
    );
}

#[test]
fn two_invocations_in_one_answer_run_in_order() {
    let (mut host, served) = host_against(vec![
        vec![advertised()],
        vec![work(json!([
            { "invocation": "a", "tool": "echo", "input": {} },
            { "invocation": "b", "tool": "echo", "input": {} }
        ]))],
        vec![routed("a")],
        vec![routed("b")],
        vec![],
    ]);
    let standing = settle(&mut host, |s| s.served == 2);
    assert_eq!(standing.served, 2);
    let requests = served.join().unwrap();
    let first: Value = serde_json::from_slice(&requests[2]).unwrap();
    let second: Value = serde_json::from_slice(&requests[3]).unwrap();
    assert_eq!(first["invocation"], "a");
    assert_eq!(second["invocation"], "b");
}

#[test]
fn a_refused_advertisement_stops_the_host_with_the_engines_sentence() {
    let refusal = json!({ "ok": false, "error": "not registered here" })
        .to_string()
        .into_bytes();
    let (mut host, _served) = host_against(vec![vec![refusal]]);
    let standing = settle(&mut host, |s| s.stopped.is_some());
    assert_eq!(standing.stopped.as_deref(), Some("not registered here"));
    assert!(!standing.advertised);
    assert_eq!(standing.served, 0);
}

#[test]
fn a_refused_completion_stops_the_host_rather_than_answering_into_it() {
    let refusal = json!({ "ok": false, "error": "no invocation \"i1\" is in flight" })
        .to_string()
        .into_bytes();
    let (mut host, _served) = host_against(vec![
        vec![advertised()],
        vec![work(
            json!([{ "invocation": "i1", "tool": "echo", "input": {} }]),
        )],
        vec![refusal],
    ]);
    let standing = settle(&mut host, |s| s.stopped.is_some());
    assert_eq!(
        standing.stopped.as_deref(),
        Some("no invocation \"i1\" is in flight")
    );
}

#[test]
fn a_wrong_reply_to_the_follow_read_names_what_came_instead() {
    let (mut host, _served) = host_against(vec![vec![advertised()], vec![advertised()]]);
    let standing = settle(&mut host, |s| s.stopped.is_some());
    let stopped = standing.stopped.unwrap_or_default();
    assert!(
        stopped == "the engine answered advertised, not this machine's work",
        "stopped: {stopped}"
    );
}

#[test]
fn a_dead_engine_stops_the_host_with_the_dial_that_failed() {
    let (mut host, served) = host_against(vec![vec![advertised()]]);
    settle(&mut host, |s| s.advertised);
    // Once the one scripted connection is served the listener is gone, so the
    // follow-class read cannot be dialled at all.
    served.join().unwrap();
    let standing = settle(&mut host, |s| s.stopped.is_some());
    assert!(standing.stopped.is_some());
    assert!(standing.advertised);
}

#[test]
fn a_frame_that_stopped_reading_stops_the_host() {
    // Dropping the handle drops the receiver, which is what a frame that went
    // away looks like from the worker's side: the host stops at its next
    // publish rather than looping into a void. One scripted connection is
    // therefore the whole script — a second would be a connection the stopped
    // host never makes.
    let (host, served) = host_against(vec![vec![advertised()]]);
    drop(host);
    // The drop does not join: it cannot, because the worker may be parked on a
    // read that answers only when there is work, and a frame blocking on that
    // is the freeze this client's whole shape excludes. That this test returns
    // at all is the assertion.
    assert_eq!(ops(&served.join().unwrap()), ["advertise"]);
}

#[test]
fn a_frame_that_goes_away_mid_run_stops_the_host_after_it_answers() {
    let dir = pki();
    // Three connections and no more: the host advertises, is handed work, and
    // posts the capture — then finds nobody reading and stops. A fourth would
    // be a read the stopped host never makes.
    let (address, served) = serve_many(
        &dir,
        "ca",
        "server",
        vec![
            vec![advertised()],
            vec![work(
                json!([{ "invocation": "i1", "tool": "echo", "input": {} }]),
            )],
            vec![routed("i1")],
        ],
    );
    let seat = Seat::open(&material(&dir, "ca", "client", &address)).unwrap();
    let mut host = Host::start(seat, table(), slow_dispatch);
    // Waiting for the advertisement is what puts the drop INSIDE the run: the
    // publish that failed is the loop's, not the one right after presenting.
    settle(&mut host, |s| s.advertised);
    drop(host);
    // The invocation still ran and was still answered: a frame going away does
    // not abandon work the engine is waiting on.
    assert_eq!(
        ops(&served.join().unwrap()),
        ["advertise", "invocations", "complete"]
    );
}
