import {
  IDocument,
  IAttachment,
} from '../types'

export interface IPrimaryStorageMutations<T extends IDocument> {
  setRev(newRev: number): void
  putDocument(document: T): Promise<void>
  addAttachment(attachment: IAttachment, attachmentPath: string): Promise<void>
  updateAttachment(attachment: IAttachment): Promise<void>
  removeAttachmentData(id: string): Promise<void>
}

export type StorageUpdater<T extends IDocument> = (mutations: IPrimaryStorageMutations<T>) => (void | Promise<void>)

export interface IPrimaryStorage<T extends IDocument> {
  getRev(): number

  getDocuments(): Promise<T[]>
  getDocument(id: string): Promise<T | undefined>
  getDocumentHistory(id: string): Promise<T[] | undefined>

  getAttachments(): Promise<IAttachment[]>
  getAttachment(id: string): Promise<IAttachment | undefined>
  getAttachmentDataPath(id: string): Promise<string | undefined>

  updateStorage(update: StorageUpdater<T>): Promise<void>
}
