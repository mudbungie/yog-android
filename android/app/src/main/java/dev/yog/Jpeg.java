package dev.yog;

import android.media.Image;

import java.io.FileOutputStream;
import java.io.IOException;
import java.nio.ByteBuffer;

/**
 * One delivered frame, on disk, and the sentence that names it — the still's
 * answer, and the whole of what crosses the wire.
 *
 * <h2>A path, never the image</h2>
 *
 * A capture is text (REMOTE §5.3). Encoding a photograph into one would be
 * this client adding a shape to the boundary, and a megabyte of base64 is not
 * something a model can read anyway. So the file lands in the app's private
 * storage and the capture names it, its size, its dimensions and which camera
 * took it; whoever wants the picture fetches it off the device. This is
 * {@link Screens}' own answer for the screenshot, given again because it is
 * the same question.
 *
 * <p>A JPEG off an {@code ImageReader} is one plane and it is already encoded —
 * there is no bitmap, no colour space and no compression step here, which is
 * the whole difference between this and the screenshot's hardware buffer.
 */
final class Jpeg {
    private Jpeg() {}

    /** Write this frame, and say what landed. */
    static String save(Image image, String lens, String path) {
        ByteBuffer buffer = image.getPlanes()[0].getBuffer();
        byte[] jpeg = new byte[buffer.remaining()];
        buffer.get(jpeg);
        try (FileOutputStream out = new FileOutputStream(path)) {
            out.write(jpeg);
        } catch (IOException | RuntimeException e) {
            return App.ERR + "the photograph could not be written to " + path + ": " + e;
        }
        return App.OK
                + "wrote a "
                + image.getWidth()
                + "x"
                + image.getHeight()
                + " JPEG from the "
                + lens
                + " camera, "
                + jpeg.length
                + " bytes, to "
                + path;
    }
}
