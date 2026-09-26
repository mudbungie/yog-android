//! The rungs' edges: the direct rung taken on an entry that roves, and a
//! cached endpoint that misses falling through to a full rendezvous.

use super::*;
use crate::test_support::{Beat, serve_held};

/// The engine moved while this device was away (DESIGN §21.3): the network
/// changed, the held stream is gone, the cached endpoint answers nothing —
/// and the climb falls through the re-punch to a full rendezvous, which
/// reads the engine's newer presence and lands there.
#[test]
fn a_cache_that_misses_falls_through_to_a_rendezvous_at_the_engines_new_home() {
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
    let seat = seat(&dir, &node, FakeClock::new());
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
    flap();
    assert_eq!(
        seat.ask(&serde_json::json!({ "op": "b" })).unwrap()[0]["n"],
        2
    );
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
    assert!(matches!(
        ladder.connect().unwrap(),
        Conn::Fresh { punched: false, .. }
    ));
}
