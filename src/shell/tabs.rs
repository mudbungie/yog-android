//! **The tab strip** (DESIGN §23 item 2; lernie `ui::tabs`): over the
//! transcript, one tab per conversation open in the focused workspace this
//! run, standing from the second tab — the name with the asking `?` before
//! it while it asks, the focused one in ink over a brand hairline, the rest
//! in weak ink, and `×` after each. A tap focuses; `×` closes to the
//! neighbour. Sideways-scrolling, never wrapped: a wrapped strip of five
//! names is three rows of a phone.
//!
//! **Nothing here decides what is open**: `crate::tabs` does, filled by the
//! frame that paints the focused conversation. The strip carries no `act:`
//! tag, as the dock does not — it is a second door onto the switcher's
//! focus, and no deleted control's op moved onto it.

use eframe::egui;

use super::app::Shell;
use super::theme::rgb;
use crate::codec::ConvRow;
use crate::seat::Snapshot;
use crate::theme::{self, State, glyph, space};

/// What a tap on the strip asked for.
enum Tapped {
    Focus(String),
    Close(String),
}

impl Shell {
    /// Paint the strip, if the focused workspace has tabs enough, and spend
    /// a tap on it.
    pub(super) fn tabs(&mut self, ui: &mut egui::Ui, snap: &Snapshot, workspace: &str) {
        let strip = self.tabs.strip(workspace);
        if strip.len() < crate::tabs::FROM {
            return;
        }
        let focus = snap.focus.agent.clone();
        let mut tapped = None;
        let mut first = true;
        egui::ScrollArea::horizontal()
            .id_salt("the tab strip")
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = space::XS;
                    for agent in &strip {
                        let row = snap.conversations.iter().find(|row| &row.root_id == agent);
                        let focused = focus.as_deref() == Some(agent.as_str());
                        let (tab, close) = tab(ui, row, agent, focused);
                        if first {
                            self.note_control("tab", ui, tab.rect);
                            self.note_control("tab-close", ui, close.rect);
                            first = false;
                        }
                        if tab.clicked() {
                            tapped = Some(Tapped::Focus(agent.clone()));
                        }
                        if close.clicked() {
                            tapped = Some(Tapped::Close(agent.clone()));
                        }
                    }
                });
            });
        let Some(tapped) = tapped else { return };
        // The open set is settled before the model is borrowed: a close
        // names the neighbour the focus moves to, or nothing where it was
        // the last, which puts the focus down to the workspace.
        let (focus, down) = match tapped {
            Tapped::Focus(agent) => (Some(agent), false),
            Tapped::Close(agent) => match self.tabs.close(workspace, &agent) {
                Some(neighbour) => (Some(neighbour), false),
                None => (None, true),
            },
        };
        let Some(model) = self.model() else { return };
        if let Some(agent) = focus {
            model.focus_conversation(workspace.to_owned(), agent);
        } else if down {
            model.focus_workspace(Some(workspace.to_owned()));
        }
    }
}

/// **One tab and its close**: the name (and the asking mark) as one touch
/// seat, the `×` as another, the brand hairline under the focused one.
fn tab(
    ui: &mut egui::Ui,
    row: Option<&ConvRow>,
    agent: &str,
    focused: bool,
) -> (egui::Response, egui::Response) {
    let name = row.map_or_else(|| agent.to_owned(), |row| row.display.clone());
    let asking = row.is_some_and(|row| row.attention > 0);
    let ink = rgb(if focused { theme::INK } else { theme::INK_WEAK });
    let mut parts = Vec::new();
    if asking {
        parts.push((
            format!("{} ", crate::roster::ATTENTION_MARK),
            rgb(theme::accent(State::Attention)),
        ));
    }
    parts.push((name, ink));
    let tab = seat(ui, &parts);
    if focused {
        let y = tab.rect.max.y - 1.0;
        let line = egui::Rect::from_x_y_ranges(tab.rect.x_range(), y..=y + 1.0);
        ui.painter().rect_filled(line, 0, rgb(theme::BRAND));
    }
    let close = seat(ui, &[(glyph::CLOSE.to_owned(), rgb(theme::INK_FAINT))]);
    (tab, close)
}

/// A touch-tall seat around one run of words, `S` of air either side, the
/// `RAISED` wash while a thumb is on it.
fn seat(ui: &mut egui::Ui, parts: &[(String, egui::Color32)]) -> egui::Response {
    let galley = ui.fonts_mut(|fonts| fonts.layout_job(super::theme::line(ui, parts)));
    let size = egui::vec2(galley.size().x + space::S * 2.0, theme::TOUCH);
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
    if response.is_pointer_button_down_on() || response.hovered() {
        ui.painter().rect_filled(
            rect,
            egui::CornerRadius::same(theme::RADIUS),
            rgb(theme::RAISED),
        );
    }
    let at = egui::pos2(
        rect.left() + space::S,
        rect.center().y - galley.size().y / 2.0,
    );
    ui.painter().galley(at, galley, rgb(theme::INK));
    response
}
