# Arhiv Encrypted File Format Specification

Status: implementation-derived (current code)
Version: 1 (storage `storage_version = 1`)

## 1. Scope

This document specifies the on-disk encrypted file formats used by Arhiv/Baza in the current repository state.

It covers:
- Main storage database files (`*.gz.age`)
- State files (`state.gz.age`)
- Search index files (`search_index.gz.age`)
- Document locks files (`document_locks.age`)
- Encrypted asset/blob files (`*.age`)
- Key file (`key.age`)

It does not define higher-level product behavior (sync policy, UI/API, merge conflict UX).

## 2. Terminology

- AGE: The [age] encryption format as used by the Rust `age` crate stream APIs.
- ASCII-armored AGE: text form with `-----BEGIN AGE ENCRYPTED FILE-----` envelope.
- Binary AGE: non-armored binary stream envelope.
- GZIP: DEFLATE stream wrapped in `.gz` framing.
- Container: newline-delimited text structure with JSON index line + value lines.
- Document key: serialized as `<id><space><revision-safe-string>`.

## 3. File Inventory and Layering

### 3.1 Storage tree (default names)

- `storage/key.age`
- `storage/baza.gz.age` (main DB)
- `storage/data/<asset_id>.age` (committed asset blobs)
- `state/state.gz.age`
- `state/search_index.gz.age`
- `state/document_locks.age`
- `state/data/<asset_id>.age` (staged/local blobs)

### 3.2 Envelope layering by file type

1. `*.gz.age` files:
- Outer: AGE binary stream encryption
- Inner: GZIP-compressed payload (`flate2` `Compression::fast()`)
- Plaintext payload: type-specific bytes (JSON/container/postcard)

2. `*.age` files (non-key):
- Outer: AGE binary stream encryption
- Plaintext payload: raw bytes or postcard bytes (no gzip)

3. `key.age`:
- Outer: AGE ASCII-armored encryption
- Plaintext payload: UTF-8 x25519 secret key string (the storage master key)

## 4. Cryptographic Key Model

### 4.1 AGE key variants used

Arhiv uses one of:
- Password-based AGE (scrypt recipient/identity)
- x25519 AGE identity keypair (recipient = public key)

### 4.2 Key roles

- Key file password-derived key: decrypts/encrypts `key.age`.
- Storage master key (x25519 private key stored inside `key.age`): encrypts/decrypts storage DB and state/search/locks files.
- Blob key (per asset, x25519 private key serialized in asset metadata): encrypts/decrypts blob files.

### 4.3 Serialization details

- x25519 private keys are serialized by `age` identity `to_string()` format.
- Password keys require minimum password length 8 bytes.
- `key.age` plaintext is the serialized x25519 secret key bytes.

## 5. Main Storage Database Format (`baza.gz.age`)

## 5.1 High-level

After AGE decrypt + GZIP decompress, plaintext is a text container:
- Line 1: JSON array index (`LinesIndex`)
- Remaining lines: one UTF-8 JSON value per index key

Container invariants:
- Number of value lines must be exactly index length.
- No extra lines are allowed.
- Missing lines are invalid.

## 5.2 Container plaintext grammar

```
container := index_line "\n" value_line*(exactly N lines)
index_line := JSON array of unique strings, length N
value_line := UTF-8 text without embedded newline
```

Important:
- Embedded newlines in values are not supported because parsing is line-based (`BufRead::lines`).

## 5.3 Storage index semantics

Index keys are ordered and unique.

For storage DB specifically:
- Index entry 0 MUST be `"info"`.
- Entries 1..N are serialized `DocumentKey` values.

`DocumentKey` serialization:
- `"<id> <revision-safe-string>"`
- `id`: document id string (current generator uses 14-char random id, but parser accepts any string without additional validation here)
- `revision-safe-string`: `instance:version` segments joined by `-`, sorted by instance id; empty string denotes initial revision

## 5.4 Storage line payload semantics

- Line for key `info`: JSON object `BazaInfo`
  - Fields:
    - `storage_version: u8`
    - `data_version: u8`

