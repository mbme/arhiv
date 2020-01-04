import { ISO8601DateString } from '~/utils'

export interface IDocument {
  readonly _id: string
  readonly _rev: number
  readonly _type: string
  readonly _createdAt: ISO8601DateString
  readonly _updatedAt: ISO8601DateString
  readonly _attachmentRefs: readonly string[]
  readonly _deleted?: boolean
}

export interface IAttachment {
  readonly _id: string
  readonly _rev: number
  readonly _createdAt: ISO8601DateString
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
   * new attachments
   */
  readonly attachments: readonly IAttachment[]
}

export type ChangesetResponseStatus = 'accepted' | 'outdated'

export interface IChangesetResponse {
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
  readonly documents: readonly ArhivDocument[]

  /**
   * attachments with _rev > baseRev
   */
  attachments: readonly IAttachment[]
}
