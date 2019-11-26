import { ArhivDB } from '../db'
import { Attachment } from './attachment'

export class AttachmentsRepository {
  constructor(protected _db: ArhivDB) { }

  createAttachment(file: File): Promise<string> {
    return this._db.saveAttachment(file)
  }

  async getAttachment(id: string): Promise<Attachment | undefined> {
    const attachment = await this._db.getAttachment(id)
    if (!attachment) {
      return undefined
    }

    return new Attachment(this._db, attachment)
  }
}
