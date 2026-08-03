package me.mbsoftware.arhiv;

public class ServerInfo {
  public final String uiUrl;
  public final String authToken;
  public final byte[] certificate;

  public ServerInfo(String uiUrl, String authToken, byte[] certificate) {
    this.uiUrl = uiUrl;
    this.authToken = authToken;
    this.certificate = certificate;
  }
}
