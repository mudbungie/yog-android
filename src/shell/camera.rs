//! The JNI half of the enrollment scanner: five static calls into
//! `dev.yog.Camera` (bl-d815).
//!
//! **The activity is an argument, not a global.** The camera's permission is
//! asked FOR an activity — `requestPermissions` puts a dialog in front of a
//! screen — and this bridge is only ever called from the frame loop, which
//! holds the `AndroidApp` handle. That is the opposite of the interface
//! bridge's rule (`crate::tools::ui::bridge`), and for the opposite reason:
//! a tool must work with no screen up, and a permission dialog cannot.
//!
//! The plumbing under it is `crate::shell::jvm`, shared with that bridge.
//! This file is android-only and excluded from coverage; the vocabulary it
//! hands back is parsed by [`crate::scan::state`], which is pure and is
//! tested.

use jni::objects::JValue;
use winit::platform::android::activity::AndroidApp;

use super::jvm::{Bridge, attached, broken};
use super::sys::activity;
use crate::scan::{Camera, state};

/// The class the static entry points live on, dotted for the class loader.
const CLASS: &str = "dev.yog.Camera";

/// What this bridge's failures are reported as.
const LABEL: &str = "camera bridge";

/// One `(Landroid/app/Activity;)Ljava/lang/String;` static.
const TAKES_ACTIVITY: &str = "(Landroid/app/Activity;)Ljava/lang/String;";

/// Whether this device can scan right now — the permission, with the camera's
/// own asynchronous failures folded in (see [`crate::scan::state`]).
pub(super) fn look(app: &AndroidApp) -> Camera {
    state(&with_activity(app, "state"))
}

/// Put the system's permission dialog up, once.
pub(super) fn ask(app: &AndroidApp) {
    drop(with_activity(app, "ask"));
}

/// Open the camera and start filling frames.
pub(super) fn start(app: &AndroidApp) {
    drop(with_activity(app, "start"));
}

/// The newest frame nobody has read yet, in [`crate::scan::read`]'s shape.
pub(super) fn frame() -> Option<Vec<u8>> {
    let mut env = attached().ok()?;
    let bridge = camera(&mut env).ok()?;
    bridge.bytes(&mut env, "frame", "()[B", &[])
}

/// Shut the camera down. Called on every way out of the scan screen — a
/// decode, a refusal, the cancel control, and the enrollment screen's own
/// back.
pub(super) fn stop() {
    let Ok(mut env) = attached() else {
        return;
    };
    if let Ok(bridge) = camera(&mut env) {
        drop(bridge.string(&mut env, "stop", "()Ljava/lang/String;", &[]));
    }
}

/// The shape three of the four activity calls share: this app's activity in,
/// one answer out.
fn with_activity(app: &AndroidApp, method: &str) -> String {
    let mut env = match attached() {
        Ok(env) => env,
        Err(why) => return broken(LABEL, &why),
    };
    let bridge = match camera(&mut env) {
        Ok(bridge) => bridge,
        Err(why) => return broken(LABEL, &why),
    };
    match activity(app) {
        Ok(object) => bridge.string(&mut env, method, TAKES_ACTIVITY, &[JValue::Object(&object)]),
        Err(why) => broken(LABEL, &why),
    }
}

/// This app's camera class, resolved once through this app's own class
/// loader. Once, because the scan screen polls twice a frame and the
/// resolution is three JNI calls that answer the same thing forever.
fn camera(env: &mut jni::JNIEnv) -> Result<Bridge, String> {
    static CAMERA: std::sync::OnceLock<Option<Bridge>> = std::sync::OnceLock::new();
    CAMERA
        .get_or_init(|| Bridge::open(env, CLASS, LABEL).ok())
        .clone()
        .ok_or_else(|| "this app's class loader did not yield the camera".to_owned())
}
