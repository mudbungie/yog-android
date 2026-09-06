"""**The third direction: a `native` the dex declares must be a symbol the
packaged library exports** (bl-8c32).

`apk-bridges.py`'s two directions are both about the crate talking INTO Java —
a name the Rust resolves must exist in the dex, and a public static of a
bridged class must be named by some Rust site. The traffic the other way is
not a call the crate makes at all: `Pocket.standing` is `private static
native` in the dex and its body is a symbol in `lib/<abi>/libyog_android.so`,
bound by the runtime the first time the method is reached.

Nothing checked it, and the failure it hides is strictly worse than the two
that were checked: a stale `.so` married to a current dex exports **none** of
the natives, so the app dies in `onResume` with `UnsatisfiedLinkError` on
every launch and paints nothing. That is what shipped — an APK the gate
passed, `nm -D` finding zero `Java_dev_yog_*` symbols in it, and "yog keeps
stopping" as the whole of what a person saw.

**The `.so` is read here rather than shelled out to.** `nm -D` is not on every
box that can assemble an APK, and the NDK's copy moves with the NDK; the
dynamic symbol table is four struct reads into a file this script already has
open, for both bitnesses and both endiannesses. **A library this reader cannot
read is an infrastructure fault reported in its own sentence** and never as an
absent symbol — the rule `leak-selftest.sh` keeps for a fixture that is not
text, and for its reason: the box's fault and the tree's fault must not arrive
as the same sentence.

**The package is derived, never spelled.** The natives belong to the classes
this crate is answerable for, and the doors it opens already name them; a
second literal `dev.yog` here would be a fact with two homes. Doors that no
longer share one package say so instead of guessing.
"""

import re
import struct
import zipfile

MAGIC = b"\x7fELF"
SHT_DYNSYM = 11
LIB = re.compile(r"lib/([^/]+)/[^/]+\.so")


class Unreadable(Exception):
    """This reader could not get at the library's symbols at all."""


# Where a section header keeps (type, offset, size, link), and how wide each
# field is, in the two ELF classes. One table rather than two branches: the
# layout is the only thing that differs between them.
SECTION = {
    False: ((4, "I"), (16, "I"), (20, "I"), (24, "I")),
    True: ((4, "I"), (24, "Q"), (32, "Q"), (40, "I")),
}


def sections(blob, end, wide):
    """Every section header as (type, offset, size, link)."""
    if wide:
        (at,) = struct.unpack_from(end + "Q", blob, 0x28)
        size, count = struct.unpack_from(end + "HH", blob, 0x3A)
    else:
        (at,) = struct.unpack_from(end + "I", blob, 0x20)
        size, count = struct.unpack_from(end + "HH", blob, 0x2E)
    return [
        tuple(
            struct.unpack_from(end + code, blob, at + i * size + place)[0]
            for place, code in SECTION[wide]
        )
        for i in range(count)
    ]


def named(strings, at):
    """One NUL-terminated name out of a string table."""
    end = strings.find(b"\0", at)
    if at >= len(strings) or end < 0:
        raise Unreadable("a symbol name runs past the string table")
    return strings[at:end].decode("ascii", "replace")


def exports(blob):
    """Every DEFINED symbol in an ELF's dynamic symbol table — the exact set a
    runtime can bind a `native` declaration to. An undefined entry (section
    index 0) is a symbol this library *wants*, and answers for nothing."""
    if blob[:4] != MAGIC:
        raise Unreadable("not an ELF object")
    if blob[4] not in (1, 2) or blob[5] not in (1, 2):
        raise Unreadable("an ELF class or byte order this reader does not know")
    wide, end = blob[4] == 2, "<" if blob[5] == 1 else ">"
    try:
        table = sections(blob, end, wide)
    except struct.error as truncated:
        raise Unreadable(f"the section table does not fit the file: {truncated}") from truncated
    found, seen = set(), False
    for kind, at, size, link in table:
        if kind != SHT_DYNSYM or link >= len(table):
            continue
        seen = True
        _, stroff, strsize, _ = table[link]
        strings = blob[stroff:stroff + strsize]
        step = 24 if wide else 16
        for entry in range(at, at + size - step + 1, step):
            try:
                (name,) = struct.unpack_from(end + "I", blob, entry)
                (where,) = struct.unpack_from(end + "H", blob, entry + (6 if wide else 14))
            except struct.error as short:
                raise Unreadable(
                    f"the symbol table runs past the file: {short}"
                ) from short
            if where:
                found.add(named(strings, name))
    if not seen:
        raise Unreadable("no dynamic symbol table")
    return found


