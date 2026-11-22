//! **An act leaves at once** (bl-2cad): what the operator says is posted on
//! the poster's own thread and connection, so it never waits behind a read —
//! and it is addressed at the focus the operator last aimed, which the poster
//! hears from the handle and not from the worker.
//!
//! Every other file here keeps the wire a script by waiting for the worker to
//! rest before it says anything. This one is about the moment it is not
//! resting, so it needs to know a request is AT the server while another is
//! still unanswered — which is what the rendezvous feed below is for.

use super::{
    Turn, ag_reply, conv_reply, model_turns, nothing_set, ops, outcome, settle, tr_reply, ws_reply,
};
use serde_json::{Value, json};
use std::sync::mpsc;
use std::time::Duration;

/// Hand `frame` to a held turn and come back once the server has taken it —
/// which it does only when the connection that turn answers has ARRIVED. The
/// feed is a rendezvous, so this is how a test knows a request is at the
/// server; the bound is what makes a seat that never sent it a failure rather
/// than a hang. The sender comes back so the caller decides when the stream
/// ends.
fn played(feed: mpsc::SyncSender<Vec<u8>>, frame: Vec<u8>) -> mpsc::SyncSender<Vec<u8>> {
    let (done, waited) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = feed.send(frame);
        let _ = done.send(feed);
    });
    waited
        .recv_timeout(Duration::from_secs(10))
        .expect("the request this turn answers never reached the server")
}

/// **A send crosses while a read is parked, and lands where the tap aimed.**
/// The operator taps a conversation and sends at once. The worker is still
/// inside the first read that tap woke — the server has taken it and not
/// answered — and the message is at the server anyway, addressed at the
/// conversation just tapped. The scripted server takes connections in order,
/// so the third turn is reached only by a request that arrived while the
/// second was unanswered: a seat that queued the send behind the read never
/// gets there.
#[test]
fn a_send_crosses_while_a_read_is_parked_and_lands_where_the_tap_aimed() {
    let (read, held) = mpsc::sync_channel(0);
    let (receipt, sent) = mpsc::sync_channel(0);
    let mut turns = vec![
        Turn::Answer(vec![ws_reply()]),
        // The first read the tap wakes: taken by the server, not answered.
        Turn::Feed(held),
        // The message, arriving beside it.
        Turn::Feed(sent),
    ];
    // The rest of the pass the tap woke, then the pass the receipt wakes.
    for _ in 0..2 {
        turns.extend(
            [ws_reply(), conv_reply(), tr_reply(), ag_reply()].map(|r| Turn::Answer(vec![r])),
        );
    }
    let (mut model, served) = model_turns(turns);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.focus_conversation("home".into(), "a1".into());
    let read = played(read, nothing_set());
    model.deposit("hello".into());
    let receipt = played(receipt, outcome(true, ""));
    // The act is answered FIRST, while the read is still open.
    drop(receipt);
    drop(read);
    let snap = settle(&mut model, &|s| s.landed == 1 && !s.transcript.is_empty());
    assert_eq!(snap.error, None);
    drop(model);
    let requests = served.join().unwrap();
    assert_eq!(
        ops(&requests)[..3],
        ["workspaces", "roles", "message"],
        "the message crossed beside the read it did not wait for"
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&requests[2]).unwrap(),
        json!({ "op": "message", "workspace": "home", "agent": "a1", "content": "hello" })
    );
    assert_eq!(
        requests.len(),
        11,
        "and both passes ran: nothing was dropped"
    );
}
