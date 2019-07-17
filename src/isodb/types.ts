export interface IDocument {
  readonly _id: string
  readonly _rev: number
  readonly _type: string
  readonly _refs: string[]
  readonly _attachmentRefs: string[]
  readonly _deleted?: boolean
}

export enum DocumentType {
  Note = 'note',
  Track = 'track',
}

export interface IAttachment {
  readonly _id: string
  readonly _rev: number
  readonly _type: string
  readonly _size: number
}

export interface IChangeset {
  /**
   * replica storage revision
   */
  readonly baseRev: number

  /**
   * new or updated documents
   */
  readonly documents: IDocument[]

  /**
   * new or updated attachments
   */
  readonly attachments: IAttachment[]
}

export interface IChangesetResult {
  /**
   * if changeset was successfully applied
   */
  readonly success: boolean

  /**
   * replica storage revision
   */
  readonly baseRev: number

  /**
   * primary storage revision
   */
  readonly currentRev: number

  /**
   * documents with _rev > baseRev
   */
  readonly documents: IDocument[]

  /**
   * attachments with _rev > baseRev
   */
  attachments: IAttachment[]
}
