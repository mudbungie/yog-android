package dev.yog;

import android.Manifest;
import android.app.Activity;
import android.content.Context;
import android.content.pm.PackageManager;

/**
 * The camera tool's three gates, in the order the platform applies them: may
 * we, is anyone looking, and is the camera already in use (DESIGN §16.1).
 *
 * <p>The capture itself is {@link Shot}'s — the same split {@link Camera} and
 * {@link Session} already make for the enrollment scanner, and for the same
 * reason: a permission question and a camera2 lifecycle are two questions.
 *
 * <h2>Foreground, and why it is a refusal rather than a try</h2>
 *
 * Android has refused the camera to a process that is not in front since
 * Android 9, and what a caller gets back is a failed open on a callback long
 * after the call returned. Asking {@link App#front} first turns that into one
 * sentence naming the act — bring yog to the screen — which is the difference
 * between a priced capability and a decoy (§16.1's corpus rule).
 *
 * <h2>The scanner holds the same camera</h2>
 *
 * The enrollment scan screen streams this device's camera (bl-d815). Opening
 * it a second time from here would evict that session and leave an operator
 * staring at a dead preview mid-enrollment, so a scan in progress is a refusal
 * naming the one act that clears it. It is only reachable at all because a
 * still needs this app in front, which is exactly when the scan screen might
 * be up.
 *
 * <h2>The grant, and the one act that fixes a refusal</h2>
 *
 * The dialog goes up once per run when the app is in front (the bl-d815
 * permission-result hook, routed by {@link MainActivity} on this class's own
 * request id — {@link Camera}'s id is the scanner's and stays the scanner's,
 * so a tool's answer can never be read as an answer to the scan screen's ask).
 * After that, and whenever nothing is on screen, the sentence names the
 * settings act instead, which is the act that works either way — Android stops
 * showing the dialog after one refusal, and a sentence about a dialog that
 * never appears teaches nothing.
 */
final class Still {
    private Still() {}

    /** This class's permission-request id; {@link Camera} and {@link Notify} hold the others. */
    static final int REQUEST = 0x0B;

    /** The one act that fixes a camera refusal, wherever it is met. */
    private static final String SETTINGS_ACT =
            "turn Camera on for yog under Settings > Apps > yog > Permissions, then call again.";

    /** Whether the system's dialog has been answered this run. */
    private static volatile boolean answered;

    /** One still, or the sentence naming what would let one be taken. */
    static String capture(Context ctx, String lens, String path) {
        Activity front = App.front();
        if (front == null) {
            return App.ERR
                    + "Android refuses the camera to an app that is not on screen, so nothing "
                    + "was photographed: open yog on the device — the notify tool can ask the "
                    + "operator to — then call again.";
        }
        if (ctx.checkSelfPermission(Manifest.permission.CAMERA)
                != PackageManager.PERMISSION_GRANTED) {
            return App.ERR + ask(front);
        }
        if (Session.busy()) {
            return App.ERR
                    + "the enrollment scanner is using this device's camera: leave the scan "
                    + "screen on the phone, then call again.";
        }
        return new Shot(lens, path).take(ctx);
    }

    /**
     * The dialog has been answered, handed over by {@link MainActivity}. WHAT
     * it was answered is not read: {@code checkSelfPermission} above is the
     * standing truth. All this records is that the one showing is spent.
     */
    static void answered() {
        answered = true;
    }

    /** The sentence a refusal carries — and the raise itself, where one is possible. */
    private static String ask(Activity front) {
        if (answered) {
            return "this app may not use this device's camera: " + SETTINGS_ACT;
        }
        front.runOnUiThread(
                () ->
                        front.requestPermissions(
                                new String[] {Manifest.permission.CAMERA}, REQUEST));
        return "this app may not use this device's camera yet: Android's own permission dialog "
                + "has just been raised on the device — grant it there and call again, or "
                + SETTINGS_ACT;
    }
}
