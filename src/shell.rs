//! The Android shell (DESIGN §3): a thin paint-and-input layer over the
//! tested core. egui via `android-activity`'s `GameActivity` backend behind
//! the minimal Gradle shell in `android/` — the four bl-8d03 device findings
//! and the bl-014e input mechanism live here, each beside the code that
//! carries it.
//!
//! Everything platform-bound is `cfg(target_os = "android")`: the host gate
//! never compiles it, CI's android leg is its compile check, and it is
//! excluded from coverage with its reasoning in `tarpaulin.toml`. What IS
//! host-testable — the UTF-16 span math — stays out of the exclusion and
//! under the 100% floor.

pub mod span;

#[cfg(target_os = "android")]
mod app;
#[cfg(target_os = "android")]
mod bridge;
#[cfg(target_os = "android")]
mod chat;
#[cfg(target_os = "android")]
mod inset;
#[cfg(target_os = "android")]
mod screens;
#[cfg(target_os = "android")]
mod sys;
