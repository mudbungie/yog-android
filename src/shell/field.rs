//! **The native composer field's JNI half** (bl-8bbb): three static calls
//! into `dev.yog.Field`, and the standing this side keeps between frames.
//!
//! The composer is a platform `EditText` overlaid at the rectangle egui lays
//! out for it (DESIGN §13.2), because an egui `TextEdit` fed by the
//! `GameTextInput` mirror can only adopt a committed buffer wholesale — which
//! is why a letter typed mid-draft landed at the end, nothing was selectable,
//! and there was no paste menu. **The field's own text and cursor are the one
//! home of the draft**; this file is the wire to it.
//!
//! **The activity is an argument, not a global**, for `shell::camera`'s
//! reason exactly: a view belongs to a window, and every call here is made
//! from the frame loop, which holds the `AndroidApp` handle.
//!
//! This file is android-only and excluded from coverage: it IS the JNI call
//! and the posting of a rectangle. Everything it DECIDES is `crate::draft`,
//! which is pure and asserted by host tests — how the bridge's answer reads,
//! which side of the mirror moved, and what band the field's own measurement
//! earns.

use jni::objects::JValue;
use winit::platform::android::activity::AndroidApp;

use super::jvm::{Bridge, attached, broken};
use super::sys::activity;
use crate::draft::{Mirror, Move, band, read};

/// The class the static entry points live on, dotted for the class loader.
const CLASS: &str = "dev.yog.Field";

/// What this bridge's failures are reported as.
const LABEL: &str = "composer field";

/// The language, as the platform takes it: colours packed ARGB and every
/// length already in device pixels, plus the scale that produced them. Built
/// at the paint site by `shell::theme`, which is this crate's one adapter
/// from `crate::theme` to a face (`rules/no-literal-colour.yml`).
#[derive(Clone, Copy)]
pub(crate) struct Skin {
    pub(crate) ground: i32,
    pub(crate) ink: i32,
    pub(crate) faint: i32,
    pub(crate) ring: i32,
    pub(crate) radius: i32,
    pub(crate) pad_x: i32,
    pub(crate) pad_y: i32,
    pub(crate) text: i32,
    pub(crate) ring_px: i32,
    /// Device pixels per point, so the field's own measurement comes back
    /// into egui's units where the band is decided.
    pub(crate) scale: f32,
}

/// The field's standing between frames: what the two sides last agreed the
/// draft was, the band the field's own text earned, and whether it is on the
/// glass at all.
#[derive(Default)]
pub(crate) struct Native {
    mirror: Mirror,
    high: f32,
    /// Whether the view is showing right now, so a screen that paints no
    /// composer costs one crossing rather than one per frame.
    shown: bool,
    /// Whether THIS pass painted one. Frame-scoped like `Shell::back`: taken
    /// at the end of the pass, where a field nothing placed comes off the
    /// glass.
    placed: bool,
}

impl Native {
    /// The height the composer's band stands at, in points — the field's own
    /// last measurement, floored at the touch target and capped
    /// (`crate::draft::band`). Last frame's, because a view's height is not
    /// knowable before the platform has laid it out, and the resting height
    /// before the platform has laid one out at all.
    pub(crate) fn band(&self) -> f32 {
        self.high.max(band(0, 1.0))
    }

    /// One frame of the field: place it, read the draft back, and hand over
    /// whatever this side changed. Answers whether it holds the caret.
    pub(crate) fn frame(
        &mut self,
        app: &AndroidApp,
        hint: &str,
        at: [i32; 4],
        skin: Skin,
        draft: &mut String,
    ) -> bool {
        self.placed = true;
        self.shown = true;
        let said = match read(&place(app, hint, at, skin)) {
            Ok(said) => said,
            Err(why) => {
                // The refusal, never the answer: an answer carries the
                // operator's own prose, and logcat is device-wide (§15.2).
                log::warn!("composer field: {why}");
                return false;
            }
        };
        self.high = band(said.high, skin.scale);
        match self.mirror.step(&said.text, draft) {
            Move::Adopt(text) => *draft = text,
            Move::Push(text) => give(app, &text),
            Move::Still => {}
        }
        said.focused
    }

