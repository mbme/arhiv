# ADR-002: Use one order-independent storage rewrite path

- Status: Accepted
- Date: 2026-08-22

## Context

Storage v1 readers accept document index keys in any order because index
position, rather than canonical sorting, associates each key with its value
line. Writers normalize every completed rewrite into canonical document-key
order.

A rewrite can optimize already-canonical input by merge-sorting the source and
patch, while non-canonical input must pass through an order-restoring buffer.
Maintaining both algorithms duplicates patch semantics, validation, and failure
handling for a performance optimization that is not required by the storage
contract.

## Decision

All storage rewrites use `ContainerDraft` as one order-independent boundary.
The rewrite supplies unchanged, replaced, and new documents in convenient input
order. `ContainerDraft` validates each key-payload pair, buffers documents whose
canonical position is not yet writable, and emits one canonical index and value
sequence.

Canonical and non-canonical v1 inputs follow the same rewrite algorithm.
Canonical ordering remains a writer normalization rule and does not require a
`storage_version` bump.

## Consequences

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

## Alternatives considered

- Merge canonical input with a sorted patch: rejected because it creates a
  second rewrite algorithm with duplicated semantics and error handling.
- Bump `storage_version` and migrate all v1 files to require canonical input:
  rejected because ordering is non-semantic and does not justify migration,
  rollback, and synchronized-upgrade complexity.