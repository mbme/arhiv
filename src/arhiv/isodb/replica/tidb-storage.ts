import {
  TIDB,
} from '~/web-tidb'
import {
  IDocument,
  IAttachment,
  IChangesetResult,
} from '../types'

interface IBlob {
  _id: string
  data: Blob
}

interface IObjectStores<T extends IDocument> {
  'documents': T
  'documents-local': T
  'attachments': IAttachment
  'attachments-local': IAttachment
  'attachments-data': IBlob
}
export class TIDBStorage<T extends IDocument> {
  private _rev = 0

  private constructor(
    private _idb: TIDB<IObjectStores<T>>,
  ) { }

  getRev() {
    return this._rev
  }

  async getDocument(id: string): Promise<T | undefined> {
    const tx = this._idb.transaction('documents', 'documents-local')

    const localDocument = await tx.store('documents-local').get(id)
    if (localDocument) {
      return localDocument
    }

    return tx.store('documents').get(id)
  }

  async getAttachment(id: string): Promise<IAttachment | undefined> {
    const tx = this._idb.transaction('attachments', 'attachments-local')

    const localAttachment = await tx.store('attachments-local').get(id)
    if (localAttachment) {
      return localAttachment
    }

    return tx.store('attachments').get(id)
  }
}
