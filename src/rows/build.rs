//! **How a row is built** — the identity it wears, the constructor every
//! variant funnels through, and the preview/body split that decides what a
//! fold toggle has to reveal.
//!
//! Split from [`super::project`] on the seam that module's own doc names:
//! *what an entry becomes* is the exhaustive per-variant match up there; *what
//! a row is made of* is here. Nothing in this file knows the entry vocabulary
//! — it takes a prefix, a payload and a class — which is why the two change
//! for unrelated reasons.

use super::{Fold, Role, Row, RowClass, Tone};

/// Key namespace for a transcript row's fold override. A namespace and not a
/// bare filename so the set the caller keeps can hold other surfaces' keys
/// without a collision being possible.
const KEY_ROOT: &str = "tx";
/// Characters of payload a contracted row previews before the ellipsis.
const PREVIEW_CAP: usize = 160;

/// A row's stable identity: the entry's filename and the block ordinal.
pub(super) fn key(name: &str, block: usize) -> String {
    format!("{KEY_ROOT}/{name}#{block}")
}

/// Build a row, splitting `payload` into its one-line preview and the body
/// that folding reveals. `expanded` is filled in by [`super::rows`]; `role` is
/// who the row speaks for — `None` on machinery, where nobody is.
pub(super) fn row(
    key: String,
    prefix: String,
    payload: &str,
    class: RowClass,
    tone: Tone,
    role: Option<Role>,
) -> Row {
    let (preview, body) = split(payload);
    Row {
        key,
        prefix,
        preview,
        body,
        hover: String::new(),
        class,
        tone,
        role,
        fold: Fold::Payload,
        expanded: false,
    }
}

/// State in the prefix how big the fold is, in **characters**. Characters and
/// not bytes: a byte count lies about any payload carrying non-ASCII, and this
/// seat is read by a human sizing up a tap.
///
/// The count is the **body's** — what the toggle opens onto — which makes "a
/// row with nothing to fold says nothing" the same rule as the toggle's own
/// rather than a second one about small payloads (see [`split`]: the empty
/// body *is* the fact). It also means the plural never arises: a body exists
/// only where the payload is clipped or multi-line, so it is never one
/// character long.
///
/// Only the tool-result row wears it, because it is the row whose payload the
/// operator cannot guess: `✔ tool result — ok` says nothing about whether the
/// fold opens onto four characters or forty thousand. The live tail stays bare
/// on purpose — it is in-flight, so it is already expanded on screen, and how
/// much has landed is the in-flight strip's line to say, not a second
/// per-frame spelling of a growing number.
pub(super) fn with_size(mut row: Row) -> Row {
    if !row.body.is_empty() {
        let chars = row.body.chars().count();
        row.prefix = format!("{} · {chars} chars", row.prefix);
    }
    row
}

/// Split a payload into `(one-line preview, foldable body)`. The body is
/// **empty** when the payload is already one line short enough to show whole,
/// and otherwise the payload *entire* — first line included, because the fold
/// opens onto the thing itself and not onto its remainder.
fn split(payload: &str) -> (String, String) {
    let first = payload.lines().next().unwrap_or_default();
    let clipped = first.chars().count() > PREVIEW_CAP;
    let preview = if clipped {
        let head: String = first.chars().take(PREVIEW_CAP).collect();
        format!("{head}…")
    } else {
        first.to_string()
    };
    let more = clipped || payload.lines().nth(1).is_some();
    let body = if more {
        payload.to_string()
    } else {
        String::new()
    };
    (preview, body)
}
