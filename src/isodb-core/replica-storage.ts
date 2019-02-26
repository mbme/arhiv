import {
  IRecord,
  IAttachment,
} from './types'

export interface IReplicaStorage {
  getRev(): number

  getRecords(): IRecord[]
  getLocalRecords(): IRecord[]

  getAttachments(): IAttachment[]
  getLocalAttachments(): IAttachment[]

  getRecord(id: string): IRecord | undefined
  getLocalRecord(id: string): IRecord | undefined

  getAttachment(id: string): IAttachment | undefined
  getLocalAttachment(id: string): IAttachment | undefined

  addLocalRecord(record: IRecord): void
  addLocalAttachment(attachment: IAttachment, blob?: File): void

  removeLocalRecord(id: string): void
  removeLocalAttachment(id: string): void

  getAttachmentUrl(id: string): string | undefined
  getLocalAttachmentsData(): { [id: string]: Blob }
  upgrade(rev: number, records: IRecord[], attachments: IAttachment[]): void
  clearLocalData(): void
}
