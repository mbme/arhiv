import { moveFile } from '~/utils/fs'
import {
  IDocument,
  IAttachment,
} from '../types'
import { IPrimaryStorage } from './primary-storage'
import { removeAtMut } from '~/utils'

export class PrimaryInMemStorage<T extends IDocument> implements IPrimaryStorage<T> {
  private _documents: T[] = []
  private _attachments: IAttachment[] = []
  private _rev = 0
  private _files = new Map<string, string>()

  constructor(
    private _tempDir: string,
  ) { }

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

  getAttachmentDataPath(id: string) {
    return this._files.get(id)
  }

  putDocument(document: T) {
    this.removeDocument(document._id)
    this._documents.push(document)
  }

  removeDocument(id: string) {
    const pos = this._documents.findIndex((item) => item._id === id)
    removeAtMut(this._documents, pos)
  }

  async addAttachment(attachment: IAttachment, attachmentPath: string) {
    this._attachments.push(attachment)
    const newFile = await moveFile(attachmentPath, this._tempDir)
    this._files.set(attachment._id, newFile)
  }

  updateAttachment(attachment: IAttachment) {
    const pos = this._attachments.findIndex((item) => item._id === attachment._id)

    if (pos === -1) {
      throw new Error(`Can't update attachment ${attachment._id}: not found`)
    }
    removeAtMut(this._attachments, pos)

    this._attachments.push(attachment)
  }

  removeAttachmentData(id: string) {
    if (!this.getAttachment(id)) {
      throw new Error(`Can't remove attachment data ${id}: not found`)
    }
    this._files.delete(id)
  }
}
