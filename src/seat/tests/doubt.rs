//! **An act whose reply was lost** (yog REMOTE §3, bl-d1f1, consumed in
//! bl-07b1): the channel carried the gesture and died before the answer, so
//! the engine may have run it and this end can never learn which. What is
//! asserted here is the whole of what the contract asks a client for — the
//! act is written exactly once, the sentence says it may have run and names
//! the read that settles it, and the deposit's fate is counted apart from a
//! refusal so the composer does not hand its draft back.

use super::{Turn, conv_reply, model_turns, nothing_set, ops, settle, tr_reply, ws_reply};

/// The five turns every case here shares: the first pass, the assignments
/// preload a focus change makes, and the pass that follows it. What comes
/// after is the gesture under test.
fn focused() -> Vec<Turn> {
    vec![
        Turn::Answer(vec![ws_reply()]),
        Turn::Answer(vec![nothing_set()]),
        Turn::Answer(vec![ws_reply()]),
        Turn::Answer(vec![conv_reply()]),
        Turn::Answer(vec![tr_reply()]),
    ]
}

/// The three turns of the refresh a gesture wakes.
fn refresh() -> Vec<Turn> {
    vec![
        Turn::Answer(vec![ws_reply()]),
        Turn::Answer(vec![conv_reply()]),
        Turn::Answer(vec![tr_reply()]),
    ]
}

/// **A deposit in doubt is counted as neither taken nor refused**, and the
/// message crosses the wire once. The counter is the load-bearing half: the
/// composer's echo reads these moving (bl-66fb), and a lost reply counted as
/// a refusal would put the operator's own text back in the field one tap from
/// a second copy of a message the engine may already hold.
#[test]
fn a_deposit_whose_reply_was_lost_is_in_doubt_and_never_sent_again() {
    let mut turns = focused();
    turns.push(Turn::Hangup); // the deposit: read, then a FIN where the answer belongs
    turns.extend(refresh());
    let (mut model, served) = model_turns(turns);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.focus_conversation("home".into(), "a1".into());
    settle(&mut model, &|s| !s.transcript.is_empty());
    model.deposit("hello".into());
    let snap = settle(&mut model, &|s| s.doubted == 1);
    assert_eq!(
        (snap.landed, snap.refused),
        (0, 0),
        "neither, and that is the point"
    );
    let said = snap.error.unwrap_or_default();
    assert!(
        said.starts_with("deposit may have run: the reply was lost ("),
        "{said}"
    );
    assert!(said.contains("Nothing was sent again"), "{said}");
    assert!(
        said.contains("The transcript says whether it landed"),
        "{said}"
    );
    drop(model);
    // The recovery is a read and never a resend: one `message` on the wire,
    // and the reads that follow it are the ordinary pass.
    let wire = ops(&served.join().unwrap());
    assert_eq!(
        wire.iter().filter(|op| *op == "message").count(),
        1,
        "{wire:?}"
    );
}

/// **The same for a nudge**, which is REMOTE §9.8's own example of a gesture
/// that is not idempotent — *two clicks of Nudge are two nudges* — so a
/// client that re-sent one on a dead reply would advance a conversation twice
/// off one tap. Its sentence names the row the next pass re-reads, because
/// that is where the answer is.
#[test]
fn a_nudge_whose_reply_was_lost_names_the_row_that_settles_it() {
    let mut turns = focused();
    turns.push(Turn::Hangup);
    turns.extend(refresh());
    let (mut model, served) = model_turns(turns);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.focus_conversation("home".into(), "a1".into());
    settle(&mut model, &|s| !s.transcript.is_empty());
    model.nudge();
    let snap = settle(&mut model, &|s| s.error.is_some());
    let said = snap.error.unwrap_or_default();
    assert!(said.starts_with("nudge may have run:"), "{said}");
    assert!(said.contains("a repeat would be a second nudge"), "{said}");
    assert!(
        said.contains("The conversation's row says whether a turn is still in flight"),
        "{said}"
    );
    // A nudge is not a deposit and moves no deposit counter.
    assert_eq!((snap.landed, snap.refused, snap.doubted), (0, 0, 0));
    drop(model);
    let wire = ops(&served.join().unwrap());
    assert_eq!(
        wire.iter().filter(|op| *op == "nudge").count(),
        1,
        "{wire:?}"
    );
}

