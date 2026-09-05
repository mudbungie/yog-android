//! **The settled-failure notice as a row** (REMOTE §9.16, bl-8e3c): the
//! engine's third virtual trailing entry, painted where the tail would have
//! been — *this conversation is not coming back*, and the one remedy the
//! wound names when it names one.
//!
//! **The spellings are the desktop's records pane, byte for byte** (DESIGN
//! §7's rule): `wound: <class>`, the adapter's own words after a dash where
//! the class left any, and *a sign-in is wanted on <row>* for the refusal
//! arm — the affordance and the badge read the same word, so a step cannot
//! offer a sign-in while its badge denies there was a refusal.
//!
//! **A wound of class `none` is no row.** The engine's word for *nothing is
//! wounded* is a fact nobody recorded, and painting a badge that says so
//! would stand in for one.

use super::build::{key, row};
use super::{Row, RowClass, Tone};

/// The engine's own spelling for a wound that is not there.
const NONE: &str = "none";
/// The wound class that is a provider refusal — the arm with a remedy.
const REFUSED: &str = "refused";
/// What the row says when the class left no words of its own.
const NOT_COMING_BACK: &str = "this conversation is not coming back";

pub(super) fn wounded_row(
    name: &str,
    wound: &str,
    reason: Option<&str>,
    auth_row: Option<&str>,
) -> Option<Row> {
    if wound == NONE {
        return None;
    }
    let payload = match (wound, reason, auth_row) {
        (REFUSED, _, Some(provider)) => format!("a sign-in is wanted on {provider}"),
        (REFUSED, _, None) => "a sign-in is wanted".to_owned(),
        (_, Some(reason), _) => reason.to_owned(),
        (_, None, _) => NOT_COMING_BACK.to_owned(),
    };
    Some(row(
        key(name, 0),
        format!("wound: {wound}"),
        &payload,
        RowClass::Other,
        Tone::Bad,
        None,
    ))
}
