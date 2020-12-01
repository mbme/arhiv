import { Obj } from '@v/utils'

export interface IDocument {
  readonly id: string
  readonly rev: number
  readonly createdAt: string
  readonly updatedAt: string
  readonly refs: readonly string[]
  readonly archived: boolean
  readonly data: {
    type: string,
    [name: string]: any,
  }
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
}

export interface IDocumentFilter {
  pageOffset?: number
  pageSize?: number
  matchers: IMatcher[]
  skipArchived?: boolean
  onlyStaged?: boolean
}

export type AttachmentLocation = { Url: string } | { File: string }

export interface IListPage<T> {
  readonly items: T[]
  readonly hasMore: boolean
}

export interface IDataDescription {
  readonly documentType: string
  readonly fields: {
    readonly [name: string]: IField
  }
}

export interface IField {
  fieldType: FieldType
}

export type FieldType =
  'String'
  | 'MarkupString'
  | 'Ref'
  | { Enum: string[] }

export interface ICreateDocumentArgs {
  readonly documentType: string
  readonly args: Obj
}

export interface IPutDocumentArgs {
  readonly document: IDocument
  readonly newAttachments: IAttachmentSource[]
}
