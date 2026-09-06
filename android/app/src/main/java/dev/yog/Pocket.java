package dev.yog;

import android.app.Notification;
import android.app.Service;
import android.app.NotificationManager;
import android.content.Context;
import android.content.Intent;
import android.content.pm.ServiceInfo;
import android.os.Build;
import android.os.IBinder;

/**
 * The pocketed foot (DESIGN §18; §16.1 rung 3): a foreground service that
 * holds this process open so the tool host's {@code invocations} read keeps
 * standing while the phone is in a pocket.
 *
 * <h2>Two lanes, one service, and they never overlap</h2>
 *
 * Since bl-b82d this service also carries REMOTE §14's rung 2 (DESIGN §17.6):
 * a SEAT device holds {@code Query::Attention} open from the pocket, and a
 * frame is the wake. It is the same service and not a second one — one
 * {@code specialUse} grant per device, one arming point, one standing row —
 * and the two lanes are mutually exclusive by GRADE rather than by
 * arbitration: a foot may not ask the world anything (REMOTE §4.2), and a seat
 * hosts no tools. So exactly one of {@link #standing} and {@link #attending}
 * ever answers, which is what lets one notification say one thing.
 *
 * <h2>What it holds, and what it does not</h2>
 *
 * It runs no lane of its own. The host is the process's
 * ({@code crate::state}), started when the app boots, and this service's whole
 * effect is on the PLATFORM: a process running a foreground service is a
 * visible process, so it is not frozen, not evicted, and — the fact this rung
 * turns on — not subject to Doze's network block, which applies per UID to
 * processes below the foreground-service threshold. Without it the platform
 * ends a backgrounded app's sockets and the foot is absent until the next
 * look.
 *
 * <p>It deliberately cannot CREATE a lane. A service may be started into a
 * process with no Activity, and this app's tool bridges resolve their classes
 * through handles android-activity fills on the way to {@code android_main} —
 * so a host built here would be a foot whose platform tools all refuse. That
 * is why {@code onStartCommand} returns {@code START_NOT_STICKY}: a service the
 * system restarted into an empty process would be a notification claiming
 * something no thread is doing.
 *
 * <h2>The type is {@code specialUse}, and the alternatives are barred by the
 * platform's own clock</h2>
 *
 * Android 15 caps {@code dataSync} at six hours in any 24 and calls
 * {@code onTimeout} after it — for a foot that is meant to be reachable for
 * days that is not a cost, it is a defect on a timer. {@code connectedDevice}
 * is uncapped but describes a Bluetooth/NFC/USB companion and carries a runtime
 * prerequisite this app has no business declaring. {@code specialUse} is the
 * platform's own "none of the above": uncapped, no runtime prerequisite, and
 * its {@code <property>} subtype in the manifest is where the honest sentence
 * goes. It is reviewed by Play and this app does not ship there — the manifest
 * says so, because a justification nobody checks still has to be true.
 *
 * <h2>The operator's switches are Android's, and there are three</h2>
 *
 * The material (a Thrall-grade leaf is what makes this device hands at all —
 * {@code crate::pocket}), the {@code Serving tools} notification channel in
 * system settings, whose description carries the price, and <b>Active apps →
 * Stop</b>, which the platform documents as removing the whole app from
 * memory. No switch was added inside this app: it would be a second authority
 * beside the one the OS enforces (§16.1's refused per-tool toggle screen, for
 * its reason).
 *
 * <p>Without {@code POST_NOTIFICATIONS} the service still runs and the
 * notification is simply not in the drawer — the platform keeps it in the
 * foreground-services manager either way. The grant is asked for elsewhere
 * ({@link Notify}); nothing here asks, because a service is not a screen.
 */
public final class Pocket extends Service {
    static {
        // A service may be the first thing in a process, so this class loads
        // the library itself; a second load in a process that has it is a
        // no-op. {@link Watch} does the same for the same reason.
        System.loadLibrary("yog_android");
    }

    /**
     * How often the standing line is re-read. It is a repaint cadence for a
     * surface the operator can pull down at any moment — slower than the app's
     * own two seconds because nobody is looking at a pocketed phone, and free
     * when they are not: {@code Thread.sleep} holds no wakelock, so a suspended
     * CPU simply makes it later.
     */
    private static final long REFRESH = 5_000L;

    /**
     * The standing line, decided in Rust: the title, then the line under it,
     * or an empty string meaning this device is not enrolled as hands and there
     * is nothing here to hold. Every branch of it is
     * {@code crate::pocket::line} and is tested at the coverage floor.
     */
    private static native String standing(String dir);

    /** The watcher, or null when none runs. Written and read on two threads. */
    private volatile Thread watching;

    /** The attention lane's reader, on the same terms. */
    private volatile Thread listening;

    /** What the notification currently says, so an unchanged line is not re-posted. */
    private volatile String said = "";

