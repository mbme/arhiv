# Arhiv Domain Model

Status: intended durable domain model

## Purpose and scope

Arhiv is a private, local-first archive for one person's structured knowledge and files. The person is the implicit owner and is not a domain object.

This document defines enduring domain concepts, relationships, and rules. It governs the domain when current behavior differs. User interfaces, storage, encryption, network protocols, and implementation mechanisms are outside its scope.

## Domain objects

### Archive

An **Archive** is one person's complete personal database. It is the boundary for its records, relationships, history, changes, and conflicts.

### Record

A **Record** is an identifiable item of knowledge in one Archive. It has one unchanging identity and one unchanging Record kind.

A record may have details, references, collection memberships, and attachments as permitted by its kind.

### Record kind

A **Record kind** defines a family of records. It defines:

- required and optional details;
- allowed values for constrained details;
- allowed target kinds for references;
- attachment roles; and
- for a Collection kind, accepted member kinds.

A kind also defines how a person meaningfully identifies a record.

### Record form and history

A **Record form** is a complete state of one Record. A committed form is either current or historical. A record's **history** retains all of its committed forms.

An active record has a current form that represents it as available for new relationships. A deleted record has a current deletion form that preserves its identity and history but is unavailable for new relationships.

### Reference

A **Reference** is a meaningful link from one Record to another Record in the same Archive. References are part of a Record form.

### Collection

A **Collection** is a specialized Record that names and orders compatible Records. A collection does not own its members: membership does not create, delete, or transfer a member record.

A record may belong to multiple collections. A member occurs at most once in a collection.

### Attachment

An **Attachment** is a specialized Record whose content is a file held by the Archive. An attachment may serve a kind-defined role for another Record.

### Pending change

A **Pending change** is a durable, user-visible proposed Record form that has not been committed. It is either a proposed creation, an ordinary change based on one committed form of an existing Record, or a reconciled change based on every competing committed form of one Conflict. It may be discarded.

### Change set

A **Change set** is a durable, user-visible group of proposed Record forms. It is open, committed, or discarded. An open Change set contains one or more Pending changes. A committed Change set retains its Commit and resulting committed forms.

### Commit

A **Commit** is the durable event that makes one Change set part of Archive history. A commit is atomic: all of its proposed forms become current together, or none do.

### Conflict

A **Conflict** is a record-scoped state with two or more concurrent committed forms of the same Record. Concurrent committed forms always create a Conflict; none is silently selected as the sole outcome.

Committing a reconciled change supersedes the competing forms, creates one current form, and resolves the Conflict.

## Relationships

- Every Record belongs to exactly one Archive and exactly one Record kind.
- A Record form may reference zero or more Records in the same Archive.
- A Collection form may contain zero or more compatible Records in a meaningful order.
- An Attachment may serve zero or more Records in kind-defined roles.
- A Pending change concerns one Record, including a proposed new Record, and belongs to one open Change set.
- A Change set is made permanent by one Commit.
- A Conflict concerns one Record and its competing committed forms.

## Domain invariants

1. A Record keeps its identity and Record kind throughout its life. Every committed or pending form of that Record has the same identity and kind.
2. A committed form satisfies the required details and constrained values of its kind. A Pending change must satisfy them before its Change set is committed.
3. A relationship stays within one Archive. A new relationship may target an active Record or a proposed Record in the same Change set; it may not target a deleted Record. References, collection memberships, and attachment roles must be permitted by their source Record kind.
4. A Collection may contain only Records of kinds accepted by its Collection kind. It preserves member order and contains each member at most once.
5. An open Change set contains one or more Pending changes. A Pending change belongs to one open Change set and may be discarded only while that Change set is open. Discarding its last Pending change discards the Change set.
6. A Change set is committed atomically. Its proposed forms become current together and its preceding current forms become historical together.
7. A Record without a Conflict has exactly one current committed form. A Record with a Conflict has two or more competing current committed forms.
8. A deletion is a current committed form. It preserves the Record's identity and history and prevents new incoming references, collection memberships, and attachment relationships.
9. Historical relationships to a deleted Record may remain in history, but must identify that Record as deleted.
10. A Conflict remains until a Change set containing a reconciled change is committed. That commit creates one current form and resolves the Conflict.

## Record lifecycle

| From | Action | To |
| --- | --- | --- |
| Absent | Prepare creation | Proposed new record |
| Proposed new record | Commit Change set | Active record |
| Proposed new record | Discard | Absent |
| Active record | Prepare edit | Pending edit |
| Pending edit | Commit Change set | Active record with a new current form |
| Pending edit | Discard | Previous active record |
| Active record | Prepare deletion | Pending deletion |
| Pending deletion | Commit Change set | Deleted record |
| Pending deletion | Discard | Previous active record |
| Active or deleted record | Receive concurrent committed forms | Conflict |
| Conflict | Prepare reconciliation | Pending reconciled change |
| Pending reconciled change | Commit Change set | Active or deleted record with one current form |
| Pending reconciled change | Discard | Conflict |

A Conflict may exist alongside a prepared reconciled change. It remains unresolved until that change is committed.

## Outside this model

The following are outside this domain model:

- the archive owner;
- search and presentation;
- user sessions and access control;
- encryption and key recovery;
- storage and backup mechanisms; and
- devices and synchronization tools.
