//! **The crate's one JNI plumbing**: attach, resolve a class of this app's,
//! call a static, and never leave an exception pending.
//!
//! It was `src/tools/ui/bridge.rs`'s private half until a second bridge needed
//! it (bl-d815, the camera). Two copies of this would have drifted inside a
//! week, and the two traps below are exactly the kind that are learned once
//! and then silently un-learned in the copy.
//!
//! **`FindClass` cannot resolve this app's classes from here, and that is the
//! whole of [`Bridge::open`].** A thread the JVM did not create resolves names
//! against the SYSTEM class loader, which knows nothing an APK shipped —
//! `android_main` runs on such a thread and so does the tool-host worker, so
//! the first version of the interface bridge threw `ClassNotFoundException` on
//! every call. The application object holds the loader that does know, and one
//! global reference to the resolved class outlives the attach that found it.
//!
//! **A Java exception left pending turns the NEXT JNI call into a `CheckJNI`
//! abort** (DESIGN §3's second trap), so every path that can throw describes
//! and clears before returning. Describing rather than discarding is the
//! difference between "the bridge failed" and a sentence an operator can act
//! on.
//!
//! **The `JavaVM` comes from `ndk_context`, not from an `AndroidApp`.** One of
//! the callers is the tool-host worker, which holds no activity handle and
//! must not — a tool whose availability tracked the UI would be the wrong
//! shape. `ndk_context`'s global is set by android-activity before
//! `android_main` is entered, so it is there for the whole life of the
//! process.
//!
//! This file is android-only and excluded from coverage: it IS the JNI call.
//! What each bridge does with the answers is pure, and is tested.

use jni::objects::{GlobalRef, JByteArray, JString, JValue};

/// One resolved Java class this app calls statics on, and the name its
/// failures are reported under. The label rides with the class because a
/// sentence that does not say WHICH bridge broke is a sentence an operator
/// cannot act on, and there is now more than one.
#[derive(Clone)]
pub(crate) struct Bridge {
    class: GlobalRef,
    label: &'static str,
}

impl Bridge {
    /// Resolve `class` (dotted, as a class loader takes it) through this app's
    /// own loader.
    pub(crate) fn open(
        env: &mut jni::JNIEnv,
        class: &str,
        label: &'static str,
    ) -> Result<Self, String> {
        let raw = ndk_context::android_context().context();
        if raw.is_null() {
            return Err("no android context".to_owned());
        }
        let context = super::sys::object_from(raw);
        let loader = env
            .call_method(&context, "getClassLoader", "()Ljava/lang/ClassLoader;", &[])
            .and_then(jni::objects::JValueGen::l)
            .map_err(|e| e.to_string())?;
        let wanted = env.new_string(class).map_err(|e| e.to_string())?;
        let found = env
            .call_method(
                &loader,
                "loadClass",
                "(Ljava/lang/String;)Ljava/lang/Class;",
                &[JValue::Object(wanted.as_ref())],
            )
            .and_then(jni::objects::JValueGen::l)
            .map_err(|e| e.to_string())?;
        Ok(Self {
            class: env.new_global_ref(found).map_err(|e| e.to_string())?,
            label,
        })
    }

    /// One static call answering a Java `String`. The answer IS the bridge's
    /// product, so a failure becomes a two-line refusal rather than an error
    /// type nobody up the stack has an arm for.
    pub(crate) fn string(
        &self,
        env: &mut jni::JNIEnv,
        method: &str,
        signature: &str,
        args: &[JValue],
    ) -> String {
        let called = env
            .call_static_method(&self.class, method, signature, args)
            .and_then(jni::objects::JValueGen::l);
        let object = match called {
            Ok(object) => object,
            Err(e) => return self.broke(&thrown(env).unwrap_or_else(|| e.to_string())),
        };
        match env.get_string(<&JString>::from(&object)).map(Into::into) {
            Ok(text) => text,
            Err(e) => self.broke(&thrown(env).unwrap_or_else(|| e.to_string())),
        }
    }

    /// One static call answering a Java `byte[]`. `None` is both *the method
    /// answered null* and *the call failed*, because the one caller is a frame
    /// poll that waits for the next frame either way — and a camera failure
    /// worth a sentence is already told by the state poll beside it.
    pub(crate) fn bytes(
        &self,
        env: &mut jni::JNIEnv,
        method: &str,
        signature: &str,
        args: &[JValue],
    ) -> Option<Vec<u8>> {
        let called = env
            .call_static_method(&self.class, method, signature, args)
            .and_then(jni::objects::JValueGen::l);
        let Ok(object) = called else {
            thrown(env);
            return None;
        };
        if object.is_null() {
            return None;
        }
        let read = env.convert_byte_array(<&JByteArray>::from(&object));
        let Ok(bytes) = read else {
            thrown(env);
            return None;
        };
        Some(bytes)
    }

    /// The refusal this bridge's failures wear, in the two-line protocol so
    /// one parser reads it like any other answer.
    pub(crate) fn broke(&self, why: &str) -> String {
        broken(self.label, why)
    }
}

/// The same refusal for a failure that happened before any class was
/// resolved — no VM, no context, no loader.
pub(crate) fn broken(label: &str, why: &str) -> String {
    format!("err\nthe {label} failed: {why}")
}

/// This thread, attached to the process JVM. The guard is dropped at the end
/// of the call, which is what detaches; a worker that attached once and never
/// detached would keep a JNI frame alive for the life of the app.
pub(crate) fn attached() -> Result<jni::AttachGuard<'static>, String> {
    jvm()
        .ok_or_else(|| "no JVM is attached to this process".to_owned())?
        .attach_current_thread()
        .map_err(|e| e.to_string())
}

/// The process's JVM, resolved once. It is held for the life of the process
/// because an [`jni::AttachGuard`] borrows it, and a VM built per call would
/// be one the guard could not outlive. The raw conversion itself is
/// `crate::shell::sys`'s — the crate's one location for those (AGENTS.md
/// rule 3).
fn jvm() -> Option<&'static jni::JavaVM> {
    static VM: std::sync::OnceLock<Option<jni::JavaVM>> = std::sync::OnceLock::new();
    VM.get_or_init(|| {
        let raw = ndk_context::android_context().vm();
        if raw.is_null() {
            return None;
        }
        super::sys::vm_from(raw).ok()
    })
    .as_ref()
}

/// The pending Java exception as its own words, cleared on the way out. A
/// throw left pending turns the next JNI call into a `CheckJNI` abort, so this
/// runs on every failing path and clears before it reads.
pub(crate) fn thrown(env: &mut jni::JNIEnv) -> Option<String> {
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
