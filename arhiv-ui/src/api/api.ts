import { RPC_PROXY } from './rpc'
import {
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

  get_status(): Promise<string>

  pick_attachments(): Promise<string[]>

  get_schema(): Promise<IDataSchema>

  sync(): Promise<void>
}

export const API = RPC_PROXY as IRPC
