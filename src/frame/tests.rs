//! The framing's contract, exercised from both directions: what one side
//! writes the other reads back byte-for-byte, and every refusal fires on the
//! input that earns it.

use super::{MAX_FRAME, read_frame, write_end, write_frame};
use std::io::Cursor;

#[test]
fn roundtrip_one_frame() {
    let mut wire = Vec::new();
    write_frame(&mut wire, b"{\"ask\":1}").unwrap();
    let mut r = Cursor::new(wire);
    assert_eq!(read_frame(&mut r).unwrap(), Some(b"{\"ask\":1}".to_vec()));
}

#[test]
fn roundtrip_a_reply_stream() {
    // An answer is N >= 1 frames then the terminator (REMOTE §3); the reader
    // sees each frame, then None, in order.
    let mut wire = Vec::new();
    write_frame(&mut wire, b"one").unwrap();
    write_frame(&mut wire, b"two").unwrap();
    write_end(&mut wire).unwrap();
    let mut r = Cursor::new(wire);
    assert_eq!(read_frame(&mut r).unwrap(), Some(b"one".to_vec()));
    assert_eq!(read_frame(&mut r).unwrap(), Some(b"two".to_vec()));
    assert_eq!(read_frame(&mut r).unwrap(), None);
}

#[test]
fn the_terminator_is_four_zero_bytes() {
    // The wire shape itself, not just the API: interop with the server
    // depends on these exact bytes.
    let mut wire = Vec::new();
    write_end(&mut wire).unwrap();
    assert_eq!(wire, vec![0, 0, 0, 0]);
}

#[test]
fn the_header_is_big_endian() {
    let mut wire = Vec::new();
    write_frame(&mut wire, &[0xAB; 258]).unwrap();
    assert_eq!(&wire[..4], &[0, 0, 1, 2]);
}

#[test]
fn write_refuses_an_oversized_body() {
    let body = vec![0u8; MAX_FRAME + 1];
    let mut wire = Vec::new();
    let err = write_frame(&mut wire, &body).unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    assert!(wire.is_empty(), "a refused frame must write nothing");
}

#[test]
fn read_refuses_an_oversized_length_on_its_header() {
    // The refusal happens before any allocation: the four header bytes are
    // the whole of what the reader consumes.
    let wire = u32::MAX.to_be_bytes().to_vec();
    let mut r = Cursor::new(wire);
    let err = read_frame(&mut r).unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    assert!(err.to_string().contains("exceeds"), "{err}");
}

#[test]
fn read_errors_on_a_truncated_header() {
    let mut r = Cursor::new(vec![0u8, 0]);
    let err = read_frame(&mut r).unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::UnexpectedEof);
}

#[test]
fn read_errors_on_a_truncated_body() {
    // A header promising more bytes than the stream holds is a short stream,
    // never a short frame.
    let mut wire = Vec::new();
    write_frame(&mut wire, b"whole").unwrap();
    wire.truncate(wire.len() - 2);
    let mut r = Cursor::new(wire);
    let err = read_frame(&mut r).unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::UnexpectedEof);
}

#[test]
fn a_frame_at_the_bound_is_lawful() {
    // MAX_FRAME itself passes; only above it refuses. Written and read back,
    // so the bound means the same thing on both sides.
    let body = vec![7u8; MAX_FRAME];
    let mut wire = Vec::new();
    write_frame(&mut wire, &body).unwrap();
    let mut r = Cursor::new(wire);
    assert_eq!(read_frame(&mut r).unwrap(), Some(body));
}
