//! **The sign-in family** (REMOTE §8.3, bl-61bf and bl-c285): the act that
//! starts `bz --login` for one provider row — or, carrying a `code`, finishes
//! the one waiting for it — and the held read of what that run says.
//!
//! **The run is the server's, and that is the whole point of the pair.** A
//! sign-in performed at this seat would land the credential in this phone's
//! own directories, where no agent reads it; the act names a workspace, the server runs
//! bz inside that workspace's own directory, and the credential lands where the
//! agents that need it run. **Nothing credential-shaped crosses this wire**
//! and this codec could not carry one if it did: what a frame holds is bz's
//! human-facing stream — the authorize URL, a device code, a failure's reason
//! and its remedy — and the token moves from the provider to the box that
//! holds the row over the provider's own channel. The `code` a human copied
//! at that URL crosses back and is not a credential either: it is single-use
//! and redeems only beside a verifier that never leaves that box (§8.3).
//!
//! **A frame is an append, and the settled exit is the last one.** The server
//! hands each read the lines it has not sent yet, so [`LoginView::absorb`] is
//! `codec::follow`'s fold one subject along: lines accrete in order, and
//! `outcome` and `fallback` arrive with the frame that ends the run.
//! **`authorize` is the one key that is not an append** (edition 31): it is a
//! standing fact — the run is waiting for a code, and this is the address to
//! open — so the newest frame's word REPLACES what stood, present or absent,
//! and a server too old to say it never does. A read
//! starts holding nothing, so a lane that re-asks replays the whole buffer
//! and there is nothing to reconcile — REMOTE §8.3's *"a dropped lane, a
//! re-attached seat and a settled run are one case"*.
//!
//! **`err` is which stream a line came down, never a verdict on it.** bz
//! writes its whole human-facing flow to stderr, the authorize URL included,
//! so a surface that painted only the other stream would paint nothing at all
//! (yog's own bl-b4e5, defect 3). A frame carrying no lines at all is the
//! legible emptiness a pair with no run opens on — *nobody has signed in
//! here* is a reading, not a refusal.

use serde_json::{Map, Value};

use super::fields::{arr_of, bool_of, i64_of, opt, str_of};

/// One line the run printed, and which stream it came down.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoginLine {
    /// True for stderr, where bz writes the flow a human reads.
    pub err: bool,
    pub text: String,
}

/// Everything one sign-in has said: its lines, its exit once it settles, and
/// the command to run by hand when it settled badly.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LoginView {
    pub lines: Vec<LoginLine>,
    /// The terminal exit code once the run settles — `None` while it is
    /// still going, which is the one thing a surface must not read as zero.
    pub outcome: Option<i64>,
    /// The server's own workspace-bound spelling of the command, set only on
    /// a non-zero exit. It is carried and never composed here: yog mints it
    /// while the workspace is in hand, and a second spelling of it at this
    /// end would drift from what that server actually accepts.
    pub fallback: Option<String>,
    /// **The address to open, while the run waits for a code** (§8.3, edition
    /// 31). Never beside an `outcome`. Its presence is also the whole of the
    /// skew rule: only a server that can take a `code` ever writes it.
    pub authorize: Option<String>,
}

impl LoginView {
    /// Absorb the frame that landed **after** this one's lines (§8.3). The
    /// later frame's settlement wins where it states one, so a fold that has
    /// seen the exit keeps it and one that has not takes it the moment it
    /// arrives.
    pub fn absorb(&mut self, later: Self) {
        self.lines.extend(later.lines);
        self.outcome = later.outcome.or(self.outcome);
        self.fallback = later.fallback.or_else(|| self.fallback.take());
        self.authorize = later.authorize;
    }

    /// **The address a code is wanted from**: stated, and the run not ended.
    /// The one question the paste door asks, so a settled run can never show
    /// one whatever its last frame carried.
    pub fn awaits(&self) -> Option<String> {
        self.authorize.clone().filter(|_| !self.settled())
    }

    /// Whether the run has ENDED — the lane's own stop condition, asked of
    /// the fold rather than derived at each reader. A settled run's lane is
    /// over upstream, so a seat that went on wanting one would redial a
    /// finished sign-in every pass.
    pub fn settled(&self) -> bool {
        self.outcome.is_some()
    }
}

/// One `login` reply body.
pub(crate) fn view(o: &Map<String, Value>) -> Result<LoginView, String> {
    Ok(LoginView {
        lines: arr_of(o, "lines")?
            .iter()
            .map(line)
            .collect::<Result<Vec<_>, _>>()?,
        outcome: opt(o, "outcome", i64_of)?,
        fallback: opt(o, "fallback", str_of)?,
        authorize: opt(o, "authorize", str_of)?,
    })
}

/// One streamed line.
fn line(v: &Value) -> Result<LoginLine, String> {
    let o = v.as_object().ok_or("login line: not a JSON object")?;
    Ok(LoginLine {
        err: bool_of(o, "err")?,
        text: str_of(o, "text")?,
    })
}

#[cfg(test)]
mod tests;
