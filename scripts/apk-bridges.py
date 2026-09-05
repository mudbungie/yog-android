#!/usr/bin/env python3
"""**The JNI names, pinned against the dex that has to carry them** (bl-05b6).

Every call this crate makes into its own Java shell is resolved BY NAME at
runtime: a class through the app's class loader, then a static method by name
and descriptor (`src/tools/bridged/door.rs`, `src/shell/jvm.rs`). Nothing in a
build checks any of it. Rename `Paper.notify` in the Java and the Rust still
compiles, the APK still assembles, the app still launches — and the first
invocation of that tool answers a `NoSuchMethodError` on a device, which is
the one place this repo cannot look.

So `make apk` asks the artifact it just built. **Both directions, because the
two failures are different**: a name the crate resolves and the dex does not
carry is a tool that refuses forever, and a public static entry point in a
pinned class that no Rust site names is a door nobody comes in by — dead
weight at best, and usually half of a rename somebody stopped in the middle.

**It reads EVERY `classes*.dex` in the APK.** The shell's own classes are not
in `classes.dex` today: the multidex split put them in `classes4.dex`, and
which dex a class lands in is R8's business and moves between builds. A scan
of the first dex would have found nothing at all and passed — which is why an
empty enumeration is a failure here, the same rule `make line-cap` keeps.

The extraction knows the two doors this crate has (`Door::new` and
`Bridge::open`) and the four shapes a name reaches them by; a class discovered
with no method pinned is refused rather than skipped, and the reverse
direction is what catches a method shape the extractor cannot read.
"""

import os
import re
import shutil
import subprocess
import sys
import tempfile
import zipfile

STRING = "Ljava/lang/String;"

# Every source shape that names a method, after the file is collapsed to one
# line of single-spaced text. Each yields (method, descriptor) for the file's
# own class.
CALL_STRINGS = re.compile(r'\w+\.strings\(\s*"(\w+)"\s*,\s*&\[([^\]]*)\]')
CALL_LITERAL = re.compile(r'(?:\bcall|\.string|\.bytes)\(\s*(?:&mut env,\s*)?"(\w+)"\s*,\s*"([^"]+)"')
HELPER_DEF = re.compile(
    r'fn (\w+)\((?:(?!\bfn ).)*?\.(?:string|bytes)\(\s*&mut env,\s*method,\s*("[^"]+"|[A-Z_]+)\s*,'
)
CONST_DEF = re.compile(r'const ([A-Z_]+): &str = "([^"]+)"')
CLASS_DOOR = re.compile(r'Door::new\(\s*"([\w.]+)"')
CLASS_CONST = re.compile(r'const CLASS: &str = "([\w.]+)"')
OPEN_DOOR = re.compile(r'Bridge::open\(')


def collapsed(text):
    """One line, single-spaced — so a call rustfmt split over four lines reads
    the same as one written on one, and every pattern above is written once."""
    return re.sub(r"\s+", " ", text)


def pins_of(text):
    """The (class, method, descriptor) triples one source file resolves, and
    whether the file opens a door at all."""
    flat = collapsed(text)
    classes = CLASS_DOOR.findall(flat) + CLASS_CONST.findall(flat)
    opens = bool(CLASS_DOOR.search(flat) or OPEN_DOOR.search(flat))
    if not classes:
        return [], opens
    named = dict(CONST_DEF.findall(flat))
    found = []
    for method, args in CALL_STRINGS.findall(flat):
        count = len([a for a in args.split(",") if a.strip()])
        found.append((method, f"({STRING * count}){STRING}"))
    found += CALL_LITERAL.findall(flat)
    for helper, signature in HELPER_DEF.findall(flat):
        signature = named.get(signature, signature.strip('"'))
        for method in re.findall(rf'{helper}\([^()]*?"(\w+)"\)', flat):
            found.append((method, signature))
    return [(classes[0], method, sig) for method, sig in found], opens


def pins_in(root):
    """Every pin this tree states, plus what the extraction itself could not
    account for — **the half that keeps a silently-broken pattern from
    passing as green**. A file that opens a door and names a class owes at
    least one pin; a file that opens one and names no class is the generic
    door (`Door`, whose class is a parameter), and there may be exactly one of
    those."""
    pins, classes, generic, said = set(), set(), [], []
    for here, _, files in os.walk(root):
        for name in sorted(files):
            if not name.endswith(".rs"):
                continue
            path = os.path.join(here, name)
            with open(path, encoding="utf-8") as handle:
                found, opens = pins_of(handle.read())
            if not opens:
                continue
            if not found:
                generic.append(os.path.relpath(path, root))
                continue
            for pin in found:
                classes.add(pin[0])
                pins.add(pin)
    if len(generic) > 1:
        said.append("more than one door names no class: " + ", ".join(sorted(generic))
                    + " — one of them is a shape this extractor cannot read")
    return pins, classes, said


