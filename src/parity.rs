//! **The interface-parity gate, client half** (yog `docs/PARITY.md`, bl-fe4c).
//!
//! The operator's requirement is that the desktop seat and this client have
//! interaction parity — *not* identical placement, but if something is
//! interactable in one it must exist in the other, and drift between them is
//! caught mechanically rather than noticed by hand. PARITY §2 rules out a
//! client-vs-client diff (components meet at the interface, never pairwise)
//! and judges each client against **one roster**: the ops the engine's own
//! help table classes `control`, published inside the corpus every client
//! already vendors.
//!
//! So this module answers one question — *which ops can be reached from this
//! app?* — out of three strings and nothing else:
//!
//! - the **roster**, `corpus/reply/help.json`, whose rows carry `surface`
//!   since protocol 7 (yog bl-8758): `control` is an op every seat owes a
//!   discoverable interactable, `machine` one spoken only by programs.
//! - the **inventory**, the `act:<op>` tags read out of the accessibility
//!   dumps the `make screens` walk captures (DESIGN §15). Observed, never
//!   self-reported: what a control claims to fire is not evidence that a
//!   thumb can reach it.
//! - the **exemptions**, `parity.toml` — one line per deliberately absent op,
//!   each citing a ball or a doc section.
//!
//! **It judges strings, so it is host-testable in full.** Nothing here reads
//! a file, spawns a process or knows what a screen is; the driver that feeds
//! it lives in `tests/parity.rs` and runs after a walk, because the inventory
//! does not exist until a device has been driven. The four assertions are
//! PARITY §5's, verbatim in effect:
//!
//! ```text
//! roster − exemptions ⊆ inventory      (coverage)
//! tags(inventory) ⊆ ops(corpus)        (no unknown tag)
//! ∀e ∈ exemptions: e ∈ roster          (no rotted exemption)
//! ∀e ∈ exemptions: e ∉ inventory       (no stale exemption)
//! ```
//!
//! The last two are what keep the exemption file from becoming a place to
//! hide: an exemption for an op that is now surfaced fails, and so does one
//! naming an op the roster no longer carries.

use std::collections::BTreeSet;
use std::fmt::Write as _;

mod exempt;
mod roster;
mod tags;

#[cfg(test)]
mod tests;

/// **The tag a control carries**, and the one home of the `act:` namespace
/// (PARITY §4). The op token is the name that already exists everywhere — the
/// help row's `verb`, the envelope's `op`, the corpus filename — so no
/// translation table is born, and the visible label stays a human word.
///
/// The shell spends this at every verb-firing control; `tags::found` reads it
/// back out of a dump. Two callers, one spelling.
pub(crate) fn tag(op: &str) -> String {
    format!("{PREFIX}{op}")
}

/// The reserved prefix. Judged in both directions: a `control` op with no tag
/// fails, and a tag naming no corpus op fails.
const PREFIX: &str = "act:";

/// **The fallback inventory's file** (PARITY §6, DESIGN §15.1): the tags a
/// launch painted, one per line, as the shell writes them and the harness
/// pulls them. It is the same text `tags::found` reads back out of an
/// accessibility dump — the scanner looks for the token, not for a format —
/// so the day the platform tree works again, this renderer goes and nothing
/// else does.
///
/// `pub` rather than `pub(crate)` because the shell that calls it compiles on
/// Android alone: a crate-private renderer would be dead code in every host
/// build, and the honest answer to that is a surface entry, not a `#[cfg]`
/// that would take the function out of the 100% floor with it.
pub fn inventory(ops: &BTreeSet<String>) -> String {
    let mut out = String::new();
    for op in ops {
        let _ = writeln!(out, "{}", tag(op));
    }
    out
}

/// What one judgement says: the report a run always prints, and the failures
/// that redden it. Owned and concrete — the driver prints one and asserts the
/// other is empty.
pub struct Judgement {
    /// The roster, the inventory and every exemption with its citation.
    /// Printed on **every** run, passing or failing: an absence is never
    /// silent (PARITY §7).
    pub report: String,
    /// One line per broken assertion, naming the op and what to do about it.
    pub failures: Vec<String>,
}

impl Judgement {
    /// A judgement that could not be made: the roster or the exemption file
    /// would not parse. It is a failure and not a panic, so the driver prints
    /// the same report shape whatever went wrong.
    fn refused(why: String) -> Self {
        Self {
            report: String::new(),
            failures: vec![why],
        }
    }
}

/// Judge one walk. `help` is the vendored `reply/help.json`, `exemptions` the
/// `parity.toml` text, and `inventory` every dump the walk captured,
/// concatenated — the tags are scanned out of it, so how many files it came
/// from is the driver's business and not this function's.
pub fn judge(help: &str, exemptions: &str, inventory: &str) -> Judgement {
    let roster = match roster::read(help) {
        Ok(roster) => roster,
        Err(why) => return Judgement::refused(why),
    };
    let exemptions = match exempt::read(exemptions) {
        Ok(rows) => rows,
        Err(why) => return Judgement::refused(why),
    };
    let found = tags::found(inventory);
    let excused: BTreeSet<String> = exemptions.iter().map(|row| row.op.clone()).collect();
    let mut failures = Vec::new();

    // Coverage: every control-classed op is reachable, or cited as absent.
    for op in roster.control.difference(&excused) {
        if !found.contains(op) {
            failures.push(format!(
                "{op}: classed `control` by the engine, no {} tag in the walked \
                 inventory, and no line in parity.toml — surface it, extend the \
                 walk to the screen that already does, or record the absence \
                 with its citation",
                tag(op)
            ));
        }
    }
    // No unknown tag: a typo'd or stale tag fails exactly as a leak fixture
    // that stops matching fails.
    for op in found.difference(&roster.every) {
        failures.push(format!(
            "{}: a tag naming no op in the corpus — a typo, or a verb that \
             left the roster",
            tag(op)
        ));
    }
    for row in &exemptions {
        // Rotted: the op is not (or no longer) one a seat owes a control.
        if !roster.control.contains(&row.op) {
            failures.push(format!(
                "{}: exempted, but the roster does not class it `control` — \
                 drop the line",
                row.op
            ));
        }
        // Stale: it is surfaced, so the exemption is a lie about this tree.
        if found.contains(&row.op) {
            failures.push(format!(
                "{}: exempted, but {} is in the walked inventory — drop the \
                 line, the control exists",
                row.op,
                tag(&row.op)
            ));
        }
    }
    Judgement {
        report: report(&roster, &found, &exemptions),
        failures,
    }
}

/// The standing report. Three counts and then the exemption roster in full,
/// because a ledger nobody reads is prose again.
fn report(roster: &roster::Roster, found: &BTreeSet<String>, exemptions: &[exempt::Row]) -> String {
    let mut out = format!(
        "parity: {} ops in the corpus, {} classed `control`\n\
         parity: {} reached by a tagged control: {}\n\
         parity: {} exempt, each with its citation:\n",
        roster.every.len(),
        roster.control.len(),
        found.len(),
        found.iter().cloned().collect::<Vec<_>>().join(" "),
        exemptions.len(),
    );
    for row in exemptions {
        // Writing into a String cannot fail; the Result is discarded rather
        // than unwrapped (AGENTS.md rule 4 — no panic paths outside tests).
        let _ = writeln!(out, "    {:<18} {}", row.op, row.reason);
    }
    out
}
