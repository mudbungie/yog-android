//! The punch port, bound once per entry for the run (bl-97ed): a later
//! climb — on the same seat or on a seat opened after the first went away —
//! punches from the port the last call named, and a network change rebinds.

use super::*;
use crate::rendezvous::punch::Punch;
use crate::test_support::{Beat, serve_from, serve_held};
use std::sync::atomic::AtomicU16;

/// The punch the entry's ladder climbs from now.
fn punch_of(ladder: &Ladder) -> Arc<Punch> {
    ladder
        .port
        .with(&mut |port| port.as_ref().map(|port| Arc::clone(&port.punch)))
        .expect("the entry holds a port")
}

#[test]
fn a_second_seat_on_the_entry_re_punches_from_the_port_the_first_call_named() {
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
    let second = seat(&dir, &node, clock);
    assert_eq!(
        second.ask(&serde_json::json!({ "op": "b" })).unwrap()[0]["n"],
        2
    );
    assert_eq!(called(&node).0, seq, "the re-punch wrote no call");
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

/// The engine's listener completes a re-punch's handshake whenever the SYN
/// gets in, and serves it only inside a new call's window (DESIGN §21.9) —
/// so a re-punched line can die before it serves. The re-punch was spent by
/// trying it: the next climb writes a call instead of re-punching into the
/// same dead end on every redial.
#[test]
fn a_re_punched_line_that_dies_unserved_spends_the_cache_and_the_next_climb_calls() {
    let _serial = serial();
    let dir = pki();
    let (address, served) = serve_held(
        &dir,
        "ca",
        "server",
        vec![
            vec![Beat::Answer(reply(1)), Beat::Hangup],
            vec![Beat::Hangup],
            vec![Beat::Answer(reply(2))],
        ],
    );
    let node = commons(vec![address.parse().unwrap()]);
    let (seat, heard) = super::said::heard_seat(&dir, &node, FakeClock::new());
    assert_eq!(
        seat.ask(&serde_json::json!({ "op": "a" })).unwrap()[0]["n"],
        1
    );
    let (seq, _) = called(&node);
    assert!(until(&mut || seat.held() == 0, WAIT), "the line dropped");
    heard.take();
    assert!(seat.ask(&serde_json::json!({ "op": "b" })).is_err());
    assert!(
        heard.take().iter().any(|l| l.contains("re-punch at")),
        "the dead line was a re-punch's"
    );
    assert_eq!(
        seat.ask(&serde_json::json!({ "op": "c" })).unwrap()[0]["n"],
        2
    );
    let said = heard.take();
    assert!(said.iter().all(|l| !l.contains("re-punch")), "{said:?}");
    assert!(called(&node).0 > seq, "a fresh call was written");
    drop(seat);
    assert_eq!(served.join().unwrap().len(), 3);
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
