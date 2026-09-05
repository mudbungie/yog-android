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

/// [`recording`], and the nap then PARKS until the test lets go of the
/// returned sender. A recording nap returns at once, so the standing the host
/// published just before it — the sentence under test — stands for
/// microseconds before the next dial overwrites it, and a test polling every
/// couple of milliseconds reads it only most of the time. Holding the worker
/// inside its own nap is the one place a test can look at what it said.
fn gated() -> (Nap, mpsc::Receiver<Duration>, mpsc::Sender<()>) {
    let (tx, rests) = mpsc::channel();
    let (release, gate) = mpsc::channel::<()>();
    (
        Box::new(move |wait| {
            let _ = tx.send(wait);
            let _ = gate.recv();
        }),
        rests,
        release,
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
    // **The class is the assertion, not the verb** (bl-1ae2). Which of the
    // transport's three words a dead engine earns is the kernel's to decide:
    // a dial that arrives after the listener's thread ended is refused at the
    // connect, and one that lands in its backlog a moment earlier completes
    // and dies at the write or the read instead. All three are the channel,
    // which is the only distinction the host acts on — the standing being
    // `Redialling` at all is the class, already settled on above.
    assert!(
        ["connect ", "send:", "receive"]
            .iter()
            .any(|verb| why.starts_with(verb)),
        "{why}"
    );
    // The ladder starts at a second — the last dial had worked, so there is
    // no history for this one to answer for — and climbs from there. Read
    // before the drop: a dropped handle stops the worker at its next publish,
    // so a rest asked for afterwards may never be taken.
    assert_eq!(rests.recv().unwrap(), Duration::from_secs(1));
    assert_eq!(rests.recv().unwrap(), Duration::from_secs(2));
}

/// The ladder itself: it doubles, it stops at a minute, and it never stops
/// climbing back — a device that changes networks hourly has no number of
/// failures after which giving up is the right answer.
///
/// **The cap is above the predecessor floor and that is why it is 64** (bl-8bd0,
/// thrall's own constant). A cap under the 32-second floor would make the
/// ladder inert for the one ending that repeats — a rival permanently holding
/// this device's read would be dialled every 32 seconds for as long as the
/// battery lasted.
#[test]
fn the_ladder_doubles_to_a_minute_and_stays_there() {
    let walk: Vec<u64> = std::iter::successors(Some(Duration::from_secs(1)), |d| {
        Some(crate::host::serve::climb(*d))
    })
    .take(8)
    .map(|d| d.as_secs())
    .collect();
    assert_eq!(walk, [1, 2, 4, 8, 16, 32, 64, 64]);
}

/// **The defect a pocketed phone would have died of** (bl-8bd0, adopting
/// thrall's bl-916d). A read parked when the connection dropped does not leave
/// until the engine tries to answer it, so the redial a second later meets
/// REMOTE §5.1's one-reader guard refusing *this very device* — its own
/// predecessor, not a rival. Taken as final it made the first wifi handover
/// permanent; taken as what it is, the host waits one hold's width and asks
/// again.
#[test]
fn a_refusal_of_the_follow_read_waits_out_this_devices_own_predecessor() {
    let dir = pki();
    let refusal = json!({ "ok": false, "error": "client \"phone\" already holds a parked read" })
        .to_string()
        .into_bytes();
    let (address, _served) = crate::test_support::serve_turns(
        &dir,
        "ca",
        "server",
        // One connection per GESTURE, as every foot gesture is: the
        // advertisement lands, and the refusal is the answer to the follow
        // read that follows it. The read after the redial is HELD, as the
        // engine would hold it: a script that ended there let the host meet
        // a closed listener and publish a second `Redialling` — a connection
        // refused — that raced this test's read of the first.
        vec![
            Turn::Answer(vec![advertised()]),
            Turn::Answer(vec![refusal]),
            Turn::Answer(vec![advertised()]),
            Turn::Hold(vec![]),
        ],
    );
    let foot = Foot::open(&material(&dir, "ca", "client", &address)).unwrap();
    let (nap, rests, release) = gated();
    let mut host = Host::start(foot, table(), Box::new(dispatch), nap);
    let standing = settle(&mut host, &|s| matches!(s.health, Health::Redialling(_)));
    let Health::Redialling(why) = standing.health else {
        unreachable!()
    };
    assert_eq!(why, "client \"phone\" already holds a parked read");
    // One hold's width and two seconds, not the ladder's first rung: asking
    // sooner earns the same sentence and spends a handshake to hear it.
    assert_eq!(rests.recv().unwrap(), Duration::from_secs(32));
    drop(release);
    settle(&mut host, &|s| {
        matches!(s.health, Health::Serving) && s.advertised
    });
}

/// **A channel that ANSWERED A READ starts the ladder over**, and an accepted
/// advertisement does not. The two differ exactly where it matters: a rival
/// holding this device's read accepts every advertisement while refusing every
/// read, so resetting on acceptance would reset forever on the one ending that
/// has to back off. Here the ladder climbs across two channels that were never
/// served and returns to its floor on the third, which was.
#[test]
fn a_channel_that_was_served_returns_the_ladder_to_its_floor() {
    let dir = pki();
    let (address, _served) = serve_turns(
        &dir,
        "ca",
        "server",
        vec![
            Turn::Answer(vec![advertised()]),
            Turn::Hangup,
            Turn::Answer(vec![advertised()]),
            Turn::Hangup,
            Turn::Answer(vec![advertised()]),
            // An empty answer to the follow read is ordinary — a hold that
            // ended quietly — and it is the evidence that this channel was
            // real.
            Turn::Answer(vec![work(json!([]))]),
            Turn::Hangup,
        ],
    );
    let foot = Foot::open(&material(&dir, "ca", "client", &address)).unwrap();
    let (nap, rests) = recording();
    let host = Host::start(foot, table(), Box::new(dispatch), nap);
    assert_eq!(rests.recv().unwrap(), Duration::from_secs(1));
    assert_eq!(rests.recv().unwrap(), Duration::from_secs(2));
    assert_eq!(rests.recv().unwrap(), Duration::from_secs(1));
    drop(host);
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
            Turn::Answer(vec![advertised()]),
            Turn::Answer(vec![]),
        ],
    );
    let foot = Foot::open(&material(&dir, "ca", "client", &address)).unwrap();
    let (nap, rests) = recording();
    let mut host = Host::start(foot, table(), Box::new(dispatch), nap);
    // What is asserted on the settled standing is what that standing CARRIES
    // — the tool it ran and how it ended. Its health is not: `standing()`
    // keeps only the latest, so by the time the test reads it the host may
    // lawfully have gone on to the script's last turn and stopped there
    // (bl-1ae2). The honest reading of a transient state is the settle
    // predicate itself, which is how the redialling standing is read above.
    let standing = settle(&mut host, &|s| s.served == 1);
    assert_eq!(standing.last.as_deref(), Some("echo → 0"));
    assert_eq!(
        ops(&served.join().unwrap()),
        [
            "advertise",
            "invocations",
            "advertise",
            "invocations",
            "complete",
            "advertise",
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
