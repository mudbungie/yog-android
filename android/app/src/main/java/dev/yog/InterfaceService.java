package dev.yog;

import android.accessibilityservice.AccessibilityService;
import android.view.accessibility.AccessibilityEvent;
import android.view.accessibility.AccessibilityNodeInfo;

/**
 * The interface half of this device's tool hosting: reading what is on the
 * screen and driving it.
 *
 * <h2>Why a service at all</h2>
 *
 * Established by probe rather than assumption: an app uid cannot screenshot
 * ({@code screencap} needs a signature-level permission and MediaProjection
 * wants a consent dialog and a foreground service per session), and it cannot
 * see or touch another app's views at all. An AccessibilityService carries all
 * three capabilities in one place since API 30 — {@code takeScreenshot}, the
 * node tree of whatever is in front, and gesture dispatch.
 *
 * <h2>Enabling it is the operator's act, and that is the trust model</h2>
 *
 * The app never grants itself anything. The operator turns this on in system
 * settings, or an already-trusted device turns it on over the physically
 * attached debug bridge — the same channel that carries this seat's key
 * material. Until then every interface tool refuses in band with a sentence
 * naming the fix, which is the boundary's own staleness correction: a client
 * refuses a tool it cannot presently carry.
 *
 * <h2>The contract with the Rust side</h2>
 *
 * Every entry point is a static method returning one String, either
 * {@code "ok\n<payload>"} or {@code "err\n<sentence>"}. Two lines rather than
 * an exception because a thrown exception across JNI must be checked for and
 * cleared at every call site, and one that is missed aborts the process under
 * CheckJNI; a prefix cannot be forgotten. The split is parsed on the Rust side
 * by a pure function the suite tests.
 */
public class InterfaceService extends AccessibilityService {
    /** The prefix a successful answer carries. */
    static final String OK = "ok\n";

    /** The prefix a refusal carries. */
    static final String ERR = "err\n";

    /**
     * The running service, or null when the operator has not enabled it.
     * Written on connect and cleared on unbind, read from the tool-host
     * thread — volatile because those are different threads and a stale read
     * here is a call against a service that has gone away.
     */
    private static volatile InterfaceService live;

    @Override
    protected void onServiceConnected() {
        super.onServiceConnected();
        live = this;
    }

    @Override
    public boolean onUnbind(android.content.Intent intent) {
        live = null;
        return super.onUnbind(intent);
    }

    @Override
    public void onAccessibilityEvent(AccessibilityEvent event) {
        // Nothing is driven by events: every tool is a question asked at the
        // moment a session asks it. Declaring no event types in the service
        // config keeps this from being called at all in the ordinary case.
    }

    @Override
    public void onInterrupt() {}

    /** The service, or null — the one place the enabled check is spelled. */
    static InterfaceService live() {
        return live;
    }

    /** The sentence a tool earns when the operator has not enabled this. */
    static String notEnabled() {
        return ERR
                + "the accessibility service is not enabled on this device. Enable "
                + "\"yog\" under Settings > Accessibility > Installed apps, or have a "
                + "trusted device enable it over the debug bridge.";
    }

    /** Read the interface in front, as text. */
    public static String uiRead() {
        InterfaceService service = live;
        if (service == null) {
            return notEnabled();
        }
        AccessibilityNodeInfo root = service.getRootInActiveWindow();
        if (root == null) {
            return ERR
                    + "nothing readable is in front: the window is secure, or no window "
                    + "has focus right now.";
        }
        StringBuilder out = new StringBuilder();
        UiTree.walk(root, 0, out);
        return OK + out;
    }

    /** Press one of the system's own controls. */
    public static String uiKey(String action) {
        InterfaceService service = live;
        if (service == null) {
            return notEnabled();
        }
        int code;
        switch (action) {
            case "back": code = GLOBAL_ACTION_BACK; break;
            case "home": code = GLOBAL_ACTION_HOME; break;
            case "recents": code = GLOBAL_ACTION_RECENTS; break;
            case "notifications": code = GLOBAL_ACTION_NOTIFICATIONS; break;
            case "quick-settings": code = GLOBAL_ACTION_QUICK_SETTINGS; break;
            default:
                return ERR + "no such key " + action + "; try back, home, recents, "
                        + "notifications or quick-settings.";
        }
        return service.performGlobalAction(code) ? OK + action : ERR + action + " was refused";
    }

    /** Type into whatever field holds focus. */
    public static String uiText(String text) {
        InterfaceService service = live;
        if (service == null) {
            return notEnabled();
        }
        AccessibilityNodeInfo focused = service.findFocus(AccessibilityNodeInfo.FOCUS_INPUT);
        if (focused == null) {
            return ERR + "no text field holds focus; tap one first.";
        }
        android.os.Bundle args = new android.os.Bundle();
        args.putCharSequence(
                AccessibilityNodeInfo.ACTION_ARGUMENT_SET_TEXT_CHARSEQUENCE, text);
        boolean done =
                focused.performAction(AccessibilityNodeInfo.ACTION_SET_TEXT, args);
        return done ? OK + "typed " + text.length() + " characters"
                    : ERR + "the focused field refused the text";
    }

    /** Tap a point, in screen pixels. */
    public static String uiTap(int x, int y) {
        InterfaceService service = live;
        if (service == null) {
            return notEnabled();
        }
        return Gestures.tap(service, x, y);
    }

    /** Tap the first node whose text or description matches. */
    public static String uiTapText(String needle) {
        InterfaceService service = live;
        if (service == null) {
            return notEnabled();
        }
        AccessibilityNodeInfo root = service.getRootInActiveWindow();
        if (root == null) {
            return ERR + "nothing readable is in front to search.";
        }
        AccessibilityNodeInfo hit = UiTree.find(root, needle);
        if (hit == null) {
            return ERR + "no node matching " + needle + " is on screen.";
        }
        android.graphics.Rect bounds = new android.graphics.Rect();
        hit.getBoundsInScreen(bounds);
        return Gestures.tap(service, bounds.centerX(), bounds.centerY());
    }

    /** Save a screenshot, and answer where it went. */
    public static String screenshot(String path) {
        InterfaceService service = live;
        if (service == null) {
            return notEnabled();
        }
        return Screens.capture(service, path);
    }
}
