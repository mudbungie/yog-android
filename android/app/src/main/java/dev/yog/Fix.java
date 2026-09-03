package dev.yog;

import android.Manifest;
import android.app.Activity;
import android.content.Context;
import android.content.pm.PackageManager;
import android.location.Location;
import android.location.LocationListener;
import android.location.LocationManager;
import android.os.Bundle;
import android.os.HandlerThread;

import java.util.List;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.TimeUnit;

/**
 * One position fix: the two grants, the device's own switch, and a bounded
 * wait on every provider that is turned on (DESIGN §16.1). What a fix SAYS —
 * with its accuracy and, always, its age — is {@link Position}'s.
 *
 * <h2>Three gates, and each names the one act that lifts it</h2>
 *
 * The runtime grant (asked once per run when this app is in front, on this
 * class's own request id through the bl-d815 permission-result hook, and named
 * as a settings act otherwise — {@link Notify}'s shape exactly); the
 * device-wide location switch, which no app may touch; and having anything to
 * answer with at all.
 *
 * <h2>Fine or coarse is the operator's choice, not this app's</h2>
 *
 * Android 12 and later lets the operator answer a fine-location ask with
 * *approximate*, and the honest handling is to accept either: the fix that
 * comes back carries its own accuracy, and {@link Position} states it, so a
 * kilometre-wide answer is legible as one rather than refused. Both are
 * declared in the manifest for that reason.
 *
 * <h2>Foreground-bound, and that is the rung</h2>
 *
 * Without ACCESS_BACKGROUND_LOCATION — a separate settings trip this rung does
 * not ask for (§16.1) — Android delivers no new fix to an app that is not on
 * screen. Nothing here can lift that, so what it does instead is never lie
 * about it: an answer says whether it is new or last-known, and a refusal with
 * nothing at all names the foreground fact.
 */
final class Fix {
    private Fix() {}

    /** This class's permission-request id; {@link Camera}, {@link Notify}, {@link Still} hold the others. */
    static final int REQUEST = 0x0C;

    /** How long a fix is waited for before the last-known one is the answer. */
    private static final long PATIENCE_S = 12;

    /** The one act that fixes a location refusal, wherever it is met. */
    private static final String SETTINGS_ACT =
            "turn Location on for yog under Settings > Apps > yog > Permissions, then call again.";

    /** Whether the system's dialog has been answered this run. */
    private static volatile boolean answered;

    /** Where this device is, or the sentence naming what would let it say. */
    static String here(Context ctx) {
        LocationManager places = ctx.getSystemService(LocationManager.class);
        if (places == null) {
            return App.ERR + "this device has no location service.";
        }
        if (!granted(ctx)) {
            return App.ERR + ask(ctx);
        }
        if (!places.isLocationEnabled()) {
            return App.ERR
                    + "this device's location switch is off, so nothing on it has a position: "
                    + "turn Location on in the quick settings, or under Settings > Location, "
                    + "then call again.";
        }
        return read(places);
    }

    /**
     * The dialog has been answered, handed over by {@link MainActivity}. WHAT
     * it was answered is not read: {@link #granted} above is the standing
     * truth. All this records is that the one showing is spent.
     */
    static void answered() {
        answered = true;
    }

    private static boolean granted(Context ctx) {
        return ctx.checkSelfPermission(Manifest.permission.ACCESS_FINE_LOCATION)
                        == PackageManager.PERMISSION_GRANTED
                || ctx.checkSelfPermission(Manifest.permission.ACCESS_COARSE_LOCATION)
                        == PackageManager.PERMISSION_GRANTED;
    }

    /** The sentence a refusal carries — and the raise itself, where one is possible. */
    private static String ask(Context ctx) {
        Activity front = App.front();
        if (front == null || answered) {
            return "this app may not read this device's location: " + SETTINGS_ACT;
        }
        front.runOnUiThread(
                () ->
                        front.requestPermissions(
                                new String[] {
                                    Manifest.permission.ACCESS_FINE_LOCATION,
                                    Manifest.permission.ACCESS_COARSE_LOCATION
                                },
                                REQUEST));
        return "this app may not read this device's location yet: Android's own permission "
                + "dialog has just been raised on the device — grant it there and call again, "
                + "or "
                + SETTINGS_ACT;
    }

    /** Ask every provider that is on, and take the first that answers. */
    private static String read(LocationManager places) {
        HandlerThread thread = new HandlerThread("yog-fix");
        thread.start();
        Location[] fresh = {null};
        CountDownLatch done = new CountDownLatch(1);
        LocationListener listener = listening(fresh, done);
        List<String> providers = places.getProviders(true);
        try {
            for (String provider : providers) {
                places.requestLocationUpdates(provider, 0L, 0f, listener, thread.getLooper());
            }
            done.await(PATIENCE_S, TimeUnit.SECONDS);
        } catch (SecurityException | IllegalArgumentException e) {
            return App.ERR + "this device refused a location request: " + e;
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
            return App.ERR + "the wait for a fix was interrupted.";
        } finally {
            places.removeUpdates(listener);
            thread.quitSafely();
        }
        if (fresh[0] != null) {
            return App.OK + Position.said(fresh[0], true, PATIENCE_S);
        }
        Location last = lastKnown(places, providers);
        if (last != null) {
            return App.OK + Position.said(last, false, PATIENCE_S);
        }
        return App.ERR + nothing(providers);
    }

    /**
     * The listener, written out rather than as a lambda. {@code
     * LocationListener}'s other three methods only became default methods in
     * API 30, and this app's minSdk is 28: a lambda compiles against the new
     * shape and throws {@code AbstractMethodError} the first time an older
     * platform calls one of them.
     */
    private static LocationListener listening(Location[] fresh, CountDownLatch done) {
        return new LocationListener() {
            @Override
            public void onLocationChanged(Location where) {
                fresh[0] = where;
                done.countDown();
            }

            @Override
            public void onStatusChanged(String provider, int status, Bundle extras) {}

            @Override
            public void onProviderEnabled(String provider) {}

            @Override
            public void onProviderDisabled(String provider) {}
        };
    }

    /** The newest fix this device already had, across every provider that is on. */
    private static Location lastKnown(LocationManager places, List<String> providers) {
        Location best = null;
        for (String provider : providers) {
            Location had = places.getLastKnownLocation(provider);
            if (had != null
                    && (best == null
                            || had.getElapsedRealtimeNanos() > best.getElapsedRealtimeNanos())) {
                best = had;
            }
        }
        return best;
    }

    /** Nothing to answer with, and the reason it is nothing. */
    private static String nothing(List<String> providers) {
        if (providers.isEmpty()) {
            return "this device has no location provider turned on: turn Location on in the "
                    + "quick settings, then call again.";
        }
        if (App.front() == null) {
            return "no fix arrived and this device has recorded none. yog is not on screen, and "
                    + "Android gives no location to an app in the background without the "
                    + "background-location grant this app does not ask for: open yog on the "
                    + "device — the notify tool can ask the operator to — then call again.";
        }
        return "no fix arrived in "
                + PATIENCE_S
                + " seconds and this device has recorded none: the receivers are on but nothing "
                + "has answered yet, which is what indoors looks like. Call again in a minute, "
                + "or ask the operator to take the phone near a window.";
    }
}
