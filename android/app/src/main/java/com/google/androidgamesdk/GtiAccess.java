package com.google.androidgamesdk;

import com.google.androidgamesdk.gametextinput.InputConnection;

/**
 * Reaches {@code GameActivity.InputEnabledSurfaceView.mInputConnection}, which
 * games-activity declares package-private. Nothing else lives in this package;
 * the class exists only so {@code MainActivity} can get the InputConnection,
 * and seat its own ({@code dev.yog.Pasting}), without reflection.
 */
public final class GtiAccess {
    private GtiAccess() {}

    public static InputConnection inputConnection(GameActivity.InputEnabledSurfaceView v) {
        return v == null ? null : v.mInputConnection;
    }

    /**
     * Seat a connection on the view in place of the stock one — before
     * {@code GameActivity.onCreate} hands the view's connection to the native
     * side, which is the only window in which a swap is invisible to it.
     */
    public static void connect(GameActivity.InputEnabledSurfaceView v, InputConnection ic) {
        v.mInputConnection = ic;
    }
}
