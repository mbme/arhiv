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

type Store<T extends IDocument> = {
  'documents': T,
  'documents-local': T,
  'attachments': IAttachment,
  'attachments-local': IAttachment,
  'attachments-data': IBlob,
}

export class ReplicaIndexedDBStorage<T extends IDocument> implements IReplicaStorage<T> {
  private _rev = 0

  private constructor(
    private _idb: PIDB<Store<T>>,
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
    const db = await PIDB.open<Store<T>>('arhiv-replica', 1, (oldVersion, db) => {
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
    this._localDocuments.delete(id)
  }

  removeLocalAttachment(id: string) {
    this._localAttachments.delete(id)
    this._localFiles.delete(id)
  }

  async getLocalAttachmentData(id: string) {
    const result = await this._idb.get('attachments-data', id)

    return result?.data
  }

  upgrade(changesetResult: IChangesetResult<T>) {
    this._rev = changesetResult.currentRev

    const updatedDocumentsIds = changesetResult.documents.map(document => document._id)
    this._documents = this._documents.filter(document => !updatedDocumentsIds.includes(document._id))
    this._documents.push(...changesetResult.documents)

    const updatedAttachmentsIds = changesetResult.attachments.map(attachment => attachment._id)
    this._attachments = this._attachments.filter(attachment => !updatedAttachmentsIds.includes(attachment._id))
    this._attachments.push(...changesetResult.attachments)
  }
}
