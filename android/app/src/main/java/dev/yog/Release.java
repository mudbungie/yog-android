package dev.yog;

import android.content.Context;
import java.io.File;
import java.io.FileOutputStream;
import java.io.InputStream;
import java.io.OutputStream;
import java.net.HttpURLConnection;
import java.net.URL;
import java.nio.charset.StandardCharsets;

/**
 * The two reads the release channel makes over the network, and nothing else
 * (bl-7a68, DESIGN §20): the newest release as JSON, and one APK as bytes.
 *
 * <h2>Anonymous, like every other box in the fleet</h2>
 *
 * No credential of any kind. A device holds no registry token and must not —
 * that is the property that makes this channel possible rather than a place to
 * park a secret, and it is the same property the engine's ghcr pull and the
 * workstation's sparse-index read have. GitHub answers an unauthenticated read
 * of a public repository's releases at 60 requests an hour per address, and
 * this app makes one per launch.
 *
 * <h2>Both reads are bounded</h2>
 *
 * A timeout on connect and on read, and a ceiling on how many bytes either
 * answer may be. An unbounded read off a host this app does not run is how a
 * phone fills its own storage answering a question about an update.
 */
final class Release {
    private Release() {}

    /** The feed. One release, the newest, exactly as the other reconcilers read one. */
    private static final String FEED =
            "https://api.github.com/repos/mudbungie/yog-android/releases/latest";

    private static final int CONNECT_MS = 10_000;
    private static final int READ_MS = 20_000;

    /** Ceilings: a release document, and an APK. */
    private static final long FEED_MAX = 1L << 19;

    private static final long APK_MAX = 256L << 20;

    /** Where a downloaded APK lands — served to the installer by the manifest's provider. */
    private static final String DIR = "updates";

    private static final String FILE = "update.apk";

    /** The newest release, in the two-line protocol. */
    static String latest() {
        HttpURLConnection wire = null;
        try {
            wire = open(FEED);
            wire.setRequestProperty("Accept", "application/vnd.github+json");
            int code = wire.getResponseCode();
            if (code != HttpURLConnection.HTTP_OK) {
                return App.ERR + "the release feed answered " + code + ".";
            }
            byte[] body = read(wire.getInputStream(), FEED_MAX);
            return body == null
                    ? App.ERR + "the release feed answered more than this app will read."
                    : App.OK + new String(body, StandardCharsets.UTF_8);
        } catch (Exception e) {
            return App.ERR + "the release feed could not be read: " + e;
        } finally {
            if (wire != null) {
                wire.disconnect();
            }
        }
    }

    /** One APK into this app's own cache, or null when nothing arrived. */
    static File download(Context ctx, String url) {
        File dir = new File(ctx.getCacheDir(), DIR);
        if (!dir.isDirectory() && !dir.mkdirs()) {
            return null;
        }
        File apk = new File(dir, FILE);
        HttpURLConnection wire = null;
        try {
            wire = open(url);
            if (wire.getResponseCode() != HttpURLConnection.HTTP_OK) {
                return null;
            }
            try (InputStream from = wire.getInputStream();
                    OutputStream to = new FileOutputStream(apk)) {
                return copy(from, to) ? apk : null;
            }
        } catch (Exception e) {
            return null;
        } finally {
            if (wire != null) {
                wire.disconnect();
            }
        }
    }

    /** One GET, bounded, naming this app so GitHub's own logs can say who asked. */
    private static HttpURLConnection open(String url) throws java.io.IOException {
        HttpURLConnection wire = (HttpURLConnection) new URL(url).openConnection();
        wire.setConnectTimeout(CONNECT_MS);
        wire.setReadTimeout(READ_MS);
        wire.setRequestProperty("User-Agent", "yog-android");
        return wire;
    }

    /** A whole answer, or null when it is longer than {@code max}. */
    private static byte[] read(InputStream from, long max) throws java.io.IOException {
        java.io.ByteArrayOutputStream held = new java.io.ByteArrayOutputStream();
        byte[] chunk = new byte[8192];
        int got;
        while ((got = from.read(chunk)) > 0) {
            if (held.size() + got > max) {
                return null;
            }
            held.write(chunk, 0, got);
        }
        return held.toByteArray();
    }

    /** Stream an APK through, refusing one over {@link #APK_MAX}. */
    private static boolean copy(InputStream from, OutputStream to) throws java.io.IOException {
        byte[] chunk = new byte[1 << 16];
        long written = 0;
        int got;
        while ((got = from.read(chunk)) > 0) {
            written += got;
            if (written > APK_MAX) {
                return false;
            }
            to.write(chunk, 0, got);
        }
        return written > 0;
    }
}
