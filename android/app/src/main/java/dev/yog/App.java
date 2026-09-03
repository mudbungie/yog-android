package dev.yog;

import android.app.Activity;
import android.content.Context;

/**
 * The app's own two handles, and the one fact only the platform knows: is
 * this app in front right now.
 *
 * <h2>Why this exists at all (bl-f34f)</h2>
 *
 * The paper tools ({@link Paper}) run on the tool-host thread, which holds no
 * activity and must not — a tool whose availability tracked the UI would be
 * the wrong shape, and a tool that ran only while somebody was looking at the
 * phone would be no teleoperation at all. But two of the four need something
 * the host thread cannot have:
 *
 * <ul>
 *   <li>a {@link Context}, to reach the battery, the network, the clipboard
 *       and the notification manager. The APPLICATION context, deliberately:
 *       it outlives every activity, so a tool answered from it never holds a
 *       destroyed screen.</li>
 *   <li>the activity that is in front, or null. Android has refused an
 *       activity launch from a background app since API 29 and says nothing
 *       when it does — no exception, one line in logcat — so
 *       {@link Paper#open} must ask BEFORE it acts, and this is the only
 *       place the answer is known. Written by {@link MainActivity}'s own
 *       lifecycle, which is the platform's answer rather than this app's
 *       guess at it.</li>
 * </ul>
 *
 * All three fields are volatile: they are written on the UI thread and read
 * from the tool-host thread, and a stale read here is a tool acting on a
 * screen that has gone away.
 */
public final class App {
    private App() {}

    /** The prefix a successful answer carries, matching {@link InterfaceService#OK}. */
    static final String OK = "ok\n";

    /** The prefix a refusal carries. */
    static final String ERR = "err\n";

    /** The sentence a tool earns before this app's own activity has started. */
    static final String NO_CONTEXT =
            "this app has not finished starting: open yog on the device once, and call again.";

    private static volatile Context app;
    private static volatile Activity front;

    /** This app's process is up; hold the context that outlives every screen. */
    static void created(Activity activity) {
        app = activity.getApplicationContext();
    }

    /** This app is what the operator is looking at. */
    static void resumed(Activity activity) {
        front = activity;
    }

    /**
     * It is not any more — unless what paused is a screen we already replaced,
     * which is the ordinary order of a configuration change.
     */
    static void paused(Activity activity) {
        if (front == activity) {
            front = null;
        }
    }

    /** The application context, or null before the first activity started. */
    static Context context() {
        return app;
    }

    /** The activity in front, or null when this app is not on screen. */
    static Activity front() {
        return front;
    }
}
