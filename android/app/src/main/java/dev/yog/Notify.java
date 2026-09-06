package dev.yog;

import android.Manifest;
import android.app.Activity;
import android.app.Notification;
import android.app.NotificationChannel;
import android.app.NotificationManager;
import android.app.PendingIntent;
import android.content.Context;
import android.content.Intent;
import android.content.pm.PackageManager;
import android.os.Build;
import android.os.PowerManager;
import java.util.concurrent.atomic.AtomicInteger;

/**
 * The app's ONE notification mechanism: the permission, the channels, and the
 * post. Two callers and no second copy — the {@code notify} tool an agent
 * spends (DESIGN §16.1) and the scheduled fetch (DESIGN §17, {@link Watch}).
 *
 * <h2>The grant, and the one act that fixes a refusal</h2>
 *
 * {@code POST_NOTIFICATIONS} is a runtime permission on API 33+, and the
 * operator may also turn this app's notifications off in settings on any
 * version. {@code areNotificationsEnabled} answers both at once and is
 * therefore the only check here — a permission result kept beside it would be
 * a second copy of a fact that moves.
 *
 * <p>When it answers no and this app is in front, the system's own dialog goes
 * up (the bl-d815 permission-result hook, routed by {@link MainActivity} on
 * this class's own request id) and the invocation refuses saying so. Asking is
 * once per run: Android stops showing the dialog after a refusal, so a second
 * ask would be a sentence about a dialog that never appears. Every other path
 * names the settings act instead, which is the act that works even when
 * nothing is on screen.
 *
 * <h2>Two channels, because the operator may want one and not the other</h2>
 *
 * A tool's notification is an agent reaching the operator's pocket; the
 * attention channel is the seat's own machinery saying a workspace wants
 * them. Separate channels put the choice between them in Android's own
 * settings, per channel, which is where a capability's severability belongs
 * (§16.1's refused per-tool toggle screen, for its reason: a second authority
 * beside the OS grant drifts the first time one of them is revoked). The
 * attention channel's description states what leaving it on costs, because
 * that switch is also the fetch's own off switch — see {@link Watch}.
 */
final class Notify {
    private Notify() {}

    /** This class's permission-request id; {@link Camera} holds the other. */
    static final int REQUEST = 0x0A;

    /** The channel a tool-posted notification lands on. */
    static final String TOOLS = "yog.tools";

    /** The channel the scheduled fetch posts on. */
    static final String ATTENTION = "yog.attention";

    /** The channel the pocketed foot's standing notification lives on. */
    static final String FOOT = "yog.foot";

    /**
     * The channel the HELD attention lane's standing notification lives on
     * (DESIGN §17.6). Its own rather than {@link #ATTENTION}'s because the two
     * say different things and an operator may want one and not the other: the
     * wakes are news and are posted at default importance, and this one is the
     * evidence that a connection is being held, which is a low-importance row
     * that must not buzz every time the service is re-armed.
     */
    static final String HOLDING_ATTENTION = "yog.attention.held";

    /**
     * The attention post's fixed id: a later one REPLACES the one before it,
     * so a pocketed phone carries one standing row instead of a stack of
     * them. Tool posts count up from 1 and cannot collide with it.
     */
    static final int STANDING = 0;

    /** The one act that fixes a notification refusal, wherever it is met. */
    private static final String SETTINGS_ACT =
            "turn notifications on for yog under Settings > Apps > yog > Notifications, "
                    + "then call again.";

    /** Whether the system's dialog has been answered this run. */
    private static volatile boolean answered;

    /**
     * The pocketed foot's standing post (DESIGN §18): the notification a
     * foreground service HOLDS, so it is never replaced by anything else. It
     * sits at the top of the id space because tool posts count up from 1 —
     * reaching it would take two billion of them in one run.
     */
    static final int HOLDING = Integer.MAX_VALUE;

    /** Tool notification ids, so a second post does not replace the first. */
    private static final AtomicInteger POSTED = new AtomicInteger();

    /** Post one on `channel`, or say which act would let one be posted. */
    static String post(Context ctx, String channel, String title, String text) {
        NotificationManager manager = ctx.getSystemService(NotificationManager.class);
        if (manager == null) {
            return App.ERR + "this device has no notification service.";
        }
        if (!manager.areNotificationsEnabled()) {
            return App.ERR + ask(ctx);
        }
        int id = ATTENTION.equals(channel) ? STANDING : POSTED.incrementAndGet();
        manager.notify(id, build(ctx, channel, title, text, false));
        return App.OK + "posted notification " + id;
    }

    /**
     * Build one, without posting it. The foreground service (DESIGN §18) needs
     * the Notification itself rather than an id — {@code startForeground} takes
     * the object — and this is the one builder either caller uses, so a
     * notification from yog looks like a notification from yog wherever it
     * came from.
     *
     * <p>{@code ongoing} is the standing kind: it is not dismissed by a tap and
     * it is not auto-cancelled, because it is the operator's evidence that this
     * phone is holding a lane open.
     */
    static Notification build(
            Context ctx, String channel, String title, String text, boolean ongoing) {
        NotificationManager manager = ctx.getSystemService(NotificationManager.class);
        if (manager != null) {
            manager.createNotificationChannel(described(channel));
        }
        Notification.Builder building =
                new Notification.Builder(ctx, channel)
                        .setSmallIcon(android.R.drawable.stat_notify_chat)
                        .setContentTitle(title)
                        .setOngoing(ongoing)
                        .setAutoCancel(!ongoing);
        if (!text.isEmpty()) {
            building.setContentText(text).setStyle(new Notification.BigTextStyle().bigText(text));
        }
        PendingIntent tap = launch(ctx);
        if (tap != null) {
            building.setContentIntent(tap);
        }
        return building.build();
    }

