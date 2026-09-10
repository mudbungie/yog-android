package dev.yog;

/**
 * The door the {@code http} tool comes in by (DESIGN §16.1, the net rung):
 * one static entry point, and the platform work behind it in {@link Fetch} —
 * the split {@link Paper} and {@link Device} already draw, so the door stays
 * a door.
 *
 * <h2>Why the platform's stack and not a bundled client</h2>
 *
 * {@code HttpsURLConnection} verifies against the DEVICE's trust store: the
 * system roots plus whatever the operator installed. Nothing here ships a
 * certificate bundle, so nothing here can go stale, and a phone that trusts a
 * private CA reaches its own hosts with no argument. The honest other half is
 * that this tool can never be more permissive than the device it runs on —
 * a host Android will not trust is a refusal here, carrying the platform's
 * own sentence.
 *
 * <h2>The contract with the Rust side</h2>
 *
 * {@link InterfaceService}'s two-line answer protocol exactly: {@code
 * "ok\n<payload>"} or {@code "err\n<sentence>"}, parsed by one pure function
 * the suite tests. Five strings in, because a door's descriptor is built from
 * the argument count and a second marshalling would be a second grammar: an
 * absent header set, an absent body and an absent save path are each the
 * empty string.
 */
public final class Net {
    private Net() {}

    /**
     * Make one request. {@code headers} is {@code Name: value} lines,
     * {@code body} and {@code save} are empty when unstated.
     */
    public static String http(String method, String url, String headers, String body,
            String save) {
        return Fetch.request(method, url, headers, body, save);
    }
}
