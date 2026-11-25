//! **The pushing host** (REMOTE §5.8, edition 32; bl-1782): one read held
//! open, every pushed job started at once on its own thread, each result on
//! its own connection, `holding` on every read — and, against a server below
//! edition 32, the serial host unchanged. The server here answers by op
//! (`serve_by_op`), because a pushing host speaks on several connections at
//! once and the order they arrive in is nobody's to script.

mod reconnect;

use super::super::{Dispatch, Host};
use super::{advertised, ops, pki, routed, settle, table, unslept, work};
use crate::foot::Foot;
use crate::test_support::{Turn, material, serve_by_op};
use serde_json::{Value, json};
use std::sync::{Arc, Barrier, mpsc};
use std::time::{Duration, Instant};

/// What a job did, as the dispatch saw it.
#[derive(Debug, PartialEq, Eq)]
enum Seen {
    Started(String),
    Ended(String),
}

/// One pushed job's frame: exactly one row, as the server writes it. `wait`
/// makes the job park on the test's barrier until the test lets it go.
fn job(id: &str, wait: bool) -> Vec<u8> {
    work(json!([{ "invocation": id, "tool": "echo",
                  "input": { "id": id, "wait": wait } }]))
}

/// The heartbeat: a reply frame with no rows (REMOTE §5.8.1 rule 5).
fn beat() -> Vec<u8> {
    work(json!([]))
}

/// A dispatch that tells the test when each job starts and ends, holds each
/// for `hold`, and parks a job asked to wait on `gate`.
fn recorder(hold: Duration, gate: &Arc<Barrier>) -> (Dispatch, mpsc::Receiver<(Seen, Instant)>) {
    let (tx, seen) = mpsc::channel();
    let gate = Arc::clone(gate);
    let dispatch: Dispatch = Box::new(move |tool, input| {
        let id = input["id"].as_str().unwrap_or_default().to_owned();
        let _ = tx.send((Seen::Started(id.clone()), Instant::now()));
        if input["wait"] == true {
            gate.wait();
        }
        std::thread::sleep(hold);
        let _ = tx.send((Seen::Ended(id.clone()), Instant::now()));
        super::dispatch(tool, input)
    });
    (dispatch, seen)
}

/// The next thing a job did, or a failed test rather than a hung one.
fn next(seen: &mpsc::Receiver<(Seen, Instant)>) -> (Seen, Instant) {
    seen.recv_timeout(Duration::from_secs(10)).unwrap()
}

/// Wait for the server to read a request with this op, skipping others.
fn heard(requests: &mpsc::Receiver<Value>, op: &str) -> Value {
    loop {
        let asked = requests.recv_timeout(Duration::from_secs(10)).unwrap();
        if asked["op"] == op {
            return asked;
        }
    }
}

/// The `holding` each `invocations` request stated, in order — `None` for a
/// serial read.
fn holdings(requests: &[Value]) -> Vec<Option<Value>> {
    requests
        .iter()
        .filter(|r| r["op"] == "invocations")
        .map(|r| r.get("holding").cloned())
        .collect()
}

fn ops_of(requests: &[Value]) -> Vec<String> {
    let bytes: Vec<Vec<u8>> = requests
        .iter()
        .map(|r| r.to_string().into_bytes())
        .collect();
    ops(&bytes)
}

