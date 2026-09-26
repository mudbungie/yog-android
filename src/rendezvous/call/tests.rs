//! The call names the observed address beside the local ones, at the punch
//! port, and round-trips through the seal the engine opens it with.

use super::*;
use crate::dht::tests::fake::{FakeNode, Mood};
use crate::dht::{Config, NodeId, Udp};
use crate::rendezvous::item::Call;
use std::time::Duration;

fn pairing() -> Pairing {
    Pairing {
        engine: [1u8; 32],
        salt: [2u8; 32],
    }
}

/// Place a call from `mine` at port 4000 through a commons whose one node
/// past the router answers in `mood`, and read the endpoints back opened.
fn placed(mood: Mood, mine: Vec<IpAddr>) -> Vec<SocketAddr> {
    let mut holder = FakeNode::bind(NodeId([2u8; 20]));
    holder.serve(vec![], mood, vec![]);
    let mut router = FakeNode::bind(NodeId([1u8; 20]));
    router.serve(vec![holder.node()], Mood::Router, vec![]);
    let config = Config {
        alpha: 3,
        k: 3,
        deadline: Duration::from_millis(300),
        max_queries: 64,
    };
    let fresh = || {
        let udp = Udp::bind("127.0.0.1:0".parse().unwrap()).unwrap();
        Dht::new(Box::new(udp), vec![router.addr], config.clone()).unwrap()
    };
    let p = pairing();
    let mut dht = fresh();
    dht.lookup(NodeId([0xff; 20])).unwrap();
    call(&mut dht, &p, 1, mine, 4000).unwrap();
    let item = fresh()
        .get(p.inbox_keypair().unwrap().public(), p.inbox_salt())
        .unwrap()
        .expect("the call was written");
    Call::open(&p.seal_key(), &item.value).unwrap().endpoints
}

fn at(s: &str) -> SocketAddr {
    s.parse().unwrap()
}

#[test]
fn the_observed_address_joins_the_call_at_the_punch_port() {
    let local: IpAddr = "192.0.2.2".parse().unwrap();
    let observed = Mood::Claim(at("203.0.113.7:6881"));
    assert_eq!(
        placed(observed, vec![local]),
        vec![at("192.0.2.2:4000"), at("203.0.113.7:4000")],
        "the address is taken, the observed port is not"
    );
}

#[test]
fn with_nothing_observed_the_call_is_the_local_addresses() {
    let local: IpAddr = "192.0.2.2".parse().unwrap();
    assert_eq!(
        placed(Mood::Answer, vec![local]),
        vec![at("192.0.2.2:4000")]
    );
}

#[test]
fn an_observed_address_that_is_already_local_is_not_listed_twice() {
    let local: IpAddr = "203.0.113.7".parse().unwrap();
    let observed = Mood::Claim(at("203.0.113.7:6881"));
    assert_eq!(placed(observed, vec![local]), vec![at("203.0.113.7:4000")]);
}
