import { IPrimaryStorage, IRecord } from './types'

export default class PrimaryInMemStorage implements IPrimaryStorage {
  _records: IRecord[] = []
  _rev = 0
  _attachments: { [id: string]: string } = {}

  getRecords() {
    return this._records.slice(0)
  }

  getRev() {
    return this._rev
  }

  setRev(rev: number) {
    this._rev = rev
  }

  getAttachment(id: string) {
    return this._attachments[id]
  }

  putRecord(record: IRecord, attachmentPath?: string) {
    this.removeRecord(record._id)
    this._records.push(record)
    if (attachmentPath) {
      this._attachments[record._id] = attachmentPath
    }
  }

  removeRecord(id: string) {
    const pos = this._records.findIndex((item) => item._id === id)
    if (pos > -1) {
      this._records.splice(pos, 1)
      delete this._attachments[id]
    }
  }
}
