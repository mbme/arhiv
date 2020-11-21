/* eslint-disable @typescript-eslint/triple-slash-reference */
/// <reference path="../../../app-shell/src/rpc.d.ts" />

import { Obj } from '@v/utils'
import {
  AttachmentLocation,
  CreateDocumentArgs,
  IAttachment,
  IAttachmentSource,
  IDocument,
  IDocumentFilter,
  IListPage,
  MarkupNode,
  PutDocumentArgs,
} from './types'

interface IRPC {
  list<D extends IDocument = IDocument>(filter: IDocumentFilter): Promise<IListPage<D>>

  get(id: string): Promise<IDocument | null>

  put(args: PutDocumentArgs): Promise<void>

  create<D extends IDocument<T, P>, T extends string = string, P extends Obj = Obj>(
    args: CreateDocumentArgs<T>,
  ): Promise<D>

  parse_markup(markup: string): Promise<MarkupNode[]>

  get_attachment(id: string): Promise<IAttachment | null>
  get_attachment_location(id: string): Promise<AttachmentLocation>
  pick_attachments(): Promise<IAttachmentSource[]>
}

export const API = window.RPC_PROXY as IRPC
