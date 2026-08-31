//! **The three components, and the bootstrap each is gated behind** (yog
//! bl-15bd, operator ruling 2026-08-30).
//!
//! The ruling: this app is named yog and ships all three runnable components —
//! the seat, the foot and the server — *each gated behind an explicit
//! bootstrap rather than auto-started*. The default path is mTLS client
//! enrolment; running the server on the phone is allowed but is the
//! deliberate, non-default choice.
//!
//! **The component is derived, never stored, and that is the whole design.**
//! There is no chosen-mode setting, no first-run flag and nothing to keep in
//! sync, because the question *"which component is this device?"* is already
//! answered by what an operator provisioned onto it:
//!
//! - **No material** — nothing runs. That is the gate: an unbootstrapped yog
//!   is inert by construction, not by a check, and the first-run surface is
//!   the three [`offers`] rather than a component that started itself.
//! - **A leaf** — REMOTE §4.2 puts the grade *on the certificate*, so the leaf
//!   the operator issued already says which component this is: `OU=foot` is
//!   the foot, and everything else is a seat. Reading it is
//!   [`crate::leaf::grade`].
//!
//! A stored choice would have been a second authority for one fact, and the
//! two would disagree the first time an operator replaced a seat's leaf with a
//! foot's — which is exactly the act §4.2 says is how a grade changes:
//! *"making a foot into an operator is minting a new certificate, which is
//! exactly the friction that ruling wants."*
//!
//! **Nothing here enrols anything.** REMOTE §1.4 stands, and DESIGN §5
//! restates it: the new device never enrols over its own unauthenticated
//! connection. What an offer describes is where material goes when it arrives
//! through existing trust — a cable, an authenticated tool route, or a screen
//! the operator photographed — and this module only ever *reads* the result.

use std::path::Path;

use crate::leaf::Grade;
use crate::material::Material;

/// The three runnable components this app ships.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Component {
    /// The seat: asks the standing questions, paints replies, dispatches
    /// gestures. Operator-grade leaf (REMOTE §4.2).
    Seat,
    /// The foot: the thrall's surface — advertise, ride the invocations read,
    /// complete. A foot-grade leaf, and nothing else may be said on it.
    Foot,
    /// The server: holder of the world, the balls, the conversations. It holds
    /// no leaf of its own on this box, so [`standing`] never answers it —
    /// bootstrapping it is founding a world, which is bl-d6c6's question.
    Server,
}

impl Component {
    /// The word this component wears at the glass.
    pub fn word(self) -> String {
        match self {
            Self::Seat => "seat",
            Self::Foot => "tool host",
            Self::Server => "server",
        }
        .to_owned()
    }
}

/// What a provisioned device already is: the component its leaf enrols it as,
/// the client identity that leaf carries, and the material both connections
/// ride.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Enrolment {
    pub component: Component,
    /// The leaf's subject common name — REMOTE §2's *"one certificate = one
    /// client identity"*, and the name the engine's registry knows this
    /// device by. Empty when the certificate carries none, which is a leaf the
    /// engine will refuse; saying so at the glass beats a silent empty roster.
    pub client: String,
    pub material: Material,
}

/// What this device is, read off `dir`.
///
/// The three answers are [`crate::material::read_dir`]'s three, one layer up:
/// `Ok(None)` is cold, `Err` is a half-provisioned trust store named in full,
/// and provisioned material is read for the grade it carries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Standing {
    /// Nothing is provisioned. The app runs no component and offers three.
    Cold,
    /// A leaf is here, and it says which component this device is.
    Enrolled(Enrolment),
}

/// Read this device's standing out of its material directory.
pub fn standing(dir: &Path) -> Result<Standing, String> {
    let Some(material) = crate::material::read_dir(dir)? else {
        return Ok(Standing::Cold);
    };
    let der = first_certificate(&material)?;
    Ok(Standing::Enrolled(Enrolment {
        component: match crate::leaf::grade(&der) {
            Grade::Foot => Component::Foot,
            Grade::Operator => Component::Seat,
        },
        client: crate::leaf::common_name(&der).unwrap_or_default(),
        material,
    }))
}

/// The leaf's own DER. **The first certificate is the leaf**: a chain is
/// written end-entity first and toward the anchor after it, so reading any
/// other one would answer the issuing CA's grade for every device on the box.
fn first_certificate(material: &Material) -> Result<Vec<u8>, String> {
    use rustls::pki_types::CertificateDer;
    use rustls::pki_types::pem::PemObject;
    let path = &material.chain;
    let leaf = CertificateDer::pem_file_iter(path)
        .map_err(|e| format!("{}: {e}", path.display()))?
        .next()
        .ok_or_else(|| format!("{}: no certificate in it", path.display()))?
        .map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(leaf.as_ref().to_vec())
}

/// One bootstrap the first-run surface offers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Offer {
    pub component: Component,
    /// What taking this makes this device.
    pub title: String,
    /// The act, in the operator's own terms. Every one of them happens
    /// **outside this app**, which is the point rather than a limitation.
    pub how: String,
    /// Whether this is the ruling's default path. Exactly the two enrolments
    /// carry it: the server is *"allowed but … the deliberate, non-default
    /// choice."*
    pub default: bool,
}

/// The three offers, each naming the act that takes it. `dir` is where
/// material lands, and it is named in full because it is the single most
/// useful fact on a first-run screen — an operator with a cable in their hand
/// needs the path, not a description of one.
pub fn offers(dir: &Path) -> Vec<Offer> {
    let at = dir.display();
    vec![
        Offer {
            component: Component::Seat,
            title: "enrol as a seat".to_owned(),
            how: format!(
                "Put an operator-grade leaf at {at}: ca.pem, client.pem, \
                 client.key and an address file holding one host:port. \
                 The engine's own box mints it; a cable, an already-trusted \
                 device's tools, or a screen you photographed carries it \
                 here. This app never mints and never enrols itself."
            ),
            default: true,
        },
        Offer {
            component: Component::Foot,
            title: "enrol as a tool host".to_owned(),
            how: format!(
                "The same four files at {at}, on a leaf minted with OU=foot. \
                 A foot advertises what this machine can run, waits for work \
                 addressed to it, and hands back what happened — and may say \
                 nothing else about the world, which is why it is the right \
                 grade for a phone."
            ),
            default: true,
        },
        Offer {
            component: Component::Server,
            title: "run the engine here".to_owned(),
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

#[cfg(test)]
mod tests;