/// A host over a server stating `edition`, with `dispatch`, silent reads
/// bounded at `quiet`.
fn pushing(
    edition: u32,
    turns: Vec<(&'static str, Turn)>,
    dispatch: Dispatch,
    quiet: Duration,
) -> (
    Host,
    mpsc::Receiver<Value>,
    std::thread::JoinHandle<Vec<Value>>,
) {
    let dir = pki();
    let (address, requests, served) = serve_by_op(&dir, "ca", "server", edition, turns);
    let foot = Foot::open(&material(&dir, "ca", "client", &address))
        .unwrap()
        .hushed(quiet);
    (
        Host::start(foot, table(), dispatch, unslept()),
        requests,
        served,
    )
}

/// The ordinary opening: the presentation, then the read.
fn opening(read: Turn) -> Vec<(&'static str, Turn)> {
    vec![
        ("advertise", Turn::Answer(vec![advertised()])),
        ("invocations", read),
    ]
}

const LONG: Duration = Duration::from_mins(2);

/// **Two jobs pushed back to back run at once.** Each is held for 300ms, and
/// both start before either ends — which a serial host could not do.
#[test]
fn two_jobs_pushed_back_to_back_both_run_at_once() {
    let gate = Arc::new(Barrier::new(1));
    let (dispatch, seen) = recorder(Duration::from_millis(300), &gate);
    let mut turns = opening(Turn::Hold(vec![job("a", false), job("b", false)]));
    turns.push(("complete", Turn::Answer(vec![routed("a")])));
    turns.push(("complete", Turn::Answer(vec![routed("b")])));
    let (mut host, _requests, served) = pushing(32, turns, dispatch, LONG);
    let events: Vec<(Seen, Instant)> = (0..4).map(|_| next(&seen)).collect();
    let started = |e: &&(Seen, Instant)| matches!(e.0, Seen::Started(_));
    let last_start = events.iter().filter(started).map(|e| e.1).max().unwrap();
    let first_end = events
        .iter()
        .filter(|e| !started(e))
        .map(|e| e.1)
        .min()
        .unwrap();
    assert!(last_start < first_end, "the jobs ran one after the other");
    let requests = served.join().unwrap();
    assert_eq!(
        ops_of(&requests),
        ["advertise", "invocations", "complete", "complete"]
    );
    // The read stated what it held — nothing, on a fresh host — and the
    // presentation was not repeated after a hand-off: the read never closed.
    assert_eq!(holdings(&requests), [Some(json!([]))]);
    let standing = settle(&mut host, &|s| s.served == 2);
    assert_eq!(standing.last.as_deref(), Some("echo → 0"));
}

/// **A job pushed while another runs starts at once.** The first is parked on
/// the test's barrier; the second is pushed, starts and finishes while the
/// first is still running.
#[test]
fn a_job_pushed_while_another_runs_starts_at_once() {
    let gate = Arc::new(Barrier::new(2));
    let (dispatch, seen) = recorder(Duration::ZERO, &gate);
    let (feed, frames) = mpsc::channel();
    let mut turns = opening(Turn::Feed(frames));
    turns.push(("complete", Turn::Answer(vec![routed("b")])));
    turns.push(("complete", Turn::Answer(vec![routed("a")])));
    let (_host, requests, served) = pushing(32, turns, dispatch, LONG);
    feed.send(job("a", true)).unwrap();
    assert_eq!(next(&seen).0, Seen::Started("a".into()));
    feed.send(job("b", false)).unwrap();
    assert_eq!(next(&seen).0, Seen::Started("b".into()));
    assert_eq!(next(&seen).0, Seen::Ended("b".into()));
    // `b` is answered on its own connection while `a` still runs.
    assert_eq!(heard(&requests, "complete")["invocation"], "b");
    gate.wait();
    assert_eq!(next(&seen).0, Seen::Ended("a".into()));
    assert_eq!(heard(&requests, "complete")["invocation"], "a");
    served.join().unwrap();
}

/// **A pushed id already held is not started twice** (REMOTE §5.8.1).
#[test]
fn a_pushed_id_already_held_is_not_started_twice() {
    let gate = Arc::new(Barrier::new(2));
    let (dispatch, seen) = recorder(Duration::ZERO, &gate);
    let mut turns = opening(Turn::Hold(vec![
        job("a", true),
        job("a", true),
        job("b", false),
    ]));
    turns.push(("complete", Turn::Answer(vec![routed("b")])));
    turns.push(("complete", Turn::Answer(vec![routed("a")])));
    let (_host, _requests, served) = pushing(32, turns, dispatch, LONG);
    assert_eq!(next(&seen).0, Seen::Started("a".into()));
    // `b` came after the repeat, so by the time it has run the repeat was read.
    assert_eq!(next(&seen).0, Seen::Started("b".into()));
    assert_eq!(next(&seen).0, Seen::Ended("b".into()));
    gate.wait();
    assert_eq!(next(&seen).0, Seen::Ended("a".into()));
    served.join().unwrap();
    assert!(seen.recv_timeout(Duration::from_millis(200)).is_err());
}

/// **A server below edition 32 gets the serial host, byte for byte**: no
/// `holding`, the job run before the next read, the set re-asserted after
/// the hand-off.
#[test]
fn a_server_below_edition_32_gets_the_serial_host() {
    let gate = Arc::new(Barrier::new(1));
    let (dispatch, _seen) = recorder(Duration::ZERO, &gate);
    let turns = vec![
        ("advertise", Turn::Answer(vec![advertised()])),
        ("invocations", Turn::Answer(vec![job("a", false)])),
        ("complete", Turn::Answer(vec![routed("a")])),
        ("advertise", Turn::Answer(vec![advertised()])),
        ("invocations", Turn::Answer(vec![])),
    ];
    let (mut host, _requests, served) = pushing(31, turns, dispatch, LONG);
    settle(&mut host, &|s| super::stopped(s).is_some());
    let requests = served.join().unwrap();
    assert_eq!(
        ops_of(&requests),
        [
            "advertise",
            "invocations",
            "complete",
            "advertise",
            "invocations"
        ]
    );
    assert_eq!(holdings(&requests), [None, None]);
}
