#!/usr/bin/env python3
"""**The bridge gate's own regression half** (bl-05b6), in `leak-scan
--self-test`'s shape and for its reason: a gate dies by matching nothing, and
one nobody can see fail is indistinguishable from one that works.

It imports the gate itself rather than restating any of it, so every arm below
runs the functions `make apk` runs. Six of them, and each is a way the check
could be wrong rather than a way the tree could be:

* the clean pair passes;
* a name the crate resolves that the dex does not carry is caught;
* a public static of a bridged class that no site resolves is caught;
* a method that is not PUBLIC STATIC does not answer for one that is;
* a source tree that yields no pin at all fails rather than passing empty;
* a second door naming no class fails, because that is the shape a new bridge
  the extractor cannot read would arrive in.
"""

import importlib.util
import os
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
SPEC = importlib.util.spec_from_file_location("bridges", os.path.join(HERE, "apk-bridges.py"))
GATE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(GATE)

# One source file per shape the extractor knows: the door with its argument
# count, a literal signature, and a helper that carries one for its callers.
DOOR = '''
use crate::tools::bridged::Door;
static PAPER: Door = Door::new("dev.yog.Fixture", "fixture bridge");
pub(crate) fn a() -> String { PAPER.strings("plain", &[]) }
pub(crate) fn b(t: &str) -> String { PAPER.strings("oneArg", &[t]) }
'''

BRIDGE = '''
const CLASS: &str = "dev.yog.Other";
const TAKES_ACTIVITY: &str = "(Landroid/app/Activity;)Ljava/lang/String;";
fn open(env: &mut jni::JNIEnv) { Bridge::open(env, CLASS, LABEL) }
fn stop() { bridge.string(&mut env, "stop", "()Ljava/lang/String;", &[]) }
fn with_activity(app: &A, method: &str) -> String {
    bridge.string(&mut env, method, TAKES_ACTIVITY, &[JValue::Object(&object)])
}
pub(super) fn look(app: &A) -> C { state(&with_activity(app, "state")) }
'''

# A generic door: it opens one and names no class, because its class is a
# parameter. Exactly one of these may exist.
GENERIC = '''
fn bridge(&self, env: &mut jni::JNIEnv) -> Result<Bridge, String> {
    Bridge::open(env, self.class, self.label)
}
'''


def dump(methods):
    """A `dexdump -l plain` extract carrying exactly these methods."""
    out = []
    for klass, name, type_, access in methods:
        out.append(f"  Class descriptor  : 'L{klass.replace('.', '/')};'")
        out.append("  Direct methods    -")
        out.append(f"    #0              : (in L{klass.replace('.', '/')};)")
        out.append(f"      name          : '{name}'")
        out.append(f"      type          : '{type_}'")
        out.append(f"      access        : 0x0009 ({access})")
    return "\n".join(out)


STRING = GATE.STRING
WHOLE = [
    ("dev.yog.Fixture", "plain", f"(){STRING}", "PUBLIC STATIC"),
    ("dev.yog.Fixture", "oneArg", f"({STRING}){STRING}", "PUBLIC STATIC"),
    ("dev.yog.Other", "stop", f"(){STRING}", "PUBLIC STATIC"),
    ("dev.yog.Other", "state", f"(Landroid/app/Activity;){STRING}", "PUBLIC STATIC"),
]


def tree(files):
    """A throwaway source tree of the named fixture files."""
    at = tempfile.mkdtemp(prefix="yog-bridges-")
    for name, body in files.items():
        with open(os.path.join(at, name), "w", encoding="utf-8") as handle:
            handle.write(body)
    return at


def verdict(files, methods):
    pins, classes, said = GATE.pins_in(tree(files))
    return GATE.judge(pins, classes, GATE.dex_statics(dump(methods)), said)


def beat(name, said, wanted):
    if wanted is None:
        ok, why = not said, f"expected nothing, said: {said}"
    else:
        ok = any(wanted in line for line in said)
        why = f"expected {wanted!r}, said: {said}"
    print(f"  {'pass' if ok else 'FAIL'}  {name}")
    return None if ok else why


def main():
    whole = {"door.rs": DOOR, "bridge.rs": BRIDGE, "generic.rs": GENERIC}
    failed = [
        beat("the clean pair passes", verdict(whole, WHOLE), None),
        beat(
            "a name the dex does not carry is caught",
            verdict(whole, [m for m in WHOLE if m[1] != "oneArg"]),
            "dev.yog.Fixture.oneArg",
        ),
        beat(
            "an entry point nothing resolves is caught",
            verdict(whole, WHOLE + [("dev.yog.Fixture", "orphan", f"(){STRING}", "PUBLIC STATIC")]),
            "no site in this crate resolves",
        ),
        beat(
            "a private static answers for nothing",
            verdict(whole, [m for m in WHOLE if m[1] != "plain"]
                    + [("dev.yog.Fixture", "plain", f"(){STRING}", "PRIVATE STATIC")]),
            "dev.yog.Fixture.plain",
        ),
        beat(
            "a tree with no pin at all fails",
            verdict({"nothing.rs": "fn main() {}\n"}, WHOLE),
            "the scan is broken, not the tree",
        ),
        beat(
            "a second door naming no class fails",
            verdict(dict(whole, **{"second.rs": GENERIC}), WHOLE),
            "more than one door names no class",
        ),
    ]
    failed = [why for why in failed if why]
    for why in failed:
        print(f"bridges: self-test: {why}", file=sys.stderr)
    if not failed:
        print("bridges: self-test OK — both directions live, extraction guards live")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
