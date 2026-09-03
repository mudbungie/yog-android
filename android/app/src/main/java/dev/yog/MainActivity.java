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
 * <h2>The other overrides (bl-d815, bl-f34f)</h2>
 *
 * {@code onRequestPermissionsResult} is the only signal that separates "the
 * operator is looking at the dialog" from "the operator said no" — the
 * platform's own {@code checkSelfPermission} answers DENIED for both, and a
 * scan screen that cannot tell them apart either spins forever or gives up on
 * a dialog still on screen. It routes on the request code, because four
 * classes ask now: {@link Camera} for the enrollment scanner, {@link Notify}
 * for the notification tool, and {@link Still} and {@link Fix} for the sighted
 * pair (bl-b0a9). A class that owns an ask owns the constant that names its
 * answers — the scanner's id stays the scanner's, so a tool's dialog can never
 * be read as an answer to the screen's.
 *
 * <p>{@code onCreate}/{@code onResume}/{@code onPause} hand {@link App} the
 * two things a tool running on the host thread cannot get for itself: this
 * app's context, and whether it is in front. The second is not a nicety —
 * Android has refused an activity launch from a background app since API 29
 * and reports nothing when it does, so the {@code open} tool must be able to
 * ask before it acts.
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
    public void onRequestPermissionsResult(
            int request, String[] permissions, int[] grants) {
        super.onRequestPermissionsResult(request, permissions, grants);
        if (request == Camera.REQUEST) {
            Camera.answered(grants);
        } else if (request == Notify.REQUEST) {
            Notify.answered();
        } else if (request == Still.REQUEST) {
            Still.answered();
        } else if (request == Fix.REQUEST) {
            Fix.answered();
        }
    }

    @Override
    protected void onResume() {
        super.onResume();
        App.resumed(this);
        // The scheduled fetch (DESIGN §17). On resume rather than on create,
        // because the resume AFTER the notification dialog is answered is the
        // first moment the grant can be read — and re-scheduling an identical
        // job is how JobScheduler is told nothing changed.
        Watch.arm(this);
    }

    @Override
    protected void onPause() {
        App.paused(this);
        super.onPause();
    }

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        App.created(this);
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
