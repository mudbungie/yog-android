//! **The conversation row's three acts** (DESIGN §13.5, bl-f97c): the
//! envelopes each puts on the wire, the receipt each earns, and the two
//! sentences a misdirected one earns.
//!
//! The load-bearing assertion is the SUBJECT. Every other act this seat posts
//! addresses the focus; these address the row that was long-pressed, and the
//! test proves it by firing at a conversation the model has never focused —
//! only the workspace is. A regression that reached for `focus.agent` would
//! refuse here instead of quietly acting on the wrong conversation.

use serde_json::{Value, json};

use super::{conv_reply, nothing_set, outcome, settle, ws_reply};
use crate::codec::RowAct;

/// The `flagged` receipt, which is the one reply this group added to the
/// codec. It carries nothing but its own verdict.
fn flagged() -> Vec<u8> {
    json!({ "ok": true, "kind": "flagged" })
        .to_string()
        .into_bytes()
}

fn interrupting() -> RowAct {
    RowAct::Interrupt {
        content: "no, this".to_owned(),
    }
}

fn flagging() -> RowAct {
    RowAct::Flag {
        reason: "wandered".to_owned(),
    }
}

/// The three envelopes, byte for byte, fired at a row the seat has not opened.
#[test]
fn the_three_row_acts_address_the_row_and_not_the_focus() {
    let (mut model, served) = super::model_against(vec![
        vec![ws_reply()],
        vec![nothing_set()],
        vec![ws_reply()],
        vec![conv_reply()],
        vec![outcome(true, "")], // interrupt
        vec![ws_reply()],
        vec![conv_reply()],
        vec![outcome(true, "")], // retarget
        vec![ws_reply()],
        vec![conv_reply()],
        vec![flagged()], // flag
        vec![ws_reply()],
        vec![conv_reply()],
    ]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    // The WORKSPACE is focused and no conversation is: this is the
    // conversation-list screen, which is where a row menu opens.
    model.focus_workspace(Some("home".into()));
    settle(&mut model, &|s| !s.conversations.is_empty());
    assert_eq!(model.snapshot().focus.agent, None, "nothing is opened");

    for act in [interrupting(), RowAct::Retarget, flagging()] {
        model.row_act("a1".into(), act);
        settle(&mut model, &|s| {
            s.error.is_none() && !s.conversations.is_empty()
        });
    }

    drop(model);
    let requests = served.join().unwrap();
    let sent = |at: usize| serde_json::from_slice::<Value>(&requests[at]).unwrap();
    assert_eq!(
        sent(4),
        json!({ "op": "interrupt", "workspace": "home", "agent": "a1",
                "content": "no, this" })
    );
    assert_eq!(
        sent(7),
        json!({ "op": "retarget", "workspace": "home", "agent": "a1" })
    );
    assert_eq!(
        sent(10),
        json!({ "op": "flag", "workspace": "home", "agent": "a1",
                "reason": "wandered" })
    );
}

/// **The floor pair** (§13.7, bl-b39d): both envelopes, and the receipt read
/// as the floor that stands AFTERWARDS rather than as an echo of what was
/// asked. Agreement is silence; a disagreement is the one sentence an
/// operator cannot get any other way, because no read this seat makes says
/// which floor is standing.
#[test]
fn the_floor_pair_reads_what_stands_rather_than_what_was_asked() {
    let floored = |standing: bool| {
        json!({ "ok": true, "kind": "floored", "standing": standing })
            .to_string()
            .into_bytes()
    };
    let (mut model, served) = super::model_against(vec![
        vec![ws_reply()],
        vec![nothing_set()],
        vec![ws_reply()],
        vec![conv_reply()],
        vec![floored(true)], // revoke: what was asked for
        vec![ws_reply()],
        vec![conv_reply()],
        vec![floored(true)], // restore: an ancestor is still revoked
        vec![ws_reply()],
        vec![conv_reply()],
    ]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.focus_workspace(Some("home".into()));
    settle(&mut model, &|s| !s.conversations.is_empty());
    model.row_act("a1".into(), RowAct::Revoke);
    settle(&mut model, &|s| {
        s.error.is_none() && !s.conversations.is_empty()
    });
    model.row_act("a1".into(), RowAct::Restore);
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(
        snap.error.as_deref(),
        Some("restore: the conversation stays floored — an ancestor of it is still revoked")
    );
    drop(model);
    let requests = served.join().unwrap();
    let sent = |at: usize| serde_json::from_slice::<Value>(&requests[at]).unwrap();
    assert_eq!(
        sent(4),
        json!({ "op": "revoke", "workspace": "home", "agent": "a1" })
    );
    assert_eq!(
        sent(7),
        json!({ "op": "restore", "workspace": "home", "agent": "a1" })
    );
}

/// The other half of the same reading, which should never happen and is said
/// anyway: a revoke the engine took that left no floor standing.
#[test]
fn a_revoke_that_left_no_floor_standing_says_so() {
    let (mut model, _s) = super::model_against(vec![
        vec![ws_reply()],
        vec![nothing_set()],
        vec![ws_reply()],
        vec![conv_reply()],
        vec![
            json!({ "ok": true, "kind": "floored", "standing": false })
                .to_string()
                .into_bytes(),
        ],
    ]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.focus_workspace(Some("home".into()));
    settle(&mut model, &|s| !s.conversations.is_empty());
    model.row_act("a1".into(), RowAct::Revoke);
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(
        snap.error.as_deref(),
        Some("revoke: the engine took it and says no floor stands over the conversation")
    );
}

/// A refusal is the engine's own sentence, named by the op that earned it —
/// the same shape `stop` has had since bl-48fa.
#[test]
fn a_refused_row_act_carries_the_engines_words_under_its_own_name() {
    let (mut model, _s) = super::model_against(vec![
        vec![ws_reply()],
        vec![nothing_set()],
        vec![ws_reply()],
        vec![conv_reply()],
        vec![outcome(false, "nothing is running")],
    ]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.focus_workspace(Some("home".into()));
    settle(&mut model, &|s| !s.conversations.is_empty());
    model.row_act("a1".into(), interrupting());
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(
        snap.error.as_deref(),
        Some("interrupt refused: nothing is running")
    );
}

/// **The receipt is read off the ACT, not guessed at the reply.** A flag
/// answered with an `outcome` is the wrong shape and says so, rather than
/// being read as the success an `ok: true` would look like — which is the
/// whole reason `seat::acts::row` decides the expected receipt before it
/// sends.
#[test]
fn a_row_act_answered_with_the_other_shape_names_what_came_back() {
    let (mut model, _s) = super::model_against(vec![
        vec![ws_reply()],
        vec![nothing_set()],
        vec![ws_reply()],
        vec![conv_reply()],
        vec![outcome(true, "")], // the flag, answered as though it ran something
        vec![ws_reply()],
        vec![conv_reply()],
        vec![flagged()], // the retarget, answered with the flag's own receipt
    ]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.focus_workspace(Some("home".into()));
    settle(&mut model, &|s| !s.conversations.is_empty());
    model.row_act("a1".into(), flagging());
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(
        snap.error.as_deref(),
        Some("flag: the engine answered outcome instead")
    );
    model.row_act("a1".into(), RowAct::Retarget);
    let snap = settle(&mut model, &|s| {
        s.error.as_deref() != Some("flag: the engine answered outcome instead")
    });
    assert_eq!(
        snap.error.as_deref(),
        Some("retarget: the engine answered flagged instead")
    );
}

/// A row act with no workspace under it refuses before the wire, under its own
/// op's name — a row can only be painted inside a workspace, so this is a
/// defect one level up rather than a state an operator can reach.
#[test]
fn a_row_act_with_no_workspace_focused_refuses_before_the_wire() {
    let (mut model, served) = super::model_against(vec![vec![ws_reply()]]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.row_act("a1".into(), flagging());
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(snap.error.as_deref(), Some("flag: no workspace is focused"));
    drop(model);
    // One connection, carrying the opening roster read and nothing else.
    assert_eq!(super::ops(&served.join().unwrap()), ["workspaces"]);
}
