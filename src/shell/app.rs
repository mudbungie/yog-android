//! **What the shell is**: the owned state a frame paints from, and the
//! editable fields the IME mirror addresses. Every frame renders owned state
//! and blocks on nothing — the wire runs only on the model's worker thread.
//!
//! The per-frame pass itself is `app/pass.rs`, a child module so it can reach
//! these private fields. The seam is what the shell IS against what one frame
//! DOES with it.

mod fields;
mod pass;
mod probe;
mod running;

pub(crate) use fields::{COMPOSER, ENVELOPE, NEEDLE};
pub(crate) use pass::run;

use winit::platform::android::activity::AndroidApp;

use super::boot::{Running, boot};
use super::bridge::Bridge;
use super::enroll::Scanner;
use super::field::Native;
use super::inset::InsetPx;

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
    /// **Which overlay is floating over the chat** (DESIGN §22 item 6) — a
    /// world surface, the records or files depth, or none. One value, so one
    /// stands at a time; navigation and no more durable than a scroll
    /// position: the focus underneath is untouched, which is what makes
    /// closing one land where the operator was. `crate::overlay` decides
    /// whether a held one stands at the focus's depth.
    pub(crate) overlay: Option<crate::overlay::Overlay>,
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
    /// **Which config file the admin screen is showing** (§13.17), and which
    /// destination's bytes the editor was last seeded with. Two fields and not
    /// one: the pick is what a write is addressed at, and the seed is what
    /// keeps a re-read of the file being edited from discarding the draft.
    pub(crate) destination: Option<crate::codec::Destination>,
    pub(crate) seeded: Option<String>,
    /// **Which fork point the records screen's fork addresses** (§13.16) —
    /// the REF itself, an operable notch's whole commit or a `config/<name>`
    /// head, off the row that was tapped. The value picked is the value sent,
    /// so nothing looks one up again at fire time. `step`'s twin, and
    /// navigation for its reason exactly.
    pub(crate) from: Option<String>,
    /// **Which attempt the candidates screen's acts address** (§13.12): the
    /// project, the ball and the handle off the row that was tapped — the
    /// handle empty for the claim, which is what decides which controls are
    /// live. `ball`'s twin, and navigation for its reason exactly.
    pub(crate) candidate: Option<(String, String, String)>,
    /// **How many candidates a fan would spread** (§13.12). Floored at two by
    /// the control that moves it, because one and zero are a start and this
    /// app already has one.
    pub(crate) spread: usize,
    /// **How many drones a fleet would run at once** (§13.13). Floored at one
    /// by the control that moves it: a cap of zero is a loop that spawns
    /// nothing and still reaps, which upstream refuses to spell as a cap.
    pub(crate) cap: usize,
    /// **The open set** (DESIGN §23 item 2): which conversations were
    /// focused this run, what the tab strip paints. Per run, like a fold.
    pub(crate) tabs: crate::tabs::Tabs,
    /// **Whether the ask's `why` is open** (§22 item 8, §13.7, bl-8c94):
    /// the control's decision about the parked call, folded away so the box
    /// says one line about it. A fold, and no more durable than one.
    pub(crate) why: bool,
    /// **Which proposal the admin screen's settle addresses** (§13.17) — the
    /// id off the row that was tapped. `ball`'s twin, and navigation for its
    /// reason exactly.
    pub(crate) proposal: Option<String>,
    /// **Which step the records screen's one act addresses** (§13.11) — the
    /// sequence off the census row that was tapped. `ball`'s twin, and
    /// navigation for its reason exactly.
    pub(crate) step: Option<String>,
    pub(crate) composer: String,
    /// **What the verb sheet said when a line could not fire** (§22 item 5):
    /// a verb missing its words, or a line naming no verb. Stands on the
    /// rule's banner line until a fire replaces it, a tap puts it down, or
    /// the sheet closes.
    pub(crate) verb_said: Option<String>,
    /// **The composer's native field** (bl-8bbb, `shell::field`): the
    /// platform `EditText` overlaid at the rectangle the composer row lays
    /// out, and the standing this side keeps between frames. The DRAFT above
    /// is not a second home for the text — the field is the home, and this is
    /// what agrees the two after every frame.
    pub(crate) field: Native,
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
    /// **This device's own knobs** (DESIGN §23 item 11), read from
    /// `knobs_file` at boot and written back as each one moves; the
    /// transcript's policy is three of them.
    pub(crate) knobs: crate::settings::Settings,
    /// Where the knobs are kept: beside the cache, never the material.
    pub(crate) knobs_file: std::path::PathBuf,
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
    /// server has not yet shown back, in the composer's own state and nowhere
    /// near the projection — `crate::rows` is pure over what the server
    /// wrote down, and this is a message it has not written down yet. The
    /// type is `crate::outbox`'s and so is every decision about it: this file
    /// is excluded from the coverage floor, and the echo's states are a
    /// reading of the server's answers rather than a paint fact (bl-07b1).
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
    /// the guess and the guess goes — whether the server took the act or
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
    /// **What a dark tuning control said when it was tapped** (bl-0691): the
    /// sentence `codec::pick::knob` made about why the knob cannot be set,
    /// standing over the controls row until the next act replaces the picks.
    /// A viewport fact like the picks above it, and it travels with their
    /// reset — a reason about a provider the focus has left is not a reason
    /// about anything.
    pub(crate) tuning_said: Option<String>,
    /// **A platform back press this frame has not yet been taken** (bl-550e).
    /// Read once at the top of the pass and consumed by whatever has a depth
    /// to walk — the bar wherever it paints a back control, the scan screen
    /// because closing a camera is one depth up. Still standing at the end of
    /// the frame means nothing had a depth, which is where leaving the app
    /// belongs (`shell::back`). A frame-scoped fact, not state: it is
    /// rewritten every pass.
    pub(crate) back: bool,
    /// The switcher's `+ new` asked for the caret: taken by the next pass
    /// that places the field (§22 item 7).
    pub(crate) summon: bool,
    /// The status rule's between-frames state (`shell/status.rs`).
    pub(crate) rule: super::status::Rule,
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
        let knobs_file = super::boot::settings_file(&android);
        Self {
            running: boot(&android),
            chose: None,
            settings: false,
            overlay: None,
            armed: false,
            ball: None,
            step: None,
            why: false,
            proposal: None,
            from: None,
            destination: None,
            seeded: None,
            candidate: None,
            spread: crate::shell::screens::FLOOR,
            cap: crate::shell::screens::CAP,
            tabs: crate::tabs::Tabs::default(),
            scanner: Scanner::new(android.clone()),
            android,
            bridge: Bridge::default(),
            composer: String::new(),
            verb_said: None,
            field: Native::default(),
            needle: String::new(),
            envelope: String::new(),
            envelope_said: None,
            knobs: crate::settings::read(&knobs_file),
            knobs_file,
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
            tuning_said: None,
            tuned_at: 0,
            back: false,
            summon: false,
            rule: super::status::Rule::default(),
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
}
