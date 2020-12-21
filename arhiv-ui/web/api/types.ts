import { EmptyObject, Obj } from '@v/utils'

export interface IDocument {
  readonly id: string
  readonly rev: number
  readonly createdAt: string
  readonly updatedAt: string
  readonly refs: readonly string[]
  readonly archived: boolean
  readonly data: IDocumentData
}

export interface IDocumentData {
  readonly type: string
  readonly [name: string]: any
}

export interface IAttachmentSource {
  readonly id: string
  readonly filePath: string
  readonly filename: string
  readonly copy: boolean
}

export interface IAttachment {
  readonly id: string
  readonly rev: number
  readonly createdAt: string
  readonly filename: string
}

export interface IMatcher {
  selector: string
  pattern: string
  fuzzy: boolean
}

export type OrderBy =
  { Field: { selector: string, asc: boolean } }
  | { EnumField: { selector: string, asc: boolean, enumOrder: string[] } }
  | { UpdatedAt: { asc: boolean } }

export interface IDocumentFilter {
  pageOffset?: number
  pageSize?: number
  matchers: IMatcher[]
  archived?: boolean
  onlyStaged?: boolean
  order: OrderBy[],
}

export type AttachmentLocation = { Url: string } | { File: string }

export interface IListPage<T> {
  readonly items: T[]
  readonly hasMore: boolean
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
