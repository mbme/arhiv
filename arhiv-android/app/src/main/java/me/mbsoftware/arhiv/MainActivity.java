package me.mbsoftware.arhiv;

import android.annotation.SuppressLint;
import android.content.ClipData;
import android.content.Intent;
import android.content.pm.PackageInfo;
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
import android.webkit.URLUtil;
import android.webkit.ValueCallback;
import android.webkit.WebChromeClient;
import android.webkit.WebResourceRequest;
import android.webkit.WebSettings;
import android.webkit.WebView;
import android.webkit.WebViewClient;

import androidx.activity.result.ActivityResultLauncher;
import androidx.activity.result.contract.ActivityResultContracts;
import androidx.annotation.NonNull;
import androidx.appcompat.app.AlertDialog;
import androidx.appcompat.app.AppCompatActivity;
import androidx.biometric.BiometricPrompt;
import androidx.webkit.WebSettingsCompat;
import androidx.webkit.WebViewFeature;

import java.io.ByteArrayInputStream;
import java.io.IOException;
import java.io.InputStream;
import java.security.GeneralSecurityException;
import java.security.KeyStore;
import java.security.cert.Certificate;
import java.security.cert.CertificateFactory;
import java.security.cert.X509Certificate;
import java.util.Arrays;
import java.util.Objects;

import javax.net.ssl.SSLContext;
import javax.net.ssl.SSLSocketFactory;
import javax.net.ssl.TrustManager;
import javax.net.ssl.TrustManagerFactory;
import javax.net.ssl.X509TrustManager;

public class MainActivity extends AppCompatActivity {
  private static final String TAG = "MainActivity";

  private WebView webView;

  private final FileChooserRequest fileChooserRequest = new FileChooserRequest();
  private ActivityResultLauncher<Intent> filePickerLauncher;
  private ActivityResultLauncher<Intent> downloadFileLocationPicker;
  private DownloadRequest pendingDownload;
  private boolean appStarted;

  @Override
  protected void onCreate(Bundle savedInstanceState) {
    super.onCreate(savedInstanceState);

    boolean hasAcceptableWebView = this.ensureMinWebViewVersion();
    if (!hasAcceptableWebView) {
      return;
    }

    // Mark window as secure to avoid screenshots & avoid previews when pressing Overview button
    getWindow().setFlags(WindowManager.LayoutParams.FLAG_SECURE,
      WindowManager.LayoutParams.FLAG_SECURE);

    setContentView(R.layout.activity_main);

    // Set up ActivityResultLauncher to handle file picker results
    filePickerLauncher = registerForActivityResult(
      new ActivityResultContracts.StartActivityForResult(),
      result -> {
        Log.d(TAG, "Got result from File Picker activity: " + result.getResultCode());

        Uri[] results = null;
        if (result.getResultCode() == RESULT_OK) {
          Intent data = result.getData();
          if (data != null) {
            if (data.getClipData() != null) {
              ClipData clip = data.getClipData();
              int count = clip.getItemCount();
              results = new Uri[count];
              for (int i = 0; i < count; i++) {
                results[i] = clip.getItemAt(i).getUri();
              }
            } else if (data.getData() != null) {
              results = new Uri[]{data.getData()};
            }
          }
        }
        fileChooserRequest.complete(results);
      });

    downloadFileLocationPicker = registerForActivityResult(
      new ActivityResultContracts.StartActivityForResult(),
      result -> {
        DownloadRequest download = pendingDownload;
        pendingDownload = null;
        if (result.getResultCode() == RESULT_OK && download != null && result.getData() != null) {
          Uri dest = result.getData().getData();
          if (dest != null) {
            download.performDownloadToUri(this, dest);
          }
        }
      });
    ensureIsExternalStorageManager();
  }

  private boolean ensureMinWebViewVersion() {
    PackageInfo info = WebView.getCurrentWebViewPackage();
    String version = info == null ? "0" : info.versionName;
    int major = 0;
    try {
      assert version != null;
      major = Integer.parseInt(version.split("\\.")[0]);
    } catch (Exception ignored) {
    }

    if (major < 111) {
      Log.e(TAG, "WebView version is too old: " + version);

      AlertDialog.Builder builder = new AlertDialog.Builder(this);
      builder
        .setTitle("Update WebView")
        .setMessage("Please install Android System WebView v111+")
        .setPositiveButton("Update", (d, w) -> startActivity(new Intent(
          Intent.ACTION_VIEW,
          Uri.parse("market://details?id=com.google.android.webview")
        )))
        .setCancelable(false);

      // Create and attach the OnDismissListener to exit the app immediately when the dialog is dismissed
      AlertDialog dialog = builder.create();
      dialog.setOnDismissListener(d -> {
        // Closes all activities in this task and exits
        finishAffinity();
      });

      dialog.show();

      return false;
    }

    return true;
  }