# `dexdump -l plain` states a method as four lines under its class.
DEX_CLASS = re.compile(r"^\s*Class descriptor\s*:\s*'L([\w/$]+);'")
DEX_NAME = re.compile(r"^\s*name\s*:\s*'([^']*)'")
DEX_TYPE = re.compile(r"^\s*type\s*:\s*'([^']*)'")
DEX_ACCESS = re.compile(r"^\s*access\s*:\s*0x[0-9a-f]+ \(([^)]*)\)")


def dex_statics(dump):
    """Every PUBLIC STATIC method the dump declares, as (class, name, type)."""
    found, here, name, type_ = set(), None, None, None
    for line in dump.splitlines():
        klass = DEX_CLASS.match(line)
        if klass:
            here, name, type_ = klass.group(1).replace("/", "."), None, None
            continue
        named = DEX_NAME.match(line)
        if named:
            name, type_ = named.group(1), None
            continue
        typed = DEX_TYPE.match(line)
        if typed:
            type_ = typed.group(1)
            continue
        access = DEX_ACCESS.match(line)
        if access and here and name and type_ and access.group(1) == "PUBLIC STATIC":
            found.add((here, name, type_))
    return found


def dumped(apk, dexdump):
    """Every `classes*.dex` in the APK, dumped. An APK with none is refused:
    a scan that enumerates nothing must fail rather than pass."""
    out, at = [], tempfile.mkdtemp(prefix="yog-bridges-")
    with zipfile.ZipFile(apk) as archive:
        names = [n for n in archive.namelist() if re.fullmatch(r"classes\d*\.dex", n)]
        for name in names:
            path = os.path.join(at, name)
            with open(path, "wb") as handle:
                handle.write(archive.read(name))
            # `errors="replace"`: a dex carries UTF-8-ish MUTF-8 in its
            # string pool and some of it is not valid UTF-8 at all. A byte
            # this reader cannot decode is in a constant, never in a class
            # descriptor or a method signature — the two things read here.
            out.append(subprocess.run(
                [dexdump, "-l", "plain", path],
                check=True, capture_output=True, text=True, errors="replace",
            ).stdout)
    shutil.rmtree(at, ignore_errors=True)
    if not names:
        raise SystemExit(f"bridges: {apk} carries no classes*.dex — the scan is broken")
    return "\n".join(out), len(names)


def judge(pins, classes, statics, said):
    """The two directions, after the extraction's own complaints."""
    said = list(said)
    if not pins:
        said.append("no JNI pin was extracted at all — the scan is broken, not the tree")
    for klass, method, signature in sorted(pins):
        if (klass, method, signature) not in statics:
            said.append(f"{klass}.{method}{signature} is resolved by name and the dex "
                        "carries no public static of that descriptor")
    for klass, method, signature in sorted(statics):
        if klass in classes and (klass, method, signature) not in pins:
            said.append(f"{klass}.{method}{signature} is a public static of a bridged class "
                        "that no site in this crate resolves")
    return said


def tool():
    """The `dexdump` this box has. It ships with the build tools, so a box that
    can assemble an APK has one; the newest is taken because a dex format the
    older one predates would read as a class it could not see."""
    named = os.environ.get("DEXDUMP")
    if named:
        return named
    sdk = os.environ.get("ANDROID_HOME") or os.environ.get("ANDROID_SDK_ROOT") \
        or os.path.join(os.path.expanduser("~"), "Android", "Sdk")
    root = os.path.join(sdk, "build-tools")
    found = sorted(
        (os.path.join(root, v, "dexdump") for v in os.listdir(root)),
        key=lambda at: [int(n) for n in re.findall(r"\d+", os.path.basename(os.path.dirname(at)))],
    ) if os.path.isdir(root) else []
    if not found:
        raise SystemExit(f"bridges: no dexdump under {root} — install the Android build tools")
    return found[-1]


def main(argv):
    """`--self-test` runs the harness beside this file, over the same
    functions the gate spends — both directions, like every other gate here.
    A path runs the real thing."""
    if "--self-test" in argv:
        here = os.path.dirname(os.path.abspath(__file__))
        return subprocess.run(
            [sys.executable, os.path.join(here, "bridge-selftest.py")], check=False
        ).returncode
    apk = argv[1] if len(argv) > 1 else ""
    if not apk or not os.path.isfile(apk):
        raise SystemExit("usage: apk-bridges.py <apk>")
    root = os.path.join(os.path.dirname(os.path.abspath(__file__)), os.pardir, "src")
    pins, classes, complaints = pins_in(root)
    dump, dexes = dumped(apk, tool())
    said = judge(pins, classes, dex_statics(dump), complaints)
    if said:
        print("bridges: the crate and the dex disagree:", file=sys.stderr)
        for line in said:
            print(f"  {line}", file=sys.stderr)
        return 1
    print(f"bridges: {len(pins)} JNI name(s) over {len(classes)} class(es), "
          f"both directions, against {dexes} dex file(s)")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
