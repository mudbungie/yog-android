//! **One Java door class this crate calls statics on**: resolve it once
//! through this app's own loader, call a method by name, and hand back the
//! two-line answer [`super::answer`] reads.
//!
//! It was `tools/paper/bridge.rs`'s private half until the sighted pair
//! became the second door (bl-b0a9). Every door is the same four moves —
//! attach, resolve, marshal N Java strings, build the descriptor from their
//! count — and the descriptor is exactly the half a second copy gets wrong: a
//! hand-written signature that drifts from its argument list is a
//! `NoSuchMethodError` at the far end rather than a compile error here. The
//! protocol has one home (`tools::bridged`) and now so does the call.
//!
//! The plumbing under it — the attach, this app's class loader, and the
//! pending-exception discipline — is `crate::shell::jvm`; the traps it
//! carries are written there once.
//!
//! This file is android-only and excluded from coverage: it IS the JNI call.

use std::sync::OnceLock;

use jni::objects::{JObject, JValue};

use crate::shell::jvm::{Bridge, attached, broken, thrown};

/// One Java `String` — what every entry point behind a door answers, and the
/// only argument type any of them takes.
const STRING: &str = "Ljava/lang/String;";

/// A class of this app's, and the name its failures are reported under. Built
/// as a `static` beside the calls that spend it, so the resolution is paid for
/// once per class rather than once per invocation.
pub(crate) struct Door {
    class: &'static str,
    label: &'static str,
    resolved: OnceLock<Option<Bridge>>,
}

impl Door {
    /// A door, unopened. `class` is dotted, as a class loader takes it.
    pub(crate) const fn new(class: &'static str, label: &'static str) -> Self {
        Self {
            class,
            label,
            resolved: OnceLock::new(),
        }
    }

    /// Call a static taking N Java strings and answering one. The signature is
    /// built from the count rather than written per method because every entry
    /// point behind a door has that shape.
    pub(crate) fn strings(&self, method: &str, args: &[&str]) -> String {
        let mut env = match attached() {
            Ok(env) => env,
            Err(why) => return broken(self.label, &why),
        };
        let bridge = match self.bridge(&mut env) {
            Ok(bridge) => bridge,
            Err(why) => return broken(self.label, &why),
        };
        let mut held = Vec::with_capacity(args.len());
        for arg in args {
            match env.new_string(arg) {
                Ok(text) => held.push(text),
                Err(e) => {
                    let said = thrown(&mut env).unwrap_or_else(|| e.to_string());
                    return broken(self.label, &said);
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

    /// This door's class, resolved once through this app's own class loader.
    fn bridge(&self, env: &mut jni::JNIEnv) -> Result<Bridge, String> {
        self.resolved
            .get_or_init(|| Bridge::open(env, self.class, self.label).ok())
            .clone()
            .ok_or_else(|| format!("this app's class loader did not yield {}", self.class))
    }
}
