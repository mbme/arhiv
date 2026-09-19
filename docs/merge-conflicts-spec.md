# Arhiv Merge Conflict Handling Spec

Scope: how Arhiv detects, represents, merges, exposes, and commits conflicting
document revisions.

## 1. Domain and synchronization terms

The [domain model](domain-model.md) defines document, document version,
snapshot, staged change, conflict, and staged merge. This document uses those
terms without redefining them.

Synchronization adds these technical terms:

- Stored snapshot: an immutable serialized document version in
  `storage/baza*.gz.age`.
- Revision (`rev`): the vector clock attached to a stored snapshot.
- Branch: a latest snapshot that is not causally superseded by another snapshot
  of the same document.
- Merge base: the common stored ancestor used for three-way reconciliation.
- Snapshot identity: the `(id, rev)` pair used to deduplicate stored snapshots.

## 2. Conflict Detection Model

### 2.1 Revision ordering

Each revision is a vector clock map `{instance_id -> counter}`.

Its causal comparison has four possible results:

- `Before`: all components <= other, at least one <
- `After`: all components >= other, at least one >
- `Equal`: all equal
- `Concurrent`: mixed (< on some, > on others)

A conflict branch exists when latest revisions for a document are concurrent.

### 2.2 Selecting latest branches per document

When loading storage, Arhiv computes the latest revisions:

- drops revisions dominated by newer ones
- keeps each concurrent branch

Result:

- one latest revision means the document has no conflict
- more than one latest revision means the document has a conflict

Base revision for 3-way merge:

- computed as the unique latest revision strictly older than every latest
  conflicting revision
- absent when there is no common stored ancestor or when multiple concurrent
  common ancestors have no single causal maximum

## 3. Storage-Level Merge (multiple db files)

Before opening state, Arhiv merges all `baza*.gz.age`-matching files in storage dir into main `baza.gz.age`.

Important details:

- filename matching accepts sync-conflict files such as `baza.gz.sync-conflict-...age`.
- merge is a key-level union using `(id, rev)` as snapshot identity, not a
  semantic document merge.
- duplicate keys are deduplicated.

## 4. State Refresh From Storage

### 4.1 Precondition gate

If state has any staged changes, refresh exits early with no merge/import.

Policy and implication:

- staged changes are expected to be brief;
- the global gate intentionally keeps their base state stable rather than
  importing remote snapshots while staged changes exist;
- incoming remote snapshots are not incorporated while staged changes exist.

### 4.2 Outdated document selection

For each document id in storage:

- compute latest revisions (+ optional merge base)
- skip if the current state already has exactly the same committed revision set
- otherwise mark as outdated and load required snapshots

Trust boundary:

- synchronized snapshots are treated as valid committed Arhiv history;
- refresh parses and merges them, but does not rerun local staging validation.

### 4.3 Conflict state construction

For each outdated id:

- rebuild the document's current state from the latest snapshots
- if conflict:
  - load base snapshot if base revision exists
  - run the semantic merge
  - store the merge result as a staged merge

When the merge produces a result, it becomes the staged merge. A conflict can
remain without one, although this refresh path does not normally
produce that state.

### 4.4 Snapshot count

After refresh, `snapshots_count` is updated from the full storage index count
for the document, including historical snapshots rather than only latest branches.

## 5. Semantic Merge Algorithm

Arhiv performs a field-aware three-way merge.

Input constraints:

- at least two competing snapshots
- same document id
- same document type (except erased-vs-non-erased handled specially)

Competing-snapshot ordering:

- snapshots sorted by `updated_at` ascending
- merged left-to-right

Erasure handling:

- all competing snapshots erased => return oldest erased snapshot
- mix erased/non-erased => drop erased snapshots
- if only one non-erased left => return it

Field strategies:

- `String`, `People`, `Countries`, `MarkupString`: word-level three-way text merge
- `RefList`: three-way list merge
- `Flag`, `NaturalNumber`, `Ref`, `Enum`, `Date`, `Duration`: last-write-wins

No conflict markers are emitted. Overlaps are synthesized into a single value
by algorithmic reconciliation.

## 6. Synchronization effects on the conflict lifecycle

### 6.1 Created

A conflict appears when refresh finds more than one concurrent latest snapshot
for the same document.

### 6.2 Surfaced

API/UI flags conflict via `has_conflict` and conflict count endpoint:

- list/get responses include `has_conflict`
- `CountConflicts` returns the number of conflicted documents
- catalog filter `onlyConflicts` supported

