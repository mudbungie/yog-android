//! **A failure is not an error until it persists** (bl-3202): the grace, the
//! rows that survive a failed pass, and the one thing it must never do —
//! paint one focus's rows under another focus.
//!
//! Every case here is deterministic by construction rather than by timing:
//! the cadence is an hour, so each pass is a gesture the test sent, and the
//! scripted turns say which pass sees what. `Turn::Hangup` is what makes a
//! channel BREAK in the middle of a live listener (bl-8641), so a pass can
//! fail with more passes still to come.

use super::{Model, cache_in, conv_reply, material, nothing_set, pki, settle, ws_named, ws_reply};
use crate::test_support::{Turn, serve_turns};
use crate::transport::Seat;

/// A model over a scripted sequence of turns, at the module's own long rest.
fn model_turns(turns: Vec<Turn>) -> Model {
    let dir = pki();
    let (address, _served) = serve_turns(&dir, "ca", "server", turns);
    let seat = Seat::open(&material(&dir, "ca", "client", &address)).unwrap();
    Model::start(seat, super::REST, cache_in(&dir))
}

/// The whole rule in one walk: a first failure is silent and keeps the rows
/// it had; the second earns the banner; a pass that answers clears it at
/// once. Each stage is read off a roster that names its own pass, so no
/// assertion here can be satisfied by a snapshot from another one.
#[test]
fn a_failure_paints_only_once_it_has_persisted_and_a_success_clears_it_at_once() {
    let mut model = model_turns(vec![
        Turn::Answer(vec![ws_named("home")]),
        Turn::Hangup,
        Turn::Hangup,
        Turn::Answer(vec![ws_named("away")]),
    ]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    // Pass two: the channel breaks. The rows stand and nothing is said.
    model.focus_workspace(None);
    // Pass three: it broke again, so now it is an error — and the roster is
    // still the one the engine gave, under the banner rather than replaced
    // by it.
    model.focus_workspace(None);
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(snap.workspaces[0].workspace, "home");
    // Pass four answers, and the sentence goes with the pass that earned it.
    model.focus_workspace(None);
    let snap = settle(&mut model, &|s| s.workspaces[0].workspace == "away");
    assert_eq!(snap.error, None);
}

/// The one thing the grace may not buy: [`crate::seat::Snapshot`] promises a
/// frame never pairs one focus's rows with another's, so a failed pass under
/// a focus that just MOVED publishes the empty lists it honestly has rather
/// than the previous focus's answer.
#[test]
fn a_failed_pass_under_a_new_focus_publishes_no_other_focuss_rows() {
    let mut model = model_turns(vec![
        Turn::Answer(vec![ws_reply()]),
        Turn::Answer(vec![nothing_set()]),
        Turn::Answer(vec![ws_reply()]),
        Turn::Answer(vec![conv_reply()]),
        Turn::Hangup,
    ]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.focus_workspace(Some("home".into()));
    settle(&mut model, &|s| !s.conversations.is_empty());
    // The transcript ask never lands: the channel breaks under the deeper
    // focus, and what the frame is handed is that focus with nothing in it.
    model.focus_conversation("home".into(), "a1".into());
    let snap = settle(&mut model, &|s| s.focus.agent.is_some());
    assert!(snap.workspaces.is_empty());
    assert!(snap.conversations.is_empty());
    assert!(snap.transcript.is_empty());
    assert_eq!(snap.error, None);
}

/// Both sentences at once: a gesture's own answer joins a refresh failure
/// that has ALREADY persisted, in the pass that earns both. The deposit
/// refuses before it reaches the wire (nothing is focused), so it costs no
/// turn — the two hangups are the two passes, and the second is what lifts
/// the refresh's sentence out of the grace.
#[test]
fn a_gestures_sentence_joins_a_failure_that_has_already_persisted() {
    let mut model = model_turns(vec![
        Turn::Answer(vec![ws_reply()]),
        Turn::Hangup,
        Turn::Hangup,
    ]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.focus_workspace(None);
    model.deposit("hello".into());
    let snap = settle(&mut model, &|s| s.error.is_some());
    let banner = snap.error.unwrap_or_default();
    assert!(
        banner.starts_with("deposit: no conversation is focused; "),
        "banner: {banner}"
    );
    assert!(
        ["connect ", "send:", "receive"]
            .iter()
            .any(|verb| banner.contains(verb)),
        "banner: {banner}"
    );
}
