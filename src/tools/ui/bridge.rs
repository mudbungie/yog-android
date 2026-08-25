//! The JNI half of the interface tools: six static calls into
//! `dev.yog.seat.YogAccessibilityService`, each answering one string.
//!
//! **The `JavaVM` comes from `ndk_context`, not from an `AndroidApp`.** This
//! runs on the tool-host worker, which holds no activity handle and must not
//! — a tool that only worked while a particular screen was up would be a tool
//! whose availability tracked the UI. `ndk_context`'s global is set by
//! android-activity before `android_main` is entered, so it is there for the
//! whole life of the process.
//!
//! **Every failure is a sentence in the answer, never a panic and never a
//! pending exception.** A Java exception left pending turns the next JNI call
//! into a `CheckJNI` abort (DESIGN §3's second trap), so every path that can
//! throw clears before returning — the same discipline the inset probe keeps.
//!
//! This file is android-only and excluded from coverage; what it hands back
//! is parsed by [`super::answer`], which is pure and is tested.

use jni::objects::{GlobalRef, JObject, JString, JValue};

/// The class the static entry points live on, in the form a class loader
/// takes (dots, not slashes — `FindClass` is not what resolves it; see
/// [`class`]).
const CLASS: &str = "dev.yog.seat.YogAccessibilityService";

pub(crate) fn bridge_read() -> String {
    call("uiRead", "()Ljava/lang/String;", &[])
}

pub(crate) fn bridge_tap(x: i32, y: i32) -> String {
    call(
        "uiTap",
        "(II)Ljava/lang/String;",
        &[JValue::Int(x), JValue::Int(y)],
    )
}

pub(crate) fn bridge_tap_text(text: &str) -> String {
    with_string(text, |env, arg| {
        call_in(
            env,
            "uiTapText",
            "(Ljava/lang/String;)Ljava/lang/String;",
            &[JValue::Object(arg)],
        )
    })
}

pub(crate) fn bridge_type(text: &str) -> String {
    with_string(text, |env, arg| {
        call_in(
            env,
            "uiText",
            "(Ljava/lang/String;)Ljava/lang/String;",
            &[JValue::Object(arg)],
        )
    })
}

pub(crate) fn bridge_key(key: &str) -> String {
    with_string(key, |env, arg| {
        call_in(
            env,
            "uiKey",
            "(Ljava/lang/String;)Ljava/lang/String;",
            &[JValue::Object(arg)],
        )
    })
}

pub(crate) fn bridge_shot(path: &str) -> String {
    with_string(path, |env, arg| {
        call_in(
            env,
            "screenshot",
            "(Ljava/lang/String;)Ljava/lang/String;",
            &[JValue::Object(arg)],
        )
    })
}

/// The refusal every bridge failure wears, in the two-line protocol so the
/// one parser reads it like any other answer.
fn broken(why: &str) -> String {
    format!("err\nthe interface bridge failed: {why}")
}

/// Attach, call a no-argument static, detach.
fn call(method: &str, signature: &str, args: &[JValue]) -> String {
    let Ok(mut env) = attached() else {
        return broken("no JVM is attached to this process");
    };
    call_in(&mut env, method, signature, args)
}

/// Attach and run `body` with one Java string built from `text` — the shape
/// every argument-taking entry point shares.
fn with_string(text: &str, body: impl FnOnce(&mut jni::JNIEnv, &JObject) -> String) -> String {
    let Ok(mut env) = attached() else {
        return broken("no JVM is attached to this process");
    };
    match env.new_string(text) {
        Ok(arg) => {
            let obj: &JObject = arg.as_ref();
            body(&mut env, obj)
        }
        Err(e) => {
            let _ = env.exception_clear();
            broken(&e.to_string())
        }
    }
}

