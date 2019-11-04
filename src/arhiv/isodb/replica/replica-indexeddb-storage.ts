import {
  openDB,
  request2promise,
} from '~/indexeddb'
import {
  IDocument,
  IAttachment,
  IChangesetResult,
} from '../types'
import {
  IReplicaStorage,
} from './types'

export class ReplicaIndexedDBStorage<T extends IDocument> implements IReplicaStorage<T> {
  private _documents: T[] = []
  private _attachments: IAttachment[] = []
  private _rev = 0

  private _localDocuments = new Map<string, T>()
  private _localAttachments = new Map<string, IAttachment>()
  private _localFiles = new Map<string, Blob>()

  private constructor(
    private _idb: IDBDatabase,
  ) { }

  private async _init() {
    const maxDocumentRev = this.getDocuments()
      .reduce((acc, document) => Math.max(acc, document._rev), 0)
    const maxAttachemntRev = this.getAttachments()
      .reduce((acc, attachment) => Math.max(acc, attachment._rev), 0)

    // db rev is max document or attachment rev
    this._rev = Math.max(maxDocumentRev, maxAttachemntRev)
  }

  public static async open() {
    const db = await openDB('arhiv-replica', 1, (oldVersion, db) => {
      if (oldVersion < 1) { // create db
        db.createObjectStore('documents', { keyPath: '_id' })
        db.createObjectStore('documents-local', { keyPath: '_id' })
        db.createObjectStore('attachments', { keyPath: '_id' })
        db.createObjectStore('attachments-local', { keyPath: '_id' })
        db.createObjectStore('attachments-data', { keyPath: '_id' })
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
    return request2promise<T[]>(this._idb.transaction('documents').objectStore('documents').getAll())
  }

  getLocalDocuments() {
    return request2promise<T[]>(this._idb.transaction('documents-local').objectStore('documents-local').getAll())
  }

  getAttachments() {
    return this._attachments.slice(0)
  }

  getLocalAttachments() {
    return Array.from(this._localAttachments.values())
  }

  getDocument(id: string) {
    return this._documents.find(item => item._id === id)
  }

  getLocalDocument(id: string) {
    return this._localDocuments.get(id)
  }

  getAttachment(id: string) {
    return this._attachments.find(item => item._id === id)
  }

  getLocalAttachment(id: string) {
    return this._localAttachments.get(id)
  }

  addLocalDocument(document: T) {
    this._localDocuments.set(document._id, document)
  }

  addLocalAttachment(attachment: IAttachment, file: File) {
    this._localAttachments.set(attachment._id, attachment)
    this._localFiles.set(attachment._id, file)
  }

  removeLocalDocument(id: string) {
    this._localDocuments.delete(id)
  }

  removeLocalAttachment(id: string) {
    this._localAttachments.delete(id)
    this._localFiles.delete(id)
  }

  getLocalAttachmentData(id: string) {
    return this._localFiles.get(id)
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
