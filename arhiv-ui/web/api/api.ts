/* eslint-disable @typescript-eslint/triple-slash-reference */
/// <reference path="../../../app-shell/src/rpc.d.ts" />

import {
  AttachmentLocation,
  ICreateDocumentArgs,
  IDocument,
  IFilter,
  IListPage,
  IPutDocumentArgs,
  IDataSchema,
  IRenderMarkupArgs,
} from './types'

interface IRPC {
  list<D extends IDocument = IDocument>(filter: IFilter): Promise<IListPage<D>>

  get(id: string): Promise<IDocument | null>

  put(args: IPutDocumentArgs): Promise<void>

  create(args: ICreateDocumentArgs): Promise<IDocument>

  render_markup(args: IRenderMarkupArgs): Promise<string>

  get_attachment_location(id: string): Promise<AttachmentLocation>

  get_status(): Promise<string>

  pick_attachments(): Promise<string[]>
}

export const API = window.RPC_PROXY as IRPC

export const SCHEMA = window.JS_VARIABLES.DATA_SCHEMA as IDataSchema
