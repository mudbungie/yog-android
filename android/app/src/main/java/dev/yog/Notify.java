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
import java.util.concurrent.atomic.AtomicInteger;

/**
 * The notification tool's platform half: the permission, the channel, and the
 * post (DESIGN §16.1).
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
 * <h2>What this is not</h2>
 *
 * A tool an agent invokes to reach the operator's pocket — not the seat's own
 * attention machinery, which is the app's and fires on its own rungs.
 */
final class Notify {
    private Notify() {}

    /** This class's permission-request id; {@link Camera} holds the other. */
    static final int REQUEST = 0x0A;

    /** The channel every tool-posted notification lands on. */
    private static final String CHANNEL = "yog.tools";

    /** The one act that fixes a notification refusal, wherever it is met. */
    private static final String SETTINGS_ACT =
            "turn notifications on for yog under Settings > Apps > yog > Notifications, "
                    + "then call again.";

    /** Whether the system's dialog has been answered this run. */
    private static volatile boolean answered;

    /** Notification ids, so a second post does not replace the first. */
    private static final AtomicInteger POSTED = new AtomicInteger();

    /** Post one, or say which act would let one be posted. */
    static String post(Context ctx, String title, String text) {
        NotificationManager manager = ctx.getSystemService(NotificationManager.class);
        if (manager == null) {
            return App.ERR + "this device has no notification service.";
        }
        if (!manager.areNotificationsEnabled()) {
            return App.ERR + ask(ctx);
        }
        manager.createNotificationChannel(
                new NotificationChannel(
                        CHANNEL, "Agent notifications", NotificationManager.IMPORTANCE_DEFAULT));
        Notification.Builder building =
                new Notification.Builder(ctx, CHANNEL)
                        .setSmallIcon(android.R.drawable.stat_notify_chat)
                        .setContentTitle(title)
                        .setAutoCancel(true);
        if (!text.isEmpty()) {
            building.setContentText(text).setStyle(new Notification.BigTextStyle().bigText(text));
        }
        PendingIntent tap = launch(ctx);
        if (tap != null) {
            building.setContentIntent(tap);
        }
        int id = POSTED.incrementAndGet();
        manager.notify(id, building.build());
        return App.OK + "posted notification " + id;
    }

    /**
     * The dialog has been answered, handed over by {@link MainActivity}. WHAT
     * it was answered is not read: {@code areNotificationsEnabled} is the
     * standing truth. All this records is that the one showing is spent.
     */
    static void answered() {
        answered = true;
    }

    /** The sentence a refusal carries — and the raise itself, where one is possible. */
    private static String ask(Context ctx) {
        Activity front = App.front();
        if (front == null
                || answered
                || Build.VERSION.SDK_INT < Build.VERSION_CODES.TIRAMISU
                || ctx.checkSelfPermission(Manifest.permission.POST_NOTIFICATIONS)
                        == PackageManager.PERMISSION_GRANTED) {
            return "this app may not post notifications on this device: " + SETTINGS_ACT;
        }
        front.runOnUiThread(
                () ->
                        front.requestPermissions(
                                new String[] {Manifest.permission.POST_NOTIFICATIONS}, REQUEST));
        return "this app may not post notifications yet: Android's own permission dialog has "
                + "just been raised on the device — grant it there and call again, or "
                + SETTINGS_ACT;
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
