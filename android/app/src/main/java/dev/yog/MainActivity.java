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
        // The pocketed foot (DESIGN §18). On resume for a second reason as
        // well: a foreground service may only be started from a user-visible
        // state (API 31+), and a resume IS that state. Re-starting one that is
        // already running is how the platform is told nothing changed, and a
        // device whose leaf is no longer a foot's has its hold stopped here.
        Pocket.arm(this);
    }

    @Override
    protected void onPause() {
        App.paused(this);
        super.onPause();
    }

    /**
     * <h2>The activity's destroy is this process's end (bl-be13)</h2>
     *
     * {@code GameActivity.onDestroy} calls {@code terminateNativeCode_native},
     * whose {@code NativeCode::~NativeCode} waits on a condition variable for
     * the app thread to return from {@code android_main} — and this app's app
     * thread never will. winit 0.30.13's android backend reads, in full:
     *
     * <pre>
     *   MainEvent::Destroy =&gt; {
     *       // XXX: maybe exit mainloop to drop things before being killed by the OS?
     *       warn!("TODO: forward onDestroy notification to application");
     *   },
     * </pre>
     *
     * so the destroy is dropped and the loop runs on. Nothing in this app can
     * take it from there either: winit dispatches {@code RedrawRequested} only
     * while its {@code running} flag is set, and that flag is cleared by the
     * {@code onPause} which always precedes a destroy — so no frame is painted
     * after the pause and no line of Rust in this crate is ever entered again.
     * What the app thread does instead is spin on
     * {@code android_app_input_available_wake_up}, logging three lines a
     * ~57&nbsp;ms iteration ("after GameActivity was destroyed"), forever.
     *
     * <p>Measured: the main thread parked in
     * {@code __futex_wait_ex &lt;- pthread_cond_wait &lt;- onDestroy &lt;-
     * NativeCode::~NativeCode &lt;- terminateNativeCode_native &lt;-
     * GameActivity.onDestroy &lt;- Activity.performDestroy}, indefinitely. A
     * main thread parked there answers nothing the platform sends it: the
     * activity cannot be created again, no binder call lands, and the first
     * thing that needs it ANRs the process and the platform SIGKILLs it.
     *
     * <p>So the process ends here, deliberately, rather than leaving a wedge
     * that is killed later and worse. The manifest's {@code configChanges} is
     * the other half: every configuration change this app can redraw is
     * absorbed in place, so this path is reached when the activity genuinely
     * goes — the back gesture's {@code finish()}, a task swiped away, a
     * reclaim, or the one configuration change no flag can absorb
     * ({@code assetsPaths}, which is what installing over a running copy is).
     * The exit is one line upstream — {@code MainEvent::Destroy} calling the
     * event loop's own {@code exit()} — and it belongs on the ledger of
     * upstream defects this client shims (bl-2958); until it lands, a foot
     * this process was holding for DESIGN §18 ends with the window, and the
     * operator's remedy is the one §18 already names: open the app.
     *
     * <p>{@code super.onDestroy()} is not reachable and is kept anyway: a kill
     * that somehow did not take must still leave the platform's own path
     * running rather than a method that returned without tearing anything
     * down.
     */
    @Override
    protected void onDestroy() {
        android.os.Process.killProcess(android.os.Process.myPid());
        super.onDestroy();
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
