package me.mbsoftware.arhiv;

public class ArhivServer {
  public static native ServerInfo startServer(
    String appFilesDir,
    String externalStorageDir,
    String password,
    AndroidController controller
  );

  public static native void stopServer();

  static {
    System.loadLibrary("arhiv_android");
  }
}
