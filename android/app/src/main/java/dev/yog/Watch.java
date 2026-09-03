package dev.yog;

import android.app.job.JobInfo;
import android.app.job.JobParameters;
import android.app.job.JobScheduler;
import android.app.job.JobService;
import android.content.ComponentName;
import android.content.Context;

/**
 * The scheduled fetch (DESIGN §17; yog REMOTE §14 rung 1): the platform runs
 * this on its own schedule, the run performs one ordinary ask, and a rise
 * becomes a notification. Attention reaches a pocketed phone with no push
 * path, no third party, and no engine work at all.
 *
 * <h2>JobScheduler, not WorkManager</h2>
 *
 * WorkManager is an AndroidX library whose whole job on API 28+ is to build
 * the JobInfo below and add a database to remember it. This app schedules ONE
 * periodic job with no chaining, no constraints beyond a network, and no work
 * graph — so the library would be a dependency (rule 6, and it drags Room and
 * a startup provider) bought for an API that is already in the platform.
 *
 * <h2>What it costs, and where the operator turns it off</h2>
 *
 * The OS owns the cadence: {@code setPeriodic} is a request, and 15 minutes is
 * the platform's floor rather than this app's choice — in Doze the run is
 * batched into a maintenance window and can be hours late. That is the honest
 * limit of this rung and it cannot be engineered away here (rung 2, bl-b82d,
 * is what buys timeliness, and it buys it with a permanent notification).
 * Each run is one short mTLS connection and a handful of rows; the job asks
 * the platform for a network so a phone with none is never woken to fail.
 *
 * <p>The off switch is Android's own: the {@code Attention} notification
 * channel, whose description says what it costs. This class asks
 * {@link Notify#armed} before it arms AND at the top of every run, so
 * silencing the channel stops the checking as well as the telling — a fetch
 * whose only product is a notification nobody may see is battery spent for
 * nothing. There is no second switch inside the app (DESIGN §16.1's refused
 * per-tool toggle screen, for its reason).
 *
 * <h2>Silence is the failure mode</h2>
 *
 * Nothing here reports anything. No material, no engine, no answer: the run
 * ends and the next schedule tries again. A phone in a pocket must never nag
 * about network. The decision — what wakes a human and what stays quiet —
 * is {@code crate::attention}, on the Rust side, where the suite can reach it.
 */
public final class Watch extends JobService {
    static {
        // A job may start this process with no Activity in it, so this class
        // loads the library itself; a second load in a process that already
        // has it is a no-op.
        System.loadLibrary("yog_android");
    }

    /** This app's one job id. */
    private static final int JOB = 0xA77;

    /** The period asked for. The platform's floor is the same number. */
    private static final long PERIOD = 15 * 60 * 1000L;

    /**
     * One run, on the Rust side: dial, ask, decide, remember. Returns the
     * title and the line under it, one per line — or an empty string, which
     * is silence and is every failure.
     */
    private static native String probe(String dir);

    /**
     * Arm the fetch, or cancel it where the operator has turned the channel
     * off. Called from {@link MainActivity} on every resume, which is also
     * the resume after the permission dialog is answered: re-scheduling an
     * identical job is how JobScheduler is told nothing changed, so this is
     * safe to call as often as the app is looked at.
     */
    static void arm(Context ctx) {
        JobScheduler jobs = ctx.getSystemService(JobScheduler.class);
        if (jobs == null) {
            return;
        }
        if (!Notify.armed(ctx, Notify.ATTENTION)) {
            jobs.cancel(JOB);
            return;
        }
        jobs.schedule(
                new JobInfo.Builder(JOB, new ComponentName(ctx, Watch.class))
                        .setRequiredNetworkType(JobInfo.NETWORK_TYPE_ANY)
                        // Across a reboot too: a fetch that quietly stopped
                        // until the operator next opened the app is the
                        // silent degradation this design exists to exclude.
                        .setPersisted(true)
                        .setPeriodic(PERIOD)
                        .build());
    }

    @Override
    public boolean onStartJob(JobParameters params) {
        Context ctx = getApplicationContext();
        if (!Notify.armed(ctx, Notify.ATTENTION)) {
            JobScheduler jobs = ctx.getSystemService(JobScheduler.class);
            if (jobs != null) {
                jobs.cancel(JOB);
            }
            return false;
        }
        // onStartJob is the main thread; the ask is a socket. Returning true
        // is the promise that this thread will call jobFinished.
        new Thread(() -> sweep(ctx, params), "yog-attention").start();
        return true;
    }

    @Override
    public boolean onStopJob(JobParameters params) {
        // No retry: the next period is soon enough, and a fetch that fought
        // the platform for a slot would be spending the battery this rung is
        // cheap because it does not spend.
        return false;
    }

    private void sweep(Context ctx, JobParameters params) {
        String said = "";
        try {
            said = probe(ctx.getFilesDir().getAbsolutePath());
        } catch (RuntimeException | Error e) {
            // Nowhere to report it to, and the run is over either way.
            said = "";
        }
        if (said != null && !said.isEmpty()) {
            int cut = said.indexOf('\n');
            String title = cut < 0 ? said : said.substring(0, cut);
            String text = cut < 0 ? "" : said.substring(cut + 1);
            Notify.post(ctx, Notify.ATTENTION, title, text);
        }
        jobFinished(params, false);
    }
}
