package me.mbsoftware.arhiv;

import android.annotation.SuppressLint;
import android.content.pm.ApplicationInfo;
import android.os.Bundle;
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

  @SuppressLint("SetJavaScriptEnabled")
  @Override
  protected void onCreate(Bundle savedInstanceState) {
    super.onCreate(savedInstanceState);
    setContentView(R.layout.activity_main);

    String url = ArhivServer.startServer(this.getFilesDir().getAbsolutePath());

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
    });

    // loading url in the WebView.
    webView.loadUrl(url);

    // enable WebView debugging if app is debuggable
    if ((getApplicationInfo().flags & ApplicationInfo.FLAG_DEBUGGABLE) != 0)
      { WebView.setWebContentsDebuggingEnabled(true); }
  }

  @Override
  protected void onDestroy() {
    ArhivServer.stopServer();

    super.onDestroy();
  }
}
