import { Without } from '~/utils'

export interface IDocument {
  _id: string
  _rev: number
  _type: string
  _createdTs: number
  _updatedTs: number
  _attachmentRefs: string[]
  _deleted?: boolean
}

export interface IAttachment {
  readonly _id: string
  readonly _rev: number
  readonly _createdTs: number
  readonly _type: string
  readonly _size: number
}

export type NewAttachment = Without<IAttachment, '_type' | '_size'>

export interface IChangeset<T extends IDocument> {
  /**
   * replica storage revision
   */
  readonly baseRev: number

  /**
   * new or updated documents
   */
  readonly documents: T[]

  /**
   * new or updated attachments
   */
  readonly attachments: NewAttachment[]
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
   * primary storage revision
   */
  readonly currentRev: number

  /**
   * documents with _rev > baseRev
   */
  readonly documents: T[]

  /**
   * attachments with _rev > baseRev
   */
  attachments: IAttachment[]
}
