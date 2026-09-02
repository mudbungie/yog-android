//! **The paint-first cache** (bl-de96): the last answer the engine gave,
//! kept in this app's private storage so a resumed app paints what it had
//! instead of three empty lists while the wire re-reads the world.
//!
//! **The wire is the only authority.** This is a cache in the strict sense —
//! one writer (the model's worker, after a pass the engine answered), one
//! reader (the model's boot), and the next cadence read replaces whatever it
//! painted. Nothing here is ever consulted to decide anything; it decides
//! only what is on the glass for the second before the first answer lands.
//!
//! **It stores the ENGINE's spelling, not a second one, and this is a
//! deviation from the ball's own sketch, taken deliberately.** bl-de96 asked
//! for the decoded `Snapshot` serialized with a version stamp. Writing that
//! means writing a reply *encoder* — and `tests/conformance/replies.rs`
//! records why this client has none: *"a reply encoder would be a second
//! implementation of the engine's own spelling with nothing to check it
//! against"*. `Entry` alone is a tree of blocks and untyped usage; an encoder
//! for it would drift from the decoder the first time a field moved, silently
//! and only in the cache. So what is stored is the reply envelopes verbatim,
//! and reading them is the ONE decoder every wire answer goes through. The
//! risk the sketch was avoiding — a local file speaking with the engine's
//! authority — is answered by the decoder being strict and by this module
//! being fail-closed: any doubt at all discards the whole file.
//!
//! **Two version stamps, because two things can move.** [`VERSION`] is this
//! file's own layout; `protocol` is the wire's (`hello::PROTOCOL`), because
//! the envelopes inside are the wire's bytes and a protocol move can change
//! what they mean. Either mismatch discards.
//!
//! **It never touches the enrollment path.** Material and snapshots share no
//! file and no directory: this is `<internal>/cache/`, the wire's material is
//! `<internal>/wire/`, and nothing here reads or writes a key.

use std::path::Path;

use serde_json::{Value, json};

use crate::codec::reply::{self, Reply};
use crate::seat::{Focus, Snapshot};

/// The file's marker and its layout version, in one field — the shape
/// `crate::envelope` gives the enroll payload, for its reason: a version
/// with no name is read out of whatever JSON happens to be there.
const TAG: &str = "yog-seat-cache";
const VERSION: u64 = 2;

/// **What one answered pass carried, in the engine's own words.** Present
/// exactly as deep as the focus went: a pass under no workspace asks one
/// question and keeps one envelope.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Envelopes {
    pub workspaces: Option<Value>,
    pub conversations: Option<Value>,
    pub transcript: Option<Value>,
    /// **The composer selectors' offerings** (bl-0267), stored the same way
    /// and under the same rules: the workspace they were read for, the
    /// `providers` reply, and each provider's `models` reply. They are not
    /// part of a pass — the selectors are their own gestures — so they ride
    /// beside the pass's three rather than inside them, and a file whose
    /// options name a workspace the focus does not is discarded like any
    /// other mispairing.
    pub options_workspace: Option<String>,
    pub providers: Option<Value>,
    pub models: std::collections::BTreeMap<String, Value>,
}

/// Store one pass. The `Err` is for the caller's log and nothing else — a
/// cache that could not be written is a cache miss next boot.
pub fn write(path: &Path, focus: &Focus, kept: &Envelopes) -> Result<(), String> {
    let body = json!({
        TAG: VERSION,
        "protocol": crate::hello::PROTOCOL,
        "focus": { "workspace": focus.workspace, "agent": focus.agent },
        "workspaces": kept.workspaces,
        "conversations": kept.conversations,
        "transcript": kept.transcript,
        "options": { "workspace": kept.options_workspace,
                     "providers": kept.providers,
                     "models": kept.models },
    });
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    std::fs::write(path, body.to_string()).map_err(|e| format!("{}: {e}", path.display()))
}

