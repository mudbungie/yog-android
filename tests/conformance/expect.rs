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

/// **`fork` is the one row act this seat cannot fire, and the reason is a
/// missing READ rather than a missing surface** (DESIGN §13.5, bl-f97c). Its
/// three siblings — `interrupt`, `retarget`, `flag` — moved to `Reads` when
/// the conversation row's menu landed. `fork` did not, because its `from` is a
/// fork point and the engine's own `fork::Attempt` says *"Empty is not a value
/// — the composer refuses to fire without one, because a fork with no ref is a
/// different gesture."* Nothing this seat reads names one: the marks and the
/// tip ride `agent` (bl-146b) and the lineage names ride `lineages` (bl-3685),
/// both unbuilt. So the frame goes on being refused by name, which is the
/// honest answer rather than a shape half-spelled.
pub const NO_FORK_POINT: &str =
    "the row act whose fork point no read here names — bl-99fd builds the picking surface";

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

/// **`governing` asked ABOUT a commit is a different question** (DESIGN
/// §13.11). The read this screen makes is the standing one — *which config
/// governs this conversation now* — and the anchored form names a fork point,
/// a commit of the conversation's own history. The surface that picks one is
/// bl-99fd's, which is the same missing read `fork` is cited to in
/// `parity.toml`. So the bare frame reads and round-trips, and the anchored
/// one is refused by name rather than answered as the standing read.
pub const NO_ANCHOR: &str =
    "the config governing a COMMIT wants a fork point no read here names (bl-99fd)";

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

/// REMOTE §8.4's act and its answer. This device is the thing enrolled, and it
/// is enrolled **out of channel** — §1.4 stands, the new device performs no
/// channel act, and the material reaches it as a photograph or a paste
/// (DESIGN §11). So the phone neither mints nor is answered here: `enroll` is
/// an operator-grade act a seated operator sends from a screen this app does
/// not have, and `enrolled` is what that operator's seat is handed.
///
/// It is a *skip*, not an absence: `src/envelope.rs` already reads §8.4's six
/// fields out of the QR envelope, which is the same six facts arriving by the
/// route this device actually has. The day this app grows a screen that
/// enrolls the *next* device, the row becomes `Reads` and the decoder is one
/// function; until then a frame of either shape is refused naming itself.
pub const NOT_THE_MINTER: &str =
    "§8.4's minting side — this device is enrolled out of channel, it enrolls nobody";
