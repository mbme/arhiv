export interface IDocument {
  readonly _id: string
  readonly _rev: number
  readonly _type: string
  readonly _createdTs: number
  readonly _updatedTs: number
  readonly _attachmentRefs: readonly string[]
  readonly _deleted?: boolean
}

export interface IAttachment {
  readonly _id: string
  readonly _rev: number
  readonly _createdTs: number
  readonly _mimeType: string
  readonly _size: number
  readonly _deleted?: boolean
}

export interface IChangeset<T extends IDocument> {
  /**
   * replica storage revision
   */
  readonly baseRev: number

  /**
   * new or updated documents
   */
  readonly documents: readonly T[]

  /**
   * new or updated attachments
   */
  readonly attachments: readonly IAttachment[]
}

export interface IChangesetResult<T extends IDocument> {
  /**
   * if changeset was successfully applied
   */
  readonly success: boolean

  /**
   * replica storage revision
   */
  readonly baseRev: number

  /**
   * server storage revision
   */
  readonly currentRev: number

  /**
   * documents with _rev > baseRev
   */
  readonly documents: readonly T[]

  /**
   * attachments with _rev > baseRev
   */
  attachments: readonly IAttachment[]
}

export enum DocumentType {
  Note = 'note',
  Track = 'track',
}

export interface INote extends IDocument {
  readonly _type: DocumentType.Note
  readonly name: string
  readonly data: string
}

export interface ITrack extends IDocument {
  readonly _type: DocumentType.Track
  readonly artist: string
  readonly title: string
}

export type Record = INote | ITrack
