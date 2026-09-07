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

use super::build::{GEAR, key, row};
use super::{Row, RowClass, Tone};

pub(super) fn windowed_row(name: &str, tool: &str, input: &str, exit_code: Option<i64>) -> Row {
    let (prefix, tone) = match exit_code {
        None => (format!("{GEAR} {tool} — running"), Tone::InFlight),
        Some(code) => (format!("{GEAR} {tool} — exit {code}"), Tone::Plain),
    };
    row(key(name, 0), prefix, input, RowClass::Other, tone, None)
}
