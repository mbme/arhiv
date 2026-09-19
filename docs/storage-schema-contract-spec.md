# Arhiv Storage Schema Contract Specification

Current data version: `2`

## 1. Scope

This document defines the schema contract for document data stored in Arhiv.

It covers:

- schema composition and availability
- document type and field model
- runtime validation contract for staged writes
- compatibility and migration triggers for `data_version`

It does not define:

- low-level encrypted file/container format, covered in the
  [encrypted file format](arhiv-encrypted-file-format.md)
- merge algorithm semantics, covered in [merge conflicts](merge-conflicts-spec.md)
- migration execution details, covered in
  [storage migrations](storage-migration-playbook.md)

## 2. Schema Availability

Schema is compiled into the binary. There is no runtime user-defined schema loading.

## 3. Schema Model Contract

The schema contains an application name, a `data_version`, and a list of
document-type definitions. Each document type defines its machine name, title
format, and fields. Each field defines its name, type, and whether it is
mandatory or readonly.

## 4. Built-in and Reserved Document Types

The compiled schema always includes two reserved document types:

- erased document type (`_erased`)
- `asset` document type

Contract:

- these types are always present in runtime schema
- consumers must not assume only application-defined modules exist

The `asset` data contract is strict:

- `filename: string`
- `media_type: string`
- `size: u64`
- `content_sha256: string` (uppercase hex SHA-256 of plaintext asset bytes)
- `age_x25519_key: string` (secret material, serialized as string)

`content_sha256` is the SHA-256 digest of the decrypted/plaintext asset byte
stream, encoded as uppercase hexadecimal. It identifies asset content across
re-encryption, backup/restore, and device-local blob movement. Normal asset reads
do not verify this field; verification is an explicit workflow concern.

## 5. Field Type Contract (Current)

Supported `FieldType` variants:

- `String`
- `MarkupString`
- `Flag`
- `NaturalNumber`
- `Ref(document_types[])`
- `RefList(document_types[])`
- `Enum(options[])`
- `Date`
- `Duration`
- `People`
- `Countries`

Current validation-level JSON expectations:

- `String`/`MarkupString`/`Ref`/`Date`/`Duration`/`People`/`Countries`: JSON string (empty string accepted unless field is mandatory)
- `Flag`: JSON boolean
- `NaturalNumber`: JSON number representable as `u64`
- `RefList`: JSON `string[]`
- `Enum`: JSON string in allowed options (empty string accepted unless mandatory)

Notes:

- `Ref([])` and `RefList([])` mean any document type.
- For ref types with a non-empty allowed list, referenced document type must match one of the listed types.

## 6. Document Data Shape Contract

The top-level document envelope is strict:

- `id`
- `rev`
- `document_type`
- `updated_at`
- `data`

`data` is a dynamic JSON object (`DocumentData`).

Staging-time field presence rule:

- unknown non-null fields in `data` are rejected:
  - `"Document type '<type>' doesn't expect field '<field>'"`
- fields explicitly set to JSON `null` are treated as absent by validation accessors

## 7. Staging Validation Contract

When staging, the system enforces:

1. document-level invariants:

- erased documents cannot be staged
- for edits to an existing document, `document_type` and `updated_at` must match
  the previous staged version or current snapshot

2. field-level schema checks:

- mandatory/readonly/type/enum constraints

3. reference checks:

- referenced IDs must exist
- referenced documents must be active; erased documents cannot receive new
  references, collection memberships, or asset references
- referenced document type must satisfy ref type constraints when specified

Error model:

- field-scoped failures are grouped by field
- document-scoped failures are grouped separately

API mapping:

- create/save validation is returned in typed response payloads (`errors.documentErrors`, `errors.fieldErrors`) rather than transport-level failures

## 8. Readonly Field Contract

If a field is marked `readonly`, changing its value relative to previous document state is rejected at staging time.

Current behavior:

- readonly is enforced by runtime validation, not by storage encoding
- readonly comparison is value-based on serialized JSON values

## 9. Title/Cover/Search/Ref-Derivation Semantics

Schema fields are also used by higher-level derivation logic:

- title rendering from `title_format` and fields
- cover inference via field named `cover` with `Ref([asset])`
- search extraction from selected text-like field types
- reference/backreference extraction from `MarkupString`, `Ref`, and `RefList`

Contract implication:

- changing field type/name can affect search, title rendering, refs graph, and UI behavior even if raw storage remains parseable

## 10. Data Version Compatibility Contract

Runtime gate:

- registered safe data migrations run after unlock and before normal state loading
- after migration, the stored `data_version` must equal the schema's latest version
- unsupported versions or blocked migrations prevent normal opening and reading

Current state:

- latest `data_version` is `2`
- data version `1` is upgraded by the asset-content-hash migrator described in
  [storage migrations](storage-migration-playbook.md)
- there is no negotiated multi-version schema compatibility at runtime

## 11. Known limits

- schema is static (compile-time), not user-extensible at runtime
- `Date`, `Duration`, `People`, `Countries` currently validate as strings; domain-format semantics are not centrally enforced in schema layer
- UI/API compatibility still depends on coordinated client + server upgrades in one repo revision