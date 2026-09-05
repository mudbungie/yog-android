//! **The conversation list, and the acts its rows carry** (DESIGN §13.5,
//! bl-f97c). Split from `screens.rs` when the row menu landed, on the seam
//! that file already had: everything left there is a list of taps and a
//! standing, and this is the one screen above the transcript with mechanics of
//! its own.
//!
//! **The gesture is a long press, and egui already synthesizes it.** A touch
//! held past `max_click_duration` (0.8 s) sets `LONG_TOUCHED` on the widget
//! under it, which `Response::secondary_clicked` reports — the same predicate
//! a desktop right-click sets, which is why the two seats can land one design.
//! The two are exclusive rather than layered: `could_any_button_be_click` goes
//! false the moment the press outruns that duration, so the release of a long
//! press is not also a click, and opening the menu never navigates into the
//! conversation as well. **Verified on the emulator, not assumed** — the walk
//! opens this menu with `input motionevent DOWN`, a wait, and `UP`, and the
//! parity inventory it pulls afterwards is the evidence the items painted.
//!
//! **The composer is the parameter.** Two of the three acts need text —
//! interrupt's content, flag's reason — and this app already has exactly one
//! place text is typed (§13.2's one shared row). So an item that needs text
//! spends what is in the composer, and is DISABLED with the reason stated
//! beside it when there is none: a greyed control says a thing is not live and
//! nothing about what would make it live (the desktop's §4.20 reading, which
//! holds here). Nothing is staged and nothing is armed — §13.2's *tap is the
//! act* — because none of these three destroys anything: an interrupt keeps
//! what is committed, a retarget discards nothing, and a flag changes nothing
//! else. The first row act whose product is that its subject is gone is where
//! this app earns an arming, and it is not one of these.
//!
//! **The menu is a popup, so it obeys `shell::place`** (bl-78c2) — the same
//! rule, at a fourth site. egui's own `Popup::context_menu` would open at the
//! pointer and fall back against `Context::content_rect`, which on Android is
//! the whole display, gesture-nav zone included; a menu opened from a row near
//! the floor would paint where taps never reach the app. So it is assembled
//! here from `Popup::menu`'s pieces with the side and the cap `place::fit`
//! decided, exactly as `controls/drop.rs` does — and anchored to the ROW
//! rather than to the finger, which is what makes the two the same geometry
//! and lets one assertion cover both.

use eframe::egui;

use crate::codec::{ConvRow, RowAct};
use crate::seat::Snapshot;
use crate::shell::app::Shell;
use crate::shell::mark::{Back, TOUCH};
use crate::shell::place::{Band, fit};

/// The space between a row and its menu. `Popup::menu`'s own default is zero
/// — a menu hangs off what opened it — and it is named here because the
/// placement arithmetic must be told the same number egui is.
const GAP: f32 = 0.0;

