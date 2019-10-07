import { IDict } from '~/utils'
import {
  IDocument,
  IAttachment,
  IChangesetResult,
  IChangeset,
} from '../types'

export type LocalAttachments = IDict<Blob>

export type ChangesetExchange<T extends IDocument> = (
  changeset: IChangeset<T>,
  blobs: LocalAttachments,
) => Promise<IChangesetResult<T>>

export interface IReplicaStorage<T extends IDocument> {
  getRev(): number

  getDocuments(): T[]
  getLocalDocuments(): T[]

  getAttachments(): IAttachment[]
  getLocalAttachments(): IAttachment[]

  getDocument(id: string): T | undefined
  getLocalDocument(id: string): T | undefined

  getAttachment(id: string): IAttachment | undefined
  getLocalAttachment(id: string): IAttachment | undefined
  getLocalAttachmentData(id: string): Blob | undefined

  addLocalDocument(document: T): void
  addLocalAttachment(attachment: IAttachment, blob: File): void

  removeLocalDocument(id: string): void
  removeLocalAttachment(id: string): void

  getChangeset(): [IChangeset<T>, LocalAttachments]
  upgrade(changesetResult: IChangesetResult<T>): void
  clearLocalData(): void
}
