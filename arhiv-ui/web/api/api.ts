/* eslint-disable @typescript-eslint/triple-slash-reference */
/// <reference path="../../../app-shell/src/rpc.d.ts" />

import { Dict } from '@v/utils'
import {
  AttachmentLocation,
  ICreateDocumentArgs,
  IDataDescription,
  IAttachment,
  IAttachmentSource,
  IDocument,
  IDocumentFilter,
  IListPage,
  IPutDocumentArgs,
} from './types'

interface IRPC {
  list_data_descriptions(): Promise<Dict<IDataDescription>>

  list<D extends IDocument = IDocument>(filter: IDocumentFilter): Promise<IListPage<D>>

  get(id: string): Promise<IDocument | null>

  put(args: IPutDocumentArgs): Promise<void>

  create(args: ICreateDocumentArgs): Promise<IDocument>

  render_markup(markup: string): Promise<string>

  get_attachment(id: string): Promise<IAttachment | null>
  get_attachment_location(id: string): Promise<AttachmentLocation>
  pick_attachments(): Promise<IAttachmentSource[]>
}

export const API = window.RPC_PROXY as IRPC
