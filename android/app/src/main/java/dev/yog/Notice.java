package dev.yog;

import android.app.Notification;
import android.os.Bundle;
import android.service.notification.StatusBarNotification;

/**
 * One notification, as the lines a model reads.
 *
 * <h2>The shape, and why it is indented</h2>
 *
 * A record is one unindented header — the app that posted it, how long ago, and
 * whether it is a standing row rather than an event — followed by its title and
 * text, every line of them indented two spaces. The indent is what makes the
 * record boundary unambiguous: a notification's own text may contain blank
 * lines, and a reader that split on those would read one message as two.
 *
 * <h2>The package name, not the app's label</h2>
 *
 * {@code com.example.messaging} rather than "Messages": the label is localized
 * and an app may set it to anything, while the package is what the device
 * actually installed. A model deciding whether a code came from the bank or
 * from something imitating it needs the half that cannot be dressed up.
 *
 * <h2>The age, always</h2>
 *
 * The same rule {@link Position} keeps for a fix, for the same reason: a code
 * that arrived four minutes ago and one that arrived four hours ago are
 * different facts, and only one of them is worth typing. It is spelled by
 * {@link Span}, which both callers share.
 */
final class Notice {
    private Notice() {}

    /** What a notification that draws its own view says instead of text. */
    private static final String NO_TEXT =
            "(no text: this notification draws its own view, so only its app knows what it "
                    + "says)";

    /** One record, ending in a newline. `now` is the wall clock, read once. */
    static String of(StatusBarNotification posted, long now) {
        StringBuilder out = new StringBuilder(posted.getPackageName());
        out.append("  ").append(Span.spell((now - posted.getPostTime()) / 1000L)).append(" ago");
        if (posted.isOngoing()) {
            out.append("  ongoing");
        }
        out.append('\n');
        Notification held = posted.getNotification();
        Bundle extras = held == null ? null : held.extras;
        String title = read(extras, Notification.EXTRA_TITLE);
        String text = read(extras, Notification.EXTRA_TEXT);
        if (text.isEmpty()) {
            // The expanded body, which is where a long message lives when the
            // collapsed line is empty — a message app's own shape.
            text = read(extras, Notification.EXTRA_BIG_TEXT);
        }
        if (title.isEmpty() && text.isEmpty()) {
            return out.append(indented(NO_TEXT)).toString();
        }
        if (!title.isEmpty()) {
            out.append(indented(title));
        }
        if (!text.isEmpty()) {
            out.append(indented(text));
        }
        return out.toString();
    }

    /** One extra as a string, or empty — a missing bundle included. */
    private static String read(Bundle extras, String key) {
        if (extras == null) {
            return "";
        }
        CharSequence said = extras.getCharSequence(key);
        return said == null ? "" : said.toString().trim();
    }

    /** Every line of it under the header, so a blank line cannot split a record. */
    private static String indented(String body) {
        StringBuilder out = new StringBuilder();
        for (String line : body.split("\n", -1)) {
            out.append("  ").append(line).append('\n');
        }
        return out.toString();
    }
}
