//! **The sign-in that waits for a code, finished from the glass** (REMOTE
//! §8.3 at edition 31, DESIGN §13.19; bl-ee2d): under the tail of a run that
//! states an address and no exit, a door that opens the address in the
//! phone's browser, and a field for what that page hands back.
//!
//! **Which box is being signed in changes nothing here.** The browser is
//! this phone's whether the row is the server's or another machine's: the
//! address goes out as the lane's own `authorize` key, what the operator
//! copied goes back as `login`'s `code`, and the box that holds the row
//! checks it and keeps the credential. Neither crossing is one.
//!
//! **The server saying `authorize` IS the capability.** Only a server that
//! can take a `code` ever writes the key (§8.3's skew rule), so painting
//! this exactly when the fold states one never offers the field to a server
//! too old to read it — with no edition asked.
//!
//! **Leaving for the browser loses nothing.** Which run is followed is the
//! worker's, the address rides every replay of the lane, and what was
//! pasted stands in the shell until it is sent or the screen is left.
//!
//! Every word painted is `crate::signin`'s. The walk reaches the door at
//! `signin-page`, the field at `signin-paste` and the send at `signin-send`.

use eframe::egui;

use super::SCREEN;
use crate::shell::app::{PASTED, Shell};
use crate::shell::theme;
use crate::signin;

impl Shell {
    /// The door and the field, under the lines of a run waiting at `url`.
    pub(super) fn finish(&mut self, ui: &mut egui::Ui, provider: &str, url: &str) {
        if let Some(page) = signin::page(url) {
            self.opens(ui, signin::PAGE, &page, "signin-page", signin::PAGE_OPENS);
        }
        let field = ui.add(
            egui::TextEdit::singleline(&mut self.pasted)
                .id(egui::Id::new(PASTED.id))
                .desired_width(f32::INFINITY)
                .margin(crate::shell::composer::padding(ui))
                .hint_text(signin::PASTE),
        );
        self.note_control("signin-paste", ui, field.rect);
        let code = signin::pasted(&self.pasted);
        let send = theme::button_if(ui, code.is_some(), signin::SEND, 0.0, signin::SENDS);
        crate::shell::act::act(ui, &send, SCREEN);
        self.note_control("signin-send", ui, send.rect);
        if send.clicked()
            && let Some(code) = code
            && let Some(model) = self.model()
        {
            model.hand_code(provider.to_owned(), code);
            self.pasted.clear();
        }
    }
}
