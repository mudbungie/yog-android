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

/// An act on the world outside the chat loop and the tool-host trio. DESIGN
/// §2: the codec *"spells exactly what the phone seat spends — the chat loop …
/// and grows per consumer, never speculatively."*
///
/// **Its read-side twin is gone** (bl-5a41). `READ` stood here for the same
/// class one direction over and `login-tail` was the last shape citing it, so
/// the constant went with the row rather than waiting for another: every read
/// in the corpus is now one this seat spells. What is left on this side is
/// `pin`/`unpin`, which are VISION V1.2's tree assertions and not a slice
/// boundary at all — `NO_TREE` says the same thing about the read they would
/// anchor.
pub const ACT: &str = "a world act outside the chat-loop slice (DESIGN §2)";

/// **A fork pins no skills** (DESIGN §13.16). A skill set is a choice off the
/// same config a role is, and no read on this wire lists one — so a control
/// offering names would be inventing them, and the empty list is the honest
/// gesture. A frame that pins any is refused by name rather than read as the
/// attempt without them, which is the silent misread §3's third rule forbids.
pub const NO_SKILLS: &str =
    "a pinned skill is a choice off a config no read here lists (DESIGN §13.16)";

/// **The ball pane spells the envelope and not the scheduling inside it**
/// (DESIGN §13.9, bl-f36e). `create` and `update` may carry `fields`: an
/// ordered array of priority, tag, parent and needs applications, each of
/// which is a picker this pane does not have. The desktop refused the same
/// and recorded it by count and reason (lernie DESIGN §4.35). So the two
/// frames without it read and round-trip, and the frame with it is refused by
/// name — reading it as the edit without its fields would answer a
/// reprioritisation with a bare amendment, which is the silent misread §3's
/// third rule forbids.
pub const NO_SCHEDULING: &str =
    "the scheduling fields are pickers this pane does not have (DESIGN §13.9)";

/// **`files` asked AT a commit is a different tree** (DESIGN §13.15). `at` is
/// VISION V1.2's pin — an assertion about which commit is being read — and the
/// controls that would make one are `pin` and `unpin`, both `parity.toml`
/// lines. So the two bare-tree frames read and round-trip, and the two
/// anchored ones are refused by name rather than answered off the live
/// worktree, which is the silent misread §3's third rule forbids.
pub const NO_TREE: &str =
    "the tree a pin names is an assertion this seat has no control for (DESIGN §13.15)";

/// **The two config destinations this seat has no picker for** (DESIGN
/// §13.17). `config` is one op that takes a `target`, and two of its five
/// destinations want choices off reads this app does not make: a
/// `litany-workflow` names a workflow, and a `branch` names a lineage, an
/// origin and a path inside a config tree. Their frames are refused by name
/// rather than read as one of the three this seat spells.
pub const NO_DESTINATION: &str =
    "the workflow and branch destinations want pickers this seat has not got (DESIGN §13.17)";

/// **A candidate gesture always names its ball** (DESIGN §13.12). All three
/// take an optional `ball` upstream, and omitting it is the bare
/// project-repo gesture aimed at the integration branch — a subject this seat
/// has no row for. Every gesture here is composed off a science row that
/// names one, so a ball-less frame is refused by name rather than read as the
/// frame with a ball. The desktop recorded the same three by count and reason
/// (lernie DESIGN §4.36).
pub const ALWAYS_A_BALL: &str =
    "the bare project-repo obligation is a subject no row here names (DESIGN §13.12)";

/// **`help` is answered from the table this repository already vendors**
/// (DESIGN §13.14). §2 rules that the corpus and the spoken version move
/// together — *"a protocol bump upstream is a re-vendor and a rebuild here"* —
/// and a peer of another version is refused fail-closed at the §3 preface, so
/// for any engine this build can talk to the vendored table IS that engine's
/// table. Asking for it would be a radio spend for an answer already compiled
/// in. The surface exists (`crate::help`, and a control carries `act:help`);
/// what does not exist is the gesture.
pub const ALREADY_HELD: &str =
    "the op table is vendored and compiled in — this seat never asks for it (DESIGN §13.14)";

/// REMOTE §4.2, on the foot's gesture set: *"Note which of §5.3's four verbs
/// is absent — `invoke`, the asking side's. A foot is invoked; it never
/// invokes."* `capture` is that verb's read half and goes with it.
pub const ASKING_SIDE: &str = "§5.3's asking side — this device is invoked, it never invokes";

/// Every reply this client does not read is the answer to a gesture it does
/// not send. One reason, because it is one fact: the reply vocabulary is the
/// shadow of the request slice, and the two move together or one of them is
/// the bug.
pub const UNSENT: &str = "the answer to a gesture this codec does not send";

/// **The follow lane is held, and its frame is an append** (bl-4822, held
/// since bl-8e3c). It was skipped for two waves and the skip's reasoning is
/// kept here because it is what makes the consumption lawful: REMOTE §5.5
/// made the lane's frame an **append** —
///
/// > *"Absorb every frame of a read, in order, onto an empty fold. What you
/// > hold after the last frame you have received is what you paint."*
///
/// — under a wire spelling that did not move at all, and REMOTE says the
/// consequence out loud: *"The corpus ledger records field paths and types,
/// so it cannot see a change of meaning under an unchanged signature."* A
/// green conformance run still says nothing about that, and re-vendoring the
/// fixtures is still not consuming the section.
///
/// **What this seat does with it.** It holds the connection (DESIGN §14.1)
/// and owes the real fold, which is `codec::follow::Stream::absorb` — the
/// engine's own operation copied, so an engine frame and this seat's
/// accumulation agree by contract. `Seat::answered` is the wrong door for the
/// lane and is not on it: it decodes `stream.last()`, which for an append
/// stream is the final delta alone; a lane parks on `Seat::hold` and folds
/// every frame. The one-shot reading that stood here before was true of the
/// ask and false of the intake, which holds the read (protocol 10's lesson,
/// §14.1 there).
///
/// The tool host's `invocations` read is follow-CLASS and is **not** this
/// lane: its answer is one frame of rows, and §5.5's rule is about a text
/// fold.
/// DESIGN §8: *"One rung, and the other two are not omissions. The bare rung
/// is the whole slice: a phone is not where a work directory is chosen or a
/// ball is bound."*
pub const BARE_RUNG: &str = "the bare rung is this device's whole slice (DESIGN §8)";

/// DESIGN §8, on the firing gesture: the name prediction is *"the firing
/// seat's own"* and a phone predicts none, so this codec writes the null and
/// reads only the null.
pub const NO_SEED: &str = "this seat predicts no conversation name (DESIGN §8)";
