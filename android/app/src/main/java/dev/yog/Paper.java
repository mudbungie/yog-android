package dev.yog;

import android.content.ClipData;
import android.content.ClipboardManager;
import android.content.Context;

/**
 * The door the paper tools come in by (DESIGN §16.1, rung 1): four static
 * entry points, one per tool, and the platform work behind three of them in
 * {@link Device}, {@link Notify} and {@link Open} — four files for four
 * questions, so none of them is the file everything lands in.
 *
 * <h2>Paper because none of it needs a service</h2>
 *
 * No AccessibilityService to enable, no foreground service to run. Each
 * tool's whole price is what the OS grants an app uid, and each refusal names
 * the ONE operator act that fixes it — the corpus rule.
 *
 * <h2>The clipboard write, established rather than assumed</h2>
 *
 * The Android 10 restriction everyone quotes is on the clipboard READ:
 * {@code ClipboardService.clipboardAccessAllowed} reaches the focused-window
 * and default-IME tests only under {@code OP_READ_CLIPBOARD}, while its
 * {@code OP_WRITE_CLIPBOARD} arm is three lines — <i>"Writing is allowed
 * without focus"</i>, {@code allowed = true}, break — unchanged in every AOSP
 * branch from android10 to main. So the write below works from the tool-host
 * thread with this app nowhere near the screen. Two limits ride with it: a
 * denial is a bare {@code return} from a void binder call, so nothing throws
 * and nothing reports (the one denial left is the {@code WRITE_CLIPBOARD}
 * appop set to ignore, which defaults to allowed); and Android 13+ auto-clears
 * a clip after about an hour. There is deliberately no clipboard READ tool —
 * that one really is blocked, and ui_read is the honest alternative.
 *
 * <p>No Looper is needed either: {@code setPrimaryClip} is a plain binder
 * call, and the manager's handler has been injected from the main thread since
 * API 28 — the old <i>"Can't create handler inside thread…"</i> crash is an
 * API 27 artifact, and minSdk here is 28. The catch is insurance against an
 * OEM fork, not against AOSP.
 *
 * <h2>The contract with the Rust side</h2>
 *
 * {@link InterfaceService}'s two-line answer protocol exactly: every entry
 * point returns {@code "ok\n<payload>"} or {@code "err\n<sentence>"}, parsed
 * by one pure function the suite tests.
 */
public final class Paper {
    private Paper() {}

    /** What this device is doing right now, three lines of it. */
    public static String device() {
        Context ctx = App.context();
        if (ctx == null) {
            return App.ERR + App.NO_CONTEXT;
        }
        return App.OK + Device.state(ctx);
    }

    /** Put text on the clipboard, for the operator to paste anywhere. */
    public static String clipboardSet(String text) {
        Context ctx = App.context();
        if (ctx == null) {
            return App.ERR + App.NO_CONTEXT;
        }
        ClipboardManager clipboard = ctx.getSystemService(ClipboardManager.class);
        if (clipboard == null) {
            return App.ERR + "this device has no clipboard service.";
        }
        try {
            clipboard.setPrimaryClip(ClipData.newPlainText("yog", text));
        } catch (RuntimeException e) {
            return App.ERR + "this device refused the clipboard write: " + e;
        }
        return App.OK + "put " + text.length() + " characters on the clipboard";
    }

    /** Post a notification, or say which act would let one be posted. */
    public static String notify(String title, String text) {
        Context ctx = App.context();
        if (ctx == null) {
            return App.ERR + App.NO_CONTEXT;
        }
        return Notify.post(ctx, title, text);
    }

    /** Open a URL, or hand text to the share sheet. */
    public static String open(String kind, String value) {
        return Open.typed(kind, value);
    }
}
