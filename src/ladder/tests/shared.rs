//! One landed line, shared (bl-2ba5): callers arriving while it is held are
//! served over it and nothing re-punches; a caller that waited past
//! [`gate::LENT`] on a parked read takes a fresh call rather than a
//! re-punch at the endpoints that line was punched at; and a read that ended
//! clean is not hung up under the pool that now holds its stream.

use super::said::heard_seat;
use super::*;
use crate::test_support::{Beat, serve_held};
use std::sync::{Barrier, mpsc};

const CALLERS: u32 = 6;

/// What the dialling rungs said beyond the direct one: a climb.
fn climbed(lines: &[String]) -> Vec<String> {
    lines
        .iter()
        .filter(|l| !l.contains("held stream") && !l.contains("direct rung"))
        .cloned()
        .collect()
}

#[test]
fn callers_during_the_hold_are_served_over_the_line_and_nothing_re_punches() {
    let _serial = serial();
    let dir = pki();
    let beats = (1..=CALLERS + 1).map(|n| Beat::Answer(reply(n))).collect();
    let (address, served) = serve_held(&dir, "ca", "server", vec![beats]);
    let node = commons(vec![address.parse().unwrap()]);
    let (seat, heard) = heard_seat(&dir, &node, FakeClock::new());
    let seat = Arc::new(seat);
    assert_eq!(
        seat.ask(&serde_json::json!({ "op": "a" })).unwrap()[0]["n"],
        1
    );
    assert!(until(&mut || seat.held() == 1, WAIT), "the line is held");
    heard.take();
    // Every caller at once: each take leaves the pool empty while the
    // holder hands the line over, which is the window that used to dial.
    let start = Arc::new(Barrier::new(CALLERS as usize));
    let asks: Vec<_> = (0..CALLERS)
        .map(|_| {
            let (seat, start) = (Arc::clone(&seat), Arc::clone(&start));
            std::thread::spawn(move || {
                start.wait();
                seat.ask(&serde_json::json!({ "op": "b" })).unwrap()[0]["n"]
                    .as_u64()
                    .unwrap()
            })
        })
        .collect();
    let mut ns: Vec<u64> = asks.into_iter().map(|a| a.join().unwrap()).collect();
    ns.sort_unstable();
    assert_eq!(ns, (2..=u64::from(CALLERS) + 1).collect::<Vec<_>>());
    assert_eq!(climbed(&heard.take()), Vec::<String>::new(), "no climb");
    drop(seat);
    let served = served.join().unwrap();
    assert_eq!(served.len(), 1, "one line carried every ask");
    assert_eq!(served[0].len(), CALLERS as usize + 1);
}

#[test]
fn a_caller_past_the_bound_beside_a_parked_read_calls_afresh_and_does_not_re_punch() {
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
    let (seat, heard) = heard_seat(&dir, &node, clock.clone());
    let seat = Arc::new(seat);
    // A parked read — the attention lane — holds the one line.
    let (reading, hangup) = seat.hold(&serde_json::json!({ "op": "a" })).unwrap();
    heard.take();
    let asking = Arc::clone(&seat);
    let second = std::thread::spawn(move || asking.ask(&serde_json::json!({ "op": "b" })));
    std::thread::sleep(Duration::from_millis(400));
    assert!(!second.is_finished(), "the second caller waits on the line");
    clock.advance(gate::LENT);
    // The scripted engine serves one connection at a time: the parked read
    // ends once the second line has landed, so the second can be served.
    assert!(heard.wait("punch landed"), "a second line landed");
    hangup.hang_up();
    drop((reading, hangup, gate));
    assert_eq!(second.join().unwrap().unwrap()[0]["n"], 2);
    let said = climbed(&heard.take());
    assert!(
        said.iter().all(|l| !l.contains("re-punch")),
        "a line is out: its endpoints were punched already — {said:?}"
    );
    assert!(said.iter().any(|l| l.contains("call written")), "{said:?}");
    drop(seat);
    assert_eq!(served.join().unwrap().len(), 2, "a second line, by a call");
}

#[test]
fn a_read_that_ended_clean_is_not_hung_up_under_the_pool() {
    let _serial = serial();
    let dir = pki();
    let (address, served) = serve_held(
        &dir,
        "ca",
        "server",
        vec![vec![Beat::Answer(reply(1)), Beat::Answer(reply(2))]],
    );
    let node = commons(vec![address.parse().unwrap()]);
    let seat = seat(&dir, &node, FakeClock::new());
    // A lane: held, read to its terminator, then dropped — which hangs up.
    let (open, hangup) = seat.hold(&serde_json::json!({ "op": "a" })).unwrap();
    open.each(&mut |_| true).unwrap();
    assert!(
        until(&mut || seat.held() == 1, WAIT),
        "the stream went back"
    );
    hangup.hang_up();
    std::thread::sleep(Duration::from_millis(600));
    assert_eq!(seat.held(), 1, "the hang-up left the pool's line alone");
    assert_eq!(
        seat.ask(&serde_json::json!({ "op": "b" })).unwrap()[0]["n"],
        2
    );
    drop((hangup, seat));
    assert_eq!(
        served.join().unwrap(),
        vec![vec![br#"{"op":"a"}"#.to_vec(), br#"{"op":"b"}"#.to_vec()]]
    );
}
