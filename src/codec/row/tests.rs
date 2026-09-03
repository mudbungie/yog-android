//! The row acts' pins: exact envelope bytes against the vendored corpus's own
//! frames, the receipt the flag earns, and the three shell-facing readings
//! (`op`, `wants`, `with`) that the menu is built out of.

use serde_json::json;

use super::{RowAct, decode, encode};
use crate::codec::reply::{Reply, decode as reply_decode};
use crate::codec::{Act, Gesture, encode as gesture};

/// The corpus's own subject, so the spellings below are comparable to
/// `corpus/request/{interrupt,retarget,flag}.json` by eye.
const WS: &str = "ws";
const AGENT: &str = "c-1";

#[test]
fn interrupt_spelling() {
    let act = RowAct::Interrupt {
        content: "no, this".to_owned(),
    };
    assert_eq!(
        encode(WS, AGENT, &act),
        json!({ "op": "interrupt", "workspace": "ws", "agent": "c-1", "content": "no, this" })
    );
}

#[test]
fn retarget_spelling() {
    assert_eq!(
        encode(WS, AGENT, &RowAct::Retarget),
        json!({ "op": "retarget", "workspace": "ws", "agent": "c-1" })
    );
}

#[test]
fn flag_spelling() {
    let act = RowAct::Flag {
        reason: "it is rewriting an unrelated crate".to_owned(),
    };
    assert_eq!(
        encode(WS, AGENT, &act),
        json!({ "op": "flag", "workspace": "ws", "agent": "c-1",
                "reason": "it is rewriting an unrelated crate" })
    );
}

/// The subject rides the outer act, so a row gesture encodes through
/// `codec::encode` to the same bytes — the arm this file is reached by at
/// runtime.
#[test]
fn a_row_gesture_encodes_through_the_one_encoder() {
    let g = Gesture::Act(Act::Row {
        workspace: WS.to_owned(),
        agent: AGENT.to_owned(),
        act: RowAct::Retarget,
    });
    assert_eq!(
        gesture(&g),
        json!({ "op": "retarget", "workspace": "ws", "agent": "c-1" })
    );
}

#[test]
fn every_row_act_round_trips_through_its_own_frame() {
    let acts = [
        RowAct::Interrupt {
            content: "no, this".to_owned(),
        },
        RowAct::Retarget,
        RowAct::Flag {
            reason: "wandered".to_owned(),
        },
    ];
    for act in acts {
        let frame = encode(WS, AGENT, &act);
        let o = frame.as_object().expect("an object");
        assert_eq!(
            decode(act.op(), o).unwrap(),
            Act::Row {
                workspace: WS.to_owned(),
                agent: AGENT.to_owned(),
                act,
            }
        );
    }
}

/// A missing parameter refuses rather than defaulting: an `interrupt` with no
/// `content` is not an interrupt with an empty one.
#[test]
fn a_row_act_missing_its_parameter_refuses() {
    let frame = json!({ "op": "interrupt", "workspace": "ws", "agent": "c-1" });
    let o = frame.as_object().expect("an object");
    assert_eq!(
        decode("interrupt", o).unwrap_err(),
        "missing or non-string field \"content\""
    );
    let frame = json!({ "op": "flag", "workspace": "ws", "agent": "c-1" });
    let o = frame.as_object().expect("an object");
    assert_eq!(
        decode("flag", o).unwrap_err(),
        "missing or non-string field \"reason\""
    );
}

/// **The arm `request::decode` cannot reach, asserted anyway.** Its caller
/// matches the three ops first, so nothing routes a fourth here today — but a
/// `_` that answered one with a retarget is the silent misread REMOTE §3's
/// third rule forbids, and an unreachable-by-one-caller branch is still a
/// branch this crate ships.
#[test]
fn an_op_this_file_does_not_spell_refuses_by_name() {
    let frame = json!({ "op": "fork", "workspace": "ws", "agent": "c-1" });
    let o = frame.as_object().expect("an object");
    assert_eq!(decode("fork", o).unwrap_err(), "row: unknown op \"fork\"");
}

/// The three readings the menu is built from. One table, because the menu's
/// roster and the codec's are the same fact (§13.5).
#[test]
fn the_menu_readings_are_the_wire_tokens() {
    let empty = [
        RowAct::Interrupt {
            content: String::new(),
        },
        RowAct::Retarget,
        RowAct::Flag {
            reason: String::new(),
        },
    ];
    let ops: Vec<&str> = empty.iter().map(RowAct::op).collect();
    assert_eq!(ops, ["interrupt", "retarget", "flag"]);
    let wants: Vec<Option<&str>> = empty.iter().map(RowAct::wants).collect();
    assert_eq!(
        wants,
        [
            Some("type the text first"),
            None,
            Some("type the reason first")
        ]
    );
    // `with` puts the composer's text in whichever field this act's
    // parameter is, and leaves the one that takes none alone.
    let filled: Vec<RowAct> = empty
        .iter()
        .map(|act| act.with("said".to_owned()))
        .collect();
    assert_eq!(
        filled,
        [
            RowAct::Interrupt {
                content: "said".to_owned()
            },
            RowAct::Retarget,
            RowAct::Flag {
                reason: "said".to_owned()
            }
        ]
    );
}

/// The flag's own receipt, which is the only reply the group adds.
#[test]
fn a_flag_is_answered_flagged() {
    let frame = json!({ "kind": "flagged", "ok": true });
    assert_eq!(reply_decode(&frame).unwrap().unwrap(), Reply::Flagged);
    assert_eq!(Reply::Flagged.kind(), "flagged");
}
