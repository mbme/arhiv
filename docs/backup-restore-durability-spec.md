# Arhiv Backup, Restore, and Durability Specification

Status: implementation-aligned (current behavior)

## 1. Scope

This document specifies current backup/restore behavior and durability guarantees for Arhiv.

It covers:
- what `arhiv backup` includes and excludes
- what "safe backup" means in current implementation
- restore procedure and guarantees
- corruption detection and repair paths

It does not define:
- future transactional snapshot features
- cloud/sync provider-level durability

## 2. Backup Command Contract

Entry point:
- CLI command: `arhiv backup <absolute_backup_dir>`
- Implementation: `BazaManager::backup`

Preconditions:
1. Backup directory path must be absolute.
2. Backup directory must already exist.
3. Arhiv must be unlockable (CLI unlock flow is used before backup).
4. Current Arhiv state must have no staged changes.

Artifacts created per run:
1. `<backup_dir>/<timestamp>.key.age`
2. `<backup_dir>/<timestamp>.baza.gz.age`
3. Blob copies under `<backup_dir>/data/<asset_id>.age` (copy-if-missing)
4. `<backup_dir>/<timestamp>.manifest.age`

Timestamp format:
- `YYYY-MM-DD_HH-MM-SS` (local clock formatting used by current code).

## 3. Included vs Excluded Data

Included:
1. Current key file (`key.age`) snapshot at backup time.
2. Current main storage DB file (`baza.gz.age`) snapshot at backup time.
3. Committed storage blobs from `storage/data`.

Excluded:
1. Staged/uncommitted state changes.
2. State runtime files (`state.gz.age`, `search_index.gz.age`, `document_locks.age`).
3. Staged/local blobs in `state/data`.

Current behavior on staged changes:
- backup fails because staged/local state is excluded from backup artifacts.

Current behavior on generation name collisions:
- backup refuses to overwrite existing `<timestamp>.key.age`, `<timestamp>.baza.gz.age`, or
  `<timestamp>.manifest.age` artifacts.

## 4. Definition of "Safe Backup" (Current)

A backup is considered "safe" if it preserves enough encrypted artifacts to reopen the same committed storage state later:
1. matching decryptable key file backup (`*.key.age`)
2. matching storage DB backup (`*.baza.gz.age`)
3. required committed blob files in `data/`
4. authenticated manifest that binds the backup generation to its copied artifact bytes

This is a recoverability definition, not a strict point-in-time atomicity guarantee.

Missing committed blobs are asset-content loss, not database corruption. Metadata can still open
when the missing asset blob is not read. Blob reads fail when requested, and normal maintenance/status
paths report missing referenced blobs.

## 5. Atomicity and Durability Guarantees

### 5.1 What is guaranteed

1. Individual copied files are complete or the operation errors.
2. Existing generation artifacts and blob backups are not overwritten.
3. Backup does not mutate live storage/key data.

### 5.2 What is not guaranteed

1. No transactional all-files snapshot across key + DB + blobs.
2. Manifest proves copied artifact byte integrity, not transactional capture across concurrently
   changing live files.
3. No built-in immutability or retention policy enforcement.

Implication:
- If live data changes during backup, backup artifacts may represent slightly different moments in time.

## 6. Backup Manifest Contract

The backup manifest is an encrypted, authenticated AGE payload written as
`<backup_dir>/<timestamp>.manifest.age`.

Confidentiality/integrity:
1. The manifest is encrypted with the backed-up storage master key.
2. Restore obtains that key by decrypting the same-generation `<timestamp>.key.age`
   with the backup key password.
3. AGE authentication is the manifest tamper-detection mechanism.
4. The manifest is not separately signed. Arhiv does not define a signing-key hierarchy
   or rollback/freshness protection for backup generations.

Ownership:
1. Manifest owns backup packaging/generation facts:
   - manifest format version
   - backup timestamp and tool version
   - relative key, DB, and blob artifact paths
   - ciphertext SHA-256 for each artifact
2. The DB owns asset truth:
   - referenced asset IDs
   - asset metadata
   - per-asset blob key
   - `asset.content_sha256`
   - plaintext size
