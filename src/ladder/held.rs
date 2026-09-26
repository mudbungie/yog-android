//! **Held punched streams** (yog REMOTE §13.4): the pool a finished ask
//! returns its connection to, and the holder that reads each one through
//! its silence until the next ask takes it or the silence says it is gone.
//!
//! **Why a holder thread at all.** The engine writes `{"ping":true}` into a
//! held connection every 25 seconds of silence and hangs up after two
//! minutes of it; a stream nobody reads cannot tell a live mapping from a
//! dead one, and an ask written into a dead one is a lost reply — REMOTE §3's
//! in-doubt, manufactured. So every held stream has a reader: it discards
//! pings, hands the stream over the moment an ask wants it, and hangs up
//! itself when [`SILENCE`] passes with no frame at all — the phone's own
//! half of the two-minute rule, on the injected clock so the suite can walk
//! it in an instant. The read wakes every [`TICK`] to look for a taker,
//! which is what an ask's reuse costs in latency and what an idle held
//! stream costs the radio; both are stated defaults to revisit on evidence.
//!
//! **A ping is never the start of a reply** and is dropped wherever it is
//! read — here between asks, and in `transport::Open::each` ahead of an
//! answer, since one written just before the request landed sits in the
//! buffer ahead of the reply.

use super::Clock;
use crate::frame;
use crate::state::Slot;
use rustls::{ClientConnection, StreamOwned};
use serde_json::Value;
use std::io;
use std::net::TcpStream;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::time::{Duration, Instant};

/// How long a held stream may carry nothing at all before this end hangs
/// up: the engine's own bound, mirrored.
pub(crate) const SILENCE: Duration = Duration::from_mins(2);

/// How often a holder looks up from the socket for a taker.
const TICK: Duration = Duration::from_millis(250);

/// A punched connection with its preface spent: the stream, and the edition
/// the engine stated on it (`transport::Open::edition`).
pub struct Held {
    pub tls: StreamOwned<ClientConnection, TcpStream>,
    pub edition: u32,
}

/// One held stream's reader, from the pool's side.
struct Holder {
    wanted: Arc<AtomicBool>,
    back: mpsc::Receiver<Held>,
}

/// The pool: every held stream this ladder is keeping.
pub(crate) struct Pool {
    holders: Slot<Vec<Holder>>,
}

impl Pool {
    pub(crate) fn new() -> Pool {
        Pool {
            holders: Slot::new(Vec::new()),
        }
    }

    /// Keep `held` for the next ask, read through its silence on `clock`.
    pub(crate) fn keep(&self, held: Held, clock: Arc<dyn Clock>) {
        let wanted = Arc::new(AtomicBool::new(false));
        let (tx, back) = mpsc::channel();
        let flag = Arc::clone(&wanted);
        // The silence starts NOW, on this thread — not whenever the holder
        // is first scheduled, which a loaded box can put after the clock has
        // already moved.
        let since = clock.now();
        std::thread::spawn(move || hold(held, since, &flag, &tx, clock.as_ref()));
        let mut holder = Some(Holder { wanted, back });
        self.holders
            .with(&mut |holders| holders.extend(holder.take()));
    }

    /// The first held stream still alive, or none. A holder that has hung
    /// up, or does not answer within a few ticks, is forgotten on the way.
    pub(crate) fn take(&self) -> Option<Held> {
        while let Some(holder) = self.holders.with(&mut Vec::pop) {
            holder.wanted.store(true, Ordering::Relaxed);
            if let Ok(held) = holder.back.recv_timeout(TICK * 4) {
                return Some(held);
            }
        }
        None
    }

    /// Drop every held stream — a network change makes each of them a dead
    /// mapping. The holders see the flag, find no receiver, and hang up.
    pub(crate) fn clear(&self) {
        for holder in self.holders.with(&mut std::mem::take) {
            holder.wanted.store(true, Ordering::Relaxed);
        }
    }

    /// How many streams are being held. A holder that has hung up dropped
    /// its sender on the way out, so it is forgotten here rather than
    /// counted — the count is of live streams, not of threads ever started.
    pub(crate) fn len(&self) -> usize {
        self.holders.with(&mut |holders| {
            holders.retain(|h| matches!(h.back.try_recv(), Err(mpsc::TryRecvError::Empty)));
            holders.len()
        })
    }
}

/// A ladder that goes away takes its held streams with it — otherwise each
/// holder would keep its socket open until the silence bound, on a clock
/// that may never advance.
impl Drop for Pool {
    fn drop(&mut self) {
        self.clear();
    }
}

/// One held stream's life between asks, as the module doc spells it.
fn hold(
    mut held: Held,
    since: Instant,
    wanted: &AtomicBool,
    back: &mpsc::Sender<Held>,
    clock: &dyn Clock,
) {
    // A timeout that failed to arm costs a read that never wakes for a
    // taker; Some(nonzero) cannot be refused, so an arm here is untestable.
    let _ = held.tls.sock.set_read_timeout(Some(TICK));
    let mut last = since;
    loop {
        if wanted.load(Ordering::Relaxed) {
            let _ = back.send(held);
            return;
        }
        if clock.now().duration_since(last) >= SILENCE {
            return;
        }
        match read_frame(&mut held.tls) {
            Ok(Some(body)) if is_ping_body(&body) => last = clock.now(),
            Err(e) if timed_out(&e) => {}
            // The terminator, a frame that is not a ping where no reply is
            // due, or the socket failing: the engine is not speaking this
            // protocol on this stream any more, and it is dropped, not kept.
            _ => return,
        }
    }
}

/// One frame off a held stream, patient about the body: the header is read
/// at the tick, so a taker is never kept waiting, and once a header has
/// promised a body the body is waited for on the silence bound — the engine
/// writes the two in one breath, and a tick between them must not leave
/// four bytes consumed and the stream misframed. `frame::read_frame` is not
/// used here for exactly that reason and nowhere else.
fn read_frame(tls: &mut StreamOwned<ClientConnection, TcpStream>) -> io::Result<Option<Vec<u8>>> {
    use std::io::Read;
    let mut header = [0u8; 4];
    tls.read_exact(&mut header)?;
    let len = u32::from_be_bytes(header) as usize;
    if len == 0 {
        return Ok(None);
    }
    if len > frame::MAX_FRAME {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "oversize"));
    }
    let _ = tls.sock.set_read_timeout(Some(SILENCE));
    let mut body = vec![0u8; len];
    let read = tls.read_exact(&mut body);
    let _ = tls.sock.set_read_timeout(Some(TICK));
    read.map(|()| Some(body))
}

/// The frame a held connection carries through its silence.
pub(crate) fn is_ping(value: &Value) -> bool {
    value
        .as_object()
        .is_some_and(|o| o.len() == 1 && o.get("ping").and_then(Value::as_bool) == Some(true))
}

fn is_ping_body(body: &[u8]) -> bool {
    serde_json::from_slice::<Value>(body).is_ok_and(|v| is_ping(&v))
}

/// Did the wait run out, as opposed to the socket failing?
fn timed_out(e: &io::Error) -> bool {
    matches!(
        e.kind(),
        io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
    )
}
