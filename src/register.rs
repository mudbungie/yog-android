//! **Register this device** (yog REMOTE §15.1 steps 2 to 4; DESIGN §5): claim
//! a one-time offer with a key made here, file what comes back as this
//! device's material, and show the word the operator approves by.
//!
//! In the order it happens:
//!
//! 1. The URL is read ([`url`]); anything that is not a registration URL
//!    refuses before a socket opens.
//! 2. This device makes a P-256 key pair ([`key`]). The private half is never
//!    sent anywhere.
//! 3. The public half is posted to the URL over HTTPS. **The request itself
//!    is the caller's** — on a phone it is the platform's own HTTPS client,
//!    verifying against the platform's public web roots (`dev.yog.Register`)
//!    — and it arrives here as the `post` argument, which is what lets every
//!    line of this module run on the host. Anything but `200` is *this offer
//!    cannot be claimed*: the door answers one bare `404` for a code that is
//!    unknown, expired or already used, and does not say which.
//! 4. The answer and the private key are filed ([`entry`]).
//! 5. The confirmation word is computed from the public key and the CA the
//!    answer carried — and is computable again from the files alone
//!    ([`word`]), so no screen has to remember it.
//! 6. The wire is dialled with the new material ([`Watcher`]). The server
//!    refuses a certificate nobody has approved — that refusal is expected,
//!    and being heard at all is what marks the offer proved. So the first
//!    answer of any kind is *waiting for approval*, and the wait ends when
//!    the answer changes: a reply, or a different refusal.
//!
//! **This is the one place this app makes a key, and it never receives one.**
//! The carried envelope it replaces put a private key in a QR symbol; a copy
//! of that symbol was the device.

use std::path::{Path, PathBuf};

use serde_json::json;

use crate::codec::{Ask, Gesture, encode};
use crate::transport::{Seat, Wire};

/// What the door answered, filed as this device's material.
pub mod entry;
/// The key made here, and the confirmation word.
pub mod key;
/// The registration URL.
pub mod url;

/// What the outcome's line for the confirmation word begins with.
pub const WORD_LINE: &str = "Confirmation word";

/// What the door answered: the status, and the body where there was one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Answer {
    pub status: u16,
    pub body: Vec<u8>,
}

/// **The platform bridge's answer, read** — the two-line protocol every
/// bridge here speaks, one line longer: `ok`, the status, then the body
/// verbatim; or `err` and the sentence the request failed with.
pub fn answer(said: &str) -> Result<Answer, String> {
    let unread = || "the registration bridge answered nothing readable".to_owned();
    let (verdict, rest) = said.split_once('\n').ok_or_else(unread)?;
    if verdict != "ok" {
        return Err(rest.trim().to_owned());
    }
    let (status, body) = rest.split_once('\n').unwrap_or((rest, ""));
    Ok(Answer {
        status: status.trim().parse().map_err(|_| unread())?,
        body: body.as_bytes().to_vec(),
    })
}

/// A device that claimed its offer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Registered {
    pub workspace: String,
    /// The name the server knows this device by.
    pub name: String,
    pub grade: String,
    /// The six-character confirmation word.
    pub word: String,
    /// What this device dials, as its `address` file states it.
    pub addresses: Vec<String>,
}

impl Registered {
    /// Who this device now is, in one sentence.
    pub fn said(&self) -> String {
        format!(
            "Registered as {} ({}) in workspace {}.",
            self.name, self.grade, self.workspace
        )
    }

    /// What the operator does with the word.
    pub fn approve(&self) -> String {
        format!(
            "On a seat that is already connected, approve the offer whose row shows that word: \
             `lernie approve {} {}`, or Settings › Servers › offers.",
            self.workspace, self.name
        )
    }
}

