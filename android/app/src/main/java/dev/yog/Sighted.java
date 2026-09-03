package dev.yog;

import android.content.Context;

/**
 * The door the sighted pair comes in by (DESIGN §16.1, rung 1b): two static
 * entry points, and the platform work behind them in {@link Still} /
 * {@link Shot} and {@link Fix} / {@link Position} — a file per question, so
 * none of them is the file everything lands in.
 *
 * <h2>Sighted because each one costs a runtime grant</h2>
 *
 * The paper rung ({@link Paper}) is the tools whose whole price is what an app
 * uid gets for free. These two are the next rung up: the operator grants
 * CAMERA or ACCESS_FINE_LOCATION in system settings, and revoking either turns
 * every invocation into an in-band refusal naming the one act that restores
 * it. The app never grants itself anything — §16.1's consent surface is the
 * platform's own, and there is deliberately no per-tool toggle inside this app
 * that could drift from it.
 *
 * <h2>Both are foreground-bound at this rung, and both say so</h2>
 *
 * Android refuses the camera outright to a process that is not in front, and
 * delivers no new location to one without the separate background-location
 * grant this rung does not ask for. So each asks BEFORE it acts: a tool that
 * answered ok for a photograph the platform never took, or handed back an hour
 * old position as though it were current, would be worse than one that refuses.
 *
 * <h2>The contract with the Rust side</h2>
 *
 * {@link InterfaceService}'s two-line answer protocol, exactly as {@link Paper}
 * speaks it: every entry point returns {@code "ok\n<payload>"} or
 * {@code "err\n<sentence>"}, parsed by one pure function the suite tests.
 */
public final class Sighted {
    private Sighted() {}

    /** One still photograph, written to {@code path}, or the act that refuses. */
    public static String camera(String lens, String path) {
        Context ctx = App.context();
        if (ctx == null) {
            return App.ERR + App.NO_CONTEXT;
        }
        return Still.capture(ctx, lens, path);
    }

    /** One position fix, with its accuracy and its age, or the act that refuses. */
    public static String location() {
        Context ctx = App.context();
        if (ctx == null) {
            return App.ERR + App.NO_CONTEXT;
        }
        return Fix.here(ctx);
    }
}
