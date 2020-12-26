/* eslint-disable @typescript-eslint/triple-slash-reference */
/// <reference path="../../../app-shell/src/rpc.d.ts" />

import {
  AttachmentLocation,
  ICreateDocumentArgs,
  IDataDescription,
  IAttachmentSource,
  IDocument,
  IDocumentFilter,
  IListPage,
  IPutDocumentArgs,
} from './types'

interface IRPC {
  list<D extends IDocument = IDocument>(filter: IDocumentFilter): Promise<IListPage<D>>

  get(id: string): Promise<IDocument | null>

  put(args: IPutDocumentArgs): Promise<void>

  create(args: ICreateDocumentArgs): Promise<IDocument>

  render_markup(markup: string): Promise<string>

  get_attachment_location(id: string): Promise<AttachmentLocation>
  pick_attachments(): Promise<IAttachmentSource[]>
}

export const API = window.RPC_PROXY as IRPC

export const MODULES = window.JS_VARIABLES.DATA_DESCRIPTION as Record<string, IDataDescription>
