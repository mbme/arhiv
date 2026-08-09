# Arhiv Domain Model

Status: intended durable domain model

## Purpose and scope

Arhiv is a private, local-first application for one person's structured knowledge and files. The person is the implicit owner and is not a domain object.

This document defines enduring domain concepts, relationships, and rules. It governs the domain when current behavior differs. User interfaces, storage, encryption, network protocols, and implementation mechanisms are outside its scope.

## Domain objects

### Arhiv

An **Arhiv** is one person's complete personal database. It is the boundary for its records, relationships, history, changes, and conflicts.

### Record

A **Record** is an identifiable item of knowledge in one Arhiv. It has one unchanging identity. An active Record has one Record kind; deletion replaces its current form with an erased form.

A record may have details, references, collection memberships, and attachments as permitted by its kind.

### Record kind

A **Record kind** defines a family of active records. It defines:

- required and optional details;
- allowed values for constrained details;
- allowed target kinds for references;
- attachment roles; and
- for a Collection kind, accepted member kinds.

A kind also defines how a person meaningfully identifies a record.

### Record form and history

A **Record form** is a complete state of one Record. A committed form is either current or historical. A record's **history** contains its retained committed forms.

An active record has a current form that represents it as available for new relationships. A deleted record has an erased current form that preserves its identity but replaces its active kind and details. Deletion may prune superseded committed forms.

### Reference

A **Reference** is a meaningful link from one Record to another Record in the same Arhiv. References are part of a Record form.

### Collection

A **Collection** is a specialized Record that names and orders compatible Records. A collection does not own its members: membership does not create, delete, or transfer a member record.

A record may belong to multiple collections. A member occurs at most once in a collection.

### Attachment

An **Attachment** is a specialized Record whose content is a file held by Arhiv. An attachment may serve a kind-defined role for another Record.

An Attachment's record metadata, relationships, and history remain part of Arhiv even when its file content is unavailable. Unavailable attachment content is loss of that attachment's content, not corruption of Arhiv records, relationships, or history. Reading unavailable attachment content fails for that content, while unrelated records and metadata remain usable.

### Pending change

A **Pending change** is a durable proposed Record form that has not been committed. It is either a proposed creation, an ordinary change based on one committed form of an existing Record, or a reconciled change based on every competing committed form of one Conflict. It may be discarded.

All Pending changes together form one implicit pending change set. It is not a separate domain object.

### Commit

A **Commit** is the durable event that makes all Pending changes part of Arhiv history. A commit is atomic: all pending forms become current together, or none do.

### Conflict

A **Conflict** is a record-scoped state with two or more concurrent committed forms of the same Record. Concurrent committed forms always create a Conflict; none is silently committed as the sole outcome.

A heuristic reconciled Pending change may be prepared automatically. Committing a reconciled change supersedes the competing forms, creates one current form, and resolves the Conflict.

## Relationships

- Every Record belongs to exactly one Arhiv and exactly one Record kind.
- A Record form may reference zero or more Records in the same Arhiv.
- A Collection form may contain zero or more compatible Records in a meaningful order.
- An Attachment may serve zero or more Records in kind-defined roles.
- A Pending change concerns one Record, including a proposed new Record.
- One Commit makes all Pending changes permanent.
- A Conflict concerns one Record and its competing committed forms.

## Domain invariants

1. A Record keeps its identity throughout its life. Every active committed or pending form has the same Record kind. A deleted Record has an erased current form.
2. A committed active form satisfies the required details and constrained values of its kind. A Pending change must satisfy them before Commit.
3. A relationship stays within one Arhiv. A new relationship may target only an existing active Record; it may not target a proposed or deleted Record. References, collection memberships, and attachment roles must be permitted by their source Record kind.
4. A Collection may contain only Records of kinds accepted by its Collection kind. It preserves member order and contains each member at most once.
5. A Pending change may be discarded only before Commit. Discarding all Pending changes leaves no pending change set.
6. Commit is atomic. All pending forms become current together and their preceding current forms become historical together, except that deletion may prune superseded forms.
7. A Record without a Conflict has exactly one current committed form. A Record with a Conflict has two or more competing current committed forms.
8. A deletion is an erased current committed form. It preserves the Record's identity, may prune superseded forms, and prevents new incoming references, collection memberships, and attachment relationships.
9. Retained historical relationships to a deleted Record may remain in history, but must identify that Record as deleted.
10. A Conflict remains until a reconciled Pending change is committed. That commit creates one current form and resolves the Conflict.

## Record and pending-change transition model

| From                      | Action                             | To                                             |
| ------------------------- | ---------------------------------- | ---------------------------------------------- |
| Absent                    | Prepare creation                   | Proposed new record                            |
| Proposed new record       | Commit pending changes             | Active record                                  |
| Proposed new record       | Discard                            | Absent                                         |
| Active record             | Prepare edit                       | Pending edit                                   |
| Pending edit              | Commit pending changes             | Active record with a new current form          |
| Pending edit              | Discard                            | Previous active record                         |
| Active record             | Prepare deletion                   | Pending deletion                               |
| Pending deletion          | Commit pending changes             | Deleted record                                 |
| Pending deletion          | Discard                            | Previous active record                         |
| Active or deleted record  | Receive concurrent committed forms | Conflict                                       |
| Conflict                  | Prepare reconciliation             | Pending reconciled change                      |
| Pending reconciled change | Commit pending changes             | Active or deleted record with one current form |
| Pending reconciled change | Discard                            | Conflict                                       |

A Conflict may exist alongside a prepared reconciled change. It remains unresolved until that change is committed.

## Outside this model

The following are outside this domain model:

- the Arhiv owner;
- search and presentation;
- user sessions and access control;
- encryption and key recovery;
- storage and backup mechanisms; and
- devices and synchronization tools.