# Arhiv Crypto and Key Lifecycle

## Scope

This document defines key hierarchy, lifecycle, recovery boundaries, and secret
handling. The [encrypted file format](arhiv-encrypted-file-format.md) owns byte
formats; [authentication and sessions](auth-session-trust-chain-spec.md) owns
TLS and UI session secrets.

## Key hierarchy

1. A password-derived AGE scrypt key encrypts and decrypts only
   `storage/key.age`.
2. The x25519 storage master key is stored inside `key.age` and encrypts the
   database, state, search index, and document locks.
3. Each asset has an x25519 blob key stored in its document metadata and used
   only for that asset's blob.

## Lifecycle

### Create

Creation enforces the 8-byte minimum password length, derives its wrapping key,
generates a storage master key, writes the password-encrypted `key.age`, and
creates storage with the current `BazaInfo`.

### Unlock and lock

Unlock decrypts and parses the storage master key, validates it by opening
storage, and retains it in memory. When available, Arhiv also stores its
serialized form in platform-protected credential storage.

Lock durably deletes that platform cache before dropping in-memory access. If
cache deletion fails, lock fails and retains the in-memory key.

### Change password

Password change decrypts `key.age`, re-encrypts the same storage master key with
the new password, and replaces the key file transactionally. Storage payloads
are not re-encrypted.

### Export, import, and verification

Export decrypts the local key file and returns the same storage master key in an
ASCII-armored AGE payload protected by the export password.

Import decrypts and parses the candidate key, validates it by reading storage,
and transactionally replaces local `key.age`. Verification performs the same
validation without replacement.

## Recovery contract

Recoverable cases:

- a missing platform cache, when the owner still has the Arhiv password or an
  exported key and its password;
- a lost local `key.age`, when a usable export and password remain; and
- password rotation, because it re-wraps the same storage master key.

Storage is unrecoverable when no available credential can reconstruct its
master key, including loss or corruption of all usable key files and exports.
Arhiv has no server-side escrow or recovery service.

## Security contract

Key handling aims to:

- keep storage confidential at rest without decryption credentials;
- separate the password wrapper from the storage master key;
- allow password rotation without rewriting storage; and
- fail closed on wrong credentials, malformed keys, or incompatible storage.

Sensitive buffers and transported key material should use secret-aware memory
handling where feasible. Logs must not contain plaintext secrets, and decrypted
secret lifetimes should be minimized. Zeroization is best-effort: libraries,
allocators, and the OS may retain copies, and a compromised host remains outside
this guarantee.

## Platform credential caches

Desktop and Android may cache the serialized storage master key in their system
credential stores. A valid cache can unlock storage without the Arhiv password;
a missing, malformed, unavailable, or non-matching cache falls back to password
or imported-key recovery.

Caches are convenience mechanisms, not recovery copies or independent trust
anchors. See [Platform security boundaries](platform-security-boundaries-spec.md)
for platform authentication and failure handling.

## Operational guidance

- Keep an offline exported key protected by a strong password.
- Test export and import in a controlled environment.
- Treat password change as key-file rewrapping, not data re-encryption.
- Include both key recovery and storage backup in recovery drills.
