//! **The sign-in that waits for a code** (REMOTE §8.3 at edition 31, DESIGN
//! §13.19): the address the run states, the code handed back as the same
//! act, and the one lane followed through both.
//!
//! Scripted positionally like the tail beside it. The address and the code
//! are invented in the reserved documentation space.

use std::sync::mpsc;

use serde_json::json;

use super::super::{Turn, conv_reply, ops, settle, ws_reply};
use super::{focused, frame, tail};

const PAGE: &str = "https://provider.invalid/authorize?state=s";

/// A frame of a run waiting for a code: its lines, and the address it states.
fn waiting(lines: &[&str]) -> Vec<u8> {
    let lines: Vec<_> = lines
        .iter()
        .map(|text| json!({ "err": true, "text": text }))
        .collect();
    json!({ "ok": true, "kind": "login", "lines": lines, "authorize": PAGE })
        .to_string()
        .into_bytes()
}

/// The address the snapshot's run is waiting at, if it is waiting.
fn awaited(snap: &crate::seat::Snapshot) -> Option<String> {
    snap.login.as_ref().and_then(|held| held.view.awaits())
}

/// **The code goes back as the same act, verbatim, and the same lane is
/// followed.** The receipt takes the address off the glass at once and none
/// of its lines — the lane already held carries those at its own cursor — and
/// the exit arrives where every exit does.
#[test]
fn a_pasted_code_is_handed_back_and_the_same_tail_says_what_came_of_it() {
    let (feed, frames) = mpsc::channel();
    let (mut model, served) = focused(vec![
        Turn::Answer(vec![waiting(&[PAGE])]),
        Turn::Answer(vec![ws_reply()]),
        Turn::Answer(vec![conv_reply()]),
        Turn::Feed(frames),
        // The code's receipt: the whole standing, no longer waiting.
        Turn::Answer(vec![frame(&[(true, PAGE), (true, "sign in there")], None)]),
        Turn::Answer(vec![ws_reply()]),
        Turn::Answer(vec![conv_reply()]),
    ]);
    model.sign_in("acme".into());
    let snap = settle(&mut model, &|s| awaited(s).is_some());
    assert_eq!(awaited(&snap).as_deref(), Some(PAGE));
    feed.send(waiting(&[PAGE, "sign in there"])).unwrap();
    settle(&mut model, &|s| {
        tail(s, "acme").is_some_and(|lines| lines.len() == 2)
    });
    model.hand_code("acme".into(), "pasted-code#state".into());
    let snap = settle(&mut model, &|s| awaited(s).is_none());
    assert_eq!(snap.error, None);
    assert_eq!(
        tail(&snap, "acme").unwrap_or_default(),
        [PAGE, "sign in there"],
        "the receipt's lines are the lane's to say, not said twice"
    );
    feed.send(frame(&[(true, "signed in")], Some(0))).unwrap();
    let snap = settle(&mut model, &|s| {
        s.login.as_ref().is_some_and(|held| held.view.settled())
    });
    assert_eq!(
        tail(&snap, "acme").unwrap_or_default(),
        [PAGE, "sign in there", "signed in"]
    );
    drop(model);
    drop(feed);
    let requests = served.join().unwrap();
    let wire = ops(&requests);
    assert_eq!(wire.iter().filter(|op| *op == "login-tail").count(), 1);
    let handed: serde_json::Value = serde_json::from_slice(&requests[8]).unwrap();
    assert_eq!(
        handed,
        json!({ "op": "login", "workspace": "home", "provider": "acme",
                "code": "pasted-code#state" })
    );
}

/// **A waiting run is quiet, and a lane that ends with no exit is asked
/// again.** The server's hold expires on a run nobody has finished; the next
/// pass redials, and the replay states the address again — which is also what
/// a seat that left for the browser and came back is shown.
#[test]
fn a_waiting_run_whose_lane_ended_is_asked_again_and_still_states_its_address() {
    let (mut model, served) = focused(vec![
        Turn::Answer(vec![waiting(&[PAGE])]),
        Turn::Answer(vec![ws_reply()]),
        Turn::Answer(vec![conv_reply()]),
        // The hold expires with the buffer written and no exit.
        Turn::Answer(vec![waiting(&[PAGE, "sign in there"])]),
        Turn::Answer(vec![ws_reply()]),
        Turn::Answer(vec![conv_reply()]),
        Turn::Hold(vec![waiting(&[PAGE, "sign in there", "still waiting"])]),
    ]);
    model.sign_in("acme".into());
    settle(&mut model, &|s| {
        tail(s, "acme").is_some_and(|lines| lines.len() == 2)
    });
    model.focus_workspace(Some("home".into()));
    let snap = settle(&mut model, &|s| {
        tail(s, "acme").is_some_and(|lines| lines.len() == 3)
    });
    assert_eq!(awaited(&snap).as_deref(), Some(PAGE));
    assert_eq!(snap.error, None);
    drop(model);
    let wire = ops(&served.join().unwrap());
    assert_eq!(wire.iter().filter(|op| *op == "login-tail").count(), 2);
}

/// **A code nothing is waiting for is the server's sentence, in the banner**
/// — it names the restart, and the plain act beside the row is that restart.
/// Nothing about the tail is invented from it: the lane says what stands.
#[test]
fn a_code_nothing_waits_for_is_refused_in_the_servers_words() {
    let refusal = json!({ "ok": false,
        "error": "no sign-in for \"acme\" is waiting for a code here" })
    .to_string()
    .into_bytes();
    let (mut model, _served) = focused(vec![
        Turn::Answer(vec![refusal]),
        Turn::Answer(vec![ws_reply()]),
        Turn::Answer(vec![conv_reply()]),
    ]);
    model.hand_code("acme".into(), "stale".into());
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(
        snap.error.as_deref(),
        Some("no sign-in for \"acme\" is waiting for a code here")
    );
    assert!(snap.login.is_none());
}

/// **A code in doubt is never sent again**: it is spent once, and the tail is
/// the read that says whether it was taken.
#[test]
fn a_code_whose_reply_was_lost_is_not_sent_twice() {
    let (mut model, served) = focused(vec![
        Turn::Hangup,
        Turn::Answer(vec![ws_reply()]),
        Turn::Answer(vec![conv_reply()]),
        Turn::Hold(vec![frame(&[(true, "exchanging")], None)]),
    ]);
    model.hand_code("acme".into(), "pasted".into());
    let snap = settle(&mut model, &|s| s.error.is_some());
    let said = snap.error.unwrap_or_default();
    assert!(said.contains("a code is spent once"), "{said}");
    settle(&mut model, &|s| {
        tail(s, "acme").is_some_and(|lines| !lines.is_empty())
    });
    drop(model);
    let wire = ops(&served.join().unwrap());
    assert_eq!(wire.iter().filter(|op| *op == "login").count(), 1);
}
