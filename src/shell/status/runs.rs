//! **The runs of the rule that are not knobs** (DESIGN §23 item 14), split
//! from `shell/status.rs` at the 300-line cap on the seam between laying the
//! line and painting what stands on it: a diagnostic, the name's door, a
//! separator or a lead, the mark a knob's door wears, and each run's want.

use eframe::egui;

use super::{Segment, Shell};
use crate::overlay::Overlay;
use crate::seat::Snapshot;
use crate::shell::theme::{PAD, ink, rgb};
use crate::status::{Reload, Said, Slot};
use crate::theme::{self, State, glyph};

impl Shell {
    /// **The mark a knob's door wears**: `…` while its list is asked for,
    /// `✔` once the answer landed, else the drop mark in weak ink.
    pub(super) fn mark_of(&self, snap: &Snapshot, slot: Slot) -> (&'static str, egui::Color32) {
        let Some((asked, Some(at))) = self.rule.asked else {
            return (glyph::DROP, rgb(theme::INK_WEAK));
        };
        if asked != slot {
            return (glyph::DROP, rgb(theme::INK_WEAK));
        }
        let now = if slot == Slot::Workspace {
            snap.passed
        } else {
            snap.listed
        };
        match crate::status::reload(at, now) {
            Reload::Reloading => (glyph::ELIDED, ink(State::Annotation)),
            Reload::Fresh => (glyph::DONE, ink(State::Attention)),
        }
    }

    /// One diagnostic: its words in its ink, elided to its width. The name is
    /// the switcher's door.
    pub(super) fn said(&mut self, ui: &mut egui::Ui, said: &Said, wide: f32, tall: f32) {
        let size = egui::vec2(wide, tall);
        if said.slot != Slot::Name {
            crate::shell::theme::run(ui, (&said.text, rgb(said.ink)), size, PAD);
            return;
        }
        let hit = crate::shell::theme::door(
            ui,
            (&said.text, rgb(said.ink)),
            None,
            size,
            "opens every workspace and its conversations",
        );
        crate::shell::act::act(ui, &hit, "workspaces");
        self.note_control("rule-name", ui, hit.rect);
        if hit.clicked() {
            self.open(Overlay::Switcher);
        }
    }
}

/// A separator or a lead, in faint ink, its own width across.
pub(super) fn faint(ui: &mut egui::Ui, mark: &str, tall: f32) {
    let size = egui::vec2(crate::shell::theme::wide(ui, mark), tall);
    crate::shell::theme::run(ui, (mark, rgb(theme::INK_FAINT)), size, 0.0);
}

/// **What `—` costs over `|`**: the name stands after the lead where every
/// other run stands after a separator, so its want carries the difference.
pub(super) fn extra(ui: &egui::Ui) -> f32 {
    (crate::shell::theme::wide(ui, glyph::LEAD) - crate::shell::theme::wide(ui, glyph::SEP))
        .max(0.0)
}

/// Where a segment stands.
pub(super) fn slot(segment: &Segment) -> Slot {
    match segment {
        Segment::Knob(chip) => chip.slot(),
        Segment::Said(said) => said.slot,
    }
}

/// How wide a segment would like to be at the rule's type.
pub(super) fn want(ui: &egui::Ui, segment: &Segment, lead: f32) -> f32 {
    match segment {
        Segment::Knob(chip) => chip.wants(ui),
        Segment::Said(said) if said.slot == Slot::Name => {
            crate::shell::theme::wants(ui, &said.text, None) + lead
        }
        Segment::Said(said) => crate::shell::theme::wants(ui, &said.text, None),
    }
}
