//! **The undelivered mail** (`inbox`): the deposits sitting in a
//! conversation's inbox that nothing has taken yet.
//!
//! **The parse is what a phone paints, and the raw bytes are not read.** Each
//! row carries the file verbatim beside the envelope the engine parsed out of
//! it; the two are one deposit said twice, and a seat with one column shows
//! the words rather than the frontmatter around them.

use serde_json::{Map, Value};

use super::super::fields::{arr_of, opt, str_of};
use super::agent::object;

/// One undelivered deposit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mail {
    /// The deposit file's own name.
    pub name: String,
    /// Who deposited it, when, and how it ended — each absent rather than
    /// empty upstream, because a forgiving parse of a hand-edited file says
    /// *this was not stated* and an empty `from:` is a different claim.
    pub from: Option<String>,
    pub deposited_at: Option<String>,
    pub epitaph: Option<String>,
    pub body: String,
}

/// Read the `inbox` answer's rows.
pub(in super::super) fn mail(o: &Map<String, Value>) -> Result<Vec<Mail>, String> {
    arr_of(o, "rows")?.iter().map(row).collect()
}

/// One row.
fn row(v: &Value) -> Result<Mail, String> {
    let o = object(v, "inbox")?;
    let deposit = object(
        o.get("deposit").ok_or("inbox: a row states no deposit")?,
        "deposit",
    )?;
    Ok(Mail {
        name: str_of(&o, "name")?,
        from: opt(&deposit, "from", str_of)?,
        deposited_at: opt(&deposit, "deposited_at", str_of)?,
        epitaph: opt(&deposit, "epitaph", str_of)?,
        body: str_of(&deposit, "body")?,
    })
}
