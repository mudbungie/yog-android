//! **What the shell is**: the owned state a frame paints from, and the
//! editable fields the IME mirror addresses. Every frame renders owned state
//! and blocks on nothing — the wire runs only on the model's worker thread.
//!
//! The per-frame pass itself is `app/pass.rs`, a child module so it can reach
//! these private fields. The seam is what the shell IS against what one frame
//! DOES with it.

mod pass;
mod probe;

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
    /// Whether the configuration surface is open over a running component
    /// (bl-387f). Navigation like `chose` above, never a mode: a cold device
    /// paints the configuration regardless of this flag, and nothing about
    /// what runs is stored here.
    pub(crate) settings: bool,
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
    /// **The outbox** (bl-66fb): the message this seat has sent and the
    /// engine has not yet shown back, in the composer's own state and nowhere
    /// near the projection — `crate::rows` is pure over what the engine
    /// wrote down, and this is a message it has not written down yet.
    pub(crate) echo: Option<Echo>,
    /// **What the composer's selectors are pointed at** (bl-0267), and the
    /// workspace that owns the pointing. This device shows what it SET, never
    /// a guess at what is set — no wire shape states a workspace's current
    /// assignment — so these three are a viewport fact and go when the focus
    /// leaves the workspace they were made in.
    pub(crate) picked_in: Option<String>,
    pub(crate) provider: Option<String>,
    pub(crate) model: Option<String>,
    /// **The tuning this device set** (bl-dfbb): the effort level's own word
    /// (`off` included — it is a value, not an absence, once somebody has
    /// chosen it) and whether the priority lane was asked for. `None` is
    /// "this device has set none", which is not the same as `off`, and no
    /// shape on the wire states what the workspace is on — so these two show
    /// what was SET here and travel with `picked_in`'s reset.
    pub(crate) effort: Option<String>,
    pub(crate) priority: bool,
    /// **A platform back press this frame has not yet been taken** (bl-550e).
    /// Read once at the top of the pass and consumed by whatever has a depth
    /// to walk — the bar wherever it paints a back control, the scan screen
    /// because closing a camera is one depth up. Still standing at the end of
    /// the frame means nothing had a depth, which is where leaving the app
    /// belongs (`shell::back`). A frame-scoped fact, not state: it is
    /// rewritten every pass.
    pub(crate) back: bool,
    /// **What the render-and-see probe says about this pass** (`app/probe.rs`,
    /// bl-243b): the screen the dispatch chose, and where the mark was put in
    /// device pixels. Frame-scoped like `back` above — both are taken at the
    /// end of the pass, so a screen that stops painting stops saying it is
    /// there. `probed` is the last line said, and the reason a repaint is not
    /// news.
    pub(crate) screen: Option<&'static str>,
    pub(crate) mark_at: Option<[i32; 4]>,
    probed: String,
}

impl Shell {
    fn new(android: AndroidApp) -> Self {
        Self {
            running: boot(&android),
            chose: None,
            settings: false,
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
            echo: None,
            picked_in: None,
            provider: None,
            model: None,
            effort: None,
            priority: false,
            back: false,
            screen: None,
            mark_at: None,
            probed: String::new(),
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
        // A recheck is the configuration's exit: whatever the derivation now
        // says is the screen the operator asked to see.
        self.settings = false;
    }

    /// Where material goes — the same directory the boot derivation reads.
    /// The configuration surface paints it when it is opened over a running
    /// component, where no `Running::Cold` carries it along (bl-387f).
    pub(crate) fn material_dir(&self) -> String {
        super::boot::wire_dir(&self.android).display().to_string()
    }

    /// The seat model, when this launch is running one.
    pub(crate) fn model(&self) -> Option<&Model> {
        match &self.running {
            Running::Seat { model, .. } => Some(model.as_ref()),
            _ => None,
        }
    }

    pub(crate) fn model_mut(&mut self) -> Option<&mut Model> {
        match &mut self.running {
            Running::Seat { model, .. } => Some(model.as_mut()),
            _ => None,
        }
    }

    /// Who this device is on the wire, and as what: the leaf's own common
    /// name and the component its grade enrolled it as (REMOTE §2, §4.2).
    /// Painted rather than logged, because a seat showing an empty roster and
    /// a seat registered in no workspace look identical until this line says
    /// which client the engine was answering.
    pub(crate) fn identity(&self) -> String {
        use crate::bootstrap::Component;
        match &self.running {
            Running::Seat { client, .. } => format!("{client} · {}", Component::Seat.brand()),
            Running::Foot { client, .. } => format!("{client} · {}", Component::Foot.brand()),
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

/// **One message in flight from this seat** (bl-66fb) — the three states the
/// operator sees, held as the two facts they are made of.
///
/// It paints the instant it is sent, because that is the whole point: the
/// round trip is seconds and a chat app that shows nothing for seconds is a
/// chat app you press twice. What it cannot know by itself is whether the
/// engine took it, so it remembers the seat's deposit counters at the moment
/// it was sent and watches for either to move.
pub(crate) struct Echo {
    /// What was sent — the text, so it can be given back if the deposit is
    /// refused, and matched against the transcript when it comes home.
    pub(crate) text: String,
    /// Whether the engine's receipt has come back. Muted ink until it has,
    /// then ordinary ink with a rule under it until the message appears in a
    /// transcript read and this dissolves into that row.
    pub(crate) landed: bool,
    /// The conversation it was sent to. An echo is a message in ONE
    /// conversation, and painting it under another would be the same wrong
    /// claim every other pairing in this app refuses.
    pub(crate) agent: Option<String>,
    /// The deposit counters as they stood when this was sent.
    pub(crate) at: (usize, usize),
}
