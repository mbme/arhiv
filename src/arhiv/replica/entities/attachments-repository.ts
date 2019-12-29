import { ReplicaDB } from '../db'
import { AttachmentManager } from './attachment-manager'

export class AttachmentsRepository {
  constructor(protected _db: ReplicaDB) { }

  createAttachment(file: File): Promise<string> {
    return this._db.saveAttachment(file)
  }

  async getAttachment(id: string): Promise<AttachmentManager | undefined> {
    const attachment = await this._db.getAttachment(id)
    if (!attachment) {
      return undefined
    }

    return new AttachmentManager(this._db, attachment)
  }
}
