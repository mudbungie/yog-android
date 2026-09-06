//! **The fork points, as controls** (DESIGN §13.16): the operable notches of
//! this conversation's spine, and the workspace's `config/<name>` heads.
//!
//! **They are the picking surface `fork` was held back for.** The engine's own
//! `fork::Attempt` says *"empty is not a value — a fork with no ref is a
//! different gesture"*, and until this landed nothing on this seat could name
//! one. Both kinds are already read here — the notches by `rail`, the heads by
//! `lineages` — so the surface is a placement, not a new question.
//!
//! **A notch with no commit is not a control.** Upstream states the commit
//! exactly where the notch is reachable, so *unpinnable* is the engine's
//! answer rather than this app's reading, and an unreachable notch paints as a
//! label — the trail row's rule (§13.8) at another site.
//!
//! **What crosses is the whole commit and what paints is the clipped one.** A
//! gesture naming a prefix would be asking the engine to resolve a length this
//! app chose; the short form is for a phone's glass and stays there.
//!
//! **Picking a commit asks what governs there**, and picking a lineage asks
//! nothing: a `config/<name>` head IS a policy, so the answer would be itself.
//! What the anchored read buys is the one fact a fork's role resolves against
//! that the standing read cannot state — the policy at the child's own point.

use eframe::egui;

use crate::codec::{Lineage, Notch, Records};
use crate::shell::app::Shell;
use crate::shell::mark::TOUCH;
use crate::shell::screens::records::parts;

impl Shell {
    /// Every fork point this conversation offers, in the order a thumb meets
    /// them: its own history first, then the policies it could start under.
    pub(super) fn points(&mut self, ui: &mut egui::Ui, records: &Records) {
        for notch in &records.rail.notches {
            self.notch(ui, records, notch);
        }
        for lineage in &records.lineages {
            self.lineage(ui, lineage);
        }
    }

    /// One notch. Operable ones pick; the rest say why they cannot.
    fn notch(&mut self, ui: &mut egui::Ui, records: &Records, notch: &Notch) {
        if notch.commit.is_empty() {
            ui.weak(format!(
                "{} · unpinnable · {} tokens",
                notch.seq, notch.budget
            ));
            return;
        }
        let label = format!(
            "{}{} · {} · {} tokens",
            mark(self, &notch.commit),
            notch.seq,
            notch.short,
            notch.budget
        );
        if self.point(ui, label, notch.commit.clone())
            && let Some(model) = self.model()
        {
            model.anchor(notch.commit.clone());
        }
        // **The answer says which notch it belongs to**, because the commit it
        // was asked at rides beside it — a policy that landed after the
        // operator tapped another notch has no row here to paint under.
        if let Some((at, governing)) = records.anchored.as_ref()
            && at == &notch.commit
        {
            ui.weak(parts::governed(governing));
        }
    }

    /// One lineage head — the other kind of fork point, and the one a child
    /// starts clean under.
    fn lineage(&mut self, ui: &mut egui::Ui, lineage: &Lineage) {
        let refname = format!("config/{}", lineage.name);
        // **No age here, and that is the clock rule** (§13.8): this app has
        // one ladder for *how long ago* and it needs a `now` the paint does
        // not hold. What a fork point has to say is which policy it is and
        // where it stands.
        let label = format!("{}{refname} · {}", mark(self, &refname), lineage.short_oid);
        self.point(ui, label, refname);
    }

    /// One picking control: tapping it makes it the fork's `from`, tapping it
    /// again puts it down. A pick is navigation and carries no `act:` tag,
    /// because it fires no op (§13.10's row). Answers whether this tap PICKED
    /// — an unpick asks nothing.
    fn point(&mut self, ui: &mut egui::Ui, label: String, refname: String) -> bool {
        let picked = self.from.as_ref() == Some(&refname);
        let control =
            ui.add(egui::Button::new(label).min_size(egui::vec2(ui.available_width(), TOUCH)));
        ui.add_space(4.0);
        if !control.clicked() {
            return false;
        }
        self.from = (!picked).then_some(refname);
        !picked
    }
}

/// The picked mark, one spelling for both kinds of point.
fn mark(shell: &Shell, refname: &str) -> &'static str {
    if shell.from.as_deref() == Some(refname) {
        "▸ "
    } else {
        ""
    }
}
