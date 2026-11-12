package dev.yog;

import java.io.ByteArrayOutputStream;
import java.io.InputStream;
import java.io.OutputStream;
import java.net.URL;
import java.nio.charset.StandardCharsets;
import javax.net.ssl.HttpsURLConnection;

/**
 * The registration door's one request (yog REMOTE §15.1 step 2, §15.2; DESIGN
 * §5): one HTTPS POST of one JSON body, and what the door said.
 *
 * <h2>Why this is Java</h2>
 *
 * The URL is {@code https} and is verified against the PUBLIC web's roots —
 * never the operator CA, which this device does not hold yet and is
 * registering to get. The platform keeps those roots and its own HTTPS client
 * reads them, so the one request is made here and the Rust core links no root
 * store for it. Everything the request is FOR — the key, the body, reading
 * the answer, filing it, the confirmation word — is {@code crate::register},
 * pure and tested on the host; this class is the socket.
 *
 * <h2>What it will and will not do</h2>
 *
 * It follows no redirect: a claim is single-use, and a door that answers
 * anything but itself has not answered. It sends a {@code Content-Length}
 * (the door refuses a chunked body). It reads an answer framed either way a
 * reverse proxy may frame it — a stated length, or the connection's close —
 * because the platform's client does, and refuses one past {@link #MOST}
 * rather than truncating it: half a certificate is not a smaller certificate.
 *
 * <p>The answer is the bridge protocol {@code crate::register::answer}
 * reads: {@code ok}, the status, then the body verbatim; or {@code err} and a
 * sentence. Blocking — the caller is a worker thread, never a frame.
 */
public final class Register {
    private Register() {}

    private static final int CONNECT_MS = 5_000;
    private static final int READ_MS = 15_000;

    /** The most an answer may be. The real one is a CA and a leaf — about two kilobytes. */
    private static final int MOST = 64 * 1024;

    /** POST {@code body} as JSON to {@code url} and answer what the door said. */
    public static String post(String url, String body) {
        HttpsURLConnection wire = null;
        try {
            byte[] sent = body.getBytes(StandardCharsets.UTF_8);
            wire = (HttpsURLConnection) new URL(url).openConnection();
            wire.setInstanceFollowRedirects(false);
            wire.setConnectTimeout(CONNECT_MS);
            wire.setReadTimeout(READ_MS);
            wire.setUseCaches(false);
            wire.setRequestMethod("POST");
            wire.setRequestProperty("Content-Type", "application/json");
            wire.setRequestProperty("User-Agent", "yog-android");
            wire.setDoOutput(true);
            wire.setFixedLengthStreamingMode(sent.length);
            try (OutputStream to = wire.getOutputStream()) {
                to.write(sent);
            }
            int status = wire.getResponseCode();
            InputStream from = status >= 400 ? wire.getErrorStream() : wire.getInputStream();
            byte[] said = from == null ? new byte[0] : read(from);
            if (said == null) {
                return App.ERR + "it answered more than " + MOST + " bytes; refused unread";
            }
            return App.OK + status + "\n" + new String(said, StandardCharsets.UTF_8);
        } catch (Exception e) {
            return App.ERR + e;
        } finally {
            if (wire != null) {
                wire.disconnect();
            }
        }
    }

    /** Everything {@code from} holds, or null past {@link #MOST}. */
    private static byte[] read(InputStream from) throws java.io.IOException {
        try (InputStream in = from) {
            ByteArrayOutputStream held = new ByteArrayOutputStream();
            byte[] chunk = new byte[4096];
            int got;
            while ((got = in.read(chunk)) > 0) {
                if (held.size() + got > MOST) {
                    return null;
                }
                held.write(chunk, 0, got);
            }
            return held.toByteArray();
        }
    }
}
