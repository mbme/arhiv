# Arhiv Merge Conflict Handling Spec

Status: implementation-aligned (current behavior)

Scope: how Arhiv detects, represents, merges, exposes, and commits conflicting document revisions.

## 1. Terminology

- Snapshot: an immutable committed document version stored in `storage/baza*.gz.age`.
- Revision (`rev`): a vector clock (`Revision`) attached to each committed snapshot.
- Document head (`DocumentHead`): state-layer representation of one logical document id, containing:
  - `original`: one or more committed snapshots (conflict branches when count > 1)
  - `staged`: optional working copy (rev=`initial`/null)
  - `snapshots_count`: total known snapshot count in storage for this id
- Conflict: `DocumentHead.original.len() > 1`.
- Unresolved conflict: conflict with no staged document.
- Resolved conflict: conflict with a staged merged document.

Primary code:
- `baza/src/entities/revision.rs`
- `baza/src/baza_state/document_head.rs`
- `baza/src/baza/mod.rs`
- `baza/src/merge_expert.rs`
- `baza/src/merge.rs`

## 2. Conflict Detection Model

### 2.1 Revision ordering

`Revision` is a vector clock map `{instance_id -> counter}`.

Comparison (`compare_vector_clocks`):
- `Before`: all components <= other, at least one <
- `After`: all components >= other, at least one >
- `Equal`: all equal
- `Concurrent`: mixed (< on some, > on others)

A conflict branch exists when latest revisions for a document are concurrent.

### 2.2 Selecting latest branches per document

When loading storage, Arhiv computes latest revisions with `LatestRevComputer`:
- drops revisions dominated by newer ones
- keeps one head per concurrent branch

Result:
- 1 latest revision => non-conflict head
- >1 latest revisions => conflict head

Base revision for 3-way merge:
- computed as maximum revision strictly older than every latest conflicting revision (`Revision::find_base_rev`)
- may be absent

## 3. Storage-Level Merge (multiple db files)

Before opening state, Arhiv merges all `baza*.gz.age`-matching files in storage dir into main `baza.gz.age`.

Important details:
- filename matching is fuzzy (`is_baza_file`), so sync-conflict files like `baza.gz.sync-conflict-...age` are included.
- merge is key-level union (snapshot identity = `DocumentKey(id, rev)`), not semantic document merge.
- duplicate keys are deduplicated.

Code:
- `baza/src/baza_manager/mod.rs::merge_storages`
- `baza/src/baza_paths.rs::list_storage_db_files`
- `baza/src/baza_storage/mod.rs::merge_storages`

## 4. State Refresh From Storage

Executed in `Baza::update_state_from_storage` via `update_state_from_storage`.

### 4.1 Precondition gate

If state has any staged documents, refresh exits early with no merge/import.

Implication:
- incoming remote snapshots are not incorporated while local staged changes exist.

### 4.2 Outdated document selection

For each document id in storage:
- compute latest revisions (+ optional merge base)
- skip if state already has exactly same original revision set
- otherwise mark as outdated and load required snapshots

### 4.3 Conflict head construction

For each outdated id:
- build `DocumentHead::new` from latest snapshots
- if conflict:
  - load base snapshot if base revision exists
  - run semantic merge (`MergeExpert::merge_originals(base, originals)`)
  - store merge result as `staged` via `document_head.modify(merged)`

Resulting conflict states:
- unresolved conflict: possible when conflict exists but no staged merge result (not produced by this path under normal conditions)
- resolved conflict: conflict + staged merged document (normal state after auto-merge)

### 4.4 Snapshot count

After refresh, `snapshots_count` is updated from full storage index count per id (all historical snapshots, not only latest branches).

## 5. Semantic Merge Algorithm

`MergeExpert` performs field-aware 3-way merge.

Input constraints:
- at least 2 originals
- same document id
- same document type (except erased-vs-non-erased handled specially)

Original ordering:
- originals sorted by `updated_at` ascending
- merged left-to-right

Erasure handling:
- all originals erased => return oldest erased snapshot
- mix erased/non-erased => drop erased originals
- if only one non-erased left => return it

Field strategies:
- `String`, `People`, `Countries`, `MarkupString`: word-level three-way text merge (`merge_strings_three_way`)
- `RefList`: three-way slice merge (`merge_slices_three_way`)
- `Flag`, `NaturalNumber`, `Ref`, `Enum`, `Date`, `Duration`: last-write-wins (`value_b` in pairwise fold)

No conflict markers are emitted. Overlaps are synthesized into a single value by algorithmic reconciliation.

Code:
- `baza/src/merge_expert.rs`
- `baza/src/merge.rs`

## 6. Conflict Lifecycle

### 6.1 Created

Conflict appears when state head has >1 latest original snapshots for same id.

### 6.2 Surfaced

API/UI flags conflict via `has_conflict` and conflict count endpoint:
- list/get document responses include `has_conflict`
- `CountConflicts` returns count of heads where `is_conflict()`
- catalog filter `onlyConflicts` supported

