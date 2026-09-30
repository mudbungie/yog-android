//! The rungs' edges: the direct rung taken on an entry that roves, and a
//! re-call at a cached presence that expires dropping the presence, so the
//! next climb reads it afresh.

use super::*;
use crate::test_support::{Beat, serve_held};

/// The engine moved while this device was away (DESIGN §21.3, §21.9): the
/// held stream is gone, and the re-call at the cached presence expires
/// unanswered — which drops the presence, so the next climb reads the
/// engine's newer one and lands there.
#[test]
fn a_re_call_that_expires_drops_the_presence_and_the_next_climb_reads_the_new_home() {
    let _serial = serial();
    let dir = pki();
    let (old, served_old) = serve_held(
        &dir,
        "ca",
        "server",
        vec![vec![Beat::Answer(reply(1)), Beat::Hangup]],
    );
    let (new, served_new) = serve_held(&dir, "ca", "server", vec![vec![Beat::Answer(reply(2))]]);
    let node = commons(vec![old.parse().unwrap()]);
    let clock = FakeClock::new();
    let (seat, heard) = super::said::heard_seat(&dir, &node, clock.clone());
    assert_eq!(
        seat.ask(&serde_json::json!({ "op": "a" })).unwrap()[0]["n"],
        1
    );
    // The engine republishes at its new home, and its old one goes dark —
    // its listener gone, so a SYN there is refused rather than parked.
    assert_eq!(served_old.join().unwrap().len(), 1);
    let (engine, pairing) = pairing();
    let moved = Presence {
        endpoints: vec![new.parse().unwrap()],
    }
    .seal(&pairing.seal_key())
    .unwrap();
    let udp = crate::dht::Udp::bind("127.0.0.1:0".parse().unwrap()).unwrap();
    let mut dht = Dht::new(Box::new(udp), vec![node.addr], rove(vec![]).config).unwrap();
    dht.put(engine.sign(pairing.presence_salt(), 2, moved).unwrap())
        .unwrap();
    heard.take();
    assert!(seat.ask(&serde_json::json!({ "op": "x" })).is_err());
    let said = heard.take();
    let at = |part: &str| said.iter().position(|l| l.contains(part));
    assert!(at("re-call —").is_some(), "{said:?}");
    assert!(at("presence read").is_none(), "{said:?}");
    assert!(
        at("call written").is_some_and(|c| Some(c) < at("punch expired")),
        "{said:?}"
    );
    // The failed climb rests the ladder; the rest runs out.
    clock.advance(Duration::from_secs(1));
    assert_eq!(
        seat.ask(&serde_json::json!({ "op": "b" })).unwrap()[0]["n"],
        2
    );
    let said = heard.take();
    assert!(said.iter().any(|l| l.contains("presence read")), "{said:?}");
    assert!(said.iter().all(|l| !l.contains("re-call")), "{said:?}");
    assert_eq!(calls(&said), 1, "{said:?}");
    drop(seat);
    assert_eq!(served_new.join().unwrap().len(), 1);
}

/// A roving entry whose `address` answers never touches the commons: the
/// direct rung is the second rung, and what it hands back is a dialled
/// socket, not a punched one.
#[test]
fn a_roving_entry_whose_address_answers_dials_it_directly() {
    let _serial = serial();
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let ladder = Ladder::new(
        listener.local_addr().unwrap().to_string(),
        Some(rove(vec!["nowhere.invalid:1".to_owned()])),
        FakeClock::new(),
    );
    assert!(matches!(ladder.connect().unwrap(), Conn::Dialled(_)));
}
