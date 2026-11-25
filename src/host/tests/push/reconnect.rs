//! **What ends a pushing read, and what the next one says** (REMOTE §5.8.1,
//! §5.8.2): a quiet stream, a heartbeat that is not one, a terminator, a
//! supersession, and a post the wire swallowed. Split from the parent for
//! `redial.rs`'s reason: the parent is the push itself, this is its endings.

use super::super::super::SUPERSEDED;
use super::super::{advertised, routed, settle, stopped};
use super::{LONG, Seen, beat, heard, holdings, job, next, opening, ops_of, pushing, recorder};
use crate::test_support::Turn;
use serde_json::json;
use std::sync::{Arc, Barrier, mpsc};
use std::time::Duration;

/// **A quiet read is a dead connection, and the reconnect names what is still
/// running.** The read hands two jobs and then says nothing; `b` finishes and
/// is answered, `a` is still parked when the silence bound expires. The ladder
/// presents again — no read is open now — and the next read holds `a` alone.
#[test]
fn a_quiet_read_reconnects_holding_what_still_runs() {
    let gate = Arc::new(Barrier::new(2));
    let (dispatch, seen) = recorder(Duration::ZERO, &gate);
    let mut turns = opening(Turn::Hold(vec![job("a", true), job("b", false)]));
    turns.push(("complete", Turn::Answer(vec![routed("b")])));
    turns.push(("advertise", Turn::Answer(vec![advertised()])));
    turns.push(("invocations", Turn::Answer(vec![])));
    turns.push(("complete", Turn::Answer(vec![routed("a")])));
    let quiet = Duration::from_millis(1500);
    let (mut host, requests, served) = pushing(32, turns, dispatch, quiet);
    assert_eq!(heard(&requests, "complete")["invocation"], "b");
    heard(&requests, "advertise");
    let again = heard(&requests, "invocations");
    assert_eq!(again["holding"], json!(["a"]));
    // The job outlived its read and still posts once it is let go.
    gate.wait();
    while next(&seen).0 != Seen::Ended("a".into()) {}
    let requests = served.join().unwrap();
    assert_eq!(
        ops_of(&requests),
        [
            "advertise",
            "invocations",
            "complete",
            "advertise",
            "invocations",
            "complete"
        ]
    );
    // The second read carried nothing, which ends this host; the standing
    // says so, and both jobs are counted.
    let standing = settle(&mut host, &|s| stopped(s).is_some() && s.served == 2);
    assert_eq!(standing.served, 2);
}

/// **A heartbeat is neither work nor an end.** Beats arrive for twice the
/// silence bound, faster than it: the read is never redialled, nothing runs,
/// and the job pushed after them runs once. A read that then ends on its own
/// is opened again with no presentation between — an advertisement would end
/// it as superseded.
#[test]
fn heartbeats_start_nothing_and_keep_the_read_alive() {
    let gate = Arc::new(Barrier::new(1));
    let (dispatch, seen) = recorder(Duration::ZERO, &gate);
    let (feed, frames) = mpsc::channel();
    let mut turns = opening(Turn::Feed(frames));
    turns.push(("complete", Turn::Answer(vec![routed("a")])));
    turns.push(("invocations", Turn::Answer(vec![])));
    let (_host, requests, served) = pushing(32, turns, dispatch, Duration::from_secs(1));
    heard(&requests, "invocations");
    for _ in 0..15 {
        feed.send(beat()).unwrap();
        std::thread::sleep(Duration::from_millis(100));
    }
    assert!(requests.try_recv().is_err(), "a heartbeat ended the read");
    assert!(seen.try_recv().is_err(), "a heartbeat ran something");
    feed.send(job("a", false)).unwrap();
    heard(&requests, "complete");
    std::thread::sleep(Duration::from_millis(300));
    drop(feed);
    let requests = served.join().unwrap();
    assert_eq!(
        ops_of(&requests),
        ["advertise", "invocations", "complete", "invocations"]
    );
    assert_eq!(holdings(&requests), [Some(json!([])), Some(json!([]))]);
    assert_eq!(next(&seen).0, Seen::Started("a".into()));
}

/// **Superseded stops the host and the job it is running still answers**
/// (REMOTE §5.1, bl-83d3): no dial follows, and the post is its own
/// connection, so the read ending does not take it down.
#[test]
fn superseded_stops_the_host_and_its_running_job_still_posts() {
    let gate = Arc::new(Barrier::new(1));
    let (dispatch, _seen) = recorder(Duration::from_millis(200), &gate);
    let ended = json!({ "ok": true, "kind": "invocations", "rows": [], "superseded": true })
        .to_string()
        .into_bytes();
    let mut turns = opening(Turn::Answer(vec![job("a", false), ended]));
    turns.push(("complete", Turn::Answer(vec![routed("a")])));
    let (mut host, _requests, served) = pushing(32, turns, dispatch, LONG);
    let standing = settle(&mut host, &|s| stopped(s).is_some());
    assert_eq!(stopped(&standing).as_deref(), Some(SUPERSEDED));
    let requests = served.join().unwrap();
    assert_eq!(ops_of(&requests), ["advertise", "invocations", "complete"]);
    settle(&mut host, &|s| s.served == 1);
}

/// **A post the wire swallowed is posted again, not run again** (REMOTE §5.6
/// Ruling 1; §5.8.1: the id stays held until the post is answered). The
/// re-post rides the next frame — a heartbeat here — and only once it is
/// answered does the next read stop naming the id.
#[test]
fn a_swallowed_post_is_posted_again_and_the_job_is_not_rerun() {
    let gate = Arc::new(Barrier::new(1));
    let (dispatch, seen) = recorder(Duration::ZERO, &gate);
    let (feed, frames) = mpsc::channel();
    let mut turns = opening(Turn::Feed(frames));
    turns.push(("complete", Turn::Hangup));
    turns.push(("complete", Turn::Answer(vec![routed("a")])));
    turns.push(("invocations", Turn::Answer(vec![])));
    let (_host, requests, served) = pushing(32, turns, dispatch, LONG);
    feed.send(job("a", false)).unwrap();
    let first = heard(&requests, "complete");
    std::thread::sleep(Duration::from_millis(300));
    feed.send(beat()).unwrap();
    let second = heard(&requests, "complete");
    assert_eq!(first, second);
    std::thread::sleep(Duration::from_millis(300));
    drop(feed);
    let requests = served.join().unwrap();
    assert_eq!(holdings(&requests), [Some(json!([])), Some(json!([]))]);
    assert_eq!(next(&seen).0, Seen::Started("a".into()));
    assert_eq!(next(&seen).0, Seen::Ended("a".into()));
    assert!(seen.recv_timeout(Duration::from_millis(200)).is_err());
}