    /** Whether the attention lane's reader should still be reading — the one
     * fact {@link Lane} asks this service, so the thread's life has one home. */
    boolean listens() {
        return listening != null;
    }

    /** What the lane says it is holding, when the line it holds has changed. */
    void restate(String line) {
        NotificationManager manager = getSystemService(NotificationManager.class);
        if (!line.equals(said) && manager != null) {
            said = line;
            manager.notify(Notify.HOLDING, of(Notify.HOLDING_ATTENTION, line));
        }
    }

    /** The lane has nothing left to hold: drop the row and stop. */
    void released() {
        stopForeground(STOP_FOREGROUND_REMOVE);
        stopSelf();
    }

    /**
     * Start the hold where this device is hands, stop it where it is not.
     * Called from {@link MainActivity} on every resume — which is the moment
     * the platform's background-start restriction exempts ("transitions from a
     * user-visible state"), and re-starting a service that already runs is how
     * the platform is told nothing changed.
     */
    static void arm(Context ctx) {
        Intent intent = new Intent(ctx, Pocket.class);
        if (standing(ctx.getFilesDir().getAbsolutePath()).isEmpty()
                && Lane.line(ctx).isEmpty()) {
            ctx.stopService(intent);
            return;
        }
        ctx.startForegroundService(intent);
    }

    @Override
    public int onStartCommand(Intent intent, int flags, int startId) {
        String foot = standing(getFilesDir().getAbsolutePath());
        if (!foot.isEmpty()) {
            hold(Notify.FOOT, foot);
            if (watching == null) {
                watching = new Thread(this::watch, "yog-pocket");
                watching.start();
            }
            return START_NOT_STICKY;
        }
        String lane = Lane.line(this);
        if (lane.isEmpty()) {
            // Nothing to hold. Stop before the five-second promise
            // `startForegroundService` made comes due.
            stopSelf();
            return START_NOT_STICKY;
        }
        hold(Notify.HOLDING_ATTENTION, lane);
        if (listening == null) {
            listening = new Thread(new Lane(this), "yog-attention-lane");
            listening.start();
        }
        return START_NOT_STICKY;
    }

    @Override
    public IBinder onBind(Intent intent) {
        return null;
    }

    /**
     * Both threads are dropped, and neither is waited on. The foot's watcher
     * answers an interrupt at once; the lane's reader is parked in a socket
     * read that no interrupt reaches, so it leaves when its hold ends — at
     * worst one hold later, holding nothing but a connection the far end is
     * about to close anyway.
     */
    @Override
    public void onDestroy() {
        for (Thread thread : new Thread[] {watching, listening}) {
            if (thread != null) {
                thread.interrupt();
            }
        }
        watching = null;
        listening = null;
        super.onDestroy();
    }

    /**
     * The two-line answer protocol this crate speaks everywhere Java asks Rust
     * a question (§17.4): the title, then the line under it.
     */
    Notification of(String channel, String now) {
        int cut = now.indexOf('\n');
        return Notify.build(
                this,
                channel,
                cut < 0 ? now : now.substring(0, cut),
                cut < 0 ? "" : now.substring(cut + 1),
                true);
    }

    /** Put the line in front of the operator, as the notification this holds.
     *
     * <p>One id for both lanes ({@link Notify#HOLDING}) because it is one row —
     * the thing this service is holding — and no device is ever both a foot
     * and a seat, so the two can never be on screen together. */
    private void hold(String channel, String now) {
        said = now;
        Notification notification = of(channel, now);
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
            startForeground(
                    Notify.HOLDING, notification, ServiceInfo.FOREGROUND_SERVICE_TYPE_SPECIAL_USE);
        } else {
            startForeground(Notify.HOLDING, notification);
        }
    }

    /**
     * Follow the lane. A line that changed is re-posted — a phone that says
     * "standing by" while its channel is broken is the silent degradation this
     * design excludes — and an empty one ends the hold, which is the operator
     * having replaced this device's leaf with one that is not a foot's.
     *
     * <p>A lane that stopped for good does NOT end the hold: the notification
     * is then the only surface a pocketed phone has to say so, and it says that
     * nothing is on the network. A service that vanished instead would take the
     * evidence with it.
     */
    private void watch() {
        while (watching != null) {
            try {
                Thread.sleep(REFRESH);
            } catch (InterruptedException e) {
                return;
            }
            String now;
            try {
                now = standing(getFilesDir().getAbsolutePath());
            } catch (RuntimeException | Error e) {
                return;
            }
            if (now.isEmpty()) {
                stopForeground(STOP_FOREGROUND_REMOVE);
                stopSelf();
                return;
            }
            NotificationManager manager = getSystemService(NotificationManager.class);
            if (!now.equals(said) && manager != null) {
                said = now;
                manager.notify(Notify.HOLDING, of(Notify.FOOT, now));
            }
        }
    }
}
