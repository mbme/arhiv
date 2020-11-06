/* eslint-disable @typescript-eslint/triple-slash-reference */
/// <reference path="../../../app-shell/src/rpc.d.ts" />

import { Obj } from '@v/utils'
import {
  AttachmentLocation,
  CreateDocumentArgs,
  IAttachment,
  IDocument,
  IDocumentFilter,
  IListPage,
  MarkupNode,
} from './types'

interface IRPC {
  list<D extends IDocument<T, P>, T extends string = string, P extends Obj = Obj>(
    filter: IDocumentFilter<T>
  ): Promise<IListPage<D>>

  list(filter: IDocumentFilter): Promise<IListPage<IDocument>>

  get(id: string): Promise<IDocument | null>

  put(document: IDocument): Promise<void>

  create<D extends IDocument<T, P>, T extends string = string, P extends Obj = Obj>(
    args: CreateDocumentArgs<T>,
  ): Promise<D>

  parse_markup(markup: string): Promise<MarkupNode[]>

  get_attachment(id: string): Promise<IAttachment | null>
  get_attachment_location(id: string): Promise<AttachmentLocation>
  pick_attachments(): Promise<IAttachment[]>
}

export const API = window.RPC_PROXY as IRPC
