//! **The §9.4 tuning pair** (REMOTE §9.4, bl-dfbb): how much reasoning the
//! worker's model calls request, and whether they ask for the provider's
//! priority lane. Both are role config the engine switches at the next step,
//! so they take mid-conversation and neither restarts anything.
//!
//! **They paint always and go dark where the provider will not take them**
//! (bl-809d, amending bl-dfbb's *shown only where*). The capability is still
//! the engine's statement about itself, read in covered code and never
//! derived here; only what an unavailable one LOOKS like has changed. A
//! control that vanishes teaches nothing — the argument the greyed provider
//! row already carries — while one that is there, readable and dark says
//! *this provider has no such lane*, in the one place an operator would go
//! looking for it.
//!
//! **And a dark control ANSWERS when it is tapped** (bl-0691). Grey alone
//! turned out to teach nothing either: an operator on a provider that takes
//! reasoning effort perfectly well saw the same grey a provider without the
//! lane earns, and there is no hover on a phone to say which it was. So the
//! dark control keeps its tap and spends it on the sentence
//! `codec::pick::knob` made — whose provider's refusal it is, or that nothing
//! has been asked yet — painted above the row by `controls.rs`. The three
//! reasons are three different instructions to the operator, which is why one
//! grey could never carry them.

use eframe::egui;

use super::super::act;
use super::super::app::Shell;
use crate::codec::pick::knob::Knob;
use crate::shell::place::Band;

impl Shell {
    /// **The effort selector**: the vocabulary is closed and no wire read
    /// backs it, so the options are the codec's own constant; `off` is one of
    /// them and rides as the real null the engine reads.
    ///
    /// The face is the FILE's own word, which may be one the gesture
    /// vocabulary does not spell (bl-e9f9) — shown as itself, because an
    /// operator seeing `extreme` is being told the truth — and it carries the
    /// control's NAME as well as its value (bl-b191): a magnitude names
    /// nothing, and `medium` alone was read as a context size once already.
    pub(super) fn effort(
        &mut self,
        ui: &mut egui::Ui,
        face: String,
        wide: f32,
        area: Band,
        knob: &Knob,
    ) {
        if !knob.taken {
            self.dark(ui, face, wide, knob, "effort");
            return;
        }
        let mut picked = None;
        let opened = super::drop::drop_down(ui, area, "effort", face, wide, |ui| {
            for level in crate::codec::pick::LEVELS {
                let label = crate::codec::Effort::label(level);
                if ui.selectable_label(false, &label).clicked() {
                    picked = Some((level, label));
                }
            }
        });
        act::act(ui, &opened, "effort");
        if let Some((level, label)) = picked {
            self.tuning_said = None;
            self.effort = Some(label);
            if let Some(model) = self.model() {
                model.set_effort(level);
            }
        }
    }

    /// **The priority toggle**: ask the provider's priority lane for this
    /// role's calls, or stop asking. A toggle and not a tri-state — `off`
    /// removes the line, because asking for the *standard* lane is a
    /// different intent no config key expresses (REMOTE §9.4).
    ///
    /// It wears its state in its own words (`priority: on`) rather than in a
    /// fill, because in a band of chips the word is what an operator reads
    /// and a chip that alone among six carried a tint would be saying
    /// something about state that STYLE.md reserves for the six accents.
    pub(super) fn priority(
        &mut self,
        ui: &mut egui::Ui,
        face: String,
        wide: f32,
        knob: &Knob,
        on: bool,
    ) {
        if !knob.taken {
            self.dark(ui, face, wide, knob, "priority");
            return;
        }
        let control = crate::shell::theme::chip(ui, face.into(), wide, true);
        act::act(ui, &control, "priority");
        if control.clicked() {
            self.tuning_said = None;
            self.priority = Some(!on);
            if let Some(model) = self.model() {
                model.set_priority(!on);
            }
        }
    }

    /// **A knob the provider will not take**: on the glass, dark, and still a
    /// target — the tap is what says why (bl-0691). The `act:` tag rides it
    /// in this state too, because a disabled control is still the
    /// discoverable affordance for the act (PARITY §4).
    fn dark(&mut self, ui: &mut egui::Ui, face: String, wide: f32, knob: &Knob, op: &str) {
        let control = crate::shell::theme::dark_chip(ui, face, wide);
        act::act(ui, &control, op);
        if control.clicked() {
            self.tuning_said.clone_from(&knob.why);
        }
    }
}