/// **What a row's long press offers**, in the order a thumb meets them: the
/// chat staple first, then the two that need no reading of the conversation to
/// decide. The empty forms are the roster; the composer's text is put into
/// whichever field each takes at the moment it fires (`RowAct::with`).
///
/// `fork` is absent from this roster: its fork point is a ref no read this
/// seat makes can name, so it would be an item that cannot fire (`codec::row`,
/// bl-99fd).
///
/// **The floor pair is here and the answer is not** (§13.7, bl-b39d). Revoking
/// a conversation's tool auto-approval needs nothing typed and nothing read —
/// it is standing policy on a conversation, this menu's exact class — while
/// answering a parked call means READING the call first, so it lives on the
/// transcript screen where the call is (`controls/held.rs`).
fn roster() -> [RowAct; 5] {
    [
        RowAct::Interrupt {
            content: String::new(),
        },
        RowAct::Retarget,
        RowAct::Flag {
            reason: String::new(),
        },
        RowAct::Revoke,
        RowAct::Restore,
    ]
}

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
                // The one ball read that names a place (§13.9), offered where
                // that place is what the operator is standing in.
                self.balls_entry(ui, crate::codec::View::Here);
                // The other aimed read (§13.12): what each attempt on this
                // workspace's obligations cost, offered where that workspace
                // is what the operator is standing in.
                self.candidates_entry(ui);
                // And the workspace's two armings (§13.13) — the loop that
                // runs its ready balls, and the monitor over what they commit.
                self.fleet_entry(ui);
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

    /// The rows themselves. Newest first, and each says when (REMOTE §9.9,
    /// bl-e837).
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
            let control = super::tap(ui, label, "transcript");
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

    /// **One row's context menu.** Answers whether a menu stood over this row
    /// as the frame began — a tap that closed one is not also a navigation
    /// into it, which is the one thing a menu owes the screen under it.
    fn menu(
        &mut self,
        ui: &mut egui::Ui,
        area: Band,
        row: &ConvRow,
        control: &egui::Response,
    ) -> bool {
        let id = egui::Popup::default_response_id(control);
        let was = egui::Popup::is_id_open(ui.ctx(), id);
        let popup = egui::Popup::menu(control)
            // `Popup::menu`'s own toggle is a primary click, which on this
            // screen is the navigation. The long press is the opening, and a
            // tap on the row closes what is open.
            .open_memory(if control.secondary_clicked() {
                Some(egui::SetOpenCommand::Bool(true))
            } else if control.clicked() {
                Some(egui::SetOpenCommand::Bool(false))
            } else {
                None
            })
            .gap(GAP)
            .close_behavior(egui::PopupCloseBehavior::CloseOnClick);
        // What the menu took last time it was laid out. Before the first one
        // there is no answer, and infinity is the honest one: an unmeasured
        // list wants everything, so it is handed the room and capped to it.
        let wanted = popup
            .get_expected_size()
            .map_or(f32::INFINITY, |size| size.y);
        let anchor = Band {
            top: control.rect.top(),
            bottom: control.rect.bottom(),
        };
        let Some(placed) = fit(area, anchor, GAP, wanted) else {
            // Neither side of this row has room for a menu. Opening one would
            // put it where taps do not land, which is the defect, so nothing
            // opens — and the row is still a row, so the tap still navigates.
            return was;
        };
        let chrome = egui::Frame::popup(ui.style()).total_margin().sum().y;
        let agent = row.root_id.clone();
        popup
            .align(if placed.above {
                egui::RectAlign::TOP_START
            } else {
                egui::RectAlign::BOTTOM_START
            })
            // No alternatives: egui's fallback search judges fit against the
            // display, and the display is what puts a list under the nav bar.
            .align_alternatives(&[])
            .show(|ui| {
                egui::ScrollArea::vertical()
                    .max_height((placed.height - chrome).max(0.0))
                    .show(ui, |ui| {
                        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                        for act in roster() {
                            self.item(ui, &agent, &act);
                        }
                    });
            });
        was
    }

    /// One item. The label is the wire's own op token and so is the `act:` tag
    /// it carries (`RowAct::op`) — one name, so the paint cannot show a word
    /// and post another.
    fn item(&mut self, ui: &mut egui::Ui, agent: &str, act: &RowAct) {
        let wants = act.wants();
        let typed = !self.composer.trim().is_empty();
        let live = wants.is_none_or(|_| typed);
        let label = match wants {
            Some(ask) if !typed => format!("{} — {ask}", act.op()),
            _ => act.op().to_owned(),
        };
        let control = ui.add_enabled(
            live,
            egui::Button::new(label).min_size(egui::vec2(0.0, TOUCH)),
        );
        // The tag rides the item that FIRES the op, disabled or not: what it
        // records is that the control was laid out and its rectangle was on
        // the glass, which a disabled item's is (PARITY §4, `shell::act`).
        crate::shell::act::act(ui, &control, act.op());
        if control.clicked() {
            // Only an act that takes a parameter spends the composer: a
            // retarget that emptied the field would eat a draft it never read.
            let text = if wants.is_some() {
                std::mem::take(&mut self.composer)
            } else {
                String::new()
            };
            if let Some(model) = self.model() {
                model.row_act(agent.to_owned(), act.with(text));
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
