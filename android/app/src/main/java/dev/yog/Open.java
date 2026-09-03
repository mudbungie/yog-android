package dev.yog;

import android.app.Activity;
import android.content.ActivityNotFoundException;
import android.content.Intent;
import android.net.Uri;

/**
 * The one typed way a tool puts something on this device's screen (DESIGN
 * §16.1): a URL to view, or text handed to the share sheet.
 *
 * <h2>Typed, on purpose</h2>
 *
 * A generic run-any-intent tool is a refused shape — REMOTE §5.2 turned that
 * wrapper meta-tool down twice, and the reasoning binds here: an intent
 * assembled from a model's own JSON is an unbounded act wearing one name.
 * Two kinds, two sentences, and a mis-call that names itself.
 *
 * <h2>The background-launch refusal is asked BEFORE it is met</h2>
 *
 * Android has refused an activity launch from an app that is not in front
 * since API 29, and reports nothing when it does — no exception, one line in
 * logcat. A tool that called {@code startActivity} anyway would answer "ok"
 * for an act that never happened, which is precisely the decoy this corpus's
 * editorial rule exists to exclude. So {@link App#front} is consulted first,
 * and the refusal names the act that fixes it.
 */
final class Open {
    private Open() {}

    /** Open {@code value} as {@code kind}, or say why nothing opened. */
    static String typed(String kind, String value) {
        Activity front = App.front();
        if (front == null) {
            return App.ERR
                    + "Android refuses an activity launch from an app that is not in front, "
                    + "and yog is not on this device's screen right now: bring yog to the "
                    + "front — notify can ask the operator to — and call again.";
        }
        Intent intent;
        if ("url".equals(kind)) {
            Uri uri = Uri.parse(value);
            if (uri.getScheme() == null) {
                return App.ERR
                        + "\""
                        + value
                        + "\" names no scheme, so nothing on this device can be asked to open "
                        + "it: state a full URI — https://…, tel:…, geo:….";
            }
            intent = new Intent(Intent.ACTION_VIEW, uri);
        } else {
            intent =
                    Intent.createChooser(
                            new Intent(Intent.ACTION_SEND)
                                    .setType("text/plain")
                                    .putExtra(Intent.EXTRA_TEXT, value),
                            null);
        }
        intent.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK);
        try {
            front.startActivity(intent);
        } catch (ActivityNotFoundException e) {
            return App.ERR + "this device has no app that opens " + value + ".";
        }
        return App.OK + "opened " + value;
    }
}
