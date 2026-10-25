//! **The screen a search's answer opens** (yog DESIGN §8.5, DESIGN §13.6,
//! bl-4c2b; §24 item 2).
//!
//! **The field is the drawer's now** (`shell/drawer/head.rs`): `search` names
//! no workspace and no conversation — it is the one read this seat makes that
//! asks the server *where to look* — and the drawer is the one surface that
//! is about everything but the focused conversation, so a field there says
//! the search's scope exactly as the top depth's did (§8: the server states,
//! this seat never derives). This overlay keeps its body and loses its field.
//!
//! **The answer is an overlay, not a list swapped underneath one** (DESIGN
//! §22 item 6). It floats in the frame every depth shares, over the chat at
//! whatever depth the focus is, so it inherits the back rule unchanged — the
//! platform's back gesture or its name line closes it — and the walk's probe
//! gets a name for it. Leaving is local: the empty needle crosses no wire
//! (`seat::asks::search`), so a search can be left with the server
//! unreachable.
//!
//! **A hit is an address this seat already focuses**, which is the whole
//! point of upstream's bl-764a: the rows carry the §3.1 workspace leaf and
//! the agent id rather than server-local paths, so tapping one is the same
//! gesture as tapping a row in the drawer — and, like the drawer's, it closes
//! what it was tapped in: the hits stand over every depth, so a hit that left
//! them standing would be a tap nothing visibly answered.
//!
//! **A ball hit paints and does not tap.** There is no ball surface on this
//! device yet (bl-d587), and a row that navigates nowhere is worse than a row
//! that plainly does not: the hit is still shown, because *the server found
//! it* is the answer, and hiding a third of an answer to keep the list
//! tappable would be this app editing the server's reply.

use eframe::egui;

use crate::codec::{Address, Found, Hit};
use crate::overlay::Close;
use crate::seat::Snapshot;
use crate::shell::app::Shell;

impl Shell {
    /// **The hits, inside the overlay frame.** Closing it is the frame's, and
    /// drops the question with the answer; the question is edited again in
    /// the drawer's find field.
    pub(super) fn searched(&mut self, ui: &mut egui::Ui, snap: &Snapshot) {
        let Some(found) = snap.search.clone() else {
            return;
        };
        self.note_screen("search");
        super::banner(ui, snap);
        self.hits(ui, &found);
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
                if super::tap(ui, label.into(), "conversations", "focuses this workspace").clicked()
                {
                    self.focus_workspace(Some(name.clone()));
                    self.close(Close::Unsearch);
                }
            }
            Address::Conversation { workspace, agent } => {
                let label = format!("{workspace}/{agent} · {tier} · {excerpt}");
                if super::tap(
                    ui,
                    label.into(),
                    "transcript",
                    "opens this conversation at the hit",
                )
                .clicked()
                {
                    if let Some(model) = self.model() {
                        model.focus_conversation(workspace.clone(), agent.clone());
                    }
                    self.close(Close::Unsearch);
                }
            }
            // **An address a newer server of this major searches** (REMOTE
            // §3.2). The flat keys under an unknown token are what the token
            // would have said, so none is read and the hit is painted, not
            // tapped — the `Ball` arm's standing exactly.
            Address::Unknown(word) => {
                let at = crate::codec::unknown_label("address", word);
                ui.weak(format!("{at} · {tier} · {excerpt}"));
            }
        }
    }
}
