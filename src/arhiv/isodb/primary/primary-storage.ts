import {
  IDocument,
  IAttachment,
} from '../types'

export interface IPrimaryStorage<T extends IDocument> {
  getRev(): number
  setRev(rev: number): void

  getDocuments(): T[]
  getDocument(id: string): T | undefined
  getDocumentHistory(id: string): T[] | undefined
  putDocument(document: T): void

  getAttachments(): IAttachment[]
  getAttachment(id: string): IAttachment | undefined
  getAttachmentDataPath(id: string): string | undefined
  addAttachment(attachment: IAttachment, attachmentPath: string): Promise<void>
  updateAttachment(attachment: IAttachment): void
  removeAttachmentData(id: string): void
}
