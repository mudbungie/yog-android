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

/// The effort selector's width class. Still narrower than a provider or a
/// model selector, and now sized for the longest thing the face can say —
/// `effort: medium`, plus the room the caret is drawn into — so the control
/// keeps its width when the level changes under it. A face that resized on
/// every pick would move the priority toggle beside it.
const EFFORT: f32 = 132.0;

impl Shell {
    /// **The effort selector**: the vocabulary is closed and no wire read
    /// backs it, so the options are the codec's own constant; `off` is one of
    /// them and rides as the real null the engine reads.
    pub(super) fn effort(&mut self, ui: &mut egui::Ui, set: Option<&RoleRow>, area: Band) {
        // The read carries the FILE's own word, which may be one the gesture
        // vocabulary does not spell (bl-e9f9). It is shown as itself — an
        // operator seeing `extreme` is being told the truth, and the four
        // words below are what they may change it TO.
        //
        // **The face carries the control's NAME as well as its value**
        // (bl-b191). The two selectors beside it are named by their content —
        // a provider row reads `anthropic`, a model row reads a model — and
        // the toggle after it paints the word `priority` whatever it is set
        // to. A magnitude does not name anything: `medium` alone is a level
        // of something the operator has to guess, and the guess recorded in
        // bl-78c2 was *context size*. So the empty state's word stays on the
        // face once a level is standing, and the level is what follows it.
        // REMOTE §9.4's own word, spelled the same here as on the wire.
        let shown = self
            .effort
            .clone()
            .or_else(|| set.and_then(|row| row.effort.clone()))
            .map_or_else(|| "effort".to_owned(), |level| format!("effort: {level}"));
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
