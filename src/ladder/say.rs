//! **What the ladder says in logcat** (bl-df05; the phone's half of yog
//! bl-355c, REMOTE §13.4 "The operator's view of the loop"): one line per
//! rung event, so a live dial can be read off the log instead of off socket
//! state. Every line is built here and nowhere else, so the one rule they
//! share is enforced by the only file that could break it:
//!
//! **counts, sequence numbers, nonces and address families — never an
//! address, a key, a salt or a sealed byte.** A DHT failure is said without
//! its reason for the engine's reason: the walk's refusals name the target it
//! walked toward, a derivation of the key and the salt.
//!
//! **A repeated outcome is said once.** A redial re-enters the ladder on
//! every nap, so the direct rung's outcome and the rest's refusal would
//! otherwise be said per dial; [`Voice::once`] says a line of one kind only
//! when it differs from that kind's last.
//!
//! The sink is injected ([`Say`], on `Rove`) so the fake-DHT bench reads the
//! lines the rungs emit; the device's is [`logcat`]. Every line begins with
//! [`MARKER`] — in the message, not the tag, for `shell::app::probe`'s
//! reason: `android_logger` tags a record by module path.

use crate::rendezvous::call;
use crate::state::Slot;
use std::io;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

/// Where the ladder's lines go.
pub type Say = Arc<dyn Fn(&str) + Send + Sync>;

/// The marker every line begins with.
pub const MARKER: &str = "yog.rendezvous";

/// The device's sink: logcat at info. Off the device — the host suite,
/// which never links `log` — the process's stderr.
pub fn logcat() -> Say {
    Arc::new(|line: &str| emit(line))
}

#[cfg(target_os = "android")]
fn emit(line: &str) {
    log::info!("{line}");
}

#[cfg(not(target_os = "android"))]
fn emit(line: &str) {
    eprintln!("{line}");
}

/// A sink that says nothing: a ladder with no rove has no rungs to say.
pub(crate) fn quiet() -> Say {
    Arc::new(|_: &str| {})
}

/// One ladder's voice: its sink, and the last line of each kind said once.
pub(crate) struct Voice {
    sink: Say,
    last: Slot<Vec<(&'static str, String)>>,
}

impl Voice {
    pub(crate) fn new(sink: Say) -> Voice {
        Voice {
            sink,
            last: Slot::new(Vec::new()),
        }
    }

    /// The sink, for a holder thread that outlives the call that kept it.
    pub(crate) fn sink(&self) -> Say {
        Arc::clone(&self.sink)
    }

    pub(crate) fn say(&self, line: &str) {
        (self.sink)(line);
    }

    /// Say `line` only if the last line of `kind` was another.
    pub(crate) fn once(&self, kind: &'static str, line: &str) {
        let fresh = self.last.with(&mut |last| {
            if let Some((_, was)) = last.iter_mut().find(|(k, _)| *k == kind) {
                let fresh = was != line;
                line.clone_into(was);
                fresh
            } else {
                last.push((kind, line.to_owned()));
                true
            }
        });
        if fresh {
            self.say(line);
        }
    }

    /// Forget `kind`'s last line, so its next is said whatever it is.
    pub(crate) fn forget(&self, kind: &'static str) {
        self.last.with(&mut |last| last.retain(|(k, _)| *k != kind));
    }
}

/// How one direct dial ended, as a class.
pub(crate) fn outcome(e: &io::Error) -> &'static str {
    match e.kind() {
        io::ErrorKind::ConnectionRefused => "refused",
        io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock => "timed out",
        _ => "failed",
    }
}

