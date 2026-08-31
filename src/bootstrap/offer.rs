//! **The three bootstraps as branded choices** (bl-0d3c, amending bl-7714).
//!
//! Each component wears a name an operator can say out loud — **Lernie** the
//! seat, **Thrall** the foot, **Yog** the server — a short line saying what
//! taking it makes this device, and the long form that is the screen behind
//! the tap.
//!
//! **The tap chooses nothing durable.** The operator ruling that put controls
//! here is narrow and worth stating exactly: REMOTE §1.4 forbids the app
//! *dialling unauthenticated*, and it never forbade a control. So a choice
//! here opens the flow that acquires the matching material; it does not store
//! a mode. The component that comes up is still read off the leaf on disk
//! ([`super::standing`]), which is why there is no field on this struct that
//! anything writes back.

use super::Component;

/// One bootstrap the first-run surface offers, as a tappable choice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Offer {
    pub component: Component,
    /// The name on the control — [`Component::brand`], carried here so the
    /// screen paints one struct rather than joining two.
    pub brand: String,
    /// What taking this makes this device, in one line under the brand.
    pub tagline: String,
    /// The screen behind the tap: what material is needed, where it goes, and
    /// how it gets there. Every act it names happens **outside this app**,
    /// which is the point rather than a limitation.
    pub how: String,
    /// Whether this is the ruling's default path. Exactly the two enrollments
    /// carry it: the server is *"allowed but … the deliberate, non-default
    /// choice."*
    pub default: bool,
}

/// The three offers, each naming the act that takes it.
///
/// **It takes no directory, and that is deliberate.** The path material lands
/// in is the shell's own fact — it comes off the boot standing and the screen
/// paints it beside [`crate::material::WANTED`], which is the reader's own
/// file list. An earlier shape folded both into this prose, so the enrollment
/// screen said the path twice and the file names twice; one fact, one place
/// to paint it.
pub fn offers() -> Vec<Offer> {
    vec![
        Offer {
            component: Component::Seat,
            brand: Component::Seat.brand(),
            tagline: "the seat — operate your conversations".to_owned(),
            how: "An operator-grade leaf: a subject with no OU=foot \
                  (REMOTE §4.2), plus the one host:port it dials. \
                  The engine's own box mints it; a cable, an already-trusted \
                  device's tools, or a screen you photographed carries it \
                  here. This app never mints and never enrolls itself."
                .to_owned(),
            default: true,
        },
        Offer {
            component: Component::Foot,
            brand: Component::Foot.brand(),
            tagline: "the foot — let conversations use this device's tools".to_owned(),
            how: "The same files, on a leaf minted with OU=foot. A foot \
                  advertises what this machine can run, waits for work \
                  addressed to it, and hands back what happened — and may say \
                  nothing else about the world, which is why it is the right \
                  grade for a phone. This app never mints and never enrolls \
                  itself."
                .to_owned(),
            default: true,
        },
        Offer {
            component: Component::Server,
            brand: Component::Server.brand(),
            tagline: "the server — run the engine here".to_owned(),
            how: "Not yet, and here is exactly why. The engine cross-compiles \
                  to this architecture and links — that rung is walked. Two \
                  are not. An engine founds its world with git, commits every \
                  workspace and keeps its tasks in a git repository, and \
                  Android ships no git. And the world seeds shell shims its \
                  own agents run, which land in this app's private storage — \
                  where Android refuses to execute anything, by policy, since \
                  API 29. Both are upstream shapes, not settings. A button \
                  that started an engine which refuses every act would be \
                  worse than this sentence."
                .to_owned(),
            default: false,
        },
    ]
}

/// **How material gets onto a device**, one line each: DESIGN §5's three
/// delivery channels, in the operator's own terms. They are the enrollment
/// screen's whole content below the file list, and they are shared by both
/// enrollment offers because the channels do not care which grade the leaf
/// carries.
pub fn channels() -> Vec<String> {
    vec![
        "a cable — adb push the files into this app's storage".to_owned(),
        "an already-trusted device — its tools write the files here".to_owned(),
        "a screen — the engine shows the material and this device reads it".to_owned(),
    ]
}
