//! The JNI half of the paper tools: four static calls into `dev.yog.Paper`,
//! each answering one string in [`crate::tools::bridged`]'s protocol.
//!
//! **No activity is passed and none is held.** The sighted pair's bridge
//! passes none either, for a reason worth restating: a permission dialog goes
//! in front of a screen, but a tool must work with no screen up, so a bridge
//! asks the class and the class asks `dev.yog.App` — which is the one place
//! the *is this app in front* fact lives, written by the activity's own
//! lifecycle. A tool whose availability tracked a handle held here would be
//! answering with this bridge's memory instead of the platform's answer.
//!
//! The call itself — attach, this app's class loader, the descriptor built
//! from the argument count, and the pending-exception discipline under all of
//! it — is [`crate::tools::bridged::Door`] and `crate::shell::jvm`, shared
//! with every other bridge; the traps they carry are written there once.
//!
//! This file is android-only and excluded from coverage; what it hands back
//! is parsed by [`crate::tools::bridged::answer`], which is pure and tested.

use crate::tools::bridged::Door;

/// The class the static entry points live on, and what its failures are
/// reported as.
static PAPER: Door = Door::new("dev.yog.Paper", "paper bridge");

pub(crate) fn bridge_device() -> String {
    PAPER.strings("device", &[])
}

pub(crate) fn bridge_clipboard(text: &str) -> String {
    PAPER.strings("clipboardSet", &[text])
}

pub(crate) fn bridge_notify(title: &str, text: &str) -> String {
    PAPER.strings("notify", &[title, text])
}

pub(crate) fn bridge_open(kind: &str, value: &str) -> String {
    PAPER.strings("open", &[kind, value])
}
