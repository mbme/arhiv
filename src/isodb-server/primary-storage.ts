import {
  IRecord,
  IAttachment,
} from '~/isodb-core/types'

export interface IPrimaryStorage {
  getRev(): number
  setRev(rev: number): void

  getRecords(): IRecord[]
  getAttachments(): IAttachment[]

  getRecord(id: string): IRecord | undefined
  getAttachment(id: string): IAttachment | undefined
  getAttachmentPath(id: string): string | undefined

  putRecord(updatedRecord: IRecord): void
  addAttachment(attachment: IAttachment, attachmentPath: string): void
  updateAttachment(attachment: IAttachment): void

  removeRecord(id: string): void
  removeAttachment(id: string): void
}
