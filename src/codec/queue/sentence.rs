//! **The control's sentence, read** (DESIGN §13.7, bl-8c94): the two parts a
//! surface shows of what the capability boundary wrote about a parked call.
//!
//! **The sentence has a shape, and it is the engine's own.** yog's control
//! writes `<tool> <input, clipped to 160 chars> classified <class> (<the
//! evidence>)` in one place (`control::reason::reason`), and reads its own
//! clause back out of it to learn which class a wide answer stands over
//! (`control::reason::class_of`). This seat reads it at the same seam and for
//! the same reason: everything before the clause is the CALL, everything from
//! it on is the DECISION, and a surface that must not print a tool's JSON
//! input needs to know which is which.
//!
//! **Nothing here rewrites a word.** REMOTE §8.1 is explicit that the sentence
//! crosses unrewritten because rewriting it *"would put a different call in
//! front of the operator"* — so this splits it and never edits it. The half it
//! leaves out is not lost either: the input is a field of its own on the row
//! that carries the call (`rows::held`), which is where an operator reads the
//! command rather than under the composer, and a surface reaching for a fold
//! opens onto the engine's own words.
//!
//! **A sentence with no clause of ours in it is the decision entire.** The
//! seat cannot tell a call from prose in a sentence it does not recognise, and
//! dropping the engine's words would be worse than showing a call's input
//! behind a fold — so the split simply does not happen, and the one line says
//! what is held and nothing about its class.

/// The clause the engine's control writes between the call it parked and the
/// class it landed in. Spelled with both its spaces: the class word follows it
/// and a bare `classified` at the head of a sentence is not this clause.
const CLAUSE: &str = " classified ";

/// What a held call is called on the one line — the STYLE.md §3 `Attention`
/// word, and the same word the transcript row wears.
const HELD: &str = "held";

/// The control's sentence in the two parts a surface shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Folded {
    /// The one line: what is held, and the class the control put it in. Never
    /// the call's input — that is the whole point of the split.
    pub line: String,
    /// The decision: the class clause and the evidence behind it, in the
    /// engine's own words. What a fold opens onto.
    pub why: String,
}

/// Read `reason` — the control's sentence about a call on `tool` — into the
/// line a band shows and the paragraph its fold opens onto.
///
/// `pub`, like [`super::held_at`] and for its reason: the paint that spends it
/// is android-only, so a `pub(crate)` would be dead code on a host build and
/// the assertion would go with it.
#[must_use]
pub fn folded(tool: &str, reason: &str) -> Folded {
    let split = reason.rfind(CLAUSE).and_then(|at| {
        let why = reason.get(at..)?.trim();
        let class = reason.get(at + CLAUSE.len()..)?.split_whitespace().next()?;
        Some((format!("{HELD}: {tool} · {class}"), why.to_owned()))
    });
    let (line, why) =
        split.unwrap_or_else(|| (format!("{HELD}: {tool}"), reason.trim().to_owned()));
    Folded { line, why }
}

#[cfg(test)]
mod tests;
