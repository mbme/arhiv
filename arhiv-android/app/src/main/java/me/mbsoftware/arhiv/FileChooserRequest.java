package me.mbsoftware.arhiv;

import android.net.Uri;
import android.webkit.ValueCallback;

final class FileChooserRequest {
  private ValueCallback<Uri[]> callback;

  void replace(ValueCallback<Uri[]> nextCallback) {
    complete(null);
    callback = nextCallback;
  }

  void complete(Uri[] results) {
    if (callback == null) {
      return;
    }

    callback.onReceiveValue(results);
    callback = null;
  }
}