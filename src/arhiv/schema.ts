export const SCHEMA_VERSION = 1

export const ID_ALPHABET = '0123456789abcdefghijklmnopqrstuvwxyz'
export const ID_LENGTH = 15

export interface IAttachment {
  readonly id: string
  readonly rev: number
  readonly createdAt: string
  readonly mimeType: string
  readonly size: number
  readonly deleted: boolean
}

export interface IDocument<P extends object = {}> {
  readonly id: string
  readonly rev: number
  readonly type: string
  readonly createdAt: string
  readonly updatedAt: string
  readonly refs: readonly string[]
  readonly attachmentRefs: readonly string[]
  readonly deleted: boolean
  readonly props: P
}

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
  readonly documents: readonly IDocument[]

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
  readonly documents: readonly IDocument[]

  /**
   * attachments with _rev > baseRev
   */
  attachments: readonly IAttachment[]
}
