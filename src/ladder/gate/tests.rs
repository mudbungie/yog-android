//! The gate alone, on a clock the test turns: one dial at a time, a lent
//! line waited for, a dropped one not, and a line out past [`LENT`] no
//! longer waited for.

use super::*;
use crate::test_support::FakeClock;
use std::sync::mpsc;

/// A caller's next turn, taken on another thread: the receiver says
/// whether it was a dial, once it is taken.
fn waiter(gate: &Arc<Gate>, clock: &Arc<FakeClock>) -> mpsc::Receiver<Option<Dialling>> {
    let (tx, rx) = mpsc::channel();
    let (gate, clock) = (Arc::clone(gate), Arc::clone(clock));
    std::thread::spawn(move || {
        let seen = gate.seen();
        let turn = match gate.turn(seen, clock.as_ref()) {
            Turn::Dial(dialling) => Some(dialling),
            Turn::Look => None,
        };
        let _ = tx.send(turn);
    });
    rx
}

const SOON: Duration = Duration::from_millis(400);
const WAIT: Duration = Duration::from_secs(10);

fn dials(gate: &Arc<Gate>, clock: &FakeClock) -> Dialling {
    match gate.turn(gate.seen(), clock) {
        Turn::Dial(dialling) => dialling,
        Turn::Look => panic!("an idle gate is a dial"),
    }
}

#[test]
fn a_second_caller_waits_on_the_dial_in_flight_and_looks_when_it_ends() {
    let (gate, clock) = (Gate::new(), FakeClock::new());
    let first = dials(&gate, clock.as_ref());
    let second = waiter(&gate, &clock);
    assert!(
        second.recv_timeout(SOON).is_err(),
        "waits while a dial is in flight"
    );
    drop(first);
    assert!(
        second.recv_timeout(WAIT).unwrap().is_none(),
        "the dial ended: look in the pool again"
    );
    let _second = dials(&gate, clock.as_ref());
}

#[test]
fn a_punched_line_out_is_waited_for_and_its_return_is_a_look() {
    let (gate, clock) = (Gate::new(), FakeClock::new());
    let lease = dials(&gate, clock.as_ref()).punched();
    let second = waiter(&gate, &clock);
    assert!(second.recv_timeout(SOON).is_err(), "waits for the line");
    drop(lease);
    assert!(second.recv_timeout(WAIT).unwrap().is_none(), "back: look");
    let held = gate.lend();
    let third = waiter(&gate, &clock);
    assert!(
        third.recv_timeout(SOON).is_err(),
        "a held line out is waited for too"
    );
    drop(held);
    assert!(third.recv_timeout(WAIT).unwrap().is_none());
}

#[test]
fn a_line_out_past_the_bound_is_no_longer_waited_for() {
    let (gate, clock) = (Gate::new(), FakeClock::new());
    let _parked = dials(&gate, clock.as_ref()).punched();
    let second = waiter(&gate, &clock);
    assert!(second.recv_timeout(SOON).is_err());
    clock.advance(LENT);
    let dialling = second.recv_timeout(WAIT).unwrap();
    assert!(dialling.is_some(), "a parked read never comes back: dial");
}

#[test]
fn a_change_since_the_caller_looked_is_a_look_not_a_wait() {
    let (gate, clock) = (Gate::new(), FakeClock::new());
    let seen = gate.seen();
    drop(gate.lend());
    assert!(matches!(gate.turn(seen, clock.as_ref()), Turn::Look));
}
