//! The two-way IME mirror (bl-014e; DESIGN §3).
//!
//! winit 0.30's android loop has no `TextEvent` arm, so everything the
//! `InputConnection` commits dies before egui — and pushing synthetic events
//! into `Context::input_mut` mid-frame lands in input egui has already
//! consumed (measured dead end). So the shell owns the field strings and
//! talks to `GameTextInput`'s editor buffer directly, in BOTH directions:
//!
//! * **IME → app**: the buffer changed, so adopt it wholesale into the
//!   focused field. Wholesale is what makes a suggestion tap work (a 2-char
//!   word replaced by a 6-char one in one step), and it is only correct
//!   because of the other direction.
//! * **app → IME**: nothing changed on the IME's side, so OUR text is the
//!   truth — push it when the two have drifted (focus moved, Enter cleared
//!   the field). This direction is what backspace long-press needs: Gboard's
//!   delete-repeat is a loop against the editor state it is shown, and an
//!   editor that never reports its state is one Gboard stops deleting from.
//!
//! A push is ASYNCHRONOUS: `set_text_input_state` posts to the Java UI
//! thread, so for a few milliseconds the editor still reports its OLD text,
//! and adopting in that window feeds the stale buffer straight back into the
//! field just pushed (measured: a focus change blanked the field for a
//! frame). A push is therefore held open as a pending echo — nothing is
//! adopted until the editor reports it, polled at 8ms only inside that
//! window because a push we made ourselves generates no wake.

use eframe::egui;
use winit::platform::android::activity as aa;

/// One editable field the mirror serves, by egui id.
#[derive(Clone, Copy)]
pub(crate) struct Field {
    pub(crate) id: &'static str,
    pub(crate) kind: FieldKind,
}

/// What kind of editor the IME is told a field is (`set_ime_editor_info`).
/// `GameActivity`'s default `EditorInfo` is `inputType = 0` — `TYPE_NULL`, which
/// alone puts the IME in degraded key-event mode: no delete-repeat loop, no
/// autocorrect, no suggestion strip (bl-014e). Variants grow per field,
/// never speculatively.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum FieldKind {
    /// The chat composer: multi-line short-message text with **no action**
    /// (bl-6850). Multi-line is what makes the enter key a newline —
    /// Android shows a return key instead of an action key for a multi-line
    /// editor, and an IME that is not told a field is multi-line commits no
    /// newline into the buffer the mirror adopts, which is why enter was
    /// inert rather than merely un-sending. No action, because the action
    /// this field used to declare could not fire: `GameActivity` writes it
    /// where the enter key does not read it (DESIGN §3's residual), and a
    /// key promised to do something that cannot happen is worse than a key
    /// that plainly breaks the line. The send button is THE send (§13.2).
    Composer,
    /// **The search field** (§13.6): one line, and neither autocorrected nor
    /// capitalized. A needle is as often an id or a fragment as it is a word
    /// — the engine folds case itself and never spelling — so a correction
    /// here would search for something the operator did not type, and the
    /// only way to find out would be reading the answer's own echoed needle.
    /// No action key, for the composer's reason: `GameActivity` writes one
    /// where the enter key does not read it (DESIGN §3), and the button
    /// beside the field is the gesture.
    Needle,
    /// The enrollment screen's envelope: a long machine-written blob carried
    /// here by paste. Autocorrect and sentence capitals would corrupt it, and
    /// there is no action key — the button beside it is the gesture.
    Envelope,
}

#[derive(Default)]
pub(crate) struct Bridge {
    /// What we believe the IME's editor buffer holds.
    ime_last: String,
    /// Whether the IME was ever told an `EditorInfo`, and for which slot.
    /// A flag beside the slot rather than a nested Option, so "never told" —
    /// the state that ships `TYPE_NULL` (bl-014e) — stays a named fact.
    told: bool,
    told_slot: Option<usize>,
    /// Text handed to `set_text_input_state` that the editor has not echoed
    /// back yet, and the deadline for giving up on the echo.
    pending: Option<(String, u128)>,
}

