//! **The redial ladder** (bl-8641): what the host does when the channel
//! breaks under it, which is the thing a phone does to it hourly. Its own
//! file for `consent.rs`'s reason — the parent is the loop's own story, and
//! this is the policy that wraps it.

use super::super::{Health, Host, Nap};
use super::{advertised, dispatch, ops, pki, routed, settle, table, work};
use crate::foot::Foot;
use crate::test_support::{Turn, material, serve_many, serve_turns};
use serde_json::json;
use std::sync::mpsc;
use std::time::Duration;

/// A nap that records what it was asked to wait, so the ladder can be read
/// back. An `mpsc::Sender` rather than shared state: the worker owns one end
/// and the test owns the other, which is the same hand-off the standings
/// already cross on (and no lock, per the house rule).
fn recording() -> (Nap, mpsc::Receiver<Duration>) {
    let (tx, rests) = mpsc::channel();
    (
        Box::new(move |wait| {
            let _ = tx.send(wait);
        }),
        rests,
    )
}

/// **A dead engine is redialled, not mourned** (bl-8641). The channel is the
/// class a phone breaks every time it changes networks, so the host climbs
/// the ladder with the sentence that broke it standing where the frame can
/// read it — and the presentation is dropped with the connection that carried
/// it, because a new channel has to present again.
#[test]
fn a_dead_engine_is_redialled_with_the_dial_that_failed_standing() {
    let dir = pki();
    let (address, served) = serve_many(&dir, "ca", "server", vec![vec![advertised()]]);
    let foot = Foot::open(&material(&dir, "ca", "client", &address)).unwrap();
    let (nap, rests) = recording();
    let mut host = Host::start(foot, table(), Box::new(dispatch), nap);
    // The join is the wait: it returns when the one scripted connection has
    // been served — and the listener is gone with it, so the follow-class
    // read cannot be dialled at all. (Waiting on `advertised` instead would
    // race: `standing()` keeps only the latest, and the redial clears it.)
    assert_eq!(ops(&served.join().unwrap()), ["advertise"]);
    let standing = settle(&mut host, &|s| matches!(s.health, Health::Redialling(_)));
    assert!(!standing.advertised);
    let Health::Redialling(why) = standing.health else {
        unreachable!()
    };
    assert!(
        why.starts_with("connect ") || why.starts_with("receive"),
        "{why}"
    );
    // The ladder starts at a second — the last dial had worked, so there is
    // no history for this one to answer for — and climbs from there. Read
    // before the drop: a dropped handle stops the worker at its next publish,
    // so a rest asked for afterwards may never be taken.
    assert_eq!(rests.recv().unwrap(), Duration::from_secs(1));
    assert_eq!(rests.recv().unwrap(), Duration::from_secs(2));
}

/// The ladder itself: it doubles, it stops at half a minute, and it never
/// stops climbing back — a device that changes networks hourly has no number
/// of failures after which giving up is the right answer.
#[test]
fn the_ladder_doubles_to_thirty_seconds_and_stays_there() {
    let walk: Vec<u64> = std::iter::successors(Some(Duration::from_secs(1)), |d| {
        Some(crate::host::serve::climb(*d))
    })
    .take(8)
    .map(|d| d.as_secs())
    .collect();
    assert_eq!(walk, [1, 2, 4, 8, 16, 30, 30, 30]);
}

/// **The recovery the ball is about**: the follow read dies on the wire — the
/// operator's own `receive:` sighting — and the host presents itself again on
/// a fresh channel and goes on serving. The request log is the proof: two
/// advertisements, one per connection.
#[test]
fn a_channel_that_dies_mid_answer_is_redialled_and_the_host_serves_again() {
    let dir = pki();
    let (address, served) = serve_turns(
        &dir,
        "ca",
        "server",
        vec![
            Turn::Answer(vec![advertised()]),
            // The follow-class read, cut off at the socket.
            Turn::Hangup,
            Turn::Answer(vec![advertised()]),
            Turn::Answer(vec![work(
                json!([{ "invocation": "i1", "tool": "echo", "input": {} }]),
            )]),
            Turn::Answer(vec![routed("i1")]),
            Turn::Answer(vec![]),
        ],
    );
    let foot = Foot::open(&material(&dir, "ca", "client", &address)).unwrap();
    let (nap, rests) = recording();
    let mut host = Host::start(foot, table(), Box::new(dispatch), nap);
    let standing = settle(&mut host, &|s| s.served == 1);
    assert_eq!(standing.health, Health::Serving);
    assert!(standing.advertised);
    assert_eq!(
        ops(&served.join().unwrap()),
        [
            "advertise",
            "invocations",
            "advertise",
            "invocations",
            "complete",
            "invocations"
        ]
    );
    // One rest, at the bottom of the ladder.
    assert_eq!(rests.recv().unwrap(), Duration::from_secs(1));
}

/// A frame that goes away while the host is between channels stops it: there
/// is nobody left to publish a standing to, so a new connection would be for
/// nothing.
#[test]
fn a_frame_that_goes_away_while_redialling_stops_the_host() {
    let dir = pki();
    // Nothing listens on port 1 on loopback, so the host never gets a channel
    // at all: it redials from a standing start, which is the arm where no
    // presentation was ever made.
    let foot = Foot::open(&material(&dir, "ca", "client", "127.0.0.1:1")).unwrap();
    let (nap, rests) = recording();
    let mut host = Host::start(foot, table(), Box::new(dispatch), nap);
    let standing = settle(&mut host, &|s| matches!(s.health, Health::Redialling(_)));
    assert!(!standing.advertised);
    // The ladder never resets, because no dial ever worked.
    assert_eq!(rests.recv().unwrap(), Duration::from_secs(1));
    assert_eq!(rests.recv().unwrap(), Duration::from_secs(2));
    // And the host stops when the frame goes: the next standing it publishes
    // finds nobody reading. That this test returns at all is the assertion —
    // a worker that went on redialling would hold the recorder open forever.
    drop(host);
    while rests.recv().is_ok() {}
}
