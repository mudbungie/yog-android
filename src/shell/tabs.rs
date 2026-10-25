//! **The top line and the tab strip** (DESIGN §24 item 3, amending §23 item
//! 2; lernie `ui::tabs`): one `TOUCH`-tall line under the top inset — `≡` at
//! the left end in `INK` in a `TOUCH`-square seat, the drawer's door; the
//! strip filling the remainder, one tab per conversation open in the focused
//! workspace this run, from the FIRST tab — the name with the asking `?`
//! before it while it asks, the focused one in ink over a brand hairline, the
//! rest in weak ink, and `×` after each; and `+` at the right end in the
//! brand, the start's door. In start mode the strip's place says `new
//! conversation on <workspace>` in weak ink. A tap on a tab focuses; `×`
//! closes to the neighbour. Sideways-scrolling, never wrapped: a wrapped strip
//! of five names is three rows of a phone.
//!
//! **Nothing here decides what is open**: `crate::tabs` does, filled by the
//! frame that paints the focused conversation. The strip carries no `act:`
//! tag, as the dock does not — it is a second door onto the drawer's focus,
//! and no deleted control's op moved onto it. The two doors either end of it
//! carry theirs: `≡` the list's (`conversations`), `+` the start's.

use eframe::egui;

use super::app::Shell;
use super::theme::rgb;
use crate::codec::ConvRow;
use crate::seat::Snapshot;
use crate::theme::{self, State, glyph, space};

/// What the two doors do, told on a long press (§23 item 10).
const MENU_HELP: &str = "opens the drawer: the conversations, the workspace and every other place";
const NEW_HELP: &str = "begins a conversation on this workspace, the caret in the field";

/// What a tap on the strip asked for.
enum Tapped {
    Focus(String),
    Close(String),
}

impl Shell {
    /// **The top line**: `≡`, the strip or the start's words, and `+` while a
    /// workspace is focused. `workspace` is `None` with nothing focused — the
    /// empty chat the drawer stands over, whose line is `≡` alone.
    pub(super) fn tabs(&mut self, ui: &mut egui::Ui, snap: &Snapshot, workspace: Option<&str>) {
        let focus = snap.focus.agent.clone();
        let mut tapped = None;
        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), theme::TOUCH),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                ui.spacing_mut().item_spacing.x = space::XS;
                let menu = super::drawer::square(ui, glyph::MENU, rgb(theme::INK), MENU_HELP);
                super::act::act(ui, &menu, "conversations");
                self.note_control("menu", ui, menu.rect);
                if menu.clicked() {
                    self.drawer.open = true;
                }
                let plus = workspace.map_or(0.0, |_| theme::TOUCH + space::XS);
                let between = egui::vec2((ui.available_width() - plus).max(0.0), theme::TOUCH);
                ui.allocate_ui_with_layout(
                    between,
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| match (workspace, &focus) {
                        (Some(ws), None) => starting(ui, ws),
                        (Some(ws), Some(_)) => tapped = self.strip(ui, snap, ws, focus.as_deref()),
                        (None, _) => {}
                    },
                );
                let Some(ws) = workspace else { return };
                let add = super::drawer::square(ui, glyph::SHUT, rgb(theme::BRAND), NEW_HELP);
                super::act::acts(ui, &add, &["prepare", "prompt"]);
                self.note_control("new", ui, add.rect);
                if add.clicked() {
                    self.focus_workspace(Some(ws.to_owned()));
                    self.summon = true;
                }
            },
        );
        let (Some(tapped), Some(workspace)) = (tapped, workspace) else {
            return;
        };
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

    /// The strip itself, where the focused workspace has tabs enough.
    fn strip(
        &mut self,
        ui: &mut egui::Ui,
        snap: &Snapshot,
        workspace: &str,
        focus: Option<&str>,
    ) -> Option<Tapped> {
        let strip = self.tabs.strip(workspace);
        if strip.len() < crate::tabs::FROM {
            return None;
        }
        let mut tapped = None;
        let mut first = true;
        egui::ScrollArea::horizontal()
            .id_salt("the tab strip")
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = space::XS;
                    for agent in &strip {
                        let row = snap.conversations.iter().find(|row| &row.root_id == agent);
                        let (tab, close) = tab(ui, row, agent, focus == Some(agent.as_str()));
                        if std::mem::take(&mut first) {
                            self.note_control("tab", ui, tab.rect);
                            self.note_control("tab-close", ui, close.rect);
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
        tapped
    }
}

/// **The strip's place in start mode** (§24 item 3): what the box begins.
fn starting(ui: &mut egui::Ui, workspace: &str) {
    let said =
        egui::RichText::new(format!("new conversation on {workspace}")).color(rgb(theme::INK_WEAK));
    ui.add_space(space::S);
    ui.add(egui::Label::new(said).truncate());
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
            format!("{} ", glyph::ASKING),
            rgb(theme::accent(State::Attention)),
        ));
    }
    parts.push((name, ink));
    let tab = seat(ui, &parts, "shows this conversation");
    if focused {
        let y = tab.rect.max.y - 1.0;
        let line = egui::Rect::from_x_y_ranges(tab.rect.x_range(), y..=y + 1.0);
        ui.painter().rect_filled(line, 0, rgb(theme::BRAND));
    }
    let close = seat(
        ui,
        &[(glyph::CLOSE.to_owned(), rgb(theme::INK_FAINT))],
        "closes this tab; the conversation stays",
    );
    (tab, close)
}

/// A touch-tall seat around one run of words, `S` of air either side, the
/// `RAISED` wash while a thumb is on it.
fn seat(ui: &mut egui::Ui, parts: &[(String, egui::Color32)], help: &str) -> egui::Response {
    let galley = ui.fonts_mut(|fonts| fonts.layout_job(super::theme::line(ui, parts)));
    let size = egui::vec2(galley.size().x + space::S * 2.0, theme::TOUCH);
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
    super::theme::told::told(ui, &response, help);
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
