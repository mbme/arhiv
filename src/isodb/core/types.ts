import { Omit } from '../../utils'

// Attachments
export interface IAttachment {
  readonly _id: string
  readonly _attachment: true
  readonly _rev?: number
}
export type MutableAttachmentFields = Omit<IAttachment, '_id' | '_rev' | '_attachment'>

// Records
interface IRecord {
  readonly _id: string
  readonly _type: string
  readonly _rev?: number
  _refs: string[]
  _attachmentRefs: string[]
  _deleted?: boolean
  _createdTs: number
  _updatedTs: number
}

// Record types
export interface INote extends IRecord {
  readonly _type: 'note'
  name: string
  data: string
}

export interface ITrack extends IRecord {
  readonly _type: 'track'
  artist: string
  title: string
}

export type Record = INote | ITrack // | IProject | ITrack | IPlaylist etc.
export type MutableRecordFields = Omit<Record, '_id' | '_type' | '_rev'>

export interface IChangeset {
  /**
   * replica storage revision
   */
  baseRev: number

  /**
   * new or updated records
   */
  records: Record[]

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
  records: Array<Record | string>

  /**
   * attachment or attachment id
   */
  attachments: Array<IAttachment | string>
}
