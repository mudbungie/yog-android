//! **The word each answer is spelled with** — the reply side's mirror of
//! `codec::encode`, split from the vocabulary beside it (bl-5a41) when the
//! sign-in's frame took `reply.rs` to the 300 wall. The seam is the one
//! `codec.rs` and `codec::encode` draw one layer up and `reply::decode` draws
//! at this one: what an answer IS, and how it is SPELLED.
//!
//! It is named here rather than at each caller because two readers already
//! need it — the seat model and the tool host both say *"the engine answered
//! X instead"* — and a second table of these words would drift from the
//! decoder's own.

use super::Reply;

impl Reply {
    /// This answer's `kind` token — the word the engine wrote.
    pub fn kind(&self) -> String {
        match self {
            Self::Outcome { .. } => "outcome",
            Self::Workspaces { .. } => "workspaces",
            Self::Conversations(_) => "conversations",
            Self::Transcript(_) => "transcript",
            Self::Advertised { .. } => "advertised",
            Self::Invocations(_) => "invocations",
            Self::Routed { .. } => "routed",
            Self::Prepared(_) => "prepared",
            Self::Started { .. } => "started",
            Self::Providers(_) => "providers",
            Self::Models(_) => "models",
            Self::Roles(_) => "roles",
            Self::Applied => "applied",
            Self::Nudged => "nudged",
            Self::Flagged => "flagged",
            Self::Follow(_) => "follow",
            Self::Login(_) => "login",
            Self::Search(_) => "search",
            Self::Attention(_) => "attention",
            Self::Ops(_) => "ops",
            Self::Agent(_) => "agent",
            Self::Steps(_) => "steps",
            Self::Step(_) => "step",
            Self::Rail(_) => "rail",
            Self::Governing(_) => "governing",
            Self::Inbox(_) => "inbox",
            Self::Clients(_) => "clients",
            Self::Lineages(_) => "lineages",
            Self::Science(_) => "science",
            Self::Enrolled(_) => "enrolled",
            Self::Config(_) => "config",
            Self::Marks(_) => "marks",
            Self::Deleted => "deleted",
            Self::Files(_) => "files",
            Self::WorkDiff(_) => "work-diff",
            Self::Fanned(_) => "fanned",
            Self::Delivered(_) => "delivered",
            Self::Retired { .. } => "retired",
            Self::Armed { .. } => "armed",
            Self::Balls(_) => "balls",
            Self::WorkspaceBalls(_) => "workspace-balls",
            Self::Board(_) => "board",
            Self::Acked => "acked",
            Self::Acknowledged(_) => "acknowledged",
            Self::TrailCleared => "trail-cleared",
            Self::Answered(_) => "answered",
            Self::Floored { .. } => "floored",
        }
        .to_owned()
    }
}
