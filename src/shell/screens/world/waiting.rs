//! **The queue as a queue** (DESIGN §13.8) — the whole-queue view behind the
//! per-row attention marks this seat paints on two other screens.
//!
//! **A row navigates, because a row is an address.** The workspace and the
//! agent cross in the same words every gesture takes (`codec::queue`), so
//! tapping one opens that conversation exactly as a search hit does — no
//! resolution, no derivation, and nothing this app had to work out.
//!
//! **What a row says is the engine's, painted as it came.** The signals are
//! its own tokens, carried through as strings on purpose (a table here would
//! refuse a whole answer the day a ninth signal is minted), and the held
//! call's sentence crosses unrewritten for REMOTE §8.1's reason — rewriting
//! it *"would put a different call in front of the operator"*.

use eframe::egui;

use crate::codec::QueueRow;
use crate::seat::Snapshot;
use crate::shell::app::Shell;
use crate::shell::mark::Back;

impl Shell {
    pub(super) fn waiting(&mut self, ui: &mut egui::Ui, snap: &Snapshot) {
        self.note_screen("attention");
        if self.bar(ui, "waiting", &Back::To("workspaces")) {
            self.close_world();
        }
        super::super::banner(ui, snap);
        ui.separator();
        egui::ScrollArea::vertical()
            .min_scrolled_height(0.0)
            .show(ui, |ui| {
                if snap.queue.is_empty() {
                    ui.weak("nothing is waiting on you");
                }
                for row in snap.queue.clone() {
                    self.row(ui, &row);
                }
            });
    }

    /// One waiting conversation: the tap that opens it, then the engine's own
    /// words for why it is here.
    fn row(&mut self, ui: &mut egui::Ui, row: &QueueRow) {
        let label = format!(
            "{}/{} · {}\n{}",
            row.workspace,
            row.display,
            crate::roster::ago(row.age_secs),
            row.preview
        );
        if super::super::tap(ui, label.into(), "transcript").clicked()
            && let Some(model) = self.model()
        {
            model.focus_conversation(row.workspace.clone(), row.agent.clone());
            self.close_world();
        }
        if !row.signals.is_empty() {
            ui.weak(row.signals.join(" · "));
        }
        if let Some(held) = &row.held {
            ui.weak(format!("held · {} · {}", held.tool, held.reason));
        }
        if let Some(flag) = &row.flag {
            ui.weak(format!("flagged · {}", flag.reason));
        }
        if let Some(failure) = &row.failure {
            ui.weak(format!("failed · {failure}"));
        }
        ui.add_space(4.0);
    }
}
