+++
title = "the attention mark paints as tofu: the bundled font set has no glyph for the roster's dot"
created = 1788582619
updated = 1788582619
priority = 3
root_commit = "b8421205e882caeadc666ccff26464e4e0f60dda"
+++
Seen in the `make screens` walk's own evidence (02-roster.png, bl-35bd's run): every place this app paints the attention mark — `screens.rs`'s workspace rows (`" ●"`, U+25CF BLACK CIRCLE) and now the roster's queue entry beside them — renders as a hollow box, the missing-glyph square. So the one mark that says *something is waiting on you* reads as a font failure, on the screen an operator lands on.

The cause is the bundled set, not the paint: egui's default fonts are Ubuntu-Light (proportional), Hack (monospace) and two emoji faces, and none of them carries that codepoint — the same font-set fact bl-7355 recorded from the other direction (no bold face, and a `.ttf` cannot enter this tree because the disclosure gate refuses every binary it cannot read).

Two answers, and neither needs a dependency or a committed binary:

- **Pick a mark the bundled faces DO carry.** A word, a bullet the emoji face has, or an ASCII shape. It is one constant, it is testable by eye in the walk's own picture, and it costs nothing.
- **Load a platform face at boot** (`/system/fonts`), which is bl-7355's open ruling question and a bigger act — it changes the app's typeface story and is device-only, so the host suite cannot see it.

The first is the p3; the second is bl-7355's ruling to take. Whichever is chosen, the check is the walk: the mark is in 02-roster.png and in the conversation list, so a picture says whether it painted.