- Each document-key line: JSON object `Document` (strict unknown-field rejection)
  - `id`
  - `rev`
  - `document_type`
  - `updated_at`
  - `data`

## 5.5 Write ordering rules

When creating storage:
- Document keys are sorted by `(id ASC, rev ASC)` before writing.
- This is done to improve compression locality.
- First written value line is always `info`.
- Document lines follow sorted index order.

## 5.6 Patch semantics

Container patch is ordered map: `key -> Option<line>`.

Rules:
- Existing key + `Some(value)`: replace value, keep position.
- Existing key + `None`: delete key and its value.
- New key + `Some(value)`: append key to end of index; append value at end.
- New key + `None`: ignored for index construction but invalid if left for new-value emission.

The patched output is fully rewritten as a new container and then encrypted/compressed.

## 6. State File Format (`state.gz.age`)

After AGE decrypt + GZIP decompress: UTF-8 JSON object `BazaStateFile`.

Top-level fields:
- `instance_id`
- `info` (`BazaInfo`)
- `documents` (`HashMap<Id, DocumentHead>`)
- `refs` (`HashMap<DocumentKey, Refs>`)

`modified` is runtime-only (`#[serde(skip)]`) and not serialized.

## 7. Search Index Format (`search_index.gz.age`)

After AGE decrypt + GZIP decompress: postcard binary payload of `FTSEngine`.

No additional framing/magic bytes are added by Arhiv; payload is exactly postcard bytes.

## 8. Document Locks Format (`document_locks.age`)

After AGE decrypt (no gzip): postcard binary payload of `HashMap<Id, DocumentLock>`.

## 9. Blob Format (`<asset_id>.age`)

After AGE decrypt (no gzip): raw original file bytes.

Write/read behavior:
- Encrypt path: stream-copy source file -> AGE writer.
- Decrypt path: AGE reader stream returned directly (supports `Read + Seek`).

Arhiv stores staged blobs in `state/data/` and committed blobs in `storage/data/` with identical on-disk format.

## 10. Key File Format (`key.age`)

`key.age` is AGE ASCII-armored encrypted data.

Plaintext bytes:
- Serialized x25519 secret key string (UTF-8), used as storage master key.

Operational notes:
- Password changes re-encrypt same plaintext master key with a new password-derived AGE key.
- Key export/import is armored AGE payload string round-trip.

## 11. Validation and Error Conditions

Container-level failures:
- Invalid index JSON -> parse failure.
- Missing or extra value lines relative to index length -> failure.
- Attempt to write before index -> failure.
- Attempt to write more/fewer lines than index length -> failure.

Storage-level failures:
- Index key parse failure (for document keys after `info`) -> failure.
- `info` line missing or invalid JSON -> failure.
- Document line invalid JSON for declared key -> failure.

State/search/locks failures:
- Decrypt failure with wrong key/password.
- Decompressed payload parse failure (JSON/postcard).

## 12. Determinism and Compatibility Notes

- Storage document ordering is deterministic due to explicit key sort.
- Container index preserves insertion/patch order semantics.
- GZIP encoder uses `Compression::fast()`; compressed bytes are not guaranteed stable across library/runtime changes even for identical plaintext.
- Compatibility gate currently enforced at runtime:
  - `storage_version == 1`
  - `data_version` must match schema latest data version.

## 13. Non-goals / Not Specified

- AGE internals (recipient stanza layout, stream chunk internals) are delegated to the `age` crate specification/implementation.
- Postcard internal schema evolution strategy is not separately version-tagged in these files.
- Backup/sync conflict file naming strategy is out of scope for file payload format.

## 14. Source of Truth (Code References)

