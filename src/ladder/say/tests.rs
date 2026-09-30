//! The lines themselves: each is the house shape, says families and counts,
//! and no line built from addresses ever carries one; and the voice says a
//! repeated line of one kind once.

use super::*;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::sync::Mutex;

fn endpoints() -> Vec<SocketAddr> {
    vec![
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(203, 0, 113, 7)), 7737),
        SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), 7738),
        SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 7739),
    ]
}

#[test]
fn every_line_is_the_house_shape_and_names_no_address() {
    let e = endpoints();
    let ips: Vec<IpAddr> = e.iter().map(SocketAddr::ip).collect();
    let w = Duration::from_secs(35);
    let called_ = call::Called {
        nonce: 7,
        endpoints: e.clone(),
        acks: 2,
    };
    let lines = [
        direct(&[(ips[0], "refused"), (ips[1], "timed out")]),
        direct(&[]),
        unresolved(),
        resting(),
        moved(&ips, 1),
        repunch(&e, w),
        no_commons("no bootstrap node resolved"),
        found(9, &e),
        not_found("no DHT node answered get for 0123abcd"),
        called(&called_, 11),
        not_called(),
        punching(&e, w),
        landed(Some(ips[1])),
        landed(None),
        expired(w),
        kept(),
        handed(3),
        dropped("two minutes of silence", 0),
    ];
    for line in &lines {
        assert!(line.starts_with("yog.rendezvous: "), "{line}");
        for addr in ["203.0.113", "127.0.0.1", "::1", "0123abcd"] {
            assert!(!line.contains(addr), "{line} names {addr}");
        }
    }
    assert_eq!(
        lines[0],
        "yog.rendezvous: direct rung — 2 address(es) tried: v4 refused, v6 timed out"
    );
    assert_eq!(
        lines[1],
        "yog.rendezvous: direct rung — 0 address(es) tried: none"
    );
    assert_eq!(
        lines[9],
        "yog.rendezvous: call written — nonce 7, seq 11, 3 endpoint(s) (1 v6 loopback, 2 v4), 2 ack(s)"
    );
    assert_eq!(
        lines[13],
        "yog.rendezvous: punch landed — peer unknown family"
    );
}

#[test]
fn the_presence_rungs_own_sentences_are_said_and_the_walks_withheld() {
    for own in call::UNREAD {
        assert!(not_found(own).ends_with(own));
    }
    assert!(not_found("anything else").ends_with("(reason withheld: it names the target)"));
}

#[test]
fn a_dial_error_is_said_as_its_class() {
    let class = |kind| outcome(&io::Error::from(kind));
    assert_eq!(class(io::ErrorKind::ConnectionRefused), "refused");
    assert_eq!(class(io::ErrorKind::TimedOut), "timed out");
    assert_eq!(class(io::ErrorKind::WouldBlock), "timed out");
    assert_eq!(class(io::ErrorKind::HostUnreachable), "failed");
}

#[test]
fn families_count_v6_first_and_say_none_for_nothing() {
    assert_eq!(families(&[]), "none");
    assert_eq!(
        families(&[IpAddr::V6(Ipv6Addr::LOCALHOST)]),
        "1 v6 loopback"
    );
    assert_eq!(families(&[IpAddr::V4(Ipv4Addr::LOCALHOST)]), "1 v4");
}

/// Which v6 a line counted is its scope, grouped, and never its address
/// (bl-792e): the overlay's ULA and the internet's global read apart.
#[test]
fn a_v6_is_counted_by_its_scope_and_v4_after() {
    let ip = |s: &str| s.parse::<IpAddr>().unwrap();
    let said = families(&[
        ip("2001:db8::1"),
        ip("fd7a::1"),
        ip("192.0.2.1"),
        ip("2001:db8::2"),
    ]);
    assert_eq!(said, "2 v6 global, 1 v6 ula, 1 v4");
    assert!(!said.contains("2001"));
}

#[test]
fn a_kind_says_a_line_once_until_it_changes_or_is_forgotten() {
    let lines = Arc::new(Mutex::new(Vec::new()));
    let into = Arc::clone(&lines);
    let voice = Voice::new(Arc::new(move |l: &str| {
        into.lock().unwrap().push(l.to_owned());
    }));
    voice.once("a", "one");
    voice.once("a", "one");
    voice.once("b", "one");
    voice.once("a", "two");
    voice.once("a", "two");
    voice.forget("a");
    voice.once("a", "two");
    voice.say("said");
    voice.sink()("sunk");
    assert_eq!(
        *lines.lock().unwrap(),
        ["one", "one", "two", "two", "said", "sunk"]
    );
}

#[test]
fn the_device_sink_writes_a_line_and_the_quiet_one_does_not() {
    logcat()("yog.rendezvous: a line the suite says on stderr");
    quiet()("nothing");
}
