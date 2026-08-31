package dev.yog;

import android.media.Image;

import java.nio.ByteBuffer;

/**
 * One camera image, as the bytes the Rust side reads: a big-endian
 * {@code u16} width, a big-endian {@code u16} height, then width × height
 * bytes of 8-bit luminance.
 *
 * <h2>Only the Y plane, and no conversion</h2>
 *
 * A QR symbol is black on white, so chroma carries nothing a decoder wants,
 * and {@code YUV_420_888}'s Y plane already IS the 8-bit grayscale image the
 * binarizer takes. Converting to RGB would cost a second full-frame pass to
 * throw the result away.
 *
 * <h2>The buffer is handed in, because a frame a second is a megabyte a second
 * (bl-d815)</h2>
 *
 * A 1280 × 720 plane is ~900 KB, and allocating one per frame killed the app
 * outright on the emulator: {@code OutOfMemoryError} on the camera thread,
 * which is a thread whose death is the process's. The steady-state allocation
 * here is now zero — {@link Session} owns the buffers and passes one in, and a
 * new array is cut only when the frame size actually changes.
 *
 * <h2>The rotation is the sensor's, and it is applied here</h2>
 *
 * A back sensor reports its frames in its own landscape orientation, which on
 * a phone held upright is a quarter turn from what the operator sees. Turning
 * it here rather than at the glass keeps one buffer for both jobs: the pixels
 * the scan screen paints are the pixels the decoder reads, so a preview can
 * never show one thing while the decoder works on another. Decoding itself
 * does not need this — finder patterns are rotation-invariant — the preview
 * does.
 */
final class Frames {
    private Frames() {}

    /** The header the Rust side's slice pattern reads: two sides, two bytes each. */
    private static final int HEADER = 4;

    /**
     * Pack one image, turned by {@code degrees} (0, 90, 180 or 270 clockwise),
     * into {@code into} when it is exactly the right size and into a fresh
     * array otherwise. Returns null when the image is larger than the two-byte
     * sides can name, which nothing this app requests ever is.
     */
    static byte[] pack(Image image, int degrees, byte[] into) {
        int w = image.getWidth();
        int h = image.getHeight();
        if (w > 0xFFFF || h > 0xFFFF) {
            return null;
        }
        Image.Plane plane = image.getPlanes()[0];
        ByteBuffer in = plane.getBuffer();
        int rowStride = plane.getRowStride();
        int pixelStride = plane.getPixelStride();
        byte[] row = new byte[rowStride];
        boolean turned = degrees == 90 || degrees == 270;
        int outW = turned ? h : w;
        int outH = turned ? w : h;
        int want = HEADER + outW * outH;
        byte[] out = into != null && into.length == want ? into : new byte[want];
        out[0] = (byte) (outW >> 8);
        out[1] = (byte) outW;
        out[2] = (byte) (outH >> 8);
        out[3] = (byte) outH;
        for (int y = 0; y < h; y++) {
            in.position(y * rowStride);
            int take = Math.min(rowStride, in.remaining());
            in.get(row, 0, take);
            for (int x = 0; x < w; x++) {
                int at = x * pixelStride;
                if (at >= take) {
                    break;
                }
                out[HEADER + index(x, y, w, h, degrees, outW)] = row[at];
            }
        }
        return out;
    }

    /** Where the sensor's (x, y) lands in the turned frame. */
    private static int index(int x, int y, int w, int h, int degrees, int outW) {
        switch (degrees) {
            case 90:
                return x * outW + (h - 1 - y);
            case 180:
                return (h - 1 - y) * outW + (w - 1 - x);
            case 270:
                return (w - 1 - x) * outW + y;
            default:
                return y * outW + x;
        }
    }
}
