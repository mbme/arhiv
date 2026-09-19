# Arhiv Storage Migration Playbook

Current storage version: `1`  
Current data version: schema-defined (currently `2`)

## 1. Scope

This document explains Arhiv's current storage version model, automatic data
migration, and the operational procedure for validating or rolling back an
upgrade.

It covers:

- `storage_version` lifecycle and compatibility policy
- migration safety properties
- preflight, execution, validation, rollback, and failure handling

It does not define:

- UI/API behavior changes unrelated to on-disk storage
- Document merge/conflict semantics (covered elsewhere)

## 2. Versioning Model

`BazaInfo` is the version gate:

- `storage_version`: on-disk storage/container semantics
- `data_version`: schema/data semantics

Runtime compatibility is strict:

- stored `storage_version` must equal the supported storage version
- stored `data_version` must equal the schema's latest data version

If either check fails, open/read fails.

## 3. Version meanings

`storage_version` identifies persistent container and file-format semantics,
including index layout, key encoding, encryption/compression layering, and
storage-file merge compatibility.

`data_version` identifies the meaning and required shape of stored document
JSON. The current v1-to-v2 migration changes `data_version` while preserving
`storage_version = 1`.

Canonical writer ordering is a normalization within storage version 1 because
both canonical and non-canonical ordering satisfy the same reader and
index-value contracts.

## 4. Compatibility behavior

The current binary opens only `storage_version = 1`. After unlock, it
automatically migrates `data_version = 1` storage to version 2 before loading
normal state. Other unsupported version combinations fail instead of opening
partially.

Mixed-version clients sharing one storage root are unsupported during migration.
The upgrade is therefore a single cutover per storage root.

## 5. Migration procedure

The v1-to-v2 data migration runs automatically during `BazaManager` open after
unlock and before normal state loading. If local state is dirty, migration
stops before changing storage. The owner must resolve the local changes with
the previous compatible Arhiv version before upgrading again.

The following steps describe the surrounding upgrade procedure and the work
performed by the migrator.

### 5.1 Preflight

1. Before starting the upgraded binary, use the previous compatible Arhiv
   version to commit or reset staged changes and clear document locks or local
   state blobs.
2. With that previous version, run a backup to an absolute path:
   - `arhiv backup /absolute/path/to/backup`
     Running this command with the upgraded binary may open and migrate storage
     before the backup is created.
3. Verify backup artifacts exist:
   - timestamped `.key.age`
   - timestamped `.baza.gz.age`
   - timestamped `.manifest.age`
   - `data/` blob directory entries as expected
4. Stop all other Arhiv processes and prevent synchronization tools from
   modifying the storage root during the upgrade.
5. Verify unlock credentials are available; automatic migration requires an
   unlocked storage key.

### 5.2 Execute migration

1. Acquire the exclusive storage lock.
2. Read every mergeable storage database and require one common `BazaInfo`.
3. Process every asset snapshot, decrypt its referenced blob, and compute the
   plaintext SHA-256 value.
4. Write a migrated temporary file for each storage database.
5. Replace the storage files through `FsTransaction`.
6. Remove state, search-index, lock, and staged-blob artifacts so normal open
   regenerates them from migrated committed storage.
7. Commit the filesystem transaction after all replacements and state cleanup
   succeed.

### 5.3 Post-migration validation

Minimum required checks:

1. Open storage with target binary succeeds.
2. `status` reports expected `storage_version` and `data_version`.
3. Document count and blob references are consistent.
4. Read/list/commit smoke checks pass.
5. Re-open process (fresh process) still succeeds.

## 6. Failure and rollback handling

### 6.1 Failure classes

1. Pre-write transform failure:

- No on-disk replacement happened.
- Keep original files; abort migration.

2. Replacement-stage failure:

- The uncommitted `FsTransaction` attempts to restore its moved-aside files.
- Do not continue with partially replaced files.

3. Post-cutover validation failure:

- Treat as failed migration.
- Restore the pre-migration backup set; transaction backups have already been
  removed after a successful transaction commit.

### 6.2 Rollback procedure

1. Stop all writers/readers.
2. Restore key file and storage DB from pre-migration backup set.
3. Restore blobs if migration changed blob contracts (if applicable).
4. Start original binary version and verify open/status.
5. Preserve failed migrated artifacts for forensic analysis.

## 7. Data migration: v1 to v2 asset content hashes

Data version `2` adds mandatory readonly `asset.content_sha256`.

The data-version 1 to 2 migrator:

1. preserves `storage_version = 1`;
2. requires an unlocked storage key;
3. runs under the exclusive storage lock;
4. refuses dirty local state before rewriting storage, including staged changes
   or local state blobs.
5. processes every stored document snapshot, including historical, conflict, and
   base snapshots.
6. preserves document IDs, revisions, document types, timestamps, and all
   non-asset document data exactly.
7. decrypts the referenced blob for each asset snapshot and computes
   `content_sha256` as uppercase hex SHA-256 of the plaintext bytes.
8. fails if any required asset blob is missing, unreadable, or
   cannot be decrypted.
9. rewrites migrated storage artifacts transactionally and updates `BazaInfo` to
   `data_version = 2`.
10. removes local state/search artifacts so subsequent normal open
    observes data version `2`.

It does not:

- verify `content_sha256` during normal asset reads after migration;
- compute the hash from encrypted blob bytes; or
- migrate only current document state while leaving historical asset snapshots
  in the version 1 shape.

## 8. Current limitations

Migration support consists of the dedicated open-time data migrator from
version 1 to 2. There is no generic migration framework or public migration CLI
command.

Operational migration work therefore uses explicit backup, validation, and
rollback preparation. `FsTransaction` provides in-process rollback for
multi-file replacement, while dedicated atomic replacement helpers publish
single files. Neither mechanism replaces a verified backup.

## 9. File publication and rollback

Migration uses the same publication primitives as other storage and key
mutations. `FsTransaction` can move previous files to `*-backup` names and
publish replacements as one in-process operation. Committing removes the
temporary backups; dropping an uncommitted transaction attempts to reverse its
completed operations. Rollback failures are surfaced and logged.

Successful create, rename, copy, link, and remove operations sync their
containing directory when the local platform and filesystem support directory
fsync. Single-file publication uses `replace_file_atomically`, which writes and
syncs a same-directory temporary file, renames it into place, and then syncs the
directory.

Rollback is best-effort rather than a hard atomic commit across every touched
file and directory. `FsTransaction` is an in-process rollback guard, not a
crash-safe journal, so callers hold the relevant application-level lock for
shared paths.