3. The manifest must not duplicate DB-owned asset plaintext hashes.

## 7. Restore Command Contract

Dedicated restore commands:
1. `arhiv restore check <manifest-path>`
2. `arhiv restore apply <manifest-path>`

Options:
1. `--allow-missing-blobs` permits degraded restore only when referenced blob artifacts
   are missing. Missing/corrupt key files, missing/corrupt DB files, key/DB mismatch,
   unreadable manifests, and artifact hash mismatches remain fatal.
2. `--deep` additionally decrypts every referenced blob and verifies plaintext size and
   SHA-256 against DB asset metadata. Deep verification is explicit because it reads and
   decrypts all referenced blobs.
3. `--allow-rollback` permits `restore apply` to replace newer current storage with an
   older backup. It does not permit staged changes.

### 7.1 Restore Check

`restore check` is read-only with respect to live storage. It:
1. derives the same-generation key artifact path from `<manifest-path>`
2. prompts for the backup key password
3. decrypts the backed-up key file
4. decrypts/authenticates the manifest
5. verifies manifest-listed artifact ciphertext hashes
6. opens the backed-up DB with the backed-up storage key
7. verifies that DB-referenced asset IDs have corresponding blob artifacts unless
   `--allow-missing-blobs` is supplied
8. performs deep plaintext verification only when `--deep` is supplied

### 7.2 Restore Apply

`restore apply` is mutating and destructive. It:
1. requires current Arhiv storage to be unlocked so live safety checks can inspect state
2. refuses to run when the live storage lock is held
3. runs restore preflight before live mutation
4. refuses to run when current Arhiv state has staged changes
5. refuses to replace newer current storage with an older backup unless `--allow-rollback`
   is supplied
6. applies live file mutations through the filesystem transaction helper and rolls back
   uncommitted mutations on failure
7. replaces live `key.age` and `baza.gz.age` only with bytes that match the manifest hashes
8. copies referenced backed-up blobs into live `storage/data/` only when copied bytes match the manifest hashes
9. clears runtime state files (`state.gz.age`, `search_index.gz.age`,
   `document_locks.age`, and `state/data/*`) so restored committed DB state is canonical
10. validates the restored storage DB with the restored storage key before committing
    filesystem transaction rollback state
11. leaves state/search/locks regeneration to the next normal open

### 7.3 Restore Guarantees

If restored artifacts are mutually compatible and uncorrupted:
1. Arhiv can decrypt/open committed storage snapshot.
2. State files can be regenerated/re-synced from storage on open.

If artifacts mismatch/corrupt:
1. open/read/decrypt fails with explicit errors
2. operator must choose another backup generation or repair manually

## 8. Corruption Detection and Repair Path

Detection happens indirectly through normal open/read flows:
1. decrypt failures (wrong key/password or corrupted encrypted data)
2. storage/state parse failures (invalid JSON/container/postcard)
3. compatibility/version gate failures (`storage_version`/`data_version`)

Repair path (current):
1. restore from known-good backup artifacts
2. if multiple storage DB files exist, open path merges them (`merge_storages`) when possible
3. if no good artifacts remain, data may be unrecoverable

No automated in-place repair tool is currently provided for arbitrary corruption.

## 9. Operational Recommendations

1. Run backups only when there are no staged changes, if you need full current state.
2. Keep backup directory on different physical storage/media.
3. Periodically perform restore drills in a disposable environment.
4. Keep key export and backup strategy coordinated.
5. Preserve multiple backup generations; do not rely on a single newest copy.

## 10. Known Gaps

1. No transactional multi-file snapshot protocol.
2. No built-in backup rollback/freshness protection.
3. Deep blob plaintext verification is explicit, not part of default restore.

These are product/engineering gaps, not hidden behavior.

## 11. Source of Truth (Code References)

- `arhiv-cli/src/bin/arhiv.rs`
- `baza/src/backup/`
- `baza/src/baza_manager/manager_state.rs`
- `baza/src/baza/mod.rs`
- `baza/src/baza_manager/mod.rs`
- `baza/src/baza_paths.rs`
