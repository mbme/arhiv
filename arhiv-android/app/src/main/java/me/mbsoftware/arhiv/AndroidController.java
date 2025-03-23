package me.mbsoftware.arhiv;

import android.os.Handler;
import android.os.Looper;
import android.util.Log;

import androidx.fragment.app.FragmentActivity;

public class AndroidController {
  private static final String TAG = "AndroidController";

  private final FragmentActivity context;

  public AndroidController(FragmentActivity context) {
    this.context = context;
  }

  public void savePassword(String password) {
    if (!Keyring.isDeviceSecure(context)) {
      Log.w(TAG, "Can't save password: device is not secure");
      return;
    }

    if (password == null) {
      Log.i(TAG, "Erasing password");
      Keyring.erasePassword(context);
      return;
    }

    new Handler(Looper.getMainLooper()).post(() -> {
      try {
        Keyring.savePassword(context, password);
      } catch (Exception e) {
        Log.e(TAG, "Failed to save password: ", e);
      }
    });
  }
}
