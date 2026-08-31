.PHONY: all build release test conformance coverage lint fmt fmt-check check ci clean rules-audit line-cap leak-scan deny install-hooks apk

all: check

build:
	cargo build

release:
	cargo build --release

# The APK, two stages: cargo-ndk cross-builds the cdylib into the Gradle
# module's jniLibs, then Gradle assembles the debug APK (debug-keystore
# signing; a release channel gets its own checklist ball before the first
# APK leaves this box — AGENTS.md).
#
# RELEASE PROFILE IS LOAD-BEARING, not an optimization: android-activity
# 0.6.1 aborts under debug-assertions on GameTextInput's pre-IME null buffer
# — instant close-on-launch on frame one (bl-014e; the one-line upstream
# null guard is tracked under bl-2958).
#
# Requires: the Android NDK + cargo-ndk (`cargo install cargo-ndk`), the
# aarch64-linux-android target (rust-toolchain.toml pins it), and a SYSTEM
# `gradle` (8.7+, JDK 17). There is deliberately no gradle wrapper: the
# wrapper is a committed jar, and the leak gate refuses any binary it cannot
# read (BINARY_ALLOWED matches nothing) — which is correct, so the pin lives
# in this comment and the README instead of a jar.
apk:
	cargo ndk -t arm64-v8a -o android/app/src/main/jniLibs build --release
	cd android && gradle assembleDebug
	@echo "apk: android/app/build/outputs/apk/debug/app-debug.apk"

test:
	cargo test

# The wire conformance corpus, replayed (REMOTE §3). Part of `test` and so of
# `check` — this target is the named hand-run, for the loop where the corpus
# has just been re-vendored from a yog checkout and only its verdict matters.
conformance:
	cargo test --test conformance

TARPAULIN_PIN := 0.35.2

coverage:
	@have=$$(cargo tarpaulin --version 2>/dev/null | awk '{print $$NF}'); \
	if [ "$$have" != "$(TARPAULIN_PIN)" ]; then \
	  echo "tarpaulin $(TARPAULIN_PIN) required (have: $${have:-none}); see tarpaulin.toml" >&2; \
	  echo "  cargo install cargo-tarpaulin --version $(TARPAULIN_PIN) --locked" >&2; \
	  exit 1; \
	fi
	cargo tarpaulin --fail-under 100 --skip-clean --engine llvm --out Stdout

# The complete static gate: the 300-line cap + the disclosure scan + clippy
# (reads Cargo.toml [lints]) + the ast-grep rules audit + the cargo-deny
# supply-chain audit. All are pinned so the gate is reproducible — ast-grep
# 0.44.1 (sgconfig.yml), cargo-deny 0.20.2 (deny.toml), toolchain 1.95.0
# (rust-toolchain.toml). CI runs this exact target via `make ci`; there is no
# CI-only lint step to drift from. `line-cap` goes first: it is milliseconds,
# so a structural violation fails before the minute-scale tools start.
lint:
	$(MAKE) line-cap
	$(MAKE) leak-scan
	cargo clippy --all-targets -- -D warnings
	$(MAKE) rules-audit
	$(MAKE) deny

# The disclosure gate (yog bl-fd5a/bl-167d, adopted here from day zero): no
# credential, routable address, home path, personal address, pasted dialogue,
# agent-session artifact or unreadable blob in the tree. Both directions in
# one target — the tree must be clean, AND the scanner's own fixtures must
# still fire, per RULE and per LINE, so an edited pattern that silently
# matches nothing cannot pass. `scripts/leak-rules.sh` is the one definition
# of what counts, `leak-scan.sh` runs it, and this is the door. It reads INDEX
# BLOBS, not the worktree: the bytes scanned are the bytes committed. It also
# runs from `scripts/pre-commit` BEFORE the verdict cache is consulted — the
# one gate step no stored verdict may skip.
#
# The same scanner gates the balls TASK STORE: the machine-global
# `bl-leak-gate` plugin execs `scripts/leak-scan.sh --commit <op's commit>`
# before every `bl` publish — the scanner's presence in this repo IS that
# opt-in. Ball bodies are published text; see AGENTS.md, "What may never enter
# a ball body".
leak-scan:
	@scripts/leak-scan.sh --self-test
	@scripts/leak-scan.sh

# Supply-chain audit (cargo-deny 0.20.2 — see deny.toml): licenses,
# advisories, bans (openssl-sys / native-tls / aws-lc-sys — rustls-only,
# ring-only, before the first TLS dependency even lands), and known-registry
# sources.
deny:
	cargo deny check

