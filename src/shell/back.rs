//! **The platform's own back control** (bl-550e): the gesture every Android
//! thumb reaches for, wired to mean exactly what the bar's `<` means — one
//! focus depth up (DESIGN §13.2).
//!
//! **Why it was inert, traced rather than guessed.** The chain is entirely in
//! the stack this app already links:
//!
//! 1. The platform dispatches `KEYCODE_BACK` to the activity as an ordinary
//!    key event (no `enableOnBackInvokedCallback` is declared, so the
//!    predictive-back dispatcher is not in play).
//! 2. `GameActivity.onKeyDown` hands it to `onKeyDownNative` and returns
//!    TRUE when that does — so the platform's own default (leave the app)
//!    never runs.
//! 3. The native glue's `onKey` enqueues every key its filter allows into the
//!    input buffer and answers true; the filter drops only volume, camera and
//!    zoom. Back is enqueued.
//! 4. winit's android backend maps `Keycode::Back` to
//!    `NamedKey::BrowserBack`, and `egui-winit` maps that to
//!    [`egui::Key::BrowserBack`].
//!
//! So the gesture was arriving all along and nothing was reading it: a key
//! consumed by the glue and dropped by the app is exactly the inert control
//! the operator met. This file is the read, and the one act the app must
//! perform itself because step 2 already spent the platform's default.
//!
//! **Two halves, one rule.** The press is taken by whatever has a depth to
//! walk — the bar consumes it wherever it paints a back control, and the scan
//! screen consumes it because closing a camera IS one depth up from it. A
//! press nothing consumed means there was no depth left, and *that* is where
//! leaving the app belongs. No screen is enumerated anywhere.

use winit::platform::android::activity::AndroidApp;

use eframe::egui;

/// Whether the platform's back control was pressed this frame. Consumed on
/// read, so the frame that takes it is the only one that sees it.
pub(super) fn pressed(ctx: &egui::Context) -> bool {
    ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::BrowserBack))
}

/// Leave the app — what back means when nothing had a depth to walk. It is
/// the platform's own default performed by hand, because `GameActivity`
/// answered the key before the platform could.
pub(super) fn leave(app: &AndroidApp) {
    if let Err(why) = finish(app) {
        // logcat is the only witness, and there is nothing to paint: the
        // screen the operator is looking at is still correct.
        log::warn!("back: {why}");
    }
}

/// `Activity.finish()`, with `inset`'s discipline: the handle comes from
/// `sys::activity` and a failed call leaves a pending Java exception that
/// would turn the NEXT JNI call into a `CheckJNI` abort, so it is cleared on
/// the error path.
fn finish(app: &AndroidApp) -> Result<(), String> {
    let vm = super::sys::vm(app)?;
    let mut env = vm.attach_current_thread().map_err(|e| e.to_string())?;
    let activity = super::sys::activity(app)?;
    let called = env.call_method(&activity, "finish", "()V", &[]);
    if called.is_err() {
        let _ = env.exception_clear();
    }
    called.map(|_| ()).map_err(|e| e.to_string())
}
