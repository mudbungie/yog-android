//! **The acts addressed to a conversation ROW** (DESIGN §13.5, bl-f97c): cut
//! it off mid-work and say this instead, settle it onto this workspace's
//! config lineage, or raise a human look on it.
//!
//! **One shape, because the seat has one gesture.** The wire spells three ops
//! and this file spells all three; what it does not do is give each a variant
//! of [`Act`](super::Act) beside `Stop` and `Nudge`. They arrive together —
//! one long-press on one row opens one menu — and each addresses the same two
//! facts, the workspace and the agent, differing only in the one parameter it
//! carries. So the subject is stated once in [`Act::Row`](super::Act::Row) and
//! the choice is this enum. That grouping is the server's own habit rather
//! than an invention here: yog's `Action::Ball(verb)` folds five ball verbs
//! behind one address, and its `arm`/`disarm`/`flag` trio rides a `Verb` the
//! same way. What it buys is one roster with one home — the seat's command,
//! the worker's act and the menu's items all name a row act with this value
//! instead of keeping parallel enums that drift.
//!
//! **What is absent, and why it is not a dead item.** `fork` is the fourth act
//! of this group's roster and it is not here. Its `from` is a fork point — a
//! commit of the conversation's own history, or a `config/<name>` head — and
//! the engine's own `fork::Attempt` says of it: *"Empty is not a value — the
//! composer refuses to fire without one, because a fork with no ref is a
//! different gesture."* Nothing this seat reads names one. The marks and the
//! tip ride the `agent` read (unbuilt, bl-146b) and the lineage names ride
//! `lineages` (unbuilt, bl-3685); a text field where an operator types a
//! commit sha on a phone is not a surface, it is this app asking the operator
//! to be the read §8 forbids it to derive. So fork keeps its `parity.toml`
//! line, re-cited to the ball that builds the picking surface, and this codec
//! goes on refusing its frame by name — which is REMOTE §3's third rule, not
//! an oversight.

use serde_json::{Map, Value, json};

use super::Act;
use super::fields::str_of;

/// **What a row's menu fires.** Three acts, none of which destroys anything:
/// an interrupt keeps everything already committed (its cut tool call is
/// reported to the model in band as having produced no result), a retarget
/// discards nothing and kills nothing, and a flag *"changes nothing else"* —
/// so no arming stands between the menu item and the wire. That is a reading
/// of these three ops, not a policy: the first act here whose product is that
/// its subject is gone earns the desktop's §4.20 idiom, and none of these is
/// one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RowAct {
    /// **Cut the conversation off mid-work and send it this text.** Stop then
    /// message, as one op the engine composes (its two trail rows are its
    /// business); the deposit is what restarts the conversation, so this
    /// leaves it running on the new text. With nothing running it is simply a
    /// send.
    Interrupt { content: String },
    /// **Settle the conversation onto this workspace's config lineage.** It
    /// re-forks at the next step and replays what it has already done on top:
    /// nothing is discarded and nothing is killed. It carries no parameter,
    /// which is why it is the one item that fires with an empty composer.
    Retarget,
    /// **Raise an attention item on the conversation, with a reason.** It
    /// stops nothing, messages nothing and touches nothing else — it puts a
    /// row on the ops trail and a mark on this conversation's own row, which
    /// is the read that settles it here.
    Flag { reason: String },
    /// **Take away this conversation's tool auto-approval** (yog §8.6, DESIGN
    /// §13.7), and its descendants' — including children it has not spawned
    /// yet. From its next call, everything but a read waits for an answer. It
    /// keeps running, keeps its branch and keeps reading: standing policy, not
    /// a kill, which is what puts it in this menu rather than behind an arming.
    Revoke,
    /// **Give it back.** The ordinary policy adjudicates again from the next
    /// call. It drives nothing — a conversation parked at a held call is
    /// released by answering that call — and where an ancestor is still
    /// revoked the conversation stays floored under it, which the receipt says
    /// rather than claiming a restore it did not make.
    Restore,
}