/// **The contract is every act's, not the message's alone.** One body over
/// four gestures, each fired into a channel that hangs up where its receipt
/// belongs: each says its own name and names the read that settles it, and
/// none of them is sent a second time. `start` appears here as its staging
/// leg — a `prepare` whose reply is lost mints a body in the engine that this
/// end can never see, which is the same doubt one step earlier.
#[test]
fn every_act_in_doubt_names_itself_and_the_read_that_settles_it() {
    /// One case: the act's own name on the wire, a phrase from the read its
    /// sentence must name, and the gesture that fires it.
    struct Case(&'static str, &'static str, Box<dyn Fn(&super::Model)>);
    let cases = vec![
        Case(
            "stop",
            "The conversation's row says",
            Box::new(|m: &super::Model| m.stop_turn(false)),
        ),
        Case(
            "model",
            "The workspace's assignments are read",
            Box::new(|m: &super::Model| m.pick_model("anthropic".into(), "opus".into())),
        ),
        Case(
            "tune",
            "The workspace's assignments are read",
            Box::new(|m: &super::Model| m.set_priority(true)),
        ),
        Case(
            "start",
            "The workspace's conversation list says",
            Box::new(|m: &super::Model| m.start_conversation("look".into())),
        ),
        // The row menu's three (§13.5, bl-f97c). None of them is idempotent
        // either, and the third is the one that has to say out loud that no
        // read here settles it — which is a sentence the contract allows and
        // a claim it would not.
        Case(
            "interrupt",
            "The conversation's transcript says whether the text landed",
            Box::new(|m: &super::Model| {
                m.row_act(
                    "a1".into(),
                    crate::codec::RowAct::Interrupt {
                        content: "no, this".into(),
                    },
                );
            }),
        ),
        Case(
            "retarget",
            "The conversation's records say whether it landed",
            Box::new(|m: &super::Model| m.row_act("a1".into(), crate::codec::RowAct::Retarget)),
        ),
        Case(
            "flag",
            "The conversation's row carries the attention mark",
            Box::new(|m: &super::Model| {
                m.row_act(
                    "a1".into(),
                    crate::codec::RowAct::Flag {
                        reason: "wandered".into(),
                    },
                );
            }),
        ),
        // The capability boundary's three (§13.7, bl-b39d). The answer is the
        // one act here whose settling read is the QUEUE — a call that was
        // answered is a call the next queue read no longer carries — and the
        // floor pair is the second group that must say no read here shows it.
        Case(
            "answer",
            "The conversation's queue row says whether the call is still parked",
            Box::new(|m: &super::Model| m.answer(crate::codec::Verdict::Pass)),
        ),
        Case(
            "revoke",
            "No read this seat makes says which floor stands",
            Box::new(|m: &super::Model| m.row_act("a1".into(), crate::codec::RowAct::Revoke)),
        ),
        Case(
            "restore",
            "No read this seat makes says which floor stands",
            Box::new(|m: &super::Model| m.row_act("a1".into(), crate::codec::RowAct::Restore)),
        ),
        // The attempt (§13.16). A repeat is a second child doing the same
        // work, so the sentence names the spine the gesture was fired from —
        // a child hangs on the notch it was born at.
        Case(
            "fork",
            "The conversation's spine says whether a child appeared",
            Box::new(|m: &super::Model| m.fork("config/strict".into(), "g".into())),
        ),
        // The ball pane's own (§13.10, bl-f36e). A repeated close is a second
        // close and a repeated create is a second ball, so the sentence names
        // the read the pane makes anyway — the view it was fired on.
        Case(
            "close",
            "The pane is read again",
            Box::new(|m: &super::Model| {
                m.ball_act(
                    "p".into(),
                    crate::codec::BallAct::Close { id: "bl-1".into() },
                );
            }),
        ),
    ];
    for Case(act, read, fire) in cases {
        let mut turns = focused();
        turns.push(Turn::Hangup);
        turns.extend(refresh());
        let (mut model, _served) = model_turns(turns);
        settle(&mut model, &|s| !s.workspaces.is_empty());
        model.focus_conversation("home".into(), "a1".into());
        settle(&mut model, &|s| !s.transcript.is_empty());
        fire(&model);
        let said = settle(&mut model, &|s| s.error.is_some())
            .error
            .unwrap_or_default();
        assert!(said.starts_with(&format!("{act} may have run:")), "{said}");
        assert!(said.contains(read), "{said}");
    }
}

/// The firing leg of the same pair: the staging came back, so the engine holds
/// a prepared body, and the `prompt` that spends it is what earns the FIN.
#[test]
fn a_firing_whose_reply_was_lost_is_in_doubt_like_any_other_act() {
    let mut turns = focused();
    turns.push(Turn::Answer(vec![super::prepared()]));
    turns.push(Turn::Hangup);
    turns.extend(refresh());
    let (mut model, served) = model_turns(turns);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.focus_conversation("home".into(), "a1".into());
    settle(&mut model, &|s| !s.transcript.is_empty());
    model.start_conversation("look".into());
    let said = settle(&mut model, &|s| s.error.is_some())
        .error
        .unwrap_or_default();
    assert!(said.starts_with("start may have run:"), "{said}");
    drop(model);
    let wire = ops(&served.join().unwrap());
    assert_eq!(
        wire.iter().filter(|op| *op == "prompt").count(),
        1,
        "{wire:?}"
    );
}

/// **An engine that said no is not in doubt**, and painting it as such would
/// spend the word on the one case where the answer is known. The refusal is
/// carried whole and the contract's sentence is nowhere near it.
#[test]
fn an_engine_that_refused_the_act_is_definite_and_says_only_what_it_said() {
    let refusal = serde_json::json!({ "ok": false, "error": "this leaf may not assign models" })
        .to_string()
        .into_bytes();
    let mut turns = focused();
    turns.push(Turn::Answer(vec![refusal]));
    turns.extend(refresh());
    let (mut model, _served) = model_turns(turns);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.focus_conversation("home".into(), "a1".into());
    settle(&mut model, &|s| !s.transcript.is_empty());
    model.pick_model("anthropic".into(), "opus".into());
    let said = settle(&mut model, &|s| s.error.is_some())
        .error
        .unwrap_or_default();
    assert_eq!(said, "this leaf may not assign models");
}
