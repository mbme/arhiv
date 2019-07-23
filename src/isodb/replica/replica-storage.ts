import { IDict } from '~/utils'
import {
  IDocument,
  IAttachment,
  NewAttachment,
} from '../types'

export type LocalAttachments = IDict<Blob>

export interface IReplicaStorage {
  getRev(): number

  getDocuments(): IDocument[]
  getLocalDocuments(): IDocument[]

  getAttachments(): IAttachment[]
  getLocalAttachments(): IAttachment[]

  getDocument(id: string): IDocument | undefined
  getLocalDocument(id: string): IDocument | undefined

  getAttachment(id: string): IAttachment | undefined
  getLocalAttachment(id: string): IAttachment | undefined

  addLocalDocument(document: IDocument): void
  addLocalAttachment(attachment: NewAttachment, blob?: File): void

  removeLocalDocument(id: string): void
  removeLocalAttachment(id: string): void

  getAttachmentUrl(id: string): string | undefined
  getLocalAttachmentsData(): LocalAttachments
  upgrade(rev: number, documents: IDocument[], attachments: IAttachment[]): void
  clearLocalData(): void
}
