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

  addLocalDocument(document: T) {
    return this._idb.put('documents-local', document)
  }

  async getAttachment(id: string): Promise<IAttachment | undefined> {
    const tx = this._idb.transaction('attachments', 'attachments-local')

    const localAttachment = await tx.store('attachments-local').get(id)
    if (localAttachment) {
      return localAttachment
    }

    return tx.store('attachments').get(id)
  }

  async getLocalAttachmentData(id: string) {
    const result = await this._idb.get('attachments-data', id)

    return result?.data
  }

  async addLocalAttachment(attachment: IAttachment, file: File) {
    const tx = this._idb.transactionRW('attachments-local', 'attachments-data')

    await Promise.all([
      tx.store('attachments-local').put(attachment),
      tx.store('attachments-data').put({ _id: attachment._id, data: file }),
    ])
  }

  async getDocuments() {
    const tx = this._idb.transaction('documents', 'documents-local')

    const localDocuments = await tx.store('documents-local').getAll()
    const localIds = new Set(localDocuments.map(document => document._id))

    const documents = await tx.store('documents').getAll()

    return [
      ...localDocuments,
      ...documents.filter(document => !localIds.has(document._id)),
    ].filter(document => !document._deleted)
  }
}
