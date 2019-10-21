import {
  removeAtMut,
  getLastEl,
} from '~/utils'
import { moveFile } from '~/utils/fs'
import {
  IDocument,
  IAttachment,
} from '../types'
import { IPrimaryStorage } from './primary-storage'

export class PrimaryInMemStorage<T extends IDocument> implements IPrimaryStorage<T> {
  private _documents: Map<string, T[]> = new Map()
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
    return Array.from(this._documents.values()).map(getLastEl)
  }

  getDocument(id: string) {
    const revisions = this._documents.get(id)
    if (!revisions) {
      return undefined
    }

    return getLastEl(revisions)
  }

  getDocumentHistory(id: string) {
    const revisions = this._documents.get(id)

    if (!revisions) {
      return undefined
    }

    return [...revisions]
  }

  putDocument(document: T) {
    const revisions = this._documents.get(document._id) || []
    revisions.push(document)

    this._documents.set(document._id, revisions)
  }

  getAttachments() {
    return this._attachments.slice(0)
  }

  async addAttachment(attachment: IAttachment, attachmentPath: string) {
    this._attachments.push(attachment)
    const newFile = await moveFile(attachmentPath, this._tempDir)
    this._files.set(attachment._id, newFile)
  }

  getAttachment(id: string) {
    return this._attachments.find(item => item._id === id)
  }

  getAttachmentDataPath(id: string) {
    return this._files.get(id)
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
