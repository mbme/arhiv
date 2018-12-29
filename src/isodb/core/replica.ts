import { isString, array2object, flatten } from '../../utils'
import PubSub from '../../utils/pubsub'
import { createLogger } from '../../logger'
import { getRandomId } from './utils'
import {
  IAttachment,
  MutableAttachmentFields,
  Record,
  MutableRecordFields,
  IChangesetResult,
} from './types'

const logger = createLogger('isodb-replica')

export interface IReplicaStorage {
  getRev(): number

  getRecords(): Record[]
  getLocalRecords(): Record[]

  getAttachments(): IAttachment[]
  getLocalAttachments(): IAttachment[]

  getRecord(id: string): Record | undefined
  getLocalRecord(id: string): Record | undefined

  getAttachment(id: string): IAttachment | undefined
  getLocalAttachment(id: string): IAttachment | undefined

  addLocalRecord(record: Record): void
  addLocalAttachment(attachment: IAttachment, blob?: File): void

  removeLocalRecord(id: string): void
  removeLocalAttachment(id: string): void

  getAttachmentUrl(id: string): string | undefined
  getLocalAttachmentsData(): { [id: string]: Blob }
  upgrade(rev: number, records: Record[], attachments: IAttachment[]): void
  clearLocalData(): void
}

interface IMergeConflict<T> {
  base: T
  updated: T
  local: T
}

export interface IMergeConflicts {
  records: Array<IMergeConflict<Record>>
  attachments: Array<IMergeConflict<IAttachment>>
}

export interface IResolvedConflicts {
  records: Record[]
  attachments: IAttachment[]
}

export type MergeFunction = (conflicts: IMergeConflicts) => Promise<IResolvedConflicts>

export interface IEvents {
  'db-update': undefined
}

export default class IsodbReplica {
  constructor(
    public _storage: IReplicaStorage,
    public events = new PubSub<IEvents>()
  ) { }

  _notify() {
    this.events.emit('db-update', undefined)
  }

  getRev() {
    return this._storage.getRev()
  }

  getAttachmentUrl(id: string) {
    if (!this.getAttachment(id)) {
      return undefined
    }

    return this._storage.getAttachmentUrl(id)
  }

  getRecord(id: string) {
    return this._storage.getLocalRecord(id)
      || this._storage.getRecord(id)
  }

  getAttachment(id: string) {
    return this._storage.getLocalAttachment(id)
      || this._storage.getAttachment(id)
  }

  /**
   * @returns all records, including local
   */
  getRecords() {
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
  addAttachment(id: string, blob: File, fields: MutableAttachmentFields = {}) {
    if (this.getAttachment(id)) throw new Error(`can't add attachment ${id}: already exists`)

    this._storage.addLocalAttachment({
      _id: id,
      _attachment: true,
      ...fields,
    }, blob)

    this._notify()
  }

  updateAttachment(id: string, fields: MutableAttachmentFields) {
    const attachment = this.getAttachment(id)
    if (!attachment) throw new Error(`can't update attachment ${id}: doesn't exist`)

    this._storage.addLocalAttachment({
      ...attachment,
      ...fields,
    })

    this._notify()
  }

  /**
   * @param fields key-value object with fields
   */
  addRecord(fields: MutableRecordFields) {
    const id = getRandomId()

    this._storage.addLocalRecord({
      _id: getRandomId(),
      ...fields,
    })

    this._compact()

    this._notify()

    return id
  }

  /**
   * @param id record id
   * @param fields key-value object with changed fields
   */
  updateRecord(id: string, fields: MutableRecordFields) {
    const record = this.getRecord(id)
    if (!record) throw new Error(`can't update record ${id}: doesn't exist`)

    this._storage.addLocalRecord({
      ...record,
      ...fields,
    })

    this._compact()

    this._notify()
  }

  async applyChangesetResult(changesetResult: IChangesetResult, merge: MergeFunction) {
    if (this.getRev() !== changesetResult.baseRev) {
      throw new Error(`Got rev ${changesetResult.baseRev} instead of ${this.getRev()}`)
    }

    const currentRecords = array2object(this._storage.getRecords(), item => item._id)
    const newRecords = changesetResult.records.map(item => isString(item) ? currentRecords[item] : item)

    const currentAttachments = array2object(this._storage.getAttachments(), item => item._id)
    const newAttachments = changesetResult.attachments.map(item => isString(item) ? currentAttachments[item] : item)

    if (changesetResult.success) {
      this._storage.upgrade(changesetResult.currentRev, newRecords, newAttachments)
      this._storage.clearLocalData()
      this._notify()

      return
    }

    const recordConflicts = []
    // for each local record
    for (const localRecord of this._storage.getLocalRecords()) {
      const existingRecord = currentRecords[localRecord._id]
      const newRecord = newRecords.find(item => item._id === localRecord._id)!

      // if is existing record & revision changed
      //   mark as a conflict
      if (existingRecord._rev !== newRecord._rev) {
        recordConflicts.push({
          base: existingRecord,
          updated: newRecord,
          local: localRecord,
        })
      }
    }
    const attachmentConflicts = []
    // for each local attachment
    for (const localAttachment of this._storage.getLocalAttachments()) {
      const existingAttachment = currentAttachments[localAttachment._id]
      const newAttachment = newAttachments.find(item => item._id === localAttachment._id)!

      // if is existing attachment & revision changed
      //   mark as a conflict
      if (existingAttachment._rev !== newAttachment._rev) {
        attachmentConflicts.push({
          base: existingAttachment,
          updated: newAttachment,
          local: localAttachment,
        })
      }
    }

    // resolve conflicts if needed
    if (recordConflicts.length || attachmentConflicts.length) {
      const resolvedConflicts = await merge({ records: recordConflicts, attachments: attachmentConflicts })

      for (const updatedRecord of resolvedConflicts.records) {
        this._storage.addLocalRecord(updatedRecord)
      }

      for (const updatedAttachment of resolvedConflicts.attachments) {
        this._storage.addLocalAttachment(updatedAttachment)
      }
    }

    // for each local record
    //   if references deleted record
    //     restore deleted record & all deleted records referenced by it
    const idsToCheck = flatten(this._storage.getLocalRecords().map(item => item._refs))
    const idsChecked = new Set()
    while (idsToCheck.length) {
      const id = idsToCheck.shift()!

      if (idsChecked.has(id)) continue

      const existingRecord = currentRecords[id]
      const newRecord = newRecords.find(item => item._id === id)
      if (existingRecord && !newRecord) {
        logger.info(`Restoring record ${id}`)
        this._storage.addLocalRecord(existingRecord) // restore record
        idsToCheck.push(...existingRecord._refs)
      }

      idsChecked.add(id)
    }

    this._storage.upgrade(changesetResult.currentRev, newRecords, newAttachments)

    this._notify()
  }

  /**
   * Remove unused local attachments
   */
  _compact() {
    const idsInUse = new Set()
    for (const record of this._storage.getRecords()) {
      for (const id of record._attachmentRefs) {
        idsInUse.add(id)
      }
    }
    const localAttachmentIds = new Set(this._storage.getLocalAttachments().map(item => item._id))

    for (const id of localAttachmentIds) {
      // remove unused new local attachments
      if (!idsInUse.has(id)) {
        logger.info(`Removing unused local attachment ${id}`)
        this._storage.removeLocalAttachment(id)
      }
    }
  }
}
