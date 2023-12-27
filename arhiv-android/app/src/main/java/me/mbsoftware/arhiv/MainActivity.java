package me.mbsoftware.arhiv;

import android.annotation.SuppressLint;
import android.os.Bundle;
import android.webkit.WebView;
import android.webkit.WebViewClient;
import androidx.appcompat.app.AppCompatActivity;
import android.content.pm.ApplicationInfo;

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

    // Find the WebView by its unique ID
    WebView webView = findViewById(R.id.web);

    // loading url in the WebView.
    webView.loadUrl(url);

    // this will enable the javascript.
    webView.getSettings().setJavaScriptEnabled(true);

    // enable WebView debugging if app is debuggable
   if ((getApplicationInfo().flags & ApplicationInfo.FLAG_DEBUGGABLE) != 0)
    { WebView.setWebContentsDebuggingEnabled(true); }

    // WebViewClient allows you to handle
    // onPageFinished and override Url loading.
    webView.setWebViewClient(new WebViewClient());
  }

  @Override
  protected void onDestroy() {
    ArhivServer.stopServer();

    super.onDestroy();
  }
}
