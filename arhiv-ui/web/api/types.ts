import { Obj } from '@v/utils'

export interface CreateDocumentArgs<T extends string> {
  documentType: T
  args: any
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

export interface IDocumentFilter<T extends string | undefined = undefined> {
  type: T
  pageOffset?: number
  pageSize?: number
  matcher?: IMatcher
  skipArchived?: boolean
  onlyStaged?: boolean
}

export type AttachmentLocation = { Url: string } | { File: string }

export interface IListPage<T> {
  items: T[]
  hasMore: boolean
}

export type MarkupInlineNode =
  { String: string }
  | { Link: [string, string] }
  | { Bold: string }
  | { Mono: string }
  | { Strikethrough: string }

export type MarkupNode =
  { Newlines: number }
  | { Header: string }
  | { Line: MarkupInlineNode[] }
