//! The platform's network report (bl-792e): a change heard from the
//! platform acts at once — the held line drops and the change is said with
//! no ask in flight — and the next call names only the reported network's
//! addresses, never a ULA, a link-local, the shared space or the old
//! network's.

use super::*;
use crate::ladder::network::{self, Scope};
use crate::test_support::{Beat, serve_held};

fn ip(s: &str) -> IpAddr {
    s.parse().unwrap()
}

/// A seat like [`seat`] on a network the test reports to, saying its lines.
fn reported_seat(dir: &std::path::Path, node: &Commons) -> (Seat, Heard, Arc<Network>) {
    let heard = Heard::default();
    let mut rove = rove(vec![node.addr.to_string()]);
    rove.say = heard.sink();
    let network = Arc::clone(&rove.network);
    let m = material(dir, "ca", "client", &closed());
    let seat = Seat::open_with(&m, Some(rove), FakeClock::new()).unwrap();
    (seat, heard, network)
}

#[test]
fn a_report_drops_the_held_line_at_once_and_the_next_call_names_only_its_addresses() {
    let _serial = serial();
    let dir = pki();
    let (address, served) = serve_held(
        &dir,
        "ca",
        "server",
        vec![vec![Beat::Answer(reply(1))], vec![Beat::Answer(reply(2))]],
    );
    let node = commons(vec![address.parse().unwrap()]);
    let (seat, heard, network) = reported_seat(&dir, &node);
    assert_eq!(
        seat.ask(&serde_json::json!({ "op": "a" })).unwrap()[0]["n"],
        1
    );
    let (seq, first) = called(&node);
    assert!(until(&mut || seat.held() == 1, WAIT), "the line is held");
    heard.take();
    // What `ConnectivityManager` hands over for a new default network: its
    // link-local (with the zone the platform spells), the overlay's ULA and
    // v4, and the one address a peer could reach.
    let shared = v4(100, 100, 1, 1);
    network.changed(&format!(
        "fe80::1%wlan0\nfd7a:115c::5\n{shared}\n127.0.0.50\nnot an address\n"
    ));
    assert_eq!(
        seat.held(),
        0,
        "the held line dropped with no ask in flight"
    );
    assert!(
        heard.take().iter().any(|l| l
            == "yog.rendezvous: network changed — this box now sends from 1 v4; \
                1 held stream(s) dropped, rest cleared"),
        "and the change was said at the report"
    );
    assert_eq!(
        seat.ask(&serde_json::json!({ "op": "b" })).unwrap()[0]["n"],
        2
    );
    let (next, call) = called(&node);
    assert!(next > seq, "the climb wrote a fresh call");
    let ips: Vec<IpAddr> = call.endpoints.iter().map(SocketAddr::ip).collect();
    // The report's one reachable address, then whatever the commons
    // observed this box calling from — and nothing else.
    assert_eq!(ips.first(), Some(&ip("127.0.0.50")), "{ips:?}");
    assert!(ips[1..].iter().all(|o| *o == ip("127.0.0.1")), "{ips:?}");
    assert_ne!(
        call.endpoints[0].port(),
        first.endpoints[0].port(),
        "from a port of the new generation"
    );
    let dialled = heard.take();
    assert!(
        dialled
            .iter()
            .all(|l| !l.contains("rendezvous: network changed")),
        "said once, at the report, and not again at the dial: {dialled:?}"
    );
    drop(seat);
    assert_eq!(served.join().unwrap().len(), 2);
}

fn fallback() -> Vec<IpAddr> {
    vec![ip("127.0.0.9"), ip("fd00::1")]
}

#[test]
fn the_platform_is_the_one_source_once_it_reports_and_only_a_moved_set_is_a_change() {
    let network = Network::new(fallback);
    assert_eq!(
        network.now(),
        (0, vec![ip("127.0.0.9")]),
        "no ULA, even read"
    );
    network.changed("127.0.0.9\n");
    assert_eq!(network.generation(), 0, "the same set is no change");
    network.changed("192.0.2.7\n2001:db8::7\n");
    assert_eq!(network.now(), (1, vec![ip("192.0.2.7"), ip("2001:db8::7")]));
    network.changed(" 2001:db8::7 \n192.0.2.7\n192.0.2.7\n");
    assert_eq!(network.generation(), 1, "one set in another order");
    network.changed("");
    assert_eq!(network.now(), (2, vec![]), "no default network");
}

#[test]
fn a_report_reaches_only_the_ladders_still_alive() {
    let _serial = serial();
    let network = Network::new(fallback);
    let mut r = rove(vec![]);
    r.network = Arc::clone(&network);
    let gone = entry("127.0.0.1:2".to_owned(), Some(r), Arc::new(SystemClock));
    network.watch(&gone);
    drop(gone);
    network.changed("192.0.2.8\n");
    network.changed("192.0.2.9\n");
    assert_eq!(network.generation(), 1, "the first report is the first set");
}

fn v4(a: u8, b: u8, c: u8, d: u8) -> IpAddr {
    IpAddr::V4(Ipv4Addr::new(a, b, c, d))
}

#[test]
fn every_address_has_one_scope_and_only_the_outside_reachable_are_kept() {
    // The v4 are built from octets: a dotted routable address in the tree is
    // the leak gate's to refuse, even a fabricated one.
    let table = [
        (ip("::1"), Scope::Loopback, "loopback"),
        (ip("127.0.0.1"), Scope::Loopback, "loopback"),
        (ip("fe80::1"), Scope::LinkLocal, "link-local"),
        (ip("febf::1"), Scope::LinkLocal, "link-local"),
        (v4(169, 254, 1, 1), Scope::LinkLocal, "link-local"),
        (ip("fc00::1"), Scope::Ula, "ula"),
        (ip("fd7a:115c::1"), Scope::Ula, "ula"),
        (v4(100, 64, 0, 1), Scope::Shared, "shared"),
        (v4(100, 127, 255, 1), Scope::Shared, "shared"),
        (v4(192, 0, 0, 4), Scope::Translator, "translator"),
        (v4(10, 0, 0, 1), Scope::Private, "private"),
        (v4(192, 168, 1, 2), Scope::Private, "private"),
        (ip("2001:db8::1"), Scope::Global, "global"),
        (ip("fec0::1"), Scope::Global, "global"),
        (v4(100, 128, 0, 1), Scope::Global, "global"),
        (v4(192, 0, 1, 1), Scope::Global, "global"),
        (v4(25, 1, 2, 3), Scope::Global, "global"),
    ];
    for (addr, scope, word) in table {
        assert_eq!(Scope::of(addr), scope, "{addr}");
        assert_eq!(scope.word(), word);
    }
    let kept = network::reachable(table.iter().map(|(a, _, _)| *a).collect());
    assert_eq!(
        kept,
        [
            v4(10, 0, 0, 1),
            v4(25, 1, 2, 3),
            v4(100, 128, 0, 1),
            ip("127.0.0.1"),
            v4(192, 0, 1, 1),
            v4(192, 168, 1, 2),
            ip("::1"),
            ip("2001:db8::1"),
            ip("fec0::1"),
        ]
    );
}