/// Read one back, or `None`.
///
/// **One answer for every way it can fail**, because a caller has exactly one
/// thing to do with all of them: paint nothing and wait for the wire. Absent,
/// unreadable, not JSON, another layout version, another protocol, an
/// envelope this build's decoder refuses, an envelope of the wrong kind, or a
/// depth that disagrees with the focus — each discards the whole file rather
/// than half-reading it.
pub fn read(path: &Path) -> Option<(Focus, Snapshot, Envelopes)> {
    let value: Value = serde_json::from_slice(&std::fs::read(path).ok()?).ok()?;
    if value.get(TAG)?.as_u64()? != VERSION
        || value.get("protocol")?.as_u64()? != u64::from(crate::hello::PROTOCOL)
    {
        return None;
    }
    let focus = Focus {
        workspace: at2(&value, "focus", "workspace"),
        agent: at2(&value, "focus", "agent"),
    };
    // The pairing law `Snapshot` keeps, checked on the FILE: rows deeper than
    // the focus they were asked at are not paintable, and a file that carries
    // them is a file written by something this build does not understand.
    if held(&value, "conversations").is_some() != focus.workspace.is_some()
        || held(&value, "transcript").is_some() != focus.agent.is_some()
    {
        return None;
    }
    let mut snap = Snapshot {
        focus: focus.clone(),
        ..Snapshot::default()
    };
    let kept = options(&value, &focus)?;
    match decoded(&value, "workspaces")? {
        Reply::Workspaces { rows, .. } => snap.workspaces = rows,
        _ => return None,
    }
    if let Some(held) = held(&value, "conversations") {
        match reply::decode(&held).ok()?.ok()? {
            Reply::Conversations(rows) => snap.conversations = rows,
            _ => return None,
        }
    }
    if let Some(held) = held(&value, "transcript") {
        match reply::decode(&held).ok()?.ok()? {
            Reply::Transcript(rows) => snap.transcript = rows,
            _ => return None,
        }
    }
    Some((focus, snap, kept))
}

/// The stored selector offerings, checked against the focus that owns them.
/// **Absent is ordinary** — the selectors may simply never have been opened —
/// but options naming another workspace than the focus are the mispairing the
/// whole file is fail-closed about, so they discard it.
fn options(value: &Value, focus: &Focus) -> Option<Envelopes> {
    let Some(held) = held(value, "options") else {
        return Some(Envelopes::default());
    };
    let workspace = at(&held, "workspace");
    if workspace.is_some() && workspace != focus.workspace {
        return None;
    }
    let models = held
        .get("models")
        .and_then(Value::as_object)
        .map(|listed| {
            listed
                .iter()
                .map(|(provider, envelope)| (provider.clone(), envelope.clone()))
                .collect()
        })
        .unwrap_or_default();
    Some(Envelopes {
        options_workspace: workspace,
        providers: held.get("providers").filter(|v| !v.is_null()).cloned(),
        models,
        ..Envelopes::default()
    })
}

/// One stored envelope, decoded by the one decoder. The roster's is required:
/// a pass that answered nothing is a pass that was never stored.
fn decoded(value: &Value, key: &str) -> Option<Reply> {
    reply::decode(&held(value, key)?).ok()?.ok()
}

/// A stored envelope, with JSON `null` reading as absent — the wire's own
/// reading of a real null, kept here so a written `None` comes back as one.
/// Owned out (rule 1), and the clone is a boot-time one per envelope.
fn held(value: &Value, key: &str) -> Option<Value> {
    value.get(key).filter(|held| !held.is_null()).cloned()
}

/// One nested string field, absent for every shape that is not one.
fn at2(value: &Value, outer: &str, key: &str) -> Option<String> {
    Some(value.get(outer)?.get(key)?.as_str()?.to_owned())
}

/// One string field of a held value.
fn at(value: &Value, key: &str) -> Option<String> {
    Some(value.get(key)?.as_str()?.to_owned())
}

#[cfg(test)]
mod tests;
