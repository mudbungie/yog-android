package dev.yog;

import android.app.Activity;
import android.graphics.ImageFormat;
import android.hardware.camera2.CameraAccessException;
import android.hardware.camera2.CameraCaptureSession;
import android.hardware.camera2.CameraCharacteristics;
import android.hardware.camera2.CameraDevice;
import android.hardware.camera2.CameraManager;
import android.hardware.camera2.CaptureRequest;
import android.hardware.camera2.params.StreamConfigurationMap;
import android.media.Image;
import android.media.ImageReader;
import android.os.Handler;
import android.os.HandlerThread;
import android.util.Size;

import java.util.Arrays;

/**
 * The camera2 lifecycle behind {@link Camera}: open one device, stream it into
 * an {@link ImageReader} on a thread of its own, and hand the newest frame over
 * when the native loop asks.
 *
 * <h2>Every failure becomes one sentence, and the sentence is the state</h2>
 *
 * camera2 reports asynchronously, on threads the caller does not own, long after
 * the call that started it returned. So nothing here throws upward: a failure is
 * recorded in {@link #problem} and {@link Camera#state} reads it back on the
 * next frame, which is the only moment the app is actually listening.
 */
final class Session {
    private Session() {}

    /**
     * The frame size asked for. A version-33 symbol is 149 modules across, and
     * 1280 × 720 leaves several pixels per module with the symbol at arm's
     * length; the decoder still reads it at 640 × 480, which is what a device
     * offering nothing larger falls to.
     */
    private static final int WANT_W = 1280;
    private static final int WANT_H = 720;

    // Written on the camera thread, read from the native frame loop.
    private static volatile String problem;
    private static volatile byte[] latest;
    private static volatile int turn;

    /**
     * The two frame buffers, written only on the camera thread (bl-d815).
     *
     * <p>Two, and alternating, so the writer never touches the array the
     * reader is copying: a frame is packed only while {@link #latest} is null,
     * so handing out one buffer means the next pack goes to the other, and the
     * one after that cannot happen until the reader has taken — and therefore
     * finished with — the first.
     *
     * <p>One buffer would tear a frame under a slow reader; a fresh array per
     * frame is what killed the app on the emulator, ~900 KB at the camera's
     * own rate until the Java heap gave out.
     */
    private static final byte[][] BUFFERS = new byte[2][];
    private static int slot;
    private static CameraDevice device;
    private static CameraCaptureSession session;
    private static ImageReader reader;
    private static HandlerThread thread;

    static String problem() {
        return problem;
    }

    static byte[] frame() {
        byte[] taken = latest;
        latest = null;
        return taken;
    }

    static String open(Activity activity) {
        close();
        problem = null;
        try {
            CameraManager cameras = activity.getSystemService(CameraManager.class);
            String id = back(cameras);
            CameraCharacteristics traits = cameras.getCameraCharacteristics(id);
            Integer orientation = traits.get(CameraCharacteristics.SENSOR_ORIENTATION);
            turn = orientation == null ? 0 : orientation;
            Size size = frameSize(traits);
            thread = new HandlerThread("yog-camera");
            thread.start();
            Handler handler = new Handler(thread.getLooper());
            reader =
                    ImageReader.newInstance(
                            size.getWidth(), size.getHeight(), ImageFormat.YUV_420_888, 2);
            reader.setOnImageAvailableListener(Session::took, handler);
            cameras.openCamera(id, opened(handler), handler);
            return "ok\n" + size.getWidth() + "x" + size.getHeight();
        } catch (CameraAccessException | SecurityException | IllegalArgumentException e) {
            close();
            return Camera.ERR + "the camera would not open: " + e;
        }
    }

    static String close() {
        latest = null;
        BUFFERS[0] = null;
        BUFFERS[1] = null;
        shut(session);
        shut(device);
        shut(reader);
        session = null;
        device = null;
        reader = null;
        if (thread != null) {
            thread.quitSafely();
            thread = null;
        }
        return "ok\n";
    }

    private static void shut(AutoCloseable it) {
        try {
            if (it != null) {
                it.close();
            }
        } catch (Exception ignored) {
            // A close that fails has nothing left to fail into; the handle is
            // dropped either way.
        }
    }

