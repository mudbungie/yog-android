package dev.yog;

import android.app.Activity;
import android.content.Context;
import android.net.ConnectivityManager;
import android.net.LinkAddress;
import android.net.LinkProperties;
import android.net.Network;

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

    /** Whether this process already follows the default network. */
    private static boolean following;

    /** This app's process is up; hold the context that outlives every screen. */
    static void created(Activity activity) {
        app = activity.getApplicationContext();
        follow(app);
    }

    /**
     * The default network's addresses, one per line, or nothing when there is
     * none (bl-792e): what a roving ladder's call names and the moment every
     * cache it keeps goes stale. Decided in Rust ({@code ladder::Network}).
     */
    private static native void network(String addresses);

    /**
     * Follow the default network for the life of the process, once — from
     * whichever of {@link MainActivity} and {@link Pocket} starts it first
     * (bl-792e, DESIGN §21.11). The platform's callback, not a periodic read
     * of the routing table, is when a change is known: measured, the read
     * learned of a wifi drop 34–53 s late. The addresses are the default
     * network's own link, not every interface's, so an overlay or an IMS
     * network never lends a call its v6.
     */
    static synchronized void follow(Context context) {
        if (following) {
            return;
        }
        ConnectivityManager cm = context.getSystemService(ConnectivityManager.class);
        if (cm == null) {
            return;
        }
        following = true;
        cm.registerDefaultNetworkCallback(new ConnectivityManager.NetworkCallback() {
            @Override
            public void onAvailable(Network n) {
                report(cm.getLinkProperties(n));
            }

            @Override
            public void onLinkPropertiesChanged(Network n, LinkProperties lp) {
                report(lp);
            }

            @Override
            public void onLost(Network n) {
                network("");
            }
        });
    }

    /** One report: every address on the link, as the platform spells it. */
    private static void report(LinkProperties lp) {
        StringBuilder out = new StringBuilder();
        if (lp != null) {
            for (LinkAddress address : lp.getLinkAddresses()) {
                out.append(address.getAddress().getHostAddress()).append('\n');
            }
        }
        network(out.toString());
    }

    /**
     * Whether a roving ladder may climb, as far as the screen goes (bl-c21d):
     * a backgrounded app has no DNS and no path, so a climb there spends the
     * re-punch on nothing. Decided in Rust ({@code ladder::Awake}), beside
     * the pocket service's own report.
     */
    private static native void foreground(boolean front);

    /**
     * {@link Pocket} started (or stopped) holding the process in the
     * foreground — above the platform's network threshold, so the foot and
     * the lane may climb while the activity is behind (bl-c21d). Decided in
     * Rust beside {@link #foreground}.
     */
    static native void pocketed(boolean holding);

    /** This app is what the operator is looking at. */
    static void resumed(Activity activity) {
        front = activity;
        foreground(true);
    }

    /**
     * It is not any more — unless what paused is a screen we already replaced,
     * which is the ordinary order of a configuration change.
     */
    static void paused(Activity activity) {
        if (front == activity) {
            front = null;
            foreground(false);
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
