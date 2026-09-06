//! **The five acts of the admin surface** (DESIGN §13.17): write a config
//! file, mark a workspace's task branch, flush its inbox, and the two
//! deletions.
//!
//! **One shape, and the address is INSIDE it** — which is the difference from
//! [`BallAct`](super::super::BallAct) and is worth saying. The ball acts share
//! one address and state it once outside the choice; these five do not share
//! one at all: two config destinations name no place, the third and the
//! workspace ops name a workspace, and `delete-agent` names a conversation. So
//! the grouping here buys one command and one roster rather than a shared
//! address, and each variant carries what it is about.
//!
//! **`typed` is a PARAMETER on one and an ARMING on the other, and this codec
//! reads which off the wire's own grammar** (lernie DESIGN §4.20, whose ruling
//! transfers whole). `delete-workspace` is refused by the engine unless the
//! typed name matches, so the seat makes it an enablement; `delete-agent`'s
//! empty string deletes the one conversation and its name typed back admits
//! the descendants, so both values are gestures somebody meant. Nothing about
//! either arming is on the wire — an arm is a property of the glass (§13.8) —
//! and the DIFFERENCE between them is not this seat's invention.

use serde_json::{Map, Value, json};

use super::super::Act;
use super::super::fields::str_of;
use super::{Destination, destination, target};

/// **What the admin surface fires.**
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdminAct {
    /// **Write one config file whole.** The text is the file, not a patch:
    /// the engine takes the staged bytes and runs them through that
    /// destination's own pipeline.
    Config { at: Destination, text: String },
    /// **Point a workspace's balls space at this branch.** Undone by marking
    /// the old branch back, so nothing arms it.
    Marks { workspace: String, branch: String },
    /// **Deliver the mail nobody has taken.** It adds; nothing is unmade.
    Scan { workspace: String },
    /// **Delete one conversation**, and — when `typed` is its own name — its
    /// descendants with it.
    DeleteAgent {
        workspace: String,
        agent: String,
        typed: String,
    },
    /// **Delete a workspace.** The engine refuses unless `typed` is its name,
    /// which is what makes the control an enablement rather than an arming.
    DeleteWorkspace { workspace: String, typed: String },
}

impl AdminAct {
    /// **The wire's own op token**, which is also the control's label and the
    /// `act:` tag it carries (PARITY §4).
    pub(crate) fn op(&self) -> &'static str {
        match self {
            Self::Config { .. } => "config",
            Self::Marks { .. } => "marks",
            Self::Scan { .. } => "scan",
            Self::DeleteAgent { .. } => "delete-agent",
            Self::DeleteWorkspace { .. } => "delete-workspace",
        }
    }

    /// **The sentence a control states while it cannot fire**, or `None` when
    /// the act needs nothing typed (`BallAct::wants`' rule at a third site).
    /// `delete-agent` is deliberately not here: an empty box is one of its two
    /// lawful gestures, so it is never dark for want of one.
    ///
    /// `pub` for that method's reason — a pure reading the android paint
    /// spends, so `pub(crate)` would be dead code on a host build.
    #[must_use]
    pub fn wants(&self) -> Option<&'static str> {
        match self {
            Self::Config { .. } => Some("edit the file first"),
            Self::Marks { .. } => Some("type the branch first"),
            Self::DeleteWorkspace { .. } => Some("type this workspace's name"),
            Self::Scan { .. } | Self::DeleteAgent { .. } => None,
        }
    }
}

/// Encode one admin act. The spellings are the server codec's, field for
/// field.
pub(crate) fn encode(act: &AdminAct) -> Value {
    let op = act.op();
    match act {
        AdminAct::Config { at, text } => {
            json!({ "op": op, "target": target(at), "text": text })
        }
        AdminAct::Marks { workspace, branch } => {
            json!({ "op": op, "workspace": workspace, "branch": branch })
        }
        AdminAct::Scan { workspace } => json!({ "op": op, "workspace": workspace }),
        AdminAct::DeleteAgent {
            workspace,
            agent,
            typed,
        } => json!({ "op": op, "workspace": workspace, "agent": agent, "typed": typed }),
        AdminAct::DeleteWorkspace { workspace, typed } => {
            json!({ "op": op, "workspace": workspace, "typed": typed })
        }
    }
}

/// Read one back. The caller matches the ops before it gets here, so the last
/// arm is unreachable from `request::decode` — it refuses by name anyway, for
/// `codec::row`'s reason, and its own test calls it directly.
pub(crate) fn decode(op: &str, o: &Map<String, Value>) -> Result<Act, String> {
    let act = match op {
        "config" => AdminAct::Config {
            at: destination(o.get("target"))?,
            text: str_of(o, "text")?,
        },
        "marks" => AdminAct::Marks {
            workspace: str_of(o, "workspace")?,
            branch: str_of(o, "branch")?,
        },
        "scan" => AdminAct::Scan {
            workspace: str_of(o, "workspace")?,
        },
        "delete-agent" => AdminAct::DeleteAgent {
            workspace: str_of(o, "workspace")?,
            agent: str_of(o, "agent")?,
            typed: str_of(o, "typed")?,
        },
        "delete-workspace" => AdminAct::DeleteWorkspace {
            workspace: str_of(o, "workspace")?,
            typed: str_of(o, "typed")?,
        },
        other => return Err(format!("admin: unknown op {other:?}")),
    };
    Ok(Act::Admin(act))
}