    /**
     * One delivered image. A frame is DROPPED while the reader still holds the
     * previous one — the camera streams faster than a scan screen consumes,
     * and packing what nobody will read is the whole of the allocation that
     * killed this app once.
     *
     * <p>Everything is caught, including {@code OutOfMemoryError}: this runs
     * on the camera's own {@link android.os.HandlerThread}, whose death is the
     * process's, and a sentence the operator can read beats a stack trace only
     * logcat sees.
     */
    private static void took(ImageReader from) {
        try (Image image = from.acquireLatestImage()) {
            if (image == null || latest != null) {
                return;
            }
            slot ^= 1;
            BUFFERS[slot] = Frames.pack(image, turn, BUFFERS[slot]);
            latest = BUFFERS[slot];
        } catch (RuntimeException | OutOfMemoryError e) {
            problem = "the camera stopped delivering frames: " + e;
        }
    }

    private static CameraDevice.StateCallback opened(Handler handler) {
        return new CameraDevice.StateCallback() {
            @Override
            public void onOpened(CameraDevice open) {
                device = open;
                configure(open, handler);
            }

            @Override
            public void onDisconnected(CameraDevice open) {
                problem = "the camera was taken by something else";
                close();
            }

            @Override
            public void onError(CameraDevice open, int error) {
                problem = "the camera reported error " + error;
                close();
            }
        };
    }

    private static void configure(CameraDevice open, Handler handler) {
        try {
            CaptureRequest.Builder request =
                    open.createCaptureRequest(CameraDevice.TEMPLATE_PREVIEW);
            request.addTarget(reader.getSurface());
            request.set(
                    CaptureRequest.CONTROL_AF_MODE,
                    CaptureRequest.CONTROL_AF_MODE_CONTINUOUS_PICTURE);
            open.createCaptureSession(
                    Arrays.asList(reader.getSurface()), streaming(request, handler), handler);
        } catch (CameraAccessException | IllegalStateException e) {
            problem = "the camera would not start: " + e;
        }
    }

    private static CameraCaptureSession.StateCallback streaming(
            CaptureRequest.Builder request, Handler handler) {
        return new CameraCaptureSession.StateCallback() {
            @Override
            public void onConfigured(CameraCaptureSession made) {
                session = made;
                try {
                    made.setRepeatingRequest(request.build(), null, handler);
                } catch (CameraAccessException | IllegalStateException e) {
                    problem = "the camera would not start streaming: " + e;
                }
            }

            @Override
            public void onConfigureFailed(CameraCaptureSession made) {
                problem = "the camera refused a capture session";
            }
        };
    }

    /** The back-facing camera, or the first one when a device names no facing. */
    private static String back(CameraManager cameras) throws CameraAccessException {
        String first = null;
        for (String id : cameras.getCameraIdList()) {
            if (first == null) {
                first = id;
            }
            Integer facing =
                    cameras.getCameraCharacteristics(id).get(CameraCharacteristics.LENS_FACING);
            if (facing != null && facing == CameraCharacteristics.LENS_FACING_BACK) {
                return id;
            }
        }
        if (first == null) {
            throw new IllegalArgumentException("this device has no camera");
        }
        return first;
    }

    /** The offered size closest to {@link #WANT_W} × {@link #WANT_H}. */
    private static Size frameSize(CameraCharacteristics traits) {
        StreamConfigurationMap map =
                traits.get(CameraCharacteristics.SCALER_STREAM_CONFIGURATION_MAP);
        Size[] offered = map == null ? null : map.getOutputSizes(ImageFormat.YUV_420_888);
        if (offered == null || offered.length == 0) {
            throw new IllegalArgumentException("this camera offers no readable frame size");
        }
        Size best = offered[0];
        for (Size size : offered) {
            if (cost(size) < cost(best)) {
                best = size;
            }
        }
        return best;
    }

    private static long cost(Size size) {
        return (long) Math.abs(size.getWidth() - WANT_W) + Math.abs(size.getHeight() - WANT_H);
    }
}
