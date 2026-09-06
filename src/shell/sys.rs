//! The crate's ONE `unsafe` location (AGENTS.md rule 3, relaxed from
//! `forbid` under bl-c761; `rules/unsafe-outside-sys.yml` is the
//! enforcement, and its `ignores` list names exactly this file). Five raw
//! effects live here, all at the process edge, and their soundness arguments
//! are the file's:
//!
//! * **`android_main`** — the entry android-activity's native glue resolves
//!   by symbol name once the Activity is up. `unsafe(no_mangle)` asserts the
//!   unmangled symbol collides with nothing: this crate is the process's
//!   only Rust, and the glue declares exactly this name.
//! * **`Java_dev_yog_Watch_probe`** — the scheduled fetch's entry (DESIGN
//!   §17), resolved by the JNI's own name mangling rather than by a
//!   registration call. The same collision argument holds, and the name is
//!   not free-form: it is `Java_` plus the class's package path plus the
//!   method, which is what makes it unique by construction. It is the one
//!   entry the app has that no Activity is behind — the platform starts this
//!   process for a job, and everything the run needs is the string it is
//!   handed.
//! * **`Java_dev_yog_Pocket_standing`** — the pocketed foot's entry (DESIGN
//!   §18), the second of the two Java-calls-Rust doors and there for the same
//!   reason: a foreground service may be asking while no Activity is in
//!   front. The `Java_`-plus-package-plus-method name makes the symbol unique
//!   by construction exactly as the fetch's does.
//! * **`Java_dev_yog_Lane_attending` and `Java_dev_yog_Lane_wake`** — the
//!   held attention lane's two doors (DESIGN §17.6), the same direction and
//!   the same argument at a third and fourth site: the service asks whether
//!   there is a lane to hold, and then parks in it. `wake` BLOCKS for up to a
//!   hold, which is what a lane is, and it is called from a thread the
//!   service made for exactly that.
//! * **`Java_dev_yog_Pocket_serve`** — the boot-started foot's door (DESIGN
//!   §18.8). Two raw effects inside one call: `ndk_context`'s globals filled
//!   with this process's own VM and Application, which nothing else fills in a
//!   process no Activity created, and the global reference that publication
//!   requires be leaked. Both arguments are checked, both outlive every use,
//!   and the write happens only where nobody has written — the argument is
//!   `handed`'s own.
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

/// **The scheduled fetch, called from the platform's job** (DESIGN §17;
/// `dev.yog.Watch`). The direction is Java-calls-Rust and not the bridges'
/// Rust-calls-Java, because a job may start this process with no Activity
/// ever created — `ndk_context`'s globals are filled by android-activity on
/// the way to [`android_main`], so a bridge asking the JVM for a class would
/// be reading a handle nothing had written.
///
/// Two lines out, the answer protocol this crate already speaks: the title,
/// then the line under it. An empty string is silence, which is every failure
/// and every run that found nothing new — the decision is
/// [`crate::attention::sweep`]'s and is tested on the host.
#[unsafe(no_mangle)]
extern "system" fn Java_dev_yog_Watch_probe(
    mut env: jni::JNIEnv<'_>,
    _class: jni::objects::JClass<'_>,
    dir: jni::objects::JString<'_>,
) -> jni::sys::jstring {
    let files: String = env.get_string(&dir).map(Into::into).unwrap_or_default();
    let said = crate::attention::sweep(std::path::Path::new(&files))
        .map(|notice| format!("{}\n{}", notice.title, notice.text))
        .unwrap_or_default();
    env.new_string(said)
        .map_or(std::ptr::null_mut(), jni::objects::JString::into_raw)
}

/// **The pocketed foot's standing line, called from the foreground service**
/// (DESIGN §18; `dev.yog.Pocket`). Java-calls-Rust for the fetch's reason: the
/// service is asking about the process, not about a screen, and it may be
/// asking while nothing is in front.
///
/// The same two-line protocol, and the same meaning for an empty answer —
/// **nothing here to hold**, which is the service's whole stop condition. The
/// decision is [`crate::pocket::line`]'s and is tested on the host; the state
/// it reads is the process's one host ([`crate::state::standing`]).
#[unsafe(no_mangle)]
extern "system" fn Java_dev_yog_Pocket_standing(
    mut env: jni::JNIEnv<'_>,
    _class: jni::objects::JClass<'_>,
    dir: jni::objects::JString<'_>,
) -> jni::sys::jstring {
    let files: String = env.get_string(&dir).map(Into::into).unwrap_or_default();
    let said = crate::pocket::line(std::path::Path::new(&files), crate::state::standing())
        .map(|notice| format!("{}\n{}", notice.title, notice.text))
        .unwrap_or_default();
    env.new_string(said)
        .map_or(std::ptr::null_mut(), jni::objects::JString::into_raw)
}

/// **Whether there is an attention lane to hold** (DESIGN §17.6; yog REMOTE
/// §14 rung 2), asked by the same service. The two-line protocol again, and
/// an empty answer means what it means everywhere here: nothing to hold.
///
/// The decision is [`crate::pocket::attending`]'s — is this device a seat —
/// and the two operator gates beside it are Android's own facts, read in
/// `dev.yog.Pocket` where the platform keeps them.
#[unsafe(no_mangle)]
extern "system" fn Java_dev_yog_Lane_attending(
    mut env: jni::JNIEnv<'_>,
    _class: jni::objects::JClass<'_>,
    dir: jni::objects::JString<'_>,
) -> jni::sys::jstring {
    let files: String = env.get_string(&dir).map(Into::into).unwrap_or_default();
    let said = crate::pocket::attending(std::path::Path::new(&files))
        .map(|notice| format!("{}\n{}", notice.title, notice.text))
        .unwrap_or_default();
    env.new_string(said)
        .map_or(std::ptr::null_mut(), jni::objects::JString::into_raw)
}

