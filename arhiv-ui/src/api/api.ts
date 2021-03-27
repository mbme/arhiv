import { RPC_PROXY } from './rpc'
import {
  ICreateDocumentArgs,
  IDocument,
  IFilter,
  IListPage,
  IPutDocumentArgs,
  IDataSchema,
  IDocumentExt,
} from './types'

interface IRPC {
  list(filter: IFilter): Promise<IListPage<IDocumentExt>>

  get(id: string): Promise<IDocument | null>

  put(args: IPutDocumentArgs): Promise<void>

  create(args: ICreateDocumentArgs): Promise<IDocument>

  render_markup(markup: string): Promise<string>

  get_status(): Promise<string>

  pick_attachments(): Promise<string[]>

  get_schema(): Promise<IDataSchema>

  sync(): Promise<void>

  is_sync_required(): Promise<boolean>
}

export const API = RPC_PROXY as IRPC
