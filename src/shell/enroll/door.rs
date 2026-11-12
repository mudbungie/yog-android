//! **The registration door** (yog REMOTE §15.1 steps 2 to 4; DESIGN §5): one
//! field for the one-time registration URL, the button that claims it, the
//! camera beside it, and the outcome under them — the confirmation word, and
//! where to approve it.
//!
//! **It replaced the enroll envelope's paste field** (bl-dd7b, bl-d815). That
//! field held a private key while it was full; this one holds a URL that
//! carries no key and works once. The camera is unchanged and still only a
//! faster way to fill one text field: a symbol that reads is put in the field
//! and claimed by the same path a paste is.
//!
//! **The page dials nothing itself.** A tap hands the URL to a worker thread,
//! which makes the key, claims the offer through `dev.yog.Register` — the
//! platform's HTTPS client, verifying against the platform's public web roots
//! — files the material and then dials the wire until the server's answer
//! changes (`crate::register::Watcher`). What it finds comes back over a
//! channel this page drains each frame. Dropping the page drops the channel,
//! and the worker ends at its next send.
//!
//! Everything it decides is `crate::register`, pure and under the floor: what
//! a registration URL is, the request's body, every refusal, the word, and
//! when a wait is over. What is left here is the JNI call and which widget
//! shows which of them — android-only, excluded from coverage for
//! `camera.rs`' reason exactly.

use std::path::PathBuf;
use std::sync::mpsc::{Receiver, channel};

use eframe::egui;

use super::{Landing, Scanner};
use crate::register::{Registered, Watch};
use crate::shell::app::REGISTER;

mod work;

use work::{Told, work};

/// What the page says above the field.
const ABOUT: &str = "Joins this device to a server. An operator on a seat that is already \
     connected mints an offer for it; scan the symbol that offer shows, or paste its \
     registration URL. This device makes its own key, and the key never leaves it.";

/// What the empty field shows.
const HINT: &str = "https://<host>/yog/register/<code>";

/// Where the one act stands.
enum Phase {
    /// Nothing asked yet, or the last ask refused.
    Idle,
    /// A claim is out and has not come back.
    Claiming,
    /// The offer is claimed; the latest dial's finding, once there is one.
    Claimed(Registered, Option<Watch>),
}

/// The page's state.
pub(crate) struct Door {
    /// What is typed, pasted or scanned into the field.
    pub(crate) url: String,
    /// The last refusal: the claim's, or the camera's.
    pub(super) said: Option<String>,
    phase: Phase,
    told: Option<Receiver<Told>>,
}

impl Door {
    pub(crate) fn new() -> Self {
        Self {
            url: String::new(),
            said: None,
            phase: Phase::Idle,
            told: None,
        }
    }

    /// Whether an offer was claimed here — the material on disk changed, so
    /// whoever closes this page re-reads it.
    pub(crate) fn claimed(&self) -> bool {
        matches!(self.phase, Phase::Claimed(..))
    }

    /// Fire the claim for what the field holds.
    fn fire(&mut self, dir: &str, ctx: &egui::Context) {
        let (tell, told) = channel();
        work(
            PathBuf::from(dir),
            self.url.trim().to_owned(),
            tell,
            ctx.clone(),
        );
        self.told = Some(told);
        self.said = None;
        self.phase = Phase::Claiming;
    }

    /// Take what the worker found since the last frame. Answers whether the
    /// server stopped refusing this device.
    fn drain(&mut self) -> bool {
        let mut approved = false;
        let heard: Vec<Told> = self.told.iter().flat_map(Receiver::try_iter).collect();
        for told in heard {
            match (told, &mut self.phase) {
                (Told::Claimed(Ok(registered)), _) => {
                    // The URL is spent.
                    self.url.clear();
                    self.phase = Phase::Claimed(registered, None);
                }
                (Told::Claimed(Err(refusal)), _) => {
                    self.said = Some(refusal);
                    self.phase = Phase::Idle;
                }
                (Told::Watched(watch), Phase::Claimed(_, latest)) => {
                    approved = watch == Watch::Approved;
                    *latest = Some(watch);
                }
                (Told::Watched(_), _) => {}
            }
        }
        approved
    }
}