/// One static call on an attached environment, with the pending-exception
/// discipline: a throw is described into the answer and cleared here, always,
/// before anything else runs. Describing it rather than discarding it is the
/// difference between "the bridge failed" and a sentence an operator can act
/// on — the first version of this file said only the former.
fn call_in(env: &mut jni::JNIEnv, method: &str, signature: &str, args: &[JValue]) -> String {
    let class = match class(env) {
        Ok(class) => class,
        Err(why) => return broken(&why),
    };
    let called = env
        .call_static_method(&class, method, signature, args)
        .and_then(jni::objects::JValueGen::l);
    let object = match called {
        Ok(object) => object,
        Err(e) => return broken(&thrown(env).unwrap_or_else(|| e.to_string())),
    };
    match env.get_string(<&JString>::from(&object)).map(Into::into) {
        Ok(text) => text,
        Err(e) => broken(&thrown(env).unwrap_or_else(|| e.to_string())),
    }
}

/// The pending Java exception as its own words, cleared on the way out. A
/// throw left pending turns the next JNI call into a `CheckJNI` abort, so
/// this runs on every failing path and clears before it reads.
fn thrown(env: &mut jni::JNIEnv) -> Option<String> {
    let throwable = env.exception_occurred().ok()?;
    env.exception_clear().ok()?;
    let described = env
        .call_method(&throwable, "toString", "()Ljava/lang/String;", &[])
        .and_then(jni::objects::JValueGen::l)
        .ok()?;
    env.get_string(<&JString>::from(&described))
        .ok()
        .map(Into::into)
}

/// This app's own class, resolved through this app's own class loader.
///
/// **`FindClass` cannot do it from here, and that is the whole of this
/// function.** A thread the JVM did not create resolves names against the
/// SYSTEM class loader, which knows nothing an APK shipped; the tool-host
/// worker is such a thread, so the first version of this bridge threw
/// `ClassNotFoundException` on every call. The application object holds the
/// loader that does know, and one global reference to the resolved class
/// outlives the attach that found it.
fn class(env: &mut jni::JNIEnv) -> Result<GlobalRef, String> {
    static CLASS_REF: std::sync::OnceLock<Option<GlobalRef>> = std::sync::OnceLock::new();
    if let Some(found) = CLASS_REF.get_or_init(|| load(env).ok()) {
        return Ok(found.clone());
    }
    Err("this app's class loader did not yield the interface service".to_owned())
}

/// One resolution: the application, its loader, and the class by name.
fn load(env: &mut jni::JNIEnv) -> Result<GlobalRef, String> {
    let raw = ndk_context::android_context().context();
    if raw.is_null() {
        return Err("no android context".to_owned());
    }
    let context = crate::shell::sys::object_from(raw);
    let loader = env
        .call_method(&context, "getClassLoader", "()Ljava/lang/ClassLoader;", &[])
        .and_then(jni::objects::JValueGen::l)
        .map_err(|e| e.to_string())?;
    let name = env.new_string(CLASS).map_err(|e| e.to_string())?;
    let found = env
        .call_method(
            &loader,
            "loadClass",
            "(Ljava/lang/String;)Ljava/lang/Class;",
            &[JValue::Object(name.as_ref())],
        )
        .and_then(jni::objects::JValueGen::l)
        .map_err(|e| e.to_string())?;
    env.new_global_ref(found).map_err(|e| e.to_string())
}

/// The process's JVM, resolved once. It is held for the life of the process
/// because an [`jni::AttachGuard`] borrows it, and a VM built per call would
/// be one the guard could not outlive. The raw conversion itself is
/// `crate::shell::sys`'s — the crate's one location for those (rule 3).
fn jvm() -> Option<&'static jni::JavaVM> {
    static VM: std::sync::OnceLock<Option<jni::JavaVM>> = std::sync::OnceLock::new();
    VM.get_or_init(|| {
        let raw = ndk_context::android_context().vm();
        if raw.is_null() {
            return None;
        }
        crate::shell::sys::vm_from(raw).ok()
    })
    .as_ref()
}

/// This thread, attached to that JVM. The guard is dropped at the end of the
/// call, which is what detaches; a worker that attached once and never
/// detached would keep a JNI frame alive for the life of the app.
fn attached() -> Result<jni::AttachGuard<'static>, ()> {
    jvm().ok_or(())?.attach_current_thread().map_err(|_| ())
}
