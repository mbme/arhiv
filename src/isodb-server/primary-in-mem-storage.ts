import {
  IRecord,
  IAttachment,
} from '~/isodb-core/types'
import { IPrimaryStorage } from './primary-storage'

export default class PrimaryInMemStorage implements IPrimaryStorage {
  _records: IRecord[] = []
  _attachments: IAttachment[] = []
  _rev = 0
  _files = new Map<string, string>()

  getRev() {
    return this._rev
  }

  setRev(rev: number) {
    this._rev = rev
  }

  getRecords() {
    return this._records.slice(0)
  }

  getAttachments() {
    return this._attachments.slice(0)
  }

  getRecord(id: string) {
    return this._records.find(item => item._id === id)
  }

  getAttachment(id: string) {
    return this._attachments.find(item => item._id === id)
  }

  getAttachmentPath(id: string) {
    return this._files.get(id)
  }

  putRecord(record: IRecord) {
    this.removeRecord(record._id)
    this._records.push(record)
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

  removeRecord(id: string) {
    const pos = this._records.findIndex((item) => item._id === id)
    if (pos > -1) {
      this._records.splice(pos, 1)
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
