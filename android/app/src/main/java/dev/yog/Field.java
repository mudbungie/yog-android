package dev.yog;

import android.app.Activity;
import android.content.Context;
import android.text.Editable;
import android.text.Layout;
import android.text.TextWatcher;
import android.view.Gravity;
import android.view.View;
import android.view.ViewGroup;
import android.view.inputmethod.EditorInfo;
import android.view.inputmethod.InputMethodManager;
import android.widget.EditText;
import android.widget.FrameLayout;

/**
 * <h2>The composer is a real Android text field (bl-8bbb)</h2>
 *
 * The message field used to be an egui {@code TextEdit} fed by the
 * GameTextInput mirror, which adopts the IME's committed buffer wholesale — so
 * a letter typed with the cursor in the middle of a draft landed at the end,
 * nothing was selectable, and there was no paste menu. The ruling's answer is
 * this: ONE native {@link EditText}, overlaid at the rectangle egui lays out
 * for the composer, whose text and cursor are the one home of the draft.
 * Cursor placement, selection handles, copy and paste, autocorrect and the
 * IME's own behaviour then come from the platform rather than from a mirror.
 *
 * <p>It is added to {@code android.R.id.content}, so it is a SIBLING drawn
 * after GameActivity's own frame. A {@code SurfaceView} renders below the
 * window and GameActivity never calls {@code setZOrderOnTop}, so an ordinary
 * view above it in the hierarchy composites over the GL content with nothing
 * else to arrange.
 *
 * <p><b>Every field here is volatile and every view touch is posted.</b> The
 * Rust side calls from the app's render thread, which is not the UI thread and
 * may not touch a View; what it reads back are three volatiles a UI-thread
 * listener writes. The one exception is {@link #give}, which writes the text
 * mirror on the CALLING thread before it posts — so the very next frame reads
 * what this side handed over rather than the text the field still shows for a
 * few milliseconds. That is the echo window the GameTextInput mirror has to
 * carry and this one does not.
 *
 * <p>The whole style arrives from {@code src/theme.rs} through the call: the
 * visual language has one home (docs/STYLE.md) and this class spells no
 * colour, size or corner of its own. {@link Face} dresses the view with it.
 */
public final class Field {
    private Field() {}

    /** The sentence a call earns before this app's own activity has started. */
    private static final String NO_ACTIVITY = "the composer field has no activity";

    /** The overlaid field, once the first placement has built it. */
    private static volatile EditText view;

    /** What the field holds, mirrored for the render thread. */
    private static volatile String held = "";

    /** Whether the field holds the caret. */
    private static volatile boolean caret;

    /** The height the field's own text wants, in pixels. */
    private static volatile int high;

    /**
     * The last rectangle, hint and style asked for. An unchanged frame posts
     * only the measurement below — re-seating layout parameters sixty times a
     * second would fight the platform's own layout for no change at all.
     */
    private static volatile String asked = "";

    /**
     * Place the field at a rectangle in device pixels, dress it, and answer
     * what it holds: {@code ok}, then {@code 1} or {@code 0} for the caret,
     * then the height its text wants, then the draft — which may carry
     * newlines, and so is everything after the third line.
     */
    public static String place(
            Activity activity, String hint, int left, int top, int width, int height,
            int ground, int ink, int faint, int ring, int radius,
            int padX, int padY, int textPx, int ringPx) {
        if (activity == null) {
            return App.ERR + NO_ACTIVITY;
        }
        String want = hint + "|" + left + "," + top + "," + width + "," + height
                + "|" + ground + "," + ink + "," + faint + "," + ring + "," + radius
                + "," + padX + "," + padY + "," + textPx + "," + ringPx;
        if (want.equals(asked)) {
            // The field re-lays out after a change, so the height its text
            // wants is only readable on a LATER frame than the one that
            // changed it — which is why this runs every frame and not only
            // from the text watcher.
            activity.runOnUiThread(Field::measure);
        } else {
            asked = want;
            activity.runOnUiThread(() -> seat(activity, hint, left, top, width, height,
                    ground, ink, faint, ring, radius, padX, padY, textPx, ringPx));
        }
        return App.OK + (caret ? "1" : "0") + "\n" + high + "\n" + held;
    }

