package dev.yog;

import android.content.Context;
import android.graphics.ImageFormat;
import android.hardware.camera2.CameraAccessException;
import android.hardware.camera2.CameraCaptureSession;
import android.hardware.camera2.CameraCharacteristics;
import android.hardware.camera2.CameraDevice;
import android.hardware.camera2.CameraManager;
import android.hardware.camera2.CaptureRequest;
import android.media.Image;
import android.media.ImageReader;
import android.os.Handler;
import android.os.HandlerThread;
import android.util.Size;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.TimeUnit;

/**
 * The camera2 lifecycle behind one still: open the camera {@link Lens} chose
 * into a JPEG {@link ImageReader}, take a short burst, and hand the last frame
 * to {@link Jpeg}.
 *
 * <h2>An instance, not statics</h2>
 *
 * {@link Session} is static because the scan screen streams one camera for as
 * long as the screen is up. A still is one bounded call on the tool-host
 * thread, which runs invocations serially (REMOTE §5.3), so all of this
 * lives on the instance and dies with it — nothing to reset, nothing to leak
 * into the next call.
 *
 * <h2>Why a burst and not one frame</h2>
 *
 * A camera opened a moment ago has metered nothing: the first frame off a cold
 * sensor is usually dark and unfocused, and a tool that answered with it would
 * answer with a black rectangle and call it a photograph. Three frames is what
 * auto-exposure and continuous focus want, and the last one is the answer. The
 * earlier two are acquired and closed rather than left in the reader, because
 * a reader holding its limit stops delivering.
 *
 * <p>Every failure — including the asynchronous ones camera2 reports on its own
 * threads, long after the call that started them returned — becomes one
 * sentence in {@link InterfaceService}'s two-line protocol, which is the
 * discipline {@link Session} already keeps.
 */
final class Shot {
    /** Frames captured; the last is the answer and the rest are the sensor settling. */
    private static final int BURST = 3;

    /** How long the answer waits for the platform's callbacks. */
    private static final long PATIENCE_S = 20;

    private final String lens;
    private final String path;
    private final CountDownLatch done = new CountDownLatch(1);
    private final String[] said = {App.ERR + "the camera never answered."};

    private CameraDevice device;
    private CameraCaptureSession session;
    private ImageReader reader;
    private HandlerThread thread;
    private int taken;

    Shot(String lens, String path) {
        this.lens = lens;
        this.path = path;
    }

    /** Take it, or say what stopped it. */
    String take(Context ctx) {
        CameraManager cameras = ctx.getSystemService(CameraManager.class);
        if (cameras == null) {
            return App.ERR + "this device has no camera service.";
        }
        try {
            String id = Lens.facing(cameras, lens);
            if (id == null) {
                return App.ERR + "this device has no " + lens + "-facing camera.";
            }
            CameraCharacteristics traits = cameras.getCameraCharacteristics(id);
            Size size = Lens.size(traits);
            if (size == null) {
                return App.ERR + "this camera offers no still image size.";
            }
            open(cameras, id, traits, size);
            if (!done.await(PATIENCE_S, TimeUnit.SECONDS)) {
                return App.ERR
                        + "the camera did not deliver a photograph within "
                        + PATIENCE_S
                        + " seconds.";
            }
        } catch (CameraAccessException
                | SecurityException
                | IllegalArgumentException
                | IllegalStateException e) {
            return App.ERR + "the camera would not open: " + e;
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
            return App.ERR + "the wait for the photograph was interrupted.";
        } finally {
            shut();
        }
        return said[0];
    }

    private void open(CameraManager cameras, String id, CameraCharacteristics traits, Size size)
            throws CameraAccessException {
        thread = new HandlerThread("yog-still");
        thread.start();
        Handler handler = new Handler(thread.getLooper());
        reader = ImageReader.newInstance(size.getWidth(), size.getHeight(), ImageFormat.JPEG, BURST);
        reader.setOnImageAvailableListener(this::took, handler);
        cameras.openCamera(id, opened(traits, handler), handler);
    }

    /** One delivered frame; the last of the burst is the one that is kept. */
    private void took(ImageReader from) {
        try (Image image = from.acquireNextImage()) {
            if (image == null) {
                return;
            }
            taken++;
            if (taken < BURST) {
                return;
            }
            said[0] = Jpeg.save(image, lens, path);
        } catch (RuntimeException | OutOfMemoryError e) {
            said[0] = App.ERR + "the photograph could not be read: " + e;
        } finally {
            if (taken >= BURST) {
                done.countDown();
            }
        }
    }

    private CameraDevice.StateCallback opened(CameraCharacteristics traits, Handler handler) {
        return new CameraDevice.StateCallback() {
            @Override
            public void onOpened(CameraDevice open) {
                device = open;
                configure(open, traits, handler);
            }

            @Override
            public void onDisconnected(CameraDevice open) {
                failed("the camera was taken by something else");
            }

            @Override
            public void onError(CameraDevice open, int error) {
                failed("the camera reported error " + error);
            }
        };
    }

    private void configure(CameraDevice open, CameraCharacteristics traits, Handler handler) {
        try {
            CaptureRequest.Builder request =
                    open.createCaptureRequest(CameraDevice.TEMPLATE_STILL_CAPTURE);
            request.addTarget(reader.getSurface());
            request.set(CaptureRequest.CONTROL_AE_MODE, CaptureRequest.CONTROL_AE_MODE_ON);
            request.set(
                    CaptureRequest.CONTROL_AF_MODE,
                    CaptureRequest.CONTROL_AF_MODE_CONTINUOUS_PICTURE);
            // The sensor's own mounting, which is the upright answer for a
            // phone held or lying flat. Nothing here reads the display
            // rotation: the picture is not this app's window.
            Integer mounted = traits.get(CameraCharacteristics.SENSOR_ORIENTATION);
            request.set(CaptureRequest.JPEG_ORIENTATION, mounted == null ? 0 : mounted);
            CaptureRequest one = request.build();
            List<CaptureRequest> burst = new ArrayList<>();
            for (int i = 0; i < BURST; i++) {
                burst.add(one);
            }
            open.createCaptureSession(
                    Arrays.asList(reader.getSurface()), shooting(burst, handler), handler);
        } catch (CameraAccessException | IllegalStateException e) {
            failed("the camera would not start: " + e);
        }
    }

    private CameraCaptureSession.StateCallback shooting(
            List<CaptureRequest> burst, Handler handler) {
        return new CameraCaptureSession.StateCallback() {
            @Override
            public void onConfigured(CameraCaptureSession made) {
                session = made;
                try {
                    made.captureBurst(burst, null, handler);
                } catch (CameraAccessException | IllegalStateException e) {
                    failed("the camera refused the capture: " + e);
                }
            }

            @Override
            public void onConfigureFailed(CameraCaptureSession made) {
                failed("the camera refused a capture session");
            }
        };
    }

    private void failed(String why) {
        said[0] = App.ERR + why + ".";
        done.countDown();
    }

    private void shut() {
        Session.shut(session);
        Session.shut(device);
        Session.shut(reader);
        if (thread != null) {
            thread.quitSafely();
        }
    }
}
