//! **The ball pane's five acts** (DESIGN §13.9, bl-f36e): the half that
//! CHANGES the store, beside the reads its parent paints.
//!
//! **They hang on the AIMED view and nowhere else.** `workspace-balls` is the
//! one of the three reads that names a workspace, and the `--as` stamp every
//! `bl` verb carries IS a workspace name (lernie DESIGN §4.35) — so on that
//! screen the stamp is the pane's own subject and there is nothing to invent.
//! `balls` and `board` are opened from the roster with no workspace focused,
//! so an act there would be stamped with a name this seat made up, or with a
//! claimant read off a row for a claim it is not making. That is the desktop's
//! own placement seen from a phone: there, four of the five sit in the aimed
//! wall's section and `assign` sits on a board row *because the wall it would
//! be claimed for is the aim*; here the board is opened from the roster, where
//! there is no aim to be it.
//!
//! **Every act is addressed at the row it hangs on, `create` included.** The
//! project is the one fact these five need that only a ROW carries — a
//! workspace row states no project (`codec::ws`), and this seat reads nothing
//! else that does — so filing a new ball is filing one *in the project of the
//! ball under the thumb*. That is the rule the pane already keeps rather than
//! an exception to it, and it is why there is no project box: a picker for a
//! project this seat cannot enumerate would be this app asking the operator to
//! be a read it does not have.
//!
//! **The title is the composer's** (§13.2's one shared row, and §13.5's rule
//! at a second site): two of the five need text, this app types text in one
//! place, and an act that needs it is DISABLED with the reason stated beside
//! it when there is none. Nothing else is composed here — a ball's body and an
//! update's note are prose, and a phone types a title.
//!
//! **`close` is armed and the other four are not** (lernie §4.35's test, which
//! transfers: *undone by doing the other thing*). A filing is undone by
//! releasing or closing it, an amendment by writing the old words back, a
//! release by an assign. A close folds the trunk in, squashes and removes the
//! worktree, and no verb reverses it — so it takes the arming this app already
//! has exactly one of (`clear-trail`, §13.8): two taps on one control, spelled
//! in the label, cleared by leaving the screen and by the acts beside it.

use eframe::egui;

use crate::codec::BallAct;
use crate::shell::app::Shell;
use crate::shell::mark::TOUCH;

/// The empty forms, in the order a thumb meets them: what an operator does to
/// a ball, then what they write, and the one with no undoing last.
fn roster() -> [BallAct; 5] {
    [
        BallAct::Assign { id: String::new() },
        BallAct::Release { id: String::new() },
        BallAct::Update {
            id: String::new(),
            title: None,
            body: None,
            note: None,
        },
        BallAct::Create {
            title: String::new(),
            body: None,
        },
        BallAct::Close { id: String::new() },
    ]
}

/// The sentence the foot states while no row is picked. One sentence over the
/// row rather than one per control, because it is one fact: these acts address
/// a ball, and no ball is under the thumb.
const PICK: &str = "tap a ball to act on it";

impl Shell {
    /// The foot of the aimed pane: the title field, then the five controls.
    /// Painted whatever the pane holds — a control that vanished when a read
    /// failed would say the acts were a property of the answer rather than of
    /// the screen.
    pub(super) fn ball_acts(&mut self, ui: &mut egui::Ui) {
        // **One plain row, neither scrolled nor wrapped**, and both of the
        // alternatives were measured on the emulator rather than reasoned
        // about. A horizontal `ScrollArea` puts whatever does not fit off the
        // glass, and a control off the glass is one the parity inventory
        // cannot record and a thumb cannot reach — `close` and `create` were
        // the two that fell off. `horizontal_wrapped` inside this bottom-up
        // layout paints past the floor `app::pass` shrank the rect to, into
        // the gesture-nav zone where taps never reach the app (bl-9cfd's
        // defect, at a new site): the trail's foot ended at 2336 device
        // pixels and this one ran to 2399. And a bare `ui.horizontal` does the
        // same, which is the finding worth keeping: a row inside a BOTTOM-UP
        // layout is placed against a height egui guessed before it laid the
        // row out, so a row of §13.2 touch-floor controls hangs below the
        // cursor by the difference. The row is therefore allocated its own
        // band first — `shell::composer`'s answer to bl-193c, at a second
        // site, and for the same reason: a child that does not state its
        // height lets the layout decide it wrong.
        let band = egui::vec2(ui.available_width(), TOUCH);
        ui.allocate_ui_with_layout(
            band,
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                for act in roster() {
                    self.ball_item(ui, &act);
                }
            },
        );
        self.title(ui);
        // **The pick's sentence is said once, over the row.** It is one fact
        // about the screen — no ball is under the thumb — rather than five
        // facts about five controls, and five copies of it would be a foot
        // taller than the listing above it.
        if self.ball.is_none() {
            ui.weak(PICK);
        }
    }

    /// One control. The label is the wire's own op token and so is the `act:`
    /// tag it carries (`BallAct::op`) — one name, so the paint cannot show a
    /// word and post another.
    fn ball_item(&mut self, ui: &mut egui::Ui, act: &BallAct) {
        let picked = self.ball.clone();
        let typed = !self.composer.trim().is_empty();
        let wants = act.wants();
        let live = picked.is_some() && wants.is_none_or(|_| typed);
        let arming = matches!(act, BallAct::Close { .. });
        let label = match wants {
            Some(ask) if picked.is_some() && !typed => format!("{} — {ask}", act.op()),
            _ if arming && self.armed => format!("{} · tap again", act.op()),
            _ => act.op().to_owned(),
        };
        let control = ui.add_enabled(
            live,
            egui::Button::new(label).min_size(egui::vec2(0.0, TOUCH)),
        );
        // The tag rides the control that FIRES the op, disabled or not: what
        // it records is that the control was laid out and its rectangle was on
        // the glass, which a disabled one's is (PARITY §4, `shell::act`).
        crate::shell::act::act(ui, &control, act.op());
        if !control.clicked() {
            return;
        }
        // An arm nobody spent is dropped by the act beside it: two controls
        // holding one bool must not let a tap on one leave the other armed.
        let was = std::mem::take(&mut self.armed);
        if arming && !was {
            self.armed = true;
            return;
        }
        let Some((project, id)) = picked else { return };
        // Only an act that takes text spends the composer: a release that
        // emptied the field would eat a draft it never read.
        let text = if wants.is_some() {
            std::mem::take(&mut self.composer)
        } else {
            String::new()
        };
        if let Some(model) = self.model() {
            model.ball_act(project, act.on(id, text));
        }
    }

    /// The title field. It shares the composer's widget id and its text, for
    /// the starter's reason exactly (§13.2, `screens::rows`): only one of them
    /// is ever on screen, and the IME bridge addresses one field by that id.
    fn title(&mut self, ui: &mut egui::Ui) {
        ui.add(
            egui::TextEdit::singleline(&mut self.composer)
                .id(egui::Id::new(crate::shell::app::COMPOSER.id))
                .desired_width(f32::INFINITY)
                .margin(crate::shell::composer::padding(ui))
                .hint_text("a ball's title"),
        );
    }
}
