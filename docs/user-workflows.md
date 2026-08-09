# Arhiv User Workflows

Status: intended durable product workflow contract

## Purpose and scope

This document specifies the owner-facing outcomes for workflows that compose
Arhiv's domain, search, synchronization, recovery, and lifecycle rules. A
workflow is independent of a particular UI, CLI command, or platform launcher.

The domain model remains the source of truth for domain concepts and
invariants. The linked specifications remain the source of truth for their
technical contracts. This document does not redefine storage, cryptographic,
API, or platform-security behavior; it defines how those contracts compose into
observable owner workflows.

The Arhiv owner is the sole actor in these workflows. The owner is intentionally
not a domain object.

## Workflow rules

1. A workflow must leave committed Arhiv history valid under the domain model,
   or leave it unchanged when it fails or is discarded.
2. A workflow may expose platform-specific controls, but equivalent successful
   outcomes must have the same domain meaning on every supported surface.
3. A workflow that changes committed data must clearly distinguish pending work
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
3. On success, the owner can read committed records and prepare changes.
4. On lock, Arhiv removes the platform-protected cached storage key before it
   releases in-memory access.

### Outcomes and recovery

- Incorrect or unusable credentials fail without granting storage access.
- Losing both usable key material and the password material that decrypts it is
  unrecoverable; Arhiv has no server-side recovery service.
- A missing local key cache is recoverable with the password or an exported key.

See `docs/crypto-key-lifecycle-threat-model.md` and
`docs/auth-session-trust-chain-spec.md`.

## 2. Create, edit, attach, commit, or discard records

### Goal

Build a coherent change to structured knowledge and files without exposing
partially prepared work as committed history.

### Flow

1. The owner prepares a new Record, edits an active Record, or attaches a file
   through a role allowed by the relevant Record kind.
2. Arhiv keeps each prepared form as a Pending change. The owner may prepare
   changes for multiple Records; together they are the one implicit pending
   change set.
3. Before commit, every pending form must satisfy its kind's required details,
   constrained values, and relationship rules.
4. The owner either commits the pending change set or discards one or more
   pending changes.

### Outcomes and recovery

- Commit atomically makes all pending forms current and makes their preceding
  current forms historical. A failed commit makes none of the pending forms
  current.
- Discarding a pending creation removes the proposed Record. Discarding another
  pending change restores the preceding committed form or conflict state.
- A newly created relationship can target only an existing active Record in the
  same Arhiv.
- Attachment metadata, relationships, and history remain usable when its file
  content is unavailable. Only reading that attachment's content fails.

See `docs/domain-model.md`.

## 3. Delete a record

### Goal

Remove a Record from active use while retaining the identity and historical
context required by Arhiv.

### Flow

1. The owner prepares a deletion as a Pending change.
2. The owner commits the pending change set or discards the deletion.
3. A committed deletion replaces the active form with an erased current form.

### Outcomes and recovery

- A deleted Record retains its identity, but new incoming references,
  collection memberships, and attachment relationships cannot target it.
- Retained historical relationships may still identify the Record as deleted.
- Discarding the pending deletion preserves the prior active form.

See `docs/domain-model.md`.

## 4. Find and select records

### Goal

Quickly find a current Record to open, reference, or select.

### Flow

1. The owner enters a short query or browses the catalog.
2. Arhiv normalizes the query and returns only Records that match every
   normalized query term.
3. Arhiv orders eligible Records deterministically, favoring stronger matches
   in identifying fields.

### Outcomes and recovery

- An empty normalized query matches every indexed Record.
- If a term has no candidate indexed term, the query returns no results.
- Search does not silently relax to partial-term, OR, semantic, or
  recommendation-style matching when a strict query has no results.

See `docs/full-text-search-spec.md`.

## 5. Reconcile concurrent changes

### Goal

Bring concurrent committed forms of the same Record back to one current form
without silently selecting one branch as the sole result.

### Flow

1. After external synchronization is incorporated, Arhiv detects concurrent
   committed forms and marks the Record as a Conflict.
2. Arhiv may prepare a heuristic merged form as a Pending reconciled change.
3. The owner inspects or edits that proposed result.
4. The owner commits it to resolve the Conflict, or discards it to retain the
   competing committed forms for later reconciliation.

### Outcomes and recovery

- A Conflict remains until a reconciled Pending change is committed.
- Committing the reconciled change creates one current form and preserves the
  superseded forms in history subject to deletion rules.
- A pending change set delays incorporation of incoming synchronized snapshots;
  the owner must commit or clear pending work before refresh can incorporate
  them.
- Unrelated pending changes may be committed while an unresolved Conflict
  remains. Automatic commit waits until no Conflicts exist.

See `docs/domain-model.md` and `docs/merge-conflicts-spec.md`.

## 6. Back up and restore committed data

### Goal

Create recoverable copies of committed storage, and validate or restore a
backup generation without silently replacing live data.

### Backup flow

1. The owner first commits or discards pending changes.
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
  option; the associated attachment content remains unavailable.

See `docs/backup-restore-durability-spec.md` and
`docs/crypto-key-lifecycle-threat-model.md`.

## 7. Upgrade storage

### Goal

Open an Arhiv with a newer compatible release while preserving data and a
clear rollback path.

### Flow

1. The owner makes a backup and ensures local state is clean with no pending
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

See `docs/storage-migration-playbook.md`.
