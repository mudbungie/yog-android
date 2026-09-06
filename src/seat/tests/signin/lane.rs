//! **The held tail** (DESIGN §13.19, §14.1): the third lane's own half of the
//! sign-in — what a frame does to the fold, and the two ends of the lane's
//! life. Its own file beside the act, on the seam the wire draws: one is what
//! the operator SAYS, the other is what this seat holds open to hear.
//!
//! The lane is scripted positionally, like the follow lane: its dial is the
//! pass's, and the pass's order is a script. The attention lane is the only
//! one served aside (`test_support::serve_lanes`).

use std::sync::mpsc;

use super::super::{Turn, conv_reply, ops, settle, ws_reply};
use super::{focused, frame, tail};

/// **The lane replays, then appends.** A read starts holding nothing, so its
/// first frame is the whole buffer — which REPLACES what the act seeded
/// rather than doubling it — and every frame after it carries only what
/// landed since.
#[test]
fn the_lane_replays_first_and_appends_after() {
    let (feed, frames) = mpsc::channel();
    let (mut model, _served) = focused(vec![
        Turn::Answer(vec![frame(&[(true, "open https://auth.invalid")], None)]),
        Turn::Answer(vec![ws_reply()]),
        Turn::Answer(vec![conv_reply()]),
        Turn::Feed(frames),
    ]);
    model.sign_in("acme".into());
    settle(&mut model, &|s| tail(s, "acme").is_some());
    feed.send(frame(&[(true, "open https://auth.invalid")], None))
        .unwrap();
    feed.send(frame(&[(false, "waiting for the browser")], None))
        .unwrap();
    let snap = settle(&mut model, &|s| {
        tail(s, "acme").is_some_and(|lines| lines.len() == 2)
    });
    assert_eq!(
        tail(&snap, "acme").unwrap_or_default(),
        ["open https://auth.invalid", "waiting for the browser"]
    );
}

/// **A redialled lane replays, and the fold REPLACES rather than doubling.**
/// The engine's cursor is per read, so a lane that ended at its hold hands
/// the next one the whole buffer from zero — folding that onto what stands
/// would show every line twice.
#[test]
fn a_relaid_lane_replays_the_buffer_without_doubling_it() {
    let (mut model, _served) = focused(vec![
        Turn::Answer(vec![frame(&[(true, "first")], None)]),
        Turn::Answer(vec![ws_reply()]),
        Turn::Answer(vec![conv_reply()]),
        // The lane's hold expires with the whole buffer written and no
        // outcome — the engine's own thirty seconds, arriving at once.
        Turn::Answer(vec![frame(&[(true, "first"), (false, "second")], None)]),
        Turn::Answer(vec![ws_reply()]),
        Turn::Answer(vec![conv_reply()]),
        // …and the next pass dials it again, which replays from zero.
        Turn::Answer(vec![frame(&[(true, "first"), (false, "second")], None)]),
    ]);
    model.sign_in("acme".into());
    settle(&mut model, &|s| {
        tail(s, "acme").is_some_and(|lines| lines.len() == 2)
    });
    // A second pass, so the ended lane is redialled and replays.
    model.focus_workspace(Some("home".into()));
    let snap = settle(&mut model, &|s| tail(s, "acme").is_some());
    assert_eq!(tail(&snap, "acme").unwrap_or_default(), ["first", "second"]);
}

