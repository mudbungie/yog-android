//! The lifecycle toggle (bl-c21d, bl-c00e): a backgrounded ladder parks —
//! climbs nothing, drops nothing — a climb in flight at HOME stops at its
//! next rung boundary with no call written, and the first ask back in front
//! is one re-call from the port whose call landed.

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

/// The live case: the line died behind the app, and the first ask back is
/// a re-call — exactly one call, no presence walk, from the landed port,
/// which is the only one the fake engine accepts.
#[test]
fn the_first_ask_after_a_return_is_one_re_call_from_the_landed_port() {
    let _serial = serial();
    let dir = pki();
    let only = Arc::new(AtomicU16::new(0));
    let (address, served) = serve_from(
        &dir,
        "ca",
        "server",
        vec![
            vec![Beat::Answer(reply(1)), Beat::Hangup],
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
    awake.front(false);
    let back = asking(&seat);
    assert!(heard.wait("parked"));
    heard.take();
    awake.front(true);
    assert_eq!(back.join().unwrap().unwrap(), 2);
    let said = heard.take();
    let at = |part: &str| said.iter().position(|l| l.contains(part));
    assert_eq!(
        said.first().map(String::as_str),
        Some("yog.rendezvous: back in the foreground — rest cleared, re-call armed")
    );
    assert!(
        at("re-call —").is_some() && at("presence read").is_none(),
        "{said:?}"
    );
    assert_eq!(calls(&said), 1, "{said:?}");
    assert!(said.iter().all(|l| !l.contains("re-punch")), "{said:?}");
    assert!(called(&node).0 > seq, "the re-call's call");
    drop(seat);
    assert_eq!(served.join().unwrap().len(), 2);
}

/// HOME while the climb walks for the presence: the climb stops at the
/// next rung boundary and writes no call in the background; the return's
/// climb is a re-call from the presence that walk read.
#[test]
fn home_during_the_presence_walk_stops_the_climb_before_its_call() {
    let _serial = serial();
    let dir = pki();
    let (address, served) = serve_held(&dir, "ca", "server", vec![vec![Beat::Answer(reply(2))]]);
    let node = commons(vec![address.parse().unwrap()]);
    let heard = Heard::default();
    let awake = Awake::new();
    let mut rove = rove(vec![node.addr.to_string()]);
    let (sink, home) = (heard.sink(), Arc::clone(&awake));
    // The walk's end is said on the climbing thread: HOME lands there.
    rove.say = Arc::new(move |line: &str| {
        if line.contains("presence read") {
            home.front(false);
        }
        sink(line);
    });
    rove.awake = Arc::clone(&awake);
    let m = material(&dir, "ca", "client", &closed());
    let seat = Arc::new(Seat::open_with(&m, Some(rove), FakeClock::new()).unwrap());
    assert!(seat.ask(&serde_json::json!({ "op": "a" })).is_err());
    let said = heard.take();
    assert_eq!(calls(&said), 0, "{said:?}");
    assert_eq!(
        said.last().map(String::as_str),
        Some("yog.rendezvous: climb stopped — the app left the foreground; no call written")
    );
    let back = asking(&seat);
    assert!(heard.wait("parked"));
    awake.front(true);
    assert_eq!(back.join().unwrap().unwrap(), 2);
    let said = heard.take();
    assert_eq!(calls(&said), 1, "{said:?}");
    assert!(said.iter().any(|l| l.contains("re-call armed")), "{said:?}");
    drop(seat);
    assert_eq!(served.join().unwrap().len(), 1);
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