/// **One life of the held attention lane** (DESIGN §17.6): dial, hold, and
/// answer the first rise the engine writes. It BLOCKS — up to the engine's own
/// hold — which is what a held read is, and the caller is a thread the service
/// made to park in it.
///
/// The same two-line answer, and an empty one is silence: the hold ended with
/// nothing new, or nothing this end could use. The decision is
/// [`crate::attention::wake`]'s and is tested on the host against a real
/// server.
#[unsafe(no_mangle)]
extern "system" fn Java_dev_yog_Lane_wake(
    mut env: jni::JNIEnv<'_>,
    _class: jni::objects::JClass<'_>,
    dir: jni::objects::JString<'_>,
) -> jni::sys::jstring {
    let files: String = env.get_string(&dir).map(Into::into).unwrap_or_default();
    let said = crate::attention::wake(std::path::Path::new(&files))
        .map(|notice| format!("{}\n{}", notice.title, notice.text))
        .unwrap_or_default();
    env.new_string(said)
        .map_or(std::ptr::null_mut(), jni::objects::JString::into_raw)
}

/// **Take the tool host up in a process no Activity ever created** (DESIGN
/// §18.8; `dev.yog.Pocket`). The third Java-calls-Rust door and the one that
/// makes a boot-started foot possible at all.
///
/// **It fills `ndk_context` first, because nothing else will.** Those globals
/// are android-activity's and are written on the way to [`android_main`], so a
/// service-started process has none and every bridge answers *no JVM is
/// attached to this process*. The two values they hold are both things a
/// Service already has — the process `JavaVM`, and the **Application**, which
/// is exactly the object `jvm::Bridge::open` asks for a class loader — so
/// filling them here is a hand-over rather than an invention, and every bridge
/// under it works unchanged.
///
/// **The latch is ours, because asking `ndk_context` what it holds PANICS when
/// it holds nothing** — which is exactly the state this door exists for.
/// `android_context()` unwraps an `Option`, so the guard *"only write it if it
/// is empty"* aborts the process before it can decide: measured as
/// `panic_cannot_unwind` out of this symbol and a `SIGABRT` two seconds into a
/// boot-started process, with the platform then backing the service off for
/// half an hour. So the door remembers, once per process, that it has handed
/// over.
///
/// **Writing it a second time would be harmless anyway.** An Activity that starts later
/// re-arms this service, and android-activity writes the same two values on
/// the way to `android_main`; taking the null check makes this a fill rather
/// than a race, and leaves the Activity's own hand-over the one that happens
/// when there IS one.
///
/// Answers the sentence a host that would not open failed with, or an empty
/// string — the two-line protocol's silence — when it took or was already
/// held. `crate::state::hold` refuses a second live host, so calling this on
/// every start is idempotent by the slot rather than by a flag here.
#[unsafe(no_mangle)]
extern "system" fn Java_dev_yog_Pocket_serve(
    mut env: jni::JNIEnv<'_>,
    _class: jni::objects::JClass<'_>,
    dir: jni::objects::JString<'_>,
    context: jni::objects::JObject<'_>,
) -> jni::sys::jstring {
    let said = if crate::state::holding() {
        // A host is already up — the ordinary answer on every start after the
        // first. Building one to be refused would dial and advertise before
        // the slot could say no, which is the question §18.1 made unaskable.
        String::new()
    } else if let Err(why) = handed(&mut env, &context) {
        why
    } else {
        let files: String = env.get_string(&dir).map(Into::into).unwrap_or_default();
        match crate::pocket::footed(std::path::Path::new(&files)) {
            Err(why) => why,
            Ok(foot) => {
                super::boot::take(foot, files);
                String::new()
            }
        }
    };
    env.new_string(said)
        .map_or(std::ptr::null_mut(), jni::objects::JString::into_raw)
}

/// The hand-over itself: this process's VM and Application into
/// `ndk_context`'s globals, once, and only where nobody has written them.
///
/// **The global reference is deliberately leaked.** `ndk_context` holds a raw
/// `jobject` for the life of the process and does not own it, so a `GlobalRef`
/// dropped at the end of this call would delete the very reference it just
/// published. It is one reference per process and it is never released,
/// exactly as android-activity's own is.
fn handed(env: &mut jni::JNIEnv<'_>, context: &jni::objects::JObject<'_>) -> Result<(), String> {
    static HANDED: std::sync::OnceLock<()> = std::sync::OnceLock::new();
    if HANDED.set(()).is_err() {
        return Ok(());
    }
    let vm = env.get_java_vm().map_err(|e| e.to_string())?;
    let held = env.new_global_ref(context).map_err(|e| e.to_string())?;
    let object = held.as_raw();
    std::mem::forget(held);
    // SAFETY: both handles are this process's own and outlive every use — the
    // VM lives as long as the process and the reference above is never
    // released. The write is on the service's main thread, which is the same
    // looper android-activity's own write runs on, so the two cannot race;
    // and the null check above means only one of them ever writes at all.
    let handles = (vm.get_java_vm_pointer().cast(), object.cast());
    unsafe {
        ndk_context::initialize_android_context(handles.0, handles.1);
    }
    Ok(())
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
