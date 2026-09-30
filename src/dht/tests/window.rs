//! The sliding window (yog bl-d9c1): a silent query costs its own slot for one
//! deadline and never delays an answer beside it, and a slot is refilled the
//! moment its query answers or times out.

use super::*;
use std::time::Instant;

/// The door names a live node and a silent one together, and the live one
/// names a closer live one. Lockstep, the walk waited out the silent query's
/// whole round before asking the closer node; windowed, the closer node is
/// asked the moment the live one answers, and once the two live nodes are
/// the K closest the walk ends without waiting on the silent one at all —
/// proved by a deadline the test could not sit through: the walk ends
/// inside the minute the silent node was given, whatever the box's stalls.
#[test]
fn a_silent_node_does_not_delay_an_answer_in_the_same_window() {
    let mut near = FakeNode::bind(id(0xf0));
    near.serve(vec![], Mood::Answer, vec![]);
    let mut live = FakeNode::bind(id(0x80));
    live.serve(vec![near.node()], Mood::Answer, vec![]);
    let mut silent = FakeNode::bind(id(0x10));
    silent.serve(vec![], Mood::Silent, vec![]);
    let door = router(vec![silent.node(), live.node()]);
    let config = Config {
        k: 2,
        deadline: PATIENCE,
        ..quick()
    };
    let mut dht = client(vec![door.addr], config);
    let started = Instant::now();
    assert_eq!(
        dht.lookup(id(0xff)).unwrap(),
        vec![near.node(), live.node()]
    );
    assert!(
        started.elapsed() < PATIENCE,
        "the silent node was waited out"
    );
}

/// One slot, the closest node silent: its deadline passes, the slot is
/// refilled with the next node on the frontier, and that one answers.
#[test]
fn the_window_refills_when_a_query_times_out() {
    let mut silent = FakeNode::bind(id(0xf0));
    silent.serve(vec![], Mood::Silent, vec![]);
    let mut live = FakeNode::bind(id(0x10));
    live.serve(vec![], Mood::Answer, vec![]);
    let door = router(vec![silent.node(), live.node()]);
    let config = Config {
        alpha: 1,
        deadline: SILENCE,
        ..quick()
    };
    let mut dht = client(vec![door.addr], config);
    let started = Instant::now();
    assert_eq!(dht.lookup(id(0xff)).unwrap(), vec![live.node()]);
    // One deadline waited, not two: the second bound is a whole deadline of
    // margin for the box, the first is what the refill costs by design.
    let took = started.elapsed();
    assert!(took >= SILENCE, "{took:?}");
    assert!(took < SILENCE * 2, "{took:?}");
}

/// A holder that offered a token and is silent to `put` costs its own
/// deadline, and the holders beside it are still counted.
#[test]
fn a_put_counts_the_holders_that_answer_past_a_silent_one() {
    let kp = Keypair::from_seed([3u8; 32]).unwrap();
    let item = kp.sign(vec![], 1, b"p".to_vec()).unwrap();
    let mut holder = FakeNode::bind(id(0x42));
    holder.serve(vec![], Mood::Answer, vec![]);
    let mut mute = FakeNode::bind(id(0x43));
    mute.serve(vec![], Mood::Mute, vec![]);
    let door = router(vec![holder.node(), mute.node()]);
    let mut dht = client(vec![door.addr], waiting());
    assert_eq!(dht.put(item).unwrap(), 1);
}
