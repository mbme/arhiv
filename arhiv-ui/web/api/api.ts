/* eslint-disable @typescript-eslint/triple-slash-reference */
/// <reference path="../../../app-shell/src/rpc.d.ts" />

import { Obj } from '@v/utils'

export interface IDocument<T extends string = string, P extends Obj = Obj> {
  readonly id: string
  readonly rev: number
  readonly type: T
  readonly createdAt: string
  readonly updatedAt: string
  readonly refs: readonly string[]
  readonly attachmentRefs: readonly string[]
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

interface IRPC {
  list<T extends string, P extends Obj, D extends IDocument<T, P> | undefined>(
    filter: IDocumentFilter<D extends undefined ? undefined : T>
  ): Promise<IListPage<D extends undefined ? IDocument : D>>

  get(id: string): Promise<IDocument | null>
  put(document: IDocument): Promise<void>
  create<T extends string, P extends Obj, D extends IDocument<T, P>>(type: T): Promise<D>

  get_attachment(id: string): Promise<IAttachment | null>
  get_attachment_location(id: string): Promise<AttachmentLocation>
  pick_attachments(): Promise<IAttachment[]>
}

export const API = window.RPC_PROXY as IRPC
