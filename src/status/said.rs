//! **What the rule's runs say** (DESIGN §23 item 14): the conversation's
//! diagnostics — its state and age, its tokens, its spend, its context — and
//! its name, each in its ink. Split from `status.rs` at the 300-line cap on
//! the seam that file already had: here is what a run SAYS, there is how the
//! runs FIT the line.

use crate::codec::{Agent, ConvRow, Entry, EntryKind};
use crate::theme::{self, Rgb};

use super::Slot;

/// **One diagnostic**: its slot, its words, its ink.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Said {
    pub slot: Slot,
    pub text: String,
    pub ink: Rgb,
}

/// **The conversation's diagnostics and name**, in rule order: the state and
/// how long since it acted, the tokens, then off the conversation's own
/// `agent` row (`head`) the spend in the ledger's words and the context's
/// fullness as the server's own percent — each absent where the row is not
/// held or states none, never `$0` or `0%` — and the name. Nothing focused is
/// nothing said.
pub fn diagnostics(
    row: Option<&ConvRow>,
    transcript: &[Entry],
    head: Option<&Agent>,
    now_unix: i64,
) -> Vec<Said> {
    let Some(row) = row else {
        return Vec::new();
    };
    let mut out = vec![Said {
        slot: Slot::State,
        text: format!(
            "{} · {}",
            row.state.word(),
            crate::roster::stamp(row.last_active_unix, now_unix)
        ),
        ink: theme::tone(&row.tone),
    }];
    if let Some((input, output)) = tokens(transcript) {
        out.push(Said {
            slot: Slot::Tokens,
            text: format!("{} in · {} out", count(input), count(output)),
            ink: theme::INK_WEAK,
        });
    }
    if let Some(usd) = head.map(|h| h.usd.as_str()).filter(|usd| !usd.is_empty()) {
        out.push(Said {
            slot: Slot::Spend,
            text: usd.to_owned(),
            ink: theme::INK_WEAK,
        });
    }
    if let Some(percent) = head.and_then(|h| h.context.as_ref()).map(|c| c.percent) {
        out.push(Said {
            slot: Slot::Context,
            text: format!("context {percent}%"),
            ink: theme::INK_WEAK,
        });
    }
    out.push(Said {
        slot: Slot::Name,
        text: row.display.clone(),
        ink: theme::INK,
    });
    out
}

/// **The tokens the transcript's model turns committed**, in and out — the
/// provider's own counters, summed where they are spelled. None where no
/// turn carried either.
pub fn tokens(transcript: &[Entry]) -> Option<(u64, u64)> {
    let mut seen = false;
    let mut sum = (0, 0);
    for entry in transcript {
        if let EntryKind::Model { usage, .. } = &entry.kind {
            let read = |key: &str| usage.get(key).and_then(serde_json::Value::as_u64);
            let (input, output) = (read("input_tokens"), read("output_tokens"));
            seen |= input.is_some() || output.is_some();
            sum.0 += input.unwrap_or(0);
            sum.1 += output.unwrap_or(0);
        }
    }
    seen.then_some(sum)
}

/// A count as few characters say it: `812`, `12.3k`, `4.1M`.
pub fn count(n: u64) -> String {
    match n {
        0..1_000 => n.to_string(),
        1_000..1_000_000 => format!("{:.1}k", n as f64 / 1e3),
        _ => format!("{:.1}M", n as f64 / 1e6),
    }
}
