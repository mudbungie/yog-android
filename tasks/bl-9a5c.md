+++
title = "the registration door has no walk beat: a tap on the chooser's lernie control missed on two emulator runs in three, the cold screen's insets still settling when the probe that places the tap is read"
created = 1794460844
updated = 1794460844
parent = "bl-233d"
priority = 3
root_commit = "b8421205e882caeadc666ccff26464e4e0f60dda"
+++
The chooser names its controls for the harness (lernie, thrall, yog) and a hand-driven tap opens enroll-seat every time. As a beat after the cold capture, the probe's rectangle was stale (the mark moved from y=63 to y=128 between frames) and the capture read nothing. Wanted: a reach that re-reads the rectangle at the moment of the tap, then the beat 'tap_control lernie; capture door enroll-seat' after the cold capture.