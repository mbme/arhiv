import {
  IDocument,
  IAttachment,
} from '../types'

export interface IPrimaryStorageMutations<T extends IDocument> {
  setRev(newRev: number): void
  putDocument(document: T): void
  addAttachment(attachment: IAttachment, attachmentPath: string): Promise<void>
  updateAttachment(attachment: IAttachment): void
  removeAttachmentData(id: string): void
}

export type StorageUpdater<T extends IDocument> = (mutations: IPrimaryStorageMutations<T>) => (void | Promise<void>)

export interface IPrimaryStorage<T extends IDocument> {
  getRev(): number

  getDocuments(): T[]
  getDocument(id: string): T | undefined
  getDocumentHistory(id: string): T[] | undefined

  getAttachments(): IAttachment[]
  getAttachment(id: string): IAttachment | undefined
  getAttachmentDataPath(id: string): string | undefined

  updateStorage(update: StorageUpdater<T>): Promise<void>
}
