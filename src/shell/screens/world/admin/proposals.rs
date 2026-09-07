//! **What a reviewer has staged, and the two words that settle it** (DESIGN
//! §13.17, REMOTE §9.22) — the learning loop's veto, on the glass.
//!
//! **It is on the admin screen and not a screen of its own**, because a
//! proposal is a candidate CONFIG commit: the same subject this screen already
//! reads and writes, one branch along. A surface of its own would be a second
//! home for one subject, and the operator who is about to accept a config
//! patch is the operator who wants to read the config it patches.
//!
//! **The listing is a read and the settle is an act, so they sit where each
//! kind sits here** — the rows in the scroll with the destinations, the two
//! verdicts on the foot with the other acts. That is this screen's own layout
//! rule, not a new one.
//!
//! **Tapping a row asks for it WHOLE**, which is the same op at its second
//! depth (`Ask::Proposals`): a row says how big the change is and what the
//! reviewer called it, and the diff is what an operator actually decides on.
//! Tapping it again puts it down and re-reads the bare listing, because a
//! reading of a proposal nobody has picked is a reading under no row.
//!
//! **`reject` is armed and `accept` is not**, and the difference is the one
//! §13.8 already draws. Accepting TAKES what the operator has just read — the
//! `pass` case exactly, where the thing being read before the tap is the whole
//! reason the control is here. Rejecting DISCARDS a durable record somebody
//! else made, which is `clear-trail`'s case, so it takes two taps on one label
//! rather than a dialog a back gesture could answer.
//!
//! **Neither is dark for want of a typed word and both are dark without a
//! pick**: the settle's parameter is a proposal, and a proposal is chosen from
//! the listing rather than typed. A control with no subject says so and stays
//! on the glass (lernie §4.20's rule, which this screen already keeps for
//! `config`).

use eframe::egui;

use crate::codec::proposals::Verdict;
use crate::codec::{AdminAct, Proposal};
use crate::seat::Snapshot;
use crate::shell::app::Shell;
use crate::shell::mark::TOUCH;

/// The op token both halves carry: the read's own, on the rows that spend it.
const READ: &str = "proposals";

impl Shell {
    /// The listing, under the config destinations it belongs beside.
    pub(super) fn staged(&mut self, ui: &mut egui::Ui, snap: &Snapshot, workspace: &str) {
        ui.separator();
        let Some(staged) = snap
            .proposals
            .as_ref()
            .filter(|staged| staged.about(workspace))
        else {
            ui.weak("nothing staged read yet");
            return;
        };
        if staged.rows.is_empty() {
            ui.weak("nothing staged");
        }
        for row in &staged.rows {
            self.staged_row(ui, row);
        }
        // **The whole reading is the picked row's**, and it is painted under
        // the listing rather than under the row: a diff is the tallest thing
        // on this screen, and a row that grew by forty lines would move every
        // row under it out from beneath the thumb that tapped.
        if let Some(whole) = staged.whole.as_deref() {
            ui.separator();
            ui.add(egui::Label::new(whole).wrap());
        }
    }

    /// One staged patch, as a row. The id is the address and the subject is
    /// what a person recognises it by, so both ride the first line; `fresh` is
    /// the engine's own word and is painted as its own state, because a stale
    /// patch was written against a config that no longer governs anything.
    fn staged_row(&mut self, ui: &mut egui::Ui, row: &Proposal) {
        let picked = self.proposal.as_deref() == Some(row.id.as_str());
        let ink = if picked {
            crate::shell::theme::rgb(crate::theme::BRAND)
        } else {
            ui.visuals().text_color()
        };
        let state = if row.fresh {
            crate::theme::State::Rest
        } else {
            crate::theme::State::Annotation
        };
        let label = crate::shell::theme::line(
            ui,
            &[
                (format!("{} · ", row.subject), ink),
                (
                    if row.fresh {
                        "fresh".to_owned()
                    } else {
                        "stale".to_owned()
                    },
                    crate::shell::theme::ink(state),
                ),
                (
                    format!(" · {} · {}", row.diffstat, row.id),
                    ui.visuals().weak_text_color(),
                ),
            ],
        );
        let control = crate::shell::theme::row(ui, label.into());
        crate::shell::act::act(ui, &control, READ);
        if control.clicked() {
            self.proposal = (!picked).then(|| row.id.clone());
            if let Some(model) = self.model() {
                model.list_proposals(self.proposal.clone());
            }
        }
    }

    /// The foot's settle band: accept, then reject armed.
    pub(super) fn settling(&mut self, ui: &mut egui::Ui, workspace: &str) {
        let band = egui::vec2(ui.available_width(), TOUCH);
        ui.allocate_ui_with_layout(
            band,
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                let wide = crate::shell::theme::share(ui, Verdict::ALL.len());
                for verdict in Verdict::ALL {
                    self.settle(ui, workspace, verdict, wide);
                }
            },
        );
    }

    /// One verdict. Dark with no proposal picked, saying what would light it;
    /// `reject` states its arming in its own label, which is where this app
    /// puts an arming (§13.8).
    fn settle(&mut self, ui: &mut egui::Ui, workspace: &str, verdict: Verdict, wide: f32) {
        let arms = verdict == Verdict::Reject;
        let Some(id) = self.proposal.clone() else {
            let dark = crate::shell::theme::chip(
                ui,
                format!("{} — tap a proposal", verdict.word()).into(),
                wide,
                false,
            );
            crate::shell::act::act(ui, &dark, "proposal");
            return;
        };
        let label = if arms && self.armed {
            format!("{} — tap again", verdict.word())
        } else {
            verdict.word().to_owned()
        };
        let control = crate::shell::theme::chip(ui, label.into(), wide, true);
        crate::shell::act::act(ui, &control, "proposal");
        if !control.clicked() {
            return;
        }
        if arms && !self.armed {
            self.armed = true;
            return;
        }
        self.armed = false;
        // The pick goes with the act: what it addressed is settled, and a row
        // that no longer exists is not one an operator should still be aiming
        // the other verdict at.
        self.proposal = None;
        if let Some(model) = self.model() {
            model.admin(AdminAct::Proposal {
                workspace: workspace.to_owned(),
                id,
                verdict,
            });
        }
    }
}
