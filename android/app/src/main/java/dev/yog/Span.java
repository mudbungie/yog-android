package dev.yog;

/**
 * How long ago, in the unit a reader acts on.
 *
 * <h2>One home, because there are two callers</h2>
 *
 * {@link Position} needs it because a fix with no age can mislead while looking
 * like an answer, and {@link Notice} needs it because a two-factor code that
 * arrived four minutes ago and one that arrived four hours ago are different
 * facts. This file exists so the second caller did not arrive as a second copy
 * of the first's private speller — the ladder and its two thresholds are one
 * decision, and two of them would drift the first time either was tuned.
 *
 * <p>The unit steps up rather than the number growing: seconds under a minute
 * and a half, minutes under an hour and a half of them, hours after that. A
 * negative span is clamped to zero — the callers' clocks can disagree with the
 * one that stamped the thing, and "in 3 seconds" is never an honest answer.
 */
final class Span {
    private Span() {}

    /** Under a minute and a half, seconds are the honest unit; past that, minutes. */
    private static final long MINUTE_S = 90;

    /** Past an hour and a half of minutes, hours are. */
    private static final long HOUR_S = 90 * 60;

    /** A span of seconds, spelled — no "ago", which is the caller's sentence. */
    static String spell(long span) {
        long seconds = Math.max(span, 0);
        if (seconds < MINUTE_S) {
            return seconds + (seconds == 1 ? " second" : " seconds");
        }
        if (seconds < HOUR_S) {
            long minutes = seconds / 60;
            return minutes + (minutes == 1 ? " minute" : " minutes");
        }
        long hours = seconds / 3600;
        return hours + (hours == 1 ? " hour" : " hours");
    }
}
