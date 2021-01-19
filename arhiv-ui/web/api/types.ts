import { EmptyObject, Obj } from '@v/utils'

export interface IDocument {
  readonly id: string
  readonly rev: number
  readonly documentType: string
  readonly createdAt: string
  readonly updatedAt: string
  readonly refs: readonly string[]
  readonly archived: boolean
  readonly data: IDocumentData
}

export interface IDocumentData {
  readonly [name: string]: any
}

export interface IAttachmentSource {
  readonly id: string
  readonly filePath: string
  readonly filename: string
  readonly copy: boolean
}

export type Matcher =
  { Field: { selector: string, pattern: string } }
  | { FuzzyField: { selector: string, pattern: string } }
  | { Type: { documentType: string } }

export type OrderBy =
  { Field: { selector: string, asc: boolean } }
  | { EnumField: { selector: string, asc: boolean, enumOrder: string[] } }
  | { UpdatedAt: { asc: boolean } }

export type FilterMode = 'Staged' | 'Archived'

export interface IFilter {
  pageOffset?: number
  pageSize?: number
  matchers: Matcher[]
  order: OrderBy[],
  mode?: FilterMode,
}

export type AttachmentLocation = { Url: string } | { File: string }

export interface IListPage<T> {
  readonly items: T[]
  readonly hasMore: boolean
}

export interface IDataSchema {
  readonly version: number
  readonly modules: IDataDescription[]
}

export interface IDataDescription {
  readonly documentType: string
  readonly collectionOf?: ICollection
  readonly fields: IField[]
}

export interface ICollection {
  readonly itemType: string
}

export interface IField {
  readonly name: string
  readonly fieldType: FieldType
}

export type FieldType =
  { String: EmptyObject }
  | { MarkupString: EmptyObject }
  | { Ref: string }
  | { Enum: string[] }

export interface ICreateDocumentArgs {
  readonly documentType: string
  readonly args: Obj
}

export interface IPutDocumentArgs {
  readonly document: IDocument
  readonly newAttachments: IAttachmentSource[]
}

export interface IRenderMarkupArgs {
  readonly value: string
  readonly options: {
    readonly newAttachments: IAttachmentSource[]
    readonly documentPath: string
  }
}

export interface IStatus {
  readonly db_status: {
    readonly arhiv_id: string,
    readonly is_prime: boolean,
    readonly schema_version: number,
    readonly db_rev: number,
    readonly last_sync_time: number,
  },
  readonly root_dir: string,
  readonly last_update_time: string,
  readonly debug_mode: boolean,
  readonly committed_documents: number,
  readonly staged_documents: number,
}
