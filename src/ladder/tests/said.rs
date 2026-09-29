//! What the rungs say (bl-df05): one line per event on the injected sink,
//! families and counts only, a repeated outcome said once — read back off
//! the fake-DHT bench rung by rung.

use super::*;
use crate::test_support::{Beat, serve_held};

/// A seat like [`seat`], whose rungs say their lines to the returned sink.
fn heard_seat(dir: &std::path::Path, node: &Commons, clock: Arc<FakeClock>) -> (Seat, Heard) {
    let heard = Heard::default();
    let mut rove = rove(vec![node.addr.to_string()]);
    rove.say = heard.sink();
    let m = material(dir, "ca", "client", &closed());
    (Seat::open_with(&m, Some(rove), clock).unwrap(), heard)
}

/// A ladder over a dark address that roves with `rove`, and its sink.
fn heard_ladder(mut rove: Rove, clock: Arc<FakeClock>) -> (Ladder, Heard) {
    let heard = Heard::default();
    rove.say = heard.sink();
    (Ladder::new(closed(), Some(rove), clock), heard)
}

/// The lines the dialling thread said, without the holder threads' — whose
/// place among them is a race by design.
fn dialled(lines: &[String]) -> Vec<String> {
    lines
        .iter()
        .filter(|l| !l.contains("held stream handed") && !l.contains("held stream dropped"))
        .cloned()
        .collect()
}

const P: &str = "yog.rendezvous: ";

#[test]
fn a_rendezvous_says_each_rung_and_the_held_stream_says_its_life() {
    let _serial = serial();
    let dir = pki();
    let (address, _served) = serve_held(
        &dir,
        "ca",
        "server",
        vec![
            vec![Beat::Answer(reply(1)), Beat::Ping, Beat::Answer(reply(2))],
            vec![Beat::Answer(reply(3))],
        ],
    );
    let node = commons(vec![address.parse().unwrap()]);
    let (seat, heard) = heard_seat(&dir, &node, FakeClock::new());
    let ask = |n| seat.ask(&serde_json::json!({ "op": "a" })).unwrap()[0]["n"] == n;
    assert!(ask(1));
    let first = heard.take();
    assert_eq!(first.len(), 6, "{first:?}");
    assert_eq!(
        first[0],
        format!("{P}direct rung — 1 address(es) tried: v4 refused")
    );
    assert_eq!(
        first[1],
        format!("{P}presence read — seq 1, 1 endpoint(s) (1 v4)")
    );
    assert!(
        first[2].starts_with(&format!("{P}call written — nonce ")),
        "{first:?}"
    );
    assert!(
        first[2].ends_with(", 1 endpoint(s) (1 v4), 1 ack(s)"),
        "{first:?}"
    );
    assert_eq!(
        first[3],
        format!("{P}punch started — 1 endpoint(s) (1 v4), window 2s")
    );
    assert_eq!(first[4], format!("{P}punch landed — peer v4"));
    assert_eq!(first[5], format!("{P}held stream kept for the next ask"));
    // The ping lands in the silence and is counted when the stream is handed.
    std::thread::sleep(Duration::from_millis(600));
    assert!(ask(2));
    assert!(heard.wait("handed"));
    assert!(until(&mut || seat.held() == 1, WAIT));
    let second = heard.take();
    assert!(
        second.contains(&format!(
            "{P}held stream handed to an ask — 1 ping(s) discarded while held"
        )),
        "{second:?}"
    );
    assert_eq!(
        dialled(&second),
        [format!("{P}held stream kept for the next ask")]
    );
    // The network moves: the held stream is released and the cache re-punched;
    // the direct rung's unchanged outcome is not said again.
    flap();
    assert!(ask(3));
    assert!(heard.wait("released"));
    let third = heard.take();
    assert!(
        third.contains(&format!(
            "{P}held stream dropped — released: the network changed or the ladder went away; 0 ping(s) discarded while held"
        )),
        "{third:?}"
    );
    assert_eq!(
        dialled(&third),
        [
            format!(
                "{P}network changed — this box now sends from 1 v4; 1 held stream(s) dropped, rest cleared"
            ),
            format!("{P}re-punch at 1 cached endpoint(s) (1 v4), window 2s"),
            format!("{P}punch landed — peer v4"),
            format!("{P}held stream kept for the next ask"),
        ]
    );
    for line in first.iter().chain(&second).chain(&third) {
        assert!(!line.contains("127.0.0"), "{line} names an address");
    }
}

