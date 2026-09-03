package dev.yog;

import android.graphics.ImageFormat;
import android.hardware.camera2.CameraAccessException;
import android.hardware.camera2.CameraCharacteristics;
import android.hardware.camera2.CameraManager;
import android.hardware.camera2.params.StreamConfigurationMap;
import android.util.Size;

/**
 * Which camera, and how big a picture off it — the two questions a still asks
 * the device's own characteristics before anything is opened.
 *
 * <h2>The size is bounded on purpose</h2>
 *
 * The largest offered size inside {@link #MAX_EDGE}, falling to the smallest
 * offered when a camera has nothing that small. A full-resolution still off a
 * modern phone is several megabytes written into private storage nobody is
 * watching, for a picture whose whole use is being looked at once — and the
 * answer that crosses the wire is the path either way (REMOTE §5.3), so the
 * pixels bought nothing a reader will ever see.
 *
 * <p>Both answers are null when the device cannot give one, never a guess:
 * {@link Shot} turns each null into its own sentence, which is the same
 * discipline {@link Device} keeps for a battery that reports no level.
 */
final class Lens {
    private Lens() {}

    /** The longest edge a still is allowed, in pixels. */
    private static final int MAX_EDGE = 1920;

    /** The camera facing the way this call asked, or null when there is none. */
    static String facing(CameraManager cameras, String lens) throws CameraAccessException {
        int wanted =
                "front".equals(lens)
                        ? CameraCharacteristics.LENS_FACING_FRONT
                        : CameraCharacteristics.LENS_FACING_BACK;
        for (String id : cameras.getCameraIdList()) {
            Integer points =
                    cameras.getCameraCharacteristics(id).get(CameraCharacteristics.LENS_FACING);
            if (points != null && points == wanted) {
                return id;
            }
        }
        return null;
    }

    /** The largest offered JPEG size inside {@link #MAX_EDGE}, or the smallest there is. */
    static Size size(CameraCharacteristics traits) {
        StreamConfigurationMap map =
                traits.get(CameraCharacteristics.SCALER_STREAM_CONFIGURATION_MAP);
        Size[] offered = map == null ? null : map.getOutputSizes(ImageFormat.JPEG);
        if (offered == null || offered.length == 0) {
            return null;
        }
        Size best = null;
        Size smallest = offered[0];
        for (Size size : offered) {
            boolean fits = size.getWidth() <= MAX_EDGE && size.getHeight() <= MAX_EDGE;
            if (fits && (best == null || pixels(size) > pixels(best))) {
                best = size;
            }
            if (pixels(size) < pixels(smallest)) {
                smallest = size;
            }
        }
        return best == null ? smallest : best;
    }

    private static long pixels(Size size) {
        return (long) size.getWidth() * size.getHeight();
    }
}
