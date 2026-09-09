+++
title = "paste into the composer is unproven: Gboard's clipboard chip and KEYCODE_PASTE did not reach the text mirror on the emulator; verify on a real device and fix the IME bridge if it reproduces"
created = 1788935360
updated = 1788935360
priority = 3
root_commit = "b8421205e882caeadc666ccff26464e4e0f60dda"
tags = ["usability-r3"]
+++
From bl-7781's landing (227a205): copy-out via long press is proven (system clip preview + Gboard list), but pasting INTO the message field did not work on the AVD — Gboard's clipboard feature was off, and a raw KEYCODE_PASTE did not reach the GameActivity text mirror. Re-drive on the operator's phone (long-press the field → Paste); if it fails, the IME bridge must accept the paste action (commitText / the clipboard content) into the mirror. Also the composer should accept a long-press → Paste menu the way every Android field does.