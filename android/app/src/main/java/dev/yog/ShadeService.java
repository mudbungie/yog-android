package dev.yog;

import android.app.NotificationManager;
import android.content.ComponentName;
import android.content.Context;
import android.service.notification.NotificationListenerService;
import android.service.notification.StatusBarNotification;
import java.util.Arrays;
import java.util.Comparator;

/**
 * The shade half of this device's tool hosting (DESIGN §16.1, rung 2): the
 * notifications this phone is currently showing, read as text.
 *
 * <h2>This is the SMS-adjacent surface, and it costs one settings act</h2>
 *
 * What an agent wants when it asks to read a phone's messages is what the
 * phone was TOLD — a two-factor code, a message, an alert — and all of it
 * arrives in the shade as some app's notification text. {@code READ_SMS} and
 * {@code SEND_SMS} are hard-restricted permissions and are refused as a design
 * shape; notification access answers the read want at one enable instead.
 *
 * <h2>Enabling it is the operator's act, and that IS the consent gate</h2>
 *
 * The app never grants itself anything (§16.1's consent surface, and
 * {@link InterfaceService}'s trust model unchanged). The operator turns this on
 * under Settings &gt; Apps &gt; Special app access &gt; Notification access, or
 * an already-trusted device turns it on over the physically attached debug
 * bridge — the same channel that carries this seat's key material. Two things
 * follow from the platform's own shape and both are stated where they are met:
 * the grant is all-or-nothing (a listener sees every notification on the device
 * or none), so there is deliberately no per-app filter in this app that would
 * advertise a narrowing the OS does not enforce; and a sideloaded build meets
 * Android's restricted-settings block, which presents as a toggle that will not
 * stick rather than as a refusal.
 *
 * <h2>Nothing is retained, and that is a ruling rather than an omission</h2>
 *
 * This class overrides neither {@code onNotificationPosted} nor
 * {@code onNotificationRemoved}: it holds no history, writes no file, and logs
 * nothing — logcat is device-wide and a shade is exactly the material that must
 * not go there. Every answer is {@code getActiveNotifications} read at the
 * moment of the call, because the platform already holds the shade and a copy
 * would be a second store of one fact, durable, on a device where nothing
 * sweeps it. The cost is stated in the tool's own description: what was
 * dismissed is gone, and this cannot answer what arrived while nobody asked.
 *
 * <h2>The contract with the Rust side</h2>
 *
 * {@link InterfaceService}'s two-line answer protocol exactly, parsed by one
 * pure function the suite tests.
 */
public class ShadeService extends NotificationListenerService {
    /**
     * The connected listener, or null when the operator has not enabled
     * notification access — or has, and the platform has not bound us yet.
     * Written on the platform's own connect/disconnect callbacks and read from
     * the tool-host thread, so volatile: a stale read here is a call against a
     * listener that has gone away.
     */
    private static volatile ShadeService live;

    @Override
    public void onListenerConnected() {
        live = this;
    }

    @Override
    public void onListenerDisconnected() {
        live = null;
    }

    /** The act that turns this on, named wherever a refusal is earned. */
    private static final String ENABLE_ACT =
            "enable \"yog\" under Settings > Apps > Special app access > Notification access, "
                    + "or have a trusted device enable it over the debug bridge with "
                    + "`cmd notification allow_listener dev.yog/dev.yog.ShadeService`";

    /**
     * The second act, and the reason this sentence is long: on a sideloaded
     * build the enable silently reverts, which reads as a broken toggle rather
     * than as a refusal, and an operator who is not told about it will try the
     * first act twice.
     */
    private static final String RESTRICTED_ACT =
            "if the toggle will not stay on, that is Android's restricted-settings block on a "
                    + "sideloaded app: clear it over the same cable with `appops set dev.yog "
                    + "ACCESS_RESTRICTED_SETTINGS allow` and enable it again.";

    /** The shade, as text, or the one act that would let it be read. */
    static String read(Context ctx, int limit) {
        ShadeService service = live;
        if (service == null) {
            return App.ERR + (granted(ctx) ? notBound() : notEnabled());
        }
        StatusBarNotification[] shade;
        try {
            shade = service.getActiveNotifications();
        } catch (RuntimeException e) {
            return App.ERR + "this device refused the shade read: " + e;
        }
        if (shade == null || shade.length == 0) {
            return App.OK + "the shade is empty: this device is showing no notifications.\n";
        }
        Arrays.sort(shade, Comparator.comparingLong(StatusBarNotification::getPostTime).reversed());
        int shown = Math.min(limit, shade.length);
        StringBuilder out = new StringBuilder(said(shade.length, shown));
        long now = System.currentTimeMillis();
        for (int i = 0; i < shown; i++) {
            out.append('\n').append(Notice.of(shade[i], now));
        }
        return App.OK + out;
    }

    /** The header: how many there are, and whether this is all of them. */
    private static String said(int held, int shown) {
        String count = held + (held == 1 ? " notification" : " notifications");
        if (shown < held) {
            return count + ", showing the newest " + shown + "\n";
        }
        return count + ", newest first\n";
    }

    /**
     * Whether the operator has granted notification access to THIS component.
     * Read rather than remembered: the operator may revoke it between two
     * calls, and a cached answer would send a caller to fix something that is
     * already fixed.
     */
    private static boolean granted(Context ctx) {
        NotificationManager manager = ctx.getSystemService(NotificationManager.class);
        return manager != null
                && manager.isNotificationListenerAccessGranted(
                        new ComponentName(ctx, ShadeService.class));
    }

    /** The sentence a read earns when the access was never granted. */
    static String notEnabled() {
        return "yog may not read this device's notifications: " + ENABLE_ACT + ". " + RESTRICTED_ACT;
    }

    /**
     * The sentence a read earns when the access IS granted and the listener is
     * not bound — a real state, met right after the enable and after the system
     * restarts the service. Its own sentence because the act that fixes it is
     * not the act above: waiting, or toggling what is already on.
     */
    static String notBound() {
        return "notification access is granted, but this device has not connected the listener "
                + "yet: Android binds it within moments of the enable, so call again. If it "
                + "never connects, toggle notification access off and on under Settings > Apps "
                + "> Special app access > Notification access.";
    }
}
