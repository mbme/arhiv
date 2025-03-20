package me.mbsoftware.arhiv;

import android.annotation.SuppressLint;
import android.content.Intent;
import android.content.res.Configuration;
import android.net.Uri;
import android.net.http.SslCertificate;
import android.net.http.SslError;
import android.os.Bundle;
import android.os.Environment;
import android.provider.Settings;
import android.util.Log;
import android.view.WindowManager;
import android.webkit.CookieManager;
import android.webkit.SslErrorHandler;
import android.webkit.ValueCallback;
import android.webkit.WebChromeClient;
import android.webkit.WebResourceRequest;
import android.webkit.WebSettings;
import android.webkit.WebView;
import android.webkit.WebViewClient;

import androidx.activity.result.ActivityResultLauncher;
import androidx.activity.result.contract.ActivityResultContracts;
import androidx.annotation.NonNull;
import androidx.appcompat.app.AppCompatActivity;
import androidx.swiperefreshlayout.widget.SwipeRefreshLayout;

import java.security.cert.X509Certificate;
import java.util.Arrays;

public class MainActivity extends AppCompatActivity {
  private static final String TAG = "MainActivity";

  private WebView webView;

  private ValueCallback<Uri[]> filePathCallback;
  private ActivityResultLauncher<Intent> filePickerLauncher;

  @Override
  protected void onCreate(Bundle savedInstanceState) {
    super.onCreate(savedInstanceState);

    // Mark window as secure to avoid screenshots & avoid previews when pressing Overview button
    getWindow().setFlags(WindowManager.LayoutParams.FLAG_SECURE,
      WindowManager.LayoutParams.FLAG_SECURE);

    setContentView(R.layout.activity_main);

    // Set up ActivityResultLauncher to handle file picker results
    filePickerLauncher = registerForActivityResult(
      new ActivityResultContracts.StartActivityForResult(),
      result -> {
        Log.d(TAG, "Got result from File Picker activity: " + result.getResultCode());

        if (filePathCallback == null) {
          Log.w(TAG, "filePathCallback is null, ignoring File Picker results!");
          return;
        }

        if (result.getResultCode() == RESULT_OK && result.getData() != null) {
          Uri[] results = null;
          if (result.getData().getDataString() != null) {
            results = new Uri[]{Uri.parse(result.getData().getDataString())};
          }
          filePathCallback.onReceiveValue(results);
        } else {
          filePathCallback.onReceiveValue(null);
        }

        filePathCallback = null;
      });

    ensureIsExternalStorageManager();
  }

  private void ensureIsExternalStorageManager() {
    Log.i(TAG, "Checking external storage manager permission");

    if (Environment.isExternalStorageManager()) {
      Log.d(TAG, "Is external storage manager");
      authApp();
    } else {
      Log.d(TAG, "Requesting external storage manager permissions");

      Intent intent = new Intent(Settings.ACTION_MANAGE_APP_ALL_FILES_ACCESS_PERMISSION);
      intent.setData(Uri.parse("package:" + getPackageName()));

      ActivityResultLauncher<Intent> storagePermissionLauncher = registerForActivityResult(
        new ActivityResultContracts.StartActivityForResult(), result -> {
          Log.i(TAG, "Got permission activity result: " + result);
          ensureIsExternalStorageManager();
        });
      storagePermissionLauncher.launch(intent);
    }
  }

  private void authApp() {
    Log.i(TAG, "Authenticating");

    if (!Keyring.isDeviceSecure(this)) {
      Log.w(TAG, "Device is not secure, skipping auth");

      initApp(null);

      return;
    }

    if (!Keyring.isBiometricAvailable(this)) {
      Log.w(TAG, "Biometric auth not available");
    }

    try {
      Keyring.generateKey();
    } catch (Exception e) {
      Log.e(TAG, "Failed to generate KeyStore key:", e);

      initApp(null);

      return;
    }

    Keyring.loadPassword(this, new LoadPasswordCallback() {
      @Override
      public void onSuccess(String password) {
        if (password == null) {
          Log.i(TAG, "Authentication: no password");
        } else {
          Log.i(TAG, "Authentication: decrypted password");
        }

        initApp(password);
      }

      @Override
      public void onError(String msg) {
        Log.e(TAG, "Authentication failed: " + msg);
        initApp(null);
      }
    });

  }

  private void initApp(String password) {
    Log.i(TAG, "Starting Arhiv server");

    ServerInfo serverInfo = ArhivServer.startServer(
      this.getFilesDir().getAbsolutePath(),
      Environment.getExternalStorageDirectory().getAbsolutePath(),
      password,
      new AndroidController(this)
    );

    if (webView == null) {
      Log.i(TAG, "Initializing WebView");

      initWebView(serverInfo);
      updateWebViewAuthToken(serverInfo);
      updateWebViewDarkMode(getResources().getConfiguration());

      // loading url in the WebView.
      webView.loadUrl(serverInfo.uiUrl);
    } else {
      Log.i(TAG, "Reusing existing WebView");

      updateWebViewAuthToken(serverInfo);
      updateWebViewDarkMode(getResources().getConfiguration());
    }
  }

  @SuppressLint("SetJavaScriptEnabled")
  private void initWebView(ServerInfo serverInfo) {
    webView = findViewById(R.id.web);
    webView.getSettings().setJavaScriptEnabled(true);
    webView.getSettings().setDomStorageEnabled(true);
    webView.getSettings().setAllowFileAccess(false);

    SwipeRefreshLayout swipeRefreshLayout = findViewById(R.id.swipeRefreshLayout);
    swipeRefreshLayout.setOnRefreshListener(webView::reload);

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

    webView.setWebChromeClient(new WebChromeClient() {
      @Override
      public boolean onShowFileChooser(WebView webView, ValueCallback<Uri[]> filePathCallback,
                                       FileChooserParams fileChooserParams) {
        Log.d(TAG, "Starting File Picker activity");

        MainActivity.this.filePathCallback = filePathCallback;

        Intent intent = fileChooserParams.createIntent();

        try {
          filePickerLauncher.launch(intent);
        } catch (Exception e) {
          Log.e(TAG, "Failed to launch file picker:", e);
          MainActivity.this.filePathCallback = null;
          return false;
        }
        return true;
      }
    });
  }

  private void updateWebViewAuthToken(ServerInfo serverInfo) {
    Log.i(TAG, "Updating WebView AuthToken cookie");

    CookieManager cookieManager = CookieManager.getInstance();
    cookieManager.setAcceptCookie(true);
    cookieManager.setCookie(serverInfo.uiUrl, "AuthToken=" + serverInfo.authToken + "; Secure; HttpOnly");
  }

  private void updateWebViewDarkMode(@NonNull Configuration config) {
    boolean isNight = (config.uiMode & Configuration.UI_MODE_NIGHT_MASK)
      == Configuration.UI_MODE_NIGHT_YES;
    Log.i(TAG, "Android is in " + (isNight ? "dark mode" : "light mode"));

    webView.getSettings().setForceDark(
      isNight ? WebSettings.FORCE_DARK_ON : WebSettings.FORCE_DARK_OFF
    );
  }

  @Override
  public void onConfigurationChanged(@NonNull Configuration newConfig) {
    super.onConfigurationChanged(newConfig);

    Log.i(TAG, "Android configuration have changed");

    updateWebViewDarkMode(newConfig);
  }


  @Override
  protected void onDestroy() {
    Log.i(TAG, "Stopping Arhiv server");

    ArhivServer.stopServer();

    Log.i(TAG, "Stopped Arhiv server");

    super.onDestroy();
  }
}
