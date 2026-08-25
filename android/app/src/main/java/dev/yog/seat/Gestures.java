package dev.yog.seat;

import android.accessibilityservice.AccessibilityService;
import android.accessibilityservice.GestureDescription;
import android.graphics.Path;

import java.util.concurrent.CountDownLatch;
import java.util.concurrent.TimeUnit;

/**
 * Driving the screen: one tap, dispatched and waited for.
 *
 * <h2>Why it waits</h2>
 *
 * {@code dispatchGesture} is asynchronous, and the tool-host loop that called
 * it is about to post a capture saying what happened. A fire-and-forget tap
 * would answer "done" before the gesture had been accepted, so a session
 * reading the capture would take a refusal for a success and act on it. The
 * latch is what makes the answer true.
 */
final class Gestures {
    private Gestures() {}

    /** How long a tap is held. Long enough to register, short of a long-press. */
    private static final long PRESS_MS = 60;

    /** How long the answer waits for the platform's callback. */
    private static final long PATIENCE_S = 5;

    static String tap(AccessibilityService service, int x, int y) {
        Path path = new Path();
        path.moveTo(x, y);
        GestureDescription gesture =
                new GestureDescription.Builder()
                        .addStroke(new GestureDescription.StrokeDescription(path, 0, PRESS_MS))
                        .build();
        CountDownLatch done = new CountDownLatch(1);
        boolean[] landed = {false};
        boolean dispatched =
                service.dispatchGesture(
                        gesture,
                        new AccessibilityService.GestureResultCallback() {
                            @Override
                            public void onCompleted(GestureDescription description) {
                                landed[0] = true;
                                done.countDown();
                            }

                            @Override
                            public void onCancelled(GestureDescription description) {
                                done.countDown();
                            }
                        },
                        null);
        if (!dispatched) {
            return YogAccessibilityService.ERR + "the platform refused the gesture";
        }
        try {
            if (!done.await(PATIENCE_S, TimeUnit.SECONDS)) {
                return YogAccessibilityService.ERR + "the tap was never answered for";
            }
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
            return YogAccessibilityService.ERR + "the wait for the tap was interrupted";
        }
        return landed[0]
                ? YogAccessibilityService.OK + "tapped " + x + "," + y
                : YogAccessibilityService.ERR + "the tap was cancelled";
    }
}
