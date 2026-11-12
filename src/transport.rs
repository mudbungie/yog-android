//! The seat's transport — the mirror of the server-side client
//! (`wire/client.rs`; yog REMOTE §8, §9.5): what a client of the server
//! holds, and the only thing it holds.
//!
//! A client owns its key material and RAM, nothing else (REMOTE §6) — so
//! this is a configuration and an address, and **every ask is its own TCP
//! connection and its own handshake**: "the seat polls", at human cadence.
//! A follow-class read is the same connection kept open for as long as the
//! server holds it (REMOTE §3: the streaming form is not a second form):
//! [`Seat::hold`] is the one door every ask goes through, and [`Seat::ask`]
//! is that door with every frame collected — so a held lane and a one-shot
//! answer differ only in who reads the frames and when.
//!
//! **The server's name comes from the address, never from a second knob.** A
//! dotted quad or a bracketed v6 literal is verified as an IP address — the
//! server leaf must carry the matching `IP:` SAN — and anything else is a
//! DNS name. Nothing to configure, nothing that can disagree with what was
//! dialled.
//!
//! **The address list is dialled in order** (yog REMOTE §15.3; DESIGN §21).
//! The endpoint that worked earlier in this run is tried first
//! ([`crate::state::worked`], RAM only), then every other in the order the
//! file states. An endpoint is passed over when its socket will not open *or*
//! its handshake will not verify — a private address dialled from somebody
//! else's network may well be answered, and whatever answers there cannot
//! present the operator's CA — and nothing has crossed in either case, so
//! falling through repeats no gesture.

mod open;
mod wire;

pub use open::{Hangup, Open};
pub use wire::Wire;

use crate::codec::reply::{self, Reply};
use crate::material::Material;
use crate::{frame, hello};
use rustls::pki_types::ServerName;
use rustls::{ClientConnection, StreamOwned};
use serde_json::Value;
use std::net::{IpAddr, TcpStream, ToSocketAddrs};
use std::sync::Arc;
use std::time::Duration;

/// How long a seat waits on one answer before giving up on the connection.
const ASK_TIMEOUT: Duration = Duration::from_mins(2);

/// How long a fresh socket is given to carry the handshake and the server's
/// preface — one frame, which a server that accepted the stream writes at
/// once. Short, because a stream can connect and never be served, and a whole
/// [`ASK_TIMEOUT`] there would cost the next endpoint in the list two minutes.
const PREFACE: Duration = Duration::from_secs(10);

/// How long one resolved address is given to answer a SYN. Bounded because a
/// network that drops the SYN would otherwise cost minutes before the next
/// endpoint in the list is tried.
const CONNECT: Duration = Duration::from_secs(5);

/// **What a stream that carried no frame at all says.** One home, because two
/// readers ask the same question of one shape: `Seat::answered` here, and
/// `seat::pass::ask::wired`, which wants the envelope beside the reply and so
/// cannot go through it (bl-eec1).
pub(crate) const NO_ANSWER: &str = "the server ended the stream without answering";

/// One endpoint of the list: what is dialled, and the name the server's
/// certificate is verified against there.
struct Endpoint {
    address: String,
    name: ServerName<'static>,
}

/// A seat's end of the wire.
pub struct Seat {
    config: Arc<rustls::ClientConfig>,
    endpoints: Vec<Endpoint>,
    /// What this material's worked endpoint is kept under: its leaf's path,
    /// which is one per identity on this device.
    key: String,
}

impl Seat {
    /// Build the seat from provisioned material. Nothing is dialled here: a
    /// seat is a fact about what this device may say, not about whether a
    /// server happens to be up.
    pub fn open(m: &Material) -> Result<Self, String> {
        let endpoints = m
            .addresses
            .iter()
            .map(|address| {
                Ok(Endpoint {
                    address: address.clone(),
                    name: server_name(address)?,
                })
            })
            .collect::<Result<_, String>>()?;
        Ok(Self {
            config: crate::tls::client_config(m)?,
            endpoints,
            key: m.chain.display().to_string(),
        })
    }

    /// What this seat dials: the whole list, in order.
    pub fn address(&self) -> String {
        let listed: Vec<&str> = self
            .endpoints
            .iter()
            .map(|at| at.address.as_str())
            .collect();
        listed.join(", ")
    }

    /// Ask once and decode the answer — the last frame of the stream, which
    /// today is the only frame. One `Err` for a refusal, an unreadable answer
    /// and a socket that never opened alike: the same fact to a caller that
    /// only paints it, carrying [`Wire`]'s class for the one that redials.
    ///
    /// **`last()` is not the door to the follow lane** (REMOTE §5.5, bl-2842).
    /// Since yog bl-3655 a `Query::Follow` frame carries what landed *since
    /// the previous frame*, not the whole answer, and the rule is one line
    /// with no flag: *"Absorb every frame of a read, in order, onto an empty
    /// fold. What you hold after the last frame you have received is what you
    /// paint."* Taking the last frame of such a stream paints the final delta
    /// alone and calls it the answer. Nothing here follows — this seat
    /// re-reads the transcript at the model's cadence — so no test can catch
    /// it, which is exactly why the trap is named at the `last()` rather than
    /// in a document the author will not be reading. The lane's door is
    /// [`ask`](Self::ask), which hands back every frame; the fold goes on top
    /// of it.
    pub fn answered(&self, request: &Value) -> Result<Reply, Wire> {
        let stream = self.ask(request)?;
        let last = stream
            .last()
            .ok_or_else(|| Wire::Unusable(NO_ANSWER.to_owned()))?;
        // **The decoder already draws this line and it was being collapsed**
        // (bl-8bd0): its OUTER error is a reply this end cannot read, its
        // INNER one is the server's own `ok: false` sentence. They are two
        // different facts about who failed, and the redial matrix needs them
        // apart — a refusal on the follow read is worth another dial and an
        // unreadable answer never is.
        match reply::decode(last) {
            Err(unreadable) => Err(Wire::Unusable(unreadable)),
            Ok(Err(refusal)) => Err(Wire::Refused(refusal)),
            Ok(Ok(reply)) => Ok(reply),
        }
    }

