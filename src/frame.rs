//! The wire's framing, client side (yog REMOTE §3, decided by yog bl-b6fa):
//! **a big-endian `u32` byte length followed by that many bytes, and a
//! zero-length frame ends a reply stream.**
//!
//! This mirrors the server's `src/wire/frame.rs` and must keep mirroring it —
//! the framing is the one layer both ends must agree on byte-for-byte. The
//! properties it keeps, same as there:
//!
//! - A reader never scans. It reads four bytes, then exactly that many — so a
//!   payload's own bytes can never be mistaken for a delimiter, and no
//!   property of the *encoder* is load-bearing in the *framing*.
//! - The allocation is bounded before it is made. A length above
//!   [`MAX_FRAME`] is refused on its header, so a hostile peer cannot make a
//!   reader grow to meet it.
//! - The terminator is unambiguous by construction: a zero-length frame is
//!   not a JSON value, so nothing a payload can say collides with it.
//!
//! This layer carries BYTES, deliberately: the JSON codec (strict decode, the
//! discipline the server's boundary codec keeps) is its own module and its
//! own ball, and the framing must not presume it. A request is one frame; an
//! answer is N ≥ 1 reply frames followed by the terminator.

use std::io::{self, Read, Write};

/// The largest frame either end will write or read: 16 MiB — the same bound
/// as the server's `MAX_FRAME`, because a limit only bounds a conversation if
/// both ends hold it.
pub const MAX_FRAME: usize = 16 * 1024 * 1024;

/// The frame header's width — a big-endian `u32`.
const HEADER: usize = 4;

/// Write one frame: the length header, then the body. A zero-length body is
/// the stream terminator — [`write_end`] is that spelling, named.
pub fn write_frame(w: &mut dyn Write, body: &[u8]) -> io::Result<()> {
    if body.len() > MAX_FRAME {
        return Err(oversize(body.len()));
    }
    // Infallible after the bound above, and deliberately not a `try_from`: a
    // fallible conversion whose error arm cannot be reached is an untestable
    // branch, and this file has no untestable branches.
    let len = body.len() as u32;
    w.write_all(&len.to_be_bytes())?;
    w.write_all(body)?;
    w.flush()
}

/// Write the end-of-stream terminator: a zero-length frame.
pub fn write_end(w: &mut dyn Write) -> io::Result<()> {
    write_frame(w, &[])
}

/// Read one frame: `Some(body)` a frame, `None` the terminator. An oversized
/// length or a short stream is an error.
pub fn read_frame(r: &mut dyn Read) -> io::Result<Option<Vec<u8>>> {
    let mut header = [0u8; HEADER];
    r.read_exact(&mut header)?;
    let len = u32::from_be_bytes(header) as usize;
    if len == 0 {
        return Ok(None);
    }
    if len > MAX_FRAME {
        return Err(oversize(len));
    }
    let mut body = vec![0u8; len];
    r.read_exact(&mut body)?;
    Ok(Some(body))
}

/// The one refusal a length can earn, said the same way on both sides.
fn oversize(len: usize) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        format!("frame of {len} bytes exceeds the {MAX_FRAME}-byte limit"),
    )
}

#[cfg(test)]
mod tests;
