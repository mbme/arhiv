# Storage Migration Contract

This contract owns version compatibility, automatic migration, and publication
safety. Use the [storage upgrade runbook](../runbooks/storage-upgrade.md) for
preflight, required post-upgrade checks, and rollback.

The [storage format](storage-format.md) owns container and byte-format rules;
the [schema contract](schema.md) owns document shape. Merge behavior belongs
to [merge conflicts](merge-conflicts.md).

## Version compatibility

`BazaInfo` carries two independent version gates:

- `storage_version`: persistent container and file-format semantics, including
  index layout, key encoding, encryption/compression, and storage-file merge
  compatibility.
- `data_version`: stored document shape and meaning.

The current binary supports `storage_version = 1` and schema
`data_version = 2`. After unlock, the supported data migration upgrades version
1 to 2 before normal state loading. After migration, both stored versions must
equal the supported versions. Other unsupported combinations fail rather than
opening partially.

Canonical writer ordering is normalization within storage version 1: canonical
and non-canonical document-key orders satisfy the same reader and index-value
contracts.

Mixed-version clients sharing one storage root are unsupported during migration;
upgrade is one cutover per root.

## Automatic data migration: v1 to v2

The dedicated migrator runs during `BazaManager` open after unlock, before normal
state loading. Dirty local state blocks it before storage changes; the owner
resolves staged changes, document locks, and local state blobs with the previous
compatible version before retrying.

The migrator:

1. Holds the exclusive storage lock.
2. Requires one common `BazaInfo` across every mergeable database.
3. Processes every stored snapshot, including historical, conflict, and base
   snapshots, preserving document identity, revision, type, timestamp, and
   non-asset data.
4. Decrypts each asset snapshot's blob and adds `content_sha256` as uppercase
   plaintext SHA-256. Missing, unreadable, or undecryptable required blobs fail
   migration.
5. Writes temporary storage with `data_version = 2` and `storage_version = 1`.
6. Replaces storage files through `FsTransaction`.
7. Removes state, search-index, lock, and staged-blob artifacts so normal open
   regenerates them from committed storage.
8. Commits the filesystem transaction only after replacements and cleanup
   succeed.

There is no generic migration framework or public migration CLI command.

## File publication and rollback

Migration uses the same publication primitives as other storage and key
mutations. `FsTransaction` moves previous files to `*-backup` names and publishes
replacements within an in-process rollback guard. Commit removes temporary
backups; dropping an uncommitted transaction attempts to reverse completed
operations. Rollback failures are surfaced and logged.

Successful create, rename, copy, link, and remove operations sync their
containing directory when the platform and filesystem support directory fsync.
Single-file publication uses `replace_file_atomically`: write and sync a
same-directory temporary file, rename it into place, then sync the directory.

These mechanisms provide best-effort rollback, not a hard atomic commit across
all files and directories or a crash-safe journal. Callers hold the relevant
application-level lock for shared paths. Neither mechanism replaces a verified
backup.

## Failure and verification boundary

Transform failure leaves original storage files unchanged. Replacement failure
triggers the uncommitted transaction's rollback attempt; callers must not
continue with partial replacement.

After successful cutover, transaction backups are gone. Failed post-upgrade
validation requires recovery from the pre-migration backup. The
[runbook](../runbooks/storage-upgrade.md#verify) defines the required open,
version, document/blob consistency, read/list/commit, and fresh-process checks.