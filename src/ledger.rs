//! **The vendored corpus's edition ledger** (yog REMOTE §3.2, bl-e598): what
//! this build can spell, what the engine it dialled can spell, and the one
//! question a control asks before it greys itself.
//!
//! **The wire has two numbers now and they answer different questions.**
//! `PROTOCOL` is a MAJOR: it moves only on a breaking change, the §3 preface is
//! strict equality on it, and a mismatch is fail-closed with no negotiation
//! (`crate::hello`). The EDITION is the additive line inside one major —
//! a field, a word, an op or a reply kind ships with no bump and is stamped
//! with the edition it appeared at. So a newer engine of the same major is
//! something this build must go on talking to, which is the whole of why
//! `crate::codec::fields` reads grows-only.
//!
//! **Three facts, all computed from `corpus/shapes.json` by `build.rs`**, so
//! there is no second copy of any of them to hold equal to the first:
//! [`FLOOR`] (the edition the major was cut at), [`EDITION`] (the newest stamp
//! in the vendored corpus — what this build states in its preface) and
//! [`STAMPS`] (the post-floor paths only).
//!
//! **[`STAMPS`] is empty at a major's own cut**, and it is empty today. That is
//! not a mechanism waiting to be built: a path at or below the floor is on
//! every engine of this major, so it can never answer anything but *yes*, and
//! leaving it out of the table is what makes the table's presence mean
//! something. The first field an engine adds inside major 19 is the first row.
//!
//! **The engine's edition is carried, never stored.** It arrives on the §3
//! preface, `crate::hello::confirm` hands it back, and a held read keeps it
//! (`crate::transport::Open::edition`) — so the reading below takes it as an
//! argument. A process-wide `static` was the alternative and was refused: one
//! engine per process makes it *true*, and a mutable global makes every
//! reading of it depend on what else the process has dialled since, which is
//! exactly the property a pure question should not have. A peer that states no
//! edition is [`FLOOR`]: the oldest engine of this major, and therefore the
//! reading that greys the most and promises the least.

include!(concat!(env!("OUT_DIR"), "/ledger.rs"));

/// **Whether an engine could have said this field at all** — the question a
/// control asks before it renders an absent post-floor field as its default,
/// and before it offers a control whose gesture the engine could not act on.
///
/// The two answers are not the same sentence on a screen. A field the engine
/// CAN spell and did not send is the default — the fact before the field
/// existed. A field the engine cannot spell is *this engine cannot say*, and
/// showing the reassuring default there would be a client inventing world
/// state (DESIGN §8's rule).
#[must_use]
pub fn spells(shape: &str, path: &str, engine: u32) -> bool {
    spells_at(STAMPS, shape, path, engine)
}

/// The reading itself, over a STATED table — pure, so the rule is asserted
/// rather than the empty table it currently runs over.
pub(crate) fn spells_at(
    stamps: &[(&str, &str, u32)],
    shape: &str,
    path: &str,
    engine: u32,
) -> bool {
    stamps
        .iter()
        .find(|(said, key, _)| *said == shape && *key == path)
        .is_none_or(|(_, _, stamp)| *stamp <= engine)
}

#[cfg(test)]
mod tests;
