#!/usr/bin/env bash
# **The packaged executables** (bl-f22a, DESIGN §16.1's net rung): curl and
# busybox, built from pinned source and deposited in the Gradle module's
# jniLibs as `libcurl_bin.so` and `libbusybox_bin.so`.
#
# **Why that naming, and why jniLibs at all.** An Android app may execute a
# file under its own `nativeLibraryDir` and effectively nowhere else — since
# API 29 the app's writable data directory is W^X, so a binary copied there at
# runtime cannot be `exec`d at all. `nativeLibraryDir` is filled by the
# installer from `lib/<abi>/` in the APK, and Gradle puts a file there only if
# it is named `lib*.so`. So the name is not a disguise: it is the one shape
# the platform will carry an executable in.
#
# **Pinned by version AND sha256, and the pins live in the Makefile** — one
# home for the four numbers, and a tarball that does not hash to its pin stops
# the build rather than being built. The sources are the upstream projects'
# own release tarballs, fetched over https; nothing is vendored into this
# repository, which is also why the disclosure gate never has to judge a
# binary it cannot read.
#
# **What "static" means here, stated exactly.** curl is linked statically
# against its own OpenSSL and dynamically against bionic — `libc`, `libm`,
# `libdl`, the three libraries every Android device is guaranteed to have and
# the only ones the linker will find. A fully static bionic binary is not the
# goal and would be worse: its DNS resolution goes around the platform's own
# resolver. busybox is the same shape, with no third-party library at all.
#
# **Neither carries a CA bundle**, because the device has one: curl is built
# `--without-ca-bundle --without-ca-path`, and `dev.yog.Kit` concatenates the
# platform's own trusted roots into one PEM at every launch while the shell
# tool points `SSL_CERT_FILE` at it. A FILE rather than the platform's
# directory, and the reason is exact: Android names its certificate files by
# OpenSSL's OLD subject hash (`X509_NAME_hash_old`) and OpenSSL 1.0 and later
# look a directory up by the new one — measured, the store holds `01419da9.0`
# where OpenSSL 3 goes looking for `8d89cda1.0`, so a `capath` at the store
# finds nothing and every https fetch fails to build a chain. So the trust
# store is the device's, exactly as the `http` tool's is, and there is
# nothing here that can go stale for longer than one launch.
#
# It is idempotent and cached under `target/natives`: a second run with the
# same pins rebuilds nothing. Deleting that directory is the whole of "clean".
set -euo pipefail

BUSYBOX_VERSION="${BUSYBOX_VERSION:?}"
BUSYBOX_SHA256="${BUSYBOX_SHA256:?}"
CURL_VERSION="${CURL_VERSION:?}"
CURL_SHA256="${CURL_SHA256:?}"
OPENSSL_VERSION="${OPENSSL_VERSION:?}"
OPENSSL_SHA256="${OPENSSL_SHA256:?}"
ABIS="${ABIS:-arm64-v8a x86_64}"
JNILIBS="${JNILIBS:-android/app/src/main/jniLibs}"
NATIVES_CACHE="${NATIVES_CACHE:-target/natives}"
API="${NATIVES_API:-28}"

here=$(cd "$(dirname "$0")" && pwd)
root=$(cd "$here/.." && pwd)
cache=$(cd "$root" && mkdir -p "$NATIVES_CACHE" && cd "$NATIVES_CACHE" && pwd)
out=$(cd "$root" && mkdir -p "$JNILIBS" && cd "$JNILIBS" && pwd)

