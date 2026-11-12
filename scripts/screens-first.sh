#!/usr/bin/env bash
# **THE FIRST RUN'S BEATS** (bl-1701), sourced by `scripts/screens-walk.sh` and
# split from it at the 300-line cap on the seam the app itself draws: what a
# device with nothing provisioned can reach, before any seed exists.

# 1. Nothing provisioned: the bootstrap chooser, which is every device's first
#    screen and the one that needs no seed at all.
wipe_app; relaunch
capture cold configuration

# 1r. THE REGISTRATION DOOR (DESIGN §5): one tap behind the chooser, and the
#     only way a device gets in since the enroll envelope's import was
#     deleted — the URL field, `register`, `scan QR`, and the carried leaf's
#     four files under them. Nothing is claimed: a claim needs a door with a
#     publicly trusted certificate, which no walk can stand up.
tap_control lernie
capture door enroll-seat
