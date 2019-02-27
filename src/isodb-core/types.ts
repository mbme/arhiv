// Attachments
export interface IAttachment {
  readonly _id: string
  readonly _rev?: number
}

// Records
export interface IRecord {
  readonly _id: string
  readonly _type: string
  readonly _rev?: number
  _refs: string[]
  _attachmentRefs: string[]
  _deleted?: boolean
  _createdTs: number
  _updatedTs: number
}

export enum RecordType {
  Note = 'note',
  Track = 'track',
}

// Record types
export interface INote extends IRecord {
  readonly _type: RecordType.Note
  name: string
  data: string
}

export interface ITrack extends IRecord {
  readonly _type: RecordType.Track
  artist: string
  title: string
}

export interface IChangeset {
  /**
   * replica storage revision
   */
  baseRev: number

  /**
   * new or updated records
   */
  records: IRecord[]

  /**
   * new or updated attachments
   */
  attachments: IAttachment[]
}

export interface IChangesetResult {
  success: boolean

  /**
   * replica storage revision
   */
  baseRev: number

  /**
   * primary storage revision
   */
  currentRev: number

  /**
   * record or record id
   */
  records: Array<IRecord | string>

  /**
   * attachment or attachment id
   */
  attachments: Array<IAttachment | string>
}

interface IMergeConflict<T> {
  base: T
  updated: T
  local: T
}

export interface IMergeConflicts {
  records: Array<IMergeConflict<IRecord>>
  attachments: Array<IMergeConflict<IAttachment>>
}

export interface IResolvedConflicts {
  records: IRecord[]
  attachments: IAttachment[]
}

export type MergeFunction = (conflicts: IMergeConflicts) => Promise<IResolvedConflicts>
