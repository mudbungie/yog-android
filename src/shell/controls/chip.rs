//! **Which knobs the rule has, what each is showing, and how wide it would
//! like to be** (DESIGN §13.2, bl-0691; §22 item 2, bl-79bb) — the rule's
//! vocabulary: `controls.rs` is what the knobs forget when the focus moves,
//! `shell/status.rs` is where the rule sits, and this is what stands on it.
//!
//! **The face is decided before anything is painted**, which is why it is
//! carried rather than read at each paint site: the width every segment gets
//! is `crate::status::fit`'s answer to what all of them asked for, and a chip
//! cannot ask until it knows what it is going to say.

use eframe::egui;

use super::super::app::Shell;
use super::pick;
use crate::codec::pick::knob::Knob;
use crate::seat::Snapshot;
use crate::shell::place::Band;
use crate::status::Slot;

/// **One knob of the rule, and what it is showing**, in the rule's order:
/// workspace, provider, model, effort, priority (operator ruling 2026-10-12).
pub(in crate::shell) enum Chip {
    /// The three selectors, which never drop and elide inside their share.
    Workspace(String),
    Provider(String),
    Model(String),
    /// The §9.4 tuning pair, each with what the provider row said about it.
    /// The priority chip carries the state it stands in as well as its face:
    /// the tap is a flip, and what it flips FROM is the workspace's own
    /// assignment where nothing optimistic is standing (bl-e9f9).
    Effort(String, Knob),
    Priority(String, Knob, bool),
}

impl Chip {
    /// What this chip is showing.
    pub(in crate::shell) fn face(&self) -> String {
        match self {
            Self::Effort(face, _)
            | Self::Priority(face, _, _)
            | Self::Workspace(face)
            | Self::Provider(face)
            | Self::Model(face) => face.clone(),
        }
    }

    /// Where it stands on the rule.
    pub(in crate::shell) const fn slot(&self) -> Slot {
        match self {
            Self::Workspace(_) => Slot::Workspace,
            Self::Provider(_) => Slot::Provider,
            Self::Model(_) => Slot::Model,
            Self::Effort(..) => Slot::Effort,
            Self::Priority(..) => Slot::Priority,
        }
    }

    /// How wide it would like to be: its words, the button's padding either
    /// side, and the drop caret where it opens a list. A selector's claim is
    /// ignored by the fit — it takes a share.
    pub(in crate::shell) fn wants(&self, ui: &egui::Ui, face: &str) -> f32 {
        let ink = egui::WidgetText::from(face.to_owned()).into_galley(
            ui,
            Some(egui::TextWrapMode::Extend),
            f32::INFINITY,
            egui::TextStyle::Button,
        );
        let caret = if matches!(self, Self::Effort(_, knob) if knob.taken) {
            super::drop::CARET_ROOM
        } else {
            0.0
        };
        ink.size().x + ui.spacing().button_padding.x * 2.0 + caret
    }
}

impl Shell {
    /// **The knobs, faced.** The selectors show what the workspace is
    /// ACTUALLY set to unless an optimistic pick is standing; priority stands
    /// only where the provider's own row takes the lane.
    pub(in crate::shell) fn chips(&self, snap: &Snapshot) -> Vec<Chip> {
        let set = crate::codec::pick::worker(&snap.roles);
        let set = set.as_ref();
        let provider = pick::provider(self, set);
        let (effort, priority) =
            crate::codec::pick::knob::tunable(&snap.providers, provider.as_deref());
        let mut chips = vec![
            Chip::Workspace(snap.focus.workspace.clone().unwrap_or_default()),
            Chip::Provider(provider.clone().unwrap_or_else(|| "provider".to_owned())),
            Chip::Model(crate::codec::pick::face::model(
                self.model.clone(),
                set,
                provider.as_deref(),
            )),
            Chip::Effort(
                crate::codec::pick::face::effort(self.effort.clone(), set),
                effort,
            ),
        ];
        let (on, face) = crate::codec::pick::face::priority(self.priority, set);
        if priority.taken {
            chips.push(Chip::Priority(face, priority, on));
        }
        chips
    }

    /// Paint one knob at its width. `open` is a verb asking for its list;
    /// priority has none (`/priority` is [`Self::priority_verb`]).
    pub(in crate::shell) fn knob(
        &mut self,
        ui: &mut egui::Ui,
        snap: &Snapshot,
        chip: Chip,
        wide: f32,
        area: Band,
        open: bool,
    ) -> egui::Response {
        let face = chip.face();
        match chip {
            Chip::Workspace(_) => self.workspaces(ui, snap, face, wide, area),
            Chip::Provider(_) => self.providers(ui, snap, face, wide, area, open),
            Chip::Model(_) => self.models(ui, snap, face, wide, area, open),
            Chip::Effort(_, knob) => self.effort(ui, face, wide, area, &knob, open),
            Chip::Priority(_, knob, on) => self.priority(ui, face, wide, &knob, on),
        }
    }

    /// `/priority`: the flip, or the provider's reason it cannot be — the
    /// verb works whether or not the rule had room to carry the chip.
    pub(in crate::shell) fn priority_verb(&mut self, snap: &Snapshot) {
        let set = crate::codec::pick::worker(&snap.roles);
        let provider = pick::provider(self, set.as_ref());
        let (_, knob) = crate::codec::pick::knob::tunable(&snap.providers, provider.as_deref());
        let (on, _) = crate::codec::pick::face::priority(self.priority, set.as_ref());
        self.flip_priority(&knob, on);
    }
}
