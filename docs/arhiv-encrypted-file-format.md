# Arhiv Encrypted File Format Specification

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
- Inner: GZIP-compressed payload at compression level 6
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

- x25519 private keys use the age identity text format.
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

- Embedded newlines in values are not supported because parsing is line-based.

## 5.3 Storage index semantics

Index keys are ordered and unique.

For storage DB specifically:

- Index entry 0 MUST be `"info"`.
- Entries 1..N are serialized `DocumentKey` values.
- Readers accept document keys in any order; canonical writer ordering is a
  normalization rule rather than a read-compatibility requirement.

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

When creating or rewriting storage:

- Document keys are sorted by document id, then by the lexicographic order of
  canonical revision entries `(instance_id, counter)`.
- Storage ordering is independent of vector-clock causal dominance.
- This is done to improve compression locality.
- First written value line is always `info`.
- Document lines follow sorted index order.

## 5.6 Patch semantics

Storage patches are applied during a complete canonical rewrite.

Rules:

- Existing key + replacement document: replace the document.
- Existing key + removal: remove the key and its value.
- New key + document: add the document.
- Deleting a missing key is invalid.
- Before writing, every document payload must match its `DocumentKey` (`id` and `rev`).
- Every completed rewrite uses the canonical storage ordering, placing all known
  revisions of a document together.
- Rewriting non-canonical v1 ordering does not change `storage_version`
  because both layouts satisfy the same reader and index-value contracts.

The patched output is fully rewritten as a new container and then encrypted/compressed.
The rewrite accepts source documents in their stored order and uses one
order-independent buffering path to emit the canonical order. See
ADR-002 in [Architecture decisions](architecture-decisions.md).

## 6. State File Format (`state.gz.age`)

After AGE decrypt + GZIP decompress: UTF-8 JSON object `BazaStateFile`.

Top-level fields:

- `instance_id`
- `info` (`BazaInfo`)
- `documents` (`HashMap<Id, DocumentHead>`)
- `refs` (`HashMap<DocumentKey, Refs>`)

`modified` is runtime-only and is not serialized.

## 7. Search Index Format (`search_index.gz.age`)

After AGE decrypt + GZIP decompress: postcard binary payload of
`SearchIndexFile`.

Fields:

- `format_version` (currently `1`)
- `search_version` (currently `5`)
- `data_version`
- `schema_fingerprint`
- `fts` (`FTSEngine`)

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
- GZIP uses compression level 6; compressed bytes are not guaranteed stable
  across library or runtime changes even for identical plaintext.
- Compatibility gate currently enforced at runtime:
  - `storage_version == 1`
  - `data_version` must match schema latest data version.
- Search-index reuse additionally requires matching format, search algorithm,
  data version, and schema fingerprint values. A mismatch causes the index to
  be rebuilt from current document state.

## 13. Non-goals / Not Specified

- AGE internals, including recipient stanza layout and stream chunk details,
  follow the age format specification.
- Postcard internal schema evolution strategy is not separately version-tagged in these files.
- Backup/sync conflict file naming strategy is out of scope for file payload format.

## 14. Related behavior

This document owns file contents and compatibility boundaries. Related
operations are explained elsewhere:

- [Crypto and key lifecycle](crypto-key-lifecycle-threat-model.md) explains
  creation, password changes, key import/export, cached credentials, and
  recovery.
- [Backup and restore](backup-restore-durability-spec.md) explains backup
  generations, manifests, verification, and restoration.
- [Storage migrations](storage-migration-playbook.md) explains version
  upgrades and rollback.
- [Merge conflicts](merge-conflicts-spec.md) explains storage-file union and
  semantic merging of concurrent document revisions.