//! **The knob table** (DESIGN §23 item 11; lernie §4.46 item 4): every
//! setting that is a fact about this device, as one row each — its key, its
//! label, its help sentence, its kind with the bounds a control paints, and
//! the way to read and write it on the typed [`Settings`] the readers read.
//!
//! **One table, three readers.** The pane paints it (`shell::settings`), the
//! file reads and writes it ([`read`], [`write`]) and the tests judge it, so
//! a knob added here is painted, remembered and judged with no second edit.
//! The readers — the transcript, the dock, the frame's scale — read the
//! typed value and never the table.
//!
//! **Nothing here crosses the boundary**, and nothing here is the mark's
//! configuration surface (§13.2): that is the engine's enrollment and the
//! foot's; these are the operator's own knobs, REMOTE §7's per-seat state.
//!
//! **Written only where set.** [`to_json`] writes a knob only when it stands
//! off its default, so a device that touched nothing writes nothing and a
//! default moved in a later build reaches it. **The file is the operator's,
//! never the engine's**: the cache's fail-closed rule does not apply, and an
//! unreadable file is the defaults.

use std::collections::BTreeSet;
use std::path::Path;

use serde_json::{Map, Value, json};

use crate::rows::AutoExpand;

/// The glass's scale: egui's zoom factor, one by default.
pub const SCALE: f32 = 1.0;
/// The file's name, beside the cache in the app's private storage.
pub const FILE: &str = "settings.json";

/// **Everything this device can be set to**, typed. `Default` is the
/// table's defaults.
///
/// **The switches are a set**, not a field apiece: which of them are on is
/// the one fact, [`Settings::on`] is the query, and the default is the set
/// of those that stand on.
#[derive(Debug, Clone, PartialEq)]
pub struct Settings {
    pub scale: f32,
    /// The switches that are on.
    pub on: BTreeSet<Toggle>,
}

/// **The six switches.**
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Toggle {
    /// Whether a long press on a control tells what it does (§23 item 10).
    Help,
    /// The fills under the transcript's blocks (§23 item 7).
    Bubbles,
    /// Whether a working folds itself once the answer stands (§23 item 6).
    Fold,
    /// Whether delivered prose and the live tail stand open (was `/full`).
    Full,
    /// Whether thinking, calls, results and rollups stand open (was
    /// `/steps`).
    Steps,
    /// Whether the dock stands unfolded when a conversation is focused
    /// (§22 item 9).
    Dock,
}

impl Settings {
    /// **Whether `switch` is on.**
    #[must_use]
    pub fn on(&self, switch: Toggle) -> bool {
        self.on.contains(&switch)
    }

    /// **Turn `switch` on or off.**
    pub fn set(&mut self, switch: Toggle, on: bool) {
        if on {
            self.on.insert(switch);
        } else {
            self.on.remove(&switch);
        }
    }

    /// **The transcript's policy**, read off the three switches that are it.
    #[must_use]
    pub fn auto(&self) -> AutoExpand {
        AutoExpand {
            responses: self.on(Toggle::Full),
            others: self.on(Toggle::Steps),
            fold: self.on(Toggle::Fold),
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            scale: SCALE,
            on: [Toggle::Help, Toggle::Bubbles, Toggle::Fold, Toggle::Full].into(),
        }
    }
}

/// What kind of control a knob is painted as.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Kind {
    /// On or off: a check.
    Toggle,
    /// A number between two bounds, moved by `step`, said with `unit`: a
    /// slider.
    Number {
        unit: &'static str,
        floor: f32,
        ceiling: f32,
        step: f32,
    },
}

/// A knob's value, read or written: the one shape both kinds share.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Knobbed {
    On(bool),
    At(f32),
}

/// **One row of the table.**
#[derive(Debug, Clone, Copy)]
pub struct Knob {
    /// The key it is written under, and the word a reader of the file sees.
    pub key: &'static str,
    /// The label on the pane.
    pub label: &'static str,
    /// **What it does**, in the operator's terms (§23 item 10).
    pub help: &'static str,
    pub kind: Kind,
    /// Read it off the settings.
    pub get: fn(&Settings) -> Knobbed,
    /// Write it; a value of the other kind writes nothing.
    pub set: fn(&mut Settings, Knobbed),
}

mod knobs;
pub use knobs::KNOBS;

/// **The knob `key` names**, if the table has it.
#[must_use]
pub fn knob(key: &str) -> Option<&'static Knob> {
    KNOBS.iter().find(|knob| knob.key == key)
}

/// **The settings a JSON object holds**: every knob the object names and
/// this build can read, over the defaults. An absent key, a null, a value of
/// the wrong shape and a key no knob answers to are each the default.
#[must_use]
pub fn from_json(held: &Value) -> Settings {
    let mut out = Settings::default();
    for knob in KNOBS {
        let Some(value) = held.get(knob.key) else {
            continue;
        };
        let read = match knob.kind {
            Kind::Toggle => value.as_bool().map(Knobbed::On),
            // Through serde, never a cast: `f64 as f32` is the one narrowing
            // the lint set denies.
            Kind::Number { .. } => serde_json::from_value(value.clone()).ok().map(Knobbed::At),
        };
        if let Some(read) = read {
            (knob.set)(&mut out, read);
        }
    }
    out
}

/// **The JSON object the settings are written as**: only the knobs standing
/// off their default, each under its key. Empty for a device set to nothing.
#[must_use]
pub fn to_json(settings: &Settings) -> Value {
    let rest = Settings::default();
    let mut out = Map::new();
    for knob in KNOBS {
        let held = (knob.get)(settings);
        if held == (knob.get)(&rest) {
            continue;
        }
        let value = match held {
            Knobbed::On(on) => json!(on),
            Knobbed::At(at) => json!(at),
        };
        out.insert(knob.key.to_owned(), value);
    }
    Value::Object(out)
}

/// **The settings the file at `path` holds**, or the defaults — which is
/// every way the read can fail, because a forgotten knob is a tap and a
/// refusal to start is an outage.
#[must_use]
pub fn read(path: &Path) -> Settings {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str::<Value>(&text).ok())
        .map(|held| from_json(&held))
        .unwrap_or_default()
}

/// **Write the settings to `path`**, whole, as [`to_json`] spells them. A
/// failure is said, not raised: the knob moved on the glass either way.
pub fn write(path: &Path, settings: &Settings) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    std::fs::write(path, to_json(settings).to_string())
        .map_err(|e| format!("{}: {e}", path.display()))
}

#[cfg(test)]
mod tests;