Code:
- `arhiv/src/server/ui_server/api_handler.rs`
- `arhiv/src/ui/dto.rs`
- `arhiv/src/ui/dto.ts`
- `baza/src/baza_state/query.rs`

### 6.3 Resolved (staged)

Conflict becomes resolved when `DocumentHead` has staged document (`is_resolved_conflict`).

This can happen by:
- automatic merge during state refresh from storage
- manual edit/save of a conflicted document (stage_document modifies head)

### 6.4 Committed

On commit:
- one new revision is computed globally from all original revisions + local instance increment
- every staged document (including resolved conflicts) is committed to that same new revision
- committed head becomes single-snapshot (non-conflict)
- old snapshots remain in storage history unless erased by erase rules

Code:
- `baza/src/baza_state/mod.rs::commit`
- `baza/src/baza/mod.rs::commit`

## 7. Commit and Auto-Commit Semantics

Manual commit:
- allowed as long as there are staged docs and no document locks
- not blocked by presence of conflicts in general
- therefore unresolved conflicts may coexist while unrelated staged docs are committed

Auto-commit:
- explicitly skips when `baza.has_conflicts()` is true
- requires clean no-conflict state to auto-commit

Code:
- `baza/src/auto_commit_service.rs`

## 8. External Sync / Conflict Files

Arhiv expects external sync tools may create additional storage files (including sync-conflict variants).

Behavior:
- all matching storage db files are merged on open
- this preserves all distinct `(id, rev)` snapshots
- semantic conflict resolution then happens at state refresh stage, not during file merge

## 9. Invariants

- All snapshots inside one `DocumentHead` share same document id.
- Commit revision must be strictly newer than every original revision in that head.
- Document id cannot change during stage/modify.
- Erased original documents cannot be modified directly.
- State/storage info (`data_version`, `storage_version`) must match before refresh.

## 10. Known Limitations / Behavioral Risks

1. `DocumentHead::get_single_document()` returns first item from unordered `HashSet` when conflict has no staged doc.
- This makes projected document data potentially nondeterministic for unresolved conflicts.
- Search indexing and API projections rely on `get_single_document()`.
- File note already contains `// FIXME this also wrong`.

2. Auto-merge has no explicit conflict markers.
- Overlapping edits are combined heuristically (especially strings/lists), not surfaced as structured hunks.

3. Pairwise fold order for >2 branches uses `updated_at` ordering.
- Different timestamps can influence final merged payload.

4. State refresh is blocked when any staged document exists.
- Remote conflict updates are delayed until local staged state is cleared/committed.

## 11. End-to-End Flow (Typical Sync Conflict)

1. External sync creates/retains multiple `baza*.gz.age` files (possibly `sync-conflict` named).
2. Arhiv open path merges storage files into main db by unique `(id, rev)` keys.
3. State refresh computes latest concurrent revisions per id.
4. For conflicted ids, optional base revision is located.
5. `MergeExpert` produces merged document and stages it on conflicted head.
6. UI shows conflict indicator/count (`has_conflict`, `CountConflicts`).
7. User may inspect/edit staged result.
8. Commit writes new snapshot revision, collapsing head to single committed snapshot.

## 12. Practical Observability Points

- CLI status warns when `conflicts_count > 0`.
- UI header shows conflict count button and catalog can filter to conflicts.
- Document payloads expose `hasConflict`, `isStaged`, and `snapshotsCount` for troubleshooting.

Code:
- `arhiv/src/arhiv/status.rs`
- `arhiv/src/ui/Workspace/WorkspaceHeader/ConflictsButton.tsx`
- `arhiv/src/ui/Workspace/DocumentCard/Indicators.tsx`

## 13. Consistency and Idempotency Contract

This section makes existing behavior explicit.

Storage-file merge idempotency:
- Merging storage files is key-identity based (`DocumentKey(id, rev)`).
- Re-merging the same effective set of snapshots does not create additional snapshots.
- Duplicate keys are deduplicated before write.

State refresh idempotency:
- Refresh compares latest storage revision set vs state original revision set per document id.
- If sets are equal, the document is skipped as up-to-date.
- Re-running refresh without storage/state changes yields no additional state changes.

Deterministic parts:
- latest revision selection is deterministic for a fixed snapshot set.
- base-revision lookup is deterministic for a fixed revision graph.
- conflict/non-conflict classification is deterministic for a fixed snapshot set.

Known non-deterministic edge:
- unresolved conflict projection can be nondeterministic because `get_single_document()` selects first element of unordered `HashSet`.
- this affects API/search projection only when conflict has no staged merged doc.

Code:
- `baza/src/baza/mod.rs`
- `baza/src/baza_state/document_head.rs`
- `baza/src/baza_storage/mod.rs`

## 14. Partial Sync and Concurrency Behavior

Incoming sync while local staged state exists:
- `update_state_from_storage` exits early when any staged documents exist.
- remote snapshots are not imported until staged changes are committed/cleared.

Implications:
- eventual convergence is deferred by local staged state.
- conflict counts/heads can lag behind storage changes until next successful refresh.

Code:
- `baza/src/baza/mod.rs`
- `baza/src/baza_manager/mod.rs`
