//! **The foot's whole wire surface** (yog REMOTE §4.2, upstream bl-1dd3):
//! three gestures, and nothing else is reachable from here.
//!
//! §4.2 mints exactly two certificate grades, and the foot's is stated as a
//! closed set: *"Foot grade — the tool-host gestures and **nothing else**:
//! `advertise` (§5.1), `invocations` and `complete` (§5.3). No other `Query`,
//! no other `Action`."* And the sentence the set is measured against:
//!
//! > *"A foot cannot ask about the world: not the workspaces, not the board,
//! > not the trail, not a transcript. It cannot act on the world: no message,
//! > no start, no stop, no ball, no config. What it may do is answer for the
//! > machine it is: state what this box can run, wait for work addressed to
//! > it, and hand back what happened. Note which of §5.3's four verbs is
//! > absent — `invoke`, the asking side's. A foot is invoked; it never
//! > invokes."*
//!
//! **This type is that sentence in the type system.** It owns a
//! [`Seat`](crate::transport::Seat) and never hands it out, so the general
//! "encode any gesture and send it" door is not reachable from the host loop:
//! the three methods below are the whole of what this device can say on a foot
//! channel. §4.2 also names the phone directly as the class the grade exists
//! for — *"a foot is the class most likely to run on a box the operator trusts
//! least (a build machine, a phone, a box in someone else's house)"*.
//!
//! **This is not enforcement and does not pretend to be.** The engine enforces
//! the grade with one raise at its chokepoint, in band, naming it; nothing on
//! this device can grant itself anything. What the narrowing buys is that a
//! foot-graded phone cannot *accidentally* spend a gesture its own certificate
//! refuses — which would earn a refusal per question, per pass, forever, and
//! put a wall of sentences where a component boundary belongs.
//!
//! **A seat's tool host rides this same surface**, and that is not a
//! contradiction: the tool-host gestures are the tool-host gestures whatever
//! grade the leaf carries. One code path, so the co-located case and the
//! foot-only case are the same case — REMOTE §12's *"one transport, one code
//! path, no place to hide the bug"* one component down.

use crate::codec::reply::Reply;
use crate::codec::{Act, Ask, Capture, Gesture, Invocation, Tool, encode};
use crate::material::Material;
use crate::transport::{Seat, Wire};

/// This device's end of a foot channel.
pub struct Foot {
    seat: Seat,
}

impl Foot {
    /// Open a foot channel over provisioned material. Nothing is dialled here
    /// — a foot is a fact about what this machine may say, not about whether
    /// an engine happens to be up — and every gesture below is its own
    /// connection, exactly as a seat's is.
    pub fn open(material: &Material) -> Result<Self, String> {
        Ok(Self {
            seat: Seat::open(material)?,
        })
    }

    /// The address this foot dials, for the sentence a stopped host publishes.
    pub fn address(&self) -> String {
        self.seat.address()
    }

    /// **`advertise`** (REMOTE §5.1): what this machine can run, presented on
    /// connect. It names no client, and that is the gesture — the identity a
    /// set lands under is the connection's certificate common name, and a
    /// `client` field would let any connection overwrite any other's.
    ///
    /// **The receipt's one reading is handed back** (PROTOCOL 8, yog bl-66d4):
    /// `wrote` — whether the engine changed the stored set or found it
    /// identical to the one presented and compared. The set is still not
    /// echoed, and this is not an echo of it: it is a fact about the engine's
    /// *document*, and the one fact in this exchange this box cannot compute
    /// for itself.
    ///
    /// What it MEANS depends on which presentation earned it, which this
    /// method cannot know — so it reports the reading and the judgement is
    /// [`crate::host`]'s, the only layer that knows whether a channel had
    /// already presented.
    pub fn advertise(&self, tools: Vec<Tool>) -> Result<bool, Wire> {
        match self.said(&Gesture::Act(Act::Advertise { tools }))? {
            Reply::Advertised { wrote } => Ok(wrote),
            other => Err(wrong(&other)),
        }
    }

    /// **`invocations`** (REMOTE §5.3): the follow-class read — this
    /// machine's next work, answered when there is some. The ask never
    /// inverts (§3), so this device waits here rather than listening on a
    /// socket it would have to open. An empty answer is ordinary: a hold that
    /// ended quietly.
    pub fn invocations(&self) -> Result<Vec<Invocation>, Wire> {
        match self.said(&Gesture::Ask(Ask::Invocations))? {
            Reply::Invocations(rows) => Ok(rows),
            other => Err(wrong(&other)),
        }
    }

    /// **`complete`** (REMOTE §5.3): one invocation answered with what running
    /// it captured. Only the client it was addressed to may post one, so this
    /// too names no client.
    ///
    /// The receipt is read rather than discarded: an engine that refused the
    /// completion — an expired handle, a slot addressed elsewhere — is
    /// something this device must stop against rather than keep answering
    /// into.
    pub fn complete(&self, invocation: String, capture: Capture) -> Result<(), Wire> {
        match self.said(&Gesture::Act(Act::Complete {
            invocation,
            capture,
        }))? {
            Reply::Routed { .. } => Ok(()),
            other => Err(wrong(&other)),
        }
    }

    /// One gesture over the wire, in the one codec and the one reply decoder —
    /// so this device speaks exactly what every other client speaks and can
    /// add nothing to it.
    ///
    /// The **kind** each gesture earns is matched by its own method above, and
    /// the sentence for a wrong one is [`wrong`]'s — one spelling shared by
    /// three callers, rather than one check that would leave each caller's
    /// own arm unreachable and unprovable.
    fn said(&self, gesture: &Gesture) -> Result<Reply, Wire> {
        self.seat.answered(&encode(gesture))
    }
}

/// The wrong-kind sentence. It names the kind and never the rows it carried:
/// a reply this device did not ask for is not content to render. A
/// [`Wire::Unusable`] and never a transport failure — the channel worked
/// perfectly and carried an answer this gesture does not earn, so the host
/// that redials a broken socket stops dead on this one (bl-8641), on every leg
/// including the follow read that a bare refusal IS worth re-dialling
/// (bl-8bd0: the two used to share one class and the matrix needs them apart).
fn wrong(reply: &Reply) -> Wire {
    Wire::Unusable(format!(
        "the engine answered {}, not this machine's work",
        reply.kind()
    ))
}

#[cfg(test)]
mod tests;
