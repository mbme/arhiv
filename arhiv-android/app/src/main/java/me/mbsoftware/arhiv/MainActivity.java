package me.mbsoftware.arhiv;

import android.annotation.SuppressLint;
import android.content.Intent;
import android.content.pm.ApplicationInfo;
import android.net.Uri;
import android.os.Bundle;
import android.util.Log;
import android.webkit.WebResourceRequest;
import android.webkit.WebView;
import android.webkit.WebViewClient;
import androidx.appcompat.app.AppCompatActivity;
import androidx.swiperefreshlayout.widget.SwipeRefreshLayout;

class ArhivServer {
  public static native String startServer(String filesDir);

  public static native void stopServer();

  static {
    System.loadLibrary("arhiv_android");
  }
}

public class MainActivity extends AppCompatActivity {
  private static final String TAG = "MainActivity";

  @SuppressLint("SetJavaScriptEnabled")
  @Override
  protected void onCreate(Bundle savedInstanceState) {
    super.onCreate(savedInstanceState);
    setContentView(R.layout.activity_main);

    String arhivUrl = ArhivServer.startServer(this.getFilesDir().getAbsolutePath());
    Log.i(TAG, "Started Arhiv server");

    WebView webView = findViewById(R.id.web);
    webView.getSettings().setJavaScriptEnabled(true);
    webView.getSettings().setDomStorageEnabled(true);

    SwipeRefreshLayout swipeRefreshLayout = findViewById(R.id.swipeRefreshLayout);
    swipeRefreshLayout.setOnRefreshListener(webView::reload);

    webView.setWebViewClient(new WebViewClient() {
      @Override
      public void onPageFinished(WebView view, String url) {
        super.onPageFinished(view, url);
        swipeRefreshLayout.setRefreshing(false);
      }

      @Override
      public boolean shouldOverrideUrlLoading(WebView view, WebResourceRequest request) {
        Uri requestUrl = request.getUrl();

        if (requestUrl.toString().startsWith(arhivUrl)) {
          return false;
        }

        Log.d(TAG, "Open external URL: " +  requestUrl);

        // Open the URL in an external browser
        Intent intent = new Intent(Intent.ACTION_VIEW, requestUrl);
        view.getContext().startActivity(intent);

        return true;
      }
    });

    // loading url in the WebView.
    webView.loadUrl(arhivUrl);

    // enable WebView debugging if app is debuggable
    if ((getApplicationInfo().flags & ApplicationInfo.FLAG_DEBUGGABLE) != 0)
      { WebView.setWebContentsDebuggingEnabled(true); }
  }

  @Override
  protected void onDestroy() {
    Log.i(TAG, "Stopping Arhiv server");

    ArhivServer.stopServer();

    Log.i(TAG, "Stopped Arhiv server");

    super.onDestroy();
  }
}
