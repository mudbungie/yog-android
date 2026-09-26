//! The ladder end to end: a seat over the real transport against a fake
//! DHT on loopback UDP and a scripted engine that holds its connection —
//! every rung taken and fallen through, the backoff, the network change,
//! and the held stream's pings and silence on a clock the test turns.
//!
//! **One lock over every test here.** The addresses this "box" sends from
//! are one static the suite moves ([`flap`]), and a held stream is a fact
//! about a socket another test's flap would clear between two asks;
//! serialising the file is the ordering the assertions stand on.

use super::*;
use crate::dht::Config;
use crate::dht::tests::fake::{FakeNode, Mood};
use crate::dht::{Keypair, NodeId};
use crate::rendezvous::Pairing;
use crate::rendezvous::item::Presence;
use crate::test_support::{FakeClock, material, mint_ca, mint_leaf, scratch, until};
use crate::transport::Seat;
use std::net::Ipv4Addr;
use std::path::PathBuf;
use std::sync::atomic::AtomicU8;
use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, Instant};

mod fallthrough;
mod held;
mod rungs;

const WAIT: Duration = Duration::from_secs(10);

fn serial() -> MutexGuard<'static, ()> {
    static SERIAL: Mutex<()> = Mutex::new(());
    SERIAL
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// The last octet of the one address the suite's box sends from.
static NET: AtomicU8 = AtomicU8::new(1);

fn addresses() -> Vec<IpAddr> {
    vec![IpAddr::V4(Ipv4Addr::new(
        127,
        0,
        0,
        NET.load(Ordering::Relaxed),
    ))]
}

/// The network changes: the box sends from another address.
fn flap() {
    NET.fetch_add(1, Ordering::Relaxed);
}

/// The engine's rendezvous keypair, and the pairing a client holds for it.
fn pairing() -> (Keypair, Pairing) {
    let engine = Keypair::from_seed([5u8; 32]).unwrap();
    let pairing = Pairing {
        engine: engine.public(),
        salt: [6u8; 32],
    };
    (engine, pairing)
}

/// A fake commons shaped like the mainline (yog bl-f6e1): a bootstrap
/// router that answers only `find_node`, and the one node past it that
/// holds `items` and takes a `put`. `addr` is the router's — the one
/// address a client is given.
struct Commons {
    addr: SocketAddr,
    _nodes: [FakeNode; 2],
}

fn commons_of(items: Vec<crate::dht::Mutable>) -> Commons {
    let mut holder = FakeNode::bind(NodeId([2u8; 20]));
    holder.serve(vec![], Mood::Answer, items);
    let mut router = FakeNode::bind(NodeId([1u8; 20]));
    router.serve(vec![holder.node()], Mood::Router, vec![]);
    Commons {
        addr: router.addr,
        _nodes: [router, holder],
    }
}

/// A fake commons holding the engine's presence at `endpoints`.
fn commons(endpoints: Vec<SocketAddr>) -> Commons {
    let (engine, pairing) = pairing();
    let sealed = Presence { endpoints }.seal(&pairing.seal_key()).unwrap();
    commons_of(vec![
        engine.sign(pairing.presence_salt(), 1, sealed).unwrap(),
    ])
}

fn rove(bootstrap: Vec<String>) -> Rove {
    Rove {
        pairing: pairing().1,
        bootstrap,
        config: Config {
            alpha: 3,
            k: 3,
            round: Duration::from_millis(300),
            max_queries: 64,
        },
        window: Duration::from_secs(2),
        addresses,
    }
}

/// An address nothing listens on: port 1, which no box this suite runs on
/// serves. Not a port bound and dropped — std binds with `SO_REUSEADDR`, and
/// the punch's connectors leave from `SO_REUSEPORT` ports that linger for
/// their window after a climb, so a freed ephemeral port was measured
/// answering a later test's dial one run in a few.
fn closed() -> String {
    "127.0.0.1:1".to_owned()
}

fn pki() -> PathBuf {
    let dir = scratch();
    mint_ca(&dir, "ca");
    mint_leaf(&dir, "ca", "server", true);
    mint_leaf(&dir, "ca", "client", false);
    dir
}

/// A seat whose entry names a dark address and roves through `node`.
fn seat(dir: &std::path::Path, node: &Commons, clock: Arc<FakeClock>) -> Seat {
    let m = material(dir, "ca", "client", &closed());
    Seat::open_with(&m, Some(rove(vec![node.addr.to_string()])), clock).unwrap()
}

fn reply(n: u32) -> Vec<Vec<u8>> {
    vec![
        serde_json::json!({ "ok": true, "kind": "transcript", "rows": [], "n": n })
            .to_string()
            .into_bytes(),
    ]
}

#[test]
fn an_entry_that_does_not_rove_dials_as_it_always_has() {
    let _serial = serial();
    let address = closed();
    let ladder = Ladder::new(address.clone(), None, Arc::new(SystemClock));
    let e = ladder.connect().err().unwrap();
    assert!(e.starts_with(&format!("connect {address}: ")), "{e}");
    assert_eq!(ladder.held(), 0);
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let ladder = Ladder::new(
        listener.local_addr().unwrap().to_string(),
        None,
        Arc::new(SystemClock),
    );
    assert!(matches!(
        ladder.connect().unwrap(),
        Conn::Fresh { punched: false, .. }
    ));
}

#[test]
fn the_direct_rung_resolves_a_name_and_names_what_it_could_not_reach() {
    let _serial = serial();
    let e = direct("nowhere.invalid:1").unwrap_err();
    assert!(e.starts_with("connect nowhere.invalid:1: "), "{e}");
    let e = direct(&closed()).unwrap_err();
    assert!(e.contains("refused"), "{e}");
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let address = format!("localhost:{}", listener.local_addr().unwrap().port());
    // `localhost` resolves to both families on most boxes; the v6 one is
    // refused and the v4 one connects, or the other way round — either way
    // one of them lands.
    assert!(direct(&address).is_ok());
}

#[test]
fn the_mainline_shape_is_the_engines() {
    let r = Rove::mainline(pairing().1);
    assert_eq!(r.bootstrap.len(), 4);
    assert_eq!(r.window, Duration::from_secs(35));
    assert!((r.addresses)().iter().all(|ip| !ip.is_loopback()));
    assert_eq!(r.config.k, 8);
    assert!(SystemClock.unix() > 1_700_000_000);
    assert!(SystemClock.now() <= Instant::now());
}
