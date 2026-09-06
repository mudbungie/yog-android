//! **The contract over every act, as one table** — split from the two cases
//! beside it (bl-f645) when the admin surface's own two took `doubt.rs` past
//! the 300 wall. The seam is the file's own: a case that asserts something
//! particular about ONE gesture (a deposit's three counters, a nudge's row)
//! stays there, and the table that says the SAME thing about every act is
//! here.

use super::{focused, refresh};
use crate::seat::tests::{Model, Turn, model_turns, settle};

/// **The contract is every act's, not the message's alone.** One body over
/// four gestures, each fired into a channel that hangs up where its receipt
/// belongs: each says its own name and names the read that settles it, and
/// none of them is sent a second time. `start` appears here as its staging
/// leg — a `prepare` whose reply is lost mints a body in the engine that this
/// end can never see, which is the same doubt one step earlier.
/// One case: the act's own name on the wire, a phrase from the read its
/// sentence must name, and the gesture that fires it.
struct Case(&'static str, &'static str, Box<dyn Fn(&Model)>);

/// **The roster, apart from the driver that spends it, and split by subject.**
/// One function per group because the list grows with every act this seat
/// gains and the body that drives it does not — and because the two groups
/// differ in what they are ABOUT: the focused conversation, or a surface over
/// the world.
fn cases() -> Vec<Case> {
    let mut cases = aimed();
    cases.extend(surfaces());
    cases
}

/// The acts addressed at the focused conversation, or at what it is set to.
fn aimed() -> Vec<Case> {
    vec![
        Case(
            "stop",
            "The conversation's row says",
            Box::new(|m: &Model| m.stop_turn(false)),
        ),
        Case(
            "model",
            "The workspace's assignments are read",
            Box::new(|m: &Model| m.pick_model("anthropic".into(), "opus".into())),
        ),
        Case(
            "tune",
            "The workspace's assignments are read",
            Box::new(|m: &Model| m.set_priority(true)),
        ),
        Case(
            "start",
            "The workspace's conversation list says",
            Box::new(|m: &Model| m.start_conversation("look".into())),
        ),
        // The row menu's three (§13.5, bl-f97c). None of them is idempotent
        // either, and the third is the one that has to say out loud that no
        // read here settles it — which is a sentence the contract allows and
        // a claim it would not.
        Case(
            "interrupt",
            "The conversation's transcript says whether the text landed",
            Box::new(|m: &Model| {
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
            Box::new(|m: &Model| m.row_act("a1".into(), crate::codec::RowAct::Retarget)),
        ),
        Case(
            "flag",
            "The conversation's row carries the attention mark",
            Box::new(|m: &Model| {
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
            Box::new(|m: &Model| m.answer(crate::codec::Verdict::Pass)),
        ),
        Case(
            "revoke",
            "No read this seat makes says which floor stands",
            Box::new(|m: &Model| m.row_act("a1".into(), crate::codec::RowAct::Revoke)),
        ),
        Case(
            "restore",
            "No read this seat makes says which floor stands",
            Box::new(|m: &Model| m.row_act("a1".into(), crate::codec::RowAct::Restore)),
        ),
    ]
}

/// The acts fired from a surface over the world rather than at the focus.
fn surfaces() -> Vec<Case> {
    vec![
        // The attempt (§13.16). A repeat is a second child doing the same
        // work, so the sentence names the spine the gesture was fired from —
        // a child hangs on the notch it was born at.
        Case(
            "fork",
            "The conversation's spine says whether a child appeared",
            Box::new(|m: &Model| m.fork("config/strict".into(), "g".into())),
        ),
        // The admin surface's (§13.17). A repeated config write re-applies
        // bytes the operator may have edited since, so the sentence names the
        // read that shows what the file holds now.
        Case(
            "config",
            "the config read says what the file holds now",
            Box::new(|m: &Model| {
                m.admin(crate::codec::AdminAct::Config {
                    at: crate::codec::Destination::Cadence,
                    text: "cadence: {}".into(),
                });
            }),
        ),
        Case(
            "delete-workspace",
            "The workspace roster says whether it is gone",
            Box::new(|m: &Model| {
                m.admin(crate::codec::AdminAct::DeleteWorkspace {
                    workspace: "home".into(),
                    typed: "home".into(),
                });
            }),
        ),
        // The mint (§13.18). A repeat is refused by the certificate the
        // engine kept, so the sentence names the roster that lists a client
        // the moment its registration exists.
        Case(
            "enroll",
            "The workspace's machines list a client",
            Box::new(|m: &Model| m.enroll("phone-2".into(), crate::leaf::Grade::Foot)),
        ),
        // The ball pane's own (§13.10, bl-f36e). A repeated close is a second
        // close and a repeated create is a second ball, so the sentence names
        // the read the pane makes anyway — the view it was fired on.
        Case(
            "close",
            "The pane is read again",
            Box::new(|m: &Model| {
                m.ball_act(
                    "p".into(),
                    crate::codec::BallAct::Close { id: "bl-1".into() },
                );
            }),
        ),
    ]
}

#[test]
fn every_act_in_doubt_names_itself_and_the_read_that_settles_it() {
    for Case(act, read, fire) in cases() {
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
