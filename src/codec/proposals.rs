//! **The learning loop's veto** (yog REMOTE §9.22, bl-dd88): what a reviewer
//! agent has staged as a config patch, and the one word that takes it or
//! throws it away.
//!
//! litany's reviewer stages what it learned on a branch of its own and the
//! whole loop turns on a person reading it and saying yes or no. Nothing on
//! this wire could: `lineages` enumerates the config heads and `governing`
//! answers the commit a conversation resolves, never a candidate — so the veto
//! lived at a command on the engine's own box, which is an `ssh` and a
//! container `exec` on a server install and is exactly the thing this boundary
//! exists to make unnecessary.
//!
//! **New shapes, so `PROTOCOL` did not move for them** (§3): a verb a peer has
//! not heard of already refuses in band by name. What a client owes them is a
//! re-vendor to gain them, which is what bl-5070 is.
//!
//! **One op at two depths**, the `files` shape a third time: a bare frame is
//! the listing, and one naming an `id` answers that proposal WHOLE beside the
//! listing. A seat that names an id has already been answered the row it
//! names, so a second op would be a second derivation of one subject.
//!
//! **`fresh` crosses even though an empty `lineages` would imply it.** REMOTE
//! §9.22: that inference is a rule, and the wire states a derivation so a seat
//! need not own one — only the engine can be sure the two were read in one
//! pass. This seat therefore reads the word and infers nothing, which is
//! §9.4's rule at the site where getting it wrong means accepting a patch
//! written against a config that no longer governs anything.

use serde_json::{Map, Value};

use super::fields::{arr_of, bool_of, opt, str_of, strings_of};

/// One staged patch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Proposal {
    /// The proposal's own id — the handle the settle names, and the only
    /// address it has.
    pub id: String,
    /// The lineages this patch would move. Empty is a patch whose lineage has
    /// advanced under it; `fresh` is the engine's own word for that and is
    /// what this seat reads.
    pub lineages: Vec<String>,
    /// The commit the reviewer read when it wrote the patch.
    pub parent: String,
    /// Whether the lineage still stands where the reviewer read it.
    pub fresh: bool,
    /// git's own summary line for the change — carried verbatim, because
    /// counting the lines a second time here would be a second answer to the
    /// engine's own question.
    pub diffstat: String,
    /// What the reviewer called it.
    pub subject: String,
}

/// The `proposals` answer: the listing, and — when the ask named one — that
/// proposal whole beside it.
///
/// **`whole` is absent and never null** for the bare listing (REMOTE §9.22),
/// so `None` here is *nobody asked for one* rather than *there was nothing to
/// show*, and a screen can tell a listing from a reading without a second
/// flag.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Staged {
    /// The workspace the listing was read for. The reply echoes none, so the
    /// ask names it — `codec::admin`'s rule at a fourth site, and it is what
    /// makes a listing unpaintable under another workspace's name.
    pub workspace: String,
    pub rows: Vec<Proposal>,
    pub whole: Option<String>,
}

impl Staged {
    /// Whether this listing is about the workspace now focused — the pairing
    /// law every aimed answer in this seat keeps.
    #[must_use]
    pub fn about(&self, workspace: &str) -> bool {
        self.workspace == workspace
    }
}

/// **The settle's one word.** Accept and reject are two acts and not two
/// values of one, which is why the wire spells a word rather than a boolean —
/// and it is read back strictly, the closed-vocabulary discipline `effort`
/// already takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// Fast-forward the patch's lineage onto it and delete the staging branch.
    Accept,
    /// Delete the staging branch and nothing else.
    Reject,
}

impl Verdict {
    /// Both, in the order a thumb meets them: the taking first, because a
    /// listing an operator opened is one they mean to read and take.
    pub(crate) const ALL: [Self; 2] = [Self::Accept, Self::Reject];

    /// The engine's own word — the wire token and the control's label at once,
    /// so a control cannot say one thing and send another.
    pub(crate) fn word(self) -> &'static str {
        match self {
            Self::Accept => "accept",
            Self::Reject => "reject",
        }
    }
}

/// Read the verdict a frame states, strictly — found among the two that own
/// the words, so the tokens have one home and an unknown one refuses by name
/// (`codec::hold::verdict_of`'s shape, one family along).
pub(crate) fn verdict(o: &Map<String, Value>) -> Result<Verdict, String> {
    let word = str_of(o, "verdict")?;
    Verdict::ALL
        .into_iter()
        .find(|v| v.word() == word)
        .ok_or_else(|| format!("proposal: unknown verdict {word:?}"))
}

/// Read the `proposals` answer's rows and its optional whole reading.
pub(super) fn staged(o: &Map<String, Value>) -> Result<(Vec<Proposal>, Option<String>), String> {
    let rows = arr_of(o, "rows")?
        .iter()
        .map(row)
        .collect::<Result<Vec<Proposal>, String>>()?;
    Ok((rows, opt(o, "whole", str_of)?))
}

/// One row.
fn row(v: &Value) -> Result<Proposal, String> {
    let o = v
        .as_object()
        .ok_or_else(|| "proposals: row is not an object".to_owned())?;
    Ok(Proposal {
        id: str_of(o, "id")?,
        lineages: strings_of(o, "lineages", "proposals")?,
        parent: str_of(o, "parent")?,
        fresh: bool_of(o, "fresh")?,
        diffstat: str_of(o, "diffstat")?,
        subject: str_of(o, "subject")?,
    })
}

#[cfg(test)]
mod tests;
