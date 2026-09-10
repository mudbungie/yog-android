//! **Which controls the row has, what each is showing, and how wide it would
//! like to be** (DESIGN §13.2, bl-0691) — split from the row that paints them
//! when the second band was folded into the first: `controls.rs` is where the
//! row sits and what it forgets, and this is the row's vocabulary.
//!
//! **The face is decided before anything is painted**, which is why it is
//! carried rather than read at each paint site: the width every control gets
//! is `place::row`'s answer to what all of them asked for, and a chip cannot
//! ask until it knows what it is going to say.

use eframe::egui;

use super::super::app::Shell;
use super::{pick, stops};
use crate::codec::pick::knob::Knob;
use crate::seat::Snapshot;

/// **One control of the row, and what it is showing.** The face is carried
/// because the width is decided from it before anything is painted: a chip
/// asks for the width of its own words, and the row cannot ask until it knows
/// what every chip says.
pub(super) enum Chip {
    /// Stop this turn, or every turn under it (`true`).
    Stop(bool, String),
    /// Wake a branch that stopped advancing.
    Nudge,
    /// The §9.4 tuning pair, each with what the provider row said about it.
    /// The priority chip carries the state it stands in as well as its face:
    /// the tap is a flip, and what it flips FROM is the workspace's own
    /// assignment where nothing optimistic is standing (bl-e9f9).
    Effort(String, Knob),
    Priority(String, Knob, bool),
    /// The two selectors, which are what elides.
    Provider(String),
    Model(String),
}

impl Chip {
    /// What this chip is showing.
    pub(super) fn face(&self) -> String {
        match self {
            Self::Stop(_, face) | Self::Effort(face, _) | Self::Priority(face, _, _) => {
                face.clone()
            }
            Self::Nudge => "nudge".to_owned(),
            Self::Provider(face) | Self::Model(face) => face.clone(),
        }
    }

    /// **How wide this chip would like to be**: the width of its own words
    /// for a control that names one, and everything for a selector — a model
    /// name is as long as a provider chose to make it, so a selector asking
    /// for its face's width would take the row from the controls beside it.
    /// `place::row` turns the pair of answers into the width each one gets.
    ///
    /// **A control that opens a list asks for its caret too.** The triangle
    /// is painted into the chip's own right-hand end (`controls/drop.rs`), so
    /// a face measured without it is a face that will elide by exactly a
    /// caret — which is how `effort` came back as `…` on a row where it fit.
    pub(super) fn wants(&self, ui: &egui::Ui, face: &str) -> f32 {
        match self {
            Self::Provider(_) | Self::Model(_) => f32::INFINITY,
            _ => {
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
    }
}

impl Shell {
    /// **Which controls the row has, in the order it paints them.** The stop
    /// acts first — they are what an operator reaches for while something is
    /// running — then the tuning pair, then the two selectors, which are the
    /// ones that give up width.
    pub(super) fn chips(&self, snap: &Snapshot) -> Vec<Chip> {
        let set = crate::codec::pick::worker(&snap.roles);
        let set = set.as_ref();
        let provider = pick::provider(self, set);
        let (effort, priority) =
            crate::codec::pick::knob::tunable(&snap.providers, provider.as_deref());
        let mut chips = stops::chips(snap);
        chips.push(Chip::Effort(
            crate::codec::pick::face::effort(self.effort.clone(), set),
            effort,
        ));
        let (on, face) = crate::codec::pick::face::priority(self.priority, set);
        chips.push(Chip::Priority(face, priority, on));
        chips.push(Chip::Provider(
            provider.clone().unwrap_or_else(|| "provider".to_owned()),
        ));
        chips.push(Chip::Model(crate::codec::pick::face::model(
            self.model.clone(),
            set,
            provider.as_deref(),
        )));
        chips
    }

    /// One chip painted at the width the row gave it.
    pub(super) fn paint(
        &mut self,
        ui: &mut egui::Ui,
        snap: &Snapshot,
        chip: Chip,
        face: String,
        wide: f32,
        area: crate::shell::place::Band,
    ) {
        match chip {
            Chip::Stop(children, _) => self.stop(ui, face, wide, children),
            Chip::Nudge => self.nudge(ui, face, wide),
            Chip::Effort(_, knob) => self.effort(ui, face, wide, area, &knob),
            Chip::Priority(_, knob, on) => self.priority(ui, face, wide, &knob, on),
            Chip::Provider(_) => self.providers(ui, snap, face, wide, area),
            Chip::Model(_) => self.models(ui, snap, face, wide, area),
        }
    }
}