    /**
     * Take the field off the glass, and the keyboard with it. The view and its
     * text are KEPT: the draft outlives a screen change, and a field that came
     * back empty would read to the mirror as the operator having cleared it.
     */
    public static String hide(Activity activity) {
        if (activity == null) {
            return App.ERR + NO_ACTIVITY;
        }
        asked = "";
        activity.runOnUiThread(() -> {
            if (view == null || view.getVisibility() != View.VISIBLE) {
                return;
            }
            view.setVisibility(View.GONE);
            InputMethodManager imm = (InputMethodManager)
                    view.getContext().getSystemService(Context.INPUT_METHOD_SERVICE);
            if (imm != null) {
                imm.hideSoftInputFromWindow(view.getWindowToken(), 0);
            }
        });
        return App.OK + "hidden";
    }

    /**
     * Hand the field a draft this side wrote — a send clearing it, a refused
     * deposit giving it back, a menu spending it. It takes the caret back when
     * it is on the glass, because in every one of those cases what the
     * operator does next is type.
     */
    public static String give(Activity activity, String text) {
        held = text == null ? "" : text;
        if (activity == null) {
            return App.ERR + NO_ACTIVITY;
        }
        final String want = held;
        activity.runOnUiThread(() -> {
            if (view == null) {
                return;
            }
            view.setText(want);
            view.setSelection(want.length());
            if (view.getVisibility() == View.VISIBLE) {
                view.requestFocus();
            }
        });
        return App.OK + "given";
    }

    /** Build the field on first use, then dress, place and show it. */
    private static void seat(
            Activity activity, String hint, int left, int top, int width, int height,
            int ground, int ink, int faint, int ring, int radius,
            int padX, int padY, int textPx, int ringPx) {
        if (view == null && !build(activity)) {
            return;
        }
        EditText field = view;
        field.setHint(hint);
        Face.dress(field, ground, ink, faint, ring, radius, padX, padY, textPx, ringPx);
        FrameLayout.LayoutParams at =
                new FrameLayout.LayoutParams(width, height, Gravity.TOP | Gravity.START);
        at.leftMargin = left;
        at.topMargin = top;
        field.setLayoutParams(at);
        field.setVisibility(View.VISIBLE);
        measure();
    }

    /** Read back the height the field's own text wants, after a layout. */
    private static void measure() {
        EditText field = view;
        if (field == null) {
            return;
        }
        Layout laid = field.getLayout();
        if (laid != null) {
            high = laid.getHeight() + field.getPaddingTop() + field.getPaddingBottom();
        }
    }

    /**
     * The field itself, added over the GL surface. False when there is no
     * content frame to add it to, which is a window that has not been set up.
     */
    private static boolean build(Activity activity) {
        View content = activity.findViewById(android.R.id.content);
        if (!(content instanceof ViewGroup)) {
            return false;
        }
        EditText field = new EditText(activity);
        field.setInputType(EditorInfo.TYPE_CLASS_TEXT
                | EditorInfo.TYPE_TEXT_FLAG_MULTI_LINE
                | EditorInfo.TYPE_TEXT_FLAG_CAP_SENTENCES
                | EditorInfo.TYPE_TEXT_FLAG_AUTO_CORRECT
                | EditorInfo.TYPE_TEXT_VARIATION_SHORT_MESSAGE);
        field.setImeOptions(EditorInfo.IME_FLAG_NO_FULLSCREEN);
        field.setGravity(Gravity.TOP | Gravity.START);
        field.setVerticalScrollBarEnabled(true);
        field.setText(held);
        field.setSelection(held.length());
        field.addTextChangedListener(new TextWatcher() {
            @Override
            public void beforeTextChanged(CharSequence s, int at, int count, int after) {}

            @Override
            public void onTextChanged(CharSequence s, int at, int before, int count) {}

            @Override
            public void afterTextChanged(Editable s) {
                held = s.toString();
            }
        });
        field.setOnFocusChangeListener((v, focused) -> caret = focused);
        ((ViewGroup) content).addView(field);
        view = field;
        return true;
    }
}
