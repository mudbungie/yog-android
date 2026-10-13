//! **The chat screen**: one conversation, painted from [`crate::rows`] in the
//! transcript's idiom (DESIGN §22 item 4; lernie `ui::chat`).
//!
//! The projection and its grouping are pure and tested elsewhere
//! (`crate::rows::lines`); this file is the paint and the one piece of state
//! a projection cannot hold — which rows the operator has flipped by hand.
//! That split is the whole design: what a line IS answers to the suite, and
//! what it looks like answers to the glass.
//!
//! **Every block starts at one left edge with its glyph in the gutter** — `›`
//! in the brand for the operator, `¦` for the model, `·` for a peer, a tool's
//! answer or the machinery — then who spoke in their weight, then the words
//! at the gutter's edge. No stripe, no bubble. A trail and its stems are
//! [`trail`]; the card at the head is [`panel`].
//!
//! **The fold overrides are ephemeral, as they are upstream.** The two auto
//! knobs are policy (`/full`, `/steps`); a hand-flip is a viewport fact and
//! dies with the screen.

use eframe::egui;

use super::theme::rgb;
use crate::outbox::Fate;
use crate::rows::{Before, Line, Part, Role, Row};
use crate::seat::Snapshot;
use crate::theme::{glyph, space};

/// The card at the head of a transcript.
pub(super) mod panel;
/// A model's prose with its markdown read (bl-7355).
mod prose;
/// A line that is its own touch target, with a glyph in the gutter.
mod tap;
/// A trail's head, its steps on their stems, and the `response` elbow.
mod trail;

/// **The outbox's echo** (bl-66fb): the message this seat has sent, painted
/// where the row it will become is going to be — behind the prompt's glyph,
/// muted while the engine has not answered, ordinary ink with a rule under it
/// once it has. The rule is the "not yet in the transcript" mark, and it goes
/// when the echo does.
///
/// **The third state is in doubt** (yog REMOTE §3, bl-07b1): the reply was
/// lost, so the engine may have taken this message and may not have. It keeps
/// the unanswered ink — because that is what it is — and carries the contract
/// under it in the operator's own words, naming the read that settles it. It
/// is a state and not a control: there is no resend here, because a resend is
/// a second message and the transcript below is already the recovery.
pub(crate) fn echo(ui: &mut egui::Ui, text: &str, fate: Fate) {
    let ink = if fate == Fate::Landed {
        role_hue(Role::User)
    } else {
        ui.visuals().weak_text_color()
    };
    ui.add_space(space::S);
    ui.horizontal_top(|ui| {
        tap::glyph(ui, glyph::PROMPT, rgb(crate::theme::BRAND));
        super::clip::copyable(
            ui,
            egui::Label::new(egui::RichText::new(text).color(ink)).wrap(),
            text,
        );
    });
    if fate == Fate::InDoubt {
        ui.horizontal_top(|ui| {
            ui.add_space(tap::GUTTER);
            super::clip::copyable(
                ui,
                egui::Label::new(
                    egui::RichText::new(IN_DOUBT)
                        .small()
                        .color(ui.visuals().warn_fg_color),
                )
                .wrap(),
                IN_DOUBT,
            );
        });
    }
    if fate == Fate::Landed {
        ui.separator();
    }
}

/// What an in-doubt echo says under itself. It is the same fact the banner
/// carried for one cadence, in the place the operator is looking and for as
/// long as the doubt lasts.
const IN_DOUBT: &str = "the reply was lost — this may or may not have been taken.     It was not sent again; the transcript will show it if it landed.";

/// How dim an abridged preview paints — the desktop's own solidity for a
/// payload that is not all there, so "there is more" is visible before the
/// fold glyph is read.
const ABRIDGED: f32 = 0.55;

/// **One line of the idiom** (§22 item 4): what parts it from the line above,
/// then the line itself — something said at the gutter, a trail's head, or a
/// step on its stem. Answers whether its fold was tapped; the caller owns the
/// override set, so this stays a paint over a `&Line`.
pub(crate) fn line(ui: &mut egui::Ui, line: &Line) -> bool {
    match line.before {
        Before::Nothing => {}
        Before::Gap => ui.add_space(space::S),
        Before::Parting => parting(ui),
        Before::Response => trail::response(ui),
    }
    match line.part {
        Part::Said => said(ui, &line.row),
        Part::Head => trail::head(ui, &line.row),
        Part::Step { last } => trail::step(ui, &line.row, last),
    }
}

/// **The hairline between turns** (STYLE §5), a fill the probe can count.
fn parting(ui: &mut egui::Ui) {
    ui.add_space(space::M);
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 1.0), egui::Sense::hover());
    ui.painter()
        .rect_filled(rect, 0.0, rgb(crate::theme::HAIRLINE));
    ui.add_space(space::S);
}

