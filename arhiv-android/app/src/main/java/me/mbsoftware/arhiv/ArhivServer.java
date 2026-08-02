package me.mbsoftware.arhiv;

import androidx.annotation.NonNull;

public class ArhivServer {
  public static native ServerStartResult startServer(
    @NonNull String appFilesDir,
    @NonNull String externalStorageDir,
    @NonNull String downloadsDir,
    String storageKey,
    @NonNull AndroidController controller
  );

  public static native String stopServer();

  static {
    System.loadLibrary("arhiv_android");
  }
}
