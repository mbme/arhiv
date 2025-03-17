package me.mbsoftware.arhiv;

import android.app.KeyguardManager;
import android.content.Context;
import android.content.SharedPreferences;
import android.security.keystore.KeyGenParameterSpec;
import android.security.keystore.KeyProperties;
import android.util.Base64;
import android.util.Log;

import androidx.annotation.NonNull;
import androidx.biometric.BiometricManager;
import androidx.biometric.BiometricPrompt;
import androidx.core.content.ContextCompat;

import java.nio.charset.StandardCharsets;
import java.security.KeyStore;
import java.util.Arrays;
import java.util.Objects;

import javax.crypto.Cipher;
import javax.crypto.KeyGenerator;
import javax.crypto.SecretKey;
import javax.crypto.spec.GCMParameterSpec;


public class Keyring {
  private static final String TAG = "Keyring";

  private static final String KEYSTORE_ALIAS = "ArhivKeystore";
  private static final String SHARED_PREFS_NAME = "ArhivPrefs";
  private static final String PASSWORD_KEY = "encrypted_password";
  private static final int GCM_IV_LENGTH = 12;
  private static final int GCM_TAG_LENGTH = 128;

  public static boolean isDeviceSecure(Context context) {
    KeyguardManager keyguardManager = (KeyguardManager) context.getSystemService(Context.KEYGUARD_SERVICE);
    return keyguardManager != null && keyguardManager.isDeviceSecure();
  }

  public static boolean isBiometricAvailable(Context context) {
    BiometricManager biometricManager = BiometricManager.from(context);
    return biometricManager.canAuthenticate(BiometricManager.Authenticators.BIOMETRIC_STRONG)
      == BiometricManager.BIOMETRIC_SUCCESS;
  }

