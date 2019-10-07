import {
  IDocument,
  IAttachment,
} from '../types'
import { IPrimaryStorage } from './primary-storage'

export class PrimaryInMemStorage<T extends IDocument> implements IPrimaryStorage<T> {
  _documents: T[] = []
  _attachments: IAttachment[] = []
  _rev = 0
  _files = new Map<string, string>()

  getRev() {
    return this._rev
  }

  setRev(rev: number) {
    this._rev = rev
  }

  getDocuments() {
    return this._documents.slice(0)
  }

  getAttachments() {
    return this._attachments.slice(0)
  }

  getDocument(id: string) {
    return this._documents.find(item => item._id === id)
  }

  getAttachment(id: string) {
    return this._attachments.find(item => item._id === id)
  }

  getAttachmentPath(id: string) {
    return this._files.get(id)
  }

  putDocument(document: T) {
    this.removeDocument(document._id)
    this._documents.push(document)
  }

  removeDocument(id: string) {
    const pos = this._documents.findIndex((item) => item._id === id)
    if (pos > -1) {
      this._documents.splice(pos, 1)
    }
  }

  addAttachment(attachment: IAttachment, attachmentPath: string) {
    this._attachments.push(attachment)
    this._files.set(attachment._id, attachmentPath)
  }

  updateAttachment(attachment: IAttachment) {
    const attachmentPath = this._files.get(attachment._id)
    this.removeAttachment(attachment._id)
    if (attachmentPath) {
      this.addAttachment(attachment, attachmentPath)
    }
  }

  removeAttachment(id: string) {
    const pos = this._attachments.findIndex((item) => item._id === id)
    if (pos > -1) {
      this._attachments.splice(pos, 1)
      this._files.delete(id)
    }
  }
}
