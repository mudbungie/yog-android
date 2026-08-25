//! Turn rollup: what the agent *did* between two things it *said*, folded to
//! one aggregate line.
//!
//! The ruling this mirrors: the moment each step is done it collapses down,
//! until all that is left is a single line — "3150 thinking tokens, 9
//! inference calls, 14 tool calls", or whatever the turn actually was —
//! expandable to see each step in flight. When it is done and the agent is
//! responding, just one line before the response.
//!
//! A **turn** is derived, never stored. Delivered messages delimit the row
//! sequence — a message *to* the agent is the other half of the exchange, so
//! it is a boundary and never a step inside a turn — and within a segment the
//! turn's **answer** is its last row, when the model ended by talking.
//! Everything before that answer is the turn's machinery: thinking, tool
//! calls, tool results, and the model's own intermediate remarks. It rolls up
//! into ONE aggregate row whose fold opens onto those very step rows, each
//! still folding on its own.
//!
//! Three conditions gate the rollup, all read off the rows themselves:
//!
//! - the turn **ended by talking** — an unfinished turn keeps its steps on
//!   screen, because that is the work in progress the operator came to watch;
//! - **nothing in it is in flight** ([`super::in_flight`]) — a live tail or an
//!   unretired tool call makes the whole turn the show;
//! - it holds **at least one inference call** — a run of stray entries with no
//!   model output is not a turn, which is also why the aggregate line can
//!   never come out empty.

use std::collections::BTreeSet;

use super::{AutoExpand, Fold, Row, RowClass, Tone, expanded_for, in_flight};

mod counts;
mod steps;

use counts::Counts;
use steps::{Step, Usage};
pub(super) use steps::{step_of, usage_of};

/// Key suffix of a turn's aggregate row, where a block ordinal would sit. An
/// ordinal is always a number, so the two can never collide, and the key is
/// the turn's first entry — stable across the stateless re-read.
const TURN_SUFFIX: &str = "turn";
/// The machinery glyph, as the tool-call rows already wear it.
const TURN_GLYPH: &str = "⚙";
/// Separator between the aggregate's terms.
const TERM_SEP: &str = " · ";
/// What the aggregate's fold opens onto, said in words.
const TURN_HOVER: &str = "what the agent did before answering — open it for each step";

/// Roll every finished turn's machinery up into its aggregate row. `steps` and
/// `usage` run parallel to `flat`; `auto`/`folds` decide whether an aggregate
/// is open, because a shut one leaves its step rows out of the projection
/// entirely rather than merely marking them hidden.
pub(super) fn group(
    flat: &[Row],
    steps: &[Step],
    usage: &[Usage],
    auto: AutoExpand,
    folds: &BTreeSet<String>,
) -> Vec<Row> {
    let mut out = Vec::new();
    let mut start = 0;
    for (i, step) in steps.iter().enumerate() {
        if *step != Step::Boundary {
            continue;
        }
        push_turn(
            span(flat, start, i),
            span(steps, start, i),
            span(usage, start, i),
            auto,
            folds,
            &mut out,
        );
        if let Some(boundary) = flat.get(i) {
            out.push(boundary.clone());
        }
        start = i + 1;
    }
    push_turn(
        span(flat, start, flat.len()),
        span(steps, start, steps.len()),
        span(usage, start, usage.len()),
        auto,
        folds,
        &mut out,
    );
    out
}

/// `slice[from..to]`, checked — an out-of-range span is empty rather than a
/// panic path.
fn span<T>(slice: &[T], from: usize, to: usize) -> &[T] {
    slice.get(from..to).unwrap_or_default()
}

/// One boundary-delimited segment: the machinery run, then the answer that
/// ended it. Rolls up when the three conditions hold, else passes through.
fn push_turn(
    rows: &[Row],
    steps: &[Step],
    usage: &[Usage],
    auto: AutoExpand,
    folds: &BTreeSet<String>,
    out: &mut Vec<Row>,
) {
    let (Some((answer, run)), Some((_, run_steps))) = (rows.split_last(), steps.split_last())
    else {
        out.extend_from_slice(rows);
        return;
    };
    let run_usage = span(usage, 0, run.len());
    let counts = Counts::of(run, run_steps, run_usage);
    let rolls_up =
        answer.class == RowClass::Response && counts.inference > 0 && !rows.iter().any(in_flight);
    match run.first() {
        Some(first) if rolls_up => {
            let parent = aggregate(&first.key, &counts, auto, folds);
            let open = parent.expanded;
            out.push(parent);
            if open {
                out.extend_from_slice(run);
            }
            out.push(answer.clone());
        }
        _ => out.extend_from_slice(rows),
    }
}

/// The turn's aggregate row: machinery, so it answers the same auto-knob every
/// other machinery row does — one line by default, and the operator who set
/// that knob open gets every turn opened, steps and all. No separate knob,
/// because a third setting for the same question is a setting that can
/// disagree with itself.
fn aggregate(first_key: &str, counts: &Counts, auto: AutoExpand, folds: &BTreeSet<String>) -> Row {
    let mut row = Row {
        key: turn_key(first_key),
        prefix: format!("{TURN_GLYPH} {}", counts.say()),
        preview: String::new(),
        body: String::new(),
        hover: TURN_HOVER.to_string(),
        class: RowClass::Other,
        tone: Tone::Weak,
        // Machinery rolled up is still machinery: no one is speaking, so the
        // aggregate wears the empty role seat.
        role: None,
        fold: Fold::Steps,
        expanded: false,
    };
    row.expanded = expanded_for(&row, auto, folds.contains(&row.key));
    row
}

/// The aggregate's identity: the turn's first row's entry, with the block
/// ordinal replaced by [`TURN_SUFFIX`]. Doubles as the entry identity the
/// census counts distinct inference calls by — one spelling, not two.
fn turn_key(first_key: &str) -> String {
    let entry = first_key
        .rsplit_once('#')
        .map_or(first_key, |(head, _)| head);
    format!("{entry}#{TURN_SUFFIX}")
}
