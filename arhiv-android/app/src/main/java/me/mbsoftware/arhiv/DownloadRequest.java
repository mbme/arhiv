package me.mbsoftware.arhiv;

import android.net.Uri;
import android.widget.Toast;

import androidx.fragment.app.FragmentActivity;

import java.io.IOException;
import java.io.OutputStream;

import javax.net.ssl.SSLSocketFactory;
import javax.net.ssl.X509TrustManager;

import okhttp3.OkHttpClient;
import okhttp3.Request;
import okhttp3.Response;
import okio.BufferedSource;
import okio.Okio;

/**
 * Model a single download so it can be resumed once user picks a destination.
 */
class DownloadRequest {
  final String url, userAgent, cookie, filename, mimeType;
  final SSLSocketFactory sslFactory;
  final X509TrustManager trustManager;

  DownloadRequest(String url, String ua, String cookie,
                  String fn, String mt,
                  SSLSocketFactory sf, X509TrustManager tm) {
    this.url = url;
    this.userAgent = ua;
    this.cookie = cookie;
    this.filename = fn;
    this.mimeType = mt;
    this.sslFactory = sf;
    this.trustManager = tm;
  }

  /**
   * Perform the actual HTTP download into the user-chosen URI.
   */
  public void performDownloadToUri(FragmentActivity context, Uri destUri) {
    new Thread(() -> {
      OkHttpClient client = new OkHttpClient.Builder()
        .sslSocketFactory(sslFactory, trustManager)
        .hostnameVerifier((h, session) -> "localhost".equalsIgnoreCase(h))
        .build();
      Request http = new Request.Builder()
        .url(url)
        .header("User-Agent", userAgent)
        .header("Cookie", cookie)
        .build();
      try (Response resp = client.newCall(http).execute()) {
        if (!resp.isSuccessful()) throw new IOException("HTTP " + resp);
        try (OutputStream out = context.getContentResolver().openOutputStream(destUri)) {
          assert out != null;
          assert resp.body() != null;
          try (BufferedSource src = resp.body().source()) {
            try (okio.BufferedSink sink = Okio.buffer(Okio.sink(out))) {
              sink.writeAll(src);
            }
          }
        }
        context.runOnUiThread(() ->
          Toast.makeText(context,
            "Downloaded to “" + destUri.getPath() + "”",
            Toast.LENGTH_LONG).show()
        );
      } catch (Exception e) {
        context.runOnUiThread(() ->
          Toast.makeText(context, "Download failed: " + e.getMessage(),
            Toast.LENGTH_LONG).show()
        );
      }
    }).start();
  }
}
