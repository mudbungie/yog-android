//! **The update row, and the three JNI calls under it** (bl-7a68, DESIGN
//! §20): the one affordance this app carries for its own release channel.
//!
//! **It is a row on the roster and never a modal.** The phone seat is a
//! working surface, and an update prompt in front of a live conversation is
//! exactly the interruption this suite's notice discipline exists to avoid.
//! So the offer sits beside the other structural things the roster already
//! says about this device — its identity, and what it is hosting — where an
//! operator meets it when they are between pieces of work rather than inside
//! one.
//!
//! **It carries no `act:` tag.** The parity gate judges the tags on this
//! app's controls against the engine's own op roster, and this control fires
//! no op: nothing crosses the wire, and the release feed is not a thing the
//! engine has ever heard of. A tag here would name an op that does not exist
//! and redden the gate correctly (`crate::parity`'s second assertion), which
//! is the gate telling the truth rather than a hole in it.
//!
//! **What this file decides is nothing.** `crate::update` holds the version
//! comparison, the asset pick and the transport requirement, under the 100%
//! floor; `dev.yog.Update` holds the socket and the installer intent. This is
//! the seam between them, and it is android-only and excluded from coverage
//! for `camera.rs`' reason exactly: it IS the JNI call.

use eframe::egui;
use jni::objects::JValue;

use super::app::Shell;
use super::jvm::{Bridge, attached, broken};

/// The class the static entry points live on, dotted for the class loader.
const CLASS: &str = "dev.yog.Update";

/// What this bridge's failures are reported as.
const LABEL: &str = "update bridge";

/// The `()Ljava/lang/String;` shape both reads share.
const READS: &str = "()Ljava/lang/String;";

impl Shell {
    /// The offer, when there is one — and the tap that hands it to the system
    /// installer.
    ///
    /// The check itself is started by the first call: `Update.state` fetches
    /// once per process, so painting the roster IS the cadence. That is the
    /// right one for a phone — there is no timer worth the battery, and
    /// opening the app is the only moment an offer could be seen anyway.
    pub(super) fn update_entry(&mut self, ui: &mut egui::Ui) {
        let Some(offer) = standing() else {
            return;
        };
        let label = format!("update to {}", offer.version);
        let control =
            egui::Button::new(label).min_size(egui::vec2(ui.available_width(), super::mark::TOUCH));
        let response = ui.add(control);
        self.note_control("update", ui, response.rect);
        if response.clicked() {
            FIRED.store(true, std::sync::atomic::Ordering::Relaxed);
            install(&offer.url);
        }
        // What the download said, when it has said anything. It is the Java
        // side's sentence and not this one's: the act that produces it runs
        // for seconds after the frame that fired it, so the place it can be
        // read from is the place it happens — and it is asked for only once
        // the row has been tapped, because before that there is nothing to
        // say and the ask would be a JNI attach and detach per frame for an
        // empty string (see [`standing`]).
        if FIRED.load(std::sync::atomic::Ordering::Relaxed) {
            let said = said();
            if !said.is_empty() {
                ui.weak(said);
            }
        }
    }
}

/// **The decision, taken once and then remembered.** The feed answers exactly
/// once per process, so what this resolves to cannot change afterwards — and
/// a roster frame that crossed the JNI boundary sixty times a second to
/// re-read a settled answer would be paying an attach and a detach per frame
/// for a string that is already known. So the shell asks until the feed has
/// spoken and never again.
///
/// A `OnceLock`, which is `camera.rs`' own idiom for a JNI answer that is the
/// same forever, and not a field on `Shell`: nothing about this is the frame's
/// state, and the paint site is a free function's reach away from the model.
fn standing() -> Option<crate::update::Offer> {
    static DECIDED: std::sync::OnceLock<Option<crate::update::Offer>> = std::sync::OnceLock::new();
    if let Some(decided) = DECIDED.get() {
        return decided.clone();
    }
    let answer = read("state");
    if !crate::update::answered(&answer) {
        return None;
    }
    let decided = crate::update::offer(&answer, env!("CARGO_PKG_VERSION"));
    drop(DECIDED.set(decided.clone()));
    decided
}

/// **Whether this launch has fired the install**, and therefore whether there
/// is anything for [`said`] to answer. A plain atomic and not a field on
/// `Shell`: it is the same shape as the `OnceLock` above — a fact about the
/// PROCESS that only ever moves one way — and the paint site is a free
/// function's reach away from the model either way.
static FIRED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// What the download last said, or empty. Not in the two-line protocol: it is
/// one sentence for a person, and there is no second thing to say about it.
fn said() -> String {
    read("said")
}

/// Fetch the APK at `url` and put the system installer in front of the
/// operator. The answer is dropped — the sentence arrives through [`said`],
/// because the download outlives this frame by seconds.
fn install(url: &str) {
    let Ok(mut env) = attached() else {
        return;
    };
    let Ok(bridge) = update(&mut env) else {
        return;
    };
    let Ok(argument) = env.new_string(url) else {
        return;
    };
    drop(bridge.string(
        &mut env,
        "install",
        "(Ljava/lang/String;)Ljava/lang/String;",
        &[JValue::Object(&argument)],
    ));
}

/// The shape both reads share: no arguments, one answer.
fn read(method: &str) -> String {
    let mut env = match attached() {
        Ok(env) => env,
        Err(why) => return broken(LABEL, &why),
    };
    match update(&mut env) {
        Ok(bridge) => bridge.string(&mut env, method, READS, &[]),
        Err(why) => broken(LABEL, &why),
    }
}

/// This app's update class, resolved once through this app's own class
/// loader. Once, because the roster polls it every frame and the resolution
/// is three JNI calls that answer the same thing forever — `camera.rs`' rule.
fn update(env: &mut jni::JNIEnv) -> Result<Bridge, String> {
    static UPDATE: std::sync::OnceLock<Option<Bridge>> = std::sync::OnceLock::new();
    UPDATE
        .get_or_init(|| Bridge::open(env, CLASS, LABEL).ok())
        .clone()
        .ok_or_else(|| "this app's class loader did not yield the updater".to_owned())
}
