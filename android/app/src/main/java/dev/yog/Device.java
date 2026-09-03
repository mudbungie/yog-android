package dev.yog;

import android.content.Context;
import android.net.ConnectivityManager;
import android.net.Network;
import android.net.NetworkCapabilities;
import android.os.BatteryManager;
import android.os.StatFs;
import java.util.Locale;

/**
 * What this device is doing right now, as three lines of text — the read half
 * of the paper tools (DESIGN §16.1).
 *
 * <p>Every one of them is a plain read an app uid may make with no runtime
 * permission and no screen in front: the battery through {@code
 * BatteryManager}, the network kind through {@code ConnectivityManager} (which
 * ACCESS_NETWORK_STATE, a normal permission the installer grants outright,
 * makes readable), and free space through {@code StatFs} on this app's own
 * files directory.
 *
 * <p><b>An unknown is said, never guessed.</b> A missing system service or a
 * battery that reports no level answers "unknown" rather than a zero, because
 * a number a caller cannot tell apart from a real reading is worse than no
 * number.
 */
final class Device {
    private Device() {}

    /** The three lines, in the order an operator reads them. */
    static String state(Context ctx) {
        return battery(ctx) + "\n" + network(ctx) + "\n" + storage(ctx) + "\n";
    }

    private static String battery(Context ctx) {
        BatteryManager manager = ctx.getSystemService(BatteryManager.class);
        int level =
                manager == null
                        ? -1
                        : manager.getIntProperty(BatteryManager.BATTERY_PROPERTY_CAPACITY);
        if (level < 0) {
            return "battery unknown";
        }
        return "battery " + level + "%, " + (manager.isCharging() ? "charging" : "on battery");
    }

    private static String network(Context ctx) {
        ConnectivityManager manager = ctx.getSystemService(ConnectivityManager.class);
        Network active = manager == null ? null : manager.getActiveNetwork();
        NetworkCapabilities caps = active == null ? null : manager.getNetworkCapabilities(active);
        if (caps == null) {
            return "network none";
        }
        String kind = "other";
        if (caps.hasTransport(NetworkCapabilities.TRANSPORT_WIFI)) {
            kind = "wifi";
        } else if (caps.hasTransport(NetworkCapabilities.TRANSPORT_CELLULAR)) {
            kind = "cellular";
        } else if (caps.hasTransport(NetworkCapabilities.TRANSPORT_ETHERNET)) {
            kind = "ethernet";
        } else if (caps.hasTransport(NetworkCapabilities.TRANSPORT_VPN)) {
            kind = "vpn";
        }
        boolean validated = caps.hasCapability(NetworkCapabilities.NET_CAPABILITY_VALIDATED);
        return "network " + kind + (validated ? "" : ", no internet through it");
    }

    private static String storage(Context ctx) {
        StatFs fs = new StatFs(ctx.getFilesDir().getPath());
        return String.format(
                Locale.US,
                "storage %.1f GB free of %.1f GB",
                gigabytes(fs.getAvailableBytes()),
                gigabytes(fs.getTotalBytes()));
    }

    private static double gigabytes(long bytes) {
        return bytes / 1_000_000_000.0;
    }
}
