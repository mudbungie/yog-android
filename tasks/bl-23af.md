+++
title = "the APK's device matrix is two: an x86_64 library beside the arm64 one"
created = 1788147957
updated = 1788147957
claimant = "OrderPuppeteer"
priority = 7
root_commit = "b8421205e882caeadc666ccff26464e4e0f60dda"
+++
The APK has been arm64-only since the spike, and the Makefile says why in
as many words: *"arm64 only, matching the spike's device matrix of one; a
second ABI is added when a device that needs it exists, not speculatively."*

**A device that needs it now exists.** Story-driving the enrollment surfaces
needs a virtual device, the host is x86_64 with KVM, and an ARM image under
full emulation is slow enough that the loop stops being a loop. The
emulator's ABI is `x86_64`, so today the APK simply will not install on it:
the condition the comment names as the trigger has fired, and this is the
change it was waiting for.

**Both ABIs in one APK, not a separate testing target.** The alternative —
a second Makefile verb that builds an emulator-only APK — puts two artifacts
in play and makes "which one is on the device?" a question anyone reading a
test verdict has to ask. One APK that installs on every device in the matrix
answers it structurally. Gradle already packs per-ABI directories under
`jniLibs/`, so the second library is picked by the installer, not by the
builder, and an arm64 phone never loads the x86_64 copy.

**The list is a variable, for the reason `GRADLE` already is.** A box that
only ever flashes the phone should not have to edit this file to skip a
cross-build it has no use for — `make apk ABIS=arm64-v8a`. Default is both,
so the honest path is the one you get by typing nothing.

What ships:

- `rust-toolchain.toml` gains `x86_64-linux-android` beside the aarch64
  target, with the comment rewritten: the matrix is two, and it says which
  two and why rather than restating the old "one device" rule it no longer
  describes.
- The `apk` target takes `ABIS ?= arm64-v8a x86_64`, folded into one
  `cargo ndk` invocation (`-t` is repeatable), so both libraries land in
  `jniLibs/` before Gradle assembles.
- `README.md`'s build section names the two ABIs, the `ABIS` override and
  the emulator that motivated the second one.

Not in scope: a release channel, an app bundle, or ABI splits. The APK stays
one debug artifact carrying both libraries — splitting it is a size
optimization for a store listing that does not exist.