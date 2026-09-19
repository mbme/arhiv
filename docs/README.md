# Arhiv Documentation

These documents explain how Arhiv behaves in the current repository. They are
organized by product and system concern rather than by source-code layout.
Uncommitted ideas and possible improvements belong in the
[backlog](../BACKLOG.md). Active proposals belong in proposed
architecture decisions, not in the topic documents.

When documentation and implementation disagree, treat that as an inconsistency
to investigate. Update the explanation or the implementation so they describe
the same behavior.

The domain model and user workflows are the best starting points for product
behavior. The remaining documents explain specific storage, security, runtime,
and interface boundaries.

## Topic guide

| Topic                                                            | Document                                                                     |
| ---------------------------------------------------------------- | ---------------------------------------------------------------------------- |
| Domain concepts, relationships, and state transitions            | [Domain model](domain-model.md)                                              |
| Owner-facing operations and outcomes                             | [User workflows](user-workflows.md)                                          |
| Search eligibility, matching, and ranking                        | [Full-text search](full-text-search-spec.md)                                 |
| Encrypted files, containers, and serialized payloads             | [Encrypted file format](arhiv-encrypted-file-format.md)                      |
| Document types, fields, validation, and data versions            | [Storage schema contract](storage-schema-contract-spec.md)                   |
| Storage upgrades, file publication, validation, and rollback     | [Storage migration playbook](storage-migration-playbook.md)                  |
| Backup generations, verification, and restore                    | [Backup and restore](backup-restore-durability-spec.md)                      |
| Revision merging and conflict handling                           | [Merge conflicts](merge-conflicts-spec.md)                                   |
| Protected information, attackers, guarantees, and residual risks | [System threat model](system-threat-model.md)                                |
| Encryption keys, cached credentials, and recovery                | [Crypto and key lifecycle](crypto-key-lifecycle-threat-model.md)             |
| Server authentication, cookies, and local HTTPS trust            | [Authentication and sessions](auth-session-trust-chain-spec.md)              |
| Desktop and Android trust boundaries                             | [Platform security boundaries](platform-security-boundaries-spec.md)         |
| Server startup, discovery, locking, and shutdown                 | [Launcher-server runtime protocol](launcher-server-runtime-protocol-spec.md) |
| Rust/TypeScript request and response shapes                      | [API and DTO contract](api-dto-contract-spec.md)                             |
| Context and rationale behind significant choices                 | [Architecture decisions](architecture-decisions.md)                          |

Documents may briefly summarize related behavior for context, but detailed
rules should have one primary home and link to it from elsewhere.