ndk_root() {
  if [ -n "${ANDROID_NDK_ROOT:-}" ]; then printf '%s\n' "$ANDROID_NDK_ROOT"; return; fi
  local sdk="${ANDROID_HOME:-${ANDROID_SDK_ROOT:-$HOME/Android/Sdk}}"
  ls -d "$sdk"/ndk/* 2>/dev/null | sort -V | tail -1
}

NDK=$(ndk_root)
[ -d "$NDK" ] || { echo "natives: no NDK found (set ANDROID_NDK_ROOT)" >&2; exit 1; }
BIN="$NDK/toolchains/llvm/prebuilt/linux-x86_64/bin"
[ -d "$BIN" ] || { echo "natives: $BIN is not a toolchain" >&2; exit 1; }

fetch() { # url sha256 file
  local url=$1 want=$2 file="$cache/$3"
  if [ -f "$file" ]; then
    local have; have=$(sha256sum "$file" | cut -d' ' -f1)
    [ "$have" = "$want" ] && { printf '%s\n' "$file"; return; }
    rm -f "$file"
  fi
  curl -sSLf --max-time 900 -o "$file.part" "$url"
  local have; have=$(sha256sum "$file.part" | cut -d' ' -f1)
  [ "$have" = "$want" ] || { echo "natives: $3 is $have, pinned $want" >&2; exit 1; }
  mv "$file.part" "$file"
  printf '%s\n' "$file"
}

triple_of() { case $1 in arm64-v8a) echo aarch64-linux-android;; x86_64) echo x86_64-linux-android;;
  *) echo "natives: no toolchain for ABI $1" >&2; exit 1;; esac; }
ossl_of()   { case $1 in arm64-v8a) echo android-arm64;; x86_64) echo android-x86_64;; esac; }
arch_of()   { case $1 in arm64-v8a) echo arm64;; x86_64) echo x86_64;; esac; }

BB_TAR=$(fetch "https://busybox.net/downloads/busybox-$BUSYBOX_VERSION.tar.bz2" "$BUSYBOX_SHA256" "busybox-$BUSYBOX_VERSION.tar.bz2")
CURL_TAR=$(fetch "https://curl.se/download/curl-$CURL_VERSION.tar.xz" "$CURL_SHA256" "curl-$CURL_VERSION.tar.xz")
SSL_TAR=$(fetch "https://github.com/openssl/openssl/releases/download/openssl-$OPENSSL_VERSION/openssl-$OPENSSL_VERSION.tar.gz" "$OPENSSL_SHA256" "openssl-$OPENSSL_VERSION.tar.gz")

jobs=$(nproc 2>/dev/null || echo 4)

# **The applet list is a list of kconfig SYMBOLS, flipped in place** — never
# appended. An appended `CONFIG_X=y` is silently reset by `oldconfig` (busybox
# 1.36's kconfig re-derives the file), which is the failure mode that produces
# a 62 KB busybox carrying nothing at all and no error anywhere. Flipping the
# `# CONFIG_X is not set` line kconfig already wrote keeps the assignment, and
# a symbol that is in neither shape is a NAME THAT MATCHES NOTHING: refused
# here rather than dropped, the two-direction rule every gate in this repo
# keeps.
# **`oldconfig`'s answers, from a HERESTRING and never from a pipe.** A new
# integer symbol (busybox 1.36 adds several under the applets enabled here)
# is prompted for, and an EOF on stdin is an error rather than a default — so
# something must answer. `yes '' | make …` is the shape that must not be
# used: the writer dies of SIGPIPE the moment `make` exits, and under
# `pipefail` the pipeline then reports 141 on exactly the runs that SUCCEEDED
# (measured here; it is the same defect the repo's own beat audit bans in the
# test harness). A herestring has no second process to die.
blank_answers() { printf '\n%.0s' $(seq 1000); }

applets() { grep -v '^#' "$here/busybox-applets.conf" | grep . ; }

enable_applets() {
  local sym
  for sym in $(applets); do
    sed -i "s/^# $sym is not set\$/$sym=y/" .config
    if ! grep -q "^$sym=y" .config; then
      echo "natives: busybox $BUSYBOX_VERSION has no config symbol $sym" >&2
      return 1
    fi
  done
}

confirm_applets() {
  local sym missing=
  for sym in $(applets); do
    grep -q "^$sym=y" .config || missing="$missing $sym"
  done
  [ -z "$missing" ] || { echo "natives: busybox dropped:$missing" >&2; return 1; }
}

build_abi() {
  local abi=$1 triple; triple=$(triple_of "$abi")
  local work="$cache/$abi" prefix="$cache/$abi/prefix"
  mkdir -p "$work" "$prefix"
  export PATH="$BIN:$PATH"
  export ANDROID_NDK_ROOT="$NDK"
  export CC="$BIN/${triple}${API}-clang" AR="$BIN/llvm-ar" RANLIB="$BIN/llvm-ranlib" STRIP="$BIN/llvm-strip"

  if [ ! -f "$prefix/lib/libssl.a" ]; then
    echo "natives[$abi]: openssl $OPENSSL_VERSION"
    rm -rf "$work/openssl"; mkdir -p "$work/openssl"
    tar -C "$work/openssl" --strip-components=1 -xzf "$SSL_TAR"
    ( cd "$work/openssl" && ./Configure "$(ossl_of "$abi")" -D__ANDROID_API__=$API \
        no-shared no-tests no-docs no-legacy no-engine no-comp no-dtls no-srp no-psk \
        no-ssl3 no-weak-ssl-ciphers no-idea no-md2 no-mdc2 no-rc5 no-camellia no-aria \
        no-seed no-whirlpool no-gost no-scrypt --prefix="$prefix" >/dev/null \
      && make -j"$jobs" >/dev/null && make install_sw >/dev/null )
  fi

  if [ ! -x "$work/curl-out/bin/curl" ]; then
    echo "natives[$abi]: curl $CURL_VERSION"
    rm -rf "$work/curl"; mkdir -p "$work/curl"
    tar -C "$work/curl" --strip-components=1 -xJf "$CURL_TAR"
    ( cd "$work/curl" && ./configure --host="$triple" --prefix="$work/curl-out" \
        --with-openssl="$prefix" --disable-shared --enable-static \
        --without-ca-bundle --without-ca-path --without-libpsl --without-libidn2 \
        --without-brotli --without-zstd --without-nghttp2 --without-librtmp \
        --disable-ldap --disable-ldaps --disable-manual --disable-dict --disable-gopher \
        --disable-imap --disable-pop3 --disable-smtp --disable-rtsp --disable-telnet \
        --disable-tftp --disable-smb --disable-mqtt \
        CFLAGS="-Os -fPIE" LDFLAGS="-pie" >/dev/null \
      && make -j"$jobs" >/dev/null && make install >/dev/null )
  fi

  if [ ! -x "$work/busybox/busybox" ]; then
    echo "natives[$abi]: busybox $BUSYBOX_VERSION"
    rm -rf "$work/busybox"; mkdir -p "$work/busybox"
    tar -C "$work/busybox" --strip-components=1 -xjf "$BB_TAR"
    ( cd "$work/busybox" \
      && make ARCH="$(arch_of "$abi")" allnoconfig >/dev/null \
      && enable_applets \
      && make ARCH="$(arch_of "$abi")" CROSS_COMPILE="$BIN/llvm-" CC="$CC" HOSTCC=cc oldconfig <<<"$(blank_answers)" >/dev/null \
      && confirm_applets \
      && make ARCH="$(arch_of "$abi")" CROSS_COMPILE="$BIN/llvm-" CC="$CC" HOSTCC=cc SKIP_STRIP=y -j"$jobs" busybox )
  fi

  mkdir -p "$out/$abi"
  "$STRIP" -o "$out/$abi/libcurl_bin.so" "$work/curl-out/bin/curl"
  "$STRIP" -o "$out/$abi/libbusybox_bin.so" "$work/busybox/busybox"
  ls -l "$out/$abi/libcurl_bin.so" "$out/$abi/libbusybox_bin.so"
}

for abi in $ABIS; do build_abi "$abi"; done