impl RowAct {
    /// **The wire's own op token**, which is also the word the menu item shows
    /// and the `act:` tag it carries (PARITY §4). One name, so the paint
    /// cannot label an item one thing and tag it another.
    ///
    /// `pub(crate)` rather than `pub` because it hands back a borrow and every
    /// caller is inside this crate (bootstrap rule 2's honest demotion).
    pub(crate) fn op(&self) -> &'static str {
        match self {
            Self::Interrupt { .. } => "interrupt",
            Self::Retarget => "retarget",
            Self::Flag { .. } => "flag",
            Self::Revoke => "revoke",
            Self::Restore => "restore",
        }
    }

    /// **The parameter this act needs typed before it can fire**, or `None`
    /// when it needs none. The composer is where that text is taken (§13.5),
    /// so this is the sentence a disabled item states beside itself — a greyed
    /// control says a thing is not live and nothing about what would make it
    /// live, which is the desktop's §4.20 reading and holds here too.
    ///
    /// `pub`, like `shell::place`'s pair and for its reason: this and `with`
    /// are pure readings the ANDROID paint spends, so a `pub(crate)` here is
    /// dead code on a host build. The seam is the same one bl-78c2 drew —
    /// what a menu item IS answers to the suite, what it looks like answers
    /// to the glass.
    pub fn wants(&self) -> Option<&'static str> {
        match self {
            Self::Interrupt { .. } => Some("type the text first"),
            Self::Flag { .. } => Some("type the reason first"),
            Self::Retarget | Self::Revoke | Self::Restore => None,
        }
    }

    /// The same act carrying `text` as whatever parameter it takes. The menu
    /// builds its items from the empty forms and spends the composer here, at
    /// the one site that knows which field the text is.
    #[must_use]
    pub fn with(&self, text: String) -> Self {
        match self {
            Self::Interrupt { .. } => Self::Interrupt { content: text },
            Self::Flag { .. } => Self::Flag { reason: text },
            // The three that take no parameter are themselves, whatever is in
            // the composer — one arm, because "unchanged" is one answer.
            Self::Retarget | Self::Revoke | Self::Restore => self.clone(),
        }
    }
}

/// Encode one row act, subject first. The spellings are the server codec's,
/// field for field. Written out per arm rather than assembled from a common
/// half: spelling the wire is this file's whole job, and a frame a reader can
/// see whole is worth three repeated keys.
pub(crate) fn encode(workspace: &str, agent: &str, act: &RowAct) -> Value {
    let op = act.op();
    match act {
        RowAct::Interrupt { content } => {
            json!({ "op": op, "workspace": workspace, "agent": agent, "content": content })
        }
        RowAct::Retarget | RowAct::Revoke | RowAct::Restore => {
            json!({ "op": op, "workspace": workspace, "agent": agent })
        }
        RowAct::Flag { reason } => {
            json!({ "op": op, "workspace": workspace, "agent": agent, "reason": reason })
        }
    }
}

/// Read one back. The caller matches the three ops before it gets here, so the
/// last arm is not reachable from `request::decode` today — it refuses by name
/// anyway, because the alternative is a `_` that would quietly answer some
/// future op with a retarget, which is exactly the misread REMOTE §3's third
/// rule forbids. Its own test calls it directly.
pub(crate) fn decode(op: &str, o: &Map<String, Value>) -> Result<Act, String> {
    let act = match op {
        "interrupt" => RowAct::Interrupt {
            content: str_of(o, "content")?,
        },
        "retarget" => RowAct::Retarget,
        "revoke" => RowAct::Revoke,
        "restore" => RowAct::Restore,
        "flag" => RowAct::Flag {
            reason: str_of(o, "reason")?,
        },
        other => return Err(format!("row: unknown op {other:?}")),
    };
    Ok(Act::Row {
        workspace: str_of(o, "workspace")?,
        agent: str_of(o, "agent")?,
        act,
    })
}

#[cfg(test)]
mod tests;
