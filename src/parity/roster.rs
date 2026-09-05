//! **The roster, read out of the vendored corpus** — the one authority for
//! which ops exist and which of them a seat owes a control (PARITY §2).
//!
//! Nothing is listed here. The rows are the engine's own help table, published
//! through `corpus/reply/help.json` and vendored beside every other fixture,
//! so a verb that lands upstream arrives in this repo as a re-vendor and
//! reddens the gate until a control or a cited exemption answers it.
//!
use std::collections::BTreeSet;

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

/// Read the roster out of the vendored table. **One reader, folded** (bl-3685):
/// `crate::help` parses the file — the help screen paints the other three
/// columns out of the same rows — and this is the classification laid over it.
/// Two parses of one file would be two places for a column to move under.
///
/// **A surface value this file does not know is refused, not skipped.** Two
/// values are defined today; a third would be a classification decision made
/// upstream that this client must not silently read as "not my problem".
pub(super) fn read(help: &str) -> Result<Roster, String> {
    let mut roster = Roster {
        every: BTreeSet::new(),
        control: BTreeSet::new(),
    };
    for row in crate::help::rows(help)? {
        match row.surface.as_str() {
            CONTROL => {
                roster.control.insert(row.verb.clone());
            }
            MACHINE => {}
            other => {
                return Err(format!(
                    "{}: surface `{other}` is neither `{CONTROL}` nor `{MACHINE}` — \
                     a third class landed upstream and this gate must decide what it owes",
                    row.verb
                ));
            }
        }
        roster.every.insert(row.verb);
    }
    Ok(roster)
}
