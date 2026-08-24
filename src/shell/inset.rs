//! The window insets, asked of the platform the way a Java app would
//! (DESIGN §3 findings 2 and 3). SDK 35 is forced edge-to-edge:
//! `windowSoftInputMode=adjustResize` is ignored, `content_rect()` never
//! tracks the keyboard, content otherwise draws under the status bar, and a
//! flush-bottom widget sits in the gesture-nav zone where taps never reach
//! the app. So the shell pads all of it itself, from
//! `decorView.getRootWindowInsets()` over JNI.
//!
//! Two traps inside these lines, both learned the hard way (bl-8d03): the
//! Activity jobject comes from `sys::activity`, never `ndk_context`; and a
//! failed lookup leaves a pending Java exception that turns the NEXT JNI
//! call into a `CheckJNI` abort — clear it on every error path.

use winit::platform::android::activity::AndroidApp;

/// What the shell pads, in physical pixels.
#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct InsetPx {
    /// The band content must clear at the top (status bar).
    pub(crate) top: i32,
    /// The taller of the keyboard and the gesture-nav bar.
    pub(crate) bottom: i32,
}

/// One walk: ime + systemBars, a handful of JNI crossings. The caller
/// throttles (app.rs, 200ms) — at frame cadence this would be hundreds of
/// crossings a second for numbers that change only when the keyboard slides.
pub(crate) fn probe(app: &AndroidApp) -> Result<InsetPx, String> {
    let vm = super::sys::vm(app)?;
    let mut env = vm.attach_current_thread().map_err(|e| e.to_string())?;
    let activity = super::sys::activity(app)?;
    let walked = walk(&mut env, &activity);
    if walked.is_err() {
        // A failed lookup leaves a pending Java exception; the NEXT JNI call
        // with one pending is a CheckJNI abort, so clear it here, always.
        let _ = env.exception_clear();
    }
    walked
}

fn walk(env: &mut jni::JNIEnv, activity: &jni::objects::JObject) -> Result<InsetPx, String> {
    let window = env
        .call_method(activity, "getWindow", "()Landroid/view/Window;", &[])
        .and_then(jni::objects::JValueGen::l)
        .map_err(|e| e.to_string())?;
    let decor = env
        .call_method(&window, "getDecorView", "()Landroid/view/View;", &[])
        .and_then(jni::objects::JValueGen::l)
        .map_err(|e| e.to_string())?;
    let root = env
        .call_method(
            &decor,
            "getRootWindowInsets",
            "()Landroid/view/WindowInsets;",
            &[],
        )
        .and_then(jni::objects::JValueGen::l)
        .map_err(|e| e.to_string())?;
    if root.is_null() {
        // No insets attached yet (pre-layout); zero is the honest answer.
        return Ok(InsetPx::default());
    }
    let ime = insets_of(env, &root, "ime")?;
    let bars = insets_of(env, &root, "systemBars")?;
    Ok(InsetPx {
        top: bars.top,
        bottom: ime.bottom.max(bars.bottom),
    })
}

/// The `android.graphics.Insets` for one `WindowInsets.Type` mask, read as
/// this module's own pair.
fn insets_of(
    env: &mut jni::JNIEnv,
    root: &jni::objects::JObject,
    ty: &str,
) -> Result<InsetPx, String> {
    let mask = env
        .call_static_method("android/view/WindowInsets$Type", ty, "()I", &[])
        .and_then(jni::objects::JValueGen::i)
        .map_err(|e| e.to_string())?;
    let insets = env
        .call_method(
            root,
            "getInsets",
            "(I)Landroid/graphics/Insets;",
            &[jni::objects::JValue::Int(mask)],
        )
        .and_then(jni::objects::JValueGen::l)
        .map_err(|e| e.to_string())?;
    let top = env
        .get_field(&insets, "top", "I")
        .and_then(jni::objects::JValueGen::i)
        .map_err(|e| e.to_string())?;
    let bottom = env
        .get_field(&insets, "bottom", "I")
        .and_then(jni::objects::JValueGen::i)
        .map_err(|e| e.to_string())?;
    Ok(InsetPx { top, bottom })
}