  public static void generateKey() throws Exception {
    if (keyExists()) return; // Prevent key regeneration

    KeyGenerator keyGenerator = KeyGenerator.getInstance(KeyProperties.KEY_ALGORITHM_AES, "AndroidKeyStore");
    keyGenerator.init(new KeyGenParameterSpec.Builder(
      KEYSTORE_ALIAS,
      KeyProperties.PURPOSE_ENCRYPT | KeyProperties.PURPOSE_DECRYPT)
      .setBlockModes(KeyProperties.BLOCK_MODE_GCM)
      .setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE)
      .setUserAuthenticationRequired(true) // Biometric protection
      .setInvalidatedByBiometricEnrollment(true)
      .setUserAuthenticationParameters(0,
        KeyProperties.AUTH_BIOMETRIC_STRONG |
          KeyProperties.AUTH_DEVICE_CREDENTIAL)
      .setKeySize(256)
      .build());
    keyGenerator.generateKey();
  }

  public static void savePassword(Context context, String password) throws Exception {
    Cipher cipher = getCipher();
    cipher.init(Cipher.ENCRYPT_MODE, getSecretKey());

    BiometricPrompt biometricPrompt = new BiometricPrompt(
      (androidx.fragment.app.FragmentActivity) context,
      ContextCompat.getMainExecutor(context),
      new BiometricPrompt.AuthenticationCallback() {
        @Override
        public void onAuthenticationError(int errorCode, @NonNull CharSequence errString) {
          Log.e(TAG, "Save password: authentication error: " + errorCode + " " + errString);
        }

        @Override
        public void onAuthenticationSucceeded(@NonNull BiometricPrompt.AuthenticationResult result) {
          try {
            Cipher cipher = Objects.requireNonNull(result.getCryptoObject()).getCipher();
            assert cipher != null;

            byte[] iv = cipher.getIV();
            byte[] encrypted = cipher.doFinal(password.getBytes(StandardCharsets.UTF_8));

            byte[] combined = new byte[iv.length + encrypted.length];
            System.arraycopy(iv, 0, combined, 0, iv.length);
            System.arraycopy(encrypted, 0, combined, iv.length, encrypted.length);

            SharedPreferences prefs = context.getSharedPreferences(SHARED_PREFS_NAME, Context.MODE_PRIVATE);
            prefs.edit().putString(PASSWORD_KEY, Base64.encodeToString(combined, Base64.DEFAULT)).apply();
          } catch (Exception e) {
            Log.e(TAG, "Save password: failed to encrypt", e);
          }
        }

        @Override
        public void onAuthenticationFailed() {
          Log.e(TAG, "Save password: authentication failed");
        }
      }
    );

    BiometricPrompt.PromptInfo promptInfo = new BiometricPrompt.PromptInfo.Builder()
      .setTitle("Authentication required")
      .setSubtitle("Unlock to save Arhiv password")
      .setConfirmationRequired(false)
      .setAllowedAuthenticators(
        BiometricManager.Authenticators.BIOMETRIC_STRONG |
          BiometricManager.Authenticators.DEVICE_CREDENTIAL)
      .build();

    biometricPrompt.authenticate(promptInfo, new BiometricPrompt.CryptoObject(cipher));
  }

  public static void loadPassword(Context context, LoadPasswordCallback callback) {
    SharedPreferences prefs = context.getSharedPreferences(SHARED_PREFS_NAME, Context.MODE_PRIVATE);
    String encryptedPassword = prefs.getString(PASSWORD_KEY, null);
    if (encryptedPassword == null) {
      callback.onSuccess(null);
      return;
    }

    Cipher cipher;
    try {
      byte[] combined = Base64.decode(encryptedPassword, Base64.DEFAULT);
      byte[] iv = Arrays.copyOfRange(combined, 0, GCM_IV_LENGTH);

      cipher = getCipher();
      cipher.init(Cipher.DECRYPT_MODE, getSecretKey(), new GCMParameterSpec(GCM_TAG_LENGTH, iv));
    } catch (Exception e) {
      Log.e(TAG, "Load password: failed to build cipher for decryption", e);
      callback.onError("Failed to build cipher for decryption: " + e);
      return;
    }

    BiometricPrompt biometricPrompt = new BiometricPrompt(
      (androidx.fragment.app.FragmentActivity) context,
      ContextCompat.getMainExecutor(context),
      new BiometricPrompt.AuthenticationCallback() {
        @Override
        public void onAuthenticationError(int errorCode, @NonNull CharSequence errString) {
          callback.onError("Authentication error: " + errorCode + " " + errString);
        }

        @Override
        public void onAuthenticationSucceeded(@NonNull BiometricPrompt.AuthenticationResult result) {
          try {
            Cipher cipher = Objects.requireNonNull(result.getCryptoObject()).getCipher();
            assert cipher != null;

            byte[] combined = Base64.decode(encryptedPassword, Base64.DEFAULT);
            byte[] encrypted = Arrays.copyOfRange(combined, GCM_IV_LENGTH, combined.length);

            byte[] decryptedData = cipher.doFinal(encrypted);

            callback.onSuccess(new String(decryptedData, StandardCharsets.UTF_8));
          } catch (Exception e) {
            Log.e(TAG, "Load password: failed to decrypt", e);
            callback.onError("Failed to decrypt: " + e);
          }
        }

        @Override
        public void onAuthenticationFailed() {
          Log.e(TAG, "Load password: authentication failed");
          callback.onError("Authentication failed");
        }
      }
    );

    BiometricPrompt.PromptInfo promptInfo = new BiometricPrompt.PromptInfo.Builder()
      .setTitle("Authentication required")
      .setSubtitle("Authenticate to unlock Arhiv")
      .setConfirmationRequired(false)
      .setAllowedAuthenticators(
        BiometricManager.Authenticators.BIOMETRIC_STRONG |
          BiometricManager.Authenticators.DEVICE_CREDENTIAL)
      .build();

    biometricPrompt.authenticate(promptInfo, new BiometricPrompt.CryptoObject(cipher));
  }

  public static void erasePassword(Context context) {
    SharedPreferences prefs = context.getSharedPreferences(SHARED_PREFS_NAME, Context.MODE_PRIVATE);
    prefs.edit().remove(PASSWORD_KEY).apply();
  }

  private static SecretKey getSecretKey() throws Exception {
    KeyStore keyStore = KeyStore.getInstance("AndroidKeyStore");
    keyStore.load(null);
    return ((KeyStore.SecretKeyEntry) keyStore.getEntry(KEYSTORE_ALIAS, null)).getSecretKey();
  }

  private static boolean keyExists() throws Exception {
    KeyStore keyStore = KeyStore.getInstance("AndroidKeyStore");
    keyStore.load(null);
    return keyStore.containsAlias(KEYSTORE_ALIAS);
  }

  private static Cipher getCipher() throws Exception {
    return Cipher.getInstance(KeyProperties.KEY_ALGORITHM_AES + "/"
      + KeyProperties.BLOCK_MODE_GCM + "/"
      + KeyProperties.ENCRYPTION_PADDING_NONE);
  }
}
