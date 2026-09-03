package dev.yog;

import android.content.Context;

/**
 * The door the shade read comes in by (DESIGN §16.1, rung 2): one static entry
 * point, with the listener itself in {@link ShadeService} and the shape of one
 * row in {@link Notice} — a file per question, as the rungs below it.
 *
 * <h2>Read-only, and that is the whole rung</h2>
 *
 * A connected listener may also dismiss a notification and fire its actions.
 * Neither is built: this rung reads, and the ball that would spend a
 * notification's buttons is where that capability gets argued for rather than
 * arriving as a side effect of the enable.
 *
 * <h2>The contract with the Rust side</h2>
 *
 * {@link InterfaceService}'s two-line answer protocol exactly: {@code
 * "ok\n<payload>"} or {@code "err\n<sentence>"}, parsed by one pure function
 * the suite tests.
 */
public final class Shade {
    private Shade() {}

    /** The shade as text, newest first, or the act that refuses. */
    public static String notifications(String limit) {
        Context ctx = App.context();
        if (ctx == null) {
            return App.ERR + App.NO_CONTEXT;
        }
        return ShadeService.read(ctx, count(limit));
    }

    /**
     * How many rows to answer with. The Rust side owns the default and always
     * sends a positive integer, so an unreadable count here means the two sides
     * disagree about the protocol — in which case answering everything beats
     * inventing a second default that would then have to be kept in step with
     * the first.
     */
    private static int count(String limit) {
        try {
            int stated = Integer.parseInt(limit);
            return stated > 0 ? stated : Integer.MAX_VALUE;
        } catch (NumberFormatException e) {
            return Integer.MAX_VALUE;
        }
    }
}
