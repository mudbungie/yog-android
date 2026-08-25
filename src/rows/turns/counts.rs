//! **What a rolled-up turn holds** — the census behind the aggregate line and
//! the sentence it says. Counted from the committed bytes and nothing else: a
//! badge claiming more than its data knows is a filed bug class, so token sums
//! come only from the entries' committed `usage` records, never estimated, and
//! a mixed turn is worded for what it does not know. Split from [`super`] on
//! the seam between **deciding where a turn is** and **saying what it
//! contained**.

use super::{Row, Step, TERM_SEP, Usage, turn_key};

/// What a turn's folded machinery holds.
pub(super) struct Counts {
    /// Distinct model entries the run's rows came from.
    pub(super) inference: usize,
    tools: usize,
    thinking: usize,
    /// Per-counter token sums over the counted entries' committed records,
    /// under the counters' own committed names.
    tokens: Usage,
    /// How many of the counted entries carried a readable record at all.
    reported: usize,
}

impl Counts {
    /// Count one machinery run. Rows of one entry are contiguous (the
    /// projection walks entries in order), so a new inference call is exactly
    /// a model-authored row whose entry differs from the previous one's — and
    /// that is also the once-per-entry moment its usage record folds in.
    pub(super) fn of(rows: &[Row], steps: &[Step], usage: &[Usage]) -> Self {
        let mut counts = Self {
            inference: 0,
            tools: 0,
            thinking: 0,
            tokens: Usage::new(),
            reported: 0,
        };
        let mut entry = String::new();
        for ((row, step), report) in rows.iter().zip(steps).zip(usage) {
            match step {
                Step::ToolCall => counts.tools += 1,
                Step::Thinking => counts.thinking += 1,
                Step::Model | Step::Boundary | Step::Plain => {}
            }
            if matches!(step, Step::Model | Step::Thinking | Step::ToolCall) {
                let seen = turn_key(&row.key);
                if seen != entry {
                    counts.inference += 1;
                    entry = seen;
                    counts.fold(report);
                }
            }
        }
        counts
    }

    /// Fold one counted entry's committed counters into the running sums.
    fn fold(&mut self, report: &Usage) {
        if report.is_empty() {
            return;
        }
        self.reported += 1;
        for (counter, n) in report {
            *self.tokens.entry(counter.clone()).or_default() += n;
        }
    }

    /// The aggregate line in the operator's own phrasing — `9 inference calls
    /// · 14 tool calls · 3 thinking blocks · 3150 output tokens` — with a zero
    /// term left unsaid. The inference term always survives, so the line is
    /// never empty. Token terms state each committed counter's sum under its
    /// own name; **a mixed turn** (some counted entries record-bearing, some
    /// not) suffixes each sum with `+` — *at least this many*, because the
    /// bytes carry no more than the reporting entries said.
    pub(super) fn say(&self) -> String {
        let terms = [
            (self.inference, "inference call"),
            (self.tools, "tool call"),
            (self.thinking, "thinking block"),
        ];
        let mut said: Vec<String> = terms
            .iter()
            .filter(|(n, _)| *n > 0)
            .map(|(n, word)| format!("{n} {word}{}", if *n == 1 { "" } else { "s" }))
            .collect();
        let at_least = if self.reported < self.inference {
            "+"
        } else {
            ""
        };
        for (counter, sum) in &self.tokens {
            if *sum > 0 {
                said.push(format!("{sum}{at_least} {}", counter.replace('_', " ")));
            }
        }
        said.join(TERM_SEP)
    }
}
