package dev.yog;

import android.app.Activity;
import android.content.ActivityNotFoundException;
import android.content.Intent;
import android.net.Uri;
import androidx.core.content.FileProvider;
import java.io.File;
import java.util.concurrent.atomic.AtomicBoolean;

/**
 * The release channel's device half (bl-7a68, DESIGN §20): read the newest
 * published release, and — when a person taps — put the system installer in
 * front of them.
 *
 * <h2>Three doors, all polled, none blocking</h2>
 *
 * Both reads answer from a volatile field a worker thread fills, so the frame
 * that calls them never waits on a socket. {@link #state} starts the one feed
 * fetch this process makes, on its first call; {@link #install} starts the one
 * download; {@link #said} is how the row that fired it learns what happened,
 * seconds later, without this class holding a callback into Rust.
 *
 * <h2>Nothing here decides anything</h2>
 *
 * The version comparison, the asset pick and the https requirement are
 * {@code crate::update}'s, under the 100% floor, because this side has no test
 * harness at all. What crosses is the feed body verbatim in the two-line
 * protocol every bridge in this app answers in.
 *
 * <h2>Android will not let a process replace itself</h2>
 *
 * An unattended install needs a privileged installer, a device owner or an app
 * store, and this is none of them. So the last rung is {@code ACTION_VIEW} on
 * the downloaded APK: the platform's own installer, in front of a person, who
 * taps. That is the whole difference between this reconciler and the fleet's
 * other four.
 */
public final class Update {
    private Update() {}

    /** Where the {@code updates/} cache directory is served from, per the manifest. */
    private static final String AUTHORITY = ".updates";

    /** The one MIME type the platform's package installer answers to. */
    private static final String APK = "application/vnd.android.package-archive";

    /** The feed answer, in the two-line protocol, or null while nothing has come back. */
    private static volatile String feed;

    /**
     * Whether the one fetch this process makes has been started, and whether a
     * download is running.
     *
     * <p>Atomics rather than a {@code synchronized} method, and it is not a
     * preference: {@code make apk} pins every JNI name this crate resolves
     * against the dex that carries it (DESIGN §15.7), and it looks for access
     * flags reading exactly {@code PUBLIC STATIC}. A {@code synchronized}
     * modifier adds {@code DECLARED_SYNCHRONIZED} and the entry point stops
     * being one the gate can see. A compare-and-set says the same thing about
     * the one race there is — two frames entering at once — without touching
     * the method's signature.
     */
    private static final AtomicBoolean asked = new AtomicBoolean();

    /** What the download last said, or empty when it has nothing to say. */
    private static volatile String said = "";

    /** Whether a download is running, so a second tap does not start a second one. */
    private static final AtomicBoolean fetching = new AtomicBoolean();

    /**
     * The newest release, as the feed answered it — and the call that starts
     * the asking.
     *
     * <p>Once per process, on the first frame that paints the roster. A phone
     * has no timer worth the battery and this app has no business holding one
     * for a question whose answer changes on the order of days; opening the app
     * IS the cadence, which is also the only moment an offer could be seen.
     */
    public static String state() {
        if (asked.compareAndSet(false, true)) {
            worker(
                    () -> {
                        String read = Release.latest();
                        feed = read;
                    });
        }
        // **Empty is "nothing yet", and it is a THIRD state on purpose.** A
        // refusal spelled here while the socket is still open would be
        // indistinguishable from one the network actually gave, and the Rust
        // side stops asking the moment an answer exists — so a phone with no
        // network would then cache the wrong verdict, and one with a slow one
        // would be asked across the JNI boundary on every frame forever.
        String held = feed;
        return held == null ? "" : held;
    }

    /** What the download last said, for the row that fired it. Empty is silence. */
    public static String said() {
        return said;
    }

    /**
     * Fetch {@code url} and hand it to the system installer.
     *
     * <p>Answered immediately with the sentence {@link #said} will keep
     * repeating, because the download is seconds long and the caller is a
     * frame. A second tap while one is running is the same sentence again and
     * not a second download.
     */
    public static String install(String url) {
        if (App.context() == null) {
            said = App.NO_CONTEXT;
            return said;
        }
        if (!fetching.compareAndSet(false, true)) {
            return said;
        }
        said = "downloading the update…";
        worker(
                () -> {
                    String outcome = fetch(url);
                    said = outcome;
                    fetching.set(false);
                });
        return said;
    }

    /** Download the APK and open the installer on it; the sentence is the outcome. */
    private static String fetch(String url) {
        File apk = Release.download(App.context(), url);
        if (apk == null) {
            return "the update did not download: check this device's network and try again.";
        }
        Activity front = App.front();
        if (front == null) {
            return "yog left the screen before the installer could open: tap update again.";
        }
        Uri uri =
                FileProvider.getUriForFile(front, front.getPackageName() + AUTHORITY, apk);
        Intent intent =
                new Intent(Intent.ACTION_VIEW)
                        .setDataAndType(uri, APK)
                        .addFlags(
                                Intent.FLAG_GRANT_READ_URI_PERMISSION
                                        | Intent.FLAG_ACTIVITY_NEW_TASK);
        try {
            front.startActivity(intent);
        } catch (ActivityNotFoundException e) {
            return "this device has no package installer to hand the update to.";
        }
        return "the installer has the update — tap install, then open yog again.";
    }

    /** One named background thread. Both doors start work the frame may not wait on. */
    private static void worker(Runnable work) {
        Thread thread = new Thread(work, "yog-update");
        thread.setDaemon(true);
        thread.start();
    }
}
