//! The JNI half of the interface tools: six static calls into
//! `dev.yog.InterfaceService`, each answering one string.
//!
//! The plumbing under it — the attach, this app's class loader, and the
//! pending-exception discipline — is `crate::shell::jvm`, shared with the
//! camera bridge since bl-d815; the traps it carries are written there, once.
//!
//! This file is android-only and excluded from coverage; what it hands back
//! is parsed by [`super::answer`], which is pure and is tested.

use jni::objects::{JObject, JValue};

use crate::shell::jvm::{Bridge, attached, broken, thrown};

/// The class the static entry points live on, in the form a class loader
/// takes (dots, not slashes).
const CLASS: &str = "dev.yog.InterfaceService";

/// What this bridge's failures are reported as.
const LABEL: &str = "interface bridge";

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
    with_string(text, "uiTapText")
}

pub(crate) fn bridge_type(text: &str) -> String {
    with_string(text, "uiText")
}

pub(crate) fn bridge_key(key: &str) -> String {
    with_string(key, "uiKey")
}

pub(crate) fn bridge_shot(path: &str) -> String {
    with_string(path, "screenshot")
}

/// Attach, resolve, call a static taking no object argument.
fn call(method: &str, signature: &str, args: &[JValue]) -> String {
    let mut env = match attached() {
        Ok(env) => env,
        Err(why) => return broken(LABEL, &why),
    };
    match service(&mut env) {
        Ok(bridge) => bridge.string(&mut env, method, signature, args),
        Err(why) => broken(LABEL, &why),
    }
}

/// The shape every argument-taking entry point shares: one Java string in,
/// one answer out.
fn with_string(text: &str, method: &str) -> String {
    let mut env = match attached() {
        Ok(env) => env,
        Err(why) => return broken(LABEL, &why),
    };
    let bridge = match service(&mut env) {
        Ok(bridge) => bridge,
        Err(why) => return broken(LABEL, &why),
    };
    match env.new_string(text) {
        Ok(arg) => {
            let object: &JObject = arg.as_ref();
            bridge.string(
                &mut env,
                method,
                "(Ljava/lang/String;)Ljava/lang/String;",
                &[JValue::Object(object)],
            )
        }
        Err(e) => {
            let said = thrown(&mut env).unwrap_or_else(|| e.to_string());
            broken(LABEL, &said)
        }
    }
}

/// This app's interface service, resolved once through this app's own class
/// loader.
fn service(env: &mut jni::JNIEnv) -> Result<Bridge, String> {
    static SERVICE: std::sync::OnceLock<Option<Bridge>> = std::sync::OnceLock::new();
    SERVICE
        .get_or_init(|| Bridge::open(env, CLASS, LABEL).ok())
        .clone()
        .ok_or_else(|| "this app's class loader did not yield the interface service".to_owned())
}
