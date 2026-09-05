//! **The op table, vendored** (DESIGN §13.14): what every gesture the engine
//! speaks is called, which surface owes it a control, and what it does — read
//! out of `corpus/reply/help.json`, which this repository already carries.
//!
//! **This seat answers `help` from the table rather than from the wire, and
//! that is a reading rather than a shortcut.** DESIGN §2 rules that the corpus
//! and the spoken version move together: *"a protocol bump upstream is a
//! re-vendor and a rebuild here"*, and a peer of another version is refused
//! fail-closed at the §3 preface. So for any engine this build can talk to,
//! the vendored table IS that engine's table — asking for it would be a radio
//! spend for an answer already compiled in, on the device §14.1 exists to keep
//! asleep. `tests/conformance/requests.rs` records the wire shape as one this
//! codec never sends.
//!
//! **One reader, two consumers.** `crate::parity` judges this app against the
//! `surface` column and the help screen paints the other three; two readers of
//! one file would drift, so the parity roster is a fold over these rows rather
//! than a second parse.

use serde_json::Value;

/// The table itself, compiled in. It is the same bytes `tests/parity.rs`
/// judges the walk against — one file, one authority.
pub const TABLE: &str = include_str!("../corpus/reply/help.json");

/// One op, as the engine describes it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    /// The op token — the name that already exists everywhere: the envelope's
    /// `op`, the corpus filename, and the `act:` tag a control carries.
    pub verb: String,
    /// `control` (a seat owes it a discoverable interactable) or `machine`
    /// (spoken only by programs). Carried as the engine's own word: a third
    /// class is a classification decision made upstream, and what this app
    /// OWES for one is `crate::parity`'s question, not this reader's.
    pub surface: String,
    /// One line about what it does.
    pub summary: String,
    /// How it is said as a slash line.
    pub usage: String,
    /// The paragraph beneath, in the engine's own words.
    pub detail: String,
}

/// Read the table. Strict in the same way the codec is: a missing field is an
/// error naming what it found, never a row quietly dropped.
pub fn rows(help: &str) -> Result<Vec<Row>, String> {
    let value: Value =
        serde_json::from_str(help).map_err(|why| format!("reply/help.json is not JSON: {why}"))?;
    let rows = value
        .get("frames")
        .and_then(|frames| frames.get(0))
        .and_then(|frame| frame.get("rows"))
        .and_then(Value::as_array)
        .ok_or_else(|| "reply/help.json carries no frames[0].rows array".to_owned())?;
    rows.iter().map(row).collect()
}

/// One row, with the two load-bearing columns required and the prose optional
/// — an op that says nothing about itself is a thin row, not a broken table.
fn row(v: &Value) -> Result<Row, String> {
    let verb = said(v, "verb").ok_or_else(|| "a help row states no verb".to_owned())?;
    let surface = said(v, "surface").ok_or_else(|| {
        format!(
            "{verb}: a help row states no surface — re-vendor the corpus from a yog at \
             protocol 7 or later"
        )
    })?;
    Ok(Row {
        verb,
        surface,
        summary: said(v, "summary").unwrap_or_default(),
        usage: said(v, "usage").unwrap_or_default(),
        detail: said(v, "detail").unwrap_or_default(),
    })
}

/// A string column the row may not carry.
fn said(v: &Value, key: &str) -> Option<String> {
    v.get(key).and_then(Value::as_str).map(str::to_owned)
}

#[cfg(test)]
mod tests;
