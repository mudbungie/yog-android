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

/// **The search field** (§13.6, bl-4c2b). Its own id and its own kind: a
/// needle is one line and must not be autocorrected — a corrected needle
/// searches for a word the operator did not type — while the composer is
/// prose and wants both.
pub(crate) const NEEDLE: Field = Field {
    id: "needle",
    kind: FieldKind::Needle,
};

pub(crate) struct Shell {
    pub(super) android: AndroidApp,
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
    /// **Which world surface is open over the roster** (§13.8) — the queue,
    /// the trail, or neither. Navigation like `settings` above and no more
    /// durable than a scroll position: the focus underneath it is untouched,
    /// which is what makes backing out of one land where the operator was.
    pub(crate) opened: Option<super::screens::World>,
    /// **Whether the armed control on the open screen is armed** (§13.8,
    /// §13.9). Two controls take an arming now — the trail's truncation and
    /// the ball pane's `close` — and it is still one bool rather than a
    /// mechanism, because the two are never on screen together: what an arm
    /// IS here is *this screen's irreversible control has been tapped once*.
    /// It is cleared by leaving the screen, by the acts beside it, by picking
    /// another subject and by firing, because an arm nobody is looking at must
    /// not survive.
    pub(crate) armed: bool,
    /// **Which ball the pane's acts address** (§13.9, bl-f36e): the project
    /// and the id off the row that was tapped. Navigation like `opened` above
    /// and no more durable than a scroll position — the acts are addressed at
    /// a row, and this is which row.
    pub(crate) ball: Option<(String, String)>,
    /// **Whether the records screen is open over the transcript** (§13.11).
    /// Navigation like `opened` above and no more durable than a scroll
    /// position: the focus underneath it is untouched, which is what makes
    /// backing out of it land where the operator was.
    pub(crate) records: bool,
    /// **Which attempt the candidates screen's acts address** (§13.12): the
    /// project, the ball and the handle off the row that was tapped — the
    /// handle empty for the claim, which is what decides which controls are
    /// live. `ball`'s twin, and navigation for its reason exactly.
    pub(crate) candidate: Option<(String, String, String)>,
    /// **How many candidates a fan would spread** (§13.12). Floored at two by
    /// the control that moves it, because one and zero are a start and this
    /// app already has one.
    pub(crate) spread: usize,
    /// **Which step the records screen's one act addresses** (§13.11) — the
    /// sequence off the census row that was tapped. `ball`'s twin, and
    /// navigation for its reason exactly.
    pub(crate) step: Option<String>,
    pub(crate) composer: String,
    /// What the search field holds. A question, not an answer — the hits are
    /// the model's and ride the snapshot — and it is emptied when the search
    /// screen is left, because leaving a search leaves it.
    pub(crate) needle: String,
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
    /// How many `act:` tags the parity inventory has been written with
    /// (`shell/act.rs`). A count and not a copy of the set: the set only
    /// grows, so a length is the whole of "has it changed".
    pub(super) acted: usize,
    t0: std::time::Instant,
    /// The inset pads and when they were last probed — the JNI walk is
    /// throttled to 200ms for numbers that change only when the keyboard
    /// slides (bl-014e).
    pub(crate) inset: InsetPx,
    inset_at: u128,
    /// **The outbox** (bl-66fb): the message this seat has sent and the
    /// engine has not yet shown back, in the composer's own state and nowhere
    /// near the projection — `crate::rows` is pure over what the engine
    /// wrote down, and this is a message it has not written down yet. The
    /// type is `crate::outbox`'s and so is every decision about it: this file
    /// is excluded from the coverage floor, and the echo's states are a
    /// reading of the engine's answers rather than a paint fact (bl-07b1).
    pub(crate) echo: Option<crate::outbox::Echo>,
    /// **What the composer's selectors are pointed at** (bl-0267), and the
    /// workspace that owns the pointing. This device shows what it SET, never
    /// a guess at what is set — no wire shape states a workspace's current
    /// assignment — so these three are a viewport fact and go when the focus
    /// leaves the workspace they were made in.
    pub(crate) picked_in: Option<String>,
    pub(crate) provider: Option<String>,
    pub(crate) model: Option<String>,
    /// The assignments read at the moment an optimistic value was last set
    /// (bl-e9f9). When the seat's count moves past it, truth has overtaken
    /// the guess and the guess goes — whether the engine took the act or
    /// refused it, which is why the fallback is the read and not a memory.
    pub(crate) tuned_at: usize,
    /// **The tuning this device just set** (bl-dfbb), held only until the
    /// assignments read overtakes it (bl-e9f9). `None` is "nothing optimistic
    /// standing", and what the controls then show is what the workspace
    /// actually has. They travel with `picked_in`'s reset.
    pub(crate) effort: Option<String>,
    /// `Option`, not a bare bool: "nothing optimistic standing" and
    /// "optimistically set to off" are different states, and telling them
    /// apart is what keeps a toggle from snapping back for a cadence after
    /// it is turned off (bl-e9f9).
    pub(crate) priority: Option<bool>,
    /// **A platform back press this frame has not yet been taken** (bl-550e).
    /// Read once at the top of the pass and consumed by whatever has a depth
    /// to walk — the bar wherever it paints a back control, the scan screen
    /// because closing a camera is one depth up. Still standing at the end of
    /// the frame means nothing had a depth, which is where leaving the app
    /// belongs (`shell::back`). A frame-scoped fact, not state: it is
    /// rewritten every pass.
    pub(crate) back: bool,
    /// **What the render-and-see probe says about this pass** (`app/probe.rs`,
    /// bl-243b): the screen the dispatch chose, and where each control the
    /// harness has to reach was painted, in device pixels — the mark, the
    /// first conversation row, and the roster's two world entries (§15.2,
    /// §13.8). Frame-scoped like `back` above — taken at the end of the pass,
    /// so a screen that stops painting one stops saying where it is.
    /// `probed` is the last line said, and the reason a repaint is not news.
    pub(crate) screen: Option<&'static str>,
    pub(crate) at: Vec<(&'static str, [i32; 4])>,
    probed: String,
}

impl Shell {
    fn new(android: AndroidApp) -> Self {
        Self {
            running: boot(&android),
            chose: None,
            settings: false,
            opened: None,
            armed: false,
            ball: None,
            records: false,
            step: None,
            candidate: None,
            spread: crate::shell::screens::FLOOR,
            scanner: Scanner::new(android.clone()),
            android,
            bridge: Bridge::default(),
            composer: String::new(),
            needle: String::new(),
            envelope: String::new(),
            envelope_said: None,
            auto: AutoExpand::default(),
            folds: std::collections::BTreeSet::new(),
            acted: 0,
            t0: std::time::Instant::now(),
            inset: InsetPx::default(),
            inset_at: 0,
            echo: None,
            picked_in: None,
            provider: None,
            model: None,
            effort: None,
            priority: None,
            tuned_at: 0,
            back: false,
            screen: None,
            at: Vec::new(),
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
}
