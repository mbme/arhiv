import {
  RecordInfo,
  Record,
  ChangedRecord,
  IPrimaryStorage,
} from './types'
import { isAttachment, isDeleted } from './utils'

export default class PrimaryDB {
  _storage: IPrimaryStorage

  constructor(storage: IPrimaryStorage) {
    this._storage = storage
  }

  /**
   * @param rev minimum revision to include
   * @returns array of record if _id is >= rev, id otherwise
   */
  getAll(rev = 0): RecordInfo[] {
    const currentRev = this.getRev()
    if (rev > currentRev) {
      throw new Error(`Got request for the future rev ${rev}, current rev is ${currentRev}`)
    }

    return this._storage.getRecords().map(item => item._rev >= rev ? item : item._id)
  }

  /**
   * @returns storage revision
   */
  getRev() {
    return this._storage.getRev()
  }

  /**
   * @param id record id
   */
  getRecord(id: string): Record | undefined {
    return this._storage.getRecords().find(item => item._id === id)
  }

  /**
   * @param id attachment id
   * @returns path to attachment
   */
  getAttachment(id: string) {
    return this._storage.getAttachment(id)
  }

  /**
   * @param rev client's storage revision
   * @param records new or updated records
   * @param [newAttachments] id -> path map of new attachments
   */
  applyChanges(rev: number, records: ChangedRecord[], newAttachments: { [id: string]: string } = {}) {
    if (this._storage.getRev() !== rev) { // ensure client had latest revision
      return false
    }

    if (!records.length) { // skip empty changesets
      return true
    }

    const newRev = rev + 1

    for (const changedRecord of records) {
      const attachment = newAttachments[changedRecord._id]
      const existingRecord = this.getRecord(changedRecord._id)

      if (existingRecord && isAttachment(existingRecord) !== isAttachment(changedRecord)) {
        throw new Error(`Can't change _attachment status for the record ${changedRecord._id}`)
      }

      if (existingRecord && isAttachment(existingRecord) && attachment) {
        throw new Error(`Can't replace attachment for the record ${changedRecord._id}`)
      }

      if (isAttachment(changedRecord) && !attachment) {
        throw new Error(`Missing attachment for the record ${changedRecord._id}`)
      }

      if (!isAttachment(changedRecord) && attachment) {
        throw new Error(`Unexpected attachment for the record ${changedRecord._id}`)
      }

      this._storage.putRecord({
        ...changedRecord,
        _rev: newRev,
      }, attachment)
    }

    this._storage.setRev(newRev)

    return true
  }

  /**
   * Physically remove orphan deleted records & orphan attachments.
   */
  compact() {
    const records = this._storage.getRecords()

    const validIds = new Set(
      records.filter(item => !isAttachment(item) && !isDeleted(item)).map(item => item._id)
    )

    const idsToCheck = Array.from(validIds)
    const idsChecked = new Set()
    while (idsToCheck.length) {
      const record = records.find(item => item._id === idsToCheck[0])!

      const refs: string[] = (record as any)._refs || []

      for (const id of refs) {
        if (validIds.has(id)) {
          continue
        }

        if (!isDeleted(record)) {
          validIds.add(id)
        }

        if (!idsToCheck.includes(id) && !idsChecked.has(id)) {
          idsToCheck.push(id)
        }
      }

      idsChecked.add(idsToCheck.shift()) // pop first item
    }

    let removedRecords = 0
    let removedAttachments = 0

    for (const record of records) {
      if (validIds.has(record._id)) {
        continue
      }

      if (isAttachment(record)) {
        removedAttachments += 1
      } else {
        removedRecords += 1
      }
      this._storage.removeRecord(record._id) // FIXME in transaction
    }

    if (removedRecords + removedAttachments) { // update revision if there were any changes
      this._storage.setRev(this._storage.getRev() + 1)
      // TODO log numbers
    }
  }
}
