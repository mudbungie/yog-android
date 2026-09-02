//! **The tool host** (REMOTE §5, §5.3): this device as a machine a session can
//! call. The mirror of the server's own `src/wire/host.rs`, and deliberately
//! the same shape — one loop, three gestures, every one of them an ordinary
//! boundary verb typable at any seat.
//!
//! This file is the **handle** the frame holds and the standing it paints;
//! the loop itself is `host::serve`, split out when redialling gave the
//! worker a policy of its own (bl-8641).
//!
//! It is a *client*, not a server, and that is REMOTE §3's routing ruling: the
//! ask never inverts. The worker thread dials the engine exactly as the seat
//! model does, presents what this machine can run, then **rides a follow-class
//! read** for its next invocation — one ordinary ask whose answer takes as
//! long as it takes. It runs what comes back and posts each capture as an
//! ordinary act. The phone opens no listening socket, ever (DESIGN §1).
//!
//! **It runs serially**, as upstream does: one invocation at a time, so this
//! machine is *absent* — holding no connection — for as long as a tool takes.
//! Nothing in the engine treats presence as the routing predicate, which is
//! exactly why that is safe (REMOTE §5, bl-024b).
//!
//! **It redials a broken channel and stops on a refusal** (bl-8641, reversing
//! the founding ruling). The earlier one — a channel that fails stops the host
//! with the sentence that stopped it — was written for a supervised box, and
//! this is a phone: it changes networks hourly, and one `receive: Software
//! caused connection abort` left the host dead until the app was restarted.
//! What made silent redialling wrong was the silence, not the redial, so the
//! standing line says `reconnecting` with the sentence that broke the channel
//! for as long as it is climbing the ladder (§6). A refusal that is not the
//! channel — the engine declining the advertisement, a version that cannot be
//! spoken to, an answer of the wrong kind — still stops for good, because
//! nothing about dialling again changes any of them. [`crate::transport::Wire`]
//! is where the two are told apart.
//!
//! **One identity, two connections.** The seat model holds its own; this holds
//! another, on the same material and therefore the same certificate common
//! name. REMOTE §5's presence map is refcounted per identity precisely because
//! one client may hold more than one connection.
//!
//! **A carried working directory is refused here, and that is the whole of
//! REMOTE §5.4's worktree lane on this device** (bl-0ac8). §5.1 puts the
//! consent on the advertisement — *"it stays checkable because the box that
//! stated it is the box that enforces it (thrall refuses a carried cwd against
//! an unconsenting entry, in band, naming the key)"* — and this box states
//! none: DESIGN §6's lawful §5.2 deviation dispatches to a Rust function
//! rather than spawning an argv, so there is nothing to spawn *at*, and
//! `tools::tool` has no parameter with which to say otherwise. So the check is
//! that fact's consequence rather than a second copy of it, and
//! `tools::tests` holds the invariant that keeps them one: no advertised entry
//! consents. Refusing rather than dropping, because a `cwd` silently ignored
//! is both ends believing a tool ran in the conversation's worktree when it
//! ran wherever this app's uid happened to be — the quiet miss the lane exists
//! to exclude.

use std::sync::mpsc;
use std::time::Duration;

use crate::codec::Capture;
use crate::foot::Foot;

mod serve;

/// What a host runs an invocation with: the tool's name and the model's own
/// arguments in, a capture out. Owned and `Send` because it crosses onto the
/// worker thread and outlives the frame that built it.
pub type Dispatch = Box<dyn Fn(&str, &serde_json::Value) -> Capture + Send>;

/// **How the worker waits between redials** — a parameter for [`Dispatch`]'s
/// own reason: the ladder is policy, and a test that had to sleep through it
/// would be a test of `std::thread::sleep`. The device passes exactly that;
/// the suite passes a recorder and reads the schedule back.
pub type Nap = Box<dyn Fn(Duration) + Send>;

/// **Whether the host is serving, climbing back, or done.** One field with
/// three states rather than two booleans and two sentences: a host is in
/// exactly one of them, and the frame paints whichever it is (§6).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Health {
    /// A channel is up, or the first one is being opened.
    #[default]
    Serving,
    /// The channel broke; the sentence that broke it, and a redial is
    /// pending. Transient by construction — the worker is still alive.
    Redialling(String),
    /// Done, for good: a refusal no redial can mend, or a frame that stopped
    /// reading. The worker has returned.
    Stopped(String),
}

/// What the frame paints about this device's tool hosting: what is presented,
/// what has been run, and where the host stands.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Standing {
    /// The advertised tool names, in the order presented.
    pub tools: Vec<String>,
    /// Whether the presentation has been accepted by the engine **on the
    /// channel that is up now**. A redial clears it: the presentation went
    /// with the connection that carried it.
    pub advertised: bool,
    /// How many invocations this host has answered since it started —
    /// across every channel it has held, because it is the same host.
    pub served: usize,
    /// The tool that ran most recently, and how it ended.
    pub last: Option<String>,
    /// Serving, redialling, or stopped.
    pub health: Health,
}

/// The frame's handle on the host thread. Dropping it stops the host at its
/// next loop boundary; a host waiting on the wire ends when its connection
/// does, which a dropped process does for it.
pub struct Host {
    standings: mpsc::Receiver<Standing>,
    last: Standing,
    worker: Option<std::thread::JoinHandle<()>>,
}

impl Host {
    /// Start the host over an opened seat, presenting `tools`, dispatching
    /// through `run` and resting between redials through `nap`. All three are
    /// parameters rather than reads of [`crate::tools`] and the clock so the
    /// loop is testable against a table a test wrote, and `run` is boxed
    /// rather than a bare `fn` because the real dispatch closes over this
    /// app's own storage path (`crate::tools::run_in`).
    pub fn start(foot: Foot, tools: Vec<crate::codec::Tool>, run: Dispatch, nap: Nap) -> Self {
        let (tx, standings) = mpsc::channel();
        let worker = std::thread::spawn(move || serve::serve(&foot, tools, &run, &nap, &tx));
        Self {
            standings,
            last: Standing::default(),
            worker: Some(worker),
        }
    }

    /// The latest published standing — non-blocking, like the seat model's
    /// snapshot, because the frame paints at its own cadence.
    pub fn standing(&mut self) -> Standing {
        while let Ok(standing) = self.standings.try_recv() {
            self.last = standing;
        }
        self.last.clone()
    }
}

impl Drop for Host {
    fn drop(&mut self) {
        // The thread is not joined: it may be parked on a follow-class read
        // that answers only when there is work, and a frame that blocked on
        // that would be the UI freeze this client's whole shape excludes. The
        // process ending is what ends it — and a worker between redials ends
        // at its next publish, which finds the receiver gone.
        drop(self.worker.take());
    }
}

#[cfg(test)]
mod tests;
