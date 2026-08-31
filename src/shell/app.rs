//! **What the shell is**: the owned state a frame paints from, and the
//! editable fields the IME mirror addresses. Every frame renders owned state
//! and blocks on nothing — the wire runs only on the model's worker thread.
//!
//! The per-frame pass itself is `app/pass.rs`, a child module so it can reach
//! these private fields. The seam is what the shell IS against what one frame
//! DOES with it.

mod pass;

pub(crate) use pass::run;

use winit::platform::android::activity::AndroidApp;

use super::boot::{Running, boot};
use super::bridge::{Bridge, Field, FieldKind};
use super::enroll::Scanner;
use super::inset::InsetPx;
use crate::host::Host;
use crate::rows::AutoExpand;
use crate::seat::Model;

/// The one editable field the shell carries. The id string is the egui
/// widget id AND the bridge's address for it — one definition, used twice.
pub(crate) const COMPOSER: Field = Field {
    id: "composer",
    kind: FieldKind::Composer,
};

/// The enrollment screen's envelope field. A separate id because the two are
/// never on screen together for the composer's reason inverted: a cold device
/// has no conversation to speak into, and a provisioned one has nothing left
/// to enroll — but they are different KINDS of editor (`bridge.rs`), and the
/// IME must be told which it is focused on.
pub(crate) const ENVELOPE: Field = Field {
    id: "envelope",
    kind: FieldKind::Envelope,
};

pub(crate) struct Shell {
    android: AndroidApp,
    bridge: Bridge,
    pub(crate) running: Running,
    /// Which first-run screen is open, when nothing is provisioned. It is
    /// navigation and nothing else — no more durable than a scroll position,
    /// and deliberately not a chosen mode: the component this device runs is
    /// derived from the leaf on disk every boot (`crate::bootstrap`), so a
    /// stored choice would be a second authority for one fact (DESIGN §9).
    pub(crate) chose: Option<crate::bootstrap::Component>,
    pub(crate) composer: String,
    /// The pasted enroll envelope, and the last thing reading it said. It
    /// holds a PRIVATE KEY while it is full, so it is emptied the moment it
    /// has been landed and on the way back out of the screen
    /// (`forget_envelope`), and nothing logs it.
    pub(crate) envelope: String,
    pub(crate) envelope_said: Option<String>,
    /// The enrollment screen's camera (bl-d815). It lives beside the field it
    /// fills rather than inside it, because the camera outlives any one frame
    /// and the field does not: an open camera must be closed on the way out
    /// of the screen, whichever way out was taken.
    pub(crate) scanner: Scanner,
    /// Which KINDS of row open by default (the desktop's two knobs).
    pub(crate) auto: AutoExpand,
    /// The rows the operator has flipped by hand — overrides, never states:
    /// membership FLIPS a row's auto-state, so an empty set is "everything as
    /// configured" and the knobs above keep meaning what they say.
    pub(crate) folds: std::collections::BTreeSet<String>,
    t0: std::time::Instant,
    /// The inset pads and when they were last probed — the JNI walk is
    /// throttled to 200ms for numbers that change only when the keyboard
    /// slides (bl-014e).
    pub(crate) inset: InsetPx,
    inset_at: u128,
}

impl Shell {
    fn new(android: AndroidApp) -> Self {
        Self {
            running: boot(&android),
            chose: None,
            scanner: Scanner::new(android.clone()),
            android,
            bridge: Bridge::default(),
            composer: String::new(),
            envelope: String::new(),
            envelope_said: None,
            auto: AutoExpand::default(),
            folds: std::collections::BTreeSet::new(),
            t0: std::time::Instant::now(),
            inset: InsetPx::default(),
            inset_at: 0,
        }
    }

    /// Drop the pasted envelope and whatever reading it said. Called on the
    /// way out of the enrollment screen and the instant material lands: the
    /// text is a private key, and a buffer that outlives its use is the one
    /// place this app holds key material it was not handed a file for.
    pub(crate) fn forget_envelope(&mut self) {
        self.envelope = String::new();
        self.envelope_said = None;
        self.chose = None;
        // An open camera is a running capture session and a background
        // thread; leaving the screen is the last moment anything here knows
        // to close them.
        self.scanner.shut();
    }

    /// Re-read what is provisioned and start whatever it now names. A read of
    /// this app's own storage — never a dial — and the act that makes material
    /// pushed over a cable land without relaunching the process.
    pub(crate) fn reboot(&mut self) {
        self.running = boot(&self.android);
        self.chose = None;
    }

    /// The seat model, when this launch is running one.
    pub(crate) fn model(&self) -> Option<&Model> {
        match &self.running {
            Running::Seat { model, .. } => Some(model),
            _ => None,
        }
    }

    pub(crate) fn model_mut(&mut self) -> Option<&mut Model> {
        match &mut self.running {
            Running::Seat { model, .. } => Some(model),
            _ => None,
        }
    }

    /// Who this device is on the wire, and as what: the leaf's own common
    /// name and the component its grade enrolled it as (REMOTE §2, §4.2).
    /// Painted rather than logged, because a seat showing an empty roster and
    /// a seat registered in no workspace look identical until this line says
    /// which client the engine was answering.
    pub(crate) fn identity(&self) -> String {
        match &self.running {
            Running::Seat { client, .. } => format!("{client} · seat"),
            Running::Foot { client, .. } => format!("{client} · foot grade"),
            Running::Cold { .. } => String::new(),
        }
    }

    /// The tool host, whichever component holds one.
    pub(crate) fn host_mut(&mut self) -> Option<&mut Host> {
        match &mut self.running {
            Running::Seat { host, .. } => host.as_mut(),
            Running::Foot { host, .. } => Some(host),
            Running::Cold { .. } => None,
        }
    }
}
