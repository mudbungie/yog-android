package dev.yog;

import android.location.Location;
import android.os.SystemClock;

import java.util.Locale;

/**
 * What one fix says: where, how rough, and — always — how old.
 *
 * <h2>The age is not optional, and that is the whole of this file</h2>
 *
 * The failure this tool has to exclude is not a refusal, it is a model acting
 * on a stale fix: a phone that has been indoors for an hour still has a
 * last-known location, and it is somewhere else. So every answer carries its
 * age in the units a reader acts on, and an answer that is a last-known fix
 * rather than a new one says which it is — a provenance the caller cannot
 * derive and the platform will not volunteer.
 *
 * <h2>Monotonic, because the wall clock is not</h2>
 *
 * The age is computed from {@code getElapsedRealtimeNanos} against {@code
 * SystemClock.elapsedRealtimeNanos}, both of which count since boot and neither
 * of which a time-zone change, an NTP correction or an operator setting the
 * clock can move. {@code Location#getTime} is the wall clock, and an hour-old
 * fix would read as brand new the moment that clock jumped.
 *
 * <p>The span itself is spelled by {@link Span}, which the shade read shares —
 * one ladder of units, so a "4 minutes" here and a "4 minutes" there cannot
 * come to mean different things.
 *
 * <p><b>An unknown is said, never guessed</b> ({@link Device}'s rule): a fix
 * with no accuracy says so rather than reporting a zero a caller would read as
 * perfect.
 */
final class Position {
    private Position() {}

    /** The three lines, in the order a reader needs them. */
    static String said(Location where, boolean arrived, long waited) {
        return String.format(
                        Locale.US,
                        "location %.6f, %.6f",
                        where.getLatitude(),
                        where.getLongitude())
                + "\n"
                + rough(where)
                + "\n"
                + aged(where, arrived, waited)
                + "\n";
    }

    private static String rough(Location where) {
        if (!where.hasAccuracy()) {
            return "accuracy unknown: this device did not say how rough the fix is";
        }
        return String.format(
                Locale.US, "accurate to within %.0f m of that point", where.getAccuracy());
    }

    private static String aged(Location where, boolean arrived, long waited) {
        long seconds =
                (SystemClock.elapsedRealtimeNanos() - where.getElapsedRealtimeNanos())
                        / 1_000_000_000L;
        String ago = "fixed " + Span.spell(seconds) + " ago";
        String from = ", from " + where.getProvider();
        if (arrived) {
            return ago + from + " — a new fix, taken while this call waited";
        }
        return ago
                + from
                + " — this is the last fix this device recorded, NOT a new one: nothing arrived "
                + "in the "
                + waited
                + " seconds this waited, so the device may have moved since.";
    }
}
