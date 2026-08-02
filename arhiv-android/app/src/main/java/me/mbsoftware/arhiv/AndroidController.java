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

  public void saveStorageKey(String storageKey) {
    if (storageKey == null) {
      Log.i(TAG, "Erasing cached storage key");
      if (!Keyring.eraseStorageKey(context)) {
        throw new IllegalStateException("Failed to erase cached storage key");
      }
      return;
    }

    if (!Keyring.isDeviceSecure(context)) {
      Log.w(TAG, "Can't save storage key: device is not secure");
      return;
    }

    new Handler(Looper.getMainLooper()).post(() -> {
      try {
        Keyring.saveStorageKey(context, storageKey);
      } catch (Exception e) {
        Log.e(TAG, "Failed to save storage key: ", e);
      }
    });
  }
}