    /**
     * Whether a post on `channel` would land — and, when the runtime grant is
     * what is missing and this app is in front, the system's own dialog on the
     * way past. The scheduled fetch asks this before it arms and before it
     * dials: a fetch whose only product is a notification nobody may see is
     * battery spent for nothing.
     *
     * <p>A channel that does not exist yet is allowed: it is created by the
     * first post, and Android grants a new channel the importance it is
     * created with.
     */
    static boolean armed(Context ctx, String channel) {
        NotificationManager manager = ctx.getSystemService(NotificationManager.class);
        if (manager == null) {
            return false;
        }
        if (!manager.areNotificationsEnabled()) {
            raise(ctx);
            return false;
        }
        NotificationChannel existing = manager.getNotificationChannel(channel);
        return existing == null || existing.getImportance() != NotificationManager.IMPORTANCE_NONE;
    }

    /**
     * The dialog has been answered, handed over by {@link MainActivity}. WHAT
     * it was answered is not read: {@code areNotificationsEnabled} is the
     * standing truth. All this records is that the one showing is spent.
     */
    static void answered() {
        answered = true;
    }

    /**
     * **Whether the operator has said this app may spend battery in the
     * background** (DESIGN §17.6) — Android's own unrestricted-battery switch,
     * off by default and read where the platform keeps it. It is the held
     * lane's consent gate and there is no second copy of it in this app: a
     * stored want beside an OS switch is the second authority §16.1 refuses,
     * and it would disagree the first time one of them was changed.
     *
     * <p>It lives here rather than in {@link Pocket} because it is the same
     * question this class already answers about notifications — *may this app
     * reach the operator, and at what cost* — and one home for that keeps the
     * two gates readable side by side.
     */
    static boolean unrestricted(Context ctx) {
        PowerManager power = ctx.getSystemService(PowerManager.class);
        return power != null && power.isIgnoringBatteryOptimizations(ctx.getPackageName());
    }

    /** A channel, and what the operator reads about it in system settings. */
    private static NotificationChannel described(String channel) {
        if (HOLDING_ATTENTION.equals(channel)) {
            NotificationChannel held =
                    new NotificationChannel(
                            HOLDING_ATTENTION, "Listening for your turn",
                            NotificationManager.IMPORTANCE_LOW);
            held.setDescription(
                    "Shown while yog is holding one connection open so a workspace that wants"
                        + " you reaches this phone promptly rather than at the next scheduled"
                        + " check. That connection stays up and the radio wakes with it — this"
                        + " is the battery cost of being told in seconds instead of in"
                        + " quarter-hours. It runs only while yog is allowed unrestricted"
                        + " battery under Settings > Apps > yog > Battery; take that back, or"
                        + " turn the Attention channel off, and it stops.");
            return held;
        }
        if (FOOT.equals(channel)) {
            NotificationChannel foot =
                    new NotificationChannel(
                            FOOT, "Serving tools", NotificationManager.IMPORTANCE_LOW);
            foot.setDescription(
                    "Shown while this phone is enrolled as hands and is holding its tool"
                        + " connection open so an agent can reach it while it is pocketed. That"
                        + " connection stays up, and the radio wakes with it — this is the"
                        + " battery cost of being reachable. It starts because this device"
                        + " carries a Thrall (foot-grade) leaf; provision it a Lernie leaf"
                        + " instead, or stop yog under Settings > Apps > Active apps, and it"
                        + " does not.");
            return foot;
        }
        if (!ATTENTION.equals(channel)) {
            return new NotificationChannel(
                    TOOLS, "Agent notifications", NotificationManager.IMPORTANCE_DEFAULT);
        }
        NotificationChannel attention =
                new NotificationChannel(
                        ATTENTION, "Attention", NotificationManager.IMPORTANCE_DEFAULT);
        attention.setDescription(
                "When a workspace wants you. yog checks on the system's own schedule — no "
                    + "sooner than every 15 minutes, and hours apart when the phone is in deep"
                    + " sleep. Each check is one short connection over the network you are"
                    + " already on, and nothing runs in between. Turning this off also stops"
                    + " the checking.");
        return attention;
    }

    /** The sentence a refusal carries. */
    private static String ask(Context ctx) {
        if (!raise(ctx)) {
            return "this app may not post notifications on this device: " + SETTINGS_ACT;
        }
        return "this app may not post notifications yet: Android's own permission dialog has "
                + "just been raised on the device — grant it there and call again, or "
                + SETTINGS_ACT;
    }

    /** Raise the system's own dialog, where one is possible; whether it went up. */
    private static boolean raise(Context ctx) {
        Activity front = App.front();
        if (front == null
                || answered
                || Build.VERSION.SDK_INT < Build.VERSION_CODES.TIRAMISU
                || ctx.checkSelfPermission(Manifest.permission.POST_NOTIFICATIONS)
                        == PackageManager.PERMISSION_GRANTED) {
            return false;
        }
        front.runOnUiThread(
                () ->
                        front.requestPermissions(
                                new String[] {Manifest.permission.POST_NOTIFICATIONS}, REQUEST));
        return true;
    }

    /** Tapping the notification opens this app, when the launcher knows how. */
    private static PendingIntent launch(Context ctx) {
        Intent intent = ctx.getPackageManager().getLaunchIntentForPackage(ctx.getPackageName());
        if (intent == null) {
            return null;
        }
        return PendingIntent.getActivity(ctx, 0, intent, PendingIntent.FLAG_IMMUTABLE);
    }
}