/// **Claim the offer at `url`** into `dir`, sending the one request through
/// `post` — which takes the URL and the JSON body and answers what the door
/// said.
pub fn claim(
    dir: &Path,
    url: &str,
    post: &mut dyn FnMut(&str, &str) -> Result<Answer, String>,
) -> Result<Registered, String> {
    let offer = url::parse(url)?;
    let made = key::make()?;
    let body = json!({ "public_key": key::pem("PUBLIC KEY", &made.spki) }).to_string();
    let at = format!("https://{}{}", offer.authority(), offer.path());
    let answer = post(&at, &body).map_err(|why| {
        format!(
            "this offer could not be reached: {why}. Nothing was claimed; the URL is still good \
             if it has not expired."
        )
    })?;
    if answer.status != 200 {
        return Err(format!(
            "this offer cannot be claimed: {} answered {}. The code is unknown, expired or \
             already used — a registration URL works once. Ask for a new offer.",
            offer.authority(),
            answer.status
        ));
    }
    let claimed = entry::read(&answer.body)?;
    let word = key::word(&made.spki, &claimed.ca)?;
    entry::file(dir, &claimed, &key::pem("PRIVATE KEY", &made.pkcs8))?;
    Ok(Registered {
        workspace: claimed.workspace,
        name: claimed.name,
        grade: claimed.grade,
        word,
        addresses: claimed.addresses,
    })
}

/// **The confirmation word of the material in `dir`**, read off the files:
/// the private key's public half and the operator CA. `None` where there is
/// no material, or its key is one this app did not make — a leaf carried here
/// by hand was never offered, so it has no word.
pub fn word(dir: &Path) -> Option<String> {
    let read = |leaf: &str| std::fs::read_to_string(dir.join(leaf)).ok();
    let spki = key::spki_of(&read(crate::material::KEY)?)?;
    key::word(&spki, &read(crate::material::ANCHORS)?).ok()
}

/// Where a claimed offer stands, as one dial found it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Watch {
    /// The server heard this device and refuses it: the offer is proved, and
    /// not yet approved.
    Waiting,
    /// The server's answer changed — a reply, or another refusal than the
    /// first: nothing holds this device any more.
    Approved,
    /// The dial reached nothing, and why. The offer cannot be approved until
    /// one does.
    Unreached(String),
}

impl Watch {
    /// The line under the confirmation word.
    pub fn said(&self) -> String {
        match self {
            Self::Waiting => "Waiting for approval: the server has heard this device and will \
                              refuse it until the offer is approved."
                .to_owned(),
            Self::Approved => "Approved.".to_owned(),
            Self::Unreached(why) => format!(
                "The server has not heard this device yet, so the offer cannot be approved: \
                 {why}. The material is filed; still dialling."
            ),
        }
    }
}

/// **The proving dial, and every dial after it** (REMOTE §15.1 step 4): one
/// ordinary request down the new material, read for whether the hold stands.
///
/// **Approval is read as the answer changing, never as a sentence matched.**
/// A held certificate is refused everything in one sentence of the server's
/// own; once approved, an operator-grade device is answered and a foot is
/// refused *differently* — it may not ask this question at all. Remembering
/// the first refusal and waiting for anything else covers both grades and
/// quotes no prose this app does not own.
pub struct Watcher {
    dir: PathBuf,
    held: Option<String>,
}

impl Watcher {
    pub fn new(dir: &Path) -> Self {
        Self {
            dir: dir.to_path_buf(),
            held: None,
        }
    }

    /// Dial once.
    pub fn look(&mut self) -> Watch {
        let asked = crate::material::read_dir(&self.dir)
            .and_then(|held| held.ok_or_else(|| "no material is filed".to_owned()))
            .and_then(|held| Seat::open(&held))
            .map_err(Wire::Transport)
            .and_then(|seat| seat.answered(&encode(&Gesture::Ask(Ask::Workspaces))));
        match asked {
            Err(Wire::Refused(said)) if self.held.as_ref().is_none_or(|held| *held == said) => {
                self.held = Some(said);
                Watch::Waiting
            }
            Err(unreached) if unreached.transport() => Watch::Unreached(unreached.sentence()),
            _ => Watch::Approved,
        }
    }
}

#[cfg(test)]
mod tests;
