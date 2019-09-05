import { map2object } from '~/utils'
import {
  IDocument,
  IAttachment,
  IChangesetResult,
  IChangeset,
} from '../types'
import {
  IReplicaStorage,
  LocalAttachments,
} from './replica-storage'

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

  getChangeset(): [IChangeset<T>, LocalAttachments] {
    const changeset = {
      baseRev: this._rev,
      documents: this.getLocalDocuments(),
      attachments: this.getLocalAttachments(),
    }

    return [changeset, map2object(this._localFiles)]
  }

  upgrade(changesetResult: IChangesetResult<T>) {
    this._rev = changesetResult.currentRev
    this._documents.push(...changesetResult.documents)
    this._attachments.push(...changesetResult.attachments)
  }

  clearLocalData() {
    this._localDocuments.clear()
    this._localAttachments.clear()
    this._localFiles.clear()
  }
}
