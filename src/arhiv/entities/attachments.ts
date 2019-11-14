import { ArhivReplica } from '../types'
import { Attachment } from '../entities/attachment'

export class AttachmentsRepository {
  constructor(protected _replica: ArhivReplica) { }

  createAttachment(file: File): Promise<string> {
    return this._replica.saveAttachment(file)
  }

  async getAttachment(id: string): Promise<Attachment | undefined> {
    const attachment = await this._replica.getAttachment(id)
    if (!attachment) {
      return undefined
    }

    return new Attachment(this._replica, attachment)
  }
}
