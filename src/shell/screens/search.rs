//! **The search field, and the screen its answer opens** (yog DESIGN §8.5,
//! DESIGN §13.6, bl-4c2b).
//!
//! **The field's depth states the query's scope.** `search` names no
//! workspace and no conversation — it is the one read this seat makes that
//! asks the engine *where to look* — so it lives at the top depth, where the
//! whole world is already what is on the glass. A field on the conversation
//! list would say the search was scoped to that workspace, which the wire
//! does not offer and this app must not imply (§8: the engine states, this
//! seat never derives).
//!
//! **The answer is a screen, not a list swapped underneath one.** It paints
//! its own bar and so inherits §13.2's back rule unchanged — the platform's
//! back gesture walks out of a search exactly as it walks out of a
//! conversation — and the walk's probe gets a name for it. Leaving is local:
//! the empty needle crosses no wire (`seat::asks::search`), so a search can
//! be left with the engine unreachable.
//!
//! **A hit is an address this seat already focuses**, which is the whole
//! point of upstream's bl-764a: the rows carry the §3.1 workspace leaf and
//! the agent id rather than engine-local paths, so tapping one is the same
//! gesture as tapping a row in a list. The answer stands while the operator
//! is away in it — backing out of a conversation lands on the hits again
//! rather than on a roster that has forgotten the question.
//!
//! **A ball hit paints and does not tap.** There is no ball surface on this
//! device yet (bl-d587), and a row that navigates nowhere is worse than a row
//! that plainly does not: the hit is still shown, because *the engine found
//! it* is the answer, and hiding a third of an answer to keep the list
//! tappable would be this app editing the engine's reply.

use eframe::egui;

use crate::codec::{Address, Found, Hit};
use crate::seat::Snapshot;
use crate::shell::app::{NEEDLE, Shell};
use crate::shell::mark::{Back, TOUCH};

impl Shell {
    /// **The top depth's two screens**: the workspace roster, and the hits
    /// when an answer is standing over it. One arm each, and each names the
    /// screen it chose (`app/probe.rs` — the name lives at the branch, never
    /// derived a second time from the same state).
    pub(super) fn top(&mut self, ui: &mut egui::Ui, snap: &Snapshot) {
        let Some(found) = snap.search.clone() else {
            self.note_screen("roster");
            self.bar(ui, &crate::bootstrap::Component::Seat.brand(), &Back::None);
            self.needle(ui);
            super::banner(ui, snap);
            self.roster(ui, snap);
            return;
        };
        self.note_screen("search");
        if self.bar(ui, "search", &Back::To("workspaces")) {
            self.clear_search();
        }
        self.needle(ui);
        super::banner(ui, snap);
        self.hits(ui, &found);
    }

    /// The field and the control that fires it, on both of those screens: the
    /// question is editable wherever its answer is shown, so refining a
    /// search is one tap rather than a walk back out of it.
    ///
    /// Laid right-to-left for the composer's reason (`shell::composer`): the
    /// button claims its seat first and the field takes what is left, which
    /// is the only order in which a full-width field does not push the
    /// control off the row.
    fn needle(&mut self, ui: &mut egui::Ui) {
        let mut fired = false;
        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), TOUCH),
            egui::Layout::right_to_left(egui::Align::Center),
            |ui| {
                let control = egui::Button::new("search").min_size(egui::vec2(0.0, TOUCH));
                let response = ui.add(control);
                crate::shell::act::act(ui, &response, "search");
                fired = response.clicked();
                // The field's own box is the touch floor, not just the band
                // it sits in: `composer::padding` is the one derivation of
                // that, and a thumb aimed anywhere in this row lands in the
                // field rather than beside a thin box centred in it.
                let pad = crate::shell::composer::padding(ui);
                ui.add(
                    egui::TextEdit::singleline(&mut self.needle)
                        .id(egui::Id::new(NEEDLE.id))
                        .desired_width(f32::INFINITY)
                        .margin(pad)
                        .hint_text("find anything"),
                );
            },
        );
        if fired {
            self.fire_search();
        }
    }

    /// Ask, with whatever is in the field. An empty one is the clear, and the
    /// model knows that — nothing here decides which gesture this is.
    fn fire_search(&self) {
        if let Some(model) = self.model() {
            model.search(self.needle.clone());
        }
    }

    /// Drop the answer and the question with it: leaving a search leaves it.
    fn clear_search(&mut self) {
        self.needle = String::new();
        self.fire_search();
    }

    /// The answer: what could not be read, then the ranked hits.
    ///
    /// **Nothing matched is a sentence, not an empty list** (upstream
    /// bl-648a): the answer carries its own needle, so this screen can say
    /// which question came back empty instead of painting a blank that reads
    /// as a search that never happened.
    fn hits(&mut self, ui: &mut egui::Ui, found: &Found) {
        ui.separator();
        egui::ScrollArea::vertical()
            .min_scrolled_height(0.0)
            .show(ui, |ui| {
                for why in &found.unreadable {
                    ui.weak(format!("unreadable · {why}"));
                }
                if found.hits.is_empty() {
                    ui.weak(format!("nothing matched {:?}", found.needle));
                    return;
                }
                for hit in &found.hits {
                    self.hit(ui, hit);
                }
            });
    }

    /// One hit: a tap that focuses what it names, or — for a ball — the line
    /// itself, because this device has nowhere to open one (bl-d587).
    ///
    /// The label carries the matched field's own word beside the excerpt: a
    /// name matching a name would otherwise read as the same string twice
    /// with nothing saying why the row is there.
    fn hit(&mut self, ui: &mut egui::Ui, hit: &Hit) {
        let tier = hit.field.word();
        let excerpt = &hit.excerpt;
        match &hit.at {
            Address::Ball { project, id } => {
                ui.weak(format!("{project}/{id} · {tier} · {excerpt}"));
            }
            Address::Workspace { name } => {
                let label = format!("{name} · {tier} · {excerpt}");
                if super::tap(ui, label.into(), "conversations").clicked() {
                    self.focus_workspace(Some(name.clone()));
                }
            }
            Address::Conversation { workspace, agent } => {
                let label = format!("{workspace}/{agent} · {tier} · {excerpt}");
                if super::tap(ui, label.into(), "transcript").clicked()
                    && let Some(model) = self.model()
                {
                    model.focus_conversation(workspace.clone(), agent.clone());
                }
            }
        }
    }
}
