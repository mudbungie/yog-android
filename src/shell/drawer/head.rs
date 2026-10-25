//! **The drawer's head and its find field** (DESIGN §24 item 2): the focused
//! workspace's name with `⏶` as one door — the rule's workspace knob moved
//! whole, its list opening DOWNWARD inside the drawer as rows, its reload
//! marks in the door — and `+` at the right end, the start's own door; then
//! the §13.6 search field one `TOUCH`-tall line under it, its hint `find`,
//! whose send opens the hits overlay over the chat and closes the drawer.
//!
//! **The field is the IME mirror's `NEEDLE`** (`shell::bridge`), the one the
//! search screen placed until this ball; the search overlay keeps its body
//! and loses its field.

use eframe::egui;

use super::super::app::{NEEDLE, Shell};
use super::super::theme::{ink, rgb};
use crate::seat::Snapshot;
use crate::status::Reload;
use crate::theme::{self, State, glyph};

/// What the two doors and the field do, told on a long press (§23 item 10).
const WORKSPACE_HELP: &str = "opens the list of workspaces; one is focused at a time";
const ROW_HELP: &str = "focuses this workspace and lists its conversations";
const NEW_HELP: &str = "begins a conversation on this workspace, the caret in the field";
const FIND_HELP: &str = "searches every conversation for the words in the field";

impl Shell {
    /// **The head**: the door, the `+`, and the list under them while it is
    /// open. Answers whether `+` closed the drawer.
    pub(super) fn drawer_head(&mut self, ui: &mut egui::Ui, snap: &Snapshot) -> bool {
        let focused = snap.focus.workspace.clone();
        let listed = crate::drawer::stands(self.drawer.list, focused.is_some());
        let mut closing = false;
        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), theme::TOUCH),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                let gap = ui.spacing().item_spacing.x;
                let wide = if focused.is_some() {
                    ui.available_width() - theme::TOUCH - gap
                } else {
                    ui.available_width()
                };
                let mark = self.head_mark(snap);
                let door = super::super::theme::door(
                    ui,
                    (&crate::drawer::head(focused.as_deref()), rgb(theme::INK)),
                    Some(mark),
                    egui::vec2(wide.max(0.0), theme::TOUCH),
                    WORKSPACE_HELP,
                );
                super::super::act::act(ui, &door, "workspaces");
                self.note_control("drawer-workspace", ui, door.rect);
                if door.clicked() {
                    // A tap opens or folds the list and asks the roster
                    // afresh (`Model::refresh`, the workspace knob's reload);
                    // with nothing focused the list cannot fold.
                    self.drawer.list = !listed;
                    self.drawer.asked = Some(snap.passed);
                    if let Some(model) = self.model() {
                        model.refresh();
                    }
                }
                let Some(workspace) = focused.clone() else {
                    return;
                };
                let add = square(ui, glyph::SHUT, rgb(theme::BRAND), NEW_HELP);
                super::super::act::acts(ui, &add, &["prepare", "prompt"]);
                self.note_control("drawer-new", ui, add.rect);
                if add.clicked() {
                    self.focus_workspace(Some(workspace));
                    self.summon = true;
                    closing = true;
                }
            },
        );
        if listed {
            self.workspace_rows(ui, snap, focused.as_deref());
        }
        closing
    }

    /// **The mark the head's door wears**: `…` while the roster is asked for,
    /// `✔` once the pass that answers has moved, else the drop mark.
    fn head_mark(&self, snap: &Snapshot) -> (&'static str, egui::Color32) {
        match self
            .drawer
            .asked
            .map(|at| crate::status::reload(at, snap.passed))
        {
            Some(Reload::Reloading) => (glyph::ELIDED, ink(State::Annotation)),
            Some(Reload::Fresh) => (glyph::DONE, ink(State::Attention)),
            None => (glyph::DROP, rgb(theme::INK_WEAK)),
        }
    }

    /// **The list, downward inside the drawer**: every workspace the cached
    /// roster holds as a row, the focused one in the brand; a tap focuses it
    /// — a new list of conversations under the head — and folds the list.
    fn workspace_rows(&mut self, ui: &mut egui::Ui, snap: &Snapshot, focused: Option<&str>) {
        let mut picked = None;
        let mut first = true;
        for row in &snap.workspaces {
            let here = focused == Some(row.workspace.as_str());
            let name = egui::RichText::new(&row.workspace).color(rgb(if here {
                theme::BRAND
            } else {
                theme::INK
            }));
            let control = super::super::theme::row(ui, name.into(), ROW_HELP);
            super::super::act::act(ui, &control, "conversations");
            if std::mem::take(&mut first) {
                self.note_control("workspace-row", ui, control.rect);
            }
            if control.clicked() {
                picked = Some(row.workspace.clone());
            }
        }
        if let Some(workspace) = picked {
            if focused != Some(workspace.as_str()) {
                self.focus_workspace(Some(workspace));
            }
            self.drawer.list = false;
            self.drawer.asked = None;
        }
    }

    /// **The find field**: the needle, dressed as the prompt's field, and
    /// `find` in the brand at its right end while it holds text. Answers
    /// whether a search was asked, which closes the drawer.
    pub(super) fn drawer_find(&mut self, ui: &mut egui::Ui) -> bool {
        let mut fired = false;
        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), theme::TOUCH),
            egui::Layout::right_to_left(egui::Align::Center),
            |ui| {
                if !self.needle.trim().is_empty() {
                    let word = egui::RichText::new(crate::drawer::FIND).color(rgb(theme::BRAND));
                    let go = super::super::theme::word(ui, word, theme::TOUCH, FIND_HELP);
                    super::super::act::act(ui, &go, "search");
                    fired = go.clicked();
                }
                let pad = super::super::composer::padding(ui);
                let hint = egui::RichText::new(crate::drawer::FIND).color(rgb(theme::INK_FAINT));
                let field = ui.add(
                    egui::TextEdit::singleline(&mut self.needle)
                        .id(egui::Id::new(NEEDLE.id))
                        .desired_width(f32::INFINITY)
                        .margin(pad)
                        .hint_text(hint),
                );
                self.note_control("find", ui, field.rect);
            },
        );
        if !fired {
            return false;
        }
        if let Some(model) = self.model() {
            model.search(self.needle.clone());
        }
        true
    }
}

/// **A `TOUCH`-square seat around one glyph** in `tint`, the `RAISED` wash
/// while a thumb is on it — the head's `+`, and the top line's two doors
/// (`shell/tabs.rs`).
pub(in crate::shell) fn square(
    ui: &mut egui::Ui,
    glyph: &str,
    tint: egui::Color32,
    help: &str,
) -> egui::Response {
    let side = egui::vec2(theme::TOUCH, theme::TOUCH);
    let (rect, hit) = ui.allocate_exact_size(side, egui::Sense::click());
    super::super::theme::told::told(ui, &hit, help);
    if hit.is_pointer_button_down_on() || hit.hovered() {
        ui.painter().rect_filled(
            rect,
            egui::CornerRadius::same(theme::RADIUS),
            rgb(theme::RAISED),
        );
    }
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        glyph,
        egui::FontId::proportional(theme::type_scale::HEADING),
        tint,
    );
    hit
}
