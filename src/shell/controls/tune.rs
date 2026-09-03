//! **The §9.4 tuning pair** (REMOTE §9.4, bl-dfbb): how much reasoning the
//! worker's model calls request, and whether they ask for the provider's
//! priority lane. Both are role config the engine switches at the next step,
//! so they take mid-conversation and neither restarts anything.
//!
//! They paint only where the picked provider's own row says it will take them
//! — the capability is the engine's statement about itself, read in covered
//! code and never derived here — which is also why the `make screens` walk
//! seeds a provider that takes both: a control gated off in every walked
//! screen is a control the parity gate cannot see, and unproven is red
//! (PARITY §5).

use eframe::egui;

use super::super::act;
use super::super::app::Shell;
use crate::codec::RoleRow;
use crate::shell::place::Band;

/// The effort selector's width class. Narrower than a provider or a model
/// selector because its whole vocabulary is four short words.
const EFFORT: f32 = 76.0;

impl Shell {
    /// **The effort selector**: the vocabulary is closed and no wire read
    /// backs it, so the options are the codec's own constant; `off` is one of
    /// them and rides as the real null the engine reads.
    pub(super) fn effort(&mut self, ui: &mut egui::Ui, set: Option<&RoleRow>, area: Band) {
        // The read carries the FILE's own word, which may be one the gesture
        // vocabulary does not spell (bl-e9f9). It is shown as itself — an
        // operator seeing `extreme` is being told the truth, and the four
        // words below are what they may change it TO.
        let shown = self
            .effort
            .clone()
            .or_else(|| set.and_then(|row| row.effort.clone()))
            .unwrap_or_else(|| "effort".to_owned());
        let mut picked = None;
        let opened = super::drop::drop_down(ui, area, "effort", shown, EFFORT, |ui| {
            for level in crate::codec::pick::LEVELS {
                let label = crate::codec::Effort::label(level);
                if ui.selectable_label(false, &label).clicked() {
                    picked = Some((level, label));
                }
            }
        });
        act::act(ui, &opened, "effort");
        if let Some((level, label)) = picked {
            self.effort = Some(label);
            if let Some(model) = self.model() {
                model.set_effort(level);
            }
        }
    }

    /// **The priority toggle**: ask the provider's priority lane for this
    /// role's calls, or stop asking. A toggle and not a tri-state — `off`
    /// removes the line, and asking for the *standard* lane is a different
    /// intent no config key expresses (REMOTE §9.4).
    pub(super) fn priority(&mut self, ui: &mut egui::Ui, set: Option<&RoleRow>) {
        let mut on = self
            .priority
            .unwrap_or_else(|| set.is_some_and(|row| row.priority));
        let control = ui.toggle_value(&mut on, "priority");
        act::act(ui, &control, "priority");
        if control.clicked() {
            self.priority = Some(on);
            if let Some(model) = self.model() {
                model.set_priority(on);
            }
        }
    }
}