    /// Send one request envelope and read its whole reply stream — every
    /// frame up to the terminator. A stream of one is the ordinary answer.
    pub fn ask(&self, request: &Value) -> Result<Vec<Value>, Wire> {
        let mut stream = Vec::new();
        let (open, _hangup) = self.read(request)?;
        open.each(&mut |frame| {
            stream.push(frame);
            true
        })?;
        Ok(stream)
    }

    /// **Open one held read** (DESIGN §14.1): the request written and the
    /// server's preface confirmed, with no frame read yet — and the handle
    /// that ends it from another thread. The caller reads the frames on
    /// whatever thread it likes and hangs up from any other, which is the
    /// whole of what a lane needs and one-shot asks do not.
    pub fn hold(&self, request: &Value) -> Result<(Open, Hangup), Wire> {
        self.read(request)
    }

    /// Open a read: the request written down the first endpoint that would
    /// carry it, and the server's half of the §3 preface confirmed on the way
    /// to the answer — a skew refuses here, before a frame of another
    /// protocol is decoded.
    fn read(&self, request: &Value) -> Result<(Open, Hangup), Wire> {
        let (mut tls, hangup) = self.dial(request)?;
        let edition = hello::confirm(&mut tls).map_err(Wire::Unusable)?;
        // The preface is in, under its own bound: the answer gets the ask's.
        let _ = tls.sock.set_read_timeout(Some(ASK_TIMEOUT));
        Ok((Open::new(tls, edition), Hangup::new(hangup)))
    }

    /// The list, dialled: the endpoint that worked first, then the rest in
    /// the file's order. **Every endpoint that refused is named**, each in
    /// its own sentence, so a list that reached nothing says why of every
    /// line in it.
    fn dial(&self, request: &Value) -> Result<Dialled, Wire> {
        let worked = crate::state::worked(&self.key);
        let mut order: Vec<&Endpoint> = self.endpoints.iter().collect();
        order.sort_by_key(|at| Some(&at.address) != worked.as_ref());
        let mut refusals = Vec::new();
        for at in order {
            match self.send(at, request) {
                Ok(dialled) => {
                    crate::state::works(&self.key, &at.address);
                    return Ok(dialled);
                }
                Err(refusal) => refusals.push(refusal),
            }
        }
        Err(Wire::Transport(refusals.join("; ")))
    }

    /// One endpoint: the socket, the mTLS over it verifying the name read off
    /// that same endpoint, and this end's preface and request down it — with
    /// what the hang-up handle is made from, a second descriptor on the same
    /// socket, taken here so that a clone that cannot be had is the same
    /// sentence as a socket that would not open.
    ///
    /// **A write that failed is not in doubt, and the framing is why**
    /// (bl-07b1). The handshake happens inside the first write, so an error
    /// here is the socket, the two ends failing to accept each other's
    /// certificates, or bytes not accepted — and `frame`'s reader takes a
    /// length header and then exactly that many bytes, never scanning, so an
    /// incomplete frame is not decoded, never mind executed. The doubt
    /// begins where the write ENDS, which is why every refusal here falls
    /// through to the next endpoint.
    fn send(&self, at: &Endpoint, request: &Value) -> Result<Dialled, String> {
        let connect = |e| format!("connect {}: {e}", at.address);
        let tcp = connect_bounded(&at.address).map_err(connect)?;
        let hangup = tcp.try_clone().map_err(connect)?;
        // A timeout that failed to arm costs a slow failure, never a wrong
        // one — and Some(nonzero) cannot be refused, so an error arm here
        // would be an untestable branch.
        let _ = tcp.set_read_timeout(Some(PREFACE));
        let conn = ClientConnection::new(Arc::clone(&self.config), at.name.clone())
            .map_err(|e| format!("tls {}: {e}", at.address))?;
        let mut tls = StreamOwned::new(conn, tcp);
        hello::state(&mut tls)
            .and_then(|()| frame::write_frame(&mut tls, request.to_string().as_bytes()))
            .map_err(|e| format!("send {}: {e}", at.address))?;
        Ok((tls, hangup))
    }
}

/// What [`Seat::dial`] hands up: the stream with the request on it, and the
/// hang-up handle's descriptor.
type Dialled = (StreamOwned<ClientConnection, TcpStream>, TcpStream);

/// A connect that gives each address `endpoint` resolves to at most
/// [`CONNECT`], and answers the last refusal where none of them took it.
fn connect_bounded(endpoint: &str) -> std::io::Result<TcpStream> {
    let mut refusal = std::io::Error::other("resolved to no address");
    for addr in endpoint.to_socket_addrs()? {
        match TcpStream::connect_timeout(&addr, CONNECT) {
            Ok(tcp) => return Ok(tcp),
            Err(e) => refusal = e,
        }
    }
    Err(refusal)
}

/// The name to verify the server certificate against, read off the address.
fn server_name(address: &str) -> Result<ServerName<'static>, String> {
    let host = address.rsplit_once(':').map_or(address, |(head, _)| head);
    let host = host.trim_start_matches('[').trim_end_matches(']');
    if let Ok(ip) = host.parse::<IpAddr>() {
        return Ok(ServerName::IpAddress(ip.into()));
    }
    ServerName::try_from(host.to_owned()).map_err(|e| format!("{address}: not a server name: {e}"))
}

#[cfg(test)]
mod tests;