# Static audit of every ast-grep rule (rules/, pinned ast-grep 0.44.1 — see
# sgconfig.yml). Both directions: `src` must be clean (exit 0), and every
# deliberate violation in rules/fixtures must fire (scan exits non-zero) so a
# silently-broken rule cannot pass unnoticed.
rules-audit:
	ast-grep scan src
	@if ast-grep scan rules/fixtures >/dev/null 2>&1; then \
	  echo "rules-audit: rules/fixtures was NOT flagged — a rule has regressed" >&2; \
	  exit 1; \
	fi
	@echo "rules-audit: src clean; fixtures flagged (all rules live)"

# The 300-line cap on source files (AGENTS.md; docs and config are exempt).
# This target is the ONE definition of the cap and of what counts as a source
# file — the pre-commit hook and CI both call it, neither restates it. It
# scans the WHOLE TREE, not the staged diff: a diff-only gate is a sampling,
# not an invariant (yog bl-12dc: a file rode at 308 lines undetected until an
# unrelated task edited it). `git ls-files` reads the INDEX, so a staged
# addition is covered before it is ever committed. Offenders are reported ALL
# AT ONCE, and the empty-set guard is the target's own negative check: a
# broken pattern must not pass silently.
#
# The cap is a variable so the same target answers the design-time question:
# `make line-cap LINE_CAP=199` lists the ≥200 pre-split band — run it before
# extending a module. Anything projected ≥200 is pre-split at design time; 300
# is the wall, never the target.
LINE_CAP := 300
LINE_CAP_EXEMPT := \.(md|txt|toml|yaml|yml|json|lock)$$|(^|/)(Makefile|LICENSE|\.gitignore|\.githooks/)

line-cap:
	@files=$$(git ls-files | grep -Ev '$(LINE_CAP_EXEMPT)' || true); \
	n=$$(printf '%s\n' "$$files" | grep -c . || true); \
	over=$$(printf '%s\n' "$$files" | while IFS= read -r f; do \
	    { [ -n "$$f" ] && [ -f "$$f" ]; } || continue; \
	    c=$$(wc -l < "$$f"); \
	    [ "$$c" -gt $(LINE_CAP) ] && printf '  %s: %s lines\n' "$$f" "$$c"; \
	    true; \
	  done); \
	if [ "$$n" -eq 0 ]; then \
	  echo "line-cap: enumerated 0 source files — the scan is broken, not the tree" >&2; \
	  exit 1; \
	fi; \
	if [ -n "$$over" ]; then \
	  echo "error: source files over the $(LINE_CAP)-line cap:" >&2; \
	  printf '%s\n' "$$over" >&2; \
	  echo "       split along a real seam — do not shave lines." >&2; \
	  exit 1; \
	fi; \
	echo "line-cap: $$n source files, all within $(LINE_CAP) lines"

fmt:
	cargo fmt

fmt-check:
	cargo fmt --check

# The complete gate, and the exact target CI runs (`ci`). Coverage goes
# through `scripts/check-coverage.sh` rather than the bare `coverage` target
# so the pre-commit hook, `make check` and CI share ONE coverage step (held
# output, replayed on failure; the signaled-tarpaulin three-outcome contract —
# yog bl-673a). `make coverage` stays the bare, always-verbose hand-run.
check: fmt-check lint
	@scripts/check-coverage.sh

ci: check

# Arm this clone's git hooks: one symlink per file in .githooks/, seated in
# the repo's own hooks directory. Symlinks, not copies, so an updated hook is
# live without a re-run. NOT `core.hooksPath` — this machine may set that
# globally to a chain hook whose second job is to exec
# `<git-common-dir>/hooks/<name>`; seating the links where it looks keeps
# both. Refused from a linked worktree: `bl claim` deletes those.
install-hooks:
	@top=$$(git rev-parse --path-format=absolute --show-toplevel) && \
	common=$$(git rev-parse --path-format=absolute --git-common-dir) && \
	if [ "$$common" != "$$top/.git" ]; then \
	  echo "install-hooks: run this in the main checkout, not a linked worktree" >&2; \
	  exit 1; \
	fi; \
	mkdir -p "$$common/hooks"; \
	for h in .githooks/*; do \
	  ln -sfn "$$top/$$h" "$$common/hooks/$${h#.githooks/}"; \
	done; \
	echo "hooks: $$common/hooks/{$$(ls .githooks | tr '\n' ',' | sed 's/,$$//')} -> $$top/.githooks"

clean:
	cargo clean
