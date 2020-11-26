import { Obj } from '@v/utils'

export interface CreateDocumentArgs<T extends string> {
  readonly documentType: T
  readonly args: any
}

export interface PutDocumentArgs {
  readonly document: IDocument
  readonly newAttachments: IAttachmentSource[]
}

export interface IDocument<T extends string = string, P extends Obj = Obj> {
  readonly id: string
  readonly rev: number
  readonly type: T
  readonly createdAt: string
  readonly updatedAt: string
  readonly refs: readonly string[]
  readonly archived: boolean
  readonly data: P
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

export type Matcher = { Type: string } | { Data: { selector: string, pattern: string } }

export interface IDocumentFilter {
  pageOffset?: number
  pageSize?: number
  matchers: Matcher[]
  skipArchived?: boolean
  onlyStaged?: boolean
}

export type AttachmentLocation = { Url: string } | { File: string }

export interface IListPage<T> {
  readonly items: T[]
  readonly hasMore: boolean
}
