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

  getDocuments(): Promise<T[]>
  getLocalDocuments(): Promise<T[]>

  getAttachments(): Promise<IAttachment[]>
  getLocalAttachments(): Promise<IAttachment[]>

  getDocument(id: string): Promise<T | undefined>
  getLocalDocument(id: string): Promise<T | undefined>

  getAttachment(id: string): Promise<IAttachment | undefined>
  getLocalAttachment(id: string): Promise<IAttachment | undefined>
  getLocalAttachmentData(id: string): Promise<Blob | undefined>

  addLocalDocument(document: T): Promise<void>
  addLocalAttachment(attachment: IAttachment, blob: File): Promise<void>

  removeLocalDocument(id: string): Promise<void>
  removeLocalAttachment(id: string): Promise<void>

  upgrade(changesetResult: IChangesetResult<T>): Promise<void>
}
