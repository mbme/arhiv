import {
  IDocument,
  IAttachment,
  IChangesetResult,
} from '../types'
import {
  IReplicaStorage,
} from './types'

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

  async getDocuments() {
    return this._documents.slice(0)
  }

  async getLocalDocuments() {
    return Array.from(this._localDocuments.values())
  }

  async getAttachments() {
    return this._attachments.slice(0)
  }

  async getLocalAttachments() {
    return Array.from(this._localAttachments.values())
  }

  async getDocument(id: string) {
    return this._documents.find(item => item._id === id)
  }

  async getLocalDocument(id: string) {
    return this._localDocuments.get(id)
  }

  async getAttachment(id: string) {
    return this._attachments.find(item => item._id === id)
  }

  async getLocalAttachment(id: string) {
    return this._localAttachments.get(id)
  }

  async addLocalDocument(document: T) {
    this._localDocuments.set(document._id, document)

    return
  }

  async addLocalAttachment(attachment: IAttachment, file: File) {
    this._localAttachments.set(attachment._id, attachment)
    this._localFiles.set(attachment._id, file)

    return
  }

  async removeLocalDocument(id: string) {
    this._localDocuments.delete(id)

    return
  }

  async removeLocalAttachment(id: string) {
    this._localAttachments.delete(id)
    this._localFiles.delete(id)

    return
  }

  async getLocalAttachmentData(id: string) {
    return this._localFiles.get(id)
  }

  async upgrade(changesetResult: IChangesetResult<T>) {
    this._rev = changesetResult.currentRev

    const updatedDocumentsIds = changesetResult.documents.map(document => document._id)
    this._documents = this._documents.filter(document => !updatedDocumentsIds.includes(document._id))
    this._documents.push(...changesetResult.documents)

    const updatedAttachmentsIds = changesetResult.attachments.map(attachment => attachment._id)
    this._attachments = this._attachments.filter(attachment => !updatedAttachmentsIds.includes(attachment._id))
    this._attachments.push(...changesetResult.attachments)

    return
  }
}
