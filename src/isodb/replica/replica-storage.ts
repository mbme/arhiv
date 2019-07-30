import { IDict } from '~/utils'
import {
  IDocument,
  IAttachment,
  NewAttachment,
} from '../types'

export type LocalAttachments = IDict<Blob>

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

  addLocalDocument(document: T): void
  addLocalAttachment(attachment: NewAttachment, blob?: File): void

  removeLocalDocument(id: string): void
  removeLocalAttachment(id: string): void

  getAttachmentUrl(id: string): string | undefined
  getLocalAttachmentsData(): LocalAttachments
  upgrade(rev: number, documents: T[], attachments: IAttachment[]): void
  clearLocalData(): void
}
