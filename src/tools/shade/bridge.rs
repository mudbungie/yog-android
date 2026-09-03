//! The JNI half of the shade read: one static call into `dev.yog.Shade`,
//! answering one string in [`crate::tools::bridged`]'s protocol.
//!
//! No service handle crosses here — the class asks `dev.yog.ShadeService` for
//! the live listener, which is where that fact lives, exactly as the interface
//! tools ask theirs. The cap rides as a string because every door takes
//! strings (`Door::strings`); the Java side reads it, and a value it cannot
//! read falls back to the same default rather than refusing, because a cap is
//! not the answer.
//!
//! The call is [`crate::tools::bridged::Door`]'s and the plumbing under it is
//! `crate::shell::jvm`'s. This file is android-only and excluded from
//! coverage; what it hands back is parsed by
//! [`crate::tools::bridged::answer`], which is pure and tested.

use crate::tools::bridged::Door;

/// The class the entry point lives on, and what its failures are reported as.
static SHADE: Door = Door::new("dev.yog.Shade", "shade bridge");

pub(crate) fn bridge_notifications(limit: &str) -> String {
    SHADE.strings("notifications", &[limit])
}
