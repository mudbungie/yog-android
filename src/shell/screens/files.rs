//! **The files screen** (DESIGN §13.15): the focused conversation's worktree,
//! one depth behind the transcript — what an agent has written, read from the
//! glass.
//!
//! **It is a depth of the transcript**, the records screen's placement rule
//! (§13.11) at a second site: `files` names a CONVERSATION, which is the
//! deepest focus this app has, so the screen opens over the transcript, backs
//! out into it, and the focus underneath never moves.
//!
//! **Opening is the ask**, and a file row is the same ask one depth down: a
//! listing and one entry's bytes are one question at two depths, so tapping a
//! row re-asks `files` with that path and the answer carries the listing back
//! with it. Nothing here merges a preview into a listing it was not read
//! beside.
//!
//! **The answer says which row its bytes belong under.** A `files` reply
//! carries a preview and no path, so the fold names the path from the ask
//! (`seat::asks::review`) and the paint puts the bytes under exactly that
//! row — the `step` drill-in's guarantee, bought at the fold.
//!
//! **A directory paints and does not tap.** It has no bytes, and upstream
//! resolves `path` against the listing it just built, so asking for one would
//! be a gesture with no answer. That is the trail row's rule (§13.8) at
//! another site.

use eframe::egui;

use crate::codec::{FileRow, Files};
use crate::seat::Snapshot;
use crate::shell::app::Shell;
use crate::shell::mark::{Back, TOUCH};

/// The screen's name, the op it asks and the harness's tap target — one word,
/// because the surface has one read (§15.2).
pub(crate) const SCREEN: &str = "files";

impl Shell {
    /// The whole screen. No foot: every act here is a row, so there is
    /// nothing to anchor to the floor and the listing takes the screen.
    pub(super) fn files(
        &mut self,
        ui: &mut egui::Ui,
        snap: &Snapshot,
        workspace: &str,
        agent: &str,
    ) {
        self.note_screen(SCREEN);
        let held = snap
            .files
            .clone()
            .filter(|files| files.about(workspace, agent));
        if self.bar(ui, SCREEN, &Back::To("transcript")) {
            self.close_files();
        }
        super::banner(ui, snap);
        ui.separator();
        egui::ScrollArea::vertical()
            .min_scrolled_height(0.0)
            .show(ui, |ui| match held {
                None => {
                    ui.weak("nothing read yet");
                }
                Some(files) => self.walked(ui, &files),
            });
    }

    /// The listing, with the two facts that are about the whole of it: where
    /// this conversation's work actually lands when it is not here, and
    /// whether the walk was cut short.
    fn walked(&mut self, ui: &mut egui::Ui, files: &Files) {
        if !files.listing.worktree {
            ui.weak("the worktree is gone — nothing to list");
            return;
        }
        if !files.listing.working_dir.is_empty() {
            ui.weak(format!(
                "work lands at {} — not in this listing",
                files.listing.working_dir
            ));
        }
        if files.listing.rows.is_empty() {
            ui.weak("the worktree holds nothing");
        }
        for row in &files.listing.rows {
            self.file_row(ui, files, row);
        }
        if files.listing.truncated {
            ui.weak("the listing was cut short");
        }
    }

    /// One entry. A directory is a label; a file is a control that asks for
    /// its own bytes, and the bytes paint under the row they were asked for.
    fn file_row(&mut self, ui: &mut egui::Ui, files: &Files, row: &FileRow) {
        let label = line(row);
        if row.dir {
            ui.weak(label);
        } else {
            let control =
                ui.add(egui::Button::new(label).min_size(egui::vec2(ui.available_width(), TOUCH)));
            crate::shell::act::act(ui, &control, SCREEN);
            if control.clicked()
                && let Some(model) = self.model()
            {
                model.open_files(Some(row.path.clone()));
            }
        }
        if files.opened == row.path
            && let Some(preview) = files.listing.preview.as_ref()
        {
            super::preview::bytes(ui, preview);
        }
        ui.add_space(4.0);
    }

    /// **Open it** — and that is the ask, the records screen's rule (§13.11).
    pub(in crate::shell) fn open_files(&mut self) {
        self.files = true;
        if let Some(model) = self.model() {
            model.open_files(None);
        }
    }

    /// Leave, back to the transcript.
    fn close_files(&mut self) {
        self.files = false;
    }
}

/// One entry's label: what it is, and how big. A directory says so instead of
/// a size, because a directory's size is the walk's own bookkeeping and not a
/// fact about the tree.
fn line(row: &FileRow) -> String {
    if row.dir {
        format!("{}/", row.path)
    } else {
        format!("{} · {} bytes", row.path, row.size)
    }
}
