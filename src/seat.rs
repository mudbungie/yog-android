//! The seat's view model (bl-5a98): the phone's equivalent of the window's
//! asker — it owns the `Seat`, re-asks the standing set (workspaces, the
//! focused workspace's conversations, the focused conversation's transcript)
//! at human cadence off the UI thread, publishes decoded rows for the frame
//! to paint, and posts Acts (the message deposit) from the composer.
//!
//! Mirrors the server side's asker/poster split; one connection per ask
//! (REMOTE §3, "the seat polls") stands until upstream rules otherwise. The
//! frame renders snapshots and blocks on nothing: every wire crossing
//! happens on the model's one worker thread, and the two sides talk over
//! channels — no locks, so rule 7 stays vacuous here.

use crate::codec::{ConvRow, Entry, ProviderRow, WsRow};

/// What the frame paints: the standing set as of the last completed
/// refresh, with the focus it was asked under — one value, published
/// atomically, so a frame never pairs one focus with another focus's rows.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Snapshot {
    pub focus: Focus,
    pub workspaces: Vec<WsRow>,
    /// The focused workspace's conversations; empty when none is focused.
    pub conversations: Vec<ConvRow>,
    /// The focused conversation's transcript; empty when none is focused.
    pub transcript: Vec<Entry>,
    /// **The focused workspace's providers**, as the engine last listed them
    /// (bl-0267). Empty until the selectors have been opened once, and empty
    /// again the moment the focus moves to another workspace: a sign-in is a
    /// per-workspace fact, so one workspace's list under another's name would
    /// be the same wrong claim as one focus's rows under another's.
    pub providers: Vec<ProviderRow>,
    /// Each provider's models, keyed by the provider they belong to — the key
    /// IS the pairing, so no frame can paint one provider's list under
    /// another's name.
    pub models: std::collections::BTreeMap<String, Vec<String>>,
    /// The last refresh's failure or a refused deposit, one sentence for
    /// the banner. `None` is "the engine answered".
    pub error: Option<String>,
}

/// Where the operator is looking. Depth is monotone: an agent without a
/// workspace cannot be spelled, so narrowing focus never leaves a stale
/// deeper level behind.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Focus {
    pub workspace: Option<String>,
    pub agent: Option<String>,
}

mod acts;
mod model;
mod options;
mod pass;

pub use model::Model;

#[cfg(test)]
mod tests;
