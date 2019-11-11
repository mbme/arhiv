import {
  PIDB,
} from '~/indexeddb'
import {
  IDocument,
  IAttachment,
  IChangesetResult,
} from '../types'
import {
  IReplicaStorage,
} from './types'

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

export class ReplicaIndexedDBStorage<T extends IDocument> implements IReplicaStorage<T> {
  private _rev = 0

  private constructor(
    private _idb: PIDB<IObjectStores<T>>,
  ) { }

  private async _init() {
    const maxDocumentRev = (await this.getDocuments())
      .reduce((acc, document) => Math.max(acc, document._rev), 0)
    const maxAttachemntRev = (await this.getAttachments())
      .reduce((acc, attachment) => Math.max(acc, attachment._rev), 0)

    // db rev is max document or attachment rev
    this._rev = Math.max(maxDocumentRev, maxAttachemntRev)
  }

  public static async open<T extends IDocument>() {
    const db = await PIDB.open<IObjectStores<T>>('arhiv-replica', 1, (oldVersion, db) => {
      if (oldVersion < 1) { // create db
        db.createObjectStore('documents', '_id')
        db.createObjectStore('documents-local', '_id')
        db.createObjectStore('attachments', '_id')
        db.createObjectStore('attachments-local', '_id')
        db.createObjectStore('attachments-data', '_id')
      }
    })

    const replica = new ReplicaIndexedDBStorage(db)
    await replica._init()

    return replica
  }

  getRev() {
    return this._rev
  }

  getDocuments() {
    return this._idb.getAll('documents')
  }

  getLocalDocuments() {
    return this._idb.getAll('documents-local')
  }

  getAttachments() {
    return this._idb.getAll('attachments')
  }

  getLocalAttachments() {
    return this._idb.getAll('attachments-local')
  }

  getDocument(id: string) {
    return this._idb.get('documents', id)
  }

  getLocalDocument(id: string) {
    return this._idb.get('documents-local', id)
  }

  getAttachment(id: string) {
    return this._idb.get('attachments', id)
  }

  getLocalAttachment(id: string) {
    return this._idb.get('attachments-local', id)
  }

  addLocalDocument(document: T) {
    return this._idb.put('documents-local', document)
  }

  async addLocalAttachment(attachment: IAttachment, file: File) {
    const tx = this._idb.transactionRW('attachments-local', 'attachments-data')

    await Promise.all([
      tx.store('attachments-local').put(attachment),
      tx.store('attachments-data').put({ _id: attachment._id, data: file }),
    ])
  }

  removeLocalDocument(id: string) {
    return this._idb.delete('documents-local', id)
  }

  async removeLocalAttachment(id: string) {
    const tx = this._idb.transactionRW('attachments-local', 'attachments-data')

    await Promise.all([
      tx.store('attachments-local').delete(id),
      tx.store('attachments-data').delete(id),
    ])
  }

  async getLocalAttachmentData(id: string) {
    const result = await this._idb.get('attachments-data', id)

    return result?.data
  }

  async upgrade(changesetResult: IChangesetResult<T>) {
    this._rev = changesetResult.currentRev

    const tx = this._idb.transactionRW('documents', 'attachments')

    await Promise.all([
      tx.store('documents').putAll(changesetResult.documents),
      tx.store('attachments').putAll(changesetResult.attachments),
    ])
  }
}
