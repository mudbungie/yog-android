//! **The status rule** (DESIGN §22 items 1, 2 and 10, bl-79bb; operator
//! ruling 2026-10-12): the touch-tall band above the prompt — the mark, the
//! knobs as dropups, this conversation's diagnostics and its name — and the
//! banner line over it. Which segments stand, what each says, which drop and
//! whether a list is fresh are `crate::status`, pure and host-tested; this
//! file paints them and performs the taps.
//!
//! **A knob is a dropup**: a tap opens its list upward from the segment,
//! placed by `shell::place` (`controls/drop.rs`), showing the cached list at
//! once and asking for a fresh one; a small mark stands on the segment —
//! `…` in the annotation accent while the ask is out, `✔` in the attention
//! accent once the answer has landed. A tap on a row sets it and closes the
//! list; the platform's back closes it too.
//!
//! **The banner stands on its own line over the rule**, not in a slot of it:
//! at a phone's width the three selectors that never drop leave no room for a
//! sentence, and a refusal elided to four letters is a refusal nobody read.
//! A tap puts it down.

use eframe::egui;

use super::app::Shell;
use super::controls::{Chip, band};
use super::place::Band;
use super::theme::{ink, rgb};
use crate::overlay::Overlay;
use crate::seat::Snapshot;
use crate::status::{Place, Reload, Said, Slot};
use crate::theme::{self, State, TOUCH, glyph, space};

/// What the rule holds between frames: which knob's list was asked for and
/// the watermark it was asked at (none for a list nothing re-reads), the
/// banner sentence put down, and the knob a verb asked to open.
#[derive(Default)]
pub(crate) struct Rule {
    asked: Option<(Slot, Option<usize>)>,
    dismissed: Option<String>,
    pub(crate) verb: Option<Slot>,
}

/// **The diagnostics' own line's height**, where they stand under the rule:
/// small type with room to breathe. The name on it is still the switcher's
/// door, a target this tall — the back gesture and `/conversations` are the
/// touch-tall ways to the same place.
const LOW: f32 = TOUCH / 2.0;

/// One segment of the band, before it is given a width.
enum Segment {
    Knob(Chip),
    Said(Said),
}

impl Shell {
    /// **The rule**, added to a bottom-up block: the band first, then the
    /// banner line over it.
    pub(super) fn rule(&mut self, ui: &mut egui::Ui, snap: &Snapshot) {
        self.follow(snap);
        let ctx = ui.ctx().clone();
        if egui::Popup::is_any_open(&ctx) {
            if std::mem::take(&mut self.back) {
                egui::Popup::close_all(&ctx);
            }
        } else if self.rule.verb.is_none() {
            self.rule.asked = None;
        }
        // The tappable area a list may open into (bl-78c2): from the top of
        // what `app::pass` left down to the top of the prompt line, which
        // this bottom-up block has already laid. **A knob's list opens
        // upward** (§22 item 2): with the prompt line out of its area the
        // room below a selector is the diagnostics line at most, so the list
        // never covers the field or the facts under the rule.
        let area = Band {
            top: ui.max_rect().top(),
            bottom: ui.available_rect_before_wrap().bottom(),
        };
        ui.scope(|ui| {
            ui.spacing_mut().interact_size.y = TOUCH;
            // The rule's type is `SMALL` (STYLE §4's band exception, kept):
            // five knobs and the diagnostics share one line where they fit.
            ui.style_mut().text_styles.insert(
                egui::TextStyle::Button,
                egui::FontId::proportional(theme::type_scale::SMALL),
            );
            ui.spacing_mut().button_padding.x = space::S;
            self.segments(ui, snap, area);
        });
        self.banner_line(ui, snap);
    }

