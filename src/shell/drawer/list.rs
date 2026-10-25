//! **The drawer's conversations and its places** (DESIGN §24 item 2): the
//! §13.20 tree for the focused workspace, each row as the desktop bar's open
//! row reads it — the state glyph in its ink, the asking `?` in the attention
//! accent, the name in `INK` (the focused one in the brand), one line beneath
//! in `INK_WEAK`, the age at the right end — then, under a hairline, every
//! surface that is not about the focused conversation as chips three to a
//! row, in the sheet's own words and wearing the sheet's own tags.
//!
//! What a row says is `crate::drawer::words`; which places, in which order,
//! is `crate::verbs::places`. A tap on a row focuses and closes the drawer; a
//! long press opens §13.5's row menu with the told sentence as its head. A
//! chip floats its overlay over the chat and closes the drawer; one that
//! needs a workspace is dark with nothing focused, and its sentence says so.

use eframe::egui;

use super::super::app::Shell;
use super::super::place::Band;
use super::super::theme::rgb;
use crate::drawer::Words;
use crate::seat::Snapshot;
use crate::theme::{self, space};
use crate::verbs::{Does, Place};

/// What a dark chip would need, told on a long press.
const DARK_HELP: &str = "dark until a workspace is focused — pick one at the drawer's head";

impl Shell {
    /// **The conversations**, one row each with its rails. Answers whether a
    /// tap focused one.
    pub(super) fn drawer_rows(&mut self, ui: &mut egui::Ui, snap: &Snapshot, area: Band) -> bool {
        let Some(workspace) = snap.focus.workspace.clone() else {
            return false;
        };
        let listed = crate::roster::ordered(snap.conversations.clone());
        if listed.is_empty() {
            ui.add(
                egui::Label::new(
                    egui::RichText::new(crate::drawer::EMPTY).color(rgb(theme::INK_WEAK)),
                )
                .wrap(),
            );
            return false;
        }
        let rails = crate::roster::threads(&listed);
        let now = crate::roster::now_unix();
        let mut first = true;
        let mut opened = None;
        for (row, rails) in listed.iter().zip(&rails) {
            let focused = snap.focus.agent.as_deref() == Some(row.root_id.as_str());
            let words = crate::drawer::words(row, focused, now);
            let control = conversation(ui, rails, &words);
            // **Where the harness finds a row** (§15.2): the first only.
            if std::mem::take(&mut first) {
                self.note_control("row", ui, control.rect);
            }
            super::super::act::act(ui, &control, "transcript");
            // The menu decides whether the tap was a navigation at all.
            let menu = self.menu(ui, area, row, &control);
            if control.clicked() && !menu {
                opened = Some(row.root_id.clone());
            }
        }
        let Some(agent) = opened else {
            return false;
        };
        if let Some(model) = self.model() {
            model.focus_conversation(workspace, agent);
        }
        true
    }

    /// **The places**, three chips to a row at `SMALL`. Answers whether one
    /// floated its overlay.
    pub(super) fn drawer_places(&mut self, ui: &mut egui::Ui, snap: &Snapshot) -> bool {
        let focused = snap.focus.workspace.is_some();
        let waiting = crate::drawer::waiting(&snap.workspaces);
        let places = crate::verbs::places();
        let mut taken = None;
        ui.scope(|ui| {
            ui.style_mut().text_styles.insert(
                egui::TextStyle::Button,
                egui::FontId::proportional(theme::type_scale::SMALL),
            );
            for three in places.chunks(3) {
                ui.horizontal(|ui| {
                    let wide = super::super::theme::share(ui, 3);
                    for place in three {
                        if let Some(does) = self.chip(ui, place, wide, focused, waiting) {
                            taken = Some(does);
                        }
                    }
                });
                ui.add_space(space::XS);
            }
        });
        let Some(Does::Open(overlay)) = taken else {
            return false;
        };
        self.open(overlay);
        true
    }

