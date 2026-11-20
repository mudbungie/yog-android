//! The window insets, asked of the platform the way a Java app would
//! (DESIGN §3 findings 2 and 3). SDK 35 is forced edge-to-edge:
//! `windowSoftInputMode=adjustResize` is ignored, `content_rect()` never
//! tracks the keyboard, and content otherwise draws under whatever the
//! platform put over the window — the status bar, the navigation bar on
//! whichever edge this orientation gives it, the display cutout, the
//! keyboard — where taps never reach the app. So the shell asks for all of
//! it, on all four sides, from `decorView.getRootWindowInsets()` over JNI,
//! and `app::pass` hands the answer to egui as its safe area
//! (`place::safe`).
//!
//! Two traps inside these lines, both learned the hard way (bl-8d03): the
//! Activity jobject comes from `sys::activity`, never `ndk_context`; and a
//! failed lookup leaves a pending Java exception that turns the NEXT JNI
//! call into a `CheckJNI` abort — clear it on every error path.

use winit::platform::android::activity::AndroidApp;

use super::place::safe::Px;

/// One walk: one union of every inset type, a handful of JNI crossings. The caller
/// throttles (app.rs, 200ms) — at frame cadence this would be hundreds of
/// crossings a second for numbers that change only when the keyboard slides.
pub(crate) fn probe(app: &AndroidApp) -> Result<Px, String> {
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

fn walk(env: &mut jni::JNIEnv, activity: &jni::objects::JObject) -> Result<Px, String> {
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
        return Ok(Px::default());
    }
    // **One mask, one answer** (bl-f557): the platform's own union of the
    // keyboard, the system bars and the display cutout is the per-side
    // maximum of the three, which is exactly the area nothing tappable may
    // be painted in. It used to be two reads keeping only `top` and
    // `bottom`, so a navigation bar standing on a SIDE — landscape, three
    // buttons — was never asked about at all.
    let mut mask = 0;
    for ty in ["ime", "systemBars", "displayCutout"] {
        mask |= env
            .call_static_method("android/view/WindowInsets$Type", ty, "()I", &[])
            .and_then(jni::objects::JValueGen::i)
            .map_err(|e| e.to_string())?;
    }
    let insets = env
        .call_method(
            &root,
            "getInsets",
            "(I)Landroid/graphics/Insets;",
            &[jni::objects::JValue::Int(mask)],
        )
        .and_then(jni::objects::JValueGen::l)
        .map_err(|e| e.to_string())?;
    let mut side = |name: &str| {
        env.get_field(&insets, name, "I")
            .and_then(jni::objects::JValueGen::i)
            .map_err(|e| e.to_string())
    };
    Ok(Px {
        left: side("left")?,
        top: side("top")?,
        right: side("right")?,
        bottom: side("bottom")?,
    })
}
