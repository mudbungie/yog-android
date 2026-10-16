//! **The table**: every row the sheet offers right now, read off what is
//! focused and what the engine says the focused conversation offers (REMOTE
//! §3.1, §9.4). Nouns first, then the acts on the conversation, then the two
//! toggles — the order §22 item 5 lists them in.

use std::sync::OnceLock;

use super::{Does, Row};
use crate::codec::View;
use crate::overlay::Overlay;
use crate::status::Slot;

/// What the rows are gated on, read off the focus and the focused row.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Gates {
    /// A workspace is focused.
    pub workspace: bool,
    /// A conversation is focused, and its row is read.
    pub conversation: Option<Offers>,
}

/// What the engine's own gates on the focused row say (REMOTE §9.4).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Offers {
    /// It holds the executor lock: a turn is running.
    pub stoppable: bool,
    /// Some other agent's id extends this one.
    pub stop_children: bool,
    /// Nothing is in flight: the branch is at rest.
    pub at_rest: bool,
}

/// The summary the op table carries for `op`, so the sheet says what `help`
/// says and there is one text to keep true.
fn said(op: &str) -> String {
    static TABLE: OnceLock<Vec<crate::help::Row>> = OnceLock::new();
    let table = TABLE.get_or_init(|| crate::help::rows(crate::help::TABLE).unwrap_or_default());
    table
        .iter()
        .find(|row| row.verb == op && !row.summary.is_empty())
        .map(|row| row.summary.clone())
        .unwrap_or_default()
}

fn row(word: &'static str, ops: &'static [&'static str], does: Does, own: &str) -> Row {
    // A row's own words win where it has them: a word that is not one op
    // (`stop all`, `records`) says what IT does.
    let summary = match (own, ops.first()) {
        ("", Some(op)) => said(op),
        _ => own.to_owned(),
    };
    Row {
        word,
        summary,
        ops,
        does,
        wants: None,
    }
}

fn wanting(mut row: Row, wants: &'static str) -> Row {
    row.wants = Some(wants);
    row
}

/// **Every row the sheet offers right now.**
pub fn rows(gates: &Gates) -> Vec<Row> {
    let mut out = vec![row(
        "conversations",
        &["workspaces"],
        Does::Open(Overlay::Switcher),
        "every workspace and its conversations",
    )];
    if gates.conversation.is_some() {
        out.push(row(
            "records",
            &["agent", "steps", "rail", "governing", "inbox", "lineages"],
            Does::Open(Overlay::Records),
            "the conversation's own ledger",
        ));
        out.push(row("files", &["files"], Does::Open(Overlay::Files), ""));
    }
    for (word, ops, overlay) in WORLD {
        out.push(row(word, ops, Does::Open(overlay), ""));
    }
    if gates.workspace {
        for (word, ops, overlay) in AIMED {
            out.push(row(word, ops, Does::Open(overlay), ""));
        }
    }
    if gates.workspace {
        for (word, ops, slot, own) in KNOBS {
            out.push(row(word, ops, Does::Knob(slot), own));
        }
    }
    out.push(row("help", &["help"], Does::Open(Overlay::Help), ""));
    out.push(wanting(
        row("find", &["search"], Does::Find, ""),
        "a needle",
    ));
    if let Some(offers) = gates.conversation {
        acts(offers, &mut out);
    }
    // **The transcript's policy is the settings pane's** (§23 item 11):
    // what was `/full` and `/steps` is two of its knobs.
    out.push(row(
        "settings",
        &[],
        Does::Open(Overlay::Settings),
        "this device's own knobs",
    ));
    out
}

/// The surfaces that name no workspace, in the roster's order.
const WORLD: [(&str, &[&str], Overlay); 4] = [
    ("attention", &["attention"], Overlay::Queue),
    ("trail", &["ops"], Overlay::Trail),
    ("balls all", &["balls"], Overlay::Balls(View::Everywhere)),
    ("board", &["board"], Overlay::Balls(View::Board)),
];

/// The surfaces that name the focused workspace — the aimed bar's seven, each
/// under the word its chip carried and the tags it fired.
const AIMED: [(&str, &[&str], Overlay); 7] = [
    ("balls", &["workspace-balls"], Overlay::Balls(View::Here)),
    ("science", &["science"], Overlay::Candidates),
    ("diff", &["work-diff"], Overlay::Work),
    ("fleet", &["fleet"], Overlay::Fleet),
    ("clients", &["clients"], Overlay::Clients),
    ("config", &["config", "proposals"], Overlay::Admin),
    ("login", &["login"], Overlay::SignIn),
];

/// The rule's knobs, by verb (§22 items 2 and 10): the model's list, the
/// effort's, and the priority toggle, under the tags their controls carry.
const KNOBS: [(&str, &[&str], Slot, &str); 3] = [
    (
        "model",
        &["models", "model"],
        Slot::Model,
        "the worker's model",
    ),
    (
        "effort",
        &["effort"],
        Slot::Effort,
        "how hard the worker thinks",
    ),
    (
        "priority",
        &["priority"],
        Slot::Priority,
        "the worker's priority, on or off",
    ),
];

/// The acts on the focused conversation, under the engine's own gates: the
/// stops while something runs, nudge while it rests, the row acts always.
fn acts(offers: Offers, out: &mut Vec<Row>) {
    if offers.stoppable {
        out.push(row("stop", &["stop"], Does::Stop { children: false }, ""));
        out.push(wanting(
            row("interrupt", &["interrupt"], Does::Interrupt, ""),
            "what to say instead",
        ));
    }
    if offers.stop_children {
        out.push(row(
            "stop all",
            &["stop"],
            Does::Stop { children: true },
            "stop this turn and every turn under it",
        ));
    }
    if offers.at_rest {
        out.push(row("nudge", &["nudge"], Does::Nudge, ""));
    }
    out.push(row("retarget", &["retarget"], Does::Retarget, ""));
    out.push(wanting(row("flag", &["flag"], Does::Flag, ""), "a reason"));
    out.push(row("revoke", &["revoke"], Does::Revoke, ""));
    out.push(row("restore", &["restore"], Does::Restore, ""));
}
