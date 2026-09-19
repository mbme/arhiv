# Arhiv User Workflows

## Purpose and scope

This document explains the owner-facing outcomes for workflows that compose
Arhiv's domain, search, synchronization, recovery, and lifecycle rules. A
workflow is independent of a particular UI, CLI command, or platform launcher.

The domain model explains the concepts and invariants used here. The linked
technical documents provide details about storage, cryptography, APIs, and
platform security. This document focuses on how those behaviors compose into
observable owner workflows.

The Arhiv owner is the sole actor in these workflows. The owner is intentionally
not a domain object.

## Workflow rules

1. A workflow must leave committed Arhiv history valid under the domain model,
   or leave it unchanged when it fails or is discarded.
2. A workflow may expose platform-specific controls, but equivalent successful
   outcomes must have the same domain meaning on every supported surface.
3. A workflow that changes committed data must clearly distinguish staged work
   from committed work and state any recovery or rollback boundary.
4. Detailed preconditions and failure modes that belong to another canonical
   specification are incorporated by reference rather than duplicated here.

## 1. Open, unlock, and lock an Arhiv

### Goal

Open an existing Arhiv and make its committed data available locally.

### Flow

1. The owner supplies a password, imports a usable key, or uses an available
   platform-protected key cache.
2. Arhiv validates the resulting storage key by opening storage.
3. On success, the owner can read committed documents and prepare changes.
4. On lock, Arhiv removes the platform-protected cached storage key before it
   releases in-memory access.

### Outcomes and recovery

- Incorrect or unusable credentials fail without granting storage access.
- Losing both usable key material and the password material that decrypts it is
  unrecoverable; Arhiv has no server-side recovery service.
- A missing local key cache is recoverable with the password or an exported key.

See [Crypto and key lifecycle](crypto-key-lifecycle-threat-model.md) and
[Authentication and sessions](auth-session-trust-chain-spec.md).

## 2. Create, edit, add assets, commit, or discard documents

### Goal

Build a coherent change to structured knowledge and files without exposing
partially prepared work as committed history.

### Flow

1. The owner prepares a new document, edits an active document, or creates an
   asset from a file.
2. Unsaved form edits remain local to the current interface. Saving creates a
   staged change in Arhiv.
3. The owner may stage changes for multiple documents; together they form the
   implicit staging set.
4. Before commit, every staged version must satisfy its document type's required
   fields, constrained values, and relationship rules.
5. The owner either commits the staging set or discards one or more staged
   changes.

### Outcomes and recovery

- A commit atomically makes all staged versions current snapshots and makes
  their preceding current snapshots historical. A failed commit makes none of
  the staged versions current.
- Discarding a staged creation removes the proposed document. Discarding another
  staged change restores the preceding committed state or conflict.
- A newly created relationship can target only an existing active document in the
  same Arhiv.
- Asset metadata, relationships, and history remain usable when its blob is
  unavailable. Only reading that asset's content fails.

See the [Domain model](domain-model.md).

## 3. Erase a document

### Goal

Remove a document from active use while retaining the identity and historical
context required by Arhiv.

### Flow

1. The owner stages an erasure.
2. The owner commits the staging set or discards the erasure.
3. A committed erasure replaces the active version with an erased current
   snapshot.

### Outcomes and recovery

- An erased document retains its identity, but new incoming references,
  collection memberships, and asset references cannot target it.
- Retained historical relationships may still identify the document as erased.
- Discarding the staged erasure preserves the prior active snapshot.

See the [Domain model](domain-model.md).

## 4. Find and select documents

### Goal

Quickly find a current document to open, reference, or select.

### Flow

1. The owner enters a short query or browses the catalog.
2. Arhiv normalizes the query and returns only documents that match every
   normalized query term.
3. Arhiv orders eligible documents deterministically, favoring stronger matches
   in identifying fields.

### Outcomes and recovery

- An empty normalized query matches every indexed document.
- If a term has no candidate indexed term, the query returns no results.
- Search does not silently relax to partial-term, OR, semantic, or
  recommendation-style matching when a strict query has no results.

See [Full-text search](full-text-search-spec.md).

## 5. Reconcile concurrent changes

### Goal

Bring concurrent snapshots of the same document back to one current snapshot
without silently selecting one branch as the sole result.

### Flow

1. After external synchronization is incorporated, Arhiv detects concurrent
   snapshots and marks the document as conflicted.
2. Arhiv may prepare a heuristic staged merge.
3. The owner inspects or edits the staged merge.
4. The owner commits it to resolve the conflict, or discards it to retain the
   competing snapshots for later reconciliation.

### Outcomes and recovery

- A conflict remains until its staged merge is committed.
- Committing the staged merge creates one current snapshot and preserves the
  superseded snapshots in history subject to erasure rules.
- A staging set delays incorporation of incoming synchronized snapshots; the
  owner must commit or discard staged work before refresh can incorporate
  them.
- Unrelated staged changes may be committed while an unresolved conflict
  remains. Automatic commit waits until no conflicts exist.

See the [Domain model](domain-model.md) and
[Merge conflicts](merge-conflicts-spec.md).

## 6. Back up and restore committed data

### Goal

Create recoverable copies of committed storage, and validate or restore a
backup generation without silently replacing live data.

### Backup flow

1. The owner first commits or discards staged changes.
2. The owner chooses an existing absolute backup directory.
3. Arhiv creates a timestamped backup generation containing the key file,
   database file, committed blobs, and authenticated manifest.

### Restore flow

1. The owner runs a read-only restore check for a chosen manifest.
2. The owner may request deep blob verification when full plaintext-content
   validation is required.
3. After a successful preflight, the owner explicitly applies the restore.

### Outcomes and recovery

- A backup preserves committed state only; staged changes and local runtime
  state are excluded.
- A successful backup is recoverable, but it is not a transactional snapshot
  across all live files if those files change during backup.
- Restore apply refuses live staged changes and, by default, rollback to an
  older backup. It validates restored artifacts and clears runtime state so it
  can be regenerated from restored committed storage.
- Missing asset blobs may be restored only through the explicit degraded-restore
  option; the associated asset content remains unavailable.

See [Backup and restore](backup-restore-durability-spec.md) and
[Crypto and key lifecycle](crypto-key-lifecycle-threat-model.md).

## 7. Upgrade storage

### Goal

Open an Arhiv with a newer compatible release while preserving data and a
clear rollback path.

### Flow

1. The owner makes a backup and ensures local state is clean with no staged
   changes.
2. Arhiv unlocks storage, obtains exclusive storage ownership, and performs
   any required supported migration before normal state loading.
3. Arhiv validates the migrated storage before it becomes the active state.

### Outcomes and recovery

- A migration that cannot complete leaves pre-migration bytes available for
  rollback rather than silently continuing with partial replacement.
- During a migration window, mixed-version clients sharing one storage root are
  unsupported.
- If automatic migration stops because local state is dirty, the owner resolves
  those changes with the previous compatible version before retrying the
  upgrade.

See [Storage migrations](storage-migration-playbook.md).