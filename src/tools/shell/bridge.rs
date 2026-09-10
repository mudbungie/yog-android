//! The JNI half of the packaged executables: one static call into
//! `dev.yog.Kit`, answering in [`crate::tools::bridged`]'s two-line protocol
//! with the directory of links a command line's `PATH` should carry.
//!
//! **No argument, because there is no question.** Which executables this APK
//! packages is a fact about the APK, and the far side makes every link it can
//! and names the directory once per process — so this side asks *where*, and
//! nothing else.
//!
//! This file is android-only and excluded from coverage; the decision it
//! feeds is [`super::kit::folded`], which is pure and tested.

use crate::tools::bridged::Door;

/// The class the static entry point lives on, and what its failures are
/// reported as.
static KIT: Door = Door::new("dev.yog.Kit", "packaged executables");

pub(crate) fn bridge_bin() -> String {
    KIT.strings("path", &[])
}
