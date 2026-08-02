package me.mbsoftware.arhiv;

import androidx.annotation.NonNull;

public interface LoadStorageKeyCallback {
  void onSuccess(String storageKey);

  void onAuthenticationError(int errorCode, @NonNull CharSequence errorMessage);

  void onRecoveryRequired();
}