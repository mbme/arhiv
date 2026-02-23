# Arhiv Cross-Layer API and DTO Contract Spec

Status: implementation-aligned (current behavior)

Scope: contract between Rust server DTOs and TypeScript UI DTOs for `/ui/api`, plus adjacent typed payloads (`window.CONFIG`, multipart upload result).

## 1. Canonical DTO Sources

Primary source of truth:
- Rust server DTO definitions: `arhiv/src/ui/dto.rs`
- API handler behavior: `arhiv/src/server/ui_server/api_handler.rs`

TypeScript mirror and client typing:
- TS DTO definitions: `arhiv/src/ui/dto.ts`
- RPC client: `arhiv/src/ui/utils/network.ts`
- global config typing: `arhiv/src/ui/global.d.ts`

Rule:
- Rust + TS DTOs must stay shape-compatible.
- Any request/response variant addition/removal/rename requires coordinated changes in both files.

## 2. Serialization and Naming Contract

Envelope and discriminants:
- API uses serde-tagged enums with discriminator field `typeName`.
- Requests and responses are discriminated unions by `typeName`.

Field naming:
- Rust uses snake_case fields internally and serializes API fields with `camelCase` (`#[serde(rename_all = "camelCase")]` where applied).
- TS side uses camelCase field names.

Strictness:
- `APIRequest` in Rust is `#[serde(deny_unknown_fields, tag = "typeName")]`.
- Unknown request fields or unknown request variants fail deserialization.

## 3. API Endpoint Contract (`/ui/api`)

Transport:
- Method: `POST`
- Content-Type: `application/json`
- Body: serialized `APIRequest`
- Success body: serialized `APIResponse`

UI endpoint construction:
- Client calls `${window.CONFIG.basePath}/api` (normally `/ui/api`).

Request/response variant symmetry:
- TS `APIRequest.typeName` and `APIResponse.typeName` variants mirror Rust enum variants 1:1 for normal operation.

## 4. Stable Request/Response Variants (Current)

Request variants:
- `ListDocuments`, `GetDocuments`, `GetStatus`, `GetDocument`, `ParseMarkup`
- `CreateDocument`, `SaveDocument`, `EraseDocument`
- `ListDir`, `CreateAsset`, `Commit`
- `LockDocument`, `UnlockDocument`, `ReorderCollectionRefs`
- `CreateArhiv`, `LockArhiv`, `UnlockArhiv`
- `ImportKey`, `ExportKey`, `CountConflicts`

Response variants:
- `ListDocuments`, `GetDocuments`, `GetStatus`, `GetDocument`, `ParseMarkup`
- `CreateDocument`, `SaveDocument`, `EraseDocument`
- `ListDir`, `CreateAsset`, `Commit`
- `LockDocument`, `UnlockDocument`, `ReorderCollectionRefs`
- `CreateArhiv`, `LockArhiv`, `UnlockArhiv`
- `ImportKey`, `ExportKey`, `CountConflicts`

## 5. Domain Type Mapping Notes

Notable mappings:
- `Id` <-> `DocumentId` (TS nominal string)
- `DocumentLockKey` <-> `DocumentLockKey` (TS nominal string)
- `Timestamp` serializes to string in JSON and is typed as string in TS.
- `Commit.committedIds` in TS maps from Rust `HashSet<Id>` serialized as JSON array (ordering not guaranteed).
- `DirEntry` is tagged union (`Dir` | `File` | `Symlink`) on both sides.

## 6. Validation and Business Errors

Create/save validation:
- `CreateDocument` and `SaveDocument` do not use HTTP error for validation failures.
- They return normal typed responses with `errors` payload:
  - `errors.documentErrors: string[]`
  - `errors.fieldErrors: Record<string, string[]>`

Implication:
- Validation is part of typed API success surface, not transport failure surface.

## 7. Transport/Error Surface Contract

Server-side API handler failures:
- Most request-processing failures become non-2xx responses via `ServerError`.
- Response body for such failures is plain text (`Something went wrong:\n...`), not typed JSON `APIResponse`.

Auth middleware failures (before API handler):
- Missing/invalid auth token returns HTTP `401`.
- Malformed auth token returns HTTP `400`.
- Body is plain text error.

Client-side RPC behavior:
- `doRPC` throws when `response.ok` is false, with `API call failed: <status>\n<body>`.
- `doRPC` parses JSON only for successful responses.

## 8. Adjacent Contracted Payloads

### 8.1 `window.CONFIG` contract

Server emits `window.CONFIG = <json>` from `/ui/config.js`.

Typed by:
- Rust: `ArhivUIConfig` in `arhiv/src/ui/dto.rs`
- TS: `ArhivUIConfig` in `arhiv/src/ui/dto.ts`
- Global binding: `window.CONFIG` in `arhiv/src/ui/global.d.ts`

### 8.2 Multipart upload result (`/ui/assets`)

Upload endpoint returns JSON `FileUploadResult`:
- `ids: DocumentId[]`
- `error?: string`

This payload is separate from `APIResponse` enum and is consumed by `uploadFile()`.

## 9. Versioning and Compatibility Policy (Current)

Current state:
- No explicit API version field in request/response envelopes.
- Compatibility is source-level: Rust and TS DTO definitions are updated together within the same repo revision.

Required change policy:
- Any contract change must update at least:
  - `arhiv/src/ui/dto.rs`
  - `arhiv/src/ui/dto.ts`
  - corresponding handler/client usage (`api_handler.rs`, UI call sites)

## 10. Known Risks

- TS type safety is compile-time only; runtime payload validation on UI side is minimal.
- `APIResponse` uses `deny_unknown_fields` on Rust serialize enum; accidental shape drift can surface as runtime parse/usage issues in clients.
- Non-2xx error bodies are unstructured text, limiting programmatic error branching.

## 11. Source of Truth (Code References)

- `arhiv/src/ui/dto.rs`
- `arhiv/src/ui/dto.ts`
- `arhiv/src/server/ui_server/api_handler.rs`
- `arhiv/src/server/ui_server/mod.rs`
- `arhiv/src/ui/utils/network.ts`
- `arhiv/src/ui/global.d.ts`
- `arhiv/src/server/ui_server/assets_handler.rs`
