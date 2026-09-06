//! **The admin family** (DESIGN §13.17): the config files a world's policy is
//! written in, the task branch a workspace is marked with, the inbox flush,
//! and the two deletions.
//!
//! **The destination is the parameter, and this seat spells three of five.**
//! `config` is ONE op that takes a `target`, and the two it does not spell
//! want pickers this app has not got: `litany-workflow` names a workflow, and
//! the read that lists them is the boundary's workflow verb (unbuilt here,
//! `parity.toml`), and `branch` names a lineage, an origin and a PATH inside a
//! config tree — three choices off a read nothing here makes. Those frames are
//! refused **by name** rather than read as one of the three, which is the
//! silent misread REMOTE §3's third rule forbids.
//!
//! **`settings` rides through unread, and that is a decision** (the codec's
//! grow-per-consumer rule spent inside a shape). The engine answers a config
//! read as a typed control per setting — a provider row, a text box, a list, a
//! bounded number — beside the file's own bytes. A form built out of them is a
//! surface this seat does not have; what it has is the composer (§13.2), which
//! edits the FILE. Painting both would be two renderings of one file, and the
//! one that can be written back is the text.
//!
//! **`marks` is a read and a write under one op token**, exactly as `config`
//! is: a frame with no `branch` asks what the workspace is marked with, and
//! one with a branch sets it. So the ask and the act are spelled apart here
//! and the token is the same word on the wire.

pub mod act;

use serde_json::{Map, Value, json};

use super::fields::str_of;

/// **Which config file a gesture is about.** Three of the engine's five
/// destinations: the two that name no place, and the one that names a
/// workspace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Destination {
    /// The LLM adapter's own table — providers, models, credentials. Per
    /// workspace, because a sign-in is (DESIGN §13.2).
    Brazen { workspace: String },
    /// litany's model table, which names no workspace.
    LitanyModels,
    /// yog's own clock: the watcher-cycle periods.
    Cadence,
}

impl Destination {
    /// The word this destination is shown and tapped by. It is the wire's own
    /// `file` token, so the label and the frame cannot drift.
    ///
    /// `pub(crate)` rather than `pub` for `RowAct::op`'s reason exactly: it
    /// hands back a borrow, and every caller is inside this crate (bootstrap
    /// rule 2's honest demotion).
    pub(crate) fn file(&self) -> &'static str {
        match self {
            Self::Brazen { .. } => "brazen",
            Self::LitanyModels => "litany-models",
            Self::Cadence => "cadence",
        }
    }
}

/// The `target` object of a config frame.
pub(crate) fn target(at: &Destination) -> Value {
    match at {
        Destination::Brazen { workspace } => {
            json!({ "file": "brazen", "workspace": workspace })
        }
        Destination::LitanyModels | Destination::Cadence => json!({ "file": at.file() }),
    }
}

/// The same object read back, refusing the two destinations this seat has no
/// picker for.
pub(crate) fn destination(v: Option<&Value>) -> Result<Destination, String> {
    let o = v
        .ok_or("config: missing field \"target\"")?
        .as_object()
        .ok_or("config: \"target\" is not an object")?
        .clone();
    match str_of(&o, "file")?.as_str() {
        "brazen" => Ok(Destination::Brazen {
            workspace: str_of(&o, "workspace")?,
        }),
        "litany-models" => Ok(Destination::LitanyModels),
        "cadence" => Ok(Destination::Cadence),
        other => Err(format!("config: unimplemented destination {other:?}")),
    }
}

/// **What a config read answers**, and the destination it was asked at. The
/// reply echoes no target — it is the same shape every destination earns — so
/// the ask names it, which is `codec::files`' rule at a third site.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub at: Destination,
    /// The file's own bytes. What the composer edits and what a write sends.
    pub text: String,
}

/// **What a workspace's task branch is**, as the engine re-read it after the
/// write — or as it stands. One shape for both, because `marks` is one op.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Marks {
    pub workspace: String,
    pub branch: String,
}

impl Marks {
    /// Whether this mark is about the workspace now focused.
    #[must_use]
    pub fn about(&self, workspace: &str) -> bool {
        self.workspace == workspace
    }
}

/// Read the `config` answer. `settings` is not read — the module doc says why.
pub(super) fn config(o: &Map<String, Value>) -> Result<String, String> {
    str_of(o, "text")
}

/// Read the `marks` answer: the branch, which is the engine's own re-read.
pub(super) fn marks(o: &Map<String, Value>) -> Result<String, String> {
    str_of(o, "branch")
}

#[cfg(test)]
mod tests;
