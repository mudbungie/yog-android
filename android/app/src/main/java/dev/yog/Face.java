package dev.yog;

import android.graphics.drawable.GradientDrawable;
import android.graphics.drawable.StateListDrawable;
import android.util.TypedValue;
import android.widget.EditText;

/**
 * <h2>The visual language, applied to a platform view (bl-8bbb)</h2>
 *
 * {@link Field} is what the composer DOES per frame; this is what it LOOKS
 * like, and the seam is the one {@code src/theme.rs} and the screens already
 * keep. Every number here arrives as an argument: the language has one home
 * (docs/STYLE.md) and neither of these classes spells a colour, a size or a
 * corner of its own.
 *
 * <p>The face is STYLE.md's own — a {@code SURFACE} field with no stroke at
 * rest and the brand ring when it holds the caret, its words in {@code INK}
 * and its hint in {@code INK_FAINT}. The ring's WIDTH is the hairline's, one
 * point, because that is the most a boundary may be on this glass; the brand
 * is what a focused field changes, never the weight.
 */
final class Face {
    private Face() {}

    /** Dress a field in the language: its ink, its ground and its ring. */
    static void dress(
            EditText field, int ground, int ink, int faint, int ring, int radius,
            int padX, int padY, int textPx, int ringPx) {
        field.setTextColor(ink);
        field.setHintTextColor(faint);
        field.setTextSize(TypedValue.COMPLEX_UNIT_PX, textPx);
        field.setPadding(padX, padY, padX, padY);
        StateListDrawable states = new StateListDrawable();
        states.addState(
                new int[] {android.R.attr.state_focused}, block(ground, ring, radius, ringPx));
        states.addState(new int[] {}, block(ground, 0, radius, 0));
        field.setBackground(states);
    }

    /** One face: the ground, rounded, ringed when a stroke is asked for. */
    private static GradientDrawable block(int ground, int ring, int radius, int ringPx) {
        GradientDrawable face = new GradientDrawable();
        face.setColor(ground);
        face.setCornerRadius(radius);
        if (ringPx > 0) {
            face.setStroke(ringPx, ring);
        }
        return face;
    }
}
