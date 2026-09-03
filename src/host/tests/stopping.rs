//! **Every way a channel stops the host**, in one file: a refusal the engine
//! wrote, an answer of a kind the gesture does not earn, and a frame that
//! stopped reading — from between two channels and from inside a run. Split
//! from the parent for `consent.rs`'s reason (bl-cc54): the parent is the
//! loop's ordinary story — advertise, wait, run, answer, re-assert — and this
//! is what ends it. The class boundary these all sit on the far side of is
//! `crate::transport::Wire`'s, and the redial half of it is `redial.rs`.

use super::super::Host;
use super::{
    advertised, host_against, ops, pki, routed, settle, slow_dispatch, stopped, table, unslept,
    work,
};
use crate::foot::Foot;
use crate::test_support::{material, serve_many};
use serde_json::json;

#[test]
fn a_refused_advertisement_stops_the_host_with_the_engines_sentence() {
    let refusal = json!({ "ok": false, "error": "not registered here" })
        .to_string()
        .into_bytes();
    let (mut host, _served) = host_against(vec![vec![refusal]]);
    let standing = settle(&mut host, &|s| stopped(s).is_some());
    assert_eq!(stopped(&standing).as_deref(), Some("not registered here"));
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
    let standing = settle(&mut host, &|s| stopped(s).is_some());
    assert_eq!(
        stopped(&standing).as_deref(),
        Some("no invocation \"i1\" is in flight")
    );
}

#[test]
fn a_wrong_reply_to_the_follow_read_names_what_came_instead() {
    let (mut host, _served) = host_against(vec![vec![advertised()], vec![advertised()]]);
    let standing = settle(&mut host, &|s| stopped(s).is_some());
    let said = stopped(&standing).unwrap_or_default();
    assert!(
        said == "the engine answered advertised, not this machine's work",
        "stopped: {said}"
    );
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
    // Four connections and no more: the host advertises, is handed work,
    // posts the capture and re-asserts its set — then finds nobody reading and
    // stops. A fifth would be a read the stopped host never makes.
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
            vec![advertised()],
        ],
    );
    let foot = Foot::open(&material(&dir, "ca", "client", &address)).unwrap();
    let mut host = Host::start(foot, table(), Box::new(slow_dispatch), unslept());
    // Waiting for the advertisement is what puts the drop INSIDE the run: the
    // publish that failed is the loop's, not the one right after presenting.
    settle(&mut host, &|s| s.advertised);
    drop(host);
    // The invocation still ran and was still answered: a frame going away does
    // not abandon work the engine is waiting on.
    assert_eq!(
        ops(&served.join().unwrap()),
        ["advertise", "invocations", "complete", "advertise"]
    );
}

/// **A refusal to the hand-off's re-assertion stops the host too** (bl-cc54).
/// The re-presentation is an ordinary §5.1 gesture and earns no exemption: an
/// engine that declines it — this device no longer registered, its grade
/// withdrawn, another connection holding a parked read with a different set in
/// force — has said something no redial changes, and a host that read on
/// regardless would go on waiting for work under a set nobody confirmed.
#[test]
fn a_refused_reassertion_stops_the_host_rather_than_serving_on() {
    let refusal = json!({ "ok": false, "error": "another connection holds this client's read" })
        .to_string()
        .into_bytes();
    let (mut host, _served) = host_against(vec![
        vec![advertised()],
        vec![work(
            json!([{ "invocation": "i1", "tool": "echo", "input": {} }]),
        )],
        vec![routed("i1")],
        vec![refusal],
    ]);
    let standing = settle(&mut host, &|s| stopped(s).is_some());
    assert_eq!(
        stopped(&standing).as_deref(),
        Some("another connection holds this client's read")
    );
    // The invocation it had already answered still counted: the hand-off
    // completed, and what failed was the sentence after it.
    assert_eq!(standing.served, 1);
}
