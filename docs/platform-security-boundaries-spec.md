# Arhiv Platform Security Boundaries Spec

Status: implementation-aligned (current behavior)

Scope: platform-specific trust boundaries, process boundaries, permissions, and sensitive data flow for Desktop (Electron) and Android wrappers around the local Arhiv server.

## 1. Shared Security Model

Common model across desktop and android:
- Arhiv server is local process, started by wrapper.
- UI is served over local HTTPS with a self-signed certificate.
- Client wrappers trust only the specific server certificate delivered at startup.
- API access is gated by `AuthToken` cookie/query token.

Critical trust assumptions:
- local host process boundary is trusted more than network perimeter.
- compromise of local user account compromises local Arhiv runtime.

## 2. Desktop Boundary (Electron)

## 2.1 Process and trust boundary

- Electron main process launches `arhiv server --json` subprocess.
- Server startup info is read from stderr marker `@@SERVER_INFO:`.
- Main process computes SHA-256 certificate fingerprint from provided DER bytes.
- On TLS certificate error, Electron accepts cert only when fingerprint matches expected value.

Boundary implication:
- desktop app does not trust system CA for local server identity; it uses certificate pinning-like fingerprint check against startup payload.

## 2.2 Session/token handling

- Desktop main process sets `AuthToken` cookie for server UI URL before loading WebView content.
- Cookie is set with `secure: true` and `sameSite: 'strict'`.
- Server also sets `HttpOnly; Secure` cookie once request is authenticated.

## 2.3 UI navigation boundary

- New window requests from web content are denied in Electron window and opened externally via system browser.
- This reduces in-app surface for untrusted external pages.

## 2.4 Desktop secrets at rest

- Password persistence uses system keyring through Rust `keyring` crate (`ArhivKeyring::new_system_keyring`).
- DEV and PROD keyring service names differ (`Arhiv-dev` vs `Arhiv`).

## 3. Android Boundary

## 3.1 Process and JNI boundary

- Android app starts server through JNI (`ArhivServer.startServer`).
- Rust JNI layer stores singletons for runtime and server behind process-global mutexes.
- Start is single-instance per app process (`Runtime already started` / `Server already started`).

Boundary implication:
- Java/Kotlin UI and Rust server runtime share one app sandbox/process context (with JNI boundary).

## 3.2 Device security gate

- App requires secure device lock screen before continuing (`Keyring.isDeviceSecure`).
- App enforces minimum WebView major version 111.

## 3.3 Android credential storage

- Password is encrypted with AES-GCM key from AndroidKeyStore.
- Keystore key requires biometric strong or device credential authentication.
- Encrypted payload is stored in app SharedPreferences.
- Password retrieval requires biometric/device authentication via `BiometricPrompt`.

## 3.4 TLS trust and WebView boundary

- WebView TLS errors are handled explicitly.
- Certificate is accepted only if presented certificate DER bytes exactly match startup `serverInfo.certificate` bytes.
- Non-matching certs are rejected.

Additional download path:
- download helper builds OkHttp client with the same trust manager and a `localhost` hostname verifier.

## 3.5 Permissions and storage boundary

Manifest-declared permissions:
- `INTERNET`
- `MANAGE_EXTERNAL_STORAGE`
- `READ_EXTERNAL_STORAGE`

Runtime behavior:
- App requests all-files access (`ACTION_MANAGE_APP_ALL_FILES_ACCESS_PERMISSION`).
- Arhiv storage path is under external storage (`<externalStorage>/Arhiv`).
- App files/state are under app private files dir.

UI capture boundary:
- `FLAG_SECURE` is set to block screenshots/overview previews.

## 4. Local Server Boundary and Certificate Material

- Server TLS private key and certificate are persisted in `<state_dir>/arhiv-server.pem`.
- On unix, file permissions are set to `0600` when generated.
- Server binds HTTPS socket to `0.0.0.0`, but launchers consume localhost URLs from `ServerInfo`.

## 5. Non-goals / Not guaranteed

- This model does not protect against fully compromised local OS/user account.
- No remote multi-tenant isolation model is provided.
- No formal sandbox between UI and server beyond platform process/runtime primitives.

## 6. Known Risks

- Android `MANAGE_EXTERNAL_STORAGE` is broad and raises data exposure impact if app sandbox is compromised.
- Desktop trust chain depends on integrity of startup `@@SERVER_INFO` stream from local child process.
- Local certificate file is intentionally persistent; compromise of state dir can affect local trust material.

## 7. Source of Truth (Code References)

- `arhiv-desktop/src/index.ts`
- `arhiv-desktop/src/arhiv.ts`
- `arhiv/src/arhiv/keyring.rs`
- `arhiv/src/arhiv/mod.rs`
- `arhiv/src/server/certificate.rs`
- `rs-utils/src/http_server.rs`
- `arhiv-android/app/src/main/AndroidManifest.xml`
- `arhiv-android/app/src/main/java/me/mbsoftware/arhiv/MainActivity.java`
- `arhiv-android/app/src/main/java/me/mbsoftware/arhiv/Keyring.java`
- `arhiv-android/app/src/main/java/me/mbsoftware/arhiv/DownloadRequest.java`
- `arhiv-android/app/src/main/res/xml/network_security_config.xml`
- `arhiv-android/src/lib.rs`
