package dev.yog;

import android.content.Context;

/**
 * The held attention lane (DESIGN §17.6; yog REMOTE §14 rung 2): the read a
 * pocketed SEAT keeps standing, and the reader that turns a frame into a wake.
 *
 * <p>Its own class rather than more of {@link Pocket}, on the seam the Rust
 * half already draws ({@code crate::pocket} against {@code crate::attention}):
 * that one is a service and what it holds, and this one is a lane. Nothing
 * here knows it is a foreground service, and nothing there knows what a frame
 * is.
 *
 * <h2>The three gates, each kept by somebody else</h2>
 *
 * <ol>
 *   <li><b>This device is a seat.</b> {@link #attending} decides it in Rust
 *       off the leaf on disk — a foot may not ask the world anything (REMOTE
 *       §4.2) and never reaches this lane, which is why one service can carry
 *       one notification without arbitrating between two.
 *   <li><b>The Attention channel is not silenced.</b> The same switch that is
 *       rung 1's off switch (DESIGN §17.3), for the same reason: a lane whose
 *       only product is a notification nobody may see is battery spent for
 *       nothing.
 *   <li><b>This app is allowed unrestricted battery.</b> That is REMOTE
 *       §14.2's explicit operator act — off by default, Android's own switch,
 *       and a fact the platform keeps rather than a want this app stores. It
 *       is the same discipline the foot's leaf grade is (DESIGN §18.2): the
 *       consent lives where the operator already grants it, so there is
 *       nothing here that could disagree with the OS, and taking it back
 *       stops the hold at the next pass with no setting to visit twice.
 * </ol>
 *
 * <p>All three are re-read on every pass, not only at arming: the resume is
 * what starts a hold and this is what ends one.
 *
 * <h2>A frame is the wake, and a life that said nothing rests</h2>
 *
 * Each pass is one lane LIFE on the Rust side — dial, hold, read frames —
 * which returns when the engine writes a rise or when the hold ends. What it
 * posts goes on the ATTENTION channel at rung 1's own notification id, off
 * rung 1's own memory of what was last announced, so a pocketed phone carries
 * one standing attention row whichever rung wrote it and the two can neither
 * double-announce a rise nor hide one from each other.
 */
final class Lane implements Runnable {
    /**
     * How long the lane rests after a life that said nothing. The engine's own
     * hold is thirty seconds (REMOTE §5.1's stated width), so a lane that
     * ended without a rise is redialled a hold later — one number, from the
     * place that already states it. The honest price is that a change can wait
     * one rest before it is heard, which is still seconds against rung 1's
     * fifteen minutes; what it buys is the bound on an engine that is simply
     * gone, two connections a minute rather than a spin. No ladder and no
     * counter: a phone that changes networks hourly has no number of failures
     * after which giving up is right (DESIGN §18.5's own finding).
     */
    private static final long REST = 30_000L;

    private final Pocket service;

    Lane(Pocket service) {
        this.service = service;
    }

    /**
     * Whether there is an attention lane to hold, and the standing line that
     * holds it. Empty is *not now*, and it is the reader's whole stop
     * condition.
     */
    static String line(Context ctx) {
        if (!Notify.armed(ctx, Notify.ATTENTION) || !Notify.unrestricted(ctx)) {
            return "";
        }
        return attending(ctx.getFilesDir().getAbsolutePath());
    }

    /**
     * Whether this device is a seat, and the line the hold shows while it is
     * — {@code crate::pocket::attending}, decided in Rust off the leaf and
     * tested at the coverage floor.
     */
    private static native String attending(String dir);

    /**
     * One life of the lane: dial, hold, and answer the first rise the engine
     * writes — the title, then the line under it, or an empty string for
     * silence. It BLOCKS for up to the engine's own hold, which is what a held
     * read is. {@code crate::attention::wake}.
     */
    private static native String wake(String dir);

    @Override
    public void run() {
        while (service.listens()) {
            String line = line(service);
            if (line.isEmpty()) {
                service.released();
                return;
            }
            service.restate(line);
            String woke;
            try {
                woke = wake(service.getFilesDir().getAbsolutePath());
            } catch (RuntimeException | Error e) {
                // Nowhere to report it to, and a lane that cannot dial is the
                // silence this rung is built to fail into.
                return;
            }
            if (woke == null || woke.isEmpty()) {
                try {
                    Thread.sleep(REST);
                } catch (InterruptedException e) {
                    return;
                }
                continue;
            }
            int cut = woke.indexOf('\n');
            Notify.post(
                    service,
                    Notify.ATTENTION,
                    cut < 0 ? woke : woke.substring(0, cut),
                    cut < 0 ? "" : woke.substring(cut + 1));
        }
    }
}
