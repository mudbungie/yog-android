//! **The sign-in screen** (REMOTE §8.3, DESIGN §13.19, §24 item 5): the
//! workspace's sign-in catalog, the act that signs one row in, and the held
//! tail of whatever that run is saying — with the tail's address and code
//! made into doors a thumb can use.
//!
//! **It is a depth, not a control on the composer's row.** The provider
//! selector under the composer states each row's credential fact and stays
//! tappable *because* an operator may be about to sign it in (§13.2) — and
//! what a sign-in produces is a URL to follow, a device code to type and, when
//! it fails, a command to run by hand. None of that fits in a popup over a
//! composer, and a band under it would push the transcript off the glass for
//! the minutes a browser flow takes. So it is an aimed entry beside the admin
//! screen: one workspace, its providers, and room to read.
//!
//! **The rows are the CATALOG, not the routing table** (bl-892e). The
//! selector under the composer reads `snap.providers` — the rows a run can
//! route to — and a built-in preset nobody has signed into is absent from
//! it, so a fresh workspace's table is empty and a screen painted from it
//! would offer nothing to sign into. This screen reads `snap.catalog`: every
//! row carrying a sign-in flow, signed in or not, in the same row shape with
//! the server's own sentences on it, and derives nothing. Opening this
//! screen IS the `catalog` ask, exactly as opening the records screen is its
//! five.
//!
//! **Tapping a row opens its tail; the button beside it starts a run — and
//! watches it.** The two are different gestures and the wire says so:
//! `login-tail` is a held READ and `login` is the act. Starting a run is also
//! asking to watch it, which the worker does at the act's receipt
//! (`seat::worker::after::signed`), so the button carries both tags and the
//! URL stands under the row without a second tap. A row that cannot be signed
//! in shows the server's reason where the verb would be, and its button is
//! dark — the greying discipline the selector already keeps (§13.2).
//!
//! **The sign-in finishes on the phone** (§24 item 5). A tail line is read by
//! `crate::signin` — pure, host-tested, every word on a door spelled there —
//! and painted here: an address is a `TOUCH`-tall door in the brand whose tap
//! opens the phone's browser through the one typed door the device has
//! (`tools::paper::to_browser`, the `open` tool's own `dev.yog.Open`) and
//! whose long press copies it; a device code earns a `copy <code>` door at the
//! line's end. What a credential landing means is said under the row while
//! its tail is open: it signs in the SERVER, for every seat of the workspace.
//!
//! **There is one tail, under the row it belongs to.** The worker holds which
//! provider is watched (`seat::pass::login`) and the snapshot carries it back
//! with the lines, so which row is open has ONE home and this screen paints
//! rather than remembers it — no local pick to drift from the lane's subject.
//!
//! The walk reaches the first row at `signin-row`, its button at
//! `signin-start`, the first address door at `signin-url` and the first code
//! door at `signin-code` (`app::probe`).

use eframe::egui;

use crate::codec::{LoginView, ProviderRow};
use crate::seat::Snapshot;
use crate::shell::app::Shell;
use crate::shell::mark::TOUCH;
use crate::shell::theme::rgb;
use crate::shell::{clip, theme};
use crate::signin::{self, Part};
use crate::theme::space;

/// The screen's name, the harness's tap target (§15.2), and the act an
/// operator comes here for.
pub(in crate::shell) const SCREEN: &str = crate::overlay::Overlay::SignIn.name();

impl Shell {
    pub(in crate::shell) fn signin(&mut self, ui: &mut egui::Ui, snap: &Snapshot) {
        self.note_screen(SCREEN);
        super::super::banner(ui, snap);
        ui.separator();
        egui::ScrollArea::vertical()
            .min_scrolled_height(0.0)
            .show(ui, |ui| self.rows(ui, snap));
    }

    /// The provider rows, each with its fact line and its tail under it when
    /// it is the one being followed.
    fn rows(&mut self, ui: &mut egui::Ui, snap: &Snapshot) {
        if snap.catalog.is_empty() {
            clip::weak(ui, "nothing read yet");
        }
        for row in &snap.catalog {
            self.provider(ui, row);
            if let Some(signing) = snap.login.as_ref().filter(|held| held.about(&row.name)) {
                if let Some(workspace) = &snap.focus.workspace {
                    clip::weak(ui, signin::fact(workspace));
                }
                self.tail(ui, &signing.view);
            }
            ui.add_space(space::XS);
        }
    }

