# Arhiv System Threat Model

Status: policy + implementation-aligned (current behavior)

## 1. Purpose and Scope

This document defines the system-level security model for Arhiv: protected assets, attacker capabilities, trust boundaries, security goals, and known limitations.

It is a cross-cutting policy document. Detailed implementation contracts remain authoritative in:

- `docs/crypto-key-lifecycle-threat-model.md` for key hierarchy, recovery, and secret handling
- `docs/auth-session-trust-chain-spec.md` for local HTTPS, cookies, tokens, and browser bootstrap
- `docs/platform-security-boundaries-spec.md` for Desktop and Android boundaries
- `docs/backup-restore-durability-spec.md` for backup, restore, and durability guarantees
- `docs/merge-conflicts-spec.md` for synchronization and conflict behavior

## 2. Deployment Model

Arhiv is a single-user, local-first application. Its encrypted storage is accessed by a local Rust server and UI clients:

- CLI browser mode starts the server and opens a browser bootstrap URL.
- Desktop starts the server as a child process and loads the UI in Electron.
- Android starts the server through JNI and loads the UI in a WebView.

The HTTPS server binds only to IPv4 loopback (`127.0.0.1`). It is not a LAN or Internet service.

Desktop and Android receive server startup information through their process/JNI boundary, pin the delivered self-signed certificate, and set the primary authentication cookie directly. Generic browser launch uses a separate, one-time bootstrap token to establish that cookie, then redirects to the clean UI URL.

Recognized release builds check GitHub's latest stable release once per hour while the workspace UI is open. The browser-side request sends no Arhiv records, credentials, or current-version value, but GitHub and the platform network stack can observe normal request metadata such as IP address, user agent, and request timing.

## 3. Protected Assets

Arhiv protects the confidentiality and integrity of:

1. User records, metadata, and asset/blob plaintext.
2. The storage master key, per-asset blob keys, user passwords, and exported keys.
3. Authentication and browser-bootstrap tokens.
4. The local server TLS private key and certificate.
5. Backup artifacts needed to recover committed storage.

The following are sensitive metadata but are not encrypted by the storage format:

- storage, asset, and backup filenames
- file sizes, counts, timestamps, and directory layout
- presence and cadence of backups or synchronized conflict files

## 4. Attacker Model

### 4.1 In Scope

The system is designed to mitigate:

1. An attacker who obtains encrypted storage directories, exported keys, or backups but not the needed decryption credentials.
2. Offline password-guessing attempts against password-protected key files or exports.
3. Network peers attempting to reach the UI/API service. Loopback-only binding prevents direct LAN and Internet connections.
4. A malicious external website attempting to drive authenticated browser requests. The UI uses `Secure`, `HttpOnly`, `SameSite=Strict` cookies and does not accept reusable authentication tokens in protected-route query parameters.
5. Accidental or malicious ciphertext corruption that causes decryption, parsing, or compatibility validation to fail.
6. One-time browser bootstrap URL reuse. The bootstrap token is independently generated and consumed on first successful use.

### 4.2 Out of Scope

Arhiv does not protect against:

1. A compromised local OS account, including malware or another process running with the user's authority.
2. Root/admin compromise, kernel compromise, live memory inspection, or interception of local process/JNI boundaries.
3. Compromise of the system keyring, Android Keystore, browser, Electron runtime, or Android WebView.
4. Intentional plaintext disclosure by the user, including through screenshots, clipboard, exports, or manual sharing.
5. Availability attacks such as storage deletion, disk exhaustion, process termination, or indefinite denial of local service.

## 5. Trust Boundaries and Controls

