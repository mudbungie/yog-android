//! **What the selected provider will take, and what it says when it will
//! not** (REMOTE §9.13, bl-dfbb; the sentence bl-0691).
//!
//! The two capability booleans ride the provider row the engine listed
//! (`reply/providers` since PROTOCOL 6) and this seat re-derives neither: a
//! control offered for a provider that cannot take the setting is a control
//! that earns a refusal, and DESIGN §8's rule is that a client re-deriving
//! world state is inventing it. **The model name is never consulted** —
//! `gpt-5.6` on one row and on another are the same three tokens and
//! different capabilities, and the row is what knows.
//!
//! **A dark control carries its reason** (bl-0691). Greying alone said
//! nothing: an operator looking at a dark `effort` on a provider that takes
//! reasoning effort perfectly well cannot tell *this provider has no such
//! lane* from *this app has not asked yet* from *this app is broken*. There
//! are exactly three reasons a knob is dark and each is a different
//! instruction — set a provider, run an engine that lists this one, or
//! nothing at all because this provider does not have the lane — so the
//! answer is a sentence and not a boolean, and the sentence is made here,
//! under the coverage floor, rather than in a paint file.

use super::ProviderRow;

/// The wire's own words for the two knobs, spelled here because the sentences
/// below name them (REMOTE §9.13: the vocabulary is the operator's, and a
/// provider dialect's spelling never reaches a surface).
const EFFORT: &str = "effort";
const PRIORITY: &str = "priority";

/// **One tuning control's standing**: whether the provider row states it
/// takes the setting, and — when it does not — why, in a sentence an
/// operator can read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Knob {
    /// Whether the selected provider's own row states it takes this tuning.
    pub taken: bool,
    /// Why it is dark, when it is. `None` exactly when [`taken`](Self::taken)
    /// — a live control has nothing to explain.
    pub why: Option<String>,
}

impl Knob {
    /// The row said yes.
    fn takes() -> Self {
        Self {
            taken: true,
            why: None,
        }
    }

    /// The row said no, in the one sentence that says whose *no* it is: the
    /// provider's, about this knob, and not this app's.
    fn declines(provider: &str, knob: &str) -> Self {
        Self {
            taken: false,
            why: Some(format!("{provider} does not take {knob}")),
        }
    }

    /// Nothing is set to ask. The knob is a fact about a provider row, so
    /// with no provider there is no row and no answer — and the operator's
    /// move is the selector beside it.
    fn unaimed(knob: &str) -> Self {
        Self {
            taken: false,
            why: Some(format!(
                "no provider is set: {knob} is what a provider row takes"
            )),
        }
    }

    /// The provider is set and the engine listed no row for it — an engine
    /// too old for the read, a listing that has not answered yet, or a
    /// provider the wall no longer serves. **Unanswered, and said as
    /// unanswered**: this seat may not grant a control on a question that
    /// went unanswered (REMOTE §9.13), and it may not pretend the provider
    /// refused either.
    fn unlisted(provider: &str, knob: &str) -> Self {
        Self {
            taken: false,
            why: Some(format!(
                "the engine has not listed {provider}, so whether it takes {knob} is unanswered"
            )),
        }
    }
}

/// **What the selected provider will take**: the two knobs, read off the row
/// the engine listed, for the provider these controls are pointed at.
pub fn tunable(rows: &[ProviderRow], provider: Option<&str>) -> (Knob, Knob) {
    let Some(provider) = provider else {
        return (Knob::unaimed(EFFORT), Knob::unaimed(PRIORITY));
    };
    let Some(row) = rows.iter().find(|row| row.name == provider) else {
        return (
            Knob::unlisted(provider, EFFORT),
            Knob::unlisted(provider, PRIORITY),
        );
    };
    (
        knob(row.effort, provider, EFFORT),
        knob(row.priority, provider, PRIORITY),
    )
}

/// One column of the row, read as a control's standing.
fn knob(takes: bool, provider: &str, name: &str) -> Knob {
    if takes {
        Knob::takes()
    } else {
        Knob::declines(provider, name)
    }
}

#[cfg(test)]
mod tests {
    use super::{Knob, tunable};
    use crate::codec::pick::row;
    use serde_json::json;

    fn rows() -> Vec<crate::codec::ProviderRow> {
        [
            json!({ "name": "acme", "fact": "credential present", "blocked": null,
                    "effort": true, "priority": false }),
            json!({ "name": "rival", "fact": "no credential", "blocked": null,
                    "effort": false, "priority": true }),
        ]
        .iter()
        .map(|value| row(value).expect("a listed row"))
        .collect()
    }

