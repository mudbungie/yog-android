//! What this client says it does with one corpus shape, and the words it says
//! it in.
//!
//! REMOTE §3, on what a client owes the conformance corpus: *"A shape a client
//! does not implement is still one it must not misread, so skipping a fixture
//! is a decision recorded in the client, never a silent pass."* This enum is
//! that record's vocabulary, and the reasons below are its prose — grouped
//! rather than written per shape, because fifty shapes skipped for one reason
//! are one decision, and fifty copies of it would be fifty places to edit.

/// This client's decision about one shape.
#[derive(Debug, Clone, Copy)]
pub enum Expect {
    /// Every frame of this shape decodes, and — for a request — re-encodes to
    /// the frame it came from.
    Reads,
    /// No frame of this shape decodes. Each is refused **naming the shape**,
    /// which is the difference between a recorded skip and a silent pass.
    Refuses(&'static str),
    /// This codec spells part of the shape: exactly `reads` frames close the
    /// round trip, and every other frame is refused by name. A shape lands
    /// here when the thing it does not spell is *inside* the envelope rather
    /// than the envelope itself.
    Partial { reads: usize, reason: &'static str },
}

impl Expect {
    /// The decision's own words. The count check below prints them, which is
    /// what makes a reason load-bearing rather than a comment: when a shape's
    /// count moves, the failure says what this client had decided and why, so
    /// the next author edits the decision instead of the number.
    pub fn reason(self) -> &'static str {
        match self {
            Self::Reads => "this codec spells it",
            Self::Refuses(reason) | Self::Partial { reason, .. } => reason,
        }
    }
}

/// A read of the world outside the chat loop this codec spells. DESIGN §2:
/// the codec *"spells exactly what the phone seat spends — the chat loop …
/// and grows per consumer, never speculatively."*
pub const READ: &str = "a world read outside the chat-loop slice (DESIGN §2)";

/// An act on the world outside the chat loop and the tool-host trio.
pub const ACT: &str = "a world act outside the chat-loop slice (DESIGN §2)";

/// REMOTE §4.2, on the foot's gesture set: *"Note which of §5.3's four verbs
/// is absent — `invoke`, the asking side's. A foot is invoked; it never
/// invokes."* `capture` is that verb's read half and goes with it.
pub const ASKING_SIDE: &str = "§5.3's asking side — this device is invoked, it never invokes";

/// Every reply this client does not read is the answer to a gesture it does
/// not send. One reason, because it is one fact: the reply vocabulary is the
/// shadow of the request slice, and the two move together or one of them is
/// the bug.
pub const UNSENT: &str = "the answer to a gesture this codec does not send";

/// DESIGN §8: *"One rung, and the other two are not omissions. The bare rung
/// is the whole slice: a phone is not where a work directory is chosen or a
/// ball is bound."*
pub const BARE_RUNG: &str = "the bare rung is this device's whole slice (DESIGN §8)";

/// DESIGN §8, on the firing gesture: the name prediction is *"the firing
/// seat's own"* and a phone predicts none, so this codec writes the null and
/// reads only the null.
pub const NO_SEED: &str = "this seat predicts no conversation name (DESIGN §8)";
