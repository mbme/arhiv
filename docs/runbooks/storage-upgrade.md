# Upgrade and Roll Back Storage

Use this runbook before running a binary that changes the storage or data
version. The [migration contract](../specs/migration.md) owns compatibility,
automatic migration, and publication guarantees.

## Prepare

1. With the previous compatible Arhiv version, commit or reset staged changes
   and clear document locks or local state blobs.
2. Create a backup with that previous version:

   ```sh
   arhiv backup /absolute/path/to/backup
   ```

   The absolute backup directory must already exist. Running backup with the
   upgraded binary may open and migrate storage before creating the backup.

3. Verify the generation contains timestamped `.key.age`, `.baza.gz.age`, and
   `.manifest.age` files, plus the expected `data/` blob entries. See the
   [backup and restore contract](../specs/backup-restore.md).
4. Stop other Arhiv processes and prevent synchronization tools from modifying
   the storage root during the upgrade. Mixed-version clients sharing that
   root are unsupported during migration.
5. Ensure unlock credentials are available; migration needs the storage key.

## Upgrade

Start the upgraded binary and unlock storage. The supported v1-to-v2 data
migration runs automatically before normal state loading; there is no public
migration CLI command.

Dirty local state blocks migration before storage changes. Resolve it using
the previous compatible binary, then retry. Missing, unreadable, or
undecryptable asset blobs also block this migration.

## Verify

All of these checks are required:

1. Open storage successfully with the target binary.
2. Run `arhiv status` and confirm the expected storage and data versions.
3. Confirm document counts and blob references are consistent.
4. Exercise read/list/commit smoke checks.
5. Close and reopen in a fresh process; opening must still succeed.

Use a disposable copy for mutating smoke checks so validation does not change
the owner's data.

## Recover

- **Transform failure:** no storage replacement has occurred; retain the
  original files and resolve the reported blocker.
- **Replacement failure:** the uncommitted filesystem transaction attempts
  rollback. Do not continue using partially replaced storage.
- **Validation failure after cutover:** treat the upgrade as failed and
  restore the pre-migration backup. Successful transaction commit has already
  removed its temporary backups.

For rollback:

1. Stop all readers, writers, and synchronization touching the storage root.
2. Preserve failed migrated artifacts for forensic analysis before replacing
   them.
3. Restore the matching pre-migration key file and database, plus blobs if the
   migration changed their contract. Follow the
   [restore preflight and apply rules](../specs/backup-restore.md#7-restore-command-contract);
   restore is destructive and must use a binary compatible with the backup.
4. Start the original compatible binary and verify open/status.

In-process rollback is best-effort, not crash-safe multi-file atomicity. A
verified backup remains the recovery boundary.