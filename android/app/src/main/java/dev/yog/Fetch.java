package dev.yog;

import java.io.File;
import java.io.FileOutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.net.HttpURLConnection;
import java.net.MalformedURLException;
import java.net.ProtocolException;
import java.net.URL;
import java.net.URLConnection;
import java.nio.charset.StandardCharsets;
import java.util.List;
import java.util.Map;

/**
 * The request itself: {@link java.net.HttpURLConnection}, which on Android is
 * OkHttp behind the platform's own API — so this app links no HTTP client of
 * its own and inherits the device's TLS, its proxy settings and its trust
 * store.
 *
 * <h2>Two bounds, and they answer different questions</h2>
 *
 * {@link #HELD} is a MEMORY bound: what this process will hold of a body
 * nobody asked to save. The capture bound — how much of the answer a model
 * sees — is the Rust side's elision, which is pure and tested; this one only
 * keeps a fetch of something enormous from being an out-of-memory kill, and
 * says on its own line that it stopped. A body being SAVED is not held at
 * all: it streams to the file, so a download is bounded by storage and not by
 * this number.
 *
 * <h2>Every failure is a sentence, never an exception</h2>
 *
 * The two-line protocol is the whole contract, so an unknown host, a refused
 * certificate and a broken pipe all come back as {@code err\n…} carrying what
 * the platform said. The exception's own text is what names the fix — "Trust
 * anchor for certification path not found" is the operator's cue and this
 * layer must not paraphrase it away.
 */
final class Fetch {
    private Fetch() {}

    /** How long a connect may take before the platform gives up. */
    private static final int CONNECT_MS = 15000;

    /** How long a read may stall before the platform gives up. */
    private static final int READ_MS = 30000;

    /** How much of an unsaved body this process will hold. */
    private static final int HELD = 2 * 1024 * 1024;

    /** How much is copied at a time when a body is streaming to a file. */
    private static final int CHUNK = 64 * 1024;

    static String request(String method, String url, String headers, String body, String save) {
        HttpURLConnection conn;
        try {
            URLConnection opened = new URL(url).openConnection();
            if (!(opened instanceof HttpURLConnection)) {
                return App.ERR + "this tool speaks http and https only, and \"" + url
                        + "\" is neither.";
            }
            conn = (HttpURLConnection) opened;
        } catch (MalformedURLException e) {
            return App.ERR + "\"" + url + "\" is not a URL this device can read: " + e.getMessage();
        } catch (IOException e) {
            return App.ERR + "this device could not open " + url + ": " + e;
        }
        try {
            return sent(conn, method, headers, body, save);
        } catch (IOException | RuntimeException e) {
            return App.ERR + "the request to " + url + " failed: " + e;
        } finally {
            conn.disconnect();
        }
    }

    /** The configured request, made, and its answer read back. */
    private static String sent(HttpURLConnection conn, String method, String headers, String body,
            String save) throws IOException {
        try {
            conn.setRequestMethod(method);
        } catch (ProtocolException e) {
            return App.ERR + "this device's HTTP stack refuses the method " + method + ": " + e;
        }
        conn.setConnectTimeout(CONNECT_MS);
        conn.setReadTimeout(READ_MS);
        conn.setInstanceFollowRedirects(true);
        for (String line : headers.split("\n")) {
            int colon = line.indexOf(':');
            if (colon > 0) {
                conn.setRequestProperty(line.substring(0, colon), line.substring(colon + 1).trim());
            }
        }
        if (!body.isEmpty()) {
            conn.setDoOutput(true);
            byte[] bytes = body.getBytes(StandardCharsets.UTF_8);
            conn.setFixedLengthStreamingMode(bytes.length);
            try (OutputStream out = conn.getOutputStream()) {
                out.write(bytes);
            }
        }
        int code = conn.getResponseCode();
        StringBuilder said = new StringBuilder(status(conn, code));
        InputStream in = code >= HttpURLConnection.HTTP_BAD_REQUEST
                ? conn.getErrorStream()
                : conn.getInputStream();
        said.append('\n').append(save.isEmpty() ? held(in) : saved(in, save));
        return App.OK + said;
    }

    /** The status line and every response header, in the order they arrived. */
    private static String status(HttpURLConnection conn, int code) throws IOException {
        StringBuilder said = new StringBuilder("HTTP " + code);
        String message = conn.getResponseMessage();
        if (message != null) {
            said.append(' ').append(message);
        }
        for (Map.Entry<String, List<String>> header : conn.getHeaderFields().entrySet()) {
            // The status line arrives under a null key, and it is already the
            // first line above.
            if (header.getKey() == null) {
                continue;
            }
            for (String value : header.getValue()) {
                said.append('\n').append(header.getKey()).append(": ").append(value);
            }
        }
        return said.toString();
    }

    /** A body this process holds, as text, saying so if it stopped early. */
    private static String held(InputStream in) throws IOException {
        if (in == null) {
            return "";
        }
        byte[] buffer = new byte[CHUNK];
        java.io.ByteArrayOutputStream out = new java.io.ByteArrayOutputStream();
        int read;
        while (out.size() < HELD && (read = in.read(buffer)) > 0) {
            out.write(buffer, 0, Math.min(read, HELD - out.size()));
        }
        String text = new String(out.toByteArray(), StandardCharsets.UTF_8);
        if (out.size() >= HELD) {
            return "\n" + text + "\n[the body is larger than " + HELD
                    + " bytes; the rest was not read — use save_to to keep all of it]";
        }
        return "\n" + text;
    }

    /** A body streamed to a file, answered as the path and the byte count. */
    private static String saved(InputStream in, String save) throws IOException {
        File file = new File(save);
        File parent = file.getParentFile();
        if (parent != null) {
            parent.mkdirs();
        }
        long total = 0;
        try (OutputStream out = new FileOutputStream(file)) {
            if (in != null) {
                byte[] buffer = new byte[CHUNK];
                int read;
                while ((read = in.read(buffer)) > 0) {
                    out.write(buffer, 0, read);
                    total += read;
                }
            }
        }
        return "\nsaved " + total + " bytes to " + file.getAbsolutePath();
    }
}
