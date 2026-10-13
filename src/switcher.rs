//! **The switcher** (DESIGN §22 item 7, bl-3edc): the workspaces and the
//! focused workspace's conversations fused into one list — which rows, in
//! which order, and what a tap on each yields. Pure and host-tested;
//! `shell/switcher.rs` paints the rows and performs the taps. The desktop's
//! shape, kept (lernie `ui::switcher`): the roster as an overlay, closed by
//! choosing.
//!
//! **One heading per workspace, and only the focused one is open.** Its
//! conversations are a read this seat makes only at the focused workspace
//! (§14), so the others are folded and a tap on a heading is what focuses
//! one. Under the open heading stands `+ new`, then its conversations as
//! §13.20's tree — `crate::roster`'s order and rails, reused rather than
//! restated.

use crate::codec::{ConvRow, WsRow};

/// The word under every open workspace that begins a conversation on it.
pub const NEW: &str = "+ new";

/// **One row of the switcher.**
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Entry {
    /// A workspace: its name, how many agents it holds, whether anything in
    /// it waits on the operator, and whether it is the open one.
    Heading {
        workspace: String,
        agents: usize,
        waiting: bool,
        open: bool,
    },
    /// A conversation under the open workspace, with its rails (§13.20).
    Conversation {
        workspace: String,
        row: Box<ConvRow>,
        rails: Vec<bool>,
    },
    /// The way to begin one on the open workspace.
    New { workspace: String },
}

/// **What a tap yields.**
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Taken {
    /// Focus this workspace and keep the switcher standing: its conversations
    /// are what the operator asked to see.
    Focus(String),
    /// Focus this conversation and close the switcher.
    Open { workspace: String, agent: String },
    /// Focus this workspace with no conversation and close the switcher: the
    /// chat in start mode, the caret in the field.
    Start(String),
}

/// **The rows, in order**: every workspace in the engine's order, and under
/// the focused one `+ new`, then its conversations — newest subtree first,
/// each in the engine's descent.
pub fn entries(
    workspaces: &[WsRow],
    focused: Option<&str>,
    conversations: &[ConvRow],
) -> Vec<Entry> {
    let listed = crate::roster::ordered(conversations.to_vec());
    let rails = crate::roster::threads(&listed);
    let mut out = Vec::new();
    for ws in workspaces {
        let open = focused == Some(ws.workspace.as_str());
        out.push(Entry::Heading {
            workspace: ws.workspace.clone(),
            agents: ws.agents,
            waiting: ws.attention > 0,
            open,
        });
        if !open {
            continue;
        }
        // **`+ new` stands first** (bl-a3b6): after the tree it was pushed
        // below the fold by any workspace with a few conversations, so
        // beginning one needed a scroll the first-time thumb does not know
        // to make.
        out.push(Entry::New {
            workspace: ws.workspace.clone(),
        });
        for (row, rails) in listed.iter().zip(&rails) {
            out.push(Entry::Conversation {
                workspace: ws.workspace.clone(),
                row: Box::new(row.clone()),
                rails: rails.clone(),
            });
        }
    }
    out
}

impl Entry {
    /// What a tap on this row does. The open heading does nothing: it is
    /// already what is focused, and re-focusing it would drop the
    /// conversation the switcher was opened over.
    pub fn taken(&self) -> Option<Taken> {
        match self {
            Self::Heading { open: true, .. } => None,
            Self::Heading { workspace, .. } => Some(Taken::Focus(workspace.clone())),
            Self::Conversation { workspace, row, .. } => Some(Taken::Open {
                workspace: workspace.clone(),
                agent: row.root_id.clone(),
            }),
            Self::New { workspace } => Some(Taken::Start(workspace.clone())),
        }
    }
}

#[cfg(test)]
mod tests;
