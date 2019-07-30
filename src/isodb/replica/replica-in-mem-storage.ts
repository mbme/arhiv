import { map2object } from '~/utils'
import {
  IDocument,
  IAttachment,
} from '../types'
import { IReplicaStorage } from './replica-storage'

export class ReplicaInMemStorage<T extends IDocument> implements IReplicaStorage<T> {
  private _documents: T[] = []
  private _attachments: IAttachment[] = []
  private _rev = 0

  private _localDocuments = new Map<string, T>()
  private _localAttachments = new Map<string, IAttachment>()
  private _localFiles = new Map<string, Blob>()

  getRev() {
    return this._rev
  }

  getDocuments() {
    return this._documents.slice(0)
  }

  getLocalDocuments() {
    return Array.from(this._localDocuments.values())
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

  /**
   * add or update existing local attachment
   */
  addLocalAttachment(attachment: IAttachment, blob?: File) {
    this._localAttachments.set(attachment._id, attachment)
    if (blob) {
      this._localFiles.set(attachment._id, blob)
    }
  }

  removeLocalDocument(id: string) {
    this._localDocuments.delete(id)
  }

  removeLocalAttachment(id: string) {
    this._localAttachments.delete(id)
    this._localFiles.delete(id)
  }

  getAttachmentUrl(id: string) {
    if (this._localFiles.has(id)) {
      return `local-attachment-url(${id})`
    }

    return `attachment-url(${id})`
  }

  getLocalAttachmentsData() {
    return map2object(this._localFiles)
  }

  upgrade(rev: number, documents: T[], attachments: IAttachment[]) {
    this._rev = rev
    this._documents = documents
    this._attachments = attachments
  }

  clearLocalData() {
    this._localDocuments.clear()
    this._localAttachments.clear()
    this._localFiles.clear()
  }
}
