#!/usr/bin/env bash
# **The release signature, one recipe** (bl-7a68, DESIGN §20): align an
# assembled APK and sign it with this app's permanent key, then verify what
# was actually attached. Both doors spend it — `make apk-release` on a
# developer box, and the release workflow on a runner — so the signature a
# phone accepts has one definition and cannot differ by where it was made.
#
# THE KEY IS PERMANENT AND THAT IS THE WHOLE DESIGN CONSTRAINT (operator
# ruling 2026-09-05). An Android signing key is not rotatable: a device that
# installed under key A refuses an update signed by key B, and the only remedy
# is uninstall-and-reinstall, which loses the app's enrolment. So there is one
# key for this app's lifetime, it lives outside every checkout, and it is
# never committed — the disclosure gate refuses a `.keystore` path outright
# (`scripts/leak-rules.sh`, FORBIDDEN_PATH), which is the mechanism agreeing.
#
# WHY apksigner AND NOT A GRADLE signingConfig. Two reasons, and the second is
# the one that matters. Gradle's config picks its signature schemes from
# `minSdk` and states none of them in the build file, where this repository
# wants v3 named and asserted; and a signingConfig puts the keystore and its
# password into the Gradle daemon's own model, which is a long-lived process
# with a cache directory, for a build whose output is public. Signing after
# the assemble keeps the key in one short-lived process that reads it, uses
# it, and exits.
#
# SCHEME v3, NAMED AND ASSERTED (the operator ruling's own word), and it is
# the only one this app can carry. apksigner intersects the schemes it is
# ALLOWED with the ones the APK's own `minSdk` needs, and `minSdk 28` is
# exactly where v3 landed — measured, not assumed: asking for v1 and v2 as
# well at minSdk 28 produces an APK with no `META-INF/*.SF` in it at all,
# while the same command at `--min-sdk-version 21` writes them. So they are
# not asked for. v3 is also the scheme that carries a signer LINEAGE, which is
# what keeps the ruling's "one key for the app's lifetime" from being a trap:
# a future rotation could attach a lineage rather than orphan every installed
# copy. v4 is off — it is for incremental install over adb and needs a side
# file no release asset carries.
#
# NOTHING HERE PRINTS THE PASSWORD. apksigner reads it from the environment or
# from the file beside the keystore, by reference in both cases, and the
# verification at the end prints the CERTIFICATE — which is public by
# construction, since every installed copy of the app carries it.
set -euo pipefail

unsigned=${1:-}
signed=${2:-}
if [ -z "$unsigned" ] || [ -z "$signed" ]; then
  echo "usage: scripts/sign-apk.sh <assembled.apk> <signed.apk>" >&2
  exit 2
fi

die() { echo "sign-apk: $*" >&2; exit 1; }

[ -f "$unsigned" ] || die "no APK at $unsigned — assemble it first"

# The key, and where a box that has it keeps it. `YOG_KEYSTORE` names one
# outright — that is how CI hands over the one it materialized from a secret —
# and the default is the operator's own directory, which is severable in the
# only way that matters: a box without it builds the debug APK and this script
# is never reached.
KEYSTORE=${YOG_KEYSTORE:-$HOME/keys/yog-android/release.keystore}
ALIAS=${YOG_KEYSTORE_ALIAS:-yog-android}
[ -f "$KEYSTORE" ] || die "no signing key at $KEYSTORE — set YOG_KEYSTORE"

# The password, BY REFERENCE and never as a value on a command line, which
# every process on the box can read. The caller exports it (the runner does,
# from a repository secret); a developer box keeps it in one line beside the
# key and this reads it into the same variable, so there is ONE spelling below
# rather than two.
#
# **`env:` and not `file:`, and that is a finding rather than a taste.**
# apksigner reads a `file:` reference SEQUENTIALLY — the store password off the
# first line, the key password off the NEXT one — so a file holding one
# password answers `--ks-pass` and then dies on `--key-pass` with "end of file
# reached". `env:` is re-read independently for each, which is what a keystore
# whose key and store passwords are the same needs.
if [ -z "${YOG_KEYSTORE_PASSWORD:-}" ]; then
  [ -f "$KEYSTORE.password" ] \
    || die "no key password: export YOG_KEYSTORE_PASSWORD, or keep it beside the key"
  YOG_KEYSTORE_PASSWORD=$(head -n 1 "$KEYSTORE.password")
  export YOG_KEYSTORE_PASSWORD
fi
reference="env:YOG_KEYSTORE_PASSWORD"

# The build tools carry both commands, and the newest installed set is the one
# to use: zipalign and apksigner are format tools, not compilers, and an older
# pair cannot sign an APK a newer one assembled.
SDK=${ANDROID_HOME:-${ANDROID_SDK_ROOT:-$HOME/Android/Sdk}}
tools=$(ls -d "$SDK"/build-tools/*/ 2>/dev/null | sort -V | tail -1) \
  || die "no build-tools under $SDK — set ANDROID_HOME"
[ -n "$tools" ] || die "no build-tools under $SDK — set ANDROID_HOME"
ZIPALIGN="${tools}zipalign"
APKSIGNER="${tools}apksigner"
[ -x "$ZIPALIGN" ] || die "no zipalign at $ZIPALIGN"
[ -x "$APKSIGNER" ] || die "no apksigner at $APKSIGNER"

# ALIGNMENT BEFORE SIGNING, and it is an ordering, not a preference: a v2+
# signature covers the archive's bytes, so aligning afterwards would rewrite
# the bytes the signature was taken over and invalidate it.
mkdir -p "$(dirname "$signed")"
aligned="$signed.aligned"
"$ZIPALIGN" -p -f 4 "$unsigned" "$aligned"

"$APKSIGNER" sign \
  --ks "$KEYSTORE" \
  --ks-key-alias "$ALIAS" \
  --ks-pass "$reference" \
  --key-pass "$reference" \
  --v3-signing-enabled true \
  --v4-signing-enabled false \
  --out "$signed" \
  "$aligned"
rm -f "$aligned" "$signed.idsig"

# THE VERIFICATION IS THE POINT OF THE STEP, not a courtesy: `apksigner sign`
# answering 0 says a file was written, and what a phone judges is what is
# attached to it. This asserts the schemes and prints the certificate the
# whole channel is keyed to.
verdict=$("$APKSIGNER" verify --print-certs --verbose "$signed")
printf '%s\n' "$verdict"
# The assertion, and not the printing, is what makes this a step. `grep -q`
# reads from a HERESTRING and never from a pipe: a piped one exits the moment
# it matches, the writer dies of SIGPIPE mid-write, and `pipefail` then reports
# the pipeline failed BECAUSE the pattern matched (AGENTS.md).
grep -q '^Verified using v3 scheme.*: true$' <<<"$verdict" \
  || die "the signed APK carries no v3 signature — see the verdict above"
echo "sign-apk: $signed"