    /// One chip: its word, the waiting count after `attention` in the
    /// attention accent while it is non-zero, dark where it needs a workspace
    /// and none is focused. Answers what a tap does.
    fn chip(
        &mut self,
        ui: &mut egui::Ui,
        place: &Place,
        wide: f32,
        focused: bool,
        waiting: usize,
    ) -> Option<Does> {
        let live = focused || !place.aimed;
        let word = place.row.word;
        let tint = rgb(if live { theme::INK } else { theme::INK_FAINT });
        let mut parts = vec![(word.to_owned(), tint)];
        if word == "attention" && waiting > 0 {
            parts.push((
                format!(" {waiting}"),
                super::super::theme::ink(theme::State::Attention),
            ));
        }
        let help = if live {
            format!("opens {word}: {}", place.row.summary)
        } else {
            DARK_HELP.to_owned()
        };
        let label = super::super::theme::line(ui, &parts);
        let chip = super::super::theme::chip(ui, label.into(), wide, live, &help);
        super::super::act::acts(ui, &chip, place.row.ops);
        self.note_control(word, ui, chip.rect);
        chip.clicked().then_some(place.row.does)
    }
}

/// **One conversation row**: the rails, then the box at its indent — the
/// glyph, the asking mark and the name on the first line with the age at the
/// right end, the line beneath in weak ink — `RAISED` under a thumb and a
/// hairline under it.
fn conversation(ui: &mut egui::Ui, rails: &[bool], words: &Words) -> egui::Response {
    let font = egui::TextStyle::Body.resolve(ui.style());
    let line = ui.ctx().fonts_mut(|fonts| fonts.row_height(&font));
    let pad = ui.spacing().button_padding;
    let gap = ui.spacing().item_spacing.y;
    let lines = 1 + usize::from(words.beneath.is_some());
    let lines_tall = line * lines as f32 + pad.y * 2.0;
    let tall = lines_tall.max(theme::TOUCH);
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), tall), egui::Sense::click());
    super::super::theme::told::told(ui, &response, Shell::ROW_HELP);
    if !ui.is_rect_visible(rect) {
        return response;
    }
    let boxed = rect.with_min_x(rect.left() + crate::roster::indent(rails.len()));
    if response.is_pointer_button_down_on() || response.hovered() {
        ui.painter().rect_filled(
            boxed,
            egui::CornerRadius::same(theme::RADIUS),
            rgb(theme::RAISED),
        );
    }
    ui.painter().hline(
        boxed.x_range(),
        rect.bottom(),
        egui::Stroke::new(1.0, rgb(theme::HAIRLINE)),
    );
    let first = rect.top() + pad.y + (tall - lines_tall) / 2.0;
    super::super::screens::connectors(ui, rect, boxed.left(), rails, first + line / 2.0, gap);
    let age = ui
        .painter()
        .layout_no_wrap(words.age.clone(), font.clone(), rgb(theme::INK_WEAK));
    let left = boxed.left() + pad.x;
    let wide = boxed.width() - pad.x * 2.0;
    let mut parts = vec![
        (words.mark.0.clone(), rgb(words.mark.1)),
        (" ".to_owned(), rgb(words.name.1)),
    ];
    if words.asking {
        parts.push((
            format!("{} ", crate::theme::glyph::ASKING),
            rgb(crate::drawer::asking_ink()),
        ));
    }
    parts.push((words.name.0.clone(), rgb(words.name.1)));
    let mut head = super::super::theme::line(ui, &parts);
    head.wrap.max_width = (wide - age.size().x - space::S).max(0.0);
    let head = ui.ctx().fonts_mut(|fonts| fonts.layout_job(head));
    ui.painter()
        .galley(egui::pos2(left, first), head, rgb(theme::INK));
    ui.painter().galley(
        egui::pos2(boxed.right() - pad.x - age.size().x, first),
        age,
        rgb(theme::INK_WEAK),
    );
    if let Some(beneath) = &words.beneath {
        let mut job = super::super::theme::line(ui, &[(beneath.clone(), rgb(theme::INK_WEAK))]);
        job.wrap.max_width = wide.max(0.0);
        let galley = ui.ctx().fonts_mut(|fonts| fonts.layout_job(job));
        ui.painter()
            .galley(egui::pos2(left, first + line), galley, rgb(theme::INK_WEAK));
    }
    response
}
