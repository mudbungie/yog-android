//! **What the composer's selectors offer** (bl-0267): the provider list and
//! each provider's models, held between passes and painted into the snapshot
//! the frame reads.
//!
//! **Held as the engine's own envelopes, decoded at the publish.** The cache
//! stores what is here (§14), and §14's ruling is that a cache holds the
//! engine's bytes rather than a second spelling of them — so this holds the
//! same bytes, one representation, and the decode is done where the snapshot
//! is built. It costs a handful of rows per pass and buys no second home for
//! the fact.
//!
//! **Options belong to a workspace.** Provider sign-ins are per workspace
//! upstream, so a list read under one workspace is not paintable under
//! another: the workspace it was read for is held beside it and the paint is
//! empty when the focus has moved. That is `Snapshot`'s own pairing law, one
//! field over.

use std::collections::BTreeMap;

use serde_json::Value;

use super::{Focus, Snapshot};
use crate::codec::reply::Reply;

/// The selectors' offerings, and whose they are.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(super) struct Options {
    /// The workspace every envelope below was read for.
    workspace: Option<String>,
    /// The `providers` reply, verbatim.
    providers: Option<Value>,
    /// Each `models` reply, verbatim, keyed by the provider asked for.
    models: BTreeMap<String, Value>,
}

impl Options {
    /// Rebuild from what a cache handed back (§14), which is already exactly
    /// this shape.
    pub(super) fn resumed(
        workspace: Option<String>,
        providers: Option<Value>,
        models: BTreeMap<String, Value>,
    ) -> Self {
        Self {
            workspace,
            providers,
            models,
        }
    }

    /// The workspace these belong to, for the cache to store beside them.
    pub(super) fn workspace(&self) -> Option<String> {
        self.workspace.clone()
    }

    /// The stored envelopes, for the cache.
    pub(super) fn envelopes(&self) -> (Option<Value>, BTreeMap<String, Value>) {
        (self.providers.clone(), self.models.clone())
    }

    /// Take one answer, dropping everything that belonged to another
    /// workspace — a fresh workspace's selectors start empty rather than
    /// inheriting the last one's.
    pub(super) fn learned(&mut self, workspace: &str, provider: Option<&str>, envelope: Value) {
        if self.workspace.as_deref() != Some(workspace) {
            *self = Self {
                workspace: Some(workspace.to_owned()),
                ..Self::default()
            };
        }
        match provider {
            None => self.providers = Some(envelope),
            Some(provider) => {
                self.models.insert(provider.to_owned(), envelope);
            }
        }
    }

    /// Paint what belongs to this focus into a snapshot. A decode that will
    /// not read is simply absent: these bytes decoded once when they arrived,
    /// so a failure here is a cache that was tampered with, and an empty
    /// selector is the honest answer to it.
    pub(super) fn paint(&self, focus: &Focus, snap: &mut Snapshot) {
        if self.workspace.is_none() || self.workspace != focus.workspace {
            return;
        }
        if let Some(Reply::Providers(rows)) = self.providers.as_ref().and_then(read) {
            snap.providers = rows;
        }
        for (provider, envelope) in &self.models {
            if let Some(Reply::Models(names)) = read(envelope) {
                snap.models.insert(provider.clone(), names);
            }
        }
    }
}

/// One stored envelope through the one decoder.
fn read(envelope: &Value) -> Option<Reply> {
    crate::codec::reply::decode(envelope).ok()?.ok()
}

#[cfg(test)]
mod tests;

/// The cache's stored offerings as options — one place both the handle and
/// the worker read them from, so the seat that paints before the first pass
/// and the worker that keeps them cannot disagree about what was stored.
pub(super) fn from_cache(kept: crate::cache::Envelopes) -> Options {
    Options::resumed(kept.options_workspace, kept.providers, kept.models)
}
