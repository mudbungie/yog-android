//! The held stream between asks: pings discarded wherever they land, a
//! frame that is not one drops the stream, two minutes of silence hangs up
//! on the injected clock, and a network change drops every held stream.

use super::*;
use crate::test_support::{Beat, serve_held};
use std::sync::mpsc;

#[test]
fn a_ping_ahead_of_the_reply_and_one_in_the_silence_are_both_discarded() {
    let _serial = serial();
    let dir = pki();
    let (gate, open) = mpsc::channel();
    let (address, served) = serve_held(
        &dir,
        "ca",
        "server",
        vec![vec![
            Beat::Answer(reply(1)),
            Beat::Ping,
            Beat::Gate(open),
            Beat::Ping,
            Beat::Answer(reply(2)),
        ]],
    );
    let node = commons(vec![address.parse().unwrap()]);
    let clock = FakeClock::new();
    let seat = Arc::new(seat(&dir, &node, clock.clone()));
    assert_eq!(
        seat.ask(&serde_json::json!({ "op": "a" })).unwrap()[0]["n"],
        1
    );
    assert!(until(&mut || seat.held() == 1, WAIT));
    // The ping in the silence is read by the holder; the clock says a
    // minute and a half passed before it, and it resets the silence.
    clock.advance(Duration::from_secs(90));
    std::thread::sleep(Duration::from_millis(600));
    assert_eq!(seat.held(), 1, "a ping is not the end of the silence");
    let asking = Arc::clone(&seat);
    let second = std::thread::spawn(move || asking.ask(&serde_json::json!({ "op": "b" })).unwrap());
    assert!(
        until(&mut || seat.held() == 0, WAIT),
        "the ask took the stream"
    );
    gate.send(()).unwrap();
    let second = second.join().unwrap();
    assert_eq!(
        second.len(),
        1,
        "the ping ahead of the reply is not a frame of it"
    );
    assert_eq!(second[0]["n"], 2);
    assert!(until(&mut || seat.held() == 1, WAIT));
    // Two minutes of nothing: this end hangs up.
    clock.advance(Duration::from_mins(2));
    assert!(
        until(&mut || seat.held() == 0, WAIT),
        "the silence bound hung up"
    );
    assert_eq!(
        served.join().unwrap(),
        vec![vec![
            serde_json::json!({ "op": "a" }).to_string().into_bytes(),
            serde_json::json!({ "op": "b" }).to_string().into_bytes(),
        ]]
    );
}

/// Anything but a ping where no reply is due — a frame, a terminator, a
/// header promising more than any frame may carry — drops the stream, and
/// the engine's next read finds nobody there.
#[test]
fn anything_but_a_ping_in_the_silence_drops_the_stream() {
    let _serial = serial();
    let dir = pki();
    for unprompted in [
        Beat::Say(b"{\"ok\":true}".to_vec()),
        Beat::Say(Vec::new()),
        Beat::Raw(vec![0xff; 4]),
    ] {
        let (address, served) = serve_held(
            &dir,
            "ca",
            "server",
            vec![vec![
                Beat::Answer(reply(1)),
                unprompted,
                Beat::Answer(reply(2)),
            ]],
        );
        let node = commons(vec![address.parse().unwrap()]);
        let seat = seat(&dir, &node, FakeClock::new());
        assert_eq!(
            seat.ask(&serde_json::json!({ "op": "a" })).unwrap()[0]["n"],
            1
        );
        assert!(until(&mut || seat.held() == 0, WAIT), "dropped, not kept");
        drop(seat);
        assert_eq!(
            served.join().unwrap(),
            vec![vec![
                serde_json::json!({ "op": "a" }).to_string().into_bytes()
            ]]
        );
    }
    assert!(!until(&mut || false, Duration::ZERO), "a wait that ran out");
}

#[test]
fn a_network_change_drops_every_held_stream_and_the_next_ask_re_punches() {
    let _serial = serial();
    let dir = pki();
    let (address, served) = serve_held(
        &dir,
        "ca",
        "server",
        vec![vec![Beat::Answer(reply(1))], vec![Beat::Answer(reply(2))]],
    );
    let node = commons(vec![address.parse().unwrap()]);
    let seat = seat(&dir, &node, FakeClock::new());
    assert_eq!(
        seat.ask(&serde_json::json!({ "op": "a" })).unwrap()[0]["n"],
        1
    );
    assert!(until(&mut || seat.held() == 1, WAIT));
    flap();
    assert_eq!(
        seat.ask(&serde_json::json!({ "op": "b" })).unwrap()[0]["n"],
        2
    );
    drop(seat);
    assert_eq!(
        served.join().unwrap().len(),
        2,
        "the held stream was not reused"
    );
}

#[test]
fn the_ping_frame_is_exactly_one_key() {
    use crate::ladder::held::is_ping;
    assert!(is_ping(&serde_json::json!({ "ping": true })));
    assert!(!is_ping(&serde_json::json!({ "ping": false })));
    assert!(!is_ping(&serde_json::json!({ "ping": true, "more": 1 })));
    assert!(!is_ping(&serde_json::json!([true])));
}
