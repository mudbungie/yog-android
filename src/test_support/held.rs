//! **The engine that HOLDS a connection** (yog REMOTE §13.4), for the suite:
//! one accepted stream, the preface exchanged once, then a script of beats —
//! requests answered in order, pings written into the silence, a gate the
//! test opens when it is ready — and after the last beat the stream is held
//! until this end reads EOF. What the ladder's rungs and the held pool are
//! measured against.

use rustls::{ServerConnection, StreamOwned};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::sync::Arc;
use std::sync::mpsc::Receiver;
use std::thread::JoinHandle;

/// One step of a held connection's script.
pub enum Beat {
    /// Read one request, answer with these frames and the terminator.
    Answer(Vec<Vec<u8>>),
    /// Write a ping frame, unprompted.
    Ping,
    /// Write this frame, unprompted — what a held stream must NOT carry.
    Say(Vec<u8>),
    /// Write these bytes raw, unframed.
    Raw(Vec<u8>),
    /// Wait until the test says so (or drops the sender).
    Gate(Receiver<()>),
    /// Hang up now.
    Hangup,
}

/// Serve one held connection per script, in order, on loopback; the handle
/// carries every request read, per connection.
pub fn serve_held(
    dir: &Path,
    ca: &str,
    leaf: &str,
    scripts: Vec<Vec<Beat>>,
) -> (String, JoinHandle<Vec<Vec<Vec<u8>>>>) {
    let config = super::serve::config(dir, ca, leaf);
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = format!("127.0.0.1:{}", listener.local_addr().unwrap().port());
    let handle = std::thread::spawn(move || {
        let mut all = Vec::new();
        for beats in scripts {
            let (tcp, _) = listener.accept().unwrap();
            let conn = ServerConnection::new(Arc::clone(&config)).unwrap();
            let mut tls = StreamOwned::new(conn, tcp);
            let preface = format!("{{\"protocol\":{}}}", crate::hello::PROTOCOL);
            crate::frame::write_frame(&mut tls, preface.as_bytes()).unwrap();
            let stated = crate::frame::read_frame(&mut tls).unwrap().unwrap();
            let stated: serde_json::Value = serde_json::from_slice(&stated).unwrap();
            assert_eq!(
                stated["protocol"],
                crate::hello::PROTOCOL,
                "the seat stated its version"
            );
            let mut requests = Vec::new();
            let mut hung_up = false;
            for beat in beats {
                if !play(&mut tls, beat, &mut requests) {
                    hung_up = true;
                    break;
                }
            }
            if !hung_up {
                while crate::frame::read_frame(&mut tls).is_ok_and(|frame| frame.is_some()) {}
            }
            all.push(requests);
        }
        all
    });
    (address, handle)
}

/// One beat; `false` is the hang-up.
fn play(
    tls: &mut StreamOwned<ServerConnection, TcpStream>,
    beat: Beat,
    requests: &mut Vec<Vec<u8>>,
) -> bool {
    match beat {
        Beat::Answer(frames) => {
            let Ok(Some(request)) = crate::frame::read_frame(tls) else {
                return false;
            };
            requests.push(request);
            for frame in &frames {
                crate::frame::write_frame(tls, frame).unwrap();
            }
            crate::frame::write_end(tls).unwrap();
        }
        Beat::Ping => {
            let ping = serde_json::json!({ "ping": true }).to_string().into_bytes();
            crate::frame::write_frame(tls, &ping).unwrap();
        }
        Beat::Say(frame) => crate::frame::write_frame(tls, &frame).unwrap(),
        Beat::Raw(bytes) => std::io::Write::write_all(tls, &bytes).unwrap(),
        Beat::Gate(open) => {
            let _ = open.recv();
        }
        Beat::Hangup => return false,
    }
    true
}