    /// One row: what the server says about it, the tap that follows its run,
    /// and the act that starts one.
    fn provider(&mut self, ui: &mut egui::Ui, row: &ProviderRow) {
        let name = row.name.clone();
        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), TOUCH),
            egui::Layout::right_to_left(egui::Align::Center),
            |ui| {
                let start = theme::button_if(
                    ui,
                    row.blocked.is_none(),
                    "sign in",
                    0.0,
                    "starts this provider's sign-in on the server, and follows what it says",
                );
                // One control, two ops (lernie §4.24): the act, and the lane
                // the act's receipt makes wanted.
                crate::shell::act::acts(ui, &start, &[SCREEN, "login-tail"]);
                self.note_control("signin-start", ui, start.rect);
                if start.clicked()
                    && let Some(model) = self.model()
                {
                    model.sign_in(name.clone());
                }
                let stated = match &row.blocked {
                    Some(why) => format!("{} · {} · {why}", row.name, row.fact),
                    None => format!("{} · {}", row.name, row.fact),
                };
                // The words at the row's left edge (STYLE §5, bl-1312): a
                // growing atom after them takes the slack a full-width
                // button would otherwise share out either side.
                let follow = theme::button_led(
                    ui,
                    stated,
                    ui.available_width(),
                    "shows what this provider's sign-in has said so far",
                );
                crate::shell::act::act(ui, &follow, "login-tail");
                self.note_control("signin-row", ui, follow.rect);
                if follow.clicked()
                    && let Some(model) = self.model()
                {
                    model.watch_login(Some(name.clone()));
                }
            },
        );
    }

    /// **What one run has said.** The `err` flag is which stream a line came
    /// down and never a verdict on it — bz writes the whole human-facing
    /// flow, the authorize URL included, to stderr — so both are painted the
    /// same and neither is coloured. The outcome and the fallback are painted
    /// only once they exist: a run still going has no exit, and saying it
    /// exited zero would be the one misreading this screen must not make.
    fn tail(&mut self, ui: &mut egui::Ui, view: &LoginView) {
        if view.lines.is_empty() && view.outcome.is_none() {
            clip::weak(ui, "nothing said yet");
        }
        for line in &view.lines {
            self.tail_line(ui, &line.text);
        }
        if let Some(outcome) = view.outcome {
            let said = if outcome == 0 {
                "signed in".to_owned()
            } else {
                format!("the sign-in ended at {outcome}")
            };
            clip::said(ui, said);
        }
        if let Some(fallback) = &view.fallback {
            clip::weak(ui, format!("run this by hand: {fallback}"));
        }
    }

    /// **One line of the run.** Prose stays weak and copies on a long press
    /// as every prose surface does (§13.2); a line with a door in it is laid
    /// as a wrapped run of its parts, each address its own door, and the
    /// code's door after them at the line's end.
    fn tail_line(&mut self, ui: &mut egui::Ui, text: &str) {
        let read = signin::doors(text);
        if read.plain() {
            clip::weak(ui, text);
            return;
        }
        ui.horizontal_wrapped(|ui| {
            for part in &read.parts {
                match part {
                    Part::Prose(words) => {
                        let weak = egui::RichText::new(words).color(rgb(crate::theme::INK_WEAK));
                        ui.add(egui::Label::new(weak).wrap());
                    }
                    Part::Url(url) => self.opens(ui, url),
                }
            }
            if let Some(code) = &read.code {
                self.copies(ui, code);
            }
        });
    }

    /// **An address is a door** (§24 item 5): its words in the brand, `TOUCH`
    /// tall, never wider than the glass. A tap opens it in the phone's
    /// browser; a long press copies it, the one gesture §13.2 gives prose.
    fn opens(&mut self, ui: &mut egui::Ui, url: &str) {
        let wide = theme::wants(ui, url, None).min(ui.max_rect().width());
        let seat = theme::door(
            ui,
            (url, rgb(crate::theme::BRAND)),
            None,
            egui::vec2(wide, TOUCH),
            signin::OPENS,
        );
        self.note_control("signin-url", ui, seat.rect);
        if seat.clicked() {
            // A refusal is the platform's sentence and there is nowhere on
            // this glass for it (`clip::copied`'s reasoning): logcat is the
            // witness.
            log::info!("open: {}", crate::tools::paper::to_browser(url));
        }
        clip::copies(ui, &seat, url);
    }

    /// **A device code is a copy door**: `copy <code>` in the brand, and the
    /// platform's own clip preview is the confirmation.
    fn copies(&mut self, ui: &mut egui::Ui, code: &str) {
        let words = signin::copy_word(code);
        let wide = theme::wants(ui, &words, None).min(ui.max_rect().width());
        let seat = theme::door(
            ui,
            (&words, rgb(crate::theme::BRAND)),
            None,
            egui::vec2(wide, TOUCH),
            signin::COPIES,
        );
        self.note_control("signin-code", ui, seat.rect);
        if seat.clicked() {
            ui.ctx().copy_text(code.to_owned());
        }
    }
}
