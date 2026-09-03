//! **The pocketed foot** (DESIGN §18, bl-8bd0): whether this device holds its
//! tool lane open while nobody is looking at it, and what the standing
//! notification that holds it says.
//!
//! **The whole decision is here and the platform half holds none of it.**
//! `dev.yog.Pocket` is a foreground service that starts when this answers with
//! a line and stops when it answers with none; every question it could
//! otherwise have decided for itself — is this device enrolled as hands, is
//! there a lane to hold, what does the operator read while it is held — is
//! answered by [`line`], which is pure over its two arguments and is tested at
//! the coverage floor.
//!
//! **Off by default, and the switch is the material** (§16.1's consent gate 1,
//! and §9's bootstrap discipline: *"the component is derived, never stored"*).
//! A foot-grade leaf is the operator's explicit enrollment of this device AS
//! hands — REMOTE §4.2 puts the grade on the certificate — so a device carrying
//! one holds its lane pocketed and every other device does not. A seat phone is
//! a seat with hands beside it, which is the shape the ball calls *fine*: its
//! host lives while the app does, exactly as before this rung.
//!
//! **An in-app toggle was considered and refused**, for §16.1's own reason: it
//! would be a second authority beside the fact the certificate already states,
//! and the two would disagree the first time an operator replaced a leaf
//! without visiting the switch. It would also need to be stored, and a stored
//! want is the second home §9 refused for the component itself.
//!
//! **The price is stated in the two places the operator reads it**: the
//! notification channel's description, in system settings, where a standing
//! cost belongs (§17.3's precedent), and the notification's own text, which is
//! the surface a pocketed phone actually shows. Neither is decorative — §14.2
//! prices this rung as a permanent notification and radio wakes, and a
//! capability that hides its price is the decoy shape this corpus refuses.

use std::path::Path;

use crate::attention::{Notice, WIRE};
use crate::bootstrap::{Component, Standing as Enrolment};
use crate::host::{Health, Standing};

/// **What the pocketed foot shows, or nothing at all.**
///
/// `files` is this app's private files directory, handed in by the platform
/// exactly as the scheduled fetch is handed it (§17.4). `standing` is what the
/// process's host stands at — [`crate::state::standing`] — passed rather than
/// read so that this decision has no global in it and every branch below is
/// reachable from a test.
///
/// **`None` means exactly one thing — this device is not hands** — and it is
/// the service's whole stop condition, so it must mean nothing else. A device
/// that IS hands and holds no lane still answers, because the two ways that
/// happens both want saying rather than silence: the moment between the
/// service arming and `android_main` taking its host up, and a device whose
/// material names a foot but will not build one. A service that stopped on the
/// first would race the boot it is holding open; one that stopped on the second
/// would take the only sentence about it away.
pub fn line(files: &Path, standing: Option<Standing>) -> Option<Notice> {
    if !hands(files) {
        return None;
    }
    Some(standing.as_ref().map_or_else(idle, notice))
}

/// Hands with no lane. It names the act that answers both of its causes, and
/// it never claims to be serving.
fn idle() -> Notice {
    Notice {
        title: "this phone is not serving".to_owned(),
        text: "yog is enrolled as hands and no tool lane is up. Open yog to see why.".to_owned(),
    }
}

/// Whether this device's leaf enrols it as hands. Anything a material read
/// cannot answer — cold, half-provisioned, a certificate that will not
/// parse — is *not hands*: the direction to fail in is the one that spends no
/// battery on a device nobody asked to enrol.
fn hands(files: &Path) -> bool {
    matches!(
        crate::bootstrap::standing(&files.join(WIRE)),
        Ok(Enrolment::Enrolled(enrolled)) if enrolled.component == Component::Foot
    )
}

/// **What the shade says while the lane is held.**
///
/// One title per state of [`Health`], because the three are what an operator
/// glancing at a lock screen needs told apart, and the line under it carries
/// the detail plus what that state is spending. The vocabulary is the roster's
/// own (`shell::screens`): one fact, two surfaces, and a phone that says
/// `reconnecting` on the glass must not say `serving` in the shade.
fn notice(standing: &Standing) -> Notice {
    let (title, mut text) = match (&standing.health, &standing.last) {
        (Health::Stopped(why), _) => (
            "this phone has stopped serving".to_owned(),
            format!("{why}. Nothing is on the network now — open yog to start again."),
        ),
        (Health::Redialling(why), _) => (
            "this phone is reconnecting".to_owned(),
            format!(
                "{why}. No tool call reaches it until the connection returns; \
                 yog keeps trying, more slowly each time."
            ),
        ),
        (Health::Serving, _) if !standing.advertised => (
            "this phone is offering its tools".to_owned(),
            format!(
                "presenting {} tools to the engine. A connection stays open while this \
                 stands, and the radio wakes with it.",
                standing.tools.len()
            ),
        ),
        (Health::Serving, last) => (
            "this phone is standing by as hands".to_owned(),
            format!(
                "{} tools offered · {}. A connection stays open while this stands, \
                 and the radio wakes with it.",
                standing.tools.len(),
                match last {
                    None => "nothing called yet".to_owned(),
                    Some(last) => format!("served {} · {last}", standing.served),
                }
            ),
        ),
    };
    // **A disarming that healed itself still has to reach somebody** (REMOTE
    // §5.1, bl-cc54). The roster paints it, and a pocketed phone's roster is
    // not being looked at — so the one surface it has is this one. The words
    // are `host::RESTORED`'s, said once wherever they are said.
    if standing.restored > 0 {
        use std::fmt::Write;
        let _ = write!(text, " {} (×{})", crate::host::RESTORED, standing.restored);
    }
    Notice { title, text }
}

#[cfg(test)]
mod tests;
