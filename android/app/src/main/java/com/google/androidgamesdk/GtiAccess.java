package com.google.androidgamesdk;

import com.google.androidgamesdk.gametextinput.InputConnection;

/**
 * Reaches {@code GameActivity.InputEnabledSurfaceView.mInputConnection}, which
 * games-activity declares package-private. Nothing else lives in this package;
 * the class exists only so {@code MainActivity} can get the InputConnection
 * without reflection.
 */
public final class GtiAccess {
    private GtiAccess() {}

    public static InputConnection inputConnection(GameActivity.InputEnabledSurfaceView v) {
        return v == null ? null : v.mInputConnection;
    }
}