| Boundary | Primary control | Important limitation |
| --- | --- | --- |
| Encrypted storage or backup -> attacker | AGE encryption; password-protected key wrapper | Password strength and key/export handling determine resistance to offline guessing. |
| Storage files -> Arhiv parser | Authenticated decryption and strict format/version parsing | This detects altered or malformed bytes, but not replay of an older valid encrypted file. |
| Launcher -> local server | Startup information, local process/JNI boundary, certificate pinning in Desktop/Android | A compromised local user account can interfere with this boundary. |
| Browser/WebView -> local server | IPv4 loopback binding, HTTPS, certificate pinning where available, authenticated cookie | Loopback restricts network peers, not local processes owned by the same user. |
| Generic browser launch -> authenticated UI | Separate 256-bit one-time bootstrap token; cookie exchange and redirect to clean UI URL | The initial bootstrap URL can still appear in browser or process history before it is consumed. |
| Workspace UI -> GitHub Releases API | Validated numeric `tag_name`, hourly attempt limit, and CSP `connect-src` allowlist | GitHub and the platform network stack can observe normal request metadata and availability depends on external network access. |
| Desktop/Android secret persistence -> platform secret store | System keyring or Android Keystore with platform authentication | These stores are convenience and local UX mechanisms, not a replacement for backup or encryption-key recovery. |
| Live storage -> backup | Encrypted file copies of key, committed DB, and blobs | Backup is not a transactional snapshot and has no authenticated generation manifest. |

## 6. Security Goals

1. Preserve confidentiality of encrypted storage and backup contents without the required password or storage key.
2. Keep password-derived keys separate from the storage master key so password changes re-wrap the key file without rewriting all encrypted data.
3. Restrict UI/API access to local clients holding the current server authentication cookie.
4. Prevent direct network access to the local UI/API server.
5. Authenticate the local server identity for Desktop and Android without relying on public certificate authorities.
6. Fail closed when decryption, storage parsing, or version compatibility checks fail.
7. Preserve enough encrypted artifacts for committed-state recovery when a compatible key file, database, and blobs are backed up together.

## 7. Explicit Non-Guarantees and Residual Risks

1. Authenticated encryption does not provide rollback protection: a valid earlier encrypted storage file may still decrypt successfully.
2. Password changes do not revoke access for anyone who already possesses the storage master key or an independently decryptable export.
3. The browser bootstrap token is one-time, not secret from a compromised local account or from local browser/process history before use.
4. The system does not conceal filesystem metadata listed in Section 3.
5. Backups are recoverable copies, not atomic point-in-time snapshots; staged changes are excluded and a concurrent writer can produce a mixed-time artifact set.
6. No remote multi-user isolation, server-side escrow, or recovery service is provided.
7. Platform permissions and browser/runtime security properties are delegated to the operating system and platform runtimes.

## 8. Security-Sensitive Change Rules

Changes to the following require review of the linked detailed specification and targeted security validation:

1. Key hierarchy, password handling, exports, or storage encryption -> crypto/key lifecycle threat model.
2. Server binding, certificates, tokens, cookies, browser bootstrap, or UI routes -> auth/session trust-chain spec.
3. Desktop or Android startup, certificate trust, keyring/keystore, permissions, or WebView behavior -> platform security boundaries spec.
4. Backup, restore, storage replacement, migration, or sync merge behavior -> backup/restore, migration, and merge-conflict specs.

Security claims must distinguish confidentiality, integrity, freshness/rollback resistance, availability, and metadata protection. Do not describe a control as providing a guarantee outside its stated boundary.

## 9. Validation Expectations

For security-sensitive changes, validate the affected boundary directly. Examples include:

- verify the server is reachable on `127.0.0.1` and not on a non-loopback interface
- verify protected routes reject missing, malformed, reused, or incorrect credentials as applicable
- verify the browser bootstrap token succeeds once and cannot be reused
- verify Desktop and Android reject a certificate other than the startup-delivered certificate
- verify wrong keys, corrupted ciphertext, and incompatible storage versions fail closed
- verify backup recovery using a disposable copy of a known committed storage state

## 10. Source of Truth (Code References)

- `arhiv/src/server/mod.rs`
- `arhiv/src/server/auth_token.rs`
- `arhiv/src/server/http_server.rs`
- `arhiv/src/server/ui_server/mod.rs`
- `arhiv/src/server/certificate.rs`
- `arhiv/src/server/server_info.rs`
- `baza-storage/src/crypto/age.rs`
- `baza/src/baza_manager/keys.rs`
- `baza/src/backup.rs`
- `arhiv-desktop/src/arhiv.ts`
- `arhiv-desktop/src/index.ts`
- `arhiv-android/src/lib.rs`
- `arhiv-android/app/src/main/java/me/mbsoftware/arhiv/MainActivity.java`
