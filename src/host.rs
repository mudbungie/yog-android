//! **The tool host** (REMOTE §5, §5.3): this device as a machine a session can
//! call. The mirror of the server's own `src/wire/host.rs`, and deliberately
//! the same shape — one loop, three gestures, every one of them an ordinary
//! boundary verb typable at any seat.
//!
//! It is a *client*, not a server, and that is REMOTE §3's routing ruling: the
//! ask never inverts. This thread dials the engine exactly as the seat model
//! does, presents what this machine can run, then **rides a follow-class
//! read** for its next invocation — one ordinary ask whose answer takes as
//! long as it takes. It runs what comes back and posts each capture as an
//! ordinary act. The phone opens no listening socket, ever (DESIGN §1).
//!
//! **It runs serially**, as upstream does: one invocation at a time, so this
//! machine is *absent* — holding no connection — for as long as a tool takes.
//! Nothing in the engine treats presence as the routing predicate, which is
//! exactly why that is safe (REMOTE §5, bl-024b).
//!
//! **It does not reconnect, and it does not die quietly.** A channel that
//! fails stops the host with the sentence that stopped it, published for the
//! frame to paint. Reconnect policy is a statement about how this device is
//! supervised, and a background thread that silently redialled forever would
//! hide a broken seat from the operator holding the phone.
//!
//! **One identity, two connections.** The seat model holds its own; this holds
//! another, on the same material and therefore the same certificate common
//! name. REMOTE §5's presence map is refcounted per identity precisely because
//! one client may hold more than one connection.

use std::sync::mpsc;

use crate::codec::reply::Reply;
use crate::codec::{Act, Ask, Capture, Gesture, Invocation, encode};
use crate::transport::Seat;

/// What a host runs an invocation with: the tool's name and the model's own
/// arguments in, a capture out. Owned and `Send` because it crosses onto the
/// worker thread and outlives the frame that built it.
pub type Dispatch = Box<dyn Fn(&str, &serde_json::Value) -> Capture + Send>;

/// What the frame paints about this device's tool hosting: what is presented,
/// what has been run, and the sentence that stopped it if one did.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Standing {
    /// The advertised tool names, in the order presented.
    pub tools: Vec<String>,
    /// Whether the presentation has been accepted by the engine.
    pub advertised: bool,
    /// How many invocations this host has answered since it started.
    pub served: usize,
    /// The tool that ran most recently, and how it ended.
    pub last: Option<String>,
    /// The sentence that stopped the host. `Some` is a host that is no longer
    /// running: a channel failure is terminal here, by ruling.
    pub stopped: Option<String>,
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
    /// Start the host over an opened seat, presenting `tools` and dispatching
    /// through `run`. Both are parameters rather than reads of
    /// [`crate::tools`] so the loop is testable against a table a test wrote,
    /// and `run` is boxed rather than a bare `fn` because the real dispatch
    /// closes over this app's own storage path (`crate::tools::run_in`).
    pub fn start(seat: Seat, tools: Vec<crate::codec::Tool>, run: Dispatch) -> Self {
        let (tx, standings) = mpsc::channel();
        let worker = std::thread::spawn(move || serve(&seat, tools, &run, &tx));
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
        // process ending is what ends it.
        drop(self.worker.take());
    }
}

/// Present, then wait, run and answer, until something stops it — and publish
/// the sentence that did.
fn serve(
    seat: &Seat,
    tools: Vec<crate::codec::Tool>,
    run: &Dispatch,
    out: &mpsc::Sender<Standing>,
) {
    let mut standing = Standing {
        tools: tools.iter().map(|t| t.name.clone()).collect(),
        ..Standing::default()
    };
    let _ = out.send(standing.clone());
    standing.stopped = Some(hold(seat, tools, run, out, &mut standing));
    let _ = out.send(standing);
}

/// One channel, served. The return is the sentence that stopped it: there is
/// no success exit, so none is spelled — a host's only way out is a gesture
/// that failed, and an `Ok` arm here would be one no state of the world can
/// reach.
fn hold(
    seat: &Seat,
    tools: Vec<crate::codec::Tool>,
    run: &Dispatch,
    out: &mpsc::Sender<Standing>,
    standing: &mut Standing,
) -> String {
    if let Err(why) = tell(seat, &Gesture::Act(Act::Advertise { tools })) {
        return why;
    }
    standing.advertised = true;
    if out.send(standing.clone()).is_err() {
        return "the frame stopped reading".to_owned();
    }
    loop {
        let work = match waited(seat) {
            Ok(work) => work,
            Err(why) => return why,
        };
        for invocation in work {
            let capture = run(&invocation.tool, &invocation.input);
            standing.served += 1;
            standing.last = Some(format!("{} → {}", invocation.tool, capture.exit_code));
            if let Err(why) = answer(seat, &invocation, capture) {
                return why;
            }
            if out.send(standing.clone()).is_err() {
                return "the frame stopped reading".to_owned();
            }
        }
    }
}

/// The follow-class read: this machine's next work. An empty answer is
/// ordinary — a hold that ended quietly — and only a channel failure is not.
fn waited(seat: &Seat) -> Result<Vec<Invocation>, String> {
    match tell(seat, &Gesture::Ask(Ask::Invocations))? {
        Reply::Invocations(rows) => Ok(rows),
        other => Err(format!(
            "the engine answered {}, not this machine's work",
            other.kind()
        )),
    }
}

/// Post one capture back. The receipt is read rather than discarded: an engine
/// that refused the completion — an expired handle, a slot addressed elsewhere
/// — is a thing this host must stop rather than keep answering into.
fn answer(seat: &Seat, invocation: &Invocation, capture: Capture) -> Result<(), String> {
    tell(
        seat,
        &Gesture::Act(Act::Complete {
            invocation: invocation.id.clone(),
            capture,
        }),
    )
    .map(|_| ())
}

/// One gesture over the wire, in the one codec and the one reply decoder — so
/// this host speaks exactly what every other seat speaks and can add nothing
/// to it.
fn tell(seat: &Seat, gesture: &Gesture) -> Result<Reply, String> {
    seat.answered(&encode(gesture))
}

#[cfg(test)]
mod tests;
