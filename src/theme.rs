//! **The visual language's one home in code** (`docs/STYLE.md`, bl-549b): the
//! ground and its elevation tints, the ink scale, the soft-neon accent each
//! STATE wears, the brand, and the spacing, type and touch scales every
//! screen lays out with.
//!
//! Pure, and host-tested under the 100% floor: a colour here is three bytes
//! and a size is a number, so what the language IS answers to the suite.
//! `shell::theme` is the one adapter that turns these into egui's `Visuals`
//! at app start and into a `Color32` at a paint site; no screen spells a
//! colour of its own (`rules/no-literal-colour.yml`).
//!
//! **Colour means state.** A hue on the glass says what something is DOING —
//! asking for you, working, thinking, annotated, at rest, failed (operator
//! ruling, six and no seventh) — and nothing else is coloured: hierarchy
//! comes from spacing, type and alignment, and a speaker is told by the
//! weight of its rule rather than a hue of its own. The brand is one of the
//! six, not a seventh: the working blue, which the focus ring and the
//! operator's own words carry.

use crate::codec::{Framing, Tone};
use crate::rows::Role;

/// Straight RGB, no alpha: a tint is a fill and never a blend, so what a
/// picture shows is what this table says.
pub type Rgb = [u8; 3];

/// **The ground**: the mark's own void (`icon::VOID_DEEP`), so the app and
/// its launcher icon stand on one surface.
pub const GROUND: Rgb = [10, 8, 15];
/// **Elevation one**: a row under a thumb, the composer's field, a popup's
/// body. Raised from the ground by tint alone — there is no outline anywhere.
pub const SURFACE: Rgb = [22, 20, 30];
/// **Elevation two**: a control while it is pressed or open.
pub const RAISED: Rgb = [36, 33, 48];
/// The most a boundary may be: one point of this between two rows, and
/// nothing around a block.
pub const HAIRLINE: Rgb = [48, 44, 62];

/// Primary text.
pub const INK: Rgb = [232, 230, 238];
/// Secondary text — a stamp, a count, a hint, a resting row's words.
pub const INK_WEAK: Rgb = [152, 148, 168];
/// A placeholder, a rule nobody should read, a disabled control's words.
pub const INK_FAINT: Rgb = [98, 94, 114];

/// **The brand**: one of the six, never a seventh. The working blue — worn
/// by the focus ring, the send, and the operator's own words.
pub const BRAND: Rgb = WORKING;

/// The six accents, named here by hue ONLY so the table in `docs/STYLE.md`
/// and the operator's ruling can be read against the bytes; every paint site
/// names a `State`, never one of these.
const ATTENTION: Rgb = [110, 222, 148];
const WORKING: Rgb = [96, 168, 255];
const INFERENCE: Rgb = [196, 140, 255];
const ANNOTATION: Rgb = [255, 170, 92];
const ERROR: Rgb = [255, 108, 132];

/// What something on the glass is doing (operator ruling: six states, six
/// colours, no seventh). Each wears one accent and nothing else wears it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    /// Asking for the operator: waiting on you, a parked call held for an
    /// answer, a queue entry, the attention mark. Green.
    Attention,
    /// Doing work: a tool call running, a foot executing, a step in flight.
    /// Blue, and the brand.
    Working,
    /// Inference: the model is generating, the thinking tail. Purple.
    Inference,
    /// High-salience annotation: an operator note, an interrupt, a thing worth
    /// noticing that is neither working nor failed. Orange.
    Annotation,
    /// Done, archived, nothing happening — the opposite of attention. Grey:
    /// weak ink, so a quiet list reads as quiet.
    Rest,
    /// Failed, and it will not mend itself. Red.
    Error,
}

/// Every state, in one place so the six can be judged together.
pub const STATES: [State; 6] = [
    State::Attention,
    State::Working,
    State::Inference,
    State::Annotation,
    State::Rest,
    State::Error,
];

/// **The soft-neon accent a state wears.** Soft is a bound, not a mood: no
/// accent is more than two-thirds saturated (`tests::accents_are_soft`), so
/// a tint of one on the ground glows rather than shouts.
pub fn accent(state: State) -> Rgb {
    match state {
        State::Attention => ATTENTION,
        State::Working => WORKING,
        State::Inference => INFERENCE,
        State::Annotation => ANNOTATION,
        State::Rest => INK_WEAK,
        State::Error => ERROR,
    }
}

/// The wire's row tone (REMOTE §11) read as a state. `Good` is a result that
/// came back and is done, which is rest; `Live` is the streaming tail, which
/// is inference; `InFlight` is a tool running, which is work; `Held` is a call
/// parked for an answer, which is the one thing on this glass that is asking
/// for you. The two ink tones are the ink scale, not a state at all.
pub fn tone(tone: Tone) -> Rgb {
    match tone {
        Tone::Plain => INK,
        Tone::Weak => INK_WEAK,
        Tone::Good => accent(State::Rest),
        Tone::Bad => accent(State::Error),
        Tone::Live => accent(State::Inference),
        Tone::InFlight => accent(State::Working),
        Tone::Held => accent(State::Attention),
    }
}

/// **A step's §4.4 framing read as a state** (REMOTE §3, PROTOCOL 18), the
/// same shape [`tone`] has one surface along: the engine states the word and
/// this table says what colour that word IS, so no screen decides.
///
/// The four readings are `docs/STYLE.md` §3's own sentences. A step still
/// being written is *a step in flight*, which is `Working`; one an interrupt
/// cut is *an interrupt*, which is `Annotation` — high salience, and neither
/// working nor failed; one that failed is `Error`; and a complete step is done
/// and so `Rest`. Telling the first two apart is the whole of what PROTOCOL 18
/// bought here (yog bl-ab53): before `in_flight`, every live step wore the one
/// word that makes a real interrupt legible.
pub fn framing(framing: Framing) -> Rgb {
    accent(match framing {
        Framing::Complete => State::Rest,
        Framing::Failed => State::Error,
        Framing::Killed => State::Annotation,
        Framing::InFlight => State::Working,
    })
}

/// **A speaker's rule.** Who said a row is told by the WEIGHT of the rule
/// beside it, not by a hue of its own, because hue means state: the
/// operator's own words carry the brand, the model's a full-ink rule, a peer
/// agent's a weak one, and an ended agent's the faintest.
pub fn speaker(role: Role) -> Rgb {
    match role {
        Role::User => BRAND,
        Role::Model => INK,
        Role::Peer => INK_WEAK,
        Role::Ended => INK_FAINT,
    }
}

/// The spacing scale, in points. Every gap on the glass is one of these.
pub mod space {
    pub const XS: f32 = 4.0;
    pub const S: f32 = 8.0;
    pub const M: f32 = 12.0;
    pub const L: f32 = 16.0;
    pub const XL: f32 = 24.0;
}

/// The type scale, in points: four sizes and no fifth.
pub mod type_scale {
    pub const SMALL: f32 = 12.0;
    pub const BODY: f32 = 15.0;
    pub const HEADING: f32 = 20.0;
    pub const MONO: f32 = 13.0;
}

/// The touch floor: every row and every control stands at least this tall
/// (DESIGN §13.2). Points, which are density-independent on this stack.
pub const TOUCH: f32 = 48.0;

/// The corner a tinted block is rounded by, in points.
pub const RADIUS: u8 = 10;

/// The width of a speaker's rule beside a transcript row.
pub const RULE: f32 = 3.0;

#[cfg(test)]
mod tests;
