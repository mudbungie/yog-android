//! **The roster, read out of the vendored corpus** — the one authority for
//! which ops exist and which of them a seat owes a control (PARITY §2).
//!
//! Nothing is listed here. The rows are the engine's own help table, published
//! through `corpus/reply/help.json` and vendored beside every other fixture,
//! so a verb that lands upstream arrives in this repo as a re-vendor and
//! reddens the gate until a control or a cited exemption answers it.
//!
//! **A surface value this file does not know is refused, not skipped.** Two
//! values are defined today; a third would be a classification decision made
//! upstream that this client must not silently read as "not my problem".

use std::collections::BTreeSet;

use serde_json::Value;

/// Every op the corpus carries, and the subset a seat owes a control.
#[derive(Debug)]
pub(super) struct Roster {
    /// Each row's `verb`. What an `act:` tag must name.
    pub(super) every: BTreeSet<String>,
    /// The rows classed `control`. What must be reachable or exempt.
    pub(super) control: BTreeSet<String>,
}

/// The classification's two values (yog `docs/PARITY.md` §2).
const CONTROL: &str = "control";
const MACHINE: &str = "machine";

/// Read `reply/help.json`. Strict in the same way the codec is: a missing
/// field or an unknown classification is an error naming what it found, never
/// a row quietly dropped.
pub(super) fn read(help: &str) -> Result<Roster, String> {
    let value: Value =
        serde_json::from_str(help).map_err(|why| format!("reply/help.json is not JSON: {why}"))?;
    let rows = value
        .get("frames")
        .and_then(|frames| frames.get(0))
        .and_then(|frame| frame.get("rows"))
        .and_then(Value::as_array)
        .ok_or_else(|| "reply/help.json carries no frames[0].rows array".to_owned())?;
    let mut roster = Roster {
        every: BTreeSet::new(),
        control: BTreeSet::new(),
    };
    for row in rows {
        let verb = row
            .get("verb")
            .and_then(Value::as_str)
            .ok_or_else(|| "a help row states no verb".to_owned())?;
        let surface = row
            .get("surface")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("{verb}: a help row states no surface — re-vendor the corpus from a yog at protocol 7 or later"))?;
        match surface {
            CONTROL => {
                roster.control.insert(verb.to_owned());
            }
            MACHINE => {}
            other => {
                return Err(format!(
                    "{verb}: surface `{other}` is neither `{CONTROL}` nor `{MACHINE}` — \
                     a third class landed upstream and this gate must decide what it owes"
                ));
            }
        }
        roster.every.insert(verb.to_owned());
    }
    Ok(roster)
}
