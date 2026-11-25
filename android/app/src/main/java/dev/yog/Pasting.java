package dev.yog;

import android.content.ClipData;
import android.content.ClipboardManager;
import android.content.Context;
import android.view.View;
import android.view.inputmethod.BaseInputConnection;

import com.google.androidgamesdk.gametextinput.InputConnection;
import com.google.androidgamesdk.gametextinput.Settings;

/**
 * <h2>The keyboard's paste key reaches the egui fields (bl-95f2)</h2>
 *
 * games-activity 4.4.0's {@link InputConnection} does not override
 * {@code performContextMenuAction}, so it answers with
 * {@link BaseInputConnection}'s, which does nothing and returns false. That is
 * the call an IME's own paste key makes: the paste was refused before any of
 * this app saw it, while a clipboard suggestion chip — which the IME sends as
 * plain {@code commitText} — worked. Every field the GameTextInput mirror
 * serves (the sign-in's paste field, the registration door, the search
 * needle) had the same dead key; the composer is a real {@code EditText}
 * ({@link Field}) and never did.
 *
 * <p>So this connection answers the paste: the primary clip, coerced to text,
 * committed at the caret exactly as a chip would commit it. The read is the
 * focused app's to make — the platform allows a clipboard read to the app
 * holding input focus, which is precisely when an IME can ask for this.
 * {@code KEYCODE_PASTE}, the other way a keyboard can say paste, arrives as a
 * key event and is routed here by {@link MainActivity}'s key listener.
 *
 * <p>It is built where the stock one is built, with the same arguments
 * ({@link MainActivity#createSurfaceView}), so the native side is handed this
 * one and never knows the difference.
 */
public final class Pasting extends InputConnection {
    private final Context context;

    public Pasting(Context context, View view, Settings settings) {
        super(context, view, settings);
        this.context = context;
    }

    @Override
    public boolean performContextMenuAction(int id) {
        if (id == android.R.id.paste || id == android.R.id.pasteAsPlainText) {
            return paste(this, context);
        }
        return super.performContextMenuAction(id);
    }

    /**
     * Commit the clipboard's first item as text at the caret. False when there
     * is nothing to paste, which an IME reads as the action not taken.
     */
    static boolean paste(BaseInputConnection ic, Context context) {
        ClipboardManager board = context.getSystemService(ClipboardManager.class);
        ClipData clip = board == null ? null : board.getPrimaryClip();
        if (clip == null || clip.getItemCount() == 0) {
            return false;
        }
        CharSequence text = clip.getItemAt(0).coerceToText(context);
        return text != null && text.length() > 0 && ic.commitText(text, 1);
    }
}
