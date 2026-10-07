# Architecture Decisions

This collection preserves significant decisions, their context, alternatives,
and consequences. [Living contracts](../README.md#living-contracts) own detailed
behavior; [proposals](../proposals/p2p-sync.md) develop candidate changes and
unresolved questions. Acceptance records a direction, not implementation
completion.

## Records

| Record | Status | Date |
| --- | --- | --- |
| [ADR-001: Asset content hashes](001-asset-content-hashes.md) | Accepted | 2026-08-07 |
| [ADR-002: Order-independent storage rewrites](002-order-independent-storage-rewrites.md) | Accepted | 2026-08-22 |
| [ADR-003: Linux CPU tuning](003-linux-cpu-tuning.md) | Accepted | 2026-09-21 |

## Conventions

- Add one file per decision as `NNN-short-title.md`, using the existing
  `ADR-001`, `ADR-002`, ... sequence. Never reuse identifiers or renumber records.
- Use the established statuses: `Proposed`, `Accepted`, `Superseded`, or
  `Deprecated`.
- Preserve accepted and superseded records, including their rationale.
  A superseding ADR links to the record it replaces; the old record links back.
- Include context, decision, consequences, and material alternatives with their
  tradeoffs. Record status and the decision date.
- Update the affected canonical specification when a decision changes a
  requirement. ADRs preserve decision history rather than replacing contracts.
- Add each new record to this index.