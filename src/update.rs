//! **The phone's release channel, read half** (bl-7a68, DESIGN §20): is the
//! newest published APK newer than this one, and where is it.
//!
//! Every other box in the fleet reconciles itself — the engine off `ghcr.io`,
//! a workstation off the crates.io sparse index, the seat and the foot off
//! their own crates. All five ask one question: *read the newest live version
//! off a registry, compare it with what is installed, install if it differs.*
//! **Android will not let a process replace itself**, so this device answers
//! the first two rungs and stops at the third: it detects, and it OFFERS.
//! The install is the system installer's, in front of a person who taps.
//!
//! This module is the whole of the decision and it is pure: a string in — the
//! `dev.yog.Update` bridge's two-line answer, carrying the body of `GET
//! /repos/<owner>/<repo>/releases/latest` — and an [`Offer`] or nothing out.
//! The platform is on the other side of that string, which is what puts the
//! version comparison and the asset pick under the 100% floor rather than in
//! Java where nothing here can test them.
//!
//! **What it will not do.** No downgrade: the tag has to be strictly greater,
//! so deleting a release and publishing another is the only rollback there is
//! (there is no yank for a GitHub Release, and this states that rather than
//! implying otherwise). No version list: `latest` is one release, exactly as
//! the other four reconcilers read one. And no offer at all from a plaintext
//! URL — an APK is code this device is about to run, so the one transport it
//! may arrive over is the one the feed itself came over.

/// What is on offer: the version the newest release names, and the APK asset
/// under it. Owned and concrete — the caller hands the URL back to the bridge
/// a frame or a minute later, so a borrow of the feed would be a borrow of a
/// string the JNI call already dropped.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Offer {
    pub version: String,
    pub url: String,
}

/// The two-line protocol's success prefix, the same one every bridge in this
/// app answers in (`dev.yog.App.OK`).
const OK: &str = "ok\n";

/// The one file extension an offer may name. The release workflow attaches
/// exactly one APK and nothing else that ends this way, and a release with no
/// APK at all — a source-only tag, a half-finished upload — is an offer this
/// app must not make rather than a failure worth a sentence.
const APK: &str = ".apk";

/// **Has the feed concluded?** The bridge answers nothing at all — the empty
/// string — while its one fetch is still in flight, and a two-line answer once
/// it has finished, whether that is the release document or a refusal. One
/// caller needs those apart: the shell asks once per frame until this is true
/// and then never again, because a fetch that has concluded cannot conclude
/// differently.
///
/// **The silence is a third state and not a spelling choice.** A refusal
/// written while the socket was still open would be indistinguishable from one
/// the network actually gave, so a phone with no network would cache the wrong
/// verdict and a phone with a slow one would be crossed the JNI boundary on
/// every frame forever.
pub fn answered(answer: &str) -> bool {
    !answer.is_empty()
}

/// Read the bridge's answer against the version this build IS.
///
/// `None` is every state that offers nothing, and they are deliberately one
/// answer: the fetch has not come back, the network refused, the repository
/// has no release yet, the body was not JSON, the tag is not newer, the
/// release carries no APK. A phone with nothing to say about updates says
/// nothing — the offer is one row on a screen, never a nag, so the absence of
/// a row IS the vocabulary for all six.
pub fn offer(answer: &str, installed: &str) -> Option<Offer> {
    let body = answer.strip_prefix(OK)?;
    let feed: serde_json::Value = serde_json::from_str(body).ok()?;
    let tag = feed.get("tag_name")?.as_str()?;
    let version = tag.strip_prefix('v').unwrap_or(tag);
    if !newer(installed, version) {
        return None;
    }
    let url = asset(feed.get("assets")?.as_array()?)?;
    Some(Offer {
        version: version.to_owned(),
        url,
    })
}

/// The APK's download URL out of a release's asset list — the first asset
/// whose name ends in `.apk` and whose URL is `https`.
///
/// The scheme is checked here rather than trusted from the host, because what
/// this URL reaches is handed to the system installer: a feed that named
/// `http://` would be a downgrade of the channel's transport arranged by
/// whoever answered the read.
fn asset(assets: &[serde_json::Value]) -> Option<String> {
    assets
        .iter()
        .filter(|asset| {
            asset
                .get("name")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|name| name.ends_with(APK))
        })
        .find_map(|asset| {
            let url = asset.get("browser_download_url")?.as_str()?;
            url.starts_with("https://").then(|| url.to_owned())
        })
}

/// Is `latest` a strictly greater version than `installed`?
///
/// Dotted decimal, component by component, a missing component reading zero
/// so `0.1` and `0.1.0` are the same version rather than an ordering nobody
/// intended. **Anything that does not parse as decimal is not newer** — a
/// pre-release suffix, a tag somebody typed by hand, an empty string. That is
/// fail-closed in the direction that matters: the failure of a version
/// comparison must be *offer nothing*, never *offer whatever this was*.
pub fn newer(installed: &str, latest: &str) -> bool {
    let (Some(was), Some(is)) = (parts(installed), parts(latest)) else {
        return false;
    };
    // Padded to one width first, so the comparison is a plain lexicographic
    // one over equal-length vectors — `0.1` against `0.1.0` is then equality
    // rather than a length rule written out beside the ordering.
    let width = was.len().max(is.len());
    padded(&is, width) > padded(&was, width)
}

/// One version's components, zero-extended to `width`.
fn padded(version: &[u64], width: usize) -> Vec<u64> {
    (0..width)
        .map(|at| version.get(at).copied().unwrap_or(0))
        .collect()
}

/// One version as its decimal components, or `None` when any of them is not
/// a decimal. The whole version is refused rather than the bad component
/// skipped: a version with an unreadable part is one this app cannot order,
/// and ordering it anyway is how a `0.1.0-rc1` becomes an offer.
fn parts(version: &str) -> Option<Vec<u64>> {
    let version = version.strip_prefix('v').unwrap_or(version);
    if version.is_empty() {
        return None;
    }
    version.split('.').map(|part| part.parse().ok()).collect()
}

#[cfg(test)]
mod tests;
