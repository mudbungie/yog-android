# The wire conformance corpus

One canonical fixture set for the wire vocabulary, generated from yog's own
codec. Every client that speaks the wire replays it against its own encode and
decode, so an implementation miss fails a fixture instead of shipping.

REMOTE §3 is the protocol authority; this file states only how the corpus is
laid out and consumed. Nothing here is authored by hand — see
`src/boundary/corpus.rs` for the generator and `make corpus` to regenerate.

## Layout

    corpus/request/<op>.json     one file per request `op` token
    corpus/reply/<kind>.json     one file per reply `kind` (plus `refusal`)
    corpus/shapes.json           the standing record: per shape, its field
                                 signature and the protocol version at which
                                 that signature last moved

Each fixture file:

```json
{
  "direction": "request",
  "frames": [ { "op": "stop", "workspace": "ws", "agent": "c-1", "children": true } ],
  "protocol": 1,
  "shape": "stop"
}
```

`frames` are the wire frames themselves — byte for byte what a length-prefixed
frame carries, with no wrapper. `protocol` is **this shape's**: the version at
which its fields last changed, which is not always the version the corpus as a
whole is for. `shapes.json` carries that one, at its top level.

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
3. **A shape you do not implement is still a shape you must not misread.** The
   corpus is not a feature list; it is the vocabulary. Skipping a fixture is a
   decision to be recorded in the client, never a silent pass.

The frames carry only synthetic content: house workspace and conversation
names, `/ws`-style paths, fabricated ball ids. Nothing in this directory names a
real machine, path or account.

## Where it comes from

The yog repository is the source. A client vendors the directory, or reads it
from a checkout at build time; there is no published artifact and no endpoint
that serves it. The consumers today are the seat and the android app; the
foot's surface is small enough that it may consume the subset of shapes it
actually speaks, under rule 3 above.

## Regenerating

    make corpus

That rewrites every file from the boundary. It **refuses** when a shape already
in use changed its fields while the protocol version stood still — the change
is lawful only after `PROTOCOL` in `src/wire/hello.rs` is raised. A new shape
needs no bump: strict decode already refuses an unknown verb in band.
