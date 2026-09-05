//! **The config lineages a workspace holds** (REMOTE §9; DESIGN §13.14): the
//! named branches its policy is written on, each with the commit it stands at.
//!
//! **It is read where its names are spent.** `governing` says which lineage a
//! conversation FOLLOWS and how many diverged from it (`codec::records::spine`);
//! this is the list those names come out of, so it rides the records screen's
//! own read rather than a surface of its own — a list with no home is a screen
//! about a word.
//!
//! **The tip rides both ways, as it does everywhere upstream states one**: the
//! clipped form is what a line labels with, and the full oid is what a
//! `git show` outside yog takes. This seat paints the clipped one and reads
//! only that, which is `codec::records::spine`'s notch rule at a second site.

use serde_json::{Map, Value};

use super::fields::{arr_of, i64_of, str_of};
use super::records::words;

/// One lineage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lineage {
    pub name: String,
    /// The commit it stands at, clipped by the engine.
    pub short_oid: String,
    /// When that commit landed, as the engine's own epoch second. Carried
    /// rather than rendered: this seat has one ladder for *how long ago*
    /// (`crate::roster::ago`) and no second opinion about it.
    pub committed: i64,
    /// The files the lineage carries.
    pub files: Vec<String>,
}

/// Read the `lineages` answer's rows.
pub(super) fn rows(o: &Map<String, Value>) -> Result<Vec<Lineage>, String> {
    arr_of(o, "rows")?.iter().map(row).collect()
}

/// One row.
fn row(v: &Value) -> Result<Lineage, String> {
    let o = v
        .as_object()
        .ok_or_else(|| "lineages: row is not an object".to_owned())?;
    Ok(Lineage {
        name: str_of(o, "name")?,
        short_oid: str_of(o, "short_oid")?,
        committed: i64_of(o, "committed")?,
        files: words(o, "files")?,
    })
}

#[cfg(test)]
mod tests;
