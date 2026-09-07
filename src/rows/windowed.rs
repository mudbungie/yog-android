//! **The follow window's call as a row** (REMOTE §5.5, PROTOCOL 15): what is
//! running on the machine an agent is administering, painted while the record
//! is still catching up.
//!
//! It sits beside [`wounded`](super::wounded) and [`compacted`](super::compacted)
//! for their reason: one entry kind whose spelling is its own, lifted out of
//! the exhaustive match so a new kind touches the match and nothing else.
//!
//! **The label is the committed block's, word for word.** A tool call the
//! record already carries reads `⚙ <tool> — running` until a result retires
//! it ([`super::project::blocks`]), and a call the window is ahead of reads
//! the same — one call reads one way on either side of the commit, which is
//! §13.3's vocabulary rule applied to the two sides of one cadence.
//!
//! **A closed call states the number and claims nothing about it.** REMOTE
//! §5.5 puts no verdict on this lane: `exit_code`'s presence is the status,
//! and what its value MEANS is the reading yog states itself where it wants a
//! seat to have one — `failed`, `exit_label` and `standing` on a trail row
//! (§9.17). So the row goes plain rather than green or red, and this seat does
//! not become the second implementation that ruling exists to prevent.
//!
//! **A HELD call is the exception, and it is not a verdict** (PROTOCOL 18, yog
//! bl-58bb). `held` is not a reading this seat takes off a number — it is the
//! control stating that this call is parked and the operator is what it is
//! waiting for. That is `docs/STYLE.md` §3's `Attention` word for word (*"a
//! parked call held for an answer"*), so the row wears the attention accent,
//! which is the one hue on this glass that means *asking for you*. It is the
//! only tone here the wire never spells (`codec::Tone::Held`).
//!
//! **The control's sentence is the row's payload, and the input follows it.**
//! REMOTE §8.1: the sentence crosses unrewritten, because rewriting it *"would
//! put a different call in front of the operator"*. It goes FIRST because it
//! is what decides an answer; the command it is about opens under the fold,
//! one tap away, which is the preview/body split doing what it is for. The
//! band under the composer answers the call (§13.7); this row is the
//! transcript's account of it, in the place the operator is already reading.

use super::build::{GEAR, key, row};
use super::{Row, RowClass, Tone};

pub(super) fn windowed_row(
    name: &str,
    tool: &str,
    input: &str,
    exit_code: Option<i64>,
    held: Option<&str>,
) -> Row {
    let (prefix, tone) = match (held, exit_code) {
        (Some(_), _) => (format!("{GEAR} {tool} — held for you"), Tone::Held),
        (None, None) => (format!("{GEAR} {tool} — running"), Tone::InFlight),
        (None, Some(code)) => (format!("{GEAR} {tool} — exit {code}"), Tone::Plain),
    };
    let payload = match held {
        Some(reason) if input.is_empty() => reason.to_owned(),
        Some(reason) => format!("{reason}\n{input}"),
        None => input.to_owned(),
    };
    row(key(name, 0), prefix, &payload, RowClass::Other, tone, None)
}
