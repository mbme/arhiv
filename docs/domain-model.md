# Arhiv Domain Model

## Purpose and scope

Arhiv is a private, local-first application for one person's structured
knowledge and files. The person is the implicit owner and is not a domain
object.

This document explains the current domain concepts, relationships, and rules.
User interfaces, storage, encryption, network protocols, and implementation
mechanisms are outside its scope.

## Domain objects

### Arhiv

An **Arhiv** is one person's complete personal database. It is the boundary for
its documents, relationships, history, changes, and conflicts.

### Document

A **document** is an identifiable item of knowledge in one Arhiv. It keeps one
identity throughout its life. An active document has one document type;
erasure replaces its current version with an erased version.

A document may have fields, references, collection memberships, and asset
references as permitted by its type.

### Document type

A **document type** defines a family of active documents. It defines:

- required and optional fields;
- allowed values for constrained fields;
- allowed target types for references;
- asset-reference fields; and
- for a collection type, accepted member types.

A document type also defines how the owner meaningfully identifies a document.

### Document version, snapshot, and history

A **document version** is a complete state of one document. A **snapshot** is a
committed document version. A document's **history** contains its retained
snapshots.

An active document has one current snapshot when it has no conflict. An erased
document has an erased current snapshot that preserves its identity but
replaces its active type and fields. Erasure may prune superseded snapshots.

### Reference

A **reference** is a meaningful link from one document to another document in
the same Arhiv. References are part of a document version.

### Collection

A **collection** is a specialized document that names and orders compatible
documents. A collection does not own its members: membership does not create,
erase, or transfer a member document.

A document may belong to multiple collections. A member occurs at most once in
a collection.

### Asset and blob

An **asset** is a specialized document that describes a file held by Arhiv. Its
**blob** is the separately stored encrypted file content. An asset may be
referenced by zero or more documents in type-defined roles.

An asset's metadata, relationships, and history remain part of Arhiv when its
blob is unavailable. The missing blob is loss of that asset's content, not
corruption of unrelated documents or history. Reading the unavailable content
fails while the rest of Arhiv remains usable.

### Staged change

A **staged change** is a durable document version that has not been committed.
It is a staged creation, edit, erasure, or merge and may be discarded before
commit.

All staged changes form one implicit staging set. The staging set is not a
separate domain object.

### Commit

A **commit** is the durable event that makes every staged change part of Arhiv
history. A commit is atomic: all staged versions become current snapshots
together, or none do.

### Conflict and staged merge

A **conflict** is a document state with two or more concurrent current snapshots.
Concurrent snapshots always create a conflict; none is silently committed as
the sole outcome.

A conflict may have a **staged merge**: a staged document version based on every
competing snapshot. Arhiv may prepare one heuristically, and the owner may
inspect or edit it. The conflict remains until the staged merge is committed.
Committing it supersedes the competing snapshots, creates one current snapshot,
and resolves the conflict.

## Relationships

- Every document belongs to exactly one Arhiv and has exactly one document type
  while active.
- A document version may reference zero or more documents in the same Arhiv.
- A collection version may contain zero or more compatible documents in a
  meaningful order.
- An asset may be referenced by zero or more documents in type-defined roles.
- A staged change concerns one document, including a proposed new document.
- One commit makes all staged changes permanent.
- A conflict concerns one document and its competing snapshots.

## Domain invariants

1. A document keeps its identity throughout its life. Every active snapshot or
   staged change has the same document type. An erased document has an erased
   current snapshot.
2. A committed active snapshot satisfies the required fields and constrained
   values of its document type. A staged change must satisfy them before commit.
3. A relationship stays within one Arhiv. A new relationship may target only an
   existing active document; it may not target a staged creation or erased
   document. References, collection memberships, and asset references must be
   permitted by their source document type.
4. A collection may contain only documents of types accepted by its collection
   type. It preserves member order and contains each member at most once.
5. A staged change may be discarded only before commit. Discarding all staged
   changes leaves no staging set.
6. A commit is atomic. All staged versions become current snapshots together
   and their preceding current snapshots become historical together, except
   that erasure may prune superseded snapshots.
7. A document without a conflict has exactly one current snapshot. A document
   with a conflict has two or more competing current snapshots.
8. Erasure creates an erased current snapshot. It preserves the document's
   identity, may prune superseded snapshots, and prevents new incoming
   references, collection memberships, and asset references.
9. Retained historical relationships to an erased document may remain in
   history, but must identify that document as erased.
10. A conflict remains until its staged merge is committed. That commit creates
    one current snapshot and resolves the conflict.

## Document and staging transition model

| From                       | Action                       | To                                                  |
| -------------------------- | ---------------------------- | --------------------------------------------------- |
| Absent                     | Stage creation               | Staged creation                                     |
| Staged creation            | Commit                       | Active document                                     |
| Staged creation            | Discard                      | Absent                                              |
| Active document            | Stage edit                   | Staged edit                                         |
| Staged edit                | Commit                       | Active document with a new current snapshot         |
| Staged edit                | Discard                      | Previous active document                            |
| Active document            | Stage erasure                | Staged erasure                                      |
| Staged erasure             | Commit                       | Erased document                                     |
| Staged erasure             | Discard                      | Previous active document                            |
| Active or erased document  | Receive concurrent snapshots | Conflict                                            |
| Conflict                   | Prepare merge                | Conflict with staged merge                          |
| Conflict with staged merge | Commit                       | Active or erased document with one current snapshot |
| Conflict with staged merge | Discard                      | Conflict                                            |

A conflict may exist alongside a staged merge. It remains unresolved until that
merge is committed.

## Outside this model

The following are outside this domain model:

- the Arhiv owner;
- search and presentation;
- user sessions and access control;
- encryption and key recovery;
- storage and backup mechanisms; and
- devices and synchronization tools.