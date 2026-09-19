# Arhiv Cross-Layer API and DTO Contract Spec

Scope: contract between Rust server DTOs and TypeScript UI DTOs for `/ui/api`, plus adjacent typed payloads (`window.CONFIG`, multipart upload result).

## 1. DTO Compatibility

The Rust server DTOs and TypeScript UI DTOs are mirrored definitions that ship
together. They must remain shape-compatible. Any request or response variant
addition, removal, or rename requires a coordinated server and UI change.

## 2. Serialization and Naming Contract

Envelope and discriminants:

- API payloads use the discriminator field `typeName`.
- Requests and responses are discriminated unions by `typeName`.

Field naming:

- Transmitted API fields use `camelCase`.

Strictness:

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

Requests and responses use the same variant set:

- `ListDocuments`, `GetDocuments`, `GetStatus`, `GetDocument`, `ParseMarkup`
- `CreateDocument`, `SaveDocument`, `EraseDocument`
- `ListDir`, `CreateAsset`, `Commit`
- `LockDocument`, `UnlockDocument`, `ReorderCollectionRefs`
- `CreateArhiv`, `LockArhiv`, `UnlockArhiv`
- `ImportKey`, `ExportKey`, `CountConflicts`

`UnlockArhiv` response:

- `outcome: "unlocked" | "needsPassword"`
- `needsPassword` is a normal response for a missing, unavailable, malformed, or non-matching
  cached device key. The UI must show password/import recovery without treating it as an HTTP error.

## 5. Domain Type Mapping Notes

Notable mappings:

- `Id` <-> `DocumentId` (TS nominal string)
- `DocumentLockKey` <-> `DocumentLockKey` (TS nominal string)
- `Timestamp` serializes to string in JSON and is typed as string in TS.
- `Commit.committedIds` is a JSON array whose ordering is not guaranteed.
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

- Most request-processing failures become non-2xx responses.
- Response body for such failures is plain text (`Something went wrong:\n...`), not typed JSON `APIResponse`.

Auth middleware failures (before API handler):

- Missing/invalid auth token returns HTTP `401`.
- Malformed auth token returns HTTP `400`.
- Body is plain text error.

Client-side RPC behavior:

- The client throws when `response.ok` is false, with
  `API call failed: <status>\n<body>`.
- The client parses JSON only for successful responses.

## 8. Adjacent Contracted Payloads

### 8.1 `window.CONFIG` contract

Server emits `window.CONFIG = <json>` from `/ui/config.js`.

The server and UI share the `ArhivUIConfig` payload shape. The UI exposes the
payload through the global `window.CONFIG` binding.

### 8.2 Multipart upload result (`/ui/assets`)

Upload endpoint returns JSON `FileUploadResult`:

- `ids: DocumentId[]`
- `error?: string`

This payload is separate from the `APIResponse` union.

## 9. Versioning and compatibility

- No explicit API version field in request/response envelopes.
- Compatibility is release-level: Rust DTOs, TypeScript DTOs, handlers, and UI
  callers ship together in one repository revision.
- An automated synchronization check compares request and response variant
  names. Field-shape compatibility still relies on the Rust and TypeScript
  definitions being updated together.

## 10. Known Risks

- TS type safety is compile-time only; runtime payload validation on UI side is minimal.
- Accidental response-shape drift can surface as runtime parse or usage failures
  in clients.
- Non-2xx error bodies are unstructured text, limiting programmatic error branching.