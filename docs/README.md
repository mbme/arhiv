# Documentation

Start with [architecture](architecture.md) for system ownership, the
[domain model](domain-model.md) for concepts and rules, and
[owner workflows](guides/user-workflows.md) for product outcomes.

## Living contracts

Specs define required behavior within their scope, not proof of implementation.
Known limitations describe observed gaps. Each detailed rule has one primary
home; other documents summarize and link to it.

| Contract | Owns |
| --- | --- |
| [Storage format](specs/storage-format.md) | Encrypted files, framing, container ordering, and serialized payloads. |
| [Schema](specs/schema.md) | Document types, fields, validation, and data-version meaning. |
| [Migration](specs/migration.md) | Version gates, supported migration, publication, and rollback guarantees. |
| [Backup and restore](specs/backup-restore.md) | Generations, manifests, verification, restore, and durability. |
| [Merge conflicts](specs/merge-conflicts.md) | Revision merging, synchronization effects, and conflict handling. |
| [Full-text search](specs/full-text-search.md) | Eligibility, matching, ranking, and persisted-index reuse. |
| [Key lifecycle](specs/key-lifecycle.md) | Key hierarchy, secret handling, unlock/lock, rotation, and recovery. |
| [Authentication](specs/authentication.md) | Local HTTPS, tokens, cookies, and session establishment. |
| [Platform security](specs/platform-security.md) | Desktop/Android trust controls, credential caches, and permissions. |
| [Runtime protocol](specs/runtime-protocol.md) | Startup payloads, discovery, locking, and shutdown. |
| [API and DTOs](specs/api.md) | Serialization, payload compatibility, validation responses, and transport errors. |

The [system threat model](system-threat-model.md) owns system-wide assets,
attackers, security goals, and residual risks; the specs own detailed controls.

## Guides and runbooks

- [Owner workflows](guides/user-workflows.md): open, edit, find, reconcile,
  back up, restore, and upgrade.
- [Storage upgrade](runbooks/storage-upgrade.md): upgrade preflight, required
  validation, failure handling, and rollback.
- [Contributing](../CONTRIBUTING.md): development, checks, builds, and releases.
- [Android](../arhiv-android/README.md): platform build, install, and debugging.
- [Agent guide](../AGENTS.md): agent instructions, commands, and safety boundaries.

Guides explain tasks and outcomes; runbooks add operational safety and recovery.
Keep development commands and agent instructions in their existing guides.
Specs own command contracts; guides and runbooks own task walkthroughs.

## Decisions and exploration

- [Architecture decisions](adr/README.md): significant choices,
  alternatives, and consequences, with status per decision.
- [P2P synchronization](proposals/p2p-sync.md): draft proposal; agreed constraints
  are separate from candidate mechanisms and unresolved rules.
- [Backlog](../BACKLOG.md): uncommitted ideas.
- Research: [lifestreams](research/modern-lifestream-research-report.md) and
  [personal information management](research/pim-beyond-lifestream-research-report.md).
  These are background, not contracts. Their session-specific citation markers
  need durable source verification before supporting a decision.
- [Solstone Journal case study](research/solstone-journal-case-study.md):
  capture-first personal memory compared with Arhiv, with ideas for agent retrieval and reviewed suggestions.

## Maintenance

- Architecture, domain, and workflow overviews describe current supported
  behavior. Mark planned changes and known gaps explicitly.
- When docs and implementation disagree, investigate the intended contract
  before changing either; preserve requirements rather than weakening them to
  match code.
- Proposals develop candidate changes. Acceptance is distinct from delivery;
  record significant choices and rationale in ADRs.
- After an approved change is delivered, reconcile affected living documents
  and links while retaining useful proposal and decision history.
- Extend an existing document before adding one. Split by cohesive capability,
  audience, or operational task, not by source module or development ticket.
