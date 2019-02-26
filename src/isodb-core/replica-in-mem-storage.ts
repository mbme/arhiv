import { map2object } from '~/utils'
import {
  IRecord,
  IAttachment,
} from './types'
import { IReplicaStorage } from './replica-storage'

export default class ReplicaInMemStorage implements IReplicaStorage {
  _records: IRecord[] = []
  _attachments: IAttachment[] = []
  _rev = 0

  _localRecords = new Map<string, IRecord>()
  _localAttachments = new Map<string, IAttachment>()
  _localFiles = new Map<string, Blob>()

  getRev() {
    return this._rev
  }

  getRecords() {
    return this._records.slice(0)
  }

  getLocalRecords() {
    return Object.values(this._localRecords)
  }

  getAttachments() {
    return this._attachments.slice(0)
  }

  getLocalAttachments() {
    return Object.values(this._localAttachments)
  }

  getRecord(id: string) {
    return this._records.find(item => item._id === id)
  }

  getLocalRecord(id: string) {
    return this._localRecords.get(id)
  }

  getAttachment(id: string) {
    return this._attachments.find(item => item._id === id)
  }

  getLocalAttachment(id: string) {
    return this._localAttachments.get(id)
  }

  /**
   * add or update existing local record
   */
  addLocalRecord(record: IRecord) {
    this._localRecords.set(record._id, record)
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

  removeLocalRecord(id: string) {
    this._localRecords.delete(id)
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

  upgrade(rev: number, records: IRecord[], attachments: IAttachment[]) {
    this._rev = rev
    this._records = records
    this._attachments = attachments
  }

  clearLocalData() {
    this._localRecords.clear()
    this._localAttachments.clear()
    this._localFiles.clear()
  }
}
