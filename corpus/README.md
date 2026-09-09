# The wire conformance corpus

One canonical fixture set for the wire vocabulary, generated from yog's own
codec. Every client that speaks the wire replays it against its own encode and
decode, so an implementation miss fails a fixture instead of shipping.

REMOTE §3 is the protocol authority and §3.2 is the versioning rule; this file
states only how the corpus is laid out and consumed. Nothing here is authored
by hand — see `src/boundary/corpus.rs` for the generator and `make corpus` to
regenerate.

## Layout

    corpus/request/<op>.json     one file per request `op` token
    corpus/reply/<kind>.json     one file per reply `kind` (plus `refusal`)
    corpus/shapes.json           the standing record: per shape, every field
                                 path with the EDITION at which it appeared;
                                 the major, its floor, and what is deprecated

Each fixture file:

```json
{
  "direction": "request",
  "frames": [ { "op": "stop", "workspace": "ws", "agent": "c-1", "children": true } ],
  "protocol": 19,
  "shape": "stop"
}
```

`frames` are the wire frames themselves — byte for byte what a length-prefixed
frame carries, with no wrapper. `protocol` is the **major** the corpus is for,
the number the hello compares.

`shapes.json`:

```json
{
  "deprecated": [],
  "floor": 18,
  "protocol": 19,
  "shapes": {
    "reply/ops": { "signature": { "/kind:string": 1, "/rows/[]/client:string": 16, ":object": 1 } }
  }
}
```

A signature key is a field path with its JSON type; its value is the
**edition** at which that path first appeared. The corpus's edition is the
newest stamp in the file, computed, never stored. `floor` is the edition at
which the current major was cut: every path stamped at or below it is on every
engine of this major, every later path is optional to read. `deprecated`
lists shapes and fields (path without type) yog still writes and a reader
should stop relying on.

Keys are sorted and the file ends in a newline; regenerating on an unchanged
boundary is byte-identical. There is no timestamp, no counter and no address in
any fixture.

## The contract for a client

1. **Decode everything.** Every frame in `corpus/request/` must decode to the
   client's own gesture type; every frame in `corpus/reply/` to its own reply
   type. A frame the client cannot read is a miss, not an optional verb.
2. **Round-trip what you emit.** For every frame the client's own encoder can
   produce, decode then re-encode must return that frame exactly. A client that
   only ever *sends* requests still decodes them here — that is what catches a
   field it drops on the way out.
3. **Grow.** An unknown key is ignored; a key stamped above `floor` reads as
   its default when absent; an unknown word in a vocabulary maps to a named
   catch-all rendered as the unknown it is. Only the reply `kind` and the
   request `op` refuse by name (REMOTE §3.2). Two replays prove it:
   - **projection** — for every reply shape and every edition `e` from `floor`
     to the corpus's edition, delete every key whose stamp exceeds `e` from
     each frame, decode, and refuse nothing;
   - **word mutation** — for every string-typed path in a reply shape other
     than `kind`, replace the value in one frame with a token no build has
     heard of, decode, and refuse nothing.
4. **A shape you do not render is a parity fact, not a misread.** The corpus
   is the vocabulary; a fixture the client does not paint is recorded in its
   parity ledger with a reason, never filed as unreadable.

The frames carry only synthetic content: house workspace and conversation
names, `/ws`-style paths, fabricated ball ids. Nothing in this directory names a
real machine, path or account.

## Where it comes from

The yog repository is the source. A client vendors the directory, or reads it
from a checkout at build time; there is no published artifact and no endpoint
that serves it. The engine states the corpus's edition in its hello, so a
client that vendored this record can tell which fields the engine it dialled
spells. The consumers today are the seat and the android app; the foot's
surface is small enough that it may consume the subset of shapes it actually
speaks, under rule 4 above.

## Regenerating

    make corpus

That rewrites every file from the boundary. A field gained is stamped the next
edition and needs no bump. It **refuses** a field or shape that vanished, and a
key that started spelling a second JSON type, unless the repo-root `PROTOCOL`
file has been raised — a major bump — and, for a loss, the path is listed in
`corpus::DEPRECATED` (`src/boundary/corpus.rs`).