def mangled(text):
    """JNI's name mangling: `_` doubles up so a package separator and an
    underscore in a method name cannot collide."""
    out = []
    for ch in text:
        if ch in "./":
            out.append("_")
        elif ch == "_":
            out.append("_1")
        elif ch == ";":
            out.append("_2")
        elif ch == "[":
            out.append("_3")
        elif ch.isascii() and ch.isalnum():
            out.append(ch)
        else:
            out.append(f"_0{ord(ch):04x}")
    return "".join(out)


def symbols(klass, method, descriptor):
    """The two names a runtime looks for, in the order it looks: the short
    one, then the argument-qualified one an overload needs. Either satisfies a
    declaration, which is exactly the runtime's own rule."""
    stem = f"Java_{mangled(klass)}_{mangled(method)}"
    args = descriptor[1:descriptor.index(")")]
    return stem, f"{stem}__{mangled(args)}"


def declared(methods, classes):
    """The native declarations this crate owes a symbol for, and the
    complaints the derivation itself has. An empty set is a broken scan and
    not a clean tree — `dumped`'s rule at the third direction."""
    said = []
    packages = {klass.rsplit(".", 1)[0] for klass in classes}
    if len(packages) != 1:
        named = f"{len(packages)} packages ({', '.join(sorted(packages))})" \
            if packages else "no package"
        return set(), [f"the bridged classes name {named} — which one owns "
                       "this app's natives is no longer derivable"]
    package = f"{packages.pop()}."
    natives = {
        (klass, name, kind)
        for klass, name, kind, access in methods
        if "NATIVE" in access.split() and klass.startswith(package)
    }
    if not natives:
        said.append(f"the dex declares no native method under {package} — "
                    "the scan is broken, not the tree")
    return natives, said


def judge(natives, exported):
    """Every declaration against every READABLE ABI's exported set. Per ABI,
    because one stale library beside a fresh one is exactly the shape that
    shipped: the phone's arm64 copy was current and the emulator's x86_64 copy
    was not. An ABI whose libraries could not be read is not in this map at
    all — [`exported_of`] has already said so in its own sentence, and judging
    it against an empty set would reprint the box's fault as five of the
    tree's."""
    said = []
    for abi in sorted(exported):
        for klass, method, descriptor in sorted(natives):
            short, qualified = symbols(klass, method, descriptor)
            if short not in exported[abi] and qualified not in exported[abi]:
                said.append(f"{klass}.{method} is declared native in the dex and "
                            f"lib/{abi}/ exports neither {short} nor {qualified}")
    return said


def exported_of(apk):
    """Each ABI's exported symbols, unioned over the libraries it packs (a
    native may live in any of them, which is the runtime's own rule) — and
    what the artifact itself is missing. **An unreadable library contributes
    no ABI**, rather than an empty one: the fault is stated once here, and the
    judgement never sees a set it could mistake for a stale build."""
    out, said, packed = {}, [], 0
    with zipfile.ZipFile(apk) as archive:
        for entry in sorted(archive.namelist()):
            abi = LIB.fullmatch(entry)
            if not abi:
                continue
            packed += 1
            try:
                found = exports(archive.read(entry))
            except Unreadable as fault:
                said.append(f"{entry}: {fault} — this reader cannot judge that library, "
                            "which is not the same as a symbol being absent")
                continue
            out.setdefault(abi.group(1), set()).update(found)
    if not packed:
        said.append("the APK packs no lib/<abi>/*.so — nothing can define a native")
    return out, said