/// **One thing said**: the glyph in the gutter and who said it in their
/// weight on one line, the words under it at the gutter's edge. A row with a
/// fold says so at the line's end and the whole line is its toggle, the touch
/// floor tall; one with nothing to fold is a plain line.
fn said(ui: &mut egui::Ui, row: &Row) -> bool {
    let foldable = !row.body.is_empty();
    let who = match row.role {
        Some(role) => role_hue(role),
        None => tone_hue(ui, &row.tone),
    };
    let mut parts = vec![(row.prefix.clone(), who)];
    if foldable {
        let fold = if row.expanded {
            glyph::OPEN
        } else {
            glyph::SHUT
        };
        parts.push((format!("  {fold}"), rgb(crate::theme::INK_WEAK)));
    }
    let tall = if foldable { crate::theme::TOUCH } else { 0.0 };
    let runs = [tap::Run::body(parts)];
    let (mark, ink) = gutter(ui, row);
    let toggled = tap::line(ui, 0.0, (&mark, ink), &runs, tall).clicked() && foldable;
    ui.horizontal_top(|ui| {
        ui.add_space(tap::GUTTER);
        // Something said is prose: whole and wrapped unless it is folded,
        // when its first line stands dimmed for it.
        // A model's words are read for their marks (bl-7355); everything
        // else is painted as the bytes it is.
        let model = row.role == Some(Role::Model);
        if foldable && row.expanded && model {
            prose::paint(ui, &row.key, &row.body);
        } else if foldable && row.expanded {
            body(ui, &row.body);
        } else if foldable {
            preview(ui, &row.preview);
        } else if model && !row.preview.is_empty() {
            prose::paint(ui, &row.key, &row.preview);
        } else if !row.preview.is_empty() {
            body(ui, &row.preview);
        }
    });
    toggled
}

/// The glyph a row stands behind, in its ink: the prompt and a call in the
/// brand, a failure in the error accent, the rest in weak ink.
pub(super) fn gutter(ui: &egui::Ui, row: &Row) -> (String, egui::Color32) {
    let mark = crate::rows::glyph(row);
    let ink = match mark.as_str() {
        glyph::PROMPT | glyph::CALL => rgb(crate::theme::BRAND),
        glyph::FAILED => ui.visuals().error_fg_color,
        _ => rgb(crate::theme::INK_WEAK),
    };
    (mark, ink)
}

/// **An expanded payload, wrapped at the width it actually has** (bl-b62b,
/// bl-e86c): a label inside a horizontal layout does not wrap unless asked,
/// and one that overflows widens every label after it — so every label here
/// asks. Prose wraps and never scrolls sideways (§13.2).
pub(super) fn body(ui: &mut egui::Ui, text: &str) {
    super::clip::copyable(ui, egui::Label::new(text).wrap(), text);
}

/// The contracted payload, one line, dimmed so a run that ends in `…` reads
/// as incomplete before its fold glyph is noticed. **What is copied is the
/// whole line, not the ellipsis** (bl-7781).
fn preview(ui: &mut egui::Ui, text: &str) {
    let rich = egui::RichText::new(text).color(ui.visuals().text_color().gamma_multiply(ABRIDGED));
    super::clip::copyable(ui, egui::Label::new(rich).truncate(), text);
}

/// A speaker's rule (STYLE.md): told by weight, not hue, because on this
/// glass a hue means a state. The desktop seat ports the same table
/// (lernie bl-73d2), so a transcript read on the phone and on the laptop is
/// one vocabulary still.
fn role_hue(role: Role) -> egui::Color32 {
    super::theme::speaker(role)
}

/// A tone's ink. `Plain` and `Weak` defer to the installed visuals' own text
/// colours; the five states read the language's one table
/// (`crate::theme::tone`).
///
/// **Two surfaces spend it** (bl-ef9a): this transcript, and the conversation
/// list in `super::screens`. One map rather than two, because a hue that meant
/// one thing in a list and another in a transcript would be two colour
/// vocabularies inside one app — and because `Tone::Bad` in a roster row is
/// the operator's only passive sighting of a conversation refused at the
/// provider rung (yog bl-b43b), which is a fact worth painting the same way
/// wherever it appears.
///
/// A live row and an in-flight one share the one live accent; the desktop
/// pulses the second, which a phone's repaint budget does not buy back — the
/// word "running" in the label already says it.
pub(super) fn tone_hue(ui: &egui::Ui, tone: &crate::codec::Tone) -> egui::Color32 {
    use crate::codec::Tone;
    match tone {
        Tone::Plain => ui.visuals().text_color(),
        // **A hue word this build has not heard of reads as the weak ink**
        // (REMOTE §3.2), which is what `crate::theme::tone` already answers
        // for it — said here as `Weak`'s own case, because the two ARE one
        // reading and a second accent would claim a knowledge this build has
        // not got.
        Tone::Weak | Tone::Unknown(_) => ui.visuals().weak_text_color(),
        Tone::Good | Tone::Bad | Tone::Live | Tone::InFlight | Tone::Held => {
            super::theme::tone(tone)
        }
    }
}

/// The conversation's display name — **who** the model turns are, since a
/// speaker is an agent and not a model id. The roster row already carries it;
/// falling back to the addressed id is what a transcript opened before its
/// list arrived shows.
pub(crate) fn speaker_of(snap: &Snapshot, agent: &str) -> String {
    snap.conversations
        .iter()
        .find(|row| row.root_id == agent)
        .map_or_else(|| agent.to_owned(), |row| row.display.clone())
}
