//! The four rungs, each taken and each fallen through, and the backoff
//! behind the two that cost seconds.

use super::*;
use crate::rendezvous::item::Call;
use crate::test_support::{Beat, serve_held};

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

#[test]
fn a_dropped_stream_re_punches_at_the_cached_endpoints_without_a_walk() {
    let _serial = serial();
    let dir = pki();
    let (address, served) = serve_held(
        &dir,
        "ca",
        "server",
        vec![
            vec![Beat::Answer(reply(1)), Beat::Hangup],
            vec![Beat::Answer(reply(2))],
        ],
    );
    let node = commons(vec![address.parse().unwrap()]);
    let seat = seat(&dir, &node, FakeClock::new());
    assert_eq!(
        seat.ask(&serde_json::json!({ "op": "transcript" }))
            .unwrap()[0]["n"],
        1
    );
    assert!(
        until(&mut || seat.held() == 0, WAIT),
        "the engine hung up, and the holder noticed"
    );
    // The commons goes dark: only the cache can find the engine now.
    drop(node);
    assert_eq!(
        seat.ask(&serde_json::json!({ "op": "transcript" }))
            .unwrap()[0]["n"],
        2
    );
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
    let walked = |ladder: &Ladder| {
        let started = Instant::now();
        let e = ladder.connect().err().unwrap();
        (e, started.elapsed() >= Duration::from_millis(300))
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