    /// The knobs and the diagnostics: on the rule beside the mark where one
    /// line holds them, else the diagnostics on a small line under it
    /// (`status::layout`, bl-96fc) — added first, the block being bottom-up.
    fn segments(&mut self, ui: &mut egui::Ui, snap: &Snapshot, area: Band) {
        let row = super::controls::focused_row(snap);
        let now = crate::roster::now_unix();
        let mut segments: Vec<Segment> = self.chips(snap).into_iter().map(Segment::Knob).collect();
        let said =
            crate::status::diagnostics(row.as_ref(), &snap.transcript, snap.head.as_ref(), now);
        segments.extend(said.into_iter().map(Segment::Said));
        let slots: Vec<Slot> = segments.iter().map(slot).collect();
        let wants: Vec<f32> = segments.iter().map(|s| want(ui, s)).collect();
        let held = self.rule.verb.or(self.rule.asked.map(|(slot, _)| slot));
        let gap = ui.spacing().item_spacing.x;
        let under = ui.available_width();
        let places = crate::status::layout(&slots, &wants, under - TOUCH - gap, under, gap, held);
        let low: Vec<(Said, f32)> = segments
            .iter()
            .zip(&places)
            .filter_map(|(segment, place)| match (segment, place) {
                (Segment::Said(said), Place::Under(wide)) => Some((said.clone(), *wide)),
                _ => None,
            })
            .collect();
        if !low.is_empty() {
            ui.allocate_ui_with_layout(
                egui::vec2(under, LOW),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    for (said, wide) in &low {
                        self.said(ui, said, *wide, LOW);
                    }
                },
            );
        }
        band(ui, |ui| {
            self.mark(ui);
            for (segment, place) in segments.into_iter().zip(places) {
                let Place::Rule(wide) = place else { continue };
                match segment {
                    Segment::Knob(chip) => self.knob_on_rule(ui, snap, chip, wide, area),
                    Segment::Said(said) => self.said(ui, &said, wide, TOUCH),
                }
            }
        });
    }

    /// One knob: painted, its list asked for when it opens, its reload mark.
    fn knob_on_rule(
        &mut self,
        ui: &mut egui::Ui,
        snap: &Snapshot,
        chip: Chip,
        wide: f32,
        area: Band,
    ) {
        let slot = chip.slot();
        let open = self.rule.verb == Some(slot);
        if open {
            self.rule.verb = None;
        }
        let face = self.knob(ui, snap, chip, wide, area, open);
        let name = match slot {
            Slot::Workspace => "rule-workspace",
            Slot::Provider => "rule-provider",
            Slot::Model => "rule-model",
            Slot::Effort => "rule-effort",
            _ => "rule-priority",
        };
        self.note_control(name, ui, face.rect);
        if open || face.clicked() {
            let mark = match slot {
                Slot::Workspace => Some(snap.passed),
                Slot::Provider | Slot::Model => Some(snap.listed),
                _ => None,
            };
            self.rule.asked = Some((slot, mark));
        }
        let Some((asked, Some(at))) = self.rule.asked else {
            return;
        };
        if asked != slot {
            return;
        }
        let now = if slot == Slot::Workspace {
            snap.passed
        } else {
            snap.listed
        };
        let (mark, state) = match crate::status::reload(at, now) {
            Reload::Reloading => (glyph::ELIDED, State::Annotation),
            Reload::Fresh => (glyph::DONE, State::Attention),
        };
        ui.painter().text(
            face.rect.right_top() + egui::vec2(-space::XS, space::XS),
            egui::Align2::RIGHT_TOP,
            mark,
            egui::FontId::proportional(theme::type_scale::SMALL),
            ink(state),
        );
    }

    /// One diagnostic: its words in its ink, elided to its width. The name is
    /// the switcher's door.
    fn said(&mut self, ui: &mut egui::Ui, said: &Said, wide: f32, tall: f32) {
        let sense = if said.slot == Slot::Name {
            egui::Sense::click()
        } else {
            egui::Sense::hover()
        };
        let (rect, hit) = ui.allocate_exact_size(egui::vec2(wide, tall), sense);
        if said.slot == Slot::Name {
            super::theme::told::told(ui, &hit, "opens every workspace and its conversations");
        }
        let pad = ui.spacing().button_padding.x;
        let text = egui::RichText::new(&said.text).color(rgb(said.ink));
        let galley = egui::WidgetText::from(text).into_galley(
            ui,
            Some(egui::TextWrapMode::Truncate),
            (wide - pad * 2.0).max(0.0),
            egui::TextStyle::Button,
        );
        let at = egui::pos2(rect.left() + pad, rect.center().y - galley.size().y / 2.0);
        ui.painter().galley(at, galley, rgb(said.ink));
        if said.slot != Slot::Name {
            return;
        }
        super::act::act(ui, &hit, "workspaces");
        self.note_control("rule-name", ui, rect);
        if hit.clicked() {
            self.open(Overlay::Switcher);
        }
    }

    /// **The banner line**, while one stands: the operator's own act's
    /// sentence, the engine's failure, or the redial — and a tap puts it
    /// down.
    fn banner_line(&mut self, ui: &mut egui::Ui, snap: &Snapshot) {
        let said = self.verb_said.clone().or_else(|| self.tuning_said.clone());
        let Some((text, state)) = crate::status::banner(
            said.as_deref(),
            snap.error.as_deref(),
            snap.reconnecting,
            self.rule.dismissed.as_deref(),
        ) else {
            return;
        };
        let line = egui::RichText::new(&text).color(ink(state));
        if !super::theme::row(ui, line.into(), "puts this notice down").clicked() {
            return;
        }
        if state == State::Annotation {
            self.verb_said = None;
            self.tuning_said = None;
        } else {
            self.rule.dismissed = Some(text);
        }
    }
}

/// Where a segment stands.
fn slot(segment: &Segment) -> Slot {
    match segment {
        Segment::Knob(chip) => chip.slot(),
        Segment::Said(said) => said.slot,
    }
}

/// How wide a segment would like to be at the rule's type.
fn want(ui: &egui::Ui, segment: &Segment) -> f32 {
    match segment {
        Segment::Knob(chip) => chip.wants(ui, &chip.face()),
        Segment::Said(said) => {
            let galley = egui::WidgetText::from(said.text.clone()).into_galley(
                ui,
                Some(egui::TextWrapMode::Extend),
                f32::INFINITY,
                egui::TextStyle::Button,
            );
            galley.size().x + ui.spacing().button_padding.x * 2.0
        }
    }
}
