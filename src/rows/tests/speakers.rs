//! **The speaker lines as the desktop spells them** (DESIGN §23 item 15;
//! lernie `rows::block`, `rows::role::OPERATOR`): the operator's turn says
//! its sender, a model turn the model id, its reasoning `<model id>
//! (thinking)`, and none of them a colon.

use super::{MODEL, THOUGHT, delivered, go, go_open, model, prefixes, streaming, text, thought};
use crate::rows::{Kind, Role, Row, compact};

#[test]
fn op_is_the_operator_too() {
    let rows = go(&[delivered("001", "op", "go")]);
    assert_eq!(rows[0].prefix, "op");
    assert_eq!(rows[0].role, Some(Role::User));
}

#[test]
fn a_turn_says_its_model_and_its_reasoning_says_so() {
    let rows = go_open(&[
        delivered("001", "user", "go"),
        model("002", vec![thought("weighing it"), text("done")]),
    ]);
    assert_eq!(prefixes(&rows), ["user", "1 thought", THOUGHT, MODEL]);
}

/// The tail's entry names no model, so it is the model that last answered.
#[test]
fn the_live_tail_says_the_model_that_last_answered() {
    let rows = go(&[
        model("001", vec![text("first")]),
        streaming("002", "mulling", "so far"),
    ]);
    assert_eq!(prefixes(&rows), [MODEL, THOUGHT, MODEL]);
}

#[test]
fn a_folded_prose_step_is_who_and_its_first_line() {
    let rows = go(&[model("001", vec![thought("weighing it\nand more")])]);
    assert_eq!(rows[0].kind, Kind::Thinking);
    assert_eq!(
        compact(&rows[0]).as_deref(),
        Some("sonnet-9 (thinking) · weighing it")
    );
    let remark = Row {
        kind: Kind::Remark,
        ..rows[0].clone()
    };
    assert!(compact(&remark).is_some());
}

#[test]
fn a_call_or_a_wordless_step_has_no_one_line() {
    let rows = go(&[model("001", vec![thought("x")])]);
    let call = Row {
        kind: Kind::Answered,
        ..rows[0].clone()
    };
    assert_eq!(compact(&call), None);
    let blank = Row {
        preview: String::new(),
        ..rows[0].clone()
    };
    assert_eq!(compact(&blank), None);
}

/// **An intermediate response is a finished step**, so it answers the
/// machinery's knob and stands compact under an open working (§23 item 6).
#[test]
fn a_remark_under_an_open_working_stands_compact() {
    let entries = [
        delivered("001", "user", "go"),
        model("002", vec![text("looking"), thought("hm")]),
        model("003", vec![text("done")]),
    ];
    let open = std::collections::BTreeSet::from([go(&entries)[1].key.clone()]);
    let rows = crate::rows::rows(&entries, None, crate::rows::AutoExpand::default(), &open);
    let remark = rows.iter().find(|row| row.kind == Kind::Remark);
    assert_eq!(remark.map(|row| row.expanded), Some(false));
    assert_eq!(
        remark.and_then(compact).as_deref(),
        Some("sonnet-9 · looking")
    );
}
