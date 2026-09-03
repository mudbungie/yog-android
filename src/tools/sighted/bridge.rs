//! The JNI half of the sighted pair: two static calls into `dev.yog.Sighted`,
//! each answering one string in [`crate::tools::bridged`]'s protocol.
//!
//! No activity crosses here — the class asks `dev.yog.App` for the one in
//! front, which is where that fact lives (the paper bridge's own argument,
//! and it binds twice as hard for these two: both are refused outright by the
//! platform when nothing of this app is on screen).
//!
//! The call is [`crate::tools::bridged::Door`]'s and the plumbing under it is
//! `crate::shell::jvm`'s. This file is android-only and excluded from
//! coverage; what it hands back is parsed by
//! [`crate::tools::bridged::answer`], which is pure and tested.

use crate::tools::bridged::Door;

/// The class the two static entry points live on, and what its failures are
/// reported as.
static SIGHTED: Door = Door::new("dev.yog.Sighted", "sighted bridge");

pub(crate) fn bridge_still(lens: &str, path: &str) -> String {
    SIGHTED.strings("camera", &[lens, path])
}

pub(crate) fn bridge_location() -> String {
    SIGHTED.strings("location", &[])
}