pub(crate) fn direct(tried: &[(IpAddr, &'static str)]) -> String {
    let each: Vec<String> = tried
        .iter()
        .map(|(ip, how)| format!("{} {how}", family(*ip)))
        .collect();
    let each = if each.is_empty() {
        "none".to_owned()
    } else {
        each.join(", ")
    };
    format!(
        "{MARKER}: direct rung — {} address(es) tried: {each}",
        tried.len()
    )
}

pub(crate) fn unresolved() -> String {
    format!("{MARKER}: direct rung — the entry's name did not resolve")
}

pub(crate) fn resting() -> String {
    format!("{MARKER}: climb skipped — resting after the last failed climb")
}

pub(crate) fn moved(mine: &[IpAddr], held: usize) -> String {
    format!(
        "{MARKER}: network changed — this box now sends from {}; {held} held stream(s) dropped, rest cleared",
        families(mine)
    )
}

pub(crate) fn repunch(cached: &[SocketAddr], window: Duration) -> String {
    format!(
        "{MARKER}: re-punch at {} cached endpoint(s) ({}), window {}s",
        cached.len(),
        families(&ips(cached)),
        window.as_secs()
    )
}

pub(crate) fn no_commons(why: &str) -> String {
    format!("{MARKER}: rendezvous not started — {why}")
}

pub(crate) fn found(seq: i64, endpoints: &[SocketAddr]) -> String {
    format!(
        "{MARKER}: presence read — seq {seq}, {} endpoint(s) ({})",
        endpoints.len(),
        families(&ips(endpoints))
    )
}

/// The presence rung's own sentences carry no address and are said as they
/// stand; anything else is the walk's, and withheld.
pub(crate) fn not_found(why: &str) -> String {
    let why = if call::UNREAD.contains(&why) {
        why
    } else {
        "the DHT walk failed (reason withheld: it names the target)"
    };
    format!("{MARKER}: presence not read — {why}")
}

pub(crate) fn called(called: &call::Called, seq: i64) -> String {
    format!(
        "{MARKER}: call written — nonce {}, seq {seq}, {} endpoint(s) ({}), {} ack(s)",
        called.nonce,
        called.endpoints.len(),
        families(&ips(&called.endpoints)),
        called.acks
    )
}

pub(crate) fn not_called() -> String {
    format!(
        "{MARKER}: call not written — the DHT put failed (reason withheld: it names the target)"
    )
}

pub(crate) fn punching(endpoints: &[SocketAddr], window: Duration) -> String {
    format!(
        "{MARKER}: punch started — {} endpoint(s) ({}), window {}s",
        endpoints.len(),
        families(&ips(endpoints)),
        window.as_secs()
    )
}

pub(crate) fn landed(peer: Option<IpAddr>) -> String {
    let peer = peer.map_or("unknown family", family);
    format!("{MARKER}: punch landed — peer {peer}")
}

pub(crate) fn expired(window: Duration) -> String {
    format!(
        "{MARKER}: punch expired after {}s with no stream",
        window.as_secs()
    )
}

pub(crate) fn kept() -> String {
    format!("{MARKER}: held stream kept for the next ask")
}

pub(crate) fn handed(pings: usize) -> String {
    format!("{MARKER}: held stream handed to an ask — {pings} ping(s) discarded while held")
}

pub(crate) fn dropped(why: &str, pings: usize) -> String {
    format!("{MARKER}: held stream dropped — {why}; {pings} ping(s) discarded while held")
}

fn ips(endpoints: &[SocketAddr]) -> Vec<IpAddr> {
    endpoints.iter().map(SocketAddr::ip).collect()
}

fn family(ip: IpAddr) -> &'static str {
    if ip.is_ipv6() { "v6" } else { "v4" }
}

/// How many of `ips` are of each family, v6 first — `1 v6, 2 v4`, or `none`.
fn families(ips: &[IpAddr]) -> String {
    let v6 = ips.iter().filter(|ip| ip.is_ipv6()).count();
    let v4 = ips.len() - v6;
    let parts: Vec<String> = [(v6, "v6"), (v4, "v4")]
        .into_iter()
        .filter(|(n, _)| *n > 0)
        .map(|(n, family)| format!("{n} {family}"))
        .collect();
    if parts.is_empty() {
        "none".to_owned()
    } else {
        parts.join(", ")
    }
}

#[cfg(test)]
mod tests;
