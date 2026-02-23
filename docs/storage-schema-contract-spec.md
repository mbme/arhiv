# Arhiv Storage Schema Contract Specification

Status: implementation-aligned (current behavior)  
Current data version: `1`

## 1. Scope

This document defines the schema contract for document data stored in Arhiv.

It covers:
- canonical schema sources
- document type and field model
- runtime validation contract for staged writes
- compatibility and migration triggers for `data_version`

It does not define:
- low-level encrypted file/container format (covered in `docs/arhiv-encrypted-file-format.md`)
- merge algorithm semantics (covered in `docs/merge-conflicts-spec.md`)
- migration execution playbook details (covered in `docs/storage-migration-playbook.md`)

## 2. Canonical Schema Sources

Primary runtime schema:
- `arhiv/src/definitions/mod.rs` (`get_standard_schema`)
- `arhiv/src/definitions/*.rs` (document-type definitions)
- `baza/src/schema/mod.rs` (`DataSchema`)
- `baza/src/schema/data_description.rs` (`DataDescription`)
- `baza/src/schema/field.rs` (`Field`, `FieldType`)
- `baza/src/schema/asset.rs` (built-in `asset` type)

Schema is compiled into the binary. There is no runtime user-defined schema loading.

## 3. Schema Model Contract

`DataSchema` contains:
- app name (`name`)
- `data_version: u8`
- list of `DataDescription` modules

Each `DataDescription` contains:
- `document_type: &'static str`
- `title_format: &'static str`
- `fields: Vec<Field>`

Each `Field` contains:
- `name: &'static str`
- `field_type: FieldType`
- `mandatory: bool`
- `readonly: bool`

## 4. Built-in and Reserved Document Types

`DataSchema::new` appends two definitions automatically:
- erased document type (`_erased`)
- `asset` document type

Contract:
- these types are always present in runtime schema
- consumers must not assume only application-defined modules exist

`asset` data contract is concrete and strict (`AssetData` uses `#[serde(deny_unknown_fields)]`):
- `filename: string`
- `media_type: string`
- `size: u64`
- `age_x25519_key: string` (secret material, serialized as string)

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

Top-level document envelope is strict (`Document` uses `#[serde(deny_unknown_fields)]`):
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

When staging (`Baza::validate_staged`), the system enforces:
1. document-level invariants:
- erased docs cannot be staged
- for edits to an existing doc, `document_type` and `updated_at` must match the previous staged/current document snapshot
2. field-level schema checks:
- mandatory/readonly/type/enum constraints
3. reference checks:
- referenced IDs must exist
- referenced document type must satisfy ref type constraints when specified

Error model:
- field-scoped failures aggregate as `ValidationError::FieldError { field -> [errors] }`
- document-scoped failures aggregate as `ValidationError::DocumentError { errors }`

API mapping:
- create/save validation is returned in typed response payloads (`errors.documentErrors`, `errors.fieldErrors`) rather than transport-level failures

## 8. Readonly Field Contract

If a field is marked `readonly`, changing its value relative to previous document state is rejected at staging time.

Current behavior:
- readonly is enforced by runtime validation, not by storage encoding
- readonly comparison is value-based on serialized JSON values

## 9. Title/Cover/Search/Ref-Derivation Semantics

Schema fields are also used by higher-level derivation logic:
- title rendering from `title_format` and fields (`DocumentExpert`)
- cover inference via field named `cover` with `Ref([asset])`
- search extraction from selected text-like field types
- reference/backreference extraction from `MarkupString`, `Ref`, and `RefList`

Contract implication:
- changing field type/name can affect search, title rendering, refs graph, and UI behavior even if raw storage remains parseable

## 10. Data Version Compatibility Contract

Runtime gate:
- `state.info.data_version` must equal `schema.get_latest_data_version()`
- mismatch fails open/read with data-version mismatch

Current state:
- latest `data_version` is hardcoded in `DataSchema` (`1`)
- there is no negotiated multi-version schema compatibility at runtime

Operational policy:
- changing schema semantics that can invalidate existing stored docs requires coordinated `data_version` bump and migration plan

## 11. Migration Triggers for Schema Changes

A `data_version` bump is required when any of the following occur:
- field rename/remove/type change for persisted documents
- new mandatory field without deterministic default/backfill
- enum option changes that invalidate existing values
- document type rename/remove
- changed interpretation of existing field values

A bump is usually not required for:
- adding optional field with backward-compatible handling
- adding new document type that does not alter existing-type semantics

When in doubt, treat schema-affecting changes as migration-affecting and follow `docs/storage-migration-playbook.md`.

## 12. Known Limits (Current)

- schema is static (compile-time), not user-extensible at runtime
- `Date`, `Duration`, `People`, `Countries` currently validate as strings; domain-format semantics are not centrally enforced in schema layer
- UI/API compatibility still depends on coordinated client + server upgrades in one repo revision

## 13. Source of Truth (Code References)

- `baza/src/schema/mod.rs`
- `baza/src/schema/data_description.rs`
- `baza/src/schema/field.rs`
- `baza/src/schema/asset.rs`
- `baza/src/baza/validator.rs`
- `baza/src/entities/document.rs`
- `baza/src/entities/document_data.rs`
- `baza/src/baza/mod.rs`
- `arhiv/src/definitions/mod.rs`
- `arhiv/src/definitions/book.rs`
- `arhiv/src/definitions/contact.rs`
- `arhiv/src/definitions/film.rs`
- `arhiv/src/definitions/game.rs`
- `arhiv/src/definitions/note.rs`
- `arhiv/src/definitions/tag.rs`
- `arhiv/src/definitions/task.rs`
- `arhiv/src/definitions/track.rs`
- `arhiv/src/ui/dto.rs`
