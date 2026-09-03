//! The JNI half of the paper tools: four static calls into `dev.yog.Paper`,
//! each answering one string in [`crate::tools::bridged`]'s protocol.
//!
//! **No activity is passed and none is held.** The camera's bridge takes one
//! as an argument because a permission dialog goes in front of a screen; a
//! tool must work with no screen up, so this bridge asks the class and the
//! class asks `dev.yog.App` — which is the one place the *is this app in
//! front* fact lives, written by the activity's own lifecycle. A tool whose
//! availability tracked a handle held here would be answering with this
//! bridge's memory instead of the platform's answer.
//!
//! The plumbing under it — the attach, this app's class loader, and the
//! pending-exception discipline — is `crate::shell::jvm`, shared with the
//! interface and camera bridges; the traps it carries are written there once.
//!
//! This file is android-only and excluded from coverage; what it hands back
//! is parsed by [`crate::tools::bridged::answer`], which is pure and tested.

use jni::objects::{JObject, JValue};

use crate::shell::jvm::{Bridge, attached, broken, thrown};

/// The class the static entry points live on, in the form a class loader
/// takes (dots, not slashes).
const CLASS: &str = "dev.yog.Paper";

/// What this bridge's failures are reported as.
const LABEL: &str = "paper bridge";

/// One Java `String`, which is what every one of these answers.
const STRING: &str = "Ljava/lang/String;";

pub(crate) fn bridge_device() -> String {
    strings("device", &[])
}

pub(crate) fn bridge_clipboard(text: &str) -> String {
    strings("clipboardSet", &[text])
}

pub(crate) fn bridge_notify(title: &str, text: &str) -> String {
    strings("notify", &[title, text])
}

pub(crate) fn bridge_open(kind: &str, value: &str) -> String {
    strings("open", &[kind, value])
}

/// Attach, resolve, and call a static taking N Java strings and answering
/// one. The signature is built from the count rather than written per method
/// because every entry point on this class has that shape, and a hand-written
/// descriptor that drifted from its argument list is a `NoSuchMethodError` at
/// the far end rather than a compile error here.
fn strings(method: &str, args: &[&str]) -> String {
    let mut env = match attached() {
        Ok(env) => env,
        Err(why) => return broken(LABEL, &why),
    };
    let bridge = match paper(&mut env) {
        Ok(bridge) => bridge,
        Err(why) => return broken(LABEL, &why),
    };
    let mut held = Vec::with_capacity(args.len());
    for arg in args {
        match env.new_string(arg) {
            Ok(text) => held.push(text),
            Err(e) => {
                let said = thrown(&mut env).unwrap_or_else(|| e.to_string());
                return broken(LABEL, &said);
            }
        }
    }
    let values: Vec<_> = held
        .iter()
        .map(|text| {
            let object: &JObject = text.as_ref();
            JValue::Object(object)
        })
        .collect();
    let signature = format!("({}){STRING}", STRING.repeat(args.len()));
    bridge.string(&mut env, method, &signature, &values)
}

/// This app's paper class, resolved once through this app's own class loader.
fn paper(env: &mut jni::JNIEnv) -> Result<Bridge, String> {
    static PAPER: std::sync::OnceLock<Option<Bridge>> = std::sync::OnceLock::new();
    PAPER
        .get_or_init(|| Bridge::open(env, CLASS, LABEL).ok())
        .clone()
        .ok_or_else(|| "this app's class loader did not yield the paper tools".to_owned())
}
