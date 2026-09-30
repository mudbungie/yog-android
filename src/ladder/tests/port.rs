//! The punch port, bound once per entry for the run (bl-97ed), and the
//! presence cached beside it (bl-c00e): a later climb — on the same seat or
//! on a seat opened after the first went away — is a re-call from that port
//! with no presence walk; a network change rebinds; a call that expires
//! unanswered drops the presence.

use super::*;
use crate::rendezvous::punch::Punch;
use crate::test_support::{Beat, serve_from, serve_held};
use std::sync::atomic::AtomicU16;

/// The punch the entry's ladder climbs from now.
fn punch_of(ladder: &Ladder) -> Arc<Punch> {
    ladder
        .port
        .with(&mut |port| port.bound.as_ref().map(|(punch, _)| Arc::clone(punch)))
        .expect("the entry holds a port")
}

#[test]
fn a_second_seat_on_the_entry_re_calls_from_the_port_the_first_call_named() {
    let _serial = serial();
    let dir = pki();
    let only = Arc::new(AtomicU16::new(0));
    let (address, served) = serve_from(
        &dir,
        "ca",
        "server",
        vec![vec![Beat::Answer(reply(1))], vec![Beat::Answer(reply(2))]],
        Arc::clone(&only),
    );
    let node = commons(vec![address.parse().unwrap()]);
    let clock = FakeClock::new();
    let first = seat(&dir, &node, clock.clone());
    assert_eq!(
        first.ask(&serde_json::json!({ "op": "a" })).unwrap()[0]["n"],
        1
    );
    let (seq, call) = called(&node);
    only.store(call.endpoints[0].port(), Ordering::Relaxed);
    // The seat goes, and its ladder and held line with it; the port stays.
    drop(first);
    let (second, heard) = super::said::heard_seat(&dir, &node, clock);
    assert_eq!(
        second.ask(&serde_json::json!({ "op": "b" })).unwrap()[0]["n"],
        2
    );
    let said = heard.take();
    assert!(said.iter().any(|l| l.contains("re-call")), "{said:?}");
    assert!(
        said.iter().all(|l| !l.contains("presence read")),
        "{said:?}"
    );
    assert_eq!(calls(&said), 1, "{said:?}");
    let (next, call) = called(&node);
    assert!(next > seq, "the re-call wrote a call");
    assert_eq!(call.endpoints[0].port(), only.load(Ordering::Relaxed));
    drop(second);
    assert_eq!(served.join().unwrap().len(), 2, "two lines, one port");
}

#[test]
fn a_network_change_rebinds_the_port_and_the_next_call_names_the_new_one() {
    let _serial = serial();
    let dir = pki();
    let (address, served) = serve_held(
        &dir,
        "ca",
        "server",
        vec![vec![Beat::Answer(reply(1))], vec![Beat::Answer(reply(2))]],
    );
    let node = commons(vec![address.parse().unwrap()]);
    let clock = FakeClock::new();
    let seat = seat(&dir, &node, clock.clone());
    let ladder = entry(closed(), Some(rove(vec![node.addr.to_string()])), clock);
    assert_eq!(
        seat.ask(&serde_json::json!({ "op": "a" })).unwrap()[0]["n"],
        1
    );
    let old = punch_of(&ladder);
    let (seq, call) = called(&node);
    assert_eq!(call.endpoints[0].port(), old.port());
    flap();
    assert_eq!(
        seat.ask(&serde_json::json!({ "op": "b" })).unwrap()[0]["n"],
        2
    );
    let new = punch_of(&ladder);
    assert!(!Arc::ptr_eq(&old, &new), "the port was bound afresh");
    let (next, call) = called(&node);
    assert!(next > seq, "and the climb wrote a fresh call");
    assert_eq!(call.endpoints[0].port(), new.port());
    drop((seat, ladder));
    assert_eq!(served.join().unwrap().len(), 2);
}

/// A dial beside a line still out names its own fresh port for its own
/// window, and is forgotten (bl-c21d): the entry's port stays the one whose
/// call landed, whose mapping the engine's NAT holds.
#[test]
fn a_beside_call_does_not_change_the_entrys_port() {
    let _serial = serial();
    let dir = pki();
    let (gate, open) = std::sync::mpsc::channel::<()>();
    let (address, served) = serve_held(
        &dir,
        "ca",
        "server",
        vec![vec![Beat::Gate(open)], vec![Beat::Answer(reply(2))]],
    );
    let node = commons(vec![address.parse().unwrap()]);
    let clock = FakeClock::new();
    let (seat, heard) = super::said::heard_seat(&dir, &node, clock.clone());
    let seat = Arc::new(seat);
    let ladder = entry(
        closed(),
        Some(rove(vec![node.addr.to_string()])),
        clock.clone(),
    );
    let (reading, hangup) = seat.hold(&serde_json::json!({ "op": "a" })).unwrap();
    let landed = punch_of(&ladder);
    heard.take();
    let asking = Arc::clone(&seat);
    let beside = std::thread::spawn(move || asking.ask(&serde_json::json!({ "op": "b" })));
    std::thread::sleep(Duration::from_millis(400));
    clock.advance(gate::LENT);
    assert!(heard.wait("punch landed"), "a second line landed");
    hangup.hang_up();
    drop((reading, hangup, gate));
    assert_eq!(beside.join().unwrap().unwrap()[0]["n"], 2);
    let (_, call) = called(&node);
    assert_ne!(
        call.endpoints[0].port(),
        landed.port(),
        "the beside call named its own"
    );
    assert!(
        Arc::ptr_eq(&punch_of(&ladder), &landed),
        "the entry kept the landed port"
    );
    drop((seat, ladder));
    assert_eq!(served.join().unwrap().len(), 2);
}