impl Bridge {
    /// One frame of the mirror. Runs FIRST in the frame, on egui's settled
    /// focus (read here, never remembered from the previous frame's widget
    /// responses), so the widgets lay out from text that is already current.
    pub(crate) fn run(
        &mut self,
        ctx: &egui::Context,
        app: &aa::AndroidApp,
        fields: &mut [(Field, &mut String)],
        now: u128,
    ) {
        let state = app.text_input_state();
        let focused = ctx.memory(egui::Memory::focused);
        let slot = fields
            .iter()
            .position(|(f, _)| focused == Some(egui::Id::new(f.id)));

        if let Some((sent, deadline)) = self.pending.clone() {
            if state.text == sent {
                self.pending = None;
                self.ime_last = sent;
            } else if now >= deadline {
                // The echo never came. Take the editor's word for the buffer
                // and let the drift check below push again.
                self.pending = None;
                self.ime_last.clone_from(&state.text);
            } else {
                ctx.request_repaint_after(std::time::Duration::from_millis(8));
            }
        } else if state.text != self.ime_last {
            if let Some((field, text)) = slot.and_then(|i| fields.get_mut(i)) {
                let caret = super::span::char_index(&state.text, state.selection.end);
                text.clone_from(&state.text);
                set_caret(ctx, field.id, caret);
            }
            self.ime_last.clone_from(&state.text);
        }

        // Tell the IME what KIND of editor this is, whenever focus moves to
        // a different one. The IME only re-reads `EditorInfo` on
        // `restartInput`, and `setState` triggers one — so a change here
        // forces the push below even when the text agrees.
        let info_changed = !self.told || self.told_slot != slot;
        if info_changed {
            self.told = true;
            self.told_slot = slot;
            if let Some((field, _)) = slot.and_then(|i| fields.get(i)) {
                let (ty, action) = editor_info(field.kind);
                app.set_ime_editor_info(ty, action, aa::input::ImeOptions::IME_FLAG_NO_FULLSCREEN);
            }
        }

        // app → IME. Only when the two have actually drifted: a push per
        // frame would restart Gboard's composing session every frame and
        // take autocorrect and glide down with it.
        let want = slot
            .and_then(|i| fields.get(i))
            .map_or_else(String::new, |(_, text)| (**text).clone());
        if info_changed || want != self.ime_last {
            let caret = slot
                .and_then(|i| fields.get(i))
                .map_or(0, |(field, _)| caret_of(ctx, field.id, &want));
            let unit = super::span::utf16_index(&want, caret);
            app.set_text_input_state(aa::input::TextInputState {
                text: want.clone(),
                selection: aa::input::TextSpan {
                    start: unit,
                    end: unit,
                },
                compose_region: None,
            });
            self.ime_last.clone_from(&want);
            self.pending = Some((want, now + 250));
            ctx.request_repaint_after(std::time::Duration::from_millis(8));
        }
    }
}

/// What kind of editor a field is, in the IME's own vocabulary.
fn editor_info(kind: FieldKind) -> (aa::input::InputType, aa::input::TextInputAction) {
    use aa::input::{InputType as T, TextInputAction as A};
    let base =
        T::TYPE_CLASS_TEXT | T::TYPE_TEXT_FLAG_CAP_SENTENCES | T::TYPE_TEXT_FLAG_AUTO_CORRECT;
    match kind {
        FieldKind::Composer => (
            base | T::TYPE_TEXT_VARIATION_SHORT_MESSAGE | T::TYPE_TEXT_FLAG_MULTI_LINE,
            A::None,
        ),
        FieldKind::Needle => (
            T::TYPE_CLASS_TEXT | T::TYPE_TEXT_FLAG_NO_SUGGESTIONS,
            A::None,
        ),
        FieldKind::Envelope => (
            T::TYPE_CLASS_TEXT | T::TYPE_TEXT_FLAG_MULTI_LINE | T::TYPE_TEXT_FLAG_NO_SUGGESTIONS,
            A::None,
        ),
    }
}

/// Store egui's caret for `id` at char offset `caret`. An edit egui did not
/// itself process moves no cursor — `TextEdit`'s caret lives in egui memory
/// per widget id — so without this the caret sits at 0 while text grows at
/// the end (bl-014e).
fn set_caret(ctx: &egui::Context, id: &str, caret: usize) {
    let id = egui::Id::new(id);
    let mut state = egui::text_edit::TextEditState::load(ctx, id).unwrap_or_default();
    state
        .cursor
        .set_char_range(Some(egui::text::CCursorRange::one(
            egui::text::CCursor::new(caret),
        )));
    state.store(ctx, id);
}

/// egui's caret for `id` as a char offset, defaulting to the end of `text`.
fn caret_of(ctx: &egui::Context, id: &str, text: &str) -> usize {
    egui::text_edit::TextEditState::load(ctx, egui::Id::new(id))
        .and_then(|state| state.cursor.char_range())
        .map_or_else(
            || text.chars().count(),
            |r| usize::from(r.primary.index).min(text.chars().count()),
        )
}
