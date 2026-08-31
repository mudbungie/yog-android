package dev.yog;

import android.graphics.Rect;
import android.view.accessibility.AccessibilityNodeInfo;

/**
 * The interface in front, as text.
 *
 * <h2>Why text and not an image</h2>
 *
 * A capture is text (REMOTE §5.3), and a text outline is the form a model can
 * actually act on: it carries the words on screen, what is tappable, and where
 * each thing is, which is everything a following {@code ui_tap} needs. A
 * screenshot is a thing a person looks at, and it is a separate tool.
 *
 * <h2>What a line says</h2>
 *
 * One node per line, indented by depth:
 * {@code <indent><class> "<text>" [desc] (x,y w×h) +clickable +editable}.
 * A node with nothing to say — no text, no description, not interactive — is
 * skipped along with its own line but never with its children: the layout
 * containers of a modern app are almost all of the tree, and printing them
 * would bury the handful of nodes that matter.
 */
final class UiTree {
    private UiTree() {}

    /** How deep the walk goes before it stops, whatever the tree does. */
    private static final int MAX_DEPTH = 40;

    /** How many nodes the walk emits before it stops. */
    private static final int MAX_NODES = 400;

    /** How much of one node's text is printed. */
    private static final int TEXT_CAP = 120;

    private static int emitted;

    /** Walk `node` and its children into `out`. Depth 0 resets the budget. */
    static void walk(AccessibilityNodeInfo node, int depth, StringBuilder out) {
        if (depth == 0) {
            emitted = 0;
        }
        if (node == null || depth > MAX_DEPTH || emitted >= MAX_NODES) {
            return;
        }
        String line = describe(node, depth);
        if (line != null) {
            out.append(line).append('\n');
            emitted++;
        }
        for (int i = 0; i < node.getChildCount(); i++) {
            walk(node.getChild(i), depth + 1, out);
        }
    }

    /** The first node whose text or description contains `needle`. */
    static AccessibilityNodeInfo find(AccessibilityNodeInfo node, String needle) {
        if (node == null) {
            return null;
        }
        if (node.isClickable() && matches(node, needle)) {
            return node;
        }
        for (int i = 0; i < node.getChildCount(); i++) {
            AccessibilityNodeInfo hit = find(node.getChild(i), needle);
            if (hit != null) {
                return hit;
            }
        }
        return null;
    }

    private static boolean matches(AccessibilityNodeInfo node, String needle) {
        String text = str(node.getText());
        String desc = str(node.getContentDescription());
        return text.contains(needle) || desc.contains(needle);
    }

    /** One node's line, or null when it has nothing worth a line. */
    private static String describe(AccessibilityNodeInfo node, int depth) {
        String text = clip(str(node.getText()));
        String desc = clip(str(node.getContentDescription()));
        boolean interactive = node.isClickable() || node.isEditable() || node.isCheckable();
        if (text.isEmpty() && desc.isEmpty() && !interactive) {
            return null;
        }
        Rect bounds = new Rect();
        node.getBoundsInScreen(bounds);
        StringBuilder line = new StringBuilder();
        for (int i = 0; i < depth; i++) {
            line.append(' ');
        }
        line.append(shortClass(str(node.getClassName())));
        if (!text.isEmpty()) {
            line.append(" \"").append(text).append('"');
        }
        if (!desc.isEmpty()) {
            line.append(" [").append(desc).append(']');
        }
        line.append(" (").append(bounds.left).append(',').append(bounds.top)
            .append(' ').append(bounds.width()).append('x').append(bounds.height())
            .append(')');
        if (node.isClickable()) {
            line.append(" +clickable");
        }
        if (node.isEditable()) {
            line.append(" +editable");
        }
        if (node.isCheckable()) {
            line.append(node.isChecked() ? " +checked" : " +unchecked");
        }
        if (!node.isEnabled()) {
            line.append(" +disabled");
        }
        return line.toString();
    }

    /** The class's own name without its package: the package is never news. */
    private static String shortClass(String name) {
        int cut = name.lastIndexOf('.');
        return cut < 0 ? name : name.substring(cut + 1);
    }

    private static String str(CharSequence value) {
        return value == null ? "" : value.toString().replace('\n', ' ');
    }

    private static String clip(String value) {
        return value.length() <= TEXT_CAP ? value : value.substring(0, TEXT_CAP) + "…";
    }
}
