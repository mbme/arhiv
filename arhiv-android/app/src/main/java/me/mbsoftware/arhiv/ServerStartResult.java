package me.mbsoftware.arhiv;

import androidx.annotation.Nullable;

public class ServerStartResult {
  @Nullable
  public final ServerInfo serverInfo;

  @Nullable
  public final String error;

  public ServerStartResult(@Nullable ServerInfo serverInfo, @Nullable String error) {
    this.serverInfo = serverInfo;
    this.error = error;
  }
}