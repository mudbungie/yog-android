//! **The conversation list, and the acts its rows carry** (DESIGN §13.5,
//! bl-f97c). Split from `screens.rs` when the row menu landed, on the seam
//! that file already had: everything left there is a list of taps and a
//! standing, and this is the one screen above the transcript with mechanics of
//! its own.
//!
//! **Split again at the 300-line cap** (bl-06d3), on the seam this file's own
//! opening paragraph already named: what a list IS — its order, its indent,
//! its rows' words and the field that starts one — stays here, and the long
//! press a row carries moved whole to `rows/menu.rs`. The cap was reached by
//! the indent, which is three lines; splitting a file that was resting on the
//! wall is the rule, and shaving three lines to stay under it is what the
//! rule forbids.

use eframe::egui;

use crate::codec::ConvRow;
use crate::seat::Snapshot;
use crate::shell::app::Shell;
use crate::shell::mark::Back;
use crate::shell::place::Band;

mod menu;

impl Shell {
    pub(crate) fn conversations(&mut self, ui: &mut egui::Ui, snap: &Snapshot, workspace: &str) {
        // **The tappable band, read once at the top** (bl-78c2). `app::pass`
        // shrinks what every screen is painted into by the platform's bottom
        // inset (bl-9cfd) and this `ui` is that rect — so the two edges are
        // taken here, before the bottom-up layout and the scroller below have
        // narrowed it. A row's menu is a popup and is NOT laid out inside any
        // of them: egui gives it an `Area` of its own against the whole
        // display, which is the defect `shell::place` answers.
        let area = Band {
            top: ui.max_rect().top(),
            bottom: ui.max_rect().bottom(),
        };
        // The starter rides the BOTTOM of this screen, where the composer sits
        // on the next one: starting a conversation and speaking into one are
        // the same gesture to a thumb, so they are in the same place. The
        // bottom of this layout is the platform's floor, and **what claims it
        // first is what may never be pushed off it** (bl-192c): the controls
        // and the starter, then the chrome and the list in what remains.
        ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
            // The same controls row as the transcript's, under the same
            // composer (§13.2, bl-0267): a model is picked for the WORKSPACE,
            // so it is picked from the screen that lists it as readily as from
            // a conversation inside it.
            self.controls(ui, snap);
            self.starter(ui);
            ui.add_space(4.0);
            ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                if self.bar(ui, workspace, &Back::To("workspaces")) {
                    self.focus_workspace(None);
                }
                super::banner(ui, snap);
                // **The four surfaces that name THIS workspace** (§13.14):
                // the balls it holds, what its attempts cost, its two armings,
                // and the machines that may execute for it. All four are
                // offered where that workspace is what the operator is
                // standing in, and they share two bands rather than taking
                // four full-width rows off the list below them.
                self.aimed_entries(ui);
                ui.separator();
                egui::ScrollArea::vertical()
                    .min_scrolled_height(0.0)
                    .show(ui, |ui| {
                        ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                            self.listing(ui, snap, workspace, area);
                        });
                    });
            });
        });
    }

    /// The rows themselves. Newest subtree first, each says when (REMOTE §9.9,
    /// bl-e837), and each hangs at its own depth under its root (§11's indent,
    /// bl-06d3). The indent is the whole of what says a row is a subagent, and
    /// it only means anything because `roster::ordered` moves a subtree whole
    /// — an indented row that had drifted away from its parent would say less
    /// than no indent at all.
    fn listing(&mut self, ui: &mut egui::Ui, snap: &Snapshot, workspace: &str, area: Band) {
        if snap.conversations.is_empty() {
            ui.weak("nothing here yet — say what to start below");
        }
        // Both readings spend the stamp the engine carries; the clock is read
        // once for the whole list so no two rows are dated from different
        // instants.
        let now = crate::roster::now_unix();
        let mut first = true;
        for row in crate::roster::ordered(snap.conversations.clone()) {
            let ink = crate::shell::chat::tone_hue(ui, row.tone);
            let label = egui::RichText::new(label(&row, now)).color(ink);
            // The row is laid out beside its own indent rather than padded
            // inside it: `tap` takes `available_width`, so the space taken
            // here is width the button never claims, and the tappable
            // rectangle a walk observes is the row a thumb actually sees.
            let control = ui
                .horizontal(|ui| {
                    ui.add_space(crate::roster::indent(row.depth));
                    super::tap(ui, label, "transcript")
                })
                .inner;
            // **Where the harness finds a row** (§15.2). Only the first: the
            // walk needs one row to press, and a rectangle per row would be a
            // channel that grows with the world instead of with the app.
            if std::mem::take(&mut first) {
                self.note_control("row", ui, control.rect);
            }
            // The menu is painted before the tap is spent, because it is what
            // decides whether the tap was a navigation at all.
            let opened = self.menu(ui, area, &row, &control);
            if control.clicked()
                && !opened
                && let Some(model) = self.model()
            {
                model.focus_conversation(workspace.to_owned(), row.root_id.clone());
            }
        }
    }

    /// The one field that starts a conversation. It shares the composer's
    /// widget id with the chat screen's, and deliberately: only one of the two
    /// is ever on screen, they are the same gesture at two depths, and the IME
    /// bridge addresses exactly one field by that id (bl-014e).
    fn starter(&mut self, ui: &mut egui::Ui) {
        if let Some(goal) = crate::shell::composer::composer(
            ui,
            &mut self.composer,
            "start a conversation",
            &["prepare", "prompt"],
        ) && let Some(model) = self.model()
        {
            model.start_conversation(goal);
        }
    }
}

/// A row's whole label: who, when, what it last said, and why its latest call
/// did not run (§9.10) where the tone already inks the row — the hue is the
/// engine's reading and this is its words. A `Bad` tone with no clause is the
/// third thing it is, a failure that left none, and says nothing extra.
fn label(row: &ConvRow, now: i64) -> String {
    let mark = if row.attention > 0 {
        super::ATTENTION_MARK
    } else {
        ""
    };
    let when = crate::roster::stamp(row.last_active_unix, now);
    let line = format!("{}{mark} · {when}\n{}", row.display, row.preview);
    match &row.failure {
        Some(why) => format!("{line}\n{why}"),
        None => line,
    }
}
