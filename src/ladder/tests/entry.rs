//! One ladder per entry (bl-58a0): seats opened separately on one entry
//! share one ladder, so two callers write one call, land one punch and
//! share its line — and a caller whose line was dropped under it dials once.

use super::*;
use crate::test_support::{Beat, serve_held};
use std::sync::mpsc;

/// The newest call on the commons: its `seq` counts the calls written, since
/// the fake clock never moves and each later call is one above the last.
fn call_seq(node: &Commons) -> i64 {
    let pairing = pairing().1;
    let udp = crate::dht::Udp::bind("127.0.0.1:0".parse().unwrap()).unwrap();
    let mut dht = Dht::new(Box::new(udp), vec![node.addr], rove(vec![]).config).unwrap();
    dht.get(
        pairing.inbox_keypair().unwrap().public(),
        pairing.inbox_salt(),
    )
    .unwrap()
    .expect("a call was written")
    .seq
}

#[test]
fn two_callers_on_one_entry_write_one_call_land_one_punch_and_share_its_line() {
    let _serial = serial();
    let dir = pki();
    let (address, served) = serve_held(
        &dir,
        "ca",
        "server",
        vec![vec![Beat::Answer(reply(1)), Beat::Answer(reply(2))]],
    );
    let node = commons(vec![address.parse().unwrap()]);
    let clock = FakeClock::new();
    // Two seats, opened apart — the asker and the attention lane, say.
    let asks: Vec<_> = ["a", "b"]
        .map(|op| {
            let seat = seat(&dir, &node, clock.clone());
            std::thread::spawn(move || {
                let n = seat.ask(&serde_json::json!({ "op": op })).unwrap()[0]["n"].clone();
                (n, seat)
            })
        })
        .into_iter()
        .collect();
    let mut answered: Vec<_> = asks.into_iter().map(|a| a.join().unwrap()).collect();
    let mut ns: Vec<_> = answered.iter().map(|(n, _)| n.as_i64().unwrap()).collect();
    ns.sort_unstable();
    assert_eq!(ns, [1, 2], "both callers were answered");
    let (_, seat) = answered.pop().unwrap();
    assert!(
        until(&mut || seat.held() == 1, WAIT),
        "the one line is held"
    );
    assert_eq!(
        call_seq(&node),
        clock.unix(),
        "exactly one call was written"
    );
    drop((answered, seat));
    let served = served.join().unwrap();
    assert_eq!(served.len(), 1, "exactly one punch landed");
    assert_eq!(served[0].len(), 2, "and it carried both asks");
}

#[test]
fn a_caller_waiting_on_a_line_that_is_dropped_dials_once() {
    let _serial = serial();
    let dir = pki();
    let (gate, open) = mpsc::channel();
    let (address, served) = serve_held(
        &dir,
        "ca",
        "server",
        vec![vec![Beat::Gate(open)], vec![Beat::Answer(reply(2))]],
    );
    let node = commons(vec![address.parse().unwrap()]);
    let clock = FakeClock::new();
    let first = seat(&dir, &node, clock.clone());
    let second = seat(&dir, &node, clock.clone());
    // The first caller's read holds its line, unanswered.
    let (reading, hangup) = first.hold(&serde_json::json!({ "op": "a" })).unwrap();
    let asking = std::thread::spawn(move || second.ask(&serde_json::json!({ "op": "b" })));
    std::thread::sleep(Duration::from_millis(400));
    assert!(!asking.is_finished(), "the second caller waits on the line");
    // The read is hung up: the line goes with it, and the waiter dials.
    hangup.hang_up();
    drop((reading, hangup));
    drop(gate);
    assert_eq!(asking.join().unwrap().unwrap()[0]["n"], 2);
    assert_eq!(
        call_seq(&node),
        clock.unix() + 1,
        "the re-dial was one re-call"
    );
    drop(first);
    assert_eq!(
        served.join().unwrap().len(),
        2,
        "one new dial after the drop"
    );
}
