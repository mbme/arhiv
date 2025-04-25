package me.mbsoftware.arhiv;

import androidx.annotation.NonNull;

public class ArhivServer {
  public static native ServerInfo startServer(
    @NonNull String appFilesDir,
    @NonNull String externalStorageDir,
    @NonNull String downloadsDir,
    String password,
    @NonNull AndroidController controller
  );

  public static native void stopServer();

  static {
    System.loadLibrary("arhiv_android");
  }
}
