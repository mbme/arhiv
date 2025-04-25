package me.mbsoftware.arhiv;

public interface LoadPasswordCallback {
  void onSuccess(String password);

  void onError(String msg);
}
