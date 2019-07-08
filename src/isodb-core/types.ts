export interface IDocument {
  readonly _id: string
  readonly _rev?: number
}

// Attachments
// tslint:disable-next-line:no-empty-interface
export interface IAttachment extends IDocument { }

export enum RecordType {
  Note = 'note',
  Track = 'track',
}

// Records
export interface IRecord extends IDocument {
  readonly _type: RecordType
  _refs: string[]
  _attachmentRefs: string[]
  _deleted?: boolean
  _createdTs: number
  _updatedTs: number
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

export type CompactDocument<T extends IDocument> = T | string

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
  records: Array<CompactDocument<IRecord>>

  /**
   * attachment or attachment id
   */
  attachments: Array<CompactDocument<IAttachment>>
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