/// The page. Answers whether the material should be re-read: the server
/// approved this device, or the operator asked for a look at what a cable
/// carried.
pub(super) fn screen(
    ui: &mut egui::Ui,
    landing: &Landing,
    door: &mut Door,
    scanner: &mut Scanner,
) -> bool {
    let approved = door.drain();
    if let Phase::Claimed(registered, watch) = &door.phase {
        outcome(ui, registered, watch.as_ref());
        return approved;
    }
    crate::shell::clip::weak(ui, ABOUT);
    ui.add_space(8.0);
    ui.add(
        egui::TextEdit::singleline(&mut door.url)
            .id(egui::Id::new(REGISTER.id))
            .desired_width(f32::INFINITY)
            .hint_text(HINT),
    );
    // **What registering costs, said before the control that spends it**
    // (bl-f12d). One device holds one leaf (§9): a registration REPLACES the
    // identity running here, and the old key is destroyed.
    if let Some(running) = &landing.replacing {
        ui.add_space(4.0);
        crate::shell::clip::tinted(
            ui,
            crate::shell::theme::ink(crate::theme::State::Annotation),
            format!(
                "this device is {running}. Registering replaces that identity: its \
                 certificate and private key are overwritten in place the moment an offer \
                 is claimed, and the old key is destroyed.",
            ),
        );
    }
    ui.add_space(4.0);
    let claiming = matches!(door.phase, Phase::Claiming);
    let ready = !door.url.trim().is_empty() && !claiming;
    ui.horizontal(|ui| {
        if crate::shell::theme::button_if(
            ui,
            ready,
            "register",
            0.0,
            "claims the offer at that URL with a key made on this device, files the \
             material, and shows the confirmation word — a URL works once",
        )
        .clicked()
        {
            door.fire(&landing.dir, ui.ctx());
        }
        if crate::shell::theme::button_if(
            ui,
            !claiming,
            "scan QR",
            0.0,
            "opens the camera to read the registration URL off a screen",
        )
        .clicked()
        {
            door.said = None;
            scanner.open();
        }
    });
    if claiming {
        crate::shell::clip::weak(ui, "claiming the offer…");
    }
    if let Some(why) = &door.said {
        crate::shell::clip::tinted(
            ui,
            crate::shell::theme::ink(crate::theme::State::Error),
            why,
        );
    }
    ui.add_space(12.0);
    ui.separator();
    carried(ui, landing)
}

/// **The scan screen, and what it produces** (bl-d815): a symbol that reads
/// fills the field and is claimed at once, by the field's own path.
pub(super) fn scanned(ui: &mut egui::Ui, dir: &str, door: &mut Door, scanner: &mut Scanner) {
    if let Some(found) = scanner.run(ui, &mut door.said) {
        door.url = found;
        door.fire(dir, ui.ctx());
    }
}

/// What a claimed offer shows: who this device now is, the word — the one
/// thing the operator reads across to another screen, so the one thing
/// painted large — where to approve it, and how the wait stands.
fn outcome(ui: &mut egui::Ui, registered: &Registered, watch: Option<&Watch>) {
    crate::shell::clip::said(ui, registered.said());
    ui.add_space(12.0);
    word(ui, &registered.word);
    ui.add_space(12.0);
    crate::shell::clip::said(ui, registered.approve());
    ui.add_space(8.0);
    let line = watch.map_or_else(|| "dialling the server…".to_owned(), Watch::said);
    crate::shell::clip::tinted(
        ui,
        crate::shell::theme::ink(crate::theme::State::Annotation),
        line,
    );
}

/// The confirmation word, as every screen that says it paints it: the one
/// thing an operator reads across to another screen.
pub(super) fn word(ui: &mut egui::Ui, word: &str) {
    ui.weak(crate::register::WORD_LINE);
    crate::shell::clip::copyable(
        ui,
        egui::Label::new(egui::RichText::new(word).monospace().size(44.0).strong()),
        word,
    );
}

/// **The leaf carried here by hand** (REMOTE §15.4: `yog wire-certs
/// WIRE_LEAF` remains for a device with no route to the door): the four
/// files, where they go, and the control that looks for them again.
fn carried(ui: &mut egui::Ui, landing: &Landing) -> bool {
    crate::shell::clip::weak(
        ui,
        "or, for a device with no route to that URL, a leaf minted on the server's own box \
         and carried here by cable:",
    );
    for file in crate::material::WANTED {
        crate::shell::clip::said(ui, format!("  · {file}"));
    }
    crate::shell::clip::said(ui, format!("  at {}", landing.dir));
    if let Some(why) = &landing.refusal {
        crate::shell::clip::tinted(
            ui,
            crate::shell::theme::ink(crate::theme::State::Error),
            why,
        );
    }
    ui.add_space(4.0);
    // A read of this app's own storage, not a dial: it is the act that makes a
    // cable push land without killing and relaunching the app.
    crate::shell::theme::button(
        ui,
        "check for material",
        0.0,
        "looks again for material a cable pushed into this app",
    )
    .clicked()
}