### 6.3 Staged merge

A staged merge can result from:

- automatic merge during state refresh from storage
- manual edit or save of a conflicted document

### 6.4 Committed

On commit:

- one new revision is computed globally from all competing revisions plus the
  local instance increment
- every staged change, including staged merges, is committed to that
  same new revision
- the document returns to one current snapshot
- old snapshots remain in storage history unless erased by erase rules

## 7. Commit and Auto-Commit Semantics

Manual commit:

- allowed when staged changes exist and no edit locks remain
- not blocked by presence of conflicts in general
- therefore conflicts without staged merges may coexist while unrelated staged
  changes are committed

Auto-commit:

- skips while any conflict exists
- requires clean no-conflict state to auto-commit

## 8. External Sync / Conflict Files

Arhiv expects external sync tools may create additional storage files (including sync-conflict variants).

Behavior:

- all matching storage db files are merged on open
- this preserves all distinct `(id, rev)` snapshots
- semantic merging then happens during state refresh, not during file merge

## 9. Invariants

- Revision maps contain only positive counters; parsing treats zero counters as
  absent so equality, hashing, ordering, and serialization share one canonical
  representation.
- All snapshots grouped under one document share the same id.
- A resolution's Commit revision must be strictly newer than every competing
  revision.
- Document id cannot change during staging.
- Erased snapshots cannot be modified directly.
- State/storage info (`data_version`, `storage_version`) must match before refresh.

## 10. Known Limitations / Behavioral Risks

1. When a conflict has no staged merge, selecting one branch for API
   and search projection is nondeterministic.

2. Auto-merge has no explicit conflict markers.

- Overlapping edits are combined heuristically (especially strings/lists), not surfaced as structured hunks.

3. Pairwise fold order for >2 branches uses `updated_at` ordering.

- Different timestamps can influence final merged payload.

4. State refresh is blocked when any staged change exists.

- Remote conflict updates are delayed until staged changes are discarded or
  committed.

## 11. End-to-End Flow (Typical Sync Conflict)

1. External sync creates/retains multiple `baza*.gz.age` files (possibly `sync-conflict` named).
2. Arhiv open path merges storage files into main db by unique `(id, rev)` keys.
3. State refresh computes latest concurrent revisions per id.
4. For conflicted ids, optional base revision is located.
5. Arhiv produces a staged merge for the conflicted document.
6. UI shows conflict indicator/count (`has_conflict`, `CountConflicts`).
7. User may inspect/edit staged result.
8. The commit writes a new snapshot revision and resolves the conflict.

## 12. Practical Observability Points

- CLI status warns when `conflicts_count > 0`.
- CLI `conflicts` lists conflicted documents, and `conflict show <id>` prints
  competing branches plus any staged merge.
- CLI `reset <id>` discards a staged merge and leaves the competing snapshots in
  conflict.
- CLI `history <id>`, `snapshot get <id> <rev>`, and `revert <id> <rev>` expose committed snapshots for inspection and staged rollback.
- CLI `diff conflict <id>` compares canonical document JSON data between
  conflict branches and the staged merge, when present.
- UI header shows conflict count button and catalog can filter to conflicts.
- Document payloads expose `hasConflict`, `isStaged`, and `snapshotsCount` for troubleshooting.

## 13. Consistency and Idempotency Contract

This section makes existing behavior explicit.

Storage-file merge idempotency:

- Merging storage files uses `(id, rev)` as snapshot identity.
- Re-merging the same effective set of snapshots does not create additional snapshots.
- Duplicate keys are deduplicated before write.

State refresh idempotency:

- Refresh compares the latest storage revision set with the current committed
  revision set for each document.
- If the sets are equal, the document is skipped as up-to-date.
- Re-running refresh without storage/state changes yields no additional state changes.

Deterministic parts:

- latest revision selection is deterministic for a fixed snapshot set.
- base-revision lookup is deterministic for a fixed revision graph.
- conflict/non-conflict classification is deterministic for a fixed snapshot set.

Known non-deterministic edge:

- Conflict projection can select any competing branch when no staged merge
  exists.
- this affects API/search projection only in that state.

## 14. Partial Sync and Concurrency Behavior

Incoming sync while staged changes exist:

- state refresh exits early.
- remote snapshots are not imported until staged changes are committed or
  discarded.

Implications:

- eventual convergence is deferred by staged changes.
- conflict counts and current document states can lag behind storage changes until
  the next successful refresh.
- this is intentional because staged changes are normally brief.