  private void ensureIsExternalStorageManager() {
    Log.i(TAG, "Checking external storage manager permission");

    if (Environment.isExternalStorageManager()) {
      Log.d(TAG, "Is external storage manager");
      ensureIsSecureDevice();
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

  private void ensureIsSecureDevice() {
    Log.i(TAG, "Checking if device is secure");

    if (Keyring.isDeviceSecure(this)) {
      Log.i(TAG, "Device is secure");
      authApp();

    } else {
      Log.w(TAG, "Device is not secure");

      Intent intent = new Intent(Settings.ACTION_SECURITY_SETTINGS);

      ActivityResultLauncher<Intent> deviceSecuritySettingsLauncher = registerForActivityResult(
        new ActivityResultContracts.StartActivityForResult(), result -> {
          Log.i(TAG, "Got device security settings activity result: " + result);
          ensureIsSecureDevice();
        });
      deviceSecuritySettingsLauncher.launch(intent);
    }
  }

  private void authApp() {
    Log.i(TAG, "Authenticating");

    if (appStarted) {
      Log.w(TAG, "Ignoring authentication request after app startup");
      return;
    }

    try {
      Keyring.generateKey();
    } catch (Exception e) {
      Log.e(TAG, "Failed to generate KeyStore key:", e);
      initApp(null);
      return;
    }

    Keyring.loadStorageKey(this, new LoadStorageKeyCallback() {
      @Override
      public void onSuccess(String storageKey) {
        if (storageKey == null) {
          Log.i(TAG, "Authentication: no cached storage key");
        } else {
          Log.i(TAG, "Authentication: decrypted cached storage key");
        }

        initApp(storageKey);
      }

      @Override
      public void onAuthenticationError(int errorCode, @NonNull CharSequence errorMessage) {
        Log.w(TAG, "Authentication prompt ended: " + errorCode + " " + errorMessage);
        if (canRetryDeviceAuthentication(errorCode)) {
          showAuthenticationFallback(true);
        } else {
          showAuthenticationFallback(false);
        }
      }

      @Override
      public void onRecoveryRequired() {
        Log.w(TAG, "Cached storage key requires password or import recovery");
        initApp(null);
      }
    });
  }

  private boolean canRetryDeviceAuthentication(int errorCode) {
    switch (errorCode) {
      case BiometricPrompt.ERROR_HW_UNAVAILABLE:
      case BiometricPrompt.ERROR_HW_NOT_PRESENT:
      case BiometricPrompt.ERROR_NO_SPACE:
      case BiometricPrompt.ERROR_NO_BIOMETRICS:
      case BiometricPrompt.ERROR_NO_DEVICE_CREDENTIAL:
      case BiometricPrompt.ERROR_LOCKOUT:
      case BiometricPrompt.ERROR_LOCKOUT_PERMANENT:
        return false;
      default:
        return true;
    }
  }

  private void showAuthenticationFallback(boolean canRetry) {
    AlertDialog.Builder builder = new AlertDialog.Builder(this)
      .setMessage(canRetry
        ? "Device authentication was not completed."
        : "Device authentication is unavailable. Use your Arhiv password or import a key.")
      .setOnCancelListener(dialog -> initApp(null));

    if (canRetry) {
      builder.setPositiveButton("Try biometrics again", (dialog, which) -> authApp());
      builder.setNegativeButton("Use Arhiv password", (dialog, which) -> initApp(null));
    } else {
      builder.setPositiveButton("Use Arhiv password", (dialog, which) -> initApp(null));
    }

    builder.show();
  }

  private void initApp(String storageKey) {
    if (appStarted) {
      Log.w(TAG, "Ignoring duplicate app startup request");
      return;
    }

    Log.i(TAG, "Starting Arhiv server");

    String downloadsPath = Objects.requireNonNull(this.getExternalFilesDir(Environment.DIRECTORY_DOWNLOADS)).getAbsolutePath();

    ServerStartResult startResult = ArhivServer.startServer(
      this.getFilesDir().getAbsolutePath(),
      Environment.getExternalStorageDirectory().getAbsolutePath(),
      downloadsPath,
      storageKey,
      new AndroidController(this)
    );
    if (startResult.serverInfo == null) {
      showServerStartFailure(startResult.error);
      return;
    }
    ServerInfo serverInfo = startResult.serverInfo;
    appStarted = true;

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

    X509TrustManager trustManager;
    SSLSocketFactory sslSocketFactory;
    try {
      trustManager = trustManagerForCertificates(serverInfo.certificate);
      SSLContext sslContext = SSLContext.getInstance("TLS");
      sslContext.init(null, new TrustManager[]{trustManager}, null);
      sslSocketFactory = sslContext.getSocketFactory();
    } catch (GeneralSecurityException e) {
      throw new RuntimeException(e);
    }
    webView.setDownloadListener((url, userAgent, contentDisposition, mimeType, contentLength) -> {
      String filename = URLUtil.guessFileName(url, contentDisposition, mimeType);
      // ask the user where to save
      pendingDownload = new DownloadRequest(
        url, userAgent, CookieManager.getInstance().getCookie(url),
        filename, mimeType, sslSocketFactory, trustManager
      );
      Intent chooser = new Intent(Intent.ACTION_CREATE_DOCUMENT);
      chooser.addCategory(Intent.CATEGORY_OPENABLE);
      chooser.setType(mimeType);
      chooser.putExtra(Intent.EXTRA_TITLE, filename);
      downloadFileLocationPicker.launch(chooser);
    });


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

        fileChooserRequest.replace(filePathCallback);

        Intent intent = fileChooserParams.createIntent();
        intent.putExtra(Intent.EXTRA_ALLOW_MULTIPLE, true);

        try {
          filePickerLauncher.launch(intent);
        } catch (Exception e) {
          Log.e(TAG, "Failed to launch file picker:", e);
          fileChooserRequest.complete(null);
          return false;
        }
        return true;
      }
    });
  }

  private static X509TrustManager trustManagerForCertificates(byte[] certificate) throws GeneralSecurityException {
    // Build a KeyStore containing certificate
    CertificateFactory cf = CertificateFactory.getInstance("X.509");
    Certificate ca;
    try (InputStream is = new ByteArrayInputStream(certificate)) {
      ca = cf.generateCertificate(is);
    } catch (IOException e) {
      throw new RuntimeException(e);
    }
    KeyStore ks = KeyStore.getInstance(KeyStore.getDefaultType());
    try {
      ks.load(null, null);
    } catch (IOException e) {
      throw new RuntimeException(e);
    }
    ks.setCertificateEntry("ca", ca);

    // Create a TrustManager that trusts just that CA
    TrustManagerFactory tmf = TrustManagerFactory.getInstance(
      TrustManagerFactory.getDefaultAlgorithm());
    tmf.init(ks);

    return (X509TrustManager) tmf.getTrustManagers()[0];
  }

  private void updateWebViewAuthToken(ServerInfo serverInfo) {
    Log.i(TAG, "Updating WebView AuthToken cookie");

    CookieManager cookieManager = CookieManager.getInstance();
    cookieManager.setAcceptCookie(true);
    cookieManager.setCookie(serverInfo.uiUrl, "AuthToken=" + serverInfo.authToken + "; Secure; HttpOnly");
  }

  /** @noinspection deprecation*/
  private void updateWebViewDarkMode(@NonNull Configuration config) {
    boolean isNight = (config.uiMode & Configuration.UI_MODE_NIGHT_MASK)
      == Configuration.UI_MODE_NIGHT_YES;
    Log.i(TAG, "Android is in " + (isNight ? "dark mode" : "light mode"));


    WebSettings settings = webView.getSettings();

    // turn force-dark on (so CSS media‐queries get flipped)
    if (WebViewFeature.isFeatureSupported(WebViewFeature.FORCE_DARK)) {
      WebSettingsCompat.setForceDark(settings,  isNight ? WebSettingsCompat.FORCE_DARK_ON : WebSettingsCompat.FORCE_DARK_OFF);
    }

    // only apply “web theme” dark styles—not UA auto-darkening
    if (WebViewFeature.isFeatureSupported(WebViewFeature.FORCE_DARK_STRATEGY)) {
      WebSettingsCompat.setForceDarkStrategy(
        settings,
        WebSettingsCompat.DARK_STRATEGY_WEB_THEME_DARKENING_ONLY
      );
    }

    // disable any fallback algorithmic darkening
    if (WebViewFeature.isFeatureSupported(WebViewFeature.ALGORITHMIC_DARKENING)) {
      WebSettingsCompat.setAlgorithmicDarkeningAllowed(
        settings,
        false
      );
    }
  }

  @Override
  public void onConfigurationChanged(@NonNull Configuration newConfig) {
    super.onConfigurationChanged(newConfig);

    Log.i(TAG, "Android configuration have changed");

    updateWebViewDarkMode(newConfig);
  }

  @Override
  public void onBackPressed() {
    if (webView != null && webView.canGoBack()) {
      webView.goBack();
    } else {
      new AlertDialog.Builder(this)
        .setMessage("Are you sure you want to exit?")
        .setPositiveButton("Yes", (d, w) -> super.onBackPressed())
        .setNegativeButton("No", null)
        .show();
    }
  }

  @Override
  protected void onDestroy() {
    fileChooserRequest.complete(null);

    if (appStarted) {
      Log.i(TAG, "Stopping Arhiv server");
      String error = ArhivServer.stopServer();
      if (error == null) {
        Log.i(TAG, "Stopped Arhiv server");
      } else {
        Log.e(TAG, "Failed to stop Arhiv server: " + error);
      }
    }

    super.onDestroy();
  }

  private void showServerStartFailure(String error) {
    String message = error == null
      ? "Arhiv could not start."
      : "Arhiv could not start: " + error;
    new AlertDialog.Builder(this)
      .setTitle("Unable to start Arhiv")
      .setMessage(message)
      .setPositiveButton("Close", (dialog, which) -> finishAffinity())
      .setCancelable(false)
      .show();
  }
}
