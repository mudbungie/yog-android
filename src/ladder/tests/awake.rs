//! The lifecycle toggle (bl-c21d): a backgrounded ladder parks — climbs
//! nothing, drops nothing — and the first ask back in front re-punches from
//! the port whose call landed before it writes any call.

use super::*;
use crate::test_support::{Beat, serve_from, serve_held};
use std::sync::atomic::AtomicU16;

/// A seat like [`seat`] on a lifecycle the test toggles, saying its lines.
fn woken_seat(dir: &std::path::Path, node: &Commons) -> (Arc<Seat>, Heard, Arc<Awake>) {
    let heard = Heard::default();
    let awake = Awake::new();
    let mut rove = rove(vec![node.addr.to_string()]);
    rove.say = heard.sink();
    rove.awake = Arc::clone(&awake);
    let m = material(dir, "ca", "client", &closed());
    let seat = Seat::open_with(&m, Some(rove), FakeClock::new()).unwrap();
    (Arc::new(seat), heard, awake)
}

/// An ask on its own thread — it may park — answering its reply's `n`.
fn asking(seat: &Arc<Seat>) -> std::thread::JoinHandle<Result<u64, String>> {
    let seat = Arc::clone(seat);
    std::thread::spawn(move || {
        seat.ask(&serde_json::json!({ "op": "b" }))
            .map(|rows| rows[0]["n"].as_u64().unwrap())
            .map_err(|e| format!("{e:?}"))
    })
}

#[test]
fn a_backgrounded_ladder_parks_and_drops_nothing_until_the_return() {
    let _serial = serial();
    let dir = pki();
    let (address, served) = serve_held(
        &dir,
        "ca",
        "server",
        vec![vec![Beat::Answer(reply(1)), Beat::Answer(reply(2))]],
    );
    let node = commons(vec![address.parse().unwrap()]);
    let (seat, heard, awake) = woken_seat(&dir, &node);
    assert_eq!(
        seat.ask(&serde_json::json!({ "op": "a" })).unwrap()[0]["n"],
        1
    );
    assert!(until(&mut || seat.held() == 1, WAIT), "the line is held");
    heard.take();
    awake.front(false);
    assert!(!awake.awake());
    let parked = asking(&seat);
    assert!(heard.wait("parked"), "the ask parked");
    std::thread::sleep(Duration::from_millis(400));
    assert!(!parked.is_finished(), "nothing climbs in the background");
    assert_eq!(seat.held(), 1, "and the ladder dropped nothing it holds");
    assert_eq!(
        heard.take(),
        ["yog.rendezvous: parked — the app is not in the foreground; nothing climbs"]
    );
    awake.front(true);
    assert_eq!(
        parked.join().unwrap().unwrap(),
        2,
        "served over the held line"
    );
    let said = heard.take();
    assert!(
        said.iter().any(|l| l.contains("back in the foreground")),
        "{said:?}"
    );
    assert!(said.iter().all(|l| !l.contains("call written")), "{said:?}");
    drop(seat);
    assert_eq!(served.join().unwrap().len(), 1, "one line, never dropped");
}

/// The live case: the re-punch was spent (here by a line the engine never
/// served; live by a climb in the background), the app goes behind, and the
/// return re-arms it — so the first ask back rides the landed call's port,
/// which is the only one the engine's NAT lets in, and writes no call.
#[test]
fn the_first_ask_after_a_return_re_punches_from_the_landed_port_before_any_call() {
    let _serial = serial();
    let dir = pki();
    let only = Arc::new(AtomicU16::new(0));
    let (address, served) = serve_from(
        &dir,
        "ca",
        "server",
        vec![
            vec![Beat::Answer(reply(1)), Beat::Hangup],
            vec![Beat::Hangup],
            vec![Beat::Answer(reply(2))],
        ],
        Arc::clone(&only),
    );
    let node = commons(vec![address.parse().unwrap()]);
    let (seat, heard, awake) = woken_seat(&dir, &node);
    assert_eq!(
        seat.ask(&serde_json::json!({ "op": "a" })).unwrap()[0]["n"],
        1
    );
    let (seq, call) = called(&node);
    only.store(call.endpoints[0].port(), Ordering::Relaxed);
    assert!(until(&mut || seat.held() == 0, WAIT), "the line died");
    assert!(seat.ask(&serde_json::json!({ "op": "x" })).is_err());
    awake.front(false);
    let back = asking(&seat);
    assert!(heard.wait("parked"));
    heard.take();
    awake.front(true);
    assert_eq!(back.join().unwrap().unwrap(), 2);
    let said = heard.take();
    let at = |part: &str| said.iter().position(|l| l.contains(part));
    assert!(at("back in the foreground") < at("re-punch at"), "{said:?}");
    assert!(
        at("re-punch at").is_some() && at("call written").is_none(),
        "{said:?}"
    );
    assert_eq!(called(&node).0, seq, "no call was written");
    drop(seat);
    assert_eq!(served.join().unwrap().len(), 3);
}

#[test]
fn the_process_is_awake_while_anything_the_platform_granted_holds_it() {
    let awake = Awake::new();
    assert!(awake.awake(), "nobody said otherwise");
    assert_eq!(awake.returns(), 0);
    awake.front(false);
    assert!(!awake.awake());
    awake.service(true);
    assert!(awake.awake(), "the pocket service holds it");
    assert_eq!(awake.returns(), 1);
    awake.service(false);
    let hold = awake.hold();
    assert!(awake.awake(), "a job holds it");
    let second = awake.hold();
    drop(hold);
    assert!(awake.awake(), "until its last hold goes");
    drop(second);
    assert!(!awake.awake());
    awake.front(true);
    awake.front(true);
    assert_eq!(awake.returns(), 3, "only a wake after sleep is a return");
    assert!(Arc::ptr_eq(&Awake::process(), &Awake::process()));
    // A waiter parked on it wakes on the change, not the tick.
    awake.front(false);
    let (tx, rx) = std::sync::mpsc::channel();
    let parked = Arc::clone(&awake);
    let waiter = std::thread::spawn(move || parked.park(&mut || tx.send(()).unwrap()));
    rx.recv().unwrap();
    awake.front(true);
    assert_eq!(waiter.join().unwrap(), 4);
    assert_eq!(awake.park(&mut || unreachable!()), 4);
}
