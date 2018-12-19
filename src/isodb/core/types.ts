export type RecordType = 'note' | 'other'

export interface IRecord {
  _id: string
  _rev: number
  _refs: string[]
  _deleted?: boolean

  [key: string]: any
}
export interface IAttachment {
  _id: string
  _rev: number
  _attachment: true
}
export type Record = IRecord | IAttachment

export interface IChangedRecord {
  _id: string
  _refs: string[]
  _deleted?: boolean

  [key: string]: any
}
export interface IChangedAttachment {
  _id: string
  _attachment: true
}
export type ChangedRecord = IChangedRecord | IChangedAttachment

export interface INote extends IRecord {
  type: RecordType
  name: string
  data: string
}

// record or record id
export type RecordInfo = IRecord | IAttachment | string

export interface IPrimaryStorage {
  /**
   * @returns storage revision
   */
  getRev(): number

  setRev(rev: number): void

  /**
   * @param id attachment id
   * @returns path to attachment
   */
  getAttachment(id: string): string | undefined

  getRecords(): Record[]

  putRecord(updatedRecord: Record, attachmentPath?: string): void

  removeRecord(id: string): void
}

export interface IReplicaStorage {
  getRev(): number

  /**
   * @param id attachment id
   * @returns attachment url
   */
  getAttachmentUrl(id: string): string | undefined

  setRecords(rev: number, records: Record[]): void
  getRecords(): Record[]
  getLocalRecords(): ChangedRecord[]

  getLocalAttachments(): { [id: string]: Blob }

  addLocalRecord(record: ChangedRecord, blob?: File): void
  removeLocalRecord(id: string): void

  clearLocalRecords(): void
}

export type MergeFunction = (base: Record, updated: Record, local: ChangedRecord) => Promise<ChangedRecord>

export interface IPatchResponse {
  applied: boolean
  baseRev: number
  currentRev: number
  records: RecordInfo[]
}
