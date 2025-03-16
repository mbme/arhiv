package me.mbsoftware.arhiv;

import android.annotation.SuppressLint;
import android.content.Intent;
import android.content.pm.ApplicationInfo;
import android.net.Uri;
import android.net.http.SslCertificate;
import android.net.http.SslError;
import android.os.Bundle;
import android.os.Environment;
import android.util.Log;
import android.webkit.CookieManager;
import android.webkit.SslErrorHandler;
import android.webkit.WebResourceRequest;
import android.webkit.WebView;
import android.webkit.WebViewClient;

import androidx.annotation.NonNull;
import androidx.appcompat.app.AppCompatActivity;
import androidx.core.app.ActivityCompat;
import androidx.swiperefreshlayout.widget.SwipeRefreshLayout;

import java.security.cert.X509Certificate;
import java.util.Arrays;

class ServerInfo {
  public String uiUrl;
  public String authToken;
  public byte[] certificate;
}

class ArhivServer {
  public static native ServerInfo startServer(String appFilesDir, String externalStorageDir);

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

    ServerInfo serverInfo = ArhivServer.startServer(this.getFilesDir().getAbsolutePath(), Environment.getExternalStorageDirectory().getAbsolutePath());

    WebView webView = findViewById(R.id.web);
    webView.getSettings().setJavaScriptEnabled(true);
    webView.getSettings().setDomStorageEnabled(true);

    SwipeRefreshLayout swipeRefreshLayout = findViewById(R.id.swipeRefreshLayout);
    swipeRefreshLayout.setOnRefreshListener(webView::reload);

    CookieManager cookieManager = CookieManager.getInstance();
    cookieManager.setAcceptCookie(true);
    cookieManager.setCookie(serverInfo.uiUrl, "AuthToken=" + serverInfo.authToken + "; Secure; HttpOnly");

    webView.setWebViewClient(new WebViewClient() {
      @SuppressLint("WebViewClientOnReceivedSslError")
      @Override
      public void onReceivedSslError(WebView view, SslErrorHandler handler, SslError error) {
        if (isTrustedCertificate(error.getCertificate())) {
          Log.i(TAG, "Got valid server SSL certificate");
          handler.proceed(); // Proceed only if it matches self-signed certificate
        } else {
          Log.e(TAG, "Got invalid server SSL certificate");
          handler.cancel(); // Reject all other SSL errors
        }
      }

      private boolean isTrustedCertificate(SslCertificate certificate) {
        X509Certificate cert = certificate.getX509Certificate();
        if (cert == null) return false;

        try {
          return Arrays.equals(cert.getEncoded(), serverInfo.certificate);
        } catch (Exception e) {
          Log.e(TAG, "Failed to validate SSL certificate:", e);
          return false;
        }
      }

      @Override
      public void onPageFinished(WebView view, String url) {
        super.onPageFinished(view, url);
        swipeRefreshLayout.setRefreshing(false);
      }

      @Override
      public boolean shouldOverrideUrlLoading(WebView view, WebResourceRequest request) {
        Uri requestUrl = request.getUrl();

        if (requestUrl.toString().startsWith(serverInfo.uiUrl)) {
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
    webView.loadUrl(serverInfo.uiUrl);

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
