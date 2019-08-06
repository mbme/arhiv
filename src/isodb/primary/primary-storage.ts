import {
  IDocument,
  IAttachment,
} from '~/isodb/types'

export interface IPrimaryStorage<T extends IDocument> {
  getRev(): number
  setRev(rev: number): void

  getDocuments(): T[]
  getAttachments(): IAttachment[]

  getDocument(id: string): T | undefined
  getAttachment(id: string): IAttachment | undefined
  getAttachmentPath(id: string): string | undefined

  putDocument(document: T): void
  removeDocument(id: string): void

  addAttachment(attachment: IAttachment, attachmentPath: string): void
  updateAttachment(attachment: IAttachment): void
  removeAttachment(id: string): void
}
