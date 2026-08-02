package me.mbsoftware.arhiv;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertNull;

import android.net.Uri;
import android.webkit.ValueCallback;

import org.junit.Test;

public class FileChooserRequestTest {
  @Test
  public void replacingRequestCancelsPreviousRequest() {
    FileChooserRequest request = new FileChooserRequest();
    CapturingCallback first = new CapturingCallback();
    CapturingCallback second = new CapturingCallback();

    request.replace(first);
    request.replace(second);

    assertEquals(1, first.calls);
    assertNull(first.results);
    assertEquals(0, second.calls);
  }

  @Test
  public void completingRequestOnlyCallsCallbackOnce() {
    FileChooserRequest request = new FileChooserRequest();
    CapturingCallback callback = new CapturingCallback();

    request.replace(callback);
    request.complete(null);
    request.complete(null);

    assertEquals(1, callback.calls);
    assertNull(callback.results);
  }

  private static final class CapturingCallback implements ValueCallback<Uri[]> {
    int calls;
    Uri[] results;

    @Override
    public void onReceiveValue(Uri[] value) {
      calls++;
      results = value;
    }
  }
}