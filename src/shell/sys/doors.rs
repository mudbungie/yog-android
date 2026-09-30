//! **The plain Java-calls-Rust doors** — split from `sys` at the 300-line
//! cap (bl-c21d), and under `sys` because each is an `unsafe(no_mangle)`
//! export: the soundness argument is the parent's header, verbatim for
//! every door here — the name is `Java_` plus the class's package path plus
//! the method, which makes the symbol unique by construction. Each door is
//! a string or a boolean in and a decision made in host-tested code; the
//! door that does raw effects of its own (`Pocket_serve`) stays in `sys`.

/// **The scheduled fetch, called from the platform's job** (DESIGN §17;
/// `dev.yog.Watch`). The direction is Java-calls-Rust and not the bridges'
/// Rust-calls-Java, because a job may start this process with no Activity
/// ever created — `ndk_context`'s globals are filled by android-activity on
/// the way to [`super::android_main`], so a bridge asking the JVM for a class would
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
    // A job's run is granted its network: awake for exactly this call.
    let _awake = crate::ladder::Awake::process().hold();
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

/// **The activity resumed or paused** (bl-c21d; `dev.yog.App`): whether a
/// roving ladder may climb, as far as the screen goes (`ladder::awake`).
#[unsafe(no_mangle)]
extern "system" fn Java_dev_yog_App_foreground(
    _env: jni::JNIEnv<'_>,
    _class: jni::objects::JClass<'_>,
    front: jni::sys::jboolean,
) {
    crate::ladder::Awake::process().front(front != 0);
}

/// **The pocket service started or stopped holding the process** (bl-c21d;
/// `dev.yog.App`, from `dev.yog.Pocket`): a foreground service is above the
/// platform's network threshold (DESIGN §18.6), so the foot and the lane
/// climb while it holds.
#[unsafe(no_mangle)]
extern "system" fn Java_dev_yog_App_pocketed(
    _env: jni::JNIEnv<'_>,
    _class: jni::objects::JClass<'_>,
    held: jni::sys::jboolean,
) {
    crate::ladder::Awake::process().service(held != 0);
}

/// **The default network changed** (bl-792e; `dev.yog.App`, from
/// `ConnectivityManager`'s default-network callback): its addresses, one per
/// line, or nothing when there is none. The decision — what is reachable,
/// whether the set moved, which ladders hear of it — is
/// [`crate::ladder::Network::changed`]'s and is tested on the host.
#[unsafe(no_mangle)]
extern "system" fn Java_dev_yog_App_network(
    mut env: jni::JNIEnv<'_>,
    _class: jni::objects::JClass<'_>,
    addresses: jni::objects::JString<'_>,
) {
    let addresses: String = env
        .get_string(&addresses)
        .map(Into::into)
        .unwrap_or_default();
    crate::ladder::Network::process().changed(&addresses);
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
