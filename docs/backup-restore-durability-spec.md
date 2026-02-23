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

Artifacts created per run:
1. `<backup_dir>/<timestamp>.key.age`
2. `<backup_dir>/<timestamp>.baza.gz.age`
3. Blob copies under `<backup_dir>/data/<asset_id>.age` (copy-if-missing)

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
- backup proceeds, but logs a warning that uncommitted changes are not included.

## 4. Definition of "Safe Backup" (Current)

A backup is considered "safe" if it preserves enough encrypted artifacts to reopen the same committed storage state later:
1. matching decryptable key file backup (`*.key.age`)
2. matching storage DB backup (`*.baza.gz.age`)
3. required committed blob files in `data/`

This is a recoverability definition, not a strict point-in-time atomicity guarantee.

## 5. Atomicity and Durability Guarantees

### 5.1 What is guaranteed

1. Individual copied files are complete or the operation errors.
2. Existing blob backups are not overwritten (copy-if-missing).
3. Backup does not mutate live storage/key data.

### 5.2 What is not guaranteed

1. No transactional all-files snapshot across key + DB + blobs.
2. No manifest proving one consistent generation set.
3. No built-in immutability or retention policy enforcement.

Implication:
- If live data changes during backup, backup artifacts may represent slightly different moments in time.

## 6. Restore Contract

There is no dedicated `restore` CLI command in current implementation.

Restore is an operator-managed file restoration process.

### 6.1 Restore Procedure (Operational)

1. Stop all Arhiv processes that may read/write storage/state.
2. Select one backup generation pair:
   - one timestamped `*.key.age`
   - one timestamped `*.baza.gz.age`
3. Replace live files:
   - restore selected key backup to `<storage_dir>/key.age`
   - restore selected DB backup to `<storage_dir>/baza.gz.age`
4. Ensure required blobs exist in `<storage_dir>/data/`:
   - restore/copy from backup `data/` as needed
5. Start Arhiv and unlock with corresponding password material.
6. Validate via `arhiv status` and representative document/blob reads.

### 6.2 Restore Guarantees (Current)

If restored artifacts are mutually compatible and uncorrupted:
1. Arhiv can decrypt/open committed storage snapshot.
2. State files can be regenerated/re-synced from storage on open.

If artifacts mismatch/corrupt:
1. open/read/decrypt fails with explicit errors
2. operator must choose another backup generation or repair manually

## 7. Corruption Detection and Repair Path

Detection happens indirectly through normal open/read flows:
1. decrypt failures (wrong key/password or corrupted encrypted data)
2. storage/state parse failures (invalid JSON/container/postcard)
3. compatibility/version gate failures (`storage_version`/`data_version`)

Repair path (current):
1. restore from known-good backup artifacts
2. if multiple storage DB files exist, open path merges them (`merge_storages`) when possible
3. if no good artifacts remain, data may be unrecoverable

No automated in-place repair tool is currently provided for arbitrary corruption.

## 8. Operational Recommendations

1. Run backups only when there are no staged changes, if you need full current state.
2. Keep backup directory on different physical storage/media.
3. Periodically perform restore drills in a disposable environment.
4. Keep key export and backup strategy coordinated.
5. Preserve multiple backup generations; do not rely on a single newest copy.

## 9. Known Gaps

1. No first-class restore command.
2. No transactional multi-file snapshot protocol.
3. No backup manifest with integrity hashes for all artifacts.

These are product/engineering gaps, not hidden behavior.

## 10. Source of Truth (Code References)

- `binutils/src/bin/arhiv.rs`
- `baza/src/backup.rs`
- `baza/src/baza_manager/manager_state.rs`
- `baza/src/baza/mod.rs`
- `baza/src/baza_manager/mod.rs`
- `baza/src/baza_paths.rs`
