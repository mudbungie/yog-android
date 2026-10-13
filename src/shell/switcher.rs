//! **The switcher** (DESIGN §22 item 7, bl-3edc): the workspaces and the
//! focused one's conversations as one list in the overlay's frame, the seat
//! panel at its head. Which rows, in which order and what a tap yields are
//! `crate::switcher`, pure and host-tested; this file paints them and
//! performs the taps.
//!
//! **Its head is the seat panel**, because this is the screen every back
//! lands on: the mark (the way to the configuration surface, until the rule
//! carries it — 4/7), this device's identity, what it is hosting, and the
//! release channel's offer (§20.4) — whose once-per-process check starts on
//! the first paint of this panel.
//!
//! **A long press on a conversation keeps the §13.5 row menu**: it acts on
//! THAT row, which no verb on the focus can.

use eframe::egui;

use super::app::Shell;
use super::place::Band;
use super::screens::{ATTENTION_MARK, tap};
use super::theme::{self, ink, rgb};
use crate::overlay::{Close, Overlay};
use crate::seat::Snapshot;
use crate::switcher::{Entry, NEW, Taken, entries};
use crate::theme::{State, glyph};

impl Shell {
    /// The switcher's body, under its name line.
    pub(super) fn switcher(&mut self, ui: &mut egui::Ui, snap: &Snapshot) {
        self.note_screen(Overlay::Switcher.name());
        self.seat_panel(ui);
        super::screens::banner(ui, snap);
        ui.separator();
        // The menu is a popup and obeys `shell::place` against the band the
        // list is painted in (bl-78c2), read before the scroller narrows it.
        let area = Band {
            top: ui.max_rect().top(),
            bottom: ui.max_rect().bottom(),
        };
        let listed = entries(
            &snap.workspaces,
            snap.focus.workspace.as_deref(),
            &snap.conversations,
        );
        let mut taken = None;
        egui::ScrollArea::vertical()
            .min_scrolled_height(0.0)
            .show(ui, |ui| {
                let now = crate::roster::now_unix();
                let mut first = true;
                for entry in &listed {
                    if self.entry(ui, area, entry, now, &mut first) {
                        taken = entry.taken();
                    }
                }
            });
        if let Some(taken) = taken {
            self.take(taken);
        }
    }

    /// The panel: the mark beside who this device is, what it hosts, and the
    /// update offer when one stands.
    fn seat_panel(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            self.mark(ui);
            let who = egui::RichText::new(self.identity()).color(rgb(crate::theme::INK_WEAK));
            ui.add(egui::Label::new(who).truncate());
        });
        Self::hosting(ui);
        self.update_entry(ui);
    }

    /// One row, painted; answers whether it was tapped as a navigation.
    fn entry(
        &mut self,
        ui: &mut egui::Ui,
        area: Band,
        entry: &Entry,
        now: i64,
        first: &mut bool,
    ) -> bool {
        match entry {
            Entry::Heading {
                workspace,
                agents,
                waiting,
                open,
            } => {
                // The fold glyph, the name in ink, the mark in the attention
                // accent while anything in it waits, the count in weak ink.
                let fold = if *open { glyph::OPEN } else { glyph::SHUT };
                let mut parts = vec![(format!("{fold} {workspace}"), rgb(crate::theme::INK))];
                if *waiting {
                    parts.push((ATTENTION_MARK.to_owned(), ink(State::Attention)));
                }
                parts.push((format!("  {agents} agents"), rgb(crate::theme::INK_WEAK)));
                let label = theme::line(ui, &parts);
                tap(ui, label.into(), "conversations").clicked()
            }
            Entry::Conversation { row, rails, .. } => {
                let hue = super::chat::tone_hue(ui, &row.tone);
                let lines = crate::roster::lines(row, now);
                let control = super::screens::threaded(ui, rails, &lines, hue);
                // **Where the harness finds a row** (§15.2): the first only.
                if std::mem::take(first) {
                    self.note_control("row", ui, control.rect);
                }
                // The menu decides whether the tap was a navigation at all.
                let opened = self.menu(ui, area, row, &control);
                control.clicked() && !opened
            }
            Entry::New { .. } => {
                let word = egui::RichText::new(NEW).color(rgb(crate::theme::BRAND));
                let control = theme::row(ui, word.into());
                self.note_control("new", ui, control.rect);
                control.clicked()
            }
        }
    }

    /// What a tap yielded, performed. A heading keeps the switcher standing
    /// whatever depth its focus leaves under it; the other two close it.
    fn take(&mut self, taken: Taken) {
        let Some(model) = self.model() else { return };
        match taken {
            Taken::Focus(workspace) => {
                model.focus_workspace(Some(workspace));
                self.overlay = Some(Overlay::Switcher);
            }
            Taken::Open { workspace, agent } => {
                model.focus_conversation(workspace, agent);
                self.close(Close::Leave);
            }
            Taken::Start(workspace) => {
                model.focus_workspace(Some(workspace));
                self.close(Close::Leave);
                self.summon = true;
            }
        }
    }
}