    /// A send took the draft: clear the field, and record that this side did
    /// — otherwise the next frame reads the emptiness as the FIELD having
    /// been cleared and adopts it back over whatever replaced the draft.
    pub(crate) fn took(&mut self, app: &AndroidApp) {
        give(app, "");
        self.mirror.handed("");
    }

    /// End of the pass: a field this frame did not place comes off the glass,
    /// keeping its text — the draft outlives a screen change.
    pub(crate) fn rest(&mut self, app: &AndroidApp) {
        if std::mem::take(&mut self.placed) || !self.shown {
            return;
        }
        self.shown = false;
        hide(app);
    }
}

/// Place, dress and read the field in one crossing.
fn place(app: &AndroidApp, hint: &str, at: [i32; 4], skin: Skin) -> String {
    let mut env = match attached() {
        Ok(env) => env,
        Err(why) => return broken(LABEL, &why),
    };
    let bridge = match field(&mut env) {
        Ok(bridge) => bridge,
        Err(why) => return broken(LABEL, &why),
    };
    let object = match activity(app) {
        Ok(object) => object,
        Err(why) => return broken(LABEL, &why),
    };
    let said = match env.new_string(hint) {
        Ok(said) => said,
        Err(e) => return broken(LABEL, &e.to_string()),
    };
    let [left, top, wide, high] = at;
    bridge.string(
        &mut env,
        "place",
        "(Landroid/app/Activity;Ljava/lang/String;IIIIIIIIIIIII)Ljava/lang/String;",
        &[
            JValue::Object(&object),
            JValue::Object(said.as_ref()),
            JValue::Int(left),
            JValue::Int(top),
            JValue::Int(wide),
            JValue::Int(high),
            JValue::Int(skin.ground),
            JValue::Int(skin.ink),
            JValue::Int(skin.faint),
            JValue::Int(skin.ring),
            JValue::Int(skin.radius),
            JValue::Int(skin.pad_x),
            JValue::Int(skin.pad_y),
            JValue::Int(skin.text),
            JValue::Int(skin.ring_px),
        ],
    )
}

/// Hand the field a draft this side wrote. Its answer says only that the post
/// was made — the field's own next report is the confirmation, and this call
/// has no other outcome to act on.
fn give(app: &AndroidApp, text: &str) {
    let Ok(mut env) = attached() else {
        return;
    };
    let (Ok(bridge), Ok(object)) = (field(&mut env), activity(app)) else {
        return;
    };
    let Ok(said) = env.new_string(text) else {
        return;
    };
    drop(bridge.string(
        &mut env,
        "give",
        "(Landroid/app/Activity;Ljava/lang/String;)Ljava/lang/String;",
        &[JValue::Object(&object), JValue::Object(said.as_ref())],
    ));
}

/// Take the field off the glass, keeping its text.
fn hide(app: &AndroidApp) {
    let Ok(mut env) = attached() else {
        return;
    };
    let (Ok(bridge), Ok(object)) = (field(&mut env), activity(app)) else {
        return;
    };
    drop(bridge.string(
        &mut env,
        "hide",
        "(Landroid/app/Activity;)Ljava/lang/String;",
        &[JValue::Object(&object)],
    ));
}

/// This app's field class, resolved once through this app's own class loader.
/// Once, because the composer places itself every frame and the resolution is
/// three JNI calls that answer the same thing forever.
fn field(env: &mut jni::JNIEnv) -> Result<Bridge, String> {
    static FIELD: std::sync::OnceLock<Option<Bridge>> = std::sync::OnceLock::new();
    FIELD
        .get_or_init(|| Bridge::open(env, CLASS, LABEL).ok())
        .clone()
        .ok_or_else(|| "this app's class loader did not yield the field".to_owned())
}
