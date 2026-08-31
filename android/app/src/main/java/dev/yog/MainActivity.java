package dev.yog;

import android.content.Context;
import android.os.Bundle;
import android.text.Editable;
import android.text.Selection;
import android.view.KeyEvent;
import android.view.View;
import android.view.inputmethod.BaseInputConnection;
import android.view.inputmethod.InputMethodManager;

import com.google.androidgamesdk.GameActivity;
import com.google.androidgamesdk.GtiAccess;
import com.google.androidgamesdk.gametextinput.InputConnection;

/**
 * The whole Java surface: GameActivity brings the InputConnection
 * (GameTextInput); the Rust side is everything else — plus the one repair
 * below.
 *
 * <h2>Why the key listener is replaced (bl-014e; upstream report tracked as
 * bl-2958)</h2>
 *
 * Long-pressing backspace on Gboard deleted exactly one character, however
 * long the press. It is not a repeat-rate problem and no amount of editor
 * state fixes it. games-activity 4.4.0's
 * {@code gametextinput.InputConnection#onKey} reads, in effect:
 *
 * <pre>
 *   if (!getSoftKeyboardActive()) return false;
 *   if (processKeyEvent(event)) {
 *       stateUpdated();
 *       immUpdateSelection();
 *       restartInput();          // &lt;-- the defect
 *       return true;
 *   }
 * </pre>
 *
 * {@code InputMethodManager.restartInput} tears the InputConnection down and
 * builds a new one — visible in logcat as {@code closeConnection} — and Gboard
 * sends backspace as {@code KEYCODE_DEL} through {@code sendKeyEvent}, so the
 * very first delete destroys the connection its repeat loop is running
 * against. The loop stops; every later press pays the same price. A plain
 * {@code EditText} never restarts input on a keystroke.
 *
 * We cannot edit that class, but the listener it installs on the surface view
 * is replaceable: the InputConnection registers itself with
 * {@code targetView.setOnKeyListener(this)} in its constructor, so a listener
 * installed afterwards wins. Ours handles KEYCODE_DEL with the same public
 * calls the original reaches privately — delete, notify the app, tell the IME
 * where the cursor went — and simply omits the restart. Every other key is
 * handed straight back to the stock listener, whose restart is harmless for a
 * key that does not repeat.
 */
public class MainActivity extends GameActivity {
    static {
        System.loadLibrary("yog_android");
    }

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        final InputConnection ic = GtiAccess.inputConnection(mSurfaceView);
        if (ic == null) {
            return;
        }
        final InputMethodManager imm =
                (InputMethodManager) getSystemService(Context.INPUT_METHOD_SERVICE);
        mSurfaceView.setOnKeyListener(new View.OnKeyListener() {
            @Override
            public boolean onKey(View v, int keyCode, KeyEvent event) {
                if (keyCode != KeyEvent.KEYCODE_DEL || !ic.getSoftKeyboardActive()) {
                    return ic.onKey(v, keyCode, event);
                }
                if (event.getAction() == KeyEvent.ACTION_DOWN) {
                    Editable ed = ic.getEditable();
                    int start = Selection.getSelectionStart(ed);
                    int end = Selection.getSelectionEnd(ed);
                    if (start != end) {
                        // A ranged selection or a live composing region is the
                        // stock path's business; it happens once, not in a
                        // repeat, so the restart it does costs nothing.
                        return ic.onKey(v, keyCode, event);
                    }
                    ic.deleteSurroundingTextInCodePoints(1, 0);
                    // endBatchEdit() is what calls the library's private
                    // stateUpdated(), which is what wakes the native side.
                    ic.endBatchEdit();
                    if (imm != null) {
                        ed = ic.getEditable();
                        imm.updateSelection(
                                v,
                                Selection.getSelectionStart(ed),
                                Selection.getSelectionEnd(ed),
                                BaseInputConnection.getComposingSpanStart(ed),
                                BaseInputConnection.getComposingSpanEnd(ed));
                    }
                }
                // Swallow both halves of the press: no restartInput, and no
                // second delete from the native key path either.
                return true;
            }
        });
    }
}
