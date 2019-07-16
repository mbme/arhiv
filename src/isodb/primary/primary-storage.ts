import {
  IDocument,
  IAttachment,
} from '~/isodb/types'

export interface IPrimaryStorage {
  getRev(): number
  setRev(rev: number): void

  getDocuments(): IDocument[]
  getAttachments(): IAttachment[]

  getDocument(id: string): IDocument | undefined
  getAttachment(id: string): IAttachment | undefined
  getAttachmentPath(id: string): string | undefined

  putDocument(document: IDocument): void
  removeDocument(id: string): void

  addAttachment(attachment: IAttachment, attachmentPath: string): void
  updateAttachment(attachment: IAttachment): void
  removeAttachment(id: string): void
}
