//! **The agent worktree, read-only** (yog §11 Altitude-2 Files; DESIGN
//! §13.15): the bounded sorted listing of what a conversation's worktree
//! holds, and — when the ask named one of its listed paths — that file's
//! bounded preview.
//!
//! **A listing and one entry's bytes are one question asked at two depths**,
//! which is upstream's own wording for why `path` is a parameter of this read
//! rather than a second op. Every answer carries the listing, so a preview ask
//! replaces the whole value rather than merging into one: nothing here has to
//! pair a late preview with a listing it was not read beside.
//!
//! **`worktree` is the discriminant and it is read as exactly that.** A
//! disposable worktree that is gone is a FACT, not an empty listing (yog
//! §3.5), so the encoder states `rows` only when there is a worktree to list
//! and a reader never has to tell *torn down* from *nothing in it*.
//!
//! **[`Preview`] is spelled here and read by two answers.** The work diff's
//! `patch` is the same bounded-file object (`codec::workdiff`), so it is read
//! by this decoder rather than by a second one — one shape with two readers is
//! the drift this codec exists to prevent (DESIGN §13.14).

use serde_json::{Map, Value};

use super::fields::{arr_of, bool_of, opt, opt_val, str_of, u64_of};

/// **A bounded file, in the three classes every seat renders.** The bytes are
/// carried and never parsed: what a preview IS is the engine's reading of a
/// file it bounded, and a client that re-derived a class from the text would
/// be a second authority for one fact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Preview {
    /// The whole file, small enough to be handed over.
    Text(String),
    /// As much of it as the bound allowed, with what the whole would have
    /// been — so a reader can say how much is missing without asking again.
    Truncated { text: String, size: u64 },
    /// Bytes nothing could read as text. There is no text field at all: an
    /// empty string would read as *an empty file*.
    Binary { size: u64 },
}

/// One walked entry: its path relative to the worktree, its size, and whether
/// it is a directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileRow {
    pub path: String,
    pub size: u64,
    pub dir: bool,
}

/// One `files` answer, as the engine spelled it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Listing {
    /// Whether there is a worktree at all. `false` is the whole answer: the
    /// rows and the truncation mark are absent rather than empty.
    pub worktree: bool,
    pub rows: Vec<FileRow>,
    pub truncated: bool,
    /// **Where this conversation's work actually lands**, when that is not
    /// the worktree the listing walked (yog bl-1015). Empty is the ordinary
    /// case — the listing IS the working directory — so a reader never has to
    /// compare it against the rows it came with.
    pub working_dir: String,
    /// The asked-for file's bytes, when the ask named one the listing carried.
    pub preview: Option<Preview>,
}

/// **What the files screen is holding**, and the conversation it was read
/// for. `files` names a conversation, so a listing under another one is the
/// wrong claim — the §14 pairing law [`crate::codec::Records`] keeps, one
/// surface along.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Files {
    pub workspace: String,
    pub agent: String,
    pub listing: Listing,
    /// **The path this answer was asked at**, empty for the bare listing. The
    /// answer does not echo it — there is no path in a `files` reply — so it
    /// is named from the ask, which is the one place it is known, and the
    /// paint puts the preview under exactly that row.
    pub opened: String,
}

impl Files {
    /// Whether this listing is about the conversation now focused.
    #[must_use]
    pub fn about(&self, workspace: &str, agent: &str) -> bool {
        self.workspace == workspace && self.agent == agent
    }
}

/// Read the `files` answer.
pub(super) fn listing(o: &Map<String, Value>) -> Result<Listing, String> {
    let worktree = bool_of(o, "worktree")?;
    let (rows, truncated) = if worktree {
        (
            arr_of(o, "rows")?
                .iter()
                .map(entry)
                .collect::<Result<Vec<FileRow>, String>>()?,
            bool_of(o, "truncated")?,
        )
    } else {
        (Vec::new(), false)
    };
    Ok(Listing {
        worktree,
        rows,
        truncated,
        working_dir: opt(o, "working_dir", str_of)?.unwrap_or_default(),
        preview: opt_val(o, "preview", preview)?,
    })
}

/// One walked entry.
fn entry(v: &Value) -> Result<FileRow, String> {
    let o = v.as_object().ok_or("files: a row is not an object")?;
    Ok(FileRow {
        path: str_of(o, "path")?,
        size: u64_of(o, "size")?,
        dir: bool_of(o, "dir")?,
    })
}

/// A bounded file, in the engine's own three classes. An unknown class
/// refuses naming it: a preview whose class this codec cannot read is bytes
/// it would have to guess at, and a guess is worse than none.
pub(super) fn preview(v: &Value) -> Result<Preview, String> {
    let o = v.as_object().ok_or("preview: not an object")?;
    match str_of(o, "kind")?.as_str() {
        "text" => Ok(Preview::Text(str_of(o, "text")?)),
        "truncated" => Ok(Preview::Truncated {
            text: str_of(o, "text")?,
            size: u64_of(o, "size")?,
        }),
        "binary" => Ok(Preview::Binary {
            size: u64_of(o, "size")?,
        }),
        other => Err(format!("preview: unknown kind {other:?}")),
    }
}

#[cfg(test)]
mod tests;
