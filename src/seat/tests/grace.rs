//! **A failure is not an error until it persists** (bl-3202), and **the wait
//! is said rather than silent** (bl-eec1): the grace, what it hushes and what
//! it does not, the rows that survive a failed pass, and the one thing it
//! must never do — paint one focus's rows under another focus.
//!
//! Every case here is deterministic by construction rather than by timing:
//! the cadence is an hour, so each pass is a gesture the test sent, and the
//! scripted turns say which pass sees what. `Turn::Hangup` is what makes a
//! channel BREAK in the middle of a live listener (bl-8641), so a pass can
//! fail with more passes still to come.

use super::{Model, cache_in, conv_reply, material, nothing_set, pki, settle, ws_named, ws_reply};
use crate::seat::Snapshot;
use crate::test_support::{Turn, serve_turns};
use crate::transport::Seat;

/// A model over a scripted sequence of turns, at the module's own long rest.
fn model_turns(turns: Vec<Turn>) -> Model {
    let dir = pki();
    let (address, _served) = serve_turns(&dir, "ca", "server", turns);
    let seat = Seat::open(&material(&dir, "ca", "client", &address)).unwrap();
    Model::start(seat, super::REST, cache_in(&dir))
}

/// The grace, spent one pass at a time. Each `focus_workspace(None)` is a
/// pass the test sent, so the count here is the count of rests the banner
/// waits — read off the constant rather than written out, so the two cannot
/// drift.
fn wait_out_the_grace(model: &mut Model) {
    for _ in 0..=crate::seat::pass::GRACE {
        model.focus_workspace(None);
    }
}

/// Every hangup the grace can absorb, plus the one that spends it.
fn hangups() -> Vec<Turn> {
    (0..=crate::seat::pass::GRACE + 1)
        .map(|_| Turn::Hangup)
        .collect()
}

/// The whole rule in one walk: a broken channel says *reconnecting* and keeps
/// the rows it had; the pass that spends the grace turns that into the
/// engine's own sentence; a pass that answers clears both at once. Each stage
/// is read off a roster that names its own pass, so no assertion here can be
/// satisfied by a snapshot from another one.
#[test]
fn a_broken_channel_reconnects_until_the_grace_is_spent_and_a_success_clears_it() {
    let mut turns = vec![Turn::Answer(vec![ws_named("home")])];
    turns.extend(hangups());
    turns.push(Turn::Answer(vec![ws_named("away")]));
    let mut model = model_turns(turns);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    // The channel breaks. Nothing has failed yet as far as an operator is
    // concerned — this is a redial — so the state is Working, the sentence is
    // absent, and the rows stand.
    model.focus_workspace(None);
    let snap = settle(&mut model, &|s: &Snapshot| s.reconnecting);
    assert_eq!(snap.error, None);
    assert_eq!(snap.workspaces[0].workspace, "home");
    // It goes on breaking until the grace is spent, and then it is an error —
    // and the roster is still the one the engine gave, under the banner
    // rather than replaced by it.
    wait_out_the_grace(&mut model);
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert!(!snap.reconnecting, "the two are never both on the glass");
    assert_eq!(snap.workspaces[0].workspace, "home");
    // The next pass answers, and both go with the pass that earned them.
    model.focus_workspace(None);
    let snap = settle(&mut model, &|s| s.workspaces[0].workspace == "away");
    assert_eq!(snap.error, None);
    assert!(!snap.reconnecting);
}

/// **The grace is the CHANNEL's alone** (bl-eec1). A reply of a kind this
/// build cannot use is the engine answering, not the radio — asking again
/// gets the same answer — so it paints on the pass that met it and never
/// reads as a reconnection.
#[test]
fn an_answer_this_build_cannot_use_paints_at_once_and_is_not_a_reconnection() {
    let mut model = model_turns(vec![Turn::Answer(vec![conv_reply()])]);
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(
        snap.error.as_deref(),
        Some("workspaces: the engine answered conversations instead")
    );
    assert!(!snap.reconnecting);
}

/// **A stream that ended without an answer is unusable, not a reconnection.**
/// The channel worked — the engine wrote its terminator and nothing else — so
/// there is nothing to re-dial, and the sentence says what happened rather
/// than promising a retry.
#[test]
fn an_engine_that_answers_nothing_at_all_is_not_a_reconnection() {
    let mut model = model_turns(vec![Turn::Answer(Vec::new())]);
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(
        snap.error.as_deref(),
        Some("the engine ended the stream without answering")
    );
    assert!(!snap.reconnecting);
}

/// **A frame this end cannot read is unusable too**, and for the same reason:
/// the engine spoke, so asking again asks the same question. The sentence is
/// the decoder's own, uninterpreted.
#[test]
fn a_reply_this_build_cannot_read_is_not_a_reconnection() {
    let mut model = model_turns(vec![Turn::Answer(vec![b"{}".to_vec()])]);
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert!(!snap.reconnecting);
    assert!(snap.error.is_some());
}

/// **The error names what failed.** A banner that says only that something
/// went wrong sends an operator to a log; the class an operator meets on a
/// resume — a socket that would not open — carries the engine's address and
/// the reason the platform gave.
///
/// An empty script is a listener that is already gone, which is what a dial
/// against an engine that is not there does.
#[test]
fn the_error_names_the_address_it_could_not_reach() {
    let mut model = model_turns(Vec::new());
    wait_out_the_grace(&mut model);
    let said = settle(&mut model, &|s| s.error.is_some())
        .error
        .unwrap_or_default();
    assert!(said.starts_with("connect 127.0.0.1:"), "banner: {said}");
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
    let mut turns = vec![Turn::Answer(vec![ws_reply()])];
    turns.extend(hangups());
    let mut model = model_turns(turns);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    wait_out_the_grace(&mut model);
    model.deposit("hello".into());
    // The deposit's own sentence leads, so it is what the wait is keyed on:
    // the refresh's sentence is already standing by this point, and settling
    // on `error.is_some()` would answer with the pass before the gesture.
    let snap = settle(&mut model, &|s| {
        s.error
            .as_deref()
            .is_some_and(|e| e.starts_with("deposit:"))
    });
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
