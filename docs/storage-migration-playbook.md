# Arhiv Storage Migration Playbook

Status: policy + implementation-derived constraints  
Current storage version: `1`  
Current data version: schema-defined (currently `2`)

## 1. Scope

This document defines how Arhiv storage migrations must be planned and executed.

It covers:
- `storage_version` lifecycle and compatibility policy
- Required migration invariants
- Preflight, execution, validation, rollback, and failure handling

It does not define:
- UI/API behavior changes unrelated to on-disk storage
- Document merge/conflict semantics (covered elsewhere)

## 2. Versioning Model

`BazaInfo` is the version gate:
- `storage_version`: on-disk storage/container semantics
- `data_version`: schema/data semantics

Runtime compatibility is strict:
- `state.get_info().storage_version == STORAGE_VERSION` is required
- `state.get_info().data_version == schema.get_latest_data_version()` is required

If either check fails, open/read fails.

## 3. When To Bump Which Version

Bump `storage_version` when changing any persistent low-level format/contract, for example:
- Container/index semantics (`info` ordering, key encoding, line contract)
- Compression/encryption layering by file type
- Storage file merge assumptions across `baza*.gz.age` files

Bump `data_version` when changing document/schema semantics, for example:
- Document field meaning/types/defaults
- Validation or conversion rules that affect stored document JSON

If both change, bump both in one coordinated release.

## 4. Non-Negotiable Invariants

Any migration MUST preserve these invariants unless explicitly superseded by a new version contract:

1. Storage `info` line remains authoritative and parseable.
2. Index-value cardinality stays exact (no missing/extra value lines).
3. All post-migration storage files that may be merged together have equal `BazaInfo`.
4. Document identity is `DocumentKey(id, rev)`; migration must not silently collapse distinct snapshots.
5. Migration is deterministic for identical input bytes and target version.
6. Failure cannot silently destroy pre-migration bytes.

## 5. Compatibility Policy

Current behavior is fail-fast compatibility:
- Binary built with `STORAGE_VERSION = 1` can open only `storage_version = 1`.
- Binary with newer `STORAGE_VERSION` is expected to migrate older versions before normal open.
- Older binaries are not required to open newer storage.

Operational policy:
- Mixed-version clients pointing to one shared storage are unsupported during migration windows.
- Upgrade must be coordinated as a single cutover operation per storage root.

## 6. Migration Procedure (Operational Playbook)

Data migrations with an in-repo migrator run automatically during `BazaManager` open after unlock and before normal state loading. Migrations still use the one-shot workflow below internally. If local state is dirty, automatic migration stops before changing storage and the user must resolve local changes with the previous compatible Arhiv version before upgrading again.

### 6.1 Preflight

1. Stop all other Arhiv processes (CLI server, desktop, Android sync access).
2. Acquire exclusive ownership of the storage root through the storage lock (no concurrent writers).
3. Verify unlock credentials are available; automatic migrations require an unlocked storage key.
4. Run a backup to an absolute path:
   - `arhiv backup /absolute/path/to/backup`
5. Verify backup artifacts exist:
   - timestamped `.key.age`
   - timestamped `.baza.gz.age`
   - `data/` blob directory entries as expected
6. Require clean local state before migration:
   - no staged local changes
   - no unresolved operational locks
   - no local state blobs

### 6.2 Execute Migration

1. Read current storage using old-version code path.
2. Transform to target version in memory/temporary files.
3. Validate transformed output against target invariants.
4. Atomically replace storage artifacts using transactional file operations (`FsTransaction` pattern).
5. Keep moved-aside backups until post-cutover checks pass.

### 6.3 Post-Migration Validation

Minimum required checks:
1. Open storage with target binary succeeds.
2. `status` reports expected `storage_version` and `data_version`.
3. Document count and blob references are consistent.
4. Read/list/commit smoke checks pass.
5. Re-open process (fresh process) still succeeds.

## 7. Failure and Rollback Handling

### 7.1 Failure Classes

1. Pre-write transform failure:
- No on-disk replacement happened.
- Keep original files; abort migration.

2. Replacement-stage failure:
- Rely on transaction backups (`*-backup`) for restoration.
- Do not continue with partially replaced files.

3. Post-cutover validation failure:
- Treat as failed migration.
- Roll back immediately to pre-migration backup set.

### 7.2 Rollback Procedure

1. Stop all writers/readers.
2. Restore key file and storage DB from pre-migration backup set.
3. Restore blobs if migration changed blob contracts (if applicable).
4. Start original binary version and verify open/status.
5. Preserve failed migrated artifacts for forensic analysis.

## 8. Migration Implementation Requirements

Any future in-repo migrator command/tool or automatic open-time migration MUST:
1. Be idempotent for already-migrated storage.
2. Refuse to run without exclusive lock.
3. Emit explicit source/target versions in logs.
4. Use crash-safe replacement strategy (transactional move/swap).
5. Abort on invariant violations with actionable diagnostics.
6. Include at least one rollback test and one corruption/failure-path test.

## 9. Data Migration: v1 to v2 Asset Content Hashes

Data version `2` adds mandatory readonly `asset.content_sha256`.

Migration from data version `1` to `2` MUST:
1. Preserve `storage_version = 1`.
2. Require an unlocked storage key.
3. Run under the exclusive storage lock.
4. Refuse dirty local state before rewriting storage, including staged document
   changes or local state blobs.
5. Process every stored document snapshot, including historical, conflict, and
   base snapshots.
6. Preserve document IDs, revisions, document types, timestamps, and all
   non-asset document data exactly.
7. For each asset snapshot, decrypt the referenced asset blob and compute
   `content_sha256` as uppercase hex SHA-256 of the plaintext bytes.
8. Fail the migration if any required asset blob is missing, unreadable, or
   cannot be decrypted.
9. Rewrite migrated storage artifacts transactionally and update `BazaInfo` to
   `data_version = 2`.
10. Rebuild or rewrite local state/search artifacts so subsequent normal open
    observes data version `2`.

Migration from data version `1` to `2` MUST NOT:
- verify `content_sha256` during normal asset reads after migration
- compute the hash from encrypted blob bytes
- migrate only current heads while leaving historical asset snapshots in the v1
  shape

## 10. Current Gaps and Interim Rules

Arhiv has a dedicated in-repo data migrator for data version `1` to `2`, but
there is currently no generic migration framework or first-class public
migration CLI command.

Until a public migration command is introduced:
- treat migrations as release-engineering operations
- require explicit backup + validation + rollback readiness
- do not perform ad-hoc partial file rewrites
- use `FsTransaction` for multi-step in-process rollback and dedicated atomic
  replacement helpers for single-file publish steps; neither replaces explicit
  backup and validation for migration operations

## 11. Source of Truth (Code References)

- `baza/src/baza_info.rs`
- `baza/src/baza_storage/mod.rs`
- `baza/src/baza_manager/mod.rs`
- `baza/src/baza_manager/migration/`
- `baza/src/baza/mod.rs`
- `baza/src/backup/`
- `baza-common/src/fs_transaction.rs`
