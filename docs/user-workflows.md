# Arhiv User Workflows

## Scope

This document describes owner-visible workflows independent of UI, CLI, or
launcher details. The [domain model](domain-model.md) owns domain invariants;
linked specifications own technical preconditions and failure modes.
The owner is the sole workflow actor and is not a domain object.

Every workflow must either leave committed history valid or leave it unchanged.
Equivalent successful outcomes have the same domain meaning on every supported
surface. Workflows that change committed data distinguish staged work from
committed work and identify their recovery boundary.

## 1. Open, unlock, and lock

1. The owner supplies a password, imports a usable key, or uses an available
   platform-protected key cache.
2. Arhiv validates the resulting storage key by opening storage.
3. On success, committed documents become available and changes may be staged.
4. Lock durably removes the platform key cache before releasing in-memory
   access.

Incorrect credentials do not grant access. A missing cache is recoverable with
the password or an exported key; losing both usable key material and the
password needed to decrypt it is unrecoverable.

See [Crypto and key lifecycle](crypto-key-lifecycle-threat-model.md) and
[Authentication and sessions](auth-session-trust-chain-spec.md).

## 2. Create, edit, commit, or discard

1. The owner prepares a document edit, creation, erasure, or asset import.
2. Unsaved form edits remain interface-local; saving creates a staged change.
3. Saved changes for multiple documents form one staging set.
4. Before commit, every staged version must satisfy its type and relationship
   rules.
5. The owner commits the set atomically or discards selected staged changes.

A failed commit makes none of the staged versions current. Discard restores the
preceding committed state or conflict; discarding a staged creation removes it.
New relationships may target only existing active documents in the same Arhiv.
If an asset blob is unavailable, its metadata, relationships, and history remain
usable, while reading its content fails.

See the [Domain model](domain-model.md).

## 3. Erase a document

The owner stages an erasure, then commits or discards it. Commit replaces the
active version with an erased current snapshot; discard preserves the active
snapshot. An erased document keeps its identity and historical relationships,
but cannot receive new references, collection memberships, or asset references.

See the [Domain model](domain-model.md).

## 4. Find and select documents

The owner enters a short query or browses the catalog. Arhiv returns documents
matching every normalized query term and orders them deterministically,
favoring stronger matches in identifying fields.

An empty normalized query matches every indexed document. A term with no
candidate produces no results; search does not fall back to OR, partial-term,
semantic, or recommendation matching.

See [Full-text search](full-text-search-spec.md).

## 5. Reconcile concurrent changes

1. Refresh detects concurrent snapshots and marks the document conflicted.
2. Arhiv may prepare a heuristic staged merge.
3. The owner inspects or edits that merge, then commits it or discards it for
   later reconciliation.

The conflict remains until its staged merge is committed. Commit creates one
current snapshot and retains superseded history subject to erasure rules.
Incoming snapshots wait while any staged changes exist. Unrelated staged changes
may still be committed while a conflict remains, but automatic commit waits
until no conflicts exist.

See [Merge conflicts](merge-conflicts-spec.md).

## 6. Back up and restore

Before backup, the owner commits or discards staged changes and selects an
existing absolute backup directory. Arhiv creates a timestamped generation
containing the key file, committed database, committed blobs, and authenticated
manifest.

Restore starts with a read-only check of a selected manifest; deep blob
verification is optional. After successful preflight, the owner explicitly
applies the restore.

Backups exclude staged and runtime state and are not transactional snapshots
across concurrently changing files. Restore apply refuses staged live changes
and, by default, rollback to an older generation. It validates restored
artifacts and clears regenerable runtime state. Missing blobs require the
explicit degraded-restore option and leave their asset content unavailable.

See [Backup and restore](backup-restore-durability-spec.md).

## 7. Upgrade storage

Before upgrading, the owner creates a backup and clears staged and local state.
After unlock, Arhiv obtains exclusive storage ownership, runs any supported
migration, and validates migrated storage before normal loading.

A failed migration retains pre-migration bytes for rollback instead of
continuing with partial replacement. Mixed-version clients sharing a storage
root are unsupported during migration. If dirty local state blocks migration,
the owner resolves it with the previous compatible version before retrying.

See [Storage migrations](storage-migration-playbook.md).
