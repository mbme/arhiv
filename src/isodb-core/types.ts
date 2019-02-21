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
