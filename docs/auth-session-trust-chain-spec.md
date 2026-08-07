# Arhiv Auth, Session, and Trust-Chain Spec

Status: implementation-aligned (current behavior)

Scope: local HTTPS UI server authentication, session propagation, and certificate trust behavior across CLI/Desktop/Android launchers.

## 1. Components and Secrets

- Server auth token: 256-bit opaque bearer token generated at server startup.
- Browser bootstrap token: one-time 256-bit opaque token used only to establish a browser cookie session.
- Server certificate: self-signed X.509 certificate read from or written to state dir (`arhiv-server.pem`).
- Transport: HTTPS only for UI/API endpoints.

## 2. Server-Side Auth Contract

UI auth middleware (`client_authenticator`) enforces the following:

1. Client token source:
- Cookie `AuthToken` only.

2. Parsing and validation:
- Token must parse as `AuthToken` serialized format.
- Missing token => `401 Unauthorized` (`AuthToken is missing`).
- Parse failure => `400 Bad Request`.
- Token mismatch against server startup token => `401 Unauthorized` (`Invalid AuthToken`).

3. Session propagation:
- On successful auth, server sets cookie `AuthToken=<token>` with attributes:
  - `Path=/ui`
  - `HttpOnly`
  - `Secure`
  - `SameSite=Strict`

Auth token internals:
- token payload is 32 random bytes.
- serialized as URL-safe base64.

## 3. Token Issuance and Lifetime

- Token is generated once per server start (`AuthToken::generate`).
- No TTL/expiration mechanism is enforced by middleware.
- New server start implies new token.

Browser bootstrap:
- `browserUrl` contains a separate one-time token at `/auth`.
- The endpoint accepts that token once, sets the authenticated cookie, and redirects to the clean `/ui` URL.
- Query tokens are not accepted by protected UI/API routes.

## 4. Certificate Lifecycle

- Certificate file path: `<state_dir>/arhiv-server.pem`.
- If file exists: server loads existing certificate.
- If file is missing: server generates new self-signed certificate and writes PEM file.
- On unix, generated certificate file permissions are set to `0600`.

## 5. Desktop Trust Chain

Desktop launcher flow:

1. Starts CLI server with `arhiv server --json`.
2. Reads `@@SERVER_INFO:` JSON from server stderr.
3. Computes SHA-256 base64 fingerprint from `serverInfo.certificate` DER bytes.
4. Handles Electron `certificate-error` by accepting cert only when fingerprint matches expected fingerprint.

Desktop cookie setup:
- Launcher writes `AuthToken` cookie before opening UI URL.
- Cookie attributes set by launcher:
  - `secure: true`
  - `sameSite: strict`

## 6. Android Trust Chain

Android launcher flow:

1. Starts server through JNI and receives `ServerInfo { uiUrl, authToken, certificate }`.
2. Builds trust manager from provided DER certificate bytes.
3. WebView TLS handler (`onReceivedSslError`) accepts certificate only if presented cert DER bytes exactly match `serverInfo.certificate`.
4. Non-matching certificates are rejected (`handler.cancel()`).

Android cookie setup:
- WebView cookie set for `serverInfo.uiUrl`:
  - `AuthToken=<token>; Secure; HttpOnly`

## 7. Boundary and Assumptions

- UI URLs are emitted as `https://localhost:<port>/ui`.
- Server socket binds IPv4 loopback only (`127.0.0.1`).
- This model is local-process trust, not PKI CA trust.
- The UI Content Security Policy permits scripts from the local server and connections only to the local server or GitHub's API for stable-release update checks.

## 8. Known Limitations

- No token expiry/rotation within a running server process.
- No documented multi-client session separation; all clients use same server token for that server instance.
- No CSRF-specific mechanism beyond `SameSite=Strict` cookie handling and localhost deployment assumptions.

## 9. Source of Truth (Code References)

- `arhiv/src/server/ui_server/mod.rs`
- `arhiv/src/server/mod.rs`
- `arhiv/src/server/certificate.rs`
- `arhiv/src/server/server_info.rs`
- `arhiv/src/support/auth_token.rs`
- `arhiv-desktop/src/arhiv.ts`
- `arhiv-desktop/src/index.ts`
- `arhiv-android/app/src/main/java/me/mbsoftware/arhiv/MainActivity.java`
- `arhiv-android/src/lib.rs`
- `arhiv-android/app/src/main/java/me/mbsoftware/arhiv/ServerInfo.java`
