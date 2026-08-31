package dev.yog;

import android.Manifest;
import android.app.Activity;
import android.content.pm.PackageManager;

/**
 * The camera, for the one thing this app points it at: reading the enroll
 * envelope off a screen (bl-d815).
 *
 * <h2>What this class is</h2>
 *
 * The permission, and the door. The camera2 lifecycle is {@link Session}'s and
 * the pixels are {@link Frames}' — three files for three questions (may we,
 * which device, what bytes), so none of them is the file everything lands in.
 *
 * <h2>camera2 into an ImageReader, and no preview surface</h2>
 *
 * The session has exactly one output and it is an {@code ImageReader}. There is
 * no {@code SurfaceView}, no {@code TextureView} and no {@code SurfaceTexture}
 * handed to wgpu, because the Rust side paints the preview from the very buffer
 * it decodes — so the whole class of "the preview works but the decoder sees
 * something else" defects cannot arise, and the egui frame loop keeps the one
 * surface it already owns. That is also why CameraX is not here: its value is
 * the preview/analysis plumbing this design does not have.
 *
 * <h2>The contract with the Rust side</h2>
 *
 * {@link InterfaceService}'s two-line answer protocol, one bridge over: every
 * entry point returns a String, and {@link #state} answers one of four words or
 * {@code "err\n<sentence>"}. The camera's own asynchronous failures — a device
 * that will not open, a session the framework refused — are folded into that
 * same answer rather than polled separately, because the scan screen asks one
 * question and a second place to say no is a second place to get it wrong.
 */
public final class Camera {
    private Camera() {}

    /** The prefix a refusal carries, matching {@link InterfaceService#ERR}. */
    static final String ERR = "err\n";

    /** The permission request's own id; nothing else in this app requests one. */
    private static final int REQUEST = 0xCA;

    /**
     * Whether the dialog has been raised, and whether it has been answered.
     * Written on the UI thread, read from the native frame loop — the two
     * together are what separate "the operator is looking at the dialog" from
     * "the operator said no", which the platform's own check cannot tell apart.
     */
    private static volatile boolean asked;
    private static volatile boolean answered;

    /** Whether this device can scan right now, in one word. */
    public static String state(Activity activity) {
        if (activity == null) {
            return ERR + "the app has no activity to ask with";
        }
        String said = Session.problem();
        if (said != null) {
            return ERR + said;
        }
        if (granted(activity)) {
            return "granted";
        }
        if (answered) {
            return "denied";
        }
        return asked ? "asking" : "unasked";
    }

    /** Put the system's permission dialog up, once. */
    public static String ask(Activity activity) {
        if (activity == null) {
            return ERR + "the app has no activity to ask with";
        }
        asked = true;
        answered = false;
        activity.runOnUiThread(
                () ->
                        activity.requestPermissions(
                                new String[] {Manifest.permission.CAMERA}, REQUEST));
        return "ok\n";
    }

    /** The operator's answer, handed over by {@link MainActivity}. */
    static void answered(int[] grants) {
        answered = true;
        if (grants != null && grants.length > 0 && grants[0] == PackageManager.PERMISSION_GRANTED) {
            asked = false;
        }
    }

    /**
     * Open the back camera and start filling frames. Returns as soon as the
     * open is requested — the device arrives on a callback, and until it does
     * {@link #frame} simply answers null.
     */
    public static String start(Activity activity) {
        if (activity == null) {
            return ERR + "the app has no activity to open the camera with";
        }
        if (!granted(activity)) {
            return ERR + "the camera permission is not held";
        }
        return Session.open(activity);
    }

    /** The newest frame nobody has read yet, or null. */
    public static byte[] frame() {
        return Session.frame();
    }

    /** Shut everything down. Safe to call when nothing is open. */
    public static String stop() {
        return Session.close();
    }

    private static boolean granted(Activity activity) {
        return activity.checkSelfPermission(Manifest.permission.CAMERA)
                == PackageManager.PERMISSION_GRANTED;
    }
}
