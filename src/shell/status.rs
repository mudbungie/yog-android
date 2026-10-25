//! **The status rule** (DESIGN §22 items 1, 2 and 10, bl-79bb; §23 item
//! 14; §24 item 4): the bottom edge of the chat — one touch-tall line of
//! runs, `<provider> ⏶ | <model> ⏶ | <effort> ⏶ | <priority> | <diagnostics>
//! — <name>` — and the banner line over it. Which runs stand, what each
//! says, which drop and whether a list is fresh are `crate::status`, pure
//! and host-tested; this file paints them and performs the taps. The
//! workspace knob and the mark went to the drawer (§24 item 4).
//!
//! **Every knob is a door** (`theme::door`): its words in `INK` and its mark
//! in `INK_WEAK` inside one seat, bare at rest and `RAISED` while pressed,
//! the separators in `INK_FAINT` between. A tap opens its list upward from
//! the door, placed by `shell::place` (`controls/drop.rs`) on egui's
//! foreground layer over the box under the transcript, showing the cached
//! list at once and asking for a fresh one; while the ask is out the door's
//! mark is `…` in the annotation accent, and `✔` in the attention accent
//! once the answer has landed. A tap on a row sets it and closes the list;
//! the platform's back closes it too.
//!
//! **The banner stands on its own line over the rule**, not in a slot of it:
//! at a phone's width the three selectors that never drop leave no room for a
//! sentence, and a refusal elided to four letters is a refusal nobody read.
//! A tap puts it down.

use eframe::egui;

use super::app::Shell;
use super::controls::{Chip, Seat, band};
use super::place::Band;
use super::theme::ink;
use crate::seat::Snapshot;
use crate::status::{Place, Said, Slot};
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
/// small type with room to breathe. The name on it is still the drawer's
/// door, a target this tall — `≡` and `/conversations` are the touch-tall
/// ways to the same place.
const LOW: f32 = TOUCH / 2.0;

/// One run of the line, before it is given a width.
enum Segment {
    Knob(Chip),
    Said(Said),
}

impl Shell {
    /// **The rule**, added to a bottom-up block: the line (and the small
    /// line under it) first, so it takes the bottom edge, then the banner
    /// line over it.
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
        // The tappable area a list may open into (bl-78c2): everything
        // `app::pass` left, the floor included. **A knob's list opens
        // upward** (§22 item 2): under the rule there is the diagnostics
        // line at most, so the list stands over the box, the dock and the
        // transcript — on egui's foreground layer, anchored to the door, as
        // the desktop lays its dropup (lernie §4.45 item 3) — and the native
        // field steps off the glass while it does (`composer::overlay`).
        let area = Band {
            top: ui.max_rect().top(),
            bottom: ui.available_rect_before_wrap().bottom(),
        };
        ui.scope(|ui| {
            ui.spacing_mut().interact_size.y = TOUCH;
            // The rule's type is `SMALL` (STYLE §4's band exception, kept),
            // and a run's neighbours stand `XS` off either side of `|`.
            ui.style_mut().text_styles.insert(
                egui::TextStyle::Button,
                egui::FontId::proportional(theme::type_scale::SMALL),
            );
            ui.spacing_mut().item_spacing.x = space::XS;
            self.segments(ui, snap, area);
        });
        self.banner_line(ui, snap);
    }

    /// The knobs and the diagnostics: on the rule where one line holds
    /// them, else the diagnostics on a small line under it (`status::layout`,
    /// bl-96fc) — added first, the block being bottom-up.
    fn segments(&mut self, ui: &mut egui::Ui, snap: &Snapshot, area: Band) {
        let row = super::controls::focused_row(snap);
        let now = crate::roster::now_unix();
        let mut segments: Vec<Segment> = self.chips(snap).into_iter().map(Segment::Knob).collect();
        let said =
            crate::status::diagnostics(row.as_ref(), &snap.transcript, snap.head.as_ref(), now);
        segments.extend(said.into_iter().map(Segment::Said));
        let slots: Vec<Slot> = segments.iter().map(slot).collect();
        let lead = extra(ui);
        let wants: Vec<f32> = segments.iter().map(|s| want(ui, s, lead)).collect();
        let held = self.rule.verb.or(self.rule.asked.map(|(slot, _)| slot));
        let spacing = ui.spacing().item_spacing.x;
        let gap = spacing * 2.0 + super::theme::wide(ui, glyph::SEP);
        let under = ui.available_width();
        let places = crate::status::layout(&slots, &wants, under, under, gap, held);
        let mut top = Vec::new();
        let mut low = Vec::new();
        for (segment, place) in segments.into_iter().zip(places) {
            match place {
                Place::Rule(wide) => top.push((segment, wide)),
                Place::Under(wide) => low.push((segment, wide)),
                Place::Off => {}
            }
        }
        if !low.is_empty() {
            ui.allocate_ui_with_layout(
                egui::vec2(under, LOW),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| self.runs(ui, snap, low, LOW, area),
            );
        }
        band(ui, |ui| self.runs(ui, snap, top, TOUCH, area));
    }

    /// One line's runs, `|` between neighbours and `—` before the name.
    fn runs(
        &mut self,
        ui: &mut egui::Ui,
        snap: &Snapshot,
        line: Vec<(Segment, f32)>,
        tall: f32,
        area: Band,
    ) {
        for (at, (segment, wide)) in line.into_iter().enumerate() {
            match segment {
                Segment::Said(said) if said.slot == Slot::Name => {
                    faint(ui, glyph::LEAD, tall);
                    self.said(ui, &said, (wide - extra(ui)).max(0.0), tall);
                }
                segment => {
                    if at > 0 {
                        faint(ui, glyph::SEP, tall);
                    }
                    match segment {
                        Segment::Knob(chip) => self.knob_on_rule(ui, snap, chip, wide, area),
                        Segment::Said(said) => self.said(ui, &said, wide, tall),
                    }
                }
            }
        }
    }

    /// One knob: its door, its list asked for when it opens, its mark — the
    /// drop mark, or the reload's in its place while a list is asked for.
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
        let seat = Seat {
            face: chip.face(),
            wide,
            area,
            open,
            mark: self.mark_of(snap, slot),
        };
        let face = self.knob(ui, snap, chip, seat);
        let name = match slot {
            Slot::Provider => "rule-provider",
            Slot::Model => "rule-model",
            Slot::Effort => "rule-effort",
            _ => "rule-priority",
        };
        self.note_control(name, ui, face.rect);
        if open || face.clicked() {
            let mark = match slot {
                Slot::Provider | Slot::Model => Some(snap.listed),
                _ => None,
            };
            self.rule.asked = Some((slot, mark));
        }
    }

    /// **The banner line**, while one stands: the operator's own act's
    /// sentence, the server's failure, or the redial — and a tap puts it
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

mod runs;
use runs::{extra, faint, slot, want};
