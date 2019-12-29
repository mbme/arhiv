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

export class MarkupString {
  constructor(
    public readonly value: string,
  ) { }
}

export interface INote extends IDocument {
  readonly _type: 'note'
  readonly name: string
  readonly data: MarkupString
}

export interface ITrack extends IDocument {
  readonly _type: 'track'
  readonly artist: string
  readonly title: string
}

export type ArhivDocument = INote | ITrack
export type ArhivDocumentType = ArhivDocument['_type']

export interface IChangeset {
  /**
   * replica storage revision
   */
  readonly baseRev: number

  /**
   * replica schema version
   */
  readonly schemaVersion: number

  /**
   * new or updated documents
   */
  readonly documents: readonly ArhivDocument[]

  /**
   * new or updated attachments
   */
  readonly attachments: readonly IAttachment[]
}

export type ChangesetResponseStatus = 'accepted' | 'outdated'

export interface IChangesetResponse<T extends IDocument> {
  readonly status: ChangesetResponseStatus

  /**
   * replica schema version
   */
  readonly schemaVersion: number

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
