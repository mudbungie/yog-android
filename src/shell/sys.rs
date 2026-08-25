//! The crate's ONE `unsafe` location (AGENTS.md rule 3, relaxed from
//! `forbid` under bl-c761; `rules/unsafe-outside-sys.yml` is the
//! enforcement, and its `ignores` list names exactly this file). Three raw
//! effects live here, all at the process edge, and their soundness arguments
//! are the file's:
//!
//! * **`android_main`** — the entry android-activity's native glue resolves
//!   by symbol name once the Activity is up. `unsafe(no_mangle)` asserts the
//!   unmangled symbol collides with nothing: this crate is the process's
//!   only Rust, and the glue declares exactly this name.
//! * **the `WGPU_BACKEND` fold** — `std::env::set_var` is unsafe in edition
//!   2024 because a concurrent `getenv` is UB. Here it runs first, on the
//!   main thread, before eframe boots and before any thread this process
//!   will ever spawn exists — no concurrent reader is possible. Why an env
//!   var at all: DESIGN §3 finding 1 — the Imagination-GPU Pixel's Vulkan
//!   driver segfaults inside `vkCreateGraphicsPipelines` while egui-wgpu
//!   builds its pipeline, eframe's default `InstanceDescriptor` honors this
//!   variable, and the spike proved this exact fold on glass (bl-8d03).
//! * **the JNI handle conversions** — android-activity 0.6 documents
//!   `vm_as_ptr()` as the process `JavaVM*` and `activity_as_ptr()` as the
//!   Activity `jobject` (NOT the `ndk_context` context, which holds the
//!   Application — its `getWindow()` lookup throws, and the pending
//!   exception aborts under `CheckJNI`; bl-8d03). Both are checked non-null,
//!   both outlive every use (the VM and the Activity reference live as long
//!   as the process's native code), and `JObject` does not delete the
//!   reference on drop.

use winit::platform::android::activity::AndroidApp;

/// The process entry. Everything after the env fold is safe code in
/// `app::run`; nothing else in the crate runs before this.
#[unsafe(no_mangle)]
extern "Rust" fn android_main(app: AndroidApp) {
    // SAFETY: main thread, pre-eframe, pre-any-thread — no concurrent
    // getenv exists yet (the whole argument is this file's doc header).
    unsafe { std::env::set_var("WGPU_BACKEND", "gles") };
    super::app::run(app);
}

/// The process `JavaVM`, off the handle android-activity carries.
pub(crate) fn vm(app: &AndroidApp) -> Result<jni::JavaVM, String> {
    let raw = app.vm_as_ptr();
    if raw.is_null() {
        return Err("null JavaVM".to_owned());
    }
    vm_from(raw)
}

/// The same conversion from a raw pointer another holder of it already
/// checked — `ndk_context`'s global, which is what the tool-host worker has
/// (it holds no activity handle, and a tool whose availability tracked the UI
/// would be the wrong shape). The cast lives here rather than there because
/// this file is the crate's one location for raw handle conversions.
pub(crate) fn vm_from(raw: *mut std::ffi::c_void) -> Result<jni::JavaVM, String> {
    if raw.is_null() {
        return Err("null JavaVM".to_owned());
    }
    // SAFETY: both callers pass a pointer their own source documents as this
    // process's JavaVM* — android-activity's `vm_as_ptr`, and `ndk_context`'s
    // global, which android-activity itself fills before `android_main` — and
    // both are checked non-null above. The VM outlives all native code.
    unsafe { jni::JavaVM::from_raw(raw.cast()) }.map_err(|e| e.to_string())
}

/// Any JNI object this process was handed as a raw pointer — the
/// application object `ndk_context` carries, which the interface bridge needs
/// to reach this app's own class loader. Null-checked by the caller, because
/// what a null means differs there.
pub(crate) fn object_from(raw: *mut std::ffi::c_void) -> jni::objects::JObject<'static> {
    // SAFETY: the pointer is `ndk_context`'s application object, filled by
    // android-activity before `android_main` and live for the process; a
    // `JObject` does not delete the reference on drop.
    unsafe { jni::objects::JObject::from_raw(raw.cast()) }
}

/// The Activity as a JNI object.
pub(crate) fn activity(app: &AndroidApp) -> Result<jni::objects::JObject<'static>, String> {
    let raw = app.activity_as_ptr();
    if raw.is_null() {
        return Err("null activity".to_owned());
    }
    // SAFETY: `activity_as_ptr()` IS the Activity jobject — a reference
    // android-activity holds for the app's lifetime, checked non-null above;
    // `JObject` does not delete it on drop.
    Ok(unsafe { jni::objects::JObject::from_raw(raw.cast()) })
}
