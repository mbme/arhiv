# Arhiv Crypto and Key Lifecycle Threat Model

Status: implementation-derived (current behavior) + explicit threat model policy

## 1. Scope

This document specifies:
- Key hierarchy and lifecycle operations
- Recoverability and non-recoverability boundaries
- Threat model assumptions and security guarantees
- Zeroization expectations and limits

It does not specify:
- TLS/UI trust chain details (covered in auth/session spec)
- Storage container byte format (covered in encrypted file format spec)

## 2. Key Hierarchy and Roles

Arhiv uses AGE-based encryption with three practical key layers:

1. Password-derived key (AGE scrypt recipient/identity)
- Derived from user password
- Used only to encrypt/decrypt `storage/key.age` (ASCII-armored)

2. Storage master key (x25519 AGE identity)
- Stored encrypted inside `key.age`
- Encrypts/decrypts main storage + state/search/locks files

3. Per-asset blob keys (x25519 AGE identity per asset)
- Stored in asset document metadata
- Encrypt/decrypt asset blob payload files

## 3. Lifecycle Operations

### 3.1 Create

`create(password)`:
1. Derives password key from password (min length enforced).
2. Generates new x25519 storage master key.
3. Writes master key into `key.age` encrypted with password key.
4. Creates initial storage with current `BazaInfo`.

### 3.2 Unlock / Lock

`unlock(password)`:
1. Decrypts `key.age` with password key.
2. Parses decrypted x25519 storage master key.
3. Stores key in process memory and allows storage/state access.

`lock()`:
- Drops in-memory key/cache and clears cached opened state.
- In desktop path, password is removed from system keyring.

### 3.3 Change Password

`change_key_file_password(old_password, new_password)`:
1. Decrypts current `key.age` with old password.
2. Re-encrypts same storage master key with new password.
3. Replaces key file transactionally.

Important: storage payload files are NOT re-encrypted during password change.

### 3.4 Export Key

`export_key(password, export_password)`:
1. Decrypts local `key.age` with current password.
2. Re-encrypts same storage master key with `export_password`.
3. Returns ASCII-armored AGE payload string.

### 3.5 Import/Verify Key

`import_key(encrypted_key_data, password)`:
1. Decrypts imported key payload.
2. Parses candidate storage master key.
3. Validates candidate key by attempting to read storage.
4. Replaces local `key.age` transactionally.

`verify_key(...)` performs validation without replacing key file.

## 4. Recoverability Contract

### 4.1 Recoverable

1. Forgotten local app state/keyring password cache:
- recoverable if user still knows Arhiv password or has exported key + its password.

2. Lost local `key.age`:
- recoverable only if user has exported key payload + export password.

3. Password rotation:
- recoverable by design (same storage master key; key file re-wrapped).

### 4.2 Not Recoverable

1. Lost both:
- usable `key.age` (or export) AND password material needed to decrypt it.

2. Corrupted encrypted key payload with no valid backup/export.

3. Storage encrypted with master key that no available credential can reconstruct.

No server-side escrow or recovery service exists in current architecture.

## 5. Threat Model

### 5.1 In Scope

1. At-rest compromise of storage directories (stolen disk/backup).
2. Offline brute-force attempts against password-protected key exports.
3. File-level tampering/corruption attempts that should fail decrypt/parse/open.

### 5.2 Out of Scope

1. Fully compromised runtime host (root/admin malware, live memory scraping).
2. Compromised OS keyring/Android keystore implementation.
3. User exfiltration of plaintext via screenshots, clipboard, manual sharing.

### 5.3 Security Goals

1. Confidentiality of storage contents at rest without decryption credentials.
2. Separation of password wrapper key from storage master key.
3. Operational ability to rotate password without rewriting whole storage.
4. Deterministic fail-fast behavior on wrong key/password/version.

## 6. Zeroization and Secret Handling Expectations

Current implementation uses secrecy wrappers (`SecretString`, `SecretBytes`, `SecretBox`/`SecretSlice`) for sensitive buffers and key material transport.

Contract:
1. Secrets should be kept in secret-typed wrappers where feasible.
2. Logs must never include plaintext secret values.
3. Decrypted secret material lifetime in memory should be minimized.

Limitations:
1. Zeroization is best-effort at wrapper boundaries; full-process zeroization cannot be guaranteed.
2. External libraries/allocators/OS may retain copies outside Arhiv control.
3. Host compromise remains out of scope.

## 7. Keyring / Platform Integration Boundaries

Desktop/server:
- Optional system keyring stores the Arhiv password for convenience.
- `lock()` erases stored password entry; `unlock()` writes it.

Android:
- Password storage uses Android keystore + biometric-gated encrypt/decrypt flow.
- Rust side keeps an in-memory password copy after init in current integration design.

Security boundary statement:
- Keyring/keystore are convenience and local UX mechanisms, not trust anchors that replace encryption keys or backups.

## 8. Operational Guidance

1. Keep at least one offline exported key copy protected by a strong password.
2. Validate export/import flows periodically in a controlled test environment.
3. Treat password change as key-file rewrap only; it is not data re-encryption.
4. Pair key export with storage backup in recovery drills.

## 9. Source of Truth (Code References)

- `rs-utils/src/crypto/age.rs`
- `rs-utils/src/crypto/secret.rs`
- `baza/src/baza_manager/keys.rs`
- `baza/src/baza_manager/manager_state.rs`
- `baza/src/baza_manager/mod.rs`
- `baza/src/baza/mod.rs`
- `arhiv/src/arhiv/mod.rs`
- `arhiv/src/arhiv/keyring.rs`
- `arhiv-android/src/keyring.rs`
- `arhiv-android/app/src/main/java/me/mbsoftware/arhiv/Keyring.java`
