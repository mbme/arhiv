import { IReplicaStorage, Record, ChangedRecord } from './types'
import { removeMut, Callback } from '../../utils'

export default class ReplicaInMemStorage implements IReplicaStorage {
  _records: Record[] = []
  _rev = 0

  _localRecords: { [id: string]: ChangedRecord } = {}
  _localAttachments: { [id: string]: Blob } = {}

  _listeners: Callback[] = []

  getRev() {
    return this._rev
  }

  getRecords() {
    return this._records.slice(0)
  }

  setRecords(rev: number, records: Record[]) {
    this._rev = rev
    this._records = records
    this._onUpdate()
  }

  getLocalRecords() {
    return Object.values(this._localRecords)
  }

  getLocalAttachments() {
    return {
      ...this._localAttachments,
    }
  }

  // add or update existing local record
  addLocalRecord(record: ChangedRecord, blob?: File) {
    this._localRecords[record._id] = record
    if (blob) {
      this._localAttachments[record._id] = blob
    }
    this._onUpdate()
  }

  removeLocalRecord(id: string) {
    delete this._localRecords[id]
    delete this._localAttachments[id]
    this._onUpdate()
  }

  getAttachmentUrl(id: string) {
    if (this._localAttachments[id]) {
      return `local-attachment-url(${id})`
    }

    return `attachment-url(${id})`
  }

  clearLocalRecords() {
    this._localRecords = {}
    this._localAttachments = {}
    this._onUpdate()
  }

  _onUpdate() {
    this._listeners.forEach(cb => cb())
  }

  onUpdate(cb: Callback) {
    this._listeners.push(cb)
  }

  offUpdate(cb: Callback) {
    removeMut(this._listeners, cb)
  }
}
