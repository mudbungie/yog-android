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

use crate::codec::{ConvRow, Entry, Found, OpRow, ProviderRow, QueueRow, RoleRow, WsRow};

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
    /// **What the focused workspace's roles are actually set to** (bl-e9f9),
    /// as the engine last answered. Empty is two things a control must not
    /// tell apart wrongly: a workspace with nothing assigned, and an engine
    /// too old to answer the read at all — both mean *nothing to seed from*,
    /// and neither is an error.
    pub roles: Vec<RoleRow>,
    /// How many times the assignments have been read. The controls watch it
    /// move to know their optimistic value has been overtaken by truth —
    /// they never read the number.
    pub roles_read: usize,
    /// **What this seat's deposits have earned** (bl-66fb): how many the
    /// engine took, and how many it refused, since the worker started. The
    /// composer's echo reads the CHANGE and never the number — it remembers
    /// both at the moment it sent and watches for either to move, which is
    /// what tells a muted echo from an inked one without a receipt id the
    /// wire does not carry.
    pub landed: usize,
    pub refused: usize,
    /// **How many deposits earned no reply at all** (yog REMOTE §3, bl-07b1).
    /// A third counter and not a second reading of `refused`, because the two
    /// are opposite instructions to the operator: a refusal is the engine
    /// saying no, and the composer takes its draft back; an act in doubt may
    /// have been taken, and the one thing that must not happen is its being
    /// said again. The echo watches this move exactly as it watches the other
    /// two, and stands where it is when it does.
    pub doubted: usize,
    /// **The decision queue as the engine last answered it, whoever asked**
    /// (§13.7, §13.8). Two surfaces spend it now — the held-call band under an
    /// open conversation, which a pass reads it for, and the queue screen,
    /// which asks for it when it opens (bl-35bd) — so the rows are held by
    /// `Standing` rather than by one depth's pass, and every published
    /// snapshot carries the last answer whichever gesture earned it. That is
    /// the queue's own nature rather than a convenience: it names no workspace
    /// and no conversation, so nothing about the focus it was read under binds
    /// it to that focus.
    ///
    /// Empty is *nothing is waiting on you*, and — before the first read of a
    /// launch that also missed the §14 cache — *nobody has asked yet*. The two
    /// paint the same, because an empty band and an unasked one are the same
    /// absence on the screen in front of you.
    pub queue: Vec<QueueRow>,
    /// **The ops trail as it was last read** (yog §4.2, DESIGN §13.8). Empty
    /// is two things that paint the same and are not worth telling apart: an
    /// engine that has done nothing since the trail was cleared, and a seat
    /// that has not opened the surface. Neither is an error, and both mean
    /// *there is nothing here to read*.
    ///
    /// It rides the snapshot and never the §14 cache, for the search's reason:
    /// the cache is the standing set a pass re-asks, and this is a gesture's
    /// answer — opening the trail is what asks for it.
    pub trail: Vec<OpRow>,
    /// **What the last needle found** (yog DESIGN §8.5, bl-4c2b). `None` is
    /// *no search was made* — never *nothing matched*, which is a `Some`
    /// carrying its own needle and no hits. The two are the same value to
    /// anything that reads only the hit count, and they are opposite screens.
    ///
    /// It rides the snapshot and never the §14 cache: the cache is the world
    /// the engine wrote down, and a search is a question this operator asked
    /// a moment ago — reviving one on the next boot would be the app opening
    /// on a search nobody just made.
    pub search: Option<Found>,
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

pub(crate) mod acts;
mod asks;
mod model;
mod options;
mod pass;
mod posted;
mod worker;

pub use model::Model;

#[cfg(test)]
mod tests;
