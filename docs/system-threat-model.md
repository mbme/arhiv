# Arhiv System Threat Model

## Scope

Arhiv is a single-user, local-first application. A local Rust server exposes the
UI over HTTPS on IPv4 loopback to browser, Electron, and Android WebView clients.
This document owns system-wide assets, attackers, goals, and residual risks.
Detailed controls are defined in:

- [Crypto and key lifecycle](crypto-key-lifecycle-threat-model.md)
- [Authentication and sessions](auth-session-trust-chain-spec.md)
- [Platform security boundaries](platform-security-boundaries-spec.md)
- [Backup and restore](backup-restore-durability-spec.md)
- [Merge conflicts](merge-conflicts-spec.md)

Recognized release builds may check GitHub's latest stable release at most once
per hour. The request sends no Arhiv data, credentials, or current-version value,
but exposes ordinary network metadata such as IP address, user agent, and timing.

## Protected assets

Arhiv protects the confidentiality and integrity of:

1. Documents, metadata, and asset plaintext.
2. Passwords, storage and blob keys, and exported keys.
3. Authentication and browser-bootstrap tokens.
4. The local TLS private key and certificate.
5. Backup artifacts required for committed-state recovery.

The storage format does not conceal filenames, file sizes, counts, timestamps,
directory layout, or the presence and cadence of backups and conflict files.

## Attacker model

Arhiv is designed to mitigate:

- theft of encrypted storage, exports, or backups without their credentials;
- offline password guessing against password-protected keys;
- direct LAN or Internet access to the loopback-only server;
- cross-site attempts to drive authenticated browser requests;
- ciphertext corruption that fails authentication, parsing, or compatibility
  checks; and
- reuse of the one-time browser bootstrap token after successful consumption.

Arhiv does not protect against:

- compromise of the local OS account, root/admin, kernel, process memory, or JNI
  boundary;
- compromise of the system keyring, Android Keystore, browser, Electron, or
  WebView;
- intentional plaintext disclosure by the owner; or
- availability attacks such as deletion, disk exhaustion, or process
  termination.

## Trust boundaries

| Boundary | Primary control | Limitation |
| --- | --- | --- |
| Encrypted files or backups | AGE encryption and password-wrapped storage key | Password and export handling determine offline-guessing resistance. |
| Stored bytes to parser | Authenticated decryption and strict format/version parsing | Older valid ciphertext can be replayed. |
| Launcher to server | Local process/JNI boundary and startup-delivered certificate | A compromised local account can interfere. |
| Client to server | Loopback HTTPS, authenticated cookie, and platform certificate pinning where available | Loopback excludes network peers, not same-user local processes. |
| Browser bootstrap | Independent 256-bit one-time token and redirect to a clean URL | The initial URL may appear in local history before use. |
| Platform key cache | System keyring or Android Keystore | It is a convenience mechanism, not recovery material. |
| Live storage to backup | Encrypted artifacts and authenticated manifest | Capture is not transactional and has no freshness protection. |
| UI to GitHub Releases API | CSP allowlist, numeric tag validation, and hourly attempt limit | External request metadata and availability remain observable. |

## Security goals

1. Keep encrypted storage and backups confidential without the required key.
2. Separate password-derived wrapping keys from the storage master key so
   password changes do not rewrite storage.
3. Restrict UI/API access to local clients holding the current session cookie.
4. Prevent direct network access to the server.
5. Authenticate the local server to Desktop and Android without public CAs.
6. Fail closed on decryption, parsing, and compatibility errors.
7. Preserve recoverable committed state when matching key, database, and blob
   artifacts are backed up together.

## Residual risks

- Valid older ciphertext may be replayed; authenticated encryption does not
  provide rollback protection.
- Password changes do not revoke existing storage keys or decryptable exports.
- A compromised local account can observe bootstrap material and local runtime
  boundaries.
- Backups can represent mixed moments when live files change during capture;
  staged state is excluded.
- Arhiv provides no remote multi-user isolation, key escrow, or recovery service.
- Filesystem metadata and platform/runtime security remain outside Arhiv's
  cryptographic guarantees.
