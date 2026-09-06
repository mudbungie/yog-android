//! **What each half of the records screen SAYS** — split from the screen
//! itself (bl-146b) on the seam every paint file in this app is cut along:
//! what the surface DOES with a tap, against what it puts on the glass.
//!
//! Every figure here is the engine's own rendering and none is recomputed:
//! the flight strip's characteristics, the money, the context percent, a
//! notch's clipped commit and a step's framing and wound are all words
//! upstream minted, painted as they came (§13.9's rule, at a second site).

use eframe::egui;

use crate::codec::{Orphan, Records, Step, StepRow};

/// **The header: what the conversation is and what it is doing.** Every figure
/// here is the engine's own rendering — the strip's characteristics, the
/// money, the context percent — and nothing is recomputed: a seat that divided
/// the two context figures itself would be re-taking a decision upstream took
/// on purpose (§13.9's rule, at a second site).
pub(super) fn head(ui: &mut egui::Ui, records: &Records) {
    let head = &records.head;
    ui.label(&head.display);
    ui.weak(format!("{} · under {}", state(records), head.root));
    if let Some(failure) = &head.failure {
        ui.colored_label(egui::Color32::LIGHT_RED, failure);
    }
    if let Some(strip) = &head.strip {
        ui.weak(strip);
    }
    if let Some(context) = &head.context {
        ui.weak(format!(
            "{} · {}% of the window",
            context.model, context.percent
        ));
    }
    if !head.usd.is_empty() {
        ui.weak(&head.usd);
    }
    if !head.marks.is_empty() {
        ui.weak(head.marks.join(" · "));
    }
    if !head.tip.is_empty() {
        ui.weak(format!("tip {}", head.tip));
    }
    for seat in &head.seats {
        ui.weak(format!("{} · {}", seat.name, seat.doing));
    }
}

/// The three readings the engine takes of a conversation's standing, said in
/// one line: what state it is in, what is in flight, and whether the engine
/// can see it at all.
fn state(records: &Records) -> String {
    let head = &records.head;
    let mut said = vec![head.state.clone()];
    if !head.flight.is_empty() {
        said.push(head.flight.clone());
    }
    if !head.present {
        said.push("not present".to_owned());
    }
    if head.refused {
        said.push("refused".to_owned());
    }
    said.join(" · ")
}

/// **What governs the conversation now**, and the children already hanging
/// off its spine. The notches themselves are not here: they are fork points,
/// and a fork point is a control (§13.16) — so they paint in the screen file
/// beside the lineages, which are the other kind of one.
pub(super) fn spine(ui: &mut egui::Ui, records: &Records) {
    ui.weak(governed(&records.governing));
    if !records.governing.files.is_empty() {
        ui.weak(records.governing.files.join(" · "));
    }
    for card in &records.rail.cards {
        ui.weak(format!(
            "{} · {} · {} · at notch {} · {} tokens",
            card.name, card.state, card.fork, card.notch, card.tokens
        ));
        if !card.tail.is_empty() {
            ui.weak(&card.tail);
        }
    }
}

/// **One reading of a governing answer**, spent twice since the picking
/// surface landed (§13.16): once for the conversation's own policy, and once
/// for the policy at a picked fork point. Two spellings of one answer would
/// have drifted the first time either moved.
pub(super) fn governed(governing: &crate::codec::Governing) -> String {
    let follows = match &governing.follows {
        Some(name) => format!("follows {name}"),
        None => format!("{} lineages diverged", governing.diverged),
    };
    format!("governed by {} · {follows}", governing.short_oid)
}

/// One census row, as its control's label.
pub(super) fn step_line(row: &StepRow, picked: bool) -> String {
    let mark = if picked { "▸ " } else { "" };
    let wound = match &row.wound_reason {
        Some(why) => format!("{} — {why}", row.wound),
        None => row.wound.clone(),
    };
    format!(
        "{mark}{} · {} · {wound}\n{} tokens · {} attempt(s){}",
        row.seq,
        row.framing,
        row.tokens,
        row.attempts,
        if row.commit.is_empty() {
            String::new()
        } else {
            format!(" · {}", row.commit)
        }
    )
}

/// **One step's records, under the row they belong to.** The answer states
/// its own `seq`, so this is painted where that sequence is and nowhere else
/// — a drill-in that landed after the operator tapped another row simply has
/// no row to paint under.
pub(super) fn drilled(ui: &mut egui::Ui, step: &Step) {
    for (name, record) in [
        ("meta", &step.meta),
        ("request", &step.request),
        ("staging", &step.staging),
    ] {
        ui.weak(format!("{name} · {}", record.kind));
        said(ui, &record.note);
        said(ui, &record.raw);
    }
    for event in &step.response {
        ui.weak(format!("response · {}", event.kind));
        said(ui, &event.raw);
    }
    for tool in &step.tools {
        let mark = if tool.is_error { " · error" } else { "" };
        ui.weak(format!("{}{mark}", tool.tool_id));
        said(ui, &tool.input.raw);
        said(ui, &tool.output.raw);
    }
    for log in [&step.stderr, &step.driver].into_iter().flatten() {
        ui.weak(format!("{} · {}", log.kind, log.text));
    }
}

/// **The undelivered mail.** An empty inbox says so: the engine answered and
/// there is nothing, which is not the same sentence as nobody having asked.
pub(super) fn mail(ui: &mut egui::Ui, records: &Records) {
    if records.inbox.is_empty() {
        ui.weak("no mail waiting");
    }
    for row in &records.inbox {
        let from = row.from.clone().unwrap_or_else(|| row.name.clone());
        let when = row.deposited_at.clone().unwrap_or_default();
        ui.label(format!("{from} · {when}"));
        said(ui, &row.body);
        if let Some(epitaph) = &row.epitaph {
            ui.weak(epitaph);
        }
    }
}

/// One line of the engine's own words, painted only where there are any: an
/// absent field is a fact and paints as nothing (`codec::balls`' rule).
fn said(ui: &mut egui::Ui, text: &str) {
    if !text.is_empty() {
        ui.weak(text);
    }
}

/// **The orphaned tail, said above the census it is about.** It is a
/// view-level fact rather than any one step's — upstream puts it at the top of
/// the answer for exactly that reason — and the engine's own words are painted
/// beside it when the class left any. Nothing is said at all for the ordinary
/// conversation, which is what `none` means.
pub(super) fn orphan(ui: &mut egui::Ui, records: &Records) {
    let said = match records.steps.orphan {
        Orphan::None => return,
        Orphan::Mail => "a deposit was left undelivered",
        Orphan::ToolWindow => "a tool call was left unpaired",
    };
    match &records.steps.orphan_reason {
        Some(why) => ui.weak(format!("{said} — {why}")),
        None => ui.weak(said),
    };
}
