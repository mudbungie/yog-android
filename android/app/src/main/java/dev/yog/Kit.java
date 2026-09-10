package dev.yog;

import android.content.Context;
import android.system.ErrnoException;
import android.system.Os;
import java.io.File;
import java.io.FileOutputStream;
import java.io.IOException;
import java.io.OutputStream;
import java.nio.file.Files;

/**
 * The door the packaged executables come in by (DESIGN §16.1, the net rung):
 * one static entry point answering the directory a command line's {@code
 * PATH} should carry.
 *
 * <h2>Symlinks, because a script could never run</h2>
 *
 * Android has executed nothing from an app's writable data directory since
 * API 29 — W^X, enforced by SELinux — so a wrapper script written there could
 * not be {@code exec}d whatever its mode bits, and neither could a copy of
 * the binary. A SYMLINK in that directory is not executed: the kernel
 * resolves it and executes the TARGET, which lives under
 * {@code nativeLibraryDir} — the one place this uid may execute from, filled
 * by the installer from {@code lib/<abi>/} in the APK. That is also why the
 * executables are named {@code libcurl_bin.so} and {@code libbusybox_bin.so}:
 * Gradle carries a file into that directory only if it is named
 * {@code lib*.so}, so the name is not a disguise but the one shape the
 * platform will hold an executable in.
 *
 * <h2>The trust bundle, and why a directory will not do</h2>
 *
 * The packaged curl carries no CA bundle: the device has one. But it cannot
 * be handed the platform's certificate DIRECTORY, and the reason is exact —
 * Android names those files by OpenSSL's <i>old</i> subject hash
 * ({@code X509_NAME_hash_old}, the 0.9.8 one), while OpenSSL 1.0 and later
 * look a directory up by the NEW hash. Measured on a current emulator: the
 * platform's file is {@code 01419da9.0} and OpenSSL 3 goes looking for
 * {@code 8d89cda1.0}, so {@code --capath} at the platform's own store finds
 * nothing and every https fetch fails to build a chain.
 *
 * <p>So the roots are concatenated into one PEM file beside the links, from
 * the newest store this device has, and the shell tool points
 * {@code SSL_CERT_FILE} at it. It is REBUILT at every launch, which is what
 * keeps it a projection of the platform's store rather than a second copy of
 * it: a root the operator adds is there the next time the app starts, and
 * nothing here can go stale for longer than one launch.
 *
 * <h2>Once per process, and honest when there is nothing</h2>
 *
 * The links and the bundle are made on the first call and the answer is held
 * for the life of the process — a package cannot change under a running app,
 * and the shell tool asks before every command line. A build or an ABI that
 * packages neither executable earns a refusal naming the directory it looked
 * in, and the Rust side turns that into an empty environment rather than a
 * broken PATH: the command line then runs exactly as it would have. A device
 * whose store this app cannot read answers the links WITHOUT a bundle, and
 * curl then fails a TLS handshake in its own words rather than this app
 * pretending it has a trust store.
 */
public final class Kit {
    private Kit() {}

    /** What this APK packages, by the name a command line spends. */
    private static final String[] PACKAGED = {"curl", "busybox"};

    /** Where this platform keeps its trusted roots, newest placement first. */
    private static final String[] STORES = {
        "/apex/com.android.conscrypt/cacerts", "/system/etc/security/cacerts"
    };

    /** The directory of links and the bundle beside it, once they exist. */
    private static volatile String bin;

    /**
     * Where the packaged executables can be reached from a PATH, and — on the
     * second line, when this device has a readable trust store — the PEM
     * bundle they should verify against.
     */
    public static String path() {
        String held = bin;
        if (held != null) {
            return App.OK + held;
        }
        Context ctx = App.context();
        if (ctx == null) {
            return App.ERR + App.NO_CONTEXT;
        }
        File dir = new File(ctx.getFilesDir(), "bin");
        if (!dir.isDirectory() && !dir.mkdirs()) {
            return App.ERR + "this app could not make " + dir + " to hold the links.";
        }
        String libs = ctx.getApplicationInfo().nativeLibraryDir;
        int linked = 0;
        for (String name : PACKAGED) {
            File target = new File(libs, "lib" + name + "_bin.so");
            if (!target.canExecute()) {
                continue;
            }
            File link = new File(dir, name);
            link.delete();
            try {
                Os.symlink(target.getAbsolutePath(), link.getAbsolutePath());
            } catch (ErrnoException e) {
                return App.ERR + "this device refused a link to " + name + ": " + e.getMessage();
            }
            linked++;
        }
        if (linked == 0) {
            return App.ERR + "this build packages no executables for this device: nothing "
                    + "named lib<name>_bin.so is under " + libs + ".";
        }
        bin = dir.getAbsolutePath() + bundled(dir);
        return App.OK + bin;
    }

    /**
     * The platform's roots as one PEM file, as a second line naming it — or
     * nothing at all, which is a device this app cannot read a store from and
     * is said by saying nothing rather than by naming a file that is not
     * there.
     */
    private static String bundled(File dir) {
        for (String store : STORES) {
            File[] roots = new File(store).listFiles();
            if (roots == null || roots.length == 0) {
                continue;
            }
            File pem = new File(dir, "roots.pem");
            try (OutputStream out = new FileOutputStream(pem)) {
                for (File root : roots) {
                    Files.copy(root.toPath(), out);
                }
            } catch (IOException e) {
                return "";
            }
            return "\n" + pem.getAbsolutePath();
        }
        return "";
    }
}