#[test]
fn a_dark_commons_says_its_outcome_once_per_change() {
    let _serial = serial();
    let mut node = FakeNode::bind(NodeId([2u8; 20]));
    node.serve(vec![], Mood::Silent, vec![]);
    let clock = FakeClock::new();
    let (ladder, heard) = heard_ladder(rove(vec![node.addr.to_string()]), clock.clone());
    let walk = format!(
        "{P}presence not read — the DHT walk failed (reason withheld: it names the target)"
    );
    let skip = format!("{P}climb skipped — resting after the last failed climb");
    let dial = || {
        let _ = ladder.connect();
        heard.take()
    };
    assert_eq!(
        dial(),
        [
            format!("{P}direct rung — 1 address(es) tried: v4 refused"),
            walk.clone()
        ]
    );
    assert_eq!(
        dial(),
        std::slice::from_ref(&skip),
        "the direct outcome is not said twice"
    );
    assert!(dial().is_empty(), "nor the rest");
    clock.advance(Duration::from_secs(1));
    assert_eq!(dial(), [walk], "a climb is said every time it is climbed");
    assert_eq!(dial(), [skip], "and the rest after it said again");
}

#[test]
fn every_way_a_climb_stops_is_said_without_its_addresses() {
    let _serial = serial();
    let said = |rove: Rove| {
        let (ladder, heard) = heard_ladder(rove, FakeClock::new());
        let _ = ladder.connect();
        heard.take().split_off(1)
    };
    assert_eq!(
        said(rove(vec!["nowhere.invalid:1".to_owned()])),
        [format!(
            "{P}rendezvous not started — no bootstrap node resolved"
        )]
    );
    let empty = commons_of(vec![]);
    assert_eq!(
        said(rove(vec![empty.addr.to_string()])),
        [format!(
            "{P}presence not read — the engine has published no presence"
        )]
    );
    // A holder that serves the presence and never spends a put's token.
    let (engine, pairing) = pairing();
    let sealed = Presence {
        endpoints: vec![closed().parse().unwrap()],
    }
    .seal(&pairing.seal_key())
    .unwrap();
    let mut holder = FakeNode::bind(NodeId([2u8; 20]));
    holder.serve(
        vec![],
        Mood::Mute,
        vec![engine.sign(pairing.presence_salt(), 1, sealed).unwrap()],
    );
    let mut router = FakeNode::bind(NodeId([1u8; 20]));
    router.serve(vec![holder.node()], Mood::Router, vec![]);
    assert_eq!(
        said(rove(vec![router.addr.to_string()])),
        [
            format!("{P}presence read — seq 1, 1 endpoint(s) (1 v4)"),
            format!(
                "{P}call not written — the DHT put failed (reason withheld: it names the target)"
            ),
        ]
    );
    let gone = commons(vec![closed().parse().unwrap()]);
    let mut short = rove(vec![gone.addr.to_string()]);
    short.window = Duration::from_millis(300);
    let lines = said(short);
    assert_eq!(lines.len(), 4, "{lines:?}");
    assert_eq!(
        lines[2],
        format!("{P}punch started — 1 endpoint(s) (1 v4), window 0s")
    );
    assert_eq!(
        lines[3],
        format!("{P}punch expired after 0s with no stream")
    );
}

#[test]
fn a_held_stream_says_why_it_was_dropped() {
    let _serial = serial();
    let dir = pki();
    for (unprompted, why) in [
        (
            Beat::Say(b"{\"ok\":true}".to_vec()),
            "a frame that is not a ping",
        ),
        (Beat::Say(Vec::new()), "the engine ended it"),
        (Beat::Raw(vec![0xff; 4]), "the stream closed or failed"),
        (Beat::Hangup, "the stream closed or failed"),
    ] {
        let (address, _served) = serve_held(
            &dir,
            "ca",
            "server",
            vec![vec![Beat::Answer(reply(1)), unprompted]],
        );
        let node = commons(vec![address.parse().unwrap()]);
        let (seat, heard) = heard_seat(&dir, &node, FakeClock::new());
        seat.ask(&serde_json::json!({ "op": "a" })).unwrap();
        assert!(heard.wait(why), "{why}: {:?}", heard.take());
    }
    let (address, _served) = serve_held(&dir, "ca", "server", vec![vec![Beat::Answer(reply(1))]]);
    let node = commons(vec![address.parse().unwrap()]);
    let clock = FakeClock::new();
    let (seat, heard) = heard_seat(&dir, &node, clock.clone());
    seat.ask(&serde_json::json!({ "op": "a" })).unwrap();
    assert!(until(&mut || seat.held() == 1, WAIT));
    clock.advance(Duration::from_mins(2));
    assert!(heard.wait("two minutes of silence"));
}