Primary implementation files:
- `baza-storage/src/crypto/age.rs`
- `baza-storage/src/compression.rs`
- `baza-storage/src/container.rs`
- `baza/src/baza_storage/mod.rs`
- `baza/src/baza_storage/container_draft.rs`
- `baza/src/baza_storage/documents_index.rs`
- `baza/src/baza_info.rs`
- `baza/src/entities/document_key.rs`
- `baza/src/entities/revision.rs`
- `baza/src/baza_state/state_file.rs`
- `baza/src/baza_state/search.rs`
- `baza/src/baza_state/document_locks_file.rs`
- `baza/src/baza/blobs.rs`
- `baza/src/baza_manager/keys.rs`
- `baza/src/baza_paths.rs`

## 15. Key Lifecycle Operations (Implementation Contract)

This section documents key lifecycle behavior that is directly implemented in code.

Create:
- `BazaManager::create` generates a new storage x25519 key and writes it into `key.age` encrypted by password-derived AGE key.

Change password:
- `change_key_file_password(old_password, new_password)` decrypts existing `key.age`, re-encrypts same storage key with new password key, then locks state.
- Storage data does not get re-encrypted during password change; only `key.age` encryption changes.

Export key:
- `export_key(password, new_password)` decrypts local key file and re-encrypts it for export with `new_password`.
- Output is AGE armored text payload (string form) suitable for file/QR transfer.

Import key:
- `import_key(encrypted_key_data, password)` decrypts imported payload, validates imported storage key by reading existing storage, then replaces local `key.age`.
- On success, manager unlocks state with imported key.

Verify key:
- `verify_key(encrypted_key_data, password)` returns whether imported key can decrypt current storage.

Recovery implications:
- Password recovery without key material is not supported by format.
- If both password and exportable key material are unavailable, data is unrecoverable.

Code:
- `baza/src/baza_manager/keys.rs`
- `baza/src/baza_manager/mod.rs`

## 16. Backup/Restore and Durability Notes (Current Behavior)

Backup command behavior:
- CLI `backup` command copies:
  - `key.age` into `<backup_dir>/<timestamp>.key.age`
  - main storage db into `<backup_dir>/<timestamp>.baza.gz.age`
  - committed blob files into `<backup_dir>/data/` (copy-if-missing by blob file name)
- Backup directory must be an absolute existing directory.
- If there are staged (uncommitted) documents, backup logs warning and does not include those staged-only changes.

Restore behavior:
- There is no dedicated `restore` command in current CLI.
- Recovery is operationally performed by restoring backed-up files into storage layout and reopening Arhiv.

Durability/copy semantics:
- Backup uses file copies (`fs::copy`) and is not a transactionally consistent point-in-time snapshot across all files.
- Therefore, "safe backup" in current implementation means preserving decryptable key + storage + blobs, not strict atomic multi-file snapshot guarantees.

Code:
- `baza/src/backup.rs`
- `binutils/src/bin/arhiv.rs`

## 17. File Mutation Safety and Rollback Mechanics

Arhiv uses `FsTransaction` in several storage/key mutation paths.

Behavior:
- mutating operations can move previous files to backup names (`*-backup`),
- transaction `commit()` removes backup temp files,
- dropping an uncommitted transaction triggers rollback (best-effort reverse operations),
- rollback failures are surfaced and logged.

Important limitation:
- rollback is best-effort, not a hard atomic commit protocol across all touched files/directories.

Code:
- `baza-common/src/fs_transaction.rs`
- `baza/src/baza_manager/mod.rs`
- `baza/src/baza_manager/keys.rs`

## 18. Migration and Compatibility Policy (Current Code)

Compatibility gates:
- open/read requires exact match:
  - `storage_version == 1`
  - `data_version == schema.get_latest_data_version()`
- state refresh additionally requires `storage_info == state_info`.

Migration model:
- on open, multiple storage db files matching `baza*.gz.age` are merged into main db file by key-level union.
- this merge preserves unique `(id, rev)` snapshots and is not a schema/data migration transform.

Explicit non-goal in current implementation:
- no generic in-place storage format upgrader is specified in runtime flow.
- helper methods under `dangerously_*` exist for low-level/manual migration tasks and are not part of normal user workflow contract.

Code:
- `baza/src/baza/mod.rs`
- `baza/src/baza_manager/mod.rs`
- `baza/src/baza_manager/migration.rs`
