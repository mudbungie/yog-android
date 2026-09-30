//! The four rungs, each taken and each fallen through, and the backoff
//! behind the two that cost seconds.

use super::*;
use crate::rendezvous::item::Call;
use crate::test_support::{Beat, serve_from, serve_held};
use std::sync::atomic::AtomicU16;

#[test]
fn a_dark_address_rendezvouses_punches_and_holds_the_stream_for_the_next_ask() {
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
    let seat = seat(&dir, &node, clock);
    let first = seat
        .ask(&serde_json::json!({ "op": "transcript" }))
        .unwrap();
    assert_eq!(first[0]["n"], 1);
    assert!(
        until(&mut || seat.held() == 1, WAIT),
        "the punched stream is held"
    );
    let second = seat
        .ask(&serde_json::json!({ "op": "transcript" }))
        .unwrap();
    assert_eq!(second[0]["n"], 2);
    assert!(until(&mut || seat.held() == 1, WAIT), "and held again");
    drop(seat);
    let served = served.join().unwrap();
    assert_eq!(served.len(), 1, "one connection carried both asks");
    assert_eq!(served[0].len(), 2);
    // The call is on the commons, signed under the derived inbox key and
    // sealed under the pairing — what the engine's poll reads.
    let pairing = pairing().1;
    let udp = crate::dht::Udp::bind("127.0.0.1:0".parse().unwrap()).unwrap();
    let mut dht = Dht::new(Box::new(udp), vec![node.addr], rove(vec![]).config).unwrap();
    let item = dht
        .get(
            pairing.inbox_keypair().unwrap().public(),
            pairing.inbox_salt(),
        )
        .unwrap()
        .expect("the call was written");
    let call = Call::open(&pairing.seal_key(), &item.value).expect("sealed under the pairing");
    assert!(item.seq >= 1_700_000_000, "seq is the clock: {}", item.seq);
    assert!(
        call.endpoints
            .windows(2)
            .all(|w| w[0].port() == w[1].port()),
        "every endpoint names the one punch port: {:?}",
        call.endpoints
    );
}

/// Rung 3 is a RE-CALL (yog bl-278f): a punch with no call behind it is
/// one-sided — the engine's NAT holds no mapping once the served stream
/// ended — so a dropped stream's next climb writes a fresh call at the
/// cached presence, from the port the first call named, and punches. The
/// fake engine answers only that port, as a NAT that punched toward it
/// does, and the presence on the commons is made unreadable first, so a
/// climb that walked for it would fail.
#[test]
fn a_dropped_stream_re_calls_from_the_port_the_call_named_without_a_presence_walk() {
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
    let (seat, heard) = super::said::heard_seat(&dir, &node, FakeClock::new());
    assert_eq!(
        seat.ask(&serde_json::json!({ "op": "transcript" }))
            .unwrap()[0]["n"],
        1
    );
    let (seq, call) = called(&node);
    only.store(call.endpoints[0].port(), Ordering::Relaxed);
    // A SYN from any other port is dropped at the door, unserved.
    drop(std::net::TcpStream::connect(&address).unwrap());
    assert!(
        until(&mut || seat.held() == 0, WAIT),
        "the engine hung up, and the holder noticed"
    );
    let (engine, pairing) = pairing();
    let foreign = Presence { endpoints: vec![] }.seal(&[0u8; 32]).unwrap();
    let udp = crate::dht::Udp::bind("127.0.0.1:0".parse().unwrap()).unwrap();
    let mut dht = Dht::new(Box::new(udp), vec![node.addr], rove(vec![]).config).unwrap();
    dht.put(engine.sign(pairing.presence_salt(), 2, foreign).unwrap())
        .unwrap();
    heard.take();
    assert_eq!(
        seat.ask(&serde_json::json!({ "op": "transcript" }))
            .unwrap()[0]["n"],
        2
    );
    let said = heard.take();
    assert!(said.iter().any(|l| l.contains("re-call —")), "{said:?}");
    assert!(
        said.iter()
            .all(|l| !l.contains("presence") || l.contains("presence cached")),
        "{said:?}"
    );
    assert_eq!(calls(&said), 1, "{said:?}");
    let (next, call) = called(&node);
    assert!(next > seq, "a fresh call");
    assert_eq!(call.endpoints[0].port(), only.load(Ordering::Relaxed));
    drop(seat);
    assert_eq!(
        served.join().unwrap().len(),
        2,
        "two connections, one ask each"
    );
}

