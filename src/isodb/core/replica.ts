import { isString, array2object, flatten } from '../../utils'
import { createLogger } from '../../logger'
import { getRandomId, isAttachment } from './utils'
import {
  IReplicaStorage,
  Record,
  ChangedRecord,
  MergeFunction,
  IPatchResponse,
} from './types'

const logger = createLogger('isodb-replica')

export default class ReplicaDB {
  _storage: IReplicaStorage

  constructor(storage: IReplicaStorage) {
    this._storage = storage
  }

  /**
   * @returns storage revision
   */
  getRev() {
    return this._storage.getRev()
  }

  /**
   * @param id attachment id
   * @returns path to attachment
   */
  getAttachmentUrl(id: string) {
    if (!this.getRecord(id)) {
      return undefined
    }

    return this._storage.getAttachmentUrl(id)
  }

  /**
   * @param id record id
   */
  getRecord(id: string): Record | ChangedRecord | undefined {
    return this._storage.getLocalRecords().find(item => item._id === id)
      || this._storage.getRecords().find(item => item._id === id)
  }

  /**
   * @returns all records, including local
   */
  getAll(): Array<Record | ChangedRecord> {
    const localRecords = this._storage.getLocalRecords()
    const localIds = new Set(localRecords.map(item => item._id))

    const records = this._storage.getRecords().filter(item => !localIds.has(item._id))

    return [
      ...records,
      ...localRecords,
    ]
  }

  /**
   * @param id sha256 of file content
   * @param blob file content
   * @param [fields] additional fields
   */
  addAttachment(id: string, blob: File, fields = {}) {
    if (this.getRecord(id)) throw new Error(`can't add attachment ${id}: already exists`)

    // FIXME in transaction
    this._storage.addLocalRecord({
      _id: id,
      _attachment: true,
      ...fields,
    }, blob)
  }

  updateAttachment(id: string, fields: object) {
    const record = this.getRecord(id)
    if (!record) throw new Error(`can't update attachment ${id}: doesn't exist`)
    if (!isAttachment(record)) throw new Error(`can't update attachment ${id}: not an attachment`)

    this._storage.addLocalRecord({
      ...record,
      ...fields,
    })
  }

  /**
   * @param fields key-value object with fields
   * @param [refs=[]] record's refs
   */
  addRecord(fields: object, refs: string[] = []) {
    const id = getRandomId()

    this._storage.addLocalRecord({
      _id: getRandomId(),
      _refs: refs,
      ...fields,
    })

    this._compact()

    return id
  }

  /**
   * @param id record id
   * @param fields key-value object with changed fields
   * @param [refs] new refs (not used if record is attachment)
   * @param [deleted=false] if record is deleted
   */
  updateRecord(id: string, fields: object, refs?: string[], deleted = false) {
    const record = this.getRecord(id)
    if (!record) throw new Error(`can't update record ${id}: doesn't exist`)
    if (isAttachment(record)) throw new Error(`can't update record ${id}: its an attachment`)

    this._storage.addLocalRecord({
      ...record,
      _refs: refs || record._refs,
      _deleted: deleted,
      ...fields,
    })

    this._compact()
  }

  _getLocalRecords() {
    return this._storage.getLocalRecords()
  }

  _getLocalAttachments() {
    return this._storage.getLocalAttachments()
  }

  async applyPatch({ applied, baseRev, currentRev, records }: IPatchResponse, merge: MergeFunction) {
    if (this.getRev() !== baseRev) {
      throw new Error(`Got rev ${baseRev} instead of ${this.getRev()}`)
    }

    const currentRecords = array2object(this._storage.getRecords(), record => record._id)
    const newRecords = records.map(item => isString(item) ? currentRecords[item] : item)

    if (applied) {
      this._storage.setRecords(currentRev, newRecords)
      this._storage.clearLocalRecords()
      return
    }

    // for each local record
    for (const localRecord of this._storage.getLocalRecords()) {
      const existingRecord = currentRecords[localRecord._id]
      const newRecord = newRecords.find(item => item._id === localRecord._id)!

      // if is existing record & revision changed
      //   merge
      if (existingRecord._rev !== newRecord._rev) {
        this._storage.addLocalRecord(await merge(existingRecord, newRecord, localRecord))
      }
    }

    // for each local record
    //   if references deleted record
    //     restore deleted record & all deleted records referenced by it
    const idsToCheck = flatten(this._storage.getLocalRecords().map(item => (item as any)._refs || []))
    const idsChecked = new Set()
    while (idsToCheck.length) {
      const id = idsToCheck.shift()

      if (idsChecked.has(id)) continue

      const existingRecord = currentRecords[id]
      const newRecord = newRecords.find(item => item._id === id)
      if (existingRecord && !newRecord) {
        if (isAttachment(existingRecord)) {
          logger.warn(`Can't restore attachment ${id}, skipping`)
        } else {
          logger.info(`Restoring record ${id}`)
          this._storage.addLocalRecord(existingRecord) // restore record
          idsToCheck.push(...existingRecord._refs)
        }
      }

      idsChecked.add(id)
    }

    // merge patch
    this._storage.setRecords(currentRev, newRecords)
  }

  /**
   * Remove unused local attachments
   */
  _compact() {
    const idsInUse = new Set()
    const attachmentIds = new Set()
    for (const record of this._storage.getLocalRecords()) {
      if (isAttachment(record)) {
        attachmentIds.add(record._id)
      } else {
        record._refs.forEach(id => idsInUse.add(id))
      }
    }

    for (const id of attachmentIds) {
      const existingRecord = this._storage.getRecords().find(item => item._id === id)

      // remove unused new local attachments
      if (!idsInUse.has(id) && !existingRecord) {
        logger.info(`Removing unused local attachment ${id}`)
        this._storage.removeLocalRecord(id)
      }
    }
  }
}
