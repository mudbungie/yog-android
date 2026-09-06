package dev.yog;

import android.content.BroadcastReceiver;
import android.content.Context;
import android.content.Intent;

/**
 * The device came back (DESIGN §18.8): arm whatever this device's material
 * says it should be holding, without waiting for somebody to open the app.
 *
 * <h2>What it closes</h2>
 *
 * Until bl-d22d the pocketed foot was armed by {@code MainActivity.onResume}
 * alone, so a phone that rebooted in a pocket answered no tool call until a
 * human looked at it — the honest limit §18.3 wrote down. A phone that reboots
 * in a pocket is exactly the case that rung is for.
 *
 * <h2>Why it works now and would not have before</h2>
 *
 * A boot receiver was always LAWFUL — {@code specialUse} is on none of Android
 * 15's barred lists — and would have been useless: the service could not
 * CREATE a lane, because this app's tool bridges resolve their classes through
 * {@code ndk_context} globals that android-activity fills on the way to
 * {@code android_main}, and a process no Activity created has none. What
 * changed is one hand-over: a Service already holds both values those globals
 * carry — the process VM, and the Application whose class loader the bridges
 * ask for — so {@link Pocket#serve} fills them and every bridge under it works
 * unchanged.
 *
 * <h2>What it costs, and what it does not</h2>
 *
 * It moves the START of a cost the operator already chose; it adds none. A
 * device holding a Thrall (foot-grade) leaf was going to hold a foreground
 * service and its notification the moment the app was opened — this makes that
 * moment the boot instead, which is what enrolling a device AS hands asked
 * for. A device that is not hands, and a seat whose operator has not allowed
 * unrestricted battery (§17.6), get nothing here: {@code Pocket.arm} reads the
 * same three gates it reads on every resume, so there is exactly one place
 * that decides whether anything runs at all.
 *
 * <p>It is registered for {@code BOOT_COMPLETED} only. {@code
 * setPersisted(true)} on the scheduled fetch (§17.3) already required the
 * permission and receives nothing; this is the receiver that permission was
 * always for.
 */
public final class Boot extends BroadcastReceiver {
    @Override
    public void onReceive(Context ctx, Intent intent) {
        if (intent == null || !Intent.ACTION_BOOT_COMPLETED.equals(intent.getAction())) {
            return;
        }
        // Starting a foreground service from BOOT_COMPLETED is one of the
        // platform's own named exemptions from the API 31+ background-start
        // restriction, and `specialUse` is on none of Android 15's barred
        // lists — so this is the one call, and every gate behind it is
        // `Pocket.arm`'s.
        Pocket.arm(ctx);
    }
}
