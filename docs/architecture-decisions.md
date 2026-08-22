# Architecture Decisions

Status: decision record

This document is the append-only record of significant architecture decisions in Arhiv. It preserves the context, decision, and consequences so that durable choices are not needlessly re-litigated.

Canonical specifications and project guidance remain authoritative for their respective requirements. An ADR records the rationale for a decision; it does not replace or silently change a governing specification.

## Conventions

- Add decisions in numeric order as `ADR-001`, `ADR-002`, and so on. Do not reuse identifiers.
- Use one of these statuses: `Proposed`, `Accepted`, `Superseded`, or `Deprecated`.
- Keep accepted and superseded decisions in this document. A superseding ADR must link to the decision it replaces.
- Record material alternatives and their tradeoffs.
- Update the affected canonical specification when a decision changes a requirement.

## Decisions

## ADR-001: Store mandatory plaintext content hashes for assets

- Status: Accepted
- Date: 2026-08-07

### Context

Assets store encrypted blob bytes separately from their document metadata. The
asset document records filename, media type, size, and per-asset AGE key
material, but it does not record a stable identity for the plaintext content.
That makes content identity, auditability, dedupe, restore validation, and
future asset tooling harder to implement consistently.

The content identity must remain stable across re-encryption, backup/restore,
and device-local storage differences. Existing asset blobs are AGE-encrypted and
authenticated, so normal reads already fail on corrupted ciphertext or wrong
keys without requiring a second read-path hash check.

### Decision

Asset documents store a mandatory readonly `content_sha256` field containing the
uppercase hex SHA-256 digest of the asset plaintext bytes.

The `content_sha256` value is computed from the exact plaintext byte stream that
is encrypted into the blob. New asset creation computes the hash while streaming
plaintext into the `AgeWriter`, making the copied bytes the source of truth for
both blob content and asset metadata.

Adding the mandatory field changes the strict asset data contract, so the schema
`data_version` moves from `1` to `2`. Existing v1 storage is upgraded by an
explicit v1-to-v2 migration that decrypts every stored asset blob and backfills
`content_sha256` into every stored asset document snapshot. Migration fails if
any required asset blob is missing, unreadable, or cannot be decrypted.

Normal asset reads do not verify `content_sha256`. Hash verification belongs in
explicit status, verify, repair, restore, or dedupe workflows.

### Consequences

- Asset plaintext content has a durable, schema-owned identity value.
- The hash remains stable across blob re-encryption and storage movement.
- Existing v1 stores require a one-shot data migration before normal v2 open;
  the supported migrator runs automatically during `BazaManager` open when local
  state is clean.
- Migration must process historical and conflict/base snapshots, not only
  current document heads, because all strict asset JSON payloads must be
  v2-compatible.
- Migration requires access to decrypted asset bytes and therefore requires an
  unlocked storage key.
- Missing or unreadable asset blobs become migration blockers instead of being
  silently carried forward.
- Staged local changes, document locks, and local state blobs block automatic
  migration; users must resolve those with the previous compatible version and
  then run the upgraded version again.
- Read-path performance and behavior remain unchanged because normal asset reads
  rely on AGE authentication rather than stored-hash verification.

### Alternatives considered

- Optional `content_sha256`: rejected because it would make content identity a
  best-effort property and preserve two asset-data shapes indefinitely.
- Ciphertext or blob-file hash: rejected because it changes across
  re-encryption and is not a stable content identity.
- Pre-hash source files before encryption: rejected because a separate read can
  drift from the bytes actually encrypted.
- Verify the hash on every asset read: rejected because AGE already
  authenticates blob bytes and read-path verification is a separate product
  behavior with different performance and UX tradeoffs.

## ADR-002: Use one order-independent storage rewrite path

- Status: Accepted
- Date: 2026-08-22

### Context

Storage v1 readers accept document index keys in any order because index
position, rather than canonical sorting, associates each key with its value
line. Writers normalize every completed rewrite into canonical document-key
order.

A rewrite can optimize already-canonical input by merge-sorting the source and
patch, while non-canonical input must pass through an order-restoring buffer.
Maintaining both algorithms duplicates patch semantics, validation, and failure
handling for a performance optimization that is not required by the storage
contract.

### Decision

All storage rewrites use `ContainerDraft` as one order-independent boundary.
The rewrite supplies unchanged, replaced, and new documents in convenient input
order. `ContainerDraft` validates each key-payload pair, buffers documents whose
canonical position is not yet writable, and emits one canonical index and value
sequence.

Canonical and non-canonical v1 inputs follow the same rewrite algorithm.
Canonical ordering remains a writer normalization rule and does not require a
`storage_version` bump.

### Consequences

- Patch semantics and canonical output ordering have one implementation path.
- Reader compatibility with non-canonical v1 storage remains unchanged.
- Rewrites preserve unchanged document JSON after validating its identity.
- Pending memory can grow to the serialized size of documents waiting for an
  earlier canonical key, including much of the database when a new key sorts
  before existing input.
- Asset contents remain outside this cost because storage documents contain
  metadata while encrypted asset bytes are stored separately.
- If rewrite memory becomes a measured problem, optimization must preserve this
  single semantic boundary or supersede this ADR with evidence and an explicit
  complexity tradeoff.

### Alternatives considered

- Merge canonical input with a sorted patch: rejected because it creates a
  second rewrite algorithm with duplicated semantics and error handling.
- Bump `storage_version` and migrate all v1 files to require canonical input:
  rejected because ordering is non-semantic and does not justify migration,
  rollback, and synchronized-upgrade complexity.

<!--
## ADR-003: Short decision title

- Status: Proposed
- Date: YYYY-MM-DD

### Context

What problem, constraint, or decision boundary requires a durable choice?

### Decision

What is the chosen approach?

### Consequences

What becomes easier, harder, required, or intentionally excluded?

### Alternatives considered

- Alternative: reason it was not chosen.
-->
