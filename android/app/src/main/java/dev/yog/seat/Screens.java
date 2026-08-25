package dev.yog.seat;

import android.accessibilityservice.AccessibilityService;
import android.graphics.Bitmap;
import android.hardware.HardwareBuffer;
import android.view.Display;

import java.io.FileOutputStream;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.TimeUnit;

/**
 * A screenshot, written to this app's own storage.
 *
 * <h2>What comes back is a path, not an image</h2>
 *
 * A capture is text (REMOTE §5.3). Encoding a phone screenshot into it would
 * be this client adding a shape to the boundary, and a megabyte of base64 is
 * not something a model can read anyway. So the file lands in the app's
 * private storage and the capture names it, its size and its dimensions —
 * whoever wants the image can fetch it over the bridge they already have.
 * What a model should read is {@code ui_read}: the same screen, as words.
 */
final class Screens {
    private Screens() {}

    /** How long the answer waits for the platform's callback. */
    private static final long PATIENCE_S = 10;

    static String capture(AccessibilityService service, String path) {
        CountDownLatch done = new CountDownLatch(1);
        String[] answer = {YogAccessibilityService.ERR + "the screenshot was never answered for"};
        service.takeScreenshot(
                Display.DEFAULT_DISPLAY,
                Runnable::run,
                new AccessibilityService.TakeScreenshotCallback() {
                    @Override
                    public void onSuccess(AccessibilityService.ScreenshotResult shot) {
                        answer[0] = save(shot, path);
                        done.countDown();
                    }

                    @Override
                    public void onFailure(int code) {
                        answer[0] =
                                YogAccessibilityService.ERR
                                        + "the platform refused the screenshot (code "
                                        + code
                                        + "); a secure window may be in front.";
                        done.countDown();
                    }
                });
        try {
            if (!done.await(PATIENCE_S, TimeUnit.SECONDS)) {
                return YogAccessibilityService.ERR + "the screenshot timed out";
            }
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
            return YogAccessibilityService.ERR + "the wait for the screenshot was interrupted";
        }
        return answer[0];
    }

    /** The buffer as a PNG on disk, and the sentence naming it. */
    private static String save(AccessibilityService.ScreenshotResult shot, String path) {
        try (HardwareBuffer buffer = shot.getHardwareBuffer()) {
            Bitmap bitmap = Bitmap.wrapHardwareBuffer(buffer, shot.getColorSpace());
            if (bitmap == null) {
                return YogAccessibilityService.ERR + "the screenshot buffer could not be read";
            }
            // A hardware bitmap cannot be compressed directly on every device;
            // the software copy is what makes the write portable.
            Bitmap copy = bitmap.copy(Bitmap.Config.ARGB_8888, false);
            bitmap.recycle();
            if (copy == null) {
                return YogAccessibilityService.ERR + "the screenshot could not be copied";
            }
            try (FileOutputStream out = new FileOutputStream(path)) {
                copy.compress(Bitmap.CompressFormat.PNG, 100, out);
            }
            String size = copy.getWidth() + "x" + copy.getHeight();
            long bytes = new java.io.File(path).length();
            copy.recycle();
            return YogAccessibilityService.OK
                    + "wrote a "
                    + size
                    + " screenshot, "
                    + bytes
                    + " bytes, to "
                    + path;
        } catch (Exception e) {
            return YogAccessibilityService.ERR + "the screenshot failed: " + e;
        }
    }
}
