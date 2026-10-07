# Arhiv Documentation

Documentation is organized by product and system concern rather than by
source-code layout. Start with the domain model and workflows for product
behavior, then follow the relevant boundary contract.

## Understand Arhiv

| Document                                      | Purpose                                                                                        |
| --------------------------------------------- | ---------------------------------------------------------------------------------------------- |
| [Domain model](domain-model.md)               | Current domain vocabulary, relationships, invariants, and state transitions.                   |
| [User workflows](user-workflows.md)           | Owner-visible operations and outcomes across supported surfaces.                               |
| [System threat model](system-threat-model.md) | System-wide protected assets, attackers, trust boundaries, security goals, and residual risks. |

## Maintained contracts

These documents own detailed behavioral and compatibility rules. Operational
procedures and implementation notes provide context within their stated scope.

### Data and storage

| Document                                                    | Owns                                                                                             |
| ----------------------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| [Encrypted file format](arhiv-encrypted-file-format.md)     | Encrypted files, containers, ordering, and serialized payloads.                                  |
| [Storage schema contract](storage-schema-contract-spec.md)  | Document types, fields, validation, and data-version compatibility.                              |
| [Storage migration playbook](storage-migration-playbook.md) | Migration safety, supported upgrade behavior, file publication, and upgrade/rollback procedures. |
| [Backup and restore](backup-restore-durability-spec.md)     | Backup generations, manifests, verification, restore, and durability guarantees.                 |

### Synchronization and search

| Document                                     | Owns                                                                      |
| -------------------------------------------- | ------------------------------------------------------------------------- |
| [Merge conflicts](merge-conflicts-spec.md)   | Revision merging, synchronization effects, and conflict handling.         |
| [Full-text search](full-text-search-spec.md) | Search eligibility, matching, ranking, and persisted-index compatibility. |

### Security and key lifecycle

| Document                                                             | Owns                                                                             |
| -------------------------------------------------------------------- | -------------------------------------------------------------------------------- |
| [Crypto and key lifecycle](crypto-key-lifecycle-threat-model.md)     | Key hierarchy, lifecycle, secret handling, and recovery contract.                |
| [Authentication and sessions](auth-session-trust-chain-spec.md)      | Server authentication, cookies, and local HTTPS trust.                           |
| [Platform security boundaries](platform-security-boundaries-spec.md) | Desktop and Android trust controls, credential caches, and platform constraints. |

### Runtime and interfaces

| Document                                                                     | Owns                                                                                  |
| ---------------------------------------------------------------------------- | ------------------------------------------------------------------------------------- |
| [Launcher-server runtime protocol](launcher-server-runtime-protocol-spec.md) | Server startup, discovery, locking, startup payloads, and shutdown.                   |
| [API and DTO contract](api-dto-contract-spec.md)                             | Rust/TypeScript payload compatibility, serialization, transport, and error contracts. |

## Operate and develop

- [Contributing](../CONTRIBUTING.md): development setup, checks, builds,
  packaging, and releases.
- [Android guide](../arhiv-android/README.md): Android build, installation, and
  debugging instructions.
- [Storage migration playbook](storage-migration-playbook.md): upgrade
  preflight, validation, and rollback.
- [Backup and restore](backup-restore-durability-spec.md): backup/restore command
  contracts, recovery boundaries, and operational recommendations.
- [Agent guide](../AGENTS.md): repository commands, ownership boundaries, and
  agent-specific instructions.

## Decisions and exploration

| Document                                                                                      | Role                                                                                                    |
| --------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------- |
| [Architecture decisions](architecture-decisions.md)                                           | Significant decision history, alternatives, and consequences; each decision has its own status.         |
| [P2P sync and device trust](proposals/p2p-sync.md)                                            | Draft change proposal; distinguishes agreed constraints from candidate mechanisms and unresolved rules. |
| [Lifestream research](research/modern-lifestream-research-report.md)                          | Background research, not an Arhiv contract.                                                             |
| [Personal information management research](research/pim-beyond-lifestream-research-report.md) | Background research beyond lifestreams, not an Arhiv contract.                                          |
| [Backlog](../BACKLOG.md)                                                                      | Uncommitted ideas and possible improvements.                                                            |

The research reports retain session-specific citation markers rather than
durable source references. Their claims need source verification before they
serve as evidence for a proposal or decision.

## Authority and maintenance

- Domain and workflow documents describe current supported product behavior.
  Boundary contracts own detailed rules; summaries elsewhere link to that
  primary home.
- Specs define required behavior within their scope. Implementation notes and
  known limitations describe observed behavior; a spec alone is not proof that
  its requirements are implemented or verified.
- When documentation and implementation disagree, investigate the discrepancy
  and establish the intended contract before changing either. Reconcile them
  without silently weakening requirements to match code.
- Develop candidate changes in proposals, with explicit status and unresolved
  questions. Record significant decisions and their rationale in ADRs. An
  accepted direction is distinct from implementation completion.
- When an accepted change is implemented, update affected living documents to
  describe the supported result and retain useful proposal and decision history.
- Keep repository commands and agent instructions in their existing guides.
  Documents may summarize related behavior for context; detailed rules have one
  primary home.