#[test]
fn a_dark_commons_arms_the_backoff_and_a_network_change_or_time_clears_it() {
    let _serial = serial();
    let mut node = FakeNode::bind(NodeId([2u8; 20]));
    node.serve(vec![], Mood::Silent, vec![]);
    let clock = FakeClock::new();
    let address = closed();
    let ladder = Ladder::new(
        address.clone(),
        Some(rove(vec![node.addr.to_string()])),
        clock.clone(),
    );
    // Walked or rested is read off the node — did a query reach it — never
    // off the clock: a rest is instant, a walk is one waited-out deadline,
    // and a loaded box can stretch either past the other (bl-6bac).
    let walked = |ladder: &Ladder| {
        let before = node.heard();
        let e = ladder.connect().err().unwrap();
        (e, node.heard() > before)
    };
    let (e, walked_first) = walked(&ladder);
    assert!(walked_first, "the first climb walks the commons");
    assert!(e.starts_with(&format!("connect {address}: ")), "{e}");
    assert!(e.contains("no DHT node answered"), "{e}");
    let (again, walked_again) = walked(&ladder);
    assert_eq!(again, e, "the same sentence, from the backoff");
    assert!(!walked_again, "and no walk inside the rest");
    clock.advance(Duration::from_secs(1));
    assert!(walked(&ladder).1, "the rest expired: walked again");
    assert!(!walked(&ladder).1, "and the rest doubled");
    clock.advance(Duration::from_secs(1));
    assert!(!walked(&ladder).1, "two seconds now");
    flap();
    assert!(
        walked(&ladder).1,
        "the network changed: the rest is cleared"
    );
}

#[test]
fn every_way_the_rendezvous_can_fail_is_its_own_sentence() {
    let _serial = serial();
    let dark = closed();
    let e = Ladder::new(
        dark.clone(),
        Some(rove(vec!["nowhere.invalid:1".to_owned()])),
        FakeClock::new(),
    )
    .connect()
    .err()
    .unwrap();
    assert!(e.ends_with("no bootstrap node resolved"), "{e}");

    let empty = commons_of(vec![]);
    let e = Ladder::new(
        dark.clone(),
        Some(rove(vec![empty.addr.to_string()])),
        FakeClock::new(),
    )
    .connect()
    .err()
    .unwrap();
    assert!(e.ends_with("the engine has published no presence"), "{e}");

    let (engine, pairing) = pairing();
    let sealed = Presence { endpoints: vec![] }.seal(&[0u8; 32]).unwrap();
    let foreign = commons_of(vec![
        engine.sign(pairing.presence_salt(), 1, sealed).unwrap(),
    ]);
    let e = Ladder::new(
        dark.clone(),
        Some(rove(vec![foreign.addr.to_string()])),
        FakeClock::new(),
    )
    .connect()
    .err()
    .unwrap();
    assert!(e.ends_with("will not open under this pairing"), "{e}");

    let nowhere = commons(vec![]);
    let e = Ladder::new(
        dark.clone(),
        Some(rove(vec![nowhere.addr.to_string()])),
        FakeClock::new(),
    )
    .connect()
    .err()
    .unwrap();
    assert!(
        e.ends_with("the engine's presence names no endpoint"),
        "{e}"
    );

    let gone = commons(vec![closed().parse().unwrap()]);
    let mut rove = rove(vec![gone.addr.to_string()]);
    rove.window = Duration::from_millis(300);
    let e = Ladder::new(dark, Some(rove), FakeClock::new())
        .connect()
        .err()
        .unwrap();
    assert!(
        e.ends_with("the punch landed nothing inside its window"),
        "{e}"
    );
}