/// **A settled run takes its own lane down.** The engine's lane ends at the
/// outcome frame, so a seat that went on wanting one would redial a finished
/// sign-in once a cadence for as long as the screen stood open.
#[test]
fn a_settled_run_is_not_dialled_again() {
    let (mut model, served) = focused(vec![
        Turn::Answer(vec![frame(&[(true, "opening")], None)]),
        Turn::Answer(vec![ws_reply()]),
        Turn::Answer(vec![conv_reply()]),
        Turn::Answer(vec![frame(
            &[(true, "opening"), (true, "no device endpoint")],
            Some(78),
        )]),
        Turn::Answer(vec![ws_reply()]),
        Turn::Answer(vec![conv_reply()]),
    ]);
    model.sign_in("acme".into());
    settle(&mut model, &|s| {
        s.login.as_ref().is_some_and(|held| held.view.settled())
    });
    // Another pass, and no second lane: the fold says the run ended.
    model.focus_workspace(Some("home".into()));
    let snap = settle(&mut model, &|s| !s.conversations.is_empty());
    let held = snap.login.clone().unwrap_or_default();
    assert_eq!(held.view.outcome, Some(78));
    assert_eq!(held.view.fallback.as_deref(), Some("yog seat login acme"));
    // The whole flow is kept: the lines came back with the exit.
    assert_eq!(
        tail(&snap, "acme").unwrap_or_default(),
        ["opening", "no device endpoint"]
    );
    drop(model);
    assert_eq!(
        ops(&served.join().unwrap())
            .iter()
            .filter(|op| *op == "login-tail")
            .count(),
        1
    );
}

/// **Leaving the screen drops the tail, and crosses no wire.** The watch is
/// what makes the lane wanted, so `None` is the whole of closing it — a tail
/// can be left with the engine unreachable.
#[test]
fn leaving_the_screen_drops_the_tail() {
    let (mut model, served) = focused(vec![
        Turn::Answer(vec![frame(&[(true, "opening")], None)]),
        Turn::Answer(vec![ws_reply()]),
        Turn::Answer(vec![conv_reply()]),
        Turn::Hold(vec![frame(&[], None)]),
        Turn::Answer(vec![ws_reply()]),
        Turn::Answer(vec![conv_reply()]),
    ]);
    model.sign_in("acme".into());
    settle(&mut model, &|s| s.login.is_some());
    model.watch_login(None);
    let snap = settle(&mut model, &|s| s.login.is_none());
    assert!(snap.login.is_none());
    drop(model);
    // The watch itself asked nothing: the ops are the act, its pass, the
    // lane, and the pass the closing woke.
    assert_eq!(
        ops(&served.join().unwrap()),
        [
            "workspaces",
            "roles",
            "workspaces",
            "conversations",
            "login",
            "workspaces",
            "conversations",
            "login-tail",
            "workspaces",
            "conversations"
        ]
    );
}

/// **Watching another provider drops the first one's lines.** One provider's
/// flow under another's name is the same wrong claim as one focus's rows
/// under another's.
#[test]
fn watching_another_row_starts_that_row_empty() {
    let (mut model, _served) = focused(vec![
        Turn::Answer(vec![frame(&[(true, "opening acme")], None)]),
        Turn::Answer(vec![ws_reply()]),
        Turn::Answer(vec![conv_reply()]),
        Turn::Hold(vec![frame(&[], None)]),
        Turn::Answer(vec![ws_reply()]),
        Turn::Answer(vec![conv_reply()]),
        Turn::Hold(vec![frame(&[], None)]),
    ]);
    model.sign_in("acme".into());
    settle(&mut model, &|s| tail(s, "acme").is_some());
    model.watch_login(Some("rival".into()));
    let snap = settle(&mut model, &|s| tail(s, "rival").is_some());
    assert!(tail(&snap, "acme").is_none());
    assert_eq!(
        tail(&snap, "rival").unwrap_or_default(),
        Vec::<String>::new()
    );
}

/// The lane's own half of the same rule.
#[test]
fn a_lane_frame_of_the_wrong_kind_is_a_sentence_too() {
    let (mut model, _served) = focused(vec![
        Turn::Answer(vec![frame(&[], None)]),
        Turn::Answer(vec![ws_reply()]),
        Turn::Answer(vec![conv_reply()]),
        Turn::Hold(vec![super::super::pick::applied()]),
    ]);
    model.sign_in("acme".into());
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(
        snap.error.as_deref(),
        Some("login-tail: the engine answered applied instead")
    );
}
