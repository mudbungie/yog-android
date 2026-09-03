//! The host loop end to end, over the real transport against the scripted
//! server: the three gestures in order, the capture that goes back, and every
//! way a channel can stop it. The requests the server read back are asserted,
//! so this device's side of the wire is pinned rather than assumed.

mod consent;
mod disarm;
mod redial;
mod stopping;

use super::{Health, Host, Nap, Standing};
use crate::codec::{Capture, Tool};
use crate::foot::Foot;
use crate::test_support::{material, mint_ca, mint_leaf, scratch, serve_many};
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
        subject_cwd: false,
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

/// The ordinary receipt: the engine compared and wrote nothing (REMOTE §5.1,
/// PROTOCOL 8). Every presentation in this file earns this one except where a
/// test is about the other reading.
fn advertised() -> Vec<u8> {
    receipt(false)
}

/// The receipt that says the engine **wrote** — on a re-assertion, this
/// device's set having been replaced while it was busy.
fn restored() -> Vec<u8> {
    receipt(true)
}

fn receipt(wrote: bool) -> Vec<u8> {
    json!({ "ok": true, "kind": "advertised", "wrote": wrote })
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
    let foot = Foot::open(&material(&dir, "ca", "client", &address)).unwrap();
    (
        Host::start(foot, table(), Box::new(dispatch), unslept()),
        served,
    )
}

/// A nap that does not sleep and says nothing — the ladder is not what these
/// tests are about, and a suite that rested a real second per redial would be
/// a suite nobody runs.
fn unslept() -> Nap {
    Box::new(|_| {})
}

/// Poll until a standing satisfies `pass` — the host publishes on its own
/// thread, so the test waits the way a frame would, just faster.
///
/// `pass` is a trait object, not a bound: a generic helper is monomorphized
/// per calling module, and llvm-cov then reports one instantiation's lines as
/// uncovered the moment a sibling test module calls it. One function, one set
/// of lines, and the coverage floor measures the thing that actually runs.
fn settle(host: &mut Host, pass: &dyn Fn(&Standing) -> bool) -> Standing {
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

/// The sentence a stopped host published, if it is stopped — the reading four
/// tests share, so the three-state [`Health`] is matched in one place rather
/// than at every assertion.
fn stopped(standing: &Standing) -> Option<String> {
    match &standing.health {
        Health::Stopped(why) => Some(why.clone()),
        Health::Serving | Health::Redialling(_) => None,
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
        // The hand-off ends with the set re-asserted (REMOTE §5.1).
        vec![advertised()],
        // The loop asks again; the script ends there, which stops the host.
        vec![],
    ]);
    let standing = settle(&mut host, &|s| s.served == 1);
    assert_eq!(standing.tools, ["echo"]);
    // Nothing was disarmed: the re-assertion's receipt says the engine
    // compared and wrote nothing, which is the ordinary answer.
    assert_eq!(standing.restored, 0);
    assert!(standing.advertised);
    assert_eq!(standing.last.as_deref(), Some("echo → 0"));
    let requests = served.join().unwrap();
    assert_eq!(
        ops(&requests),
        [
            "advertise",
            "invocations",
            "complete",
            "advertise",
            "invocations"
        ]
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
        vec![advertised()],
        vec![],
    ]);
    settle(&mut host, &|s| s.served == 1);
    assert_eq!(
        ops(&served.join().unwrap()),
        [
            "advertise",
            "invocations",
            "invocations",
            "complete",
            "advertise",
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
        vec![advertised()],
        vec![routed("b")],
        vec![advertised()],
        vec![],
    ]);
    let standing = settle(&mut host, &|s| s.served == 2);
    assert_eq!(standing.served, 2);
    // Every hand-off re-asserts, so the completions are no longer adjacent.
    let requests = served.join().unwrap();
    assert_eq!(
        ops(&requests),
        [
            "advertise",
            "invocations",
            "complete",
            "advertise",
            "complete",
            "advertise",
            "invocations"
        ]
    );
    let first: Value = serde_json::from_slice(&requests[2]).unwrap();
    let second: Value = serde_json::from_slice(&requests[4]).unwrap();
    assert_eq!(first["invocation"], "a");
    assert_eq!(second["invocation"], "b");
}
