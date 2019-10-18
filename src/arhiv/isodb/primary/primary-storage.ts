import {
  IDocument,
  IAttachment,
} from '../types'

export interface IPrimaryStorage<T extends IDocument> {
  getRev(): number
  setRev(rev: number): void

  getDocuments(): T[]
  getAttachments(): IAttachment[]

  getDocument(id: string): T | undefined
  getAttachment(id: string): IAttachment | undefined
  getAttachmentDataPath(id: string): string | undefined

  putDocument(document: T): void
  removeDocument(id: string): void

  addAttachment(attachment: IAttachment, attachmentPath: string): Promise<void>
  updateAttachment(attachment: IAttachment): void
  removeAttachmentData(id: string): void
}
