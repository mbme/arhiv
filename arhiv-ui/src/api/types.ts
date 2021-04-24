import { EmptyObject, Obj } from '@v/utils'

export interface IDocument {
  readonly id: string
  readonly rev: number
  readonly prevRev: number
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

export type Matcher =
  { Field: { selector: string, pattern: string } }
  | { Search: { pattern: string } }
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
  order: OrderBy[]
  mode?: FilterMode
}

export interface IDocumentExt {
  document: IDocument
  preview: string
}

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
}
