#!/usr/bin/env python3
"""**The third direction's own regression half** (bl-8c32), in
`bridge-selftest.py`'s shape and for its reason: a gate dies by matching
nothing, and one nobody can watch fail is indistinguishable from one that
works.

The negative fixture this exists for is the APK that shipped — a dex declaring
five natives beside a library exporting none of them, which the two-direction
gate passed and which died in `onResume` on every launch. It is the second arm
below, and it must stay red.

The libraries are **fabricated ELF objects**, not files off this box: the
reader must be provable on both widths and on a symbol that is merely
*wanted*, and neither is a thing a machine's own `/usr/lib` can be asked to
hold on demand.
"""

import os
import struct
import sys
import tempfile
import zipfile

import apk_natives as NATIVES

STRING = "Ljava/lang/String;"
CLASSES = {"dev.yog.Paper", "dev.yog.Shade"}
# One native declaration per shape that matters: the plain one, and an
# overloaded one whose only symbol is the argument-qualified spelling.
WHOLE = [
    ("dev.yog.Pocket", "standing", f"({STRING}){STRING}", "PRIVATE STATIC NATIVE"),
    ("dev.yog.Watch", "probe", f"({STRING}){STRING}", "PRIVATE STATIC NATIVE"),
    ("dev.yog.Pocket", "arm", f"(){STRING}", "PUBLIC STATIC"),
    ("androidx.other.Thing", "helper", "()V", "PUBLIC STATIC NATIVE"),
]
SHORT = ["Java_dev_yog_Pocket_standing", "Java_dev_yog_Watch_probe"]
QUALIFIED = [f"{name}__Ljava_lang_String_2" for name in SHORT]


def section(end, wide, name, kind, at, size, link, entsize):
    if wide:
        return struct.pack(end + "IIQQQQIIQQ", name, kind, 0, 0, at, size, link, 0, 1, entsize)
    return struct.pack(end + "IIIIIIIIII", name, kind, 0, 0, at, size, link, 0, 1, entsize)


def elf(symbols, wide=True, end="<"):
    """A minimal ELF whose dynamic symbol table carries exactly these
    `(name, defined)` pairs — three sections, no program headers, nothing this
    reader does not look at."""
    strings, places, at = b"\0", [], 1
    for name, _ in symbols:
        strings += name.encode() + b"\0"
        places.append(at)
        at += len(name) + 1
    syms = b""
    for (_, defined), place in zip(symbols, places):
        where = 1 if defined else 0
        syms += struct.pack(end + "IBBHQQ", place, 0x12, 0, where, 0, 0) if wide \
            else struct.pack(end + "IIIBBH", place, 0, 0, 0x12, 0, where)
    head_size, sh_size, sym_size = (64, 64, 24) if wide else (52, 40, 16)
    stroff = head_size
    symoff = stroff + len(strings)
    shoff = symoff + len(syms)
    table = (section(end, wide, 0, 0, 0, 0, 0, 0)
             + section(end, wide, 0, 3, stroff, len(strings), 0, 0)
             + section(end, wide, 0, NATIVES.SHT_DYNSYM, symoff, len(syms), 1, sym_size))
    ident = NATIVES.MAGIC + bytes([2 if wide else 1, 1 if end == "<" else 2, 1]) + b"\0" * 9
    shape = "HHIQQQIHHHHHH" if wide else "HHIIIIIHHHHHH"
    head = ident + struct.pack(end + shape, 3, 183, 1, 0, 0, shoff, 0,
                               head_size, 0, 0, sh_size, 3, 0)
    return head + strings + syms + table


def apk(entries):
    """A throwaway zip carrying exactly these members."""
    at = os.path.join(tempfile.mkdtemp(prefix="yog-natives-"), "app.apk")
    with zipfile.ZipFile(at, "w") as archive:
        for name, blob in entries.items():
            archive.writestr(name, blob)
    return at


def verdict(methods, exported, classes=None):
    natives, said = NATIVES.declared(methods, CLASSES if classes is None else classes)
    return said + NATIVES.judge(natives, exported)


def beat(name, said, wanted):
    if wanted is None:
        ok, why = not said, f"expected nothing, said: {said}"
    else:
        ok = any(wanted in line for line in said)
        why = f"expected {wanted!r}, said: {said}"
    print(f"  {'pass' if ok else 'FAIL'}  {name}")
    return None if ok else why


def reading(name, got, wanted):
    ok = got == wanted
    print(f"  {'pass' if ok else 'FAIL'}  {name}")
    return None if ok else f"read {got!r}, wanted {wanted!r}"


def main():
    both = {"arm64-v8a": set(SHORT), "x86_64": set(SHORT)}
    failed = [
        beat("a library that defines every native passes", verdict(WHOLE, both), None),
        beat(
            "a library exporting none of them is caught",
            verdict(WHOLE, {"x86_64": {"main"}}),
            "dev.yog.Pocket.standing is declared native in the dex and lib/x86_64/",
        ),
        beat(
            "one stale ABI beside a fresh one is caught",
            verdict(WHOLE, {"arm64-v8a": set(SHORT), "x86_64": set()}),
            "lib/x86_64/ exports neither",
        ),
        beat(
            "the argument-qualified spelling satisfies a declaration",
            verdict(WHOLE, {"x86_64": set(QUALIFIED)}),
            None,
        ),
        beat(
            "a dex declaring no native of this crate's own is a broken scan",
            verdict([m for m in WHOLE if "NATIVE" not in m[3]], both),
            "the scan is broken, not the tree",
        ),
        beat(
            "an APK packing no library is caught",
            NATIVES.exported_of(apk({"classes.dex": b"dex\n035\0"}))[1],
            "packs no lib/<abi>/*.so",
        ),
        beat(
            "doors naming two packages refuse rather than guess",
            verdict(WHOLE, both, CLASSES | {"com.example.Other"}),
            "no longer derivable",
        ),
        reading(
            "a defined symbol is read and an undefined one answers for nothing",
            NATIVES.exports(elf([("Java_dev_yog_Watch_probe", True), ("abort", False)])),
            {"Java_dev_yog_Watch_probe"},
        ),
        reading(
            "a 32-bit big-endian library reads the same",
            NATIVES.exports(elf([("Java_dev_yog_Watch_probe", True)], wide=False, end=">")),
            {"Java_dev_yog_Watch_probe"},
        ),
        reading(
            "an ABI's libraries are unioned, and a non-ELF member is a fault of its own",
            NATIVES.exported_of(apk({
                "lib/x86_64/libyog_android.so": elf([(SHORT[0], True)]),
                "lib/x86_64/libc++_shared.so": elf([(SHORT[1], True)]),
                "lib/arm64-v8a/libyog_android.so": b"not an object at all",
                "classes.dex": b"dex\n035\0",
            })),
            ({"x86_64": set(SHORT)},
             ["lib/arm64-v8a/libyog_android.so: not an ELF object — this reader "
              "cannot judge that library, which is not the same as a symbol being absent"]),
        ),
    ]
    failed = [why for why in failed if why]
    for why in failed:
        print(f"natives: self-test: {why}", file=sys.stderr)
    if not failed:
        print("natives: self-test OK — the third direction bites, the ELF reader reads")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