    /// **The gate is the selected provider's own row** (bl-dfbb), and a live
    /// knob explains nothing because it has nothing to explain.
    #[test]
    fn the_gate_is_the_selected_providers_own_row() {
        let rows = rows();
        let (effort, priority) = tunable(&rows, Some("acme"));
        assert_eq!(effort, Knob::takes());
        assert!(!priority.taken);
        let (effort, priority) = tunable(&rows, Some("rival"));
        assert!(!effort.taken);
        assert_eq!(priority, Knob::takes());
    }

    /// **A row that declines names itself, the knob, and nothing else** — the
    /// operator is told whose refusal it is.
    #[test]
    fn a_row_that_declines_says_so_in_its_own_name() {
        let (effort, _) = tunable(&rows(), Some("rival"));
        assert_eq!(
            effort.why.as_deref(),
            Some("rival does not take effort"),
            "the sentence names the provider and the knob"
        );
    }

    /// **No provider is a different sentence from a provider that declines**
    /// — the move is the selector beside it, not a lane that does not exist.
    #[test]
    fn nothing_selected_says_there_is_no_row_to_ask() {
        let (effort, priority) = tunable(&rows(), None);
        assert!(!effort.taken && !priority.taken);
        assert_eq!(
            effort.why.as_deref(),
            Some("no provider is set: effort is what a provider row takes")
        );
        assert_eq!(
            priority.why.as_deref(),
            Some("no provider is set: priority is what a provider row takes")
        );
    }

    /// **An unlisted provider is UNANSWERED, never a refusal** (REMOTE
    /// §9.13): an engine that predates the read, or a listing not yet in,
    /// must not be reported as the provider saying no.
    #[test]
    fn a_provider_the_engine_did_not_list_is_unanswered() {
        for rows in [rows(), Vec::new()] {
            let (effort, priority) = tunable(&rows, Some("nobody"));
            assert!(!effort.taken && !priority.taken);
            for (why, knob) in [(effort.why, "effort"), (priority.why, "priority")] {
                let why = why.expect("a dark knob carries its reason");
                assert!(why.contains("has not listed nobody"), "{why}");
                assert!(why.contains(knob), "{why}");
                assert!(!why.contains("does not take"), "{why}");
            }
        }
    }
}

#[cfg(test)]
mod wire {
    /// **The engine's own answer, byte for byte** (bl-0691, measured against a
    /// live yog 0.0.55 on a fixture world): the roles read for a workspace
    /// whose worker names `gpt-5.6` on `openai-chatgpt`, and the provider rows
    /// beside it. It is here rather than in a paint file because what the
    /// phone MAY NOT do is guess either from the model name.
    #[test]
    fn the_engines_own_answer_reads_back_whole() {
        let roles = serde_json::json!({ "kind": "roles", "ok": true,
            "rows": [{ "effort": null, "model": "gpt-5.6", "priority": false,
                       "provider": "openai-chatgpt", "role": "worker" }] });
        let Ok(Ok(crate::codec::reply::Reply::Roles(rows))) = crate::codec::reply::decode(&roles)
        else {
            unreachable!("the engine's roles answer decodes")
        };
        let worker = crate::codec::pick::worker(&rows).expect("a worker row");
        assert_eq!(worker.provider, "openai-chatgpt");
        assert_eq!(worker.model, "gpt-5.6");
        assert_eq!(worker.effort, None);
        let providers = serde_json::json!({ "kind": "providers", "ok": true,
            "rows": [{ "name": "openai-chatgpt", "fact": "auth oauth2 · signed in",
                       "blocked": null, "effort": true, "priority": true },
                     { "name": "claude-code", "fact": "auth none · no credential needed",
                       "blocked": null, "effort": true, "priority": false }] });
        let Ok(Ok(crate::codec::reply::Reply::Providers(rows))) =
            crate::codec::reply::decode(&providers)
        else {
            unreachable!("the engine's providers answer decodes")
        };
        let (effort, priority) = super::tunable(&rows, Some("openai-chatgpt"));
        assert!(effort.taken && priority.taken, "the row takes both");
        let (effort, priority) = super::tunable(&rows, Some("claude-code"));
        assert!(effort.taken, "and this one takes effort");
        assert_eq!(
            priority.why.as_deref(),
            Some("claude-code does not take priority")
        );
    }
}
