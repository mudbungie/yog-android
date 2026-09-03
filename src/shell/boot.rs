//! **The bootstrap gate** (yog bl-15bd): read what an operator provisioned
//! onto this device, and start exactly the component it names. Nothing starts
//! itself, and there is nothing to choose at the glass — the choice was made
//! when the material arrived, which is the act §1.4 requires anyway.
//!
//! This file is the seam between *what is running* and *the frame loop that
//! paints it* (`app.rs`). The reasoning for deriving the component rather than
//! storing it lives in `crate::bootstrap`, which is host-tested; everything
//! here is the Android-side wiring that spends it.

use std::time::Duration;

use winit::platform::android::activity::AndroidApp;

use crate::bootstrap::{Component, Offer, Standing, offers, standing};
use crate::foot::Foot;
use crate::seat::Model;

/// How long the model rests between unprompted refreshes — the human
/// cadence of a chat glanced at, not a terminal streamed to.
const CADENCE: Duration = Duration::from_secs(2);

/// **What this launch is running** (yog bl-15bd): one of the three
/// components, or none of them and the surface that offers all three. The
/// choice is derived from the material on disk, never stored — see
/// [`crate::bootstrap`] for why a stored one would be a second authority.
pub(crate) enum Running {
    /// Nothing provisioned, or a store that will not read. The app started no
    /// component, which is the gate: an unbootstrapped yog is inert by
    /// construction rather than by a check.
    Cold {
        offers: Vec<Offer>,
        /// The half-provisioned sentence, when there is one. Absent is a
        /// genuinely empty device, and the two must not read the same.
        refusal: Option<String>,
        /// Where material goes, painted so an operator can act on it.
        dir: String,
    },
    /// The seat, with the tool host beside it — one identity, two
    /// connections (REMOTE §5's refcounted presence). The host itself is not
    /// a field of this variant: it belongs to the PROCESS (`crate::state`,
    /// DESIGN §18), because a pocketed foot outlives the activity that
    /// started it and an activity that is destroyed and created again must
    /// not build a second one.
    ///
    /// **The model is boxed** because this variant is the enum's largest by
    /// far — the handle carries a whole `Snapshot` of last-good rows
    /// (bl-de96) and the selectors' offerings (bl-0267) — and a cold device
    /// would otherwise carry that footprint to say "nothing is provisioned".
    Seat { model: Box<Model>, client: String },
    /// The foot alone. No standing questions and no chat screens: a foot-grade
    /// leaf may say `advertise`, `invocations` and `complete` and nothing
    /// else (REMOTE §4.2), so a seat loop here would earn a refusal per pass
    /// forever.
    Foot { client: String },
}

/// **The bootstrap gate**, run once at boot: read what is provisioned, and
/// start exactly the component it says. Nothing starts itself.
pub(crate) fn boot(android: &AndroidApp) -> Running {
    let dir = wire_dir(android);
    let cold = |refusal: Option<String>| Running::Cold {
        offers: offers(),
        refusal,
        dir: dir.display().to_string(),
    };
    let enrolled = match standing(&dir) {
        Err(why) => return cold(Some(why)),
        Ok(Standing::Cold) => return cold(None),
        Ok(Standing::Enrolled(enrolled)) => enrolled,
    };
    let material = &enrolled.material;
    match enrolled.component {
        // The foot holds ONE channel and it is the narrow one (bl-2040): the
        // three gestures §4.2 allows, and no seat this arm could accidentally
        // start.
        Component::Foot => match Foot::open(material) {
            Ok(foot) => {
                host(android, foot);
                Running::Foot {
                    client: enrolled.client,
                }
            }
            Err(why) => cold(Some(why)),
        },
        // A leaf that says nothing says seat, and the server holds no leaf on
        // this box at all — so `Server` is unreachable here and is served by
        // the arm that cannot be wrong about it.
        //
        // Two channels on one identity, which REMOTE §5's refcounted presence
        // expects: the seat's asker, and the tool host beside it on the same
        // foot surface. A host that will not open is simply absent — the
        // seat's own banner already says why a channel failed, and a second
        // copy of that sentence would be noise.
        Component::Seat | Component::Server => match crate::transport::Seat::open(material) {
            Ok(asker) => {
                if let Ok(foot) = Foot::open(material) {
                    host(android, foot);
                }
                Running::Seat {
                    model: Box::new(Model::start(asker, CADENCE, cache_file(android))),
                    client: enrolled.client,
                }
            }
            Err(why) => cold(Some(why)),
        },
    }
}

/// **Where the paint-first cache lives** (DESIGN §14, bl-de96): a SIBLING of
/// the material directory and never inside it. Material and snapshots share
/// no file and no directory — one is a private key an operator delivered out
/// of channel, the other is a picture of the world that is deletable by
/// definition.
fn cache_file(android: &AndroidApp) -> std::path::PathBuf {
    android
        .internal_data_path()
        .unwrap_or_default()
        .join("cache")
        .join("seat.json")
}

/// This app's private material directory (DESIGN §5: adb, remote exec or QR
/// put material there; this app never mints).
pub(super) fn wire_dir(android: &AndroidApp) -> std::path::PathBuf {
    android
        .internal_data_path()
        .unwrap_or_default()
        .join("wire")
}

/// **Take up the process's tool host over one connection**, unless it already
/// holds a live one (`crate::state::hold`, DESIGN §18). Called on every boot,
/// which is every activity creation — the second and later ones are the
/// relaunch case, and the slot refusing them is what keeps one certificate to
/// one parked `invocations` read (REMOTE §5.1).
///
/// The dispatch closes over this app's own storage, which is where a
/// screenshot goes when a caller names no path — the one directory this uid
/// can always write.
fn host(android: &AndroidApp, foot: Foot) {
    let data_dir = android
        .internal_data_path()
        .map(|p| p.display().to_string())
        .unwrap_or_default();
    crate::state::hold(crate::host::Host::start(
        foot,
        crate::tools::advertisement(),
        Box::new(move |tool, input| crate::tools::run_in(tool, input, &data_dir)),
        // The device's own rest between redials, which is the whole of what
        // the parameter is for: the suite hands the loop a recorder instead
        // and reads the ladder back without sleeping through it (bl-8641).
        Box::new(std::thread::sleep),
    ));
}
