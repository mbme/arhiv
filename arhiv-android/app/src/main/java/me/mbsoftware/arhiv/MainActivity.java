package me.mbsoftware.arhiv;

import android.annotation.SuppressLint;
import android.content.Intent;
import android.content.pm.ApplicationInfo;
import android.net.Uri;
import android.net.http.SslError;
import android.os.Bundle;
import android.os.Environment;
import android.util.Log;
import android.webkit.SslErrorHandler;
import android.webkit.WebResourceRequest;
import android.webkit.WebView;
import android.webkit.WebViewClient;

import androidx.annotation.NonNull;
import androidx.appcompat.app.AppCompatActivity;
import androidx.core.app.ActivityCompat;
import androidx.swiperefreshlayout.widget.SwipeRefreshLayout;

class ArhivServer {
  public static native String startServer(String appFilesDir, String externalStorageDir);

  public static native void stopServer();

  static {
    System.loadLibrary("arhiv_android");
  }
}

public class MainActivity extends AppCompatActivity {
  private static final String TAG = "MainActivity";
  private static final int REQUEST_STORAGE_PERMISSION = 99;

  @Override
  protected void onCreate(Bundle savedInstanceState) {
    super.onCreate(savedInstanceState);
    setContentView(R.layout.activity_main);

//    try {
//      Keyring.generateKey();
//    } catch (Exception e) {
//      Log.e(TAG, "Failed to generate KeyStore key:", e);
//    }

    if (Environment.isExternalStorageManager()) {
      initApp();
    } else {
      ActivityCompat.requestPermissions(
        this,
        new String[]{
          android.Manifest.permission.READ_EXTERNAL_STORAGE,
          android.Manifest.permission.MANAGE_EXTERNAL_STORAGE
        },
        REQUEST_STORAGE_PERMISSION
      );
    }
  }

  @SuppressLint("MissingSuperCall")
  @Override
  public void onRequestPermissionsResult(int requestCode, @NonNull String[] permissions,
                                         @NonNull int[] grantResults) {
    if (requestCode == REQUEST_STORAGE_PERMISSION) {
        initApp();
    }
  }

  @SuppressLint("SetJavaScriptEnabled")
  private void initApp() {
    Log.i(TAG, "Starting Arhiv server");
    String arhivUrl = ArhivServer.startServer(this.getFilesDir().getAbsolutePath(), Environment.getExternalStorageDirectory().getAbsolutePath());

    WebView webView = findViewById(R.id.web);
    webView.getSettings().setJavaScriptEnabled(true);
    webView.getSettings().setDomStorageEnabled(true);

    SwipeRefreshLayout swipeRefreshLayout = findViewById(R.id.swipeRefreshLayout);
    swipeRefreshLayout.setOnRefreshListener(webView::reload);

    webView.setWebViewClient(new WebViewClient() {
      @Override
      public void onReceivedSslError(WebView view, SslErrorHandler handler, SslError error) {
        handler.proceed();
      }

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

        Log.d(TAG, "Open external URL: " + requestUrl);

        // Open the URL in an external browser
        Intent intent = new Intent(Intent.ACTION_VIEW, requestUrl);
        view.getContext().startActivity(intent);

        return true;
      }
    });

    // loading url in the WebView.
    webView.loadUrl(arhivUrl);

    // enable WebView debugging if app is debuggable
    if ((getApplicationInfo().flags & ApplicationInfo.FLAG_DEBUGGABLE) != 0) {
      WebView.setWebContentsDebuggingEnabled(true);
    }
  }

  @Override
  protected void onDestroy() {
    Log.i(TAG, "Stopping Arhiv server");

    ArhivServer.stopServer();

    Log.i(TAG, "Stopped Arhiv server");

    super.onDestroy();
